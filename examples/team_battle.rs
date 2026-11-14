//! Team battle: red droids against cyan droids in the combat arena, `data/arena/combat_map.glb`.
//!
//! Five a side, all played by the AI, each one of the game's own droids (see
//! `maze::player::avatar`): red in the hostile droid's colours, `data/droid_hostile.glb`, and cyan
//! in the full deform droid's, `data/droid_full_deform.glb`, with every one of the game's droid's
//! animations: walking with no enemy about, jogging to one, running for cover and sprinting when
//! falling back hurt - skidding as they stop and turn, and standing, stepping round to face another
//! way rather than turning on their heels - lying, shuffling round on their elbows - their body
//! following where they look - strafing and crouch-strafing while they face a target, a jump now
//! and then as they break for cover, the gun drawn, raised, aimed up,
//! down and to the side at the target and fired - a glowing bolt from its muzzle, the game's own
//! (see `maze::player::pistol`), green from a pistol and blue from a rifle, which does its harm
//! when it gets there - and holstered once the fighting is over. Shot
//! down, a droid falls limp, knocked back by the shot, and may lose what it was hit in; it comes
//! back as a new droid. They cross the arena along the walk grid and fight from cover, and take in
//! what is about them with hydroxus-ai's perception, as the game's sentries do: they see in a
//! cone in front of them and only near out of the corner of their eye, and become aware of an
//! enemy gradually - at once close in front, slowly far off, crouched or to the side, quicker
//! once in the fight - fighting only those they have spotted; a shot carries along the floor,
//! round corners and not through walls, to those that make for about where it was; and a droid
//! shot at by someone it cannot see, after a few bolts, fires blind back where they came from.
//! In cover, a crouched droid behind a low block is out
//! of the line of fire, and stands up to shoot over it; behind a wall or a pillar too tall to shoot
//! over, it stands with its back to it, as the player does in cover, and leans out round the side
//! of it to shoot. Hurt, a droid
//! falls back to cover further from the enemy. The dead come back at their own end of the arena
//! a few seconds later, and the first team to reach the kill limit wins the match.
//!
//! Each droid picks its own gun each time it comes back (see `TeamBattle::pick_gun`): one holding
//! the line mostly the rifle, for its reach, and one going round the side mostly the pistol, each
//! after its own liking. A rifle fires three-round bursts, or - for one in ten - long strings on
//! full auto, out of a magazine that recharges once it is empty, as the game's droids' do; its
//! shots lose little with range, but each does less harm than a pistol's.
//!
//! Each side fights as a team: it picks its targets - whoever is shooting at it, hurt, or out in
//! the open before whoever is merely nearest - takes cover from every enemy it knows of, not
//! just one, keeps out of sight of them as it moves and spreads out, sends some of its droids
//! round the side, leaves cover only while a teammate is firing, and fires at where an enemy
//! was just seen to keep their head down; a droid shot at like that stays down a moment.
//! `BATTLE_OLD=red` (or `cyan`, or `both`) has that side fight as they did before all that, one
//! droid at a time, to see the difference - seeing, hearing and firing blind as the other side
//! does: each match ends by saying which side won.
//!
//! `BATTLE_BENCH=<seconds>` benchmarks it: with vsync off, it times each part of every frame of
//! play - the droids thinking, moving, shown, and their bolts - and the frame as a whole, for that
//! many seconds of play, then says how long each took and quits. `BATTLE_SEED=<n>` fights the same
//! battle each time, to compare one run with another.
//!
//! Run with `cargo run --example team_battle`. `BATTLE_MAP=<path>` fights in another arena, such
//! as the capture-the-flag map, `data/arena/ctf_map.glb`: the teams start at its two ends along x.
//!
//! The camera is free: WASD to fly, Q and E down and up, Shift to go faster, the right mouse
//! button held to look round. Tab follows the next droid, V sees through its eyes or from behind
//! it again, F lets it go, Space pauses, R starts a new match, Escape quits.

use fyrox::{
    core::{
        algebra::{Point3, UnitQuaternion, Vector2, Vector3},
        color::Color,
        log::Log,
        math::aabb::AxisAlignedBoundingBox,
        pool::Handle,
        reflect::prelude::*,
        sstorage::ImmutableString,
        visitor::prelude::*,
    },
    engine::{executor::Executor, GraphicsContextParams},
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    graph::SceneGraph,
    gui::{
        brush::Brush,
        screen::ScreenBuilder,
        text::{Text, TextBuilder, TextMessage},
        widget::WidgetBuilder,
        HorizontalAlignment, Thickness, UserInterface,
    },
    keyboard::{KeyCode, PhysicalKey},
    material::{Material, MaterialProperty, MaterialResource, MaterialResourceBinding},
    plugin::{error::GameResult, Plugin, PluginContext},
    resource::model::{Model, ModelResource, ModelResourceExtension},
    scene::{
        base::BaseBuilder,
        camera::CameraBuilder,
        collider::{Collider, ColliderBuilder, ColliderShape, GeometrySource},
        graph::{physics::RayCastOptions, Graph},
        light::{
            directional::{DirectionalLight, DirectionalLightBuilder},
            point::{PointLight, PointLightBuilder},
            spot::SpotLight,
            BaseLightBuilder,
        },
        mesh::Mesh,
        node::Node,
        pivot::PivotBuilder,
        rigidbody::{RigidBody, RigidBodyBuilder, RigidBodyType},
        transform::TransformBuilder,
        EnvironmentLightingSource, Scene,
    },
    window::WindowAttributes,
};
use hydroxus_ai::{
    navigation::{navmesh::NavMesh, walk},
    perception::{bolts, detection::Detection},
    prelude::*,
    tactics::{
        blind::{self, BlindFire},
        roles::{self, Chaser, Part},
        team::{self, Exposure, Intel, Stuck},
        vantage,
    },
};
use maze::{
    sight_fog, AI,
    dismember,
    player::{
        avatar::{Avatar, Going, Wall},
        pistol::{self, Bolts, FireMode, BOLT_SPEED},
        posture::{Gait, Posture},
        SHOT_NOISE,
    },
    ragdoll::{self, Ragdoll},
};

/// Where a battle is fought, and how it looks and how far the droids see there.
#[derive(Debug, Clone, Copy)]
pub struct Arena {
    /// The window's title.
    pub title: &'static str,
    pub map: &'static str,
    /// The light from all round, the fog over it if any, and a sun - its strength - if it is out
    /// under the sky.
    pub ambient: Color,
    pub fog: Option<fyrox::scene::Fog>,
    pub sun: Option<f32>,
    /// How far a droid sees, in meters, at most.
    pub sight: f32,
    /// Where the spectator camera starts, and how far down it looks, in degrees.
    pub camera: [f32; 3],
    pub pitch: f32,
    /// How many teams fight: two, from either end along x, or up to four, from its four sides - red,
    /// cyan, green and yellow, in that order (see [`Team`]).
    pub teams: usize,
    /// How many droids each team has.
    pub per_team: usize,
    /// Whether anything casts shadows: not in a city traced in light at night, as the game has it -
    /// the map's own lamps would draw every droid near them six times over, a shadow map a side.
    pub shadows: bool,
}

/// The combat arena, under its roof, lit by its strips.
pub const COMBAT: Arena = Arena {
    title: "Team battle",
    map: "data/arena/combat_map.glb",
    ambient: Color::opaque(70, 72, 80),
    fog: None,
    sun: None,
    sight: 45.0,
    // Inside the arena, under its ceiling - the roof hides everything from above - in the middle
    // of one long side, looking across it with a team to either side.
    camera: [0.0, 4.6, -29.0],
    pitch: 22.0,
    teams: 2,
    per_team: 5,
    shadows: true,
};

/// Nexus: the battle royale's city of light round a soaring spire, on a platform in the void
/// (`data/arena/br_nexus.py`), at night - its skyway ring 7 m up, its ziggurats and its domes for
/// high ground and cover. Lit brighter than the game lights it - from all round, and a moon - since
/// the battle does not light its glowing edges as the game does, and its droids would be lost
/// against the black deck.
#[allow(dead_code)] // not by every example that pulls this in
pub const NEXUS: Arena = Arena {
    title: "Team battle - Nexus, four teams of forty",
    map: "data/arena/br_nexus.glb",
    ambient: Color::opaque(96, 100, 120),
    fog: None,
    sun: Some(0.7),
    sight: 45.0,
    // Over one side of the platform, near enough to make the droids out, looking in across it
    // to the spire, with teams coming from either side and from ahead and behind.
    camera: [0.0, 16.0, -70.0],
    pitch: 14.0,
    teams: 4,
    per_team: 40,
    shadows: false,
};

/// The forest: abandoned buildings among the trees, in the Foggy Forest's fog
/// (`data/arena/tb_forest.py`). The droids see as far as in the combat arena, but the fog takes
/// that down: they make out a droid only as well as it shows through it, half as well 20 m off,
/// and none past about 45 m (see [`Battle::seeing`]). They hear as well as ever.
#[allow(dead_code)] // not by every example that pulls this in
pub const FOREST: Arena = Arena {
    title: "Team battle - forest",
    map: "data/arena/tb_forest.glb",
    // The game's on a foggy map: an overcast day, lit from all round by the grey sky.
    ambient: Color::opaque(118, 124, 120),
    fog: Some(maze::FOG),
    sun: Some(0.6),
    sight: 45.0,
    // Over one long side, looking across with a team to either side.
    camera: [0.0, 14.0, -40.0],
    pitch: 28.0,
    teams: 2,
    per_team: 5,
    shadows: true,
};

impl Default for Arena {
    fn default() -> Self {
        COMBAT
    }
}
/// What a map's leaves are named, starting: they are not solid (see `place_level`).
const FOLIAGE: &str = "Foliage";
/// How often a droid breaking for cover jumps as it goes, out of 1; how fast it leaves the
/// ground, in meters per second, for a short and a high jump; and how fast it falls.
const JUMP_CHANCE: f32 = 0.25;
const JUMP_SPEED: (f32, f32) = (3.2, 4.4);
const GRAVITY: f32 = 9.81;
/// How far off the droid it is to hit a pistol can point, in radians, and the bolt still go onto
/// them; and pointed further off, how near their chest its way has to pass to hit them, in meters.
const AIM_SNAP: f32 = 0.1;
/// How much of how far over the barrel pointed at a shot each droid takes off what it asks for
/// next time, and the most it takes off either way, in radians.
const AIM_LEARNING: f32 = 0.5;
const AIM_BIAS_MOST: f32 = 0.6;
const HIT_RADIUS: f32 = 0.4;
/// How near its gun the arena may be in the way of a shot, in meters, for a droid not to fire it:
/// it would only be shooting into its own cover, seeing over or round what its gun cannot.
const OWN_COVER: f32 = 2.0;
/// How long, in seconds, a droid keeps its pistol out after it last had a target.
const HOLSTER_AFTER: f32 = 6.0;
/// The droids each team is made from: the same droid, in red, cyan, green and yellow.
const MODELS: [&str; TEAMS] =
    ["data/droid_hostile.glb", "data/droid_full_deform.glb", "data/droid_explorer.glb", "data/droid_yellow.glb"];
/// How many kills win a match, at least, and for each droid a team has.
const KILL_LIMIT: u32 = 30;
const KILLS_PER_DROID: u32 = 3;

/// Walk grid spacing: the game's survey's (see `maze::survey`).
const CELL: f32 = 0.5;

/// How fast a droid goes advancing and running for cover, in meters per second, if its model has
/// no jog or run to go by; with them it goes at their own pace, so its feet keep to the floor.
const WALK_SPEED: f32 = 1.8;
const RUN_SPEED: f32 = 3.0;
const TURN_SPEED: f32 = 8.0;
/// How far off a target a droid's body can be turned, in radians, for it to fire at them: as far
/// as its aim reaches round. Further, it steps round first.
const BODY_AIM: f32 = 0.9;
/// Lying flat, as far: its aim lying reaches round only a little way.
const PRONE_BODY_AIM: f32 = 0.45;

/// Eye and chest heights, standing and crouched. Low cover is about 1.1 m tall, so a crouched
/// droid behind it is hidden and a standing one can shoot over it.
const EYES_STANDING: f32 = 1.55;
const EYES_CROUCHED: f32 = 0.8;
const CHEST_STANDING: f32 = 1.2;
const CHEST_CROUCHED: f32 = 0.65;
/// Lying prone, on its elbows: the game's prone crawl has its eyes about here (see
/// `maze::player::posture`). Behind cover too low to crouch behind - a fallen trunk - it lies down.
const EYES_PRONE: f32 = 0.3;
const CHEST_PRONE: f32 = 0.25;
/// How many of the droids caught in the open with no cover to be had lie down to fight, out of
/// 1, rather than stand; how much less likely a shot at one lying down is to hit; and how fast one
/// crawls, if its model has no crawl to go by.
const PRONE_CHANCE: f32 = 0.6;
const PRONE_HARDER: f32 = 0.6;
const CRAWL_SPEED: f32 = 0.6;

const HEALTH: f32 = 100.0;
const DAMAGE: (f32, f32) = (9.0, 16.0);
const FIRE_INTERVAL: f32 = 0.16;
/// The rifle: how long between the shots of a burst, as fast as it fires; how many shots a
/// burst is, three-round and on full auto; how long the pause after one, as many times
/// `FIRE_INTERVAL`; how many in ten droids set theirs to full auto; how much less harm each shot
/// does than a pistol's; and how far it carries before a shot is no likelier to hit than a
/// pistol's at its last, against the pistol's.
const RIFLE_GAP: f32 = 0.1;
const BURST_SHOTS: u8 = 3;
const AUTO_BURST: u8 = 8;
const BURST_PAUSE: (f32, f32) = (2.0, 3.5);
const AUTO_CHANCE: f32 = 0.1;
const RIFLE_HARM: f32 = 0.75;
const RIFLE_REACH: f32 = 110.0;
const PISTOL_REACH: f32 = 70.0;
const RESPAWN_TIME: f32 = 5.0;
/// Hurt this badly, a droid falls back instead of pushing on.
const RETREAT_HEALTH: f32 = 35.0;
/// How far a droid will go to reach cover, in meters of walking.
const COVER_REACH: f32 = 15.0;
/// How often, in seconds, a droid looks about it, and how it sees: in a cone in front of it, and
/// out of the corner of its eye near by (hydroxus-ai's [`Sight`]), as far as its arena says.
const LOOK_EVERY: f32 = 0.15;
/// How long a droid shot at from above goes on looking up, in seconds; and how high over a
/// droid's feet its shots leave its gun, in meters, for where it stands.
const LOOKING_UP: f32 = 20.0;
/// How often at most a droid works out its route afresh to a goal that drifts, in seconds, unless
/// it has drifted this far, in meters (see [`Battle::walk_to`]).
const REPLAN_EVERY: f32 = 0.5;
const REPLAN_FAR: f32 = 6.0;
/// How many routes droids that have one to follow may work out afresh in a frame: the rest keep
/// theirs till a later one. A droid with none always gets one.
const ROUTES_A_FRAME: usize = 12;
/// The most of a frame's time, in seconds, the droids' looking about and their team's survey of
/// the ground take in at once.
const MOST_THINKING_DT: f32 = 0.05;
/// How many of a team at most hold the high ground; and how far from the middle of the arena, in
/// meters, and how far a walk, it may be (see [`Battle::high_ground`]).
const MOST_HIGH: usize = 1;
const HIGH_NEAR: f32 = 60.0;
const HIGH_REACH: f32 = 160.0;
const HELD_AT: f32 = 1.3;
fn sight(range: f32) -> Sight {
    Sight { range, cone: 70f32.to_radians(), ..Sight::DEFAULT }
}
/// How long a droid heads for shooting it heard, in seconds, and how near it has to get to it to
/// have got there, in meters; and how far off where it was it can be wrong, heard from as far as
/// the shot carries.
const HEARD_FOR: f32 = 15.0;
const HEARD_REACHED: f32 = 3.0;
const HEARD_WRONG: f32 = 6.0;
/// How the teams fight as teams - what they know of the enemy, which ground it sees, flanking,
/// cover and covering fire - is hydroxus-ai's (`hydroxus_ai::tactics::team`).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Team {
    Red,
    Cyan,
    Green,
    Yellow,
}

/// As many teams as there can be.
const TEAMS: usize = 4;

impl Team {
    const ALL: [Team; TEAMS] = [Team::Red, Team::Cyan, Team::Green, Team::Yellow];

    fn color(self) -> Color {
        match self {
            Team::Red => Color::opaque(225, 40, 40),
            Team::Cyan => Color::opaque(0, 225, 235),
            Team::Green => Color::opaque(40, 225, 60),
            Team::Yellow => Color::opaque(240, 220, 20),
        }
    }

    fn index(self) -> usize {
        self as usize
    }

    fn name(self) -> &'static str {
        match self {
            Team::Red => "RED",
            Team::Cyan => "CYAN",
            Team::Green => "GREEN",
            Team::Yellow => "YELLOW",
        }
    }
}

/// The score of the first `teams` teams: `RED 3 : 4 CYAN` for two, `RED 3  CYAN 4  GREEN 1 ...`
/// for more.
fn score_line(score: &[u32; TEAMS], teams: usize) -> String {
    if teams <= 2 {
        format!("RED {} : {} CYAN", score[0], score[1])
    } else {
        Team::ALL.iter().take(teams).map(|t| format!("{} {}", t.name(), score[t.index()])).collect::<Vec<_>>().join("  ")
    }
}

/// A place next to an obstacle where a crouched droid is hidden from some directions.
#[derive(Debug, Clone)]
struct CoverSpot {
    position: Vector3<f32>,
    /// The directions (as unit vectors along the ground) that an obstacle shields it from.
    shields: Vec<Vector3<f32>>,
}

/// What a droid is doing.
#[derive(Debug, Clone, PartialEq)]
enum Task {
    /// Heading for the enemy: where one was last seen, or into their half of the arena.
    Advance,
    /// Running to cover, to shoot from `peek` - the same spot standing, or a step to the side.
    ToCover { spot: usize, peek: Vector3<f32> },
    /// Crouched in cover.
    Hidden { spot: usize, peek: Vector3<f32>, left: f32 },
    /// Up out of cover and shooting.
    Peeking { spot: usize, peek: Vector3<f32>, left: f32 },
    /// No cover to be had: standing and fighting where it is.
    Standing { left: f32 },
}

impl Task {
    fn label(&self) -> &'static str {
        match self {
            Task::Advance => "advancing",
            Task::ToCover { .. } => "running to cover",
            Task::Hidden { .. } => "in cover",
            Task::Peeking { .. } => "firing from cover",
            Task::Standing { .. } => "fighting in the open",
        }
    }
}

#[derive(Debug)]
struct Droid {
    team: Team,
    root: Handle<Node>,
    /// The droid it is shown as.
    avatar: Avatar,
    position: Vector3<f32>,
    /// Where it was last frame, for how fast it is going, and how its feet are going there.
    was_at: Vector3<f32>,
    gait: Gait,
    /// Whether it is falling back, hurt, which it does at a sprint.
    retreating: bool,
    /// Whether it pulled the trigger this frame, and the shot it is waiting to fire.
    fired: bool,
    /// The gun it picked this life: a rifle, or the pistol; how the rifle fires; how many shots
    /// of the burst under way are left; and how many are left in its magazine.
    rifle: bool,
    fire_mode: FireMode,
    burst_left: u8,
    rounds: u32,
    /// How much it likes the rifle, from 0 to 1, all its lives: it picks its gun by that and what
    /// it is to do.
    likes_rifle: f32,
    pending: Option<Shot>,
    /// How far its pistol's barrel has been pointing above and to the left of where it asked it
    /// to aim, in radians, as its last shots have shown: it asks that much the other way.
    aim_bias: (f32, f32),
    /// Where the chest of the target it has is, if one.
    target_at: Option<Vector3<f32>>,
    /// How long since it last had a target, in seconds: it holsters its pistol a while after.
    calm_for: f32,
    /// How high off the floor it is and how fast it is going up, while in a jump; whether it
    /// has just jumped, and whether a short jump.
    air: Option<(f32, f32)>,
    jumped: bool,
    low_jump: bool,
    /// Its body gone limp, while it lies where it fell; and whether its body is spent, to be made
    /// again when it comes back.
    ragdoll: Option<Ragdoll>,
    fallen: bool,
    heading: f32,
    /// The way its root was last turned, which its body steps round after; none for a body just
    /// made.
    shown: Option<f32>,
    health: f32,
    /// Seconds until it comes back, while dead.
    dead_for: Option<f32>,
    route: Vec<Vector3<f32>>,
    /// Where the route ends, so it is only planned again when that changes.
    going_to: Option<Vector3<f32>>,
    task: Task,
    crouched: bool,
    /// Whether it lies prone: fighting in the open, or behind cover too low to crouch behind
    /// (`cover_prone`, as the cover it went for is).
    prone: bool,
    cover_prone: bool,
    /// The enemy it is fighting, while it can see them.
    target: Option<usize>,
    /// Where it heads when there is nothing better to do.
    objective: Option<Vector3<f32>>,
    /// How long since it last worked out its route, in seconds (see [`Battle::walk_to`]).
    planned: f32,
    /// The high ground it holds this life, holding the line with the rifle (see
    /// [`Battle::high_ground`]): it makes for it, and holds it rather than going after an enemy
    /// it only knows of.
    high: Option<Vector3<f32>>,
    reload: f32,
    look_timer: f32,
    /// A moment of red after a hit.
    hurt_flash: f32,
    kills: u32,
    deaths: u32,
    /// How much longer it keeps its head down, shot at, in seconds.
    suppressed: f32,
    /// How much longer it looks up, shot at from above, in seconds: it sees those up above it
    /// only so, or the one it is already fighting (see [`Sight::overhead`]).
    looking_up: f32,
    /// Which enemy it fires at to keep their head down, and where, while it does.
    suppress_at: Option<(usize, Vector3<f32>)>,
    /// Whether it goes round the side, and which side: 1 or -1.
    flank: Option<f32>,
    /// Its part in going after an enemy on the move with the rest of its team, advancing (see
    /// hydroxus-ai's [`roles`]): running at them, or cutting them off ahead.
    part: Option<Part>,
    /// The enemy it is going after with the others, as its team knows it, while it has a part.
    after: Option<Chaser>,
    /// Where it was, and how long ago, to tell if it is stuck.
    stuck_from: Stuck,
    /// How aware it is of each droid, by its index: it fights only those it has spotted (see
    /// hydroxus-ai's [`Detection`]).
    aware: Vec<Detection>,
    /// Where it last heard shooting it could not see, and how long ago.
    heard: Option<(Vector3<f32>, f32)>,
    /// Bolts from someone it cannot see, and its firing blind back where they came from; and
    /// whose they were.
    blind: BlindFire,
    blind_at: Option<usize>,
}

impl Droid {
    fn alive(&self) -> bool {
        self.dead_for.is_none()
    }

    fn eyes(&self) -> Vector3<f32> {
        self.position + Vector3::y() * self.eye_height()
    }

    fn chest(&self) -> Vector3<f32> {
        self.position + Vector3::y() * if self.prone {
            CHEST_PRONE
        } else if self.crouched {
            CHEST_CROUCHED
        } else {
            CHEST_STANDING
        }
    }

    fn eye_height(&self) -> f32 {
        if self.prone {
            EYES_PRONE
        } else if self.crouched {
            EYES_CROUCHED
        } else {
            EYES_STANDING
        }
    }

    fn posture(&self) -> Posture {
        if self.prone {
            Posture::Crawling
        } else if self.crouched {
            Posture::Crouching
        } else {
            Posture::Standing
        }
    }

    /// Behind tall cover - a wall or a pillar, which it shoots round rather than over - which side
    /// the wall is on and which way along it the droid faces, toward the side it leans out from:
    /// with `spot` and `peek` of its task, and the cover spots.
    fn tall_cover(spot: usize, peek: Vector3<f32>, covers: &[CoverSpot]) -> Option<(Wall, f32)> {
        let at = covers.get(spot)?.position;
        let along = flat(peek - at);
        // Low cover is shot over from the spot itself.
        if along.norm() < 0.3 {
            return None;
        }
        let along = along.normalize();
        let left = -right_of(along);
        let shielded = |way: Vector3<f32>| {
            covers[spot].shields.iter().map(|s| s.dot(&way)).fold(f32::MIN, f32::max)
        };
        let wall = if shielded(left) > shielded(-left) { Wall::Left } else { Wall::Right };
        Some((wall, heading_of(along)?))
    }

    fn cover_spot(&self) -> Option<usize> {
        match self.task {
            Task::ToCover { spot, .. } | Task::Hidden { spot, .. } | Task::Peeking { spot, .. } => {
                Some(spot)
            }
            _ => None,
        }
    }
}

/// A shot a droid has decided on, waiting for its pistol to fire: which droid it is at, where it
/// is going - onto them, into the cover in the way, or wide - whether it hits them, and for how
/// much; and how long it has waited.
#[derive(Debug, Clone, Copy)]
struct Shot {
    target: usize,
    end: Vector3<f32>,
    hits: bool,
    damage: f32,
    waited: f32,
}

/// A bolt on its way to the droid it hits: who fired it, at whom, for how much, from where to
/// where, and how long till it gets there.
#[derive(Debug, Clone, Copy)]
struct Arrival {
    shooter: usize,
    target: usize,
    damage: f32,
    from: Vector3<f32>,
    to: Vector3<f32>,
    left: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Phase {
    #[default]
    Loading,
    /// The level is in the scene, waiting for its collider to reach the physics world.
    Settling(u32),
    Playing,
    /// The match is over; a new one starts when the time runs out.
    Won(Team, f32),
}

/// A benchmark of the battle (see `BATTLE_BENCH` at the top): each frame of play, how long its
/// parts took, in milliseconds - thinking, moving, showing, bolts - and the whole frame.
#[derive(Debug, Default)]
struct Bench {
    /// How long to play for, in seconds of play, and how long it has played.
    length: f32,
    played: f32,
    parts: Vec<[f32; 4]>,
    frames: Vec<f32>,
    /// When the last frame of play started, and when play started.
    last: Option<std::time::Instant>,
    started: Option<std::time::Instant>,
    /// The score as it started; how far the droids walked between them, in meters;
    /// how many frames any had a target; and how many bolts were fired.
    score: [u32; TEAMS],
    walked: f32,
    targeting: usize,
    fired: usize,
    /// The engine's own count of each frame: the time it took to draw, in milliseconds, its
    /// draw calls and its triangles.
    render: Vec<f32>,
    draws: Vec<f32>,
    triangles: Vec<f32>,
    /// The lights it drew, and the shadow maps: point, spot and cascaded, and traced shadows.
    lights: Vec<f32>,
    shadow_maps: Vec<f32>,
}

impl Bench {
    const PARTS: [&'static str; 4] = ["think", "move", "show", "bolts"];

    /// The benchmark `BATTLE_BENCH` asks for, if it asks for one.
    fn wanted() -> Option<Self> {
        let length: f32 = std::env::var("BATTLE_BENCH").ok()?.parse().ok()?;
        Some(Self { length: length.max(1.0), ..Self::default() })
    }

    /// How long `part` took, from `since`, in milliseconds.
    fn ms(since: std::time::Instant) -> f32 {
        since.elapsed().as_secs_f32() * 1000.0
    }

    /// What it found: for each part and the whole frame, its mean, middle, 95th and 99th
    /// percentile and worst, in milliseconds.
    fn report(&self, title: &str, droids: usize, score: [u32; TEAMS], teams: usize) -> String {
        let stats = |mut v: Vec<f32>| {
            v.sort_by(f32::total_cmp);
            let at = |q: f32| v[((v.len() - 1) as f32 * q).round() as usize];
            let mean = v.iter().sum::<f32>() / v.len() as f32;
            format!("{mean:8.3} {:8.3} {:8.3} {:8.3} {:8.3}", at(0.5), at(0.95), at(0.99), at(1.0))
        };
        let wall = self.started.map_or(0.0, |s| s.elapsed().as_secs_f32());
        let mut out = format!(
            "Team battle benchmark: {title}, {droids} droids, {:.0} s of play, {} frames ({:.0} fps), {}\n  \
             {} shot down, {} bolts fired, {:.0} m walked, a target in {:.0}% of frames\n",
            self.played, self.frames.len(), self.frames.len() as f32 / wall.max(1.0e-3), score_line(&score, teams),
            score.iter().sum::<u32>() - self.score.iter().sum::<u32>(), self.fired, self.walked, 100.0 * self.targeting as f32 / self.parts.len().max(1) as f32
        );
        out += "  ms per frame     mean      p50      p95      p99      max\n";
        for (k, name) in Self::PARTS.iter().enumerate() {
            out += &format!("  {name:<12} {}\n", stats(self.parts.iter().map(|p| p[k]).collect()));
        }
        let ai: Vec<f32> = self.parts.iter().map(|p| p[0] + p[1]).collect();
        out += &format!("  {:<12} {}\n", "think+move", stats(ai));
        out += &format!("  {:<12} {}\n", "whole frame", stats(self.frames.clone()));
        if !self.render.is_empty() {
            out += &format!("  {:<12} {}\n", "engine draw", stats(self.render.clone()));
            out += &format!("  {:<12} {}\n", "draw calls", stats(self.draws.clone()));
            out += &format!("  {:<12} {}\n", "kilotris", stats(self.triangles.clone()));
            out += &format!("  {:<12} {}\n", "lights", stats(self.lights.clone()));
            out += &format!("  {:<12} {}", "shadow maps", stats(self.shadow_maps.clone()));
        }
        out
    }
}

#[derive(Debug, Default)]
struct Battle {
    /// The benchmark being run, if one is (see `BATTLE_BENCH`).
    bench: Option<Bench>,
    arena: Arena,
    scene: Handle<Scene>,
    map: Option<ModelResource>,
    /// The droids' models, red's and cyan's.
    models: [Option<ModelResource>; TEAMS],
    level: Handle<Collider>,
    /// The map as placed in the scene: its own lights are the arena's to shadow or not.
    map_root: Handle<Node>,
    grid: Option<(WalkGrid, Vector3<f32>)>,
    /// The walkable floor as polygons, made from the grid: what the droids find their way by.
    mesh: Option<NavMesh>,
    /// Walkable cells at each end of the arena: red's and cyan's.
    spawns: [Vec<(usize, usize)>; TEAMS],
    covers: Vec<CoverSpot>,
    droids: Vec<Droid>,
    /// Whether each team, red and cyan, fights as a team (see the top), or as it used to.
    smart: [bool; TEAMS],
    /// For each team, how many of the enemies it knows of can see each coarse cell of the
    /// ground (see [`Exposure`]) - and how long till it is worked out again.
    exposure: [Exposure; TEAMS],
    /// Where each team's survey of it is up to, as a polygon's number (see
    /// [`Battle::survey_exposure`]).
    exposure_at: [usize; TEAMS],
    /// How many more routes may be worked out this frame (see [`ROUTES_A_FRAME`]).
    routes_left: usize,
    /// The bolts everyone fires, and those on their way to someone they will hit.
    bolts: Option<Bolts>,
    arrivals: Vec<Arrival>,
    /// Where each team last saw each enemy, and how long ago.
    intel: [Intel; TEAMS],
    /// Which way each team has seen each enemy going (see [`Battle::note_going`]): where it was
    /// when last worked out, how long ago, and its way and speed, in meters a second.
    going: [Vec<Option<(Vector3<f32>, f32, Vector3<f32>)>>; TEAMS],
    score: [u32; TEAMS],
    phase: Phase,
    paused: bool,
    rng: Option<Rng>,
    // The spectator camera.
    camera: Handle<Node>,
    camera_position: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    looking: bool,
    keys: Vec<KeyCode>,
    following: Option<usize>,
    /// Whether the droid followed is seen through its own eyes, or from behind.
    first_person: bool,
    // The interface.
    scoreboard: Handle<Text>,
    banner: Handle<Text>,
    help: Handle<Text>,
}

/// The plugin compares itself for the editor; a battle is only ever equal to itself.
impl PartialEq for Battle {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[derive(Default, Debug, PartialEq, Visit, Reflect)]
#[reflect(non_cloneable, type_uuid = "6c1f5a7e-3b8d-4e2a-9f41-7d0c2b9e8a13")]
struct TeamBattle {
    #[visit(skip)]
    #[reflect(hidden)]
    battle: Battle,
}

#[allow(dead_code)]
fn main() {
    run(COMBAT);
}

/// Fights battles in `arena` until the window is closed.
pub fn run(arena: Arena) {
    // Assets are looked up next to this crate, wherever it is started from.
    let _ = std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"));
    let mut executor = Executor::from_params(
        Some(EventLoop::new().unwrap()),
        GraphicsContextParams {
            window_attributes: WindowAttributes::default()
                .with_title(arena.title)
                .with_resizable(true),
            // A benchmark times the frames as fast as they come.
            vsync: std::env::var("BATTLE_BENCH").is_err(),
            msaa_sample_count: None,
            graphics_server_constructor: Default::default(),
            named_objects: false,
        },
    );
    let mut battle = TeamBattle::default();
    battle.battle.arena = arena;
    battle.battle.bench = Bench::wanted();
    executor.add_plugin(battle);
    executor.run()
}

impl Plugin for TeamBattle {
    fn init(&mut self, _scene_path: Option<&str>, mut ctx: PluginContext) -> GameResult {
        self.battle.init(&mut ctx);
        Ok(())
    }

    fn update(&mut self, ctx: &mut PluginContext) -> GameResult {
        self.battle.update(ctx);
        Ok(())
    }

    fn on_os_event(&mut self, event: &Event<()>, mut ctx: PluginContext) -> GameResult {
        self.battle.on_event(event, &mut ctx);
        Ok(())
    }
}

impl Battle {
    fn init(&mut self, ctx: &mut PluginContext) {
        let mut scene = Scene::new();
        scene.rendering_options.ambient_lighting_color = self.arena.ambient;
        scene.rendering_options.environment_lighting_source = EnvironmentLightingSource::AmbientColor;
        scene.rendering_options.fog = self.arena.fog;
        if let Some(strength) = self.arena.sun {
            // Behind the cloud, high up: soft light from above, a little from one side.
            DirectionalLightBuilder::new(
                BaseLightBuilder::new(
                    BaseBuilder::new().with_cast_shadows(self.arena.shadows).with_local_transform(
                        TransformBuilder::new()
                            .with_local_rotation(
                                UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 35f32.to_radians())
                                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 60f32.to_radians()),
                            )
                            .build(),
                    ),
                )
                .with_intensity(strength),
            )
            .build(&mut scene.graph);
        }
        self.camera_position = Vector3::from(self.arena.camera);
        self.pitch = self.arena.pitch.to_radians();
        self.camera = CameraBuilder::new(BaseBuilder::new()).build(&mut scene.graph).to_base();
        self.scene = ctx.scenes.add(scene);
        self.map = Some(ctx.resource_manager.request::<Model>(self.map_path()));
        let old = std::env::var("BATTLE_OLD").unwrap_or_default();
        let old = old.to_lowercase();
        self.smart = Team::ALL.map(|team| !(old == team.name().to_lowercase() || old == "both" || old == "all"));
        let teams = self.teams();
        Log::info(format!(
            "Team battle: fighting as a team - {}",
            teams.iter().map(|t| format!("{} {}", t.name().to_lowercase(), self.smart[t.index()])).collect::<Vec<_>>().join(", ")
        ));
        for team in teams {
            self.models[team.index()] = Some(ctx.resource_manager.request::<Model>(MODELS[team.index()]));
        }

        ctx.user_interfaces.add(UserInterface::new(Vector2::new(1280.0, 720.0)));
        let ui = ctx.user_interfaces.first_mut();
        self.scoreboard = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(12.0))
                .with_horizontal_alignment(HorizontalAlignment::Center)
                .with_foreground(Brush::Solid(Color::WHITE).into()),
        )
        .with_font_size(30.0.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_text("Loading the arena...")
        .build(&mut ui.build_ctx());
        self.banner = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::top(96.0))
                .with_horizontal_alignment(HorizontalAlignment::Center)
                .with_foreground(Brush::Solid(Color::WHITE).into()),
        )
        .with_font_size(24.0.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .build(&mut ui.build_ctx());
        self.help = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(12.0))
                .with_vertical_alignment(fyrox::gui::VerticalAlignment::Bottom)
                .with_foreground(Brush::Solid(Color::opaque(200, 200, 200)).into()),
        )
        .with_font_size(16.0.into())
        .with_vertical_text_alignment(fyrox::gui::VerticalAlignment::Bottom)
        .with_text(
            "WASD fly   Q/E down/up   Shift faster   Right mouse look\n\
             Tab follow a droid   V first/third person   F free camera   Space pause   R new match   Esc quit",
        )
        .build(&mut ui.build_ctx());
        // The UI's root gives its children only the size they ask for, in its corner; a screen is
        // the size of the window, so in one the scoreboard is at the top in the middle and the help
        // along the bottom.
        let children = [self.scoreboard, self.banner, self.help].map(|text| text.to_base());
        let ctx = &mut ui.build_ctx();
        ScreenBuilder::new(WidgetBuilder::new().with_children(children)).build(ctx);
    }

    /// The arena's map, or `BATTLE_MAP`'s.
    fn map_path(&self) -> String {
        std::env::var("BATTLE_MAP").unwrap_or_else(|_| self.arena.map.to_string())
    }

    /// How many kills win a match: [`KILL_LIMIT`], or more with more droids a team.
    fn kill_limit(&self) -> u32 {
        KILL_LIMIT.max(KILLS_PER_DROID * self.arena.per_team as u32)
    }

    /// The teams fighting, as many as the arena says.
    fn teams(&self) -> Vec<Team> {
        Team::ALL.into_iter().take(self.arena.teams.clamp(2, TEAMS)).collect()
    }

    fn rng(&mut self) -> &mut Rng {
        self.rng.get_or_insert_with(|| {
            if let Some(seed) = std::env::var("BATTLE_SEED").ok().and_then(|s| s.parse().ok()) {
                return Rng::new(seed);
            }
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |t| t.as_nanos() as u64);
            Rng::new(seed)
        })
    }

    fn random(&mut self, range: (f32, f32)) -> f32 {
        between(self.rng(), range)
    }

    fn update(&mut self, ctx: &mut PluginContext) {
        let dt = ctx.dt;
        match self.phase {
            Phase::Loading => {
                let Some(map) = self.map.clone() else { return };
                // The models of the teams that fight: the rest are never asked for.
                let models: Vec<Option<ModelResource>> =
                    self.teams().iter().map(|t| self.models[t.index()].clone().filter(|m| !m.is_failed_to_load())).collect();
                if map.is_failed_to_load() {
                    self.set_text(ctx, self.scoreboard, format!("Could not load {}", self.map_path()));
                    self.map = None;
                } else if models.iter().any(Option::is_none) {
                    self.set_text(ctx, self.scoreboard, format!("Could not load {MODELS:?}"));
                    self.map = None;
                } else if map.is_ok() && models.iter().flatten().all(|m| m.is_ok()) {
                    self.place_level(&mut ctx.scenes[self.scene], &map);
                    self.phase = Phase::Settling(0);
                }
            }
            // Colliders only reach the physics world with its next step.
            Phase::Settling(frames) if frames < 2 => self.phase = Phase::Settling(frames + 1),
            Phase::Settling(_) => {
                let graph = &ctx.scenes[self.scene].graph;
                self.survey(graph);
                self.find_cover(graph);
                self.start_match(&mut ctx.scenes[self.scene]);
                self.phase = Phase::Playing;
            }
            Phase::Playing if !self.paused => {
                let graph = &mut ctx.scenes[self.scene].graph;
                let were: Vec<Vector3<f32>> = self.droids.iter().map(|d| d.position).collect();
                let t = std::time::Instant::now();
                self.think(graph, dt);
                let thought = Bench::ms(t);
                let t = std::time::Instant::now();
                self.move_droids(dt);
                let moved = Bench::ms(t);
                if let Some(bench) = self.bench.as_mut() {
                    let now = std::time::Instant::now();
                    if let Some(last) = bench.last {
                        bench.frames.push((now - last).as_secs_f32() * 1000.0);
                    }
                    if bench.started.is_none() {
                        bench.started = Some(now);
                        bench.score = self.score;
                    }
                    bench.last = Some(now);
                    bench.played += dt;
                    bench.parts.push([thought, moved, 0.0, 0.0]);
                    if let fyrox::engine::GraphicsContext::Initialized(graphics) = &*ctx.graphics_context {
                        let stats = graphics.renderer.get_statistics();
                        bench.render.push(stats.pure_frame_time * 1000.0);
                        bench.draws.push(stats.geometry.draw_calls as f32);
                        bench.triangles.push(stats.geometry.triangles_rendered as f32 / 1000.0);
                        let l = &stats.lighting;
                        bench.lights.push((l.point_lights_rendered + l.spot_lights_rendered + l.directional_lights_rendered) as f32);
                        bench.shadow_maps.push((l.point_shadow_maps_rendered + l.spot_shadow_maps_rendered + l.csm_rendered) as f32);
                    }
                    bench.walked += self.droids.iter().zip(&were).filter(|(d, _)| d.alive()).map(|(d, w)| flat(d.position - w).norm().min(1.0)).sum::<f32>();
                    bench.targeting += usize::from(self.droids.iter().any(|d| d.alive() && d.target.is_some()));
                }
                if let Some(&team) = self.teams().iter().find(|team| self.score[team.index()] >= self.kill_limit()) {
                    self.phase = Phase::Won(team, 6.0);
                    Log::info(format!(
                        "Team battle: match over, {} wins, {}",
                        team.name(),
                        score_line(&self.score, self.arena.teams)
                    ));
                }
            }
            Phase::Won(team, left) if !self.paused => {
                if left - dt <= 0.0 {
                    self.start_match(&mut ctx.scenes[self.scene]);
                    self.phase = Phase::Playing;
                } else {
                    self.phase = Phase::Won(team, left - dt);
                }
            }
            _ => (),
        }
        if matches!(self.phase, Phase::Playing | Phase::Won(..)) {
            self.raise_the_fallen(&mut ctx.scenes[self.scene]);
        }
        for i in 0..self.droids.len() {
            let at = self.droids[i]
                .target
                .map(|j| self.droids[j].chest())
                .or(self.droids[i].suppress_at.map(|(_, at)| at));
            self.droids[i].target_at = at;
        }
        let graph = &mut ctx.scenes[self.scene].graph;
        let dt = if self.paused { 0.0 } else { dt };
        let t = std::time::Instant::now();
        self.show_droids(graph, dt);
        let shown = Bench::ms(t);
        let t = std::time::Instant::now();
        self.fire_bolts(graph, dt);
        self.bolts_arrive(graph, dt);
        let bolts = Bench::ms(t);
        self.move_camera(graph, dt);
        self.update_text(ctx);
        let playing = self.phase == Phase::Playing && !self.paused;
        if let Some(bench) = self.bench.as_mut().filter(|_| playing) {
            if let Some(part) = bench.parts.last_mut() {
                part[2] = shown;
                part[3] = bolts;
            }
            if bench.played >= bench.length && bench.frames.len() > 1 {
                let report = bench.report(self.arena.title, self.droids.len(), self.score, self.arena.teams);
                println!("{report}");
                Log::info(report);
                ctx.loop_controller.exit();
            }
        }
    }

    // ------------------------------------------------------------------ the level

    fn place_level(&mut self, scene: &mut Scene, map: &ModelResource) {
        let root = map.instantiate(scene);
        self.map_root = root;
        scene.graph.update_hierarchical_data();
        if !self.arena.shadows {
            let lights: Vec<Handle<Node>> = scene
                .graph
                .traverse_handle_iter(root)
                .filter(|&h| {
                    let node = &scene.graph[h];
                    node.cast::<PointLight>().is_some() || node.cast::<SpotLight>().is_some() || node.cast::<DirectionalLight>().is_some()
                })
                .collect();
            for light in lights {
                scene.graph[light].set_cast_shadows(false);
            }
        }

        // The map marks its light strips pure magenta: they glow, and each gets a lamp.
        let mut glass = Material::standard();
        glass.set_property("diffuseColor", Color::opaque(255, 240, 215));
        glass.set_property("emissionStrength", MaterialProperty::Vector3(Vector3::repeat(4.0)));
        let glass = MaterialResource::new_embedded(glass);
        let mut lamps = Vec::new();
        let meshes: Vec<Handle<Node>> = scene
            .graph
            .traverse_handle_iter(root)
            .filter(|&h| scene.graph[h].is_mesh())
            .collect();
        for &handle in &meshes {
            let bounds = scene.graph[handle].world_bounding_box();
            let Some(mesh) = scene.graph[handle].cast_mut::<Mesh>() else { continue };
            for surface in mesh.surfaces_mut() {
                let magenta = diffuse_color(&surface.material().data_ref())
                    == Some(Color::opaque(255, 0, 255));
                if magenta {
                    surface.set_material(glass.clone());
                    let center = bounds.center();
                    lamps.push(Vector3::new(center.x, bounds.min.y - 0.4, center.z));
                }
            }
        }
        for position in lamps {
            PointLightBuilder::new(
                BaseLightBuilder::new(
                    BaseBuilder::new()
                        .with_cast_shadows(false)
                        .with_local_transform(TransformBuilder::new().with_local_position(position).build()),
                )
                .with_color(Color::opaque(255, 235, 205))
                .with_intensity(1.2)
                .with_scatter_enabled(false),
            )
            .with_radius(10.0)
            .build(&mut scene.graph);
        }

        // Leaves - canopies and bushes - hide what is in them, but stop nothing: they are left out,
        // as the game leaves them out of its levels.
        let solid = meshes
            .into_iter()
            .filter(|&h| !scene.graph[h].name().starts_with(FOLIAGE))
            .map(GeometrySource)
            .collect();
        let collider = ColliderBuilder::new(BaseBuilder::new())
            .with_shape(ColliderShape::trimesh(solid))
            .build(&mut scene.graph);
        RigidBodyBuilder::new(BaseBuilder::new().with_child(collider))
            .with_body_type(RigidBodyType::Static)
            .build(&mut scene.graph);
        self.level = collider;
    }

    /// The first thing in the way from `from` to `to`, as how far along it is.
    fn blocked(graph: &Graph, from: Vector3<f32>, to: Vector3<f32>) -> Option<f32> {
        let way = to - from;
        let length = way.norm();
        if length < 1.0e-4 {
            return None;
        }
        let mut hits = Vec::new();
        graph.physics.cast_ray(
            RayCastOptions {
                ray_origin: Point3::from(from),
                ray_direction: way / length,
                max_len: length,
                groups: Default::default(),
                sort_results: true,
            },
            &mut hits,
        );
        // Only the arena itself stands in the way: not a droid lying there, nor what it lost.
        let arena = |collider: Handle<Collider>| {
            graph
                .try_get::<Node>(collider.to_base())
                .ok()
                .and_then(|c: &Node| graph.try_get_of_type::<RigidBody>(c.parent()).ok())
                .is_some_and(|body| body.body_type() == RigidBodyType::Static)
        };
        hits.iter()
            .find(|hit| arena(hit.collider))
            .map(|hit| (hit.position.coords - from).norm())
    }

    fn clear(graph: &Graph, from: Vector3<f32>, to: Vector3<f32>) -> bool {
        Self::blocked(graph, from, to).is_none()
    }

    /// Samples the arena for floor a droid can stand on, with room round it: the arena's floor,
    /// and whatever stairs lead up from it to, storey over storey.
    fn survey(&mut self, graph: &Graph) {
        let mut bounds = AxisAlignedBoundingBox::default();
        for node in graph.linear_iter() {
            if node.is_mesh() {
                bounds.add_box(node.world_bounding_box());
            }
        }
        // Keep the camera inside a narrower arena than the combat map, by its long side.
        self.camera_position.z = self.camera_position.z.max(bounds.min.z + 1.5);
        // Floor a droid can stand on, with room round it, as the game finds it: the arena's floor
        // and whatever stairs lead up from it to, storey over storey - a watchtower's platform
        // over the ground under it, a barracks' upper floor over its lower.
        let Some((mut grid, origin)) = maze::survey::survey(graph, self.level, Handle::NONE, self.arena.sun.is_some()) else {
            return;
        };
        let (width, depth, storeys) = (grid.width, grid.depth, grid.storeys);
        // The arena's floor: the level most of the floor is at. Whatever is found under it - under
        // a platform in the void, out of sight - is no part of the arena.
        let mut levels: Vec<f32> = grid.walkable_cells().map(|(x, row)| grid.floor(x, row)).collect();
        levels.sort_by(f32::total_cmp);
        let ground_level = levels.get(levels.len() / 2).copied().unwrap_or(0.0);
        let under: Vec<(usize, usize)> = grid.walkable_cells().filter(|&(x, row)| grid.floor(x, row) < ground_level - 1.0).collect();
        for &(x, row) in &under {
            grid.set(x, row, false);
        }
        if !under.is_empty() {
            Log::info(format!("Team battle: {} cells under the arena's floor at {ground_level:.1} m left out", under.len()));
        }

        // Keep only the biggest connected area: anything else is outside the walls.
        let mut best: Vec<(usize, usize)> = Vec::new();
        let mut seen = vec![false; width * grid.rows()];
        let cells: Vec<_> = grid.walkable_cells().collect();
        for (x, z) in cells {
            if seen[z * width + x] {
                continue;
            }
            let area: Vec<_> = grid
                .distances_from((x, z))
                .iter()
                .enumerate()
                .filter(|(_, d)| d.is_some())
                .map(|(i, _)| (i % width, i / width))
                .collect();
            for &(ax, az) in &area {
                seen[az * width + ax] = true;
            }
            if area.len() > best.len() {
                best = area;
            }
        }
        let mut kept = WalkGrid::with_storeys(width, depth, storeys, CELL).with_tuning(AI.grid);
        for &(x, z) in &best {
            kept.set(x, z, true);
            kept.set_floor(x, z, grid.floor(x, z));
        }

        // Each team starts in the tenth of the arena at its own end.
        // On the arena's floor, not up a barracks' stairs.
        let ground: Vec<(usize, usize)> =
            best.iter().copied().filter(|&(x, row)| (grid.floor(x, row) - ground_level).abs() < 0.5).collect();
        let (min_x, max_x) = ground.iter().fold((usize::MAX, 0), |(lo, hi), &(x, _)| (lo.min(x), hi.max(x)));
        let (min_z, max_z) = ground.iter().fold((usize::MAX, 0), |(lo, hi), &(_, z)| (lo.min(z), hi.max(z)));
        if self.arena.teams <= 2 {
            // From either end along x: the tenth of it at each.
            let band = (max_x - min_x) / 10;
            self.spawns[Team::Red.index()] = ground.iter().copied().filter(|&(x, _)| x <= min_x + band).collect();
            self.spawns[Team::Cyan.index()] = ground.iter().copied().filter(|&(x, _)| x >= max_x - band).collect();
        } else {
            // From its four sides - red's across from cyan's along x, green's from yellow's along
            // z: out from the middle along each way, the outermost tenth of the floor in a band
            // across the middle of that side - not its corners, which an octagon has not, nor
            // whatever sticks out further off to one side.
            let n = ground.len().max(1) as f32;
            let (cx, cz) = ground.iter().fold((0.0, 0.0), |(a, b), &(x, row)| (a + x as f32 / n, b + (row % depth) as f32 / n));
            let (span_x, span_z) = ((max_x - min_x) as f32, (max_z - min_z) as f32);
            let side = |along_x: bool, sign: f32| -> Vec<(usize, usize)> {
                let (span, across) = if along_x { (span_x, span_z) } else { (span_z, span_x) };
                let offsets = |&(x, row): &(usize, usize)| {
                    let (dx, dz) = (x as f32 - cx, (row % depth) as f32 - cz);
                    if along_x { (sign * dx, dz.abs()) } else { (sign * dz, dx.abs()) }
                };
                let band: Vec<(usize, usize)> =
                    ground.iter().copied().filter(|cell| offsets(cell).1 <= 0.2 * across).collect();
                let far = band.iter().map(|cell| offsets(cell).0).fold(f32::MIN, f32::max);
                band.into_iter().filter(|cell| offsets(cell).0 >= far - 0.1 * span).collect()
            };
            let sides = [(true, -1.0), (true, 1.0), (false, -1.0), (false, 1.0)];
            for (team, (along_x, sign)) in Team::ALL.iter().zip(sides).take(self.arena.teams) {
                self.spawns[team.index()] = side(along_x, sign);
            }
        }
        Log::info(format!(
            "Team battle: {} walkable cells, spawn cells {}",
            best.len(),
            self.teams().iter().map(|t| format!("{} {}", t.name().to_lowercase(), self.spawns[t.index()].len())).collect::<Vec<_>>().join(", ")
        ));
        // BATTLE_DUMP_GRID=<file> saves the arena's walk grid as hydroxus-ai's navmesh_bench reads
        // it, to time its routes on this arena.
        if let Ok(path) = std::env::var("BATTLE_DUMP_GRID") {
            let mut out = Vec::new();
            for v in [kept.width as u32, kept.depth as u32, kept.storeys as u32] { out.extend(v.to_le_bytes()); }
            for v in [CELL, origin.x, origin.y, origin.z] { out.extend(v.to_le_bytes()); }
            for row in 0..kept.rows() { for x in 0..kept.width {
                out.push(kept.is_walkable(x, row) as u8); out.extend(kept.floor(x, row).to_le_bytes());
            } }
            let _ = std::fs::write(path, out);
        }
        let mut mesh = NavMesh::build_with(&kept, origin, |_, _| 0, AI.navmesh);
        mesh.walk = AI.walk;
        self.mesh = Some(mesh);
        self.grid = Some((kept, origin));
    }

    /// Finds every place right beside an obstacle that hides a crouched droid from some side.
    fn find_cover(&mut self, graph: &Graph) {
        let Some((grid, origin)) = &self.grid else { return };
        let directions: Vec<Vector3<f32>> = (0..16)
            .map(|i| forward(i as f32 * std::f32::consts::TAU / 16.0))
            .collect();
        let mut covers = Vec::new();
        // Every other cell is plenty, and keeps droids from crowding onto neighbouring spots.
        for (x, z) in grid.walkable_cells().filter(|&(x, z)| x % 2 == 0 && z % 2 == 0) {
            let position = grid.on_floor(*origin, (x, z));
            // What hides it crouched, or - lower, a fallen trunk - lying down.
            let eyes = position + Vector3::y() * EYES_CROUCHED;
            let low = position + Vector3::y() * EYES_PRONE;
            let shields: Vec<_> = directions
                .iter()
                .copied()
                .filter(|dir| !Self::clear(graph, eyes, eyes + dir * 1.2) || !Self::clear(graph, low, low + dir * 1.2))
                .collect();
            if !shields.is_empty() {
                covers.push(CoverSpot { position, shields });
            }
        }
        let ground = covers.iter().map(|c| c.position.y).fold(f32::MAX, f32::min);
        let high = covers.iter().filter(|c| c.position.y - ground > AI.vantage.rise).count();
        Log::info(format!("Team battle: {} cover spots, {high} of them up high", covers.len()));
        self.covers = covers;
    }

    // ------------------------------------------------------------------ the match

    /// Has every light that is not the map's - each droid's eyes, its gun's muzzle, each bolt's
    /// flash - cast no shadows. Small and many, they lit only what is right by them; but each one
    /// in view drew a shadow map a side of everything round it, which with a hundred droids was
    /// most of every frame.
    fn quiet_droid_lights(&self, graph: &mut Graph) {
        let map: std::collections::HashSet<Handle<Node>> = graph.traverse_handle_iter(self.map_root).collect();
        let lights: Vec<Handle<Node>> = graph
            .pair_iter()
            .filter(|(h, node)| {
                !map.contains(h)
                    && node.cast_shadows()
                    && (node.cast::<PointLight>().is_some() || node.cast::<SpotLight>().is_some())
            })
            .map(|(h, _)| h)
            .collect();
        for light in lights {
            graph[light].set_cast_shadows(false);
        }
    }

    fn start_match(&mut self, scene: &mut Scene) {
        ragdoll::prepare(&mut scene.graph);
        match self.bolts.as_mut() {
            Some(bolts) => bolts.clear(&mut scene.graph),
            None => self.bolts = Some(Bolts::new(&mut scene.graph)),
        }
        self.arrivals.clear();
        for mut droid in self.droids.drain(..) {
            Self::clear_body(&mut scene.graph, &mut droid);
        }
        self.score = [0; TEAMS];
        self.following = None;
        for team in self.teams() {
            for _ in 0..self.arena.per_team {
                if let Some(droid) = self.make_droid(scene, team) {
                    self.droids.push(droid);
                }
            }
        }
        for i in 0..self.droids.len() {
            self.respawn(i);
        }
        self.quiet_droid_lights(&mut scene.graph);
        self.intel = std::array::from_fn(|_| Intel::new(self.droids.len()));
        self.going = std::array::from_fn(|_| vec![None; self.droids.len()]);
    }

    /// One of the team's droids, made from its model, standing where it is put. None if the
    /// model is not the droid it should be.
    fn make_droid(&mut self, scene: &mut Scene, team: Team) -> Option<Droid> {
        let model = self.models[team.index()].clone()?;
        let root = PivotBuilder::new(BaseBuilder::new()).build(&mut scene.graph).to_base();
        // Its feet at the root; only the first droid says what it finds in the model.
        let quiet = !self.droids.is_empty();
        let Some(avatar) = Avatar::spawn(&model, scene, root, 0.0, quiet) else {
            scene.graph.remove_node(root);
            return None;
        };
        avatar.set_eyes(Some(team.color()));
        // Standing still, it steps round to turn rather than turning on its heels.
        let mut avatar = avatar;
        avatar.set_steps_round(true);
        let likes_rifle = self.random((0.0, 1.0));
        Some(Droid {
            team,
            root,
            avatar,
            position: Vector3::zeros(),
            was_at: Vector3::zeros(),
            gait: Gait::Jogging,
            retreating: false,
            fired: false,
            rifle: false,
            fire_mode: FireMode::Burst,
            burst_left: 0,
            rounds: pistol::MAGAZINE,
            likes_rifle,
            pending: None,
            aim_bias: (0.0, 0.0),
            target_at: None,
            calm_for: HOLSTER_AFTER,
            air: None,
            jumped: false,
            low_jump: false,
            ragdoll: None,
            fallen: false,
            heading: 0.0,
            shown: None,
            health: HEALTH,
            dead_for: None,
            route: Vec::new(),
            going_to: None,
            task: Task::Advance,
            crouched: false,
            prone: false,
            cover_prone: false,
            target: None,
            objective: None,
            planned: 0.0,
            high: None,
            reload: 0.0,
            look_timer: 0.0,
            hurt_flash: 0.0,
            kills: 0,
            deaths: 0,
            suppressed: 0.0,
            looking_up: 0.0,
            suppress_at: None,
            flank: None,
            part: None,
            after: None,
            stuck_from: Stuck::default(),
            aware: Vec::new(),
            heard: None,
            blind: BlindFire::default(),
            blind_at: None,
        })
    }

    fn respawn(&mut self, i: usize) {
        let team = self.droids[i].team.index();
        // With nowhere of its own to start from, anywhere a team starts from.
        let spawns = match self.spawns[team].is_empty() {
            false => team,
            true => match (0..TEAMS).find(|&t| !self.spawns[t].is_empty()) {
                Some(t) => t,
                None => return,
            },
        };
        let count = self.spawns[spawns].len();
        let pick = self.rng().below(count);
        let cell = self.spawns[spawns][pick];
        let count_all = self.droids.len();
        let Some((grid, origin)) = &self.grid else { return };
        let position = grid.on_floor(*origin, cell);
        let middle = grid.center(*origin, (grid.width / 2, grid.depth / 2));
        let droid = &mut self.droids[i];
        droid.position = position;
        // Facing the middle of the arena.
        let way = flat(middle - position);
        droid.heading = if way.norm() > 1.0e-3 { way.x.atan2(way.z) } else { 0.0 };
        droid.health = HEALTH;
        droid.dead_for = None;
        droid.route.clear();
        droid.going_to = None;
        droid.task = Task::Advance;
        droid.crouched = false;
        droid.prone = false;
        droid.cover_prone = false;
        droid.target = None;
        droid.objective = None;
        droid.high = None;
        droid.suppressed = 0.0;
        droid.suppress_at = None;
        droid.stuck_from = Stuck::at(droid.position);
        // A new droid: unaware of anyone, and hearing nothing.
        droid.aware = vec![Detection::default(); count_all];
        droid.heard = None;
        droid.blind.reset();
        droid.blind_at = None;
        // Two of each five of a team go round the side, one each way.
        droid.flank = match i % self.arena.per_team % 5 {
            1 => Some(1.0),
            3 => Some(-1.0),
            _ => None,
        };
        self.pick_gun(i);
    }

    /// Droid `i` picks its gun for the life it is starting, as it likes and for what it is to do:
    /// holding the line, the rifle for its reach, unless it much prefers the pistol; going round
    /// the side, the pistol, light and quick, unless it much prefers the rifle. A rifle it sets to
    /// three-round bursts - or, now and then, to full auto - fully charged.
    fn pick_gun(&mut self, i: usize) {
        let auto = self.random((0.0, 1.0)) < AUTO_CHANCE;
        let droid = &mut self.droids[i];
        droid.rifle = match droid.flank {
            None => droid.likes_rifle > 0.25,
            Some(_) => droid.likes_rifle > 0.8,
        };
        droid.fire_mode = if auto { FireMode::Auto } else { FireMode::Burst };
        droid.burst_left = 0;
        droid.rounds = pistol::MAGAZINE;
        // Each looks about in its own time, not all on the same frame.
        let first_look = self.random((0.0, LOOK_EVERY));
        self.droids[i].look_timer = first_look;
    }

    // ------------------------------------------------------------------ the AI

    fn enemies_of(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        let team = self.droids[i].team;
        (0..self.droids.len()).filter(move |&j| self.droids[j].team != team && self.droids[j].alive())
    }

    /// What there is to see by in the arena: light, and its fog, if any - which droids see
    /// through only so far, and less well the further, as the player does; hearing goes through.
    fn seeing(&self) -> Seeing {
        Seeing { dark: false, fog: self.arena.fog.as_ref().map(sight_fog), looking_up: false }
    }

    /// How visible droid `j` is to droid `i`, from 0 to 1 (see [`Sight::visibility`]): in
    /// front of it, nearer and standing the more so, and nothing in the way. Fighting, it sees as
    /// far whether they crouch or not.
    fn visibility(&self, graph: &Graph, i: usize, j: usize) -> f32 {
        let (me, them) = (&self.droids[i], &self.droids[j]);
        let stance = match them.posture() {
            Posture::Crawling => Stance::Crawling,
            Posture::Crouching => Stance::Crouching,
            Posture::Standing => Stance::Standing,
        };
        let sight = sight(self.arena.sight);
        // Up above it, it does not look, unless shot at from up there - or it is the one it is
        // fighting, followed up with its eyes.
        let up = me.looking_up > 0.0;
        if !up && me.target != Some(j) && sight.overhead(me.position, them.position) {
            return 0.0;
        }
        let seen = sight.visibility(self.alert(i), me.position, me.heading, them.position, stance, self.seeing().looking_up(up));
        if seen > 0.0 && (Self::clear(graph, me.eyes(), them.eyes()) || Self::clear(graph, me.eyes(), them.chest())) {
            seen
        } else {
            0.0
        }
    }

    /// Whether droid `i` is in the fight - with a target, an enemy its team knows of, shooting
    /// heard or firing blind - which is Alert to hydroxus-ai, or calm.
    fn alert(&self, i: usize) -> Option<Alert> {
        let me = &self.droids[i];
        let fighting = me.target.is_some()
            || me.heard.is_some()
            || me.blind.at().is_some()
            || self.nearest_known_enemy(i).is_some();
        fighting.then_some(Alert::Alert)
    }

    /// How keenly droid `i` looks and listens: in the fight, as one searching does.
    fn eagerness(&self, i: usize) -> f32 {
        if self.alert(i).is_some() { AI.detection.searching } else { 1.0 }
    }

    fn think(&mut self, graph: &mut Graph, dt: f32) {
        for intel in &mut self.intel {
            intel.age(dt);
        }
        for (_, since, _) in self.going.iter_mut().flatten().flatten() {
            *since += dt;
        }
        self.assign_parts();
        // After a long frame, no more of the survey, and no more looking about, than after a
        // short one: it all came due at once, and made the next frame long too.
        let steady = dt.min(MOST_THINKING_DT);
        self.routes_left = ROUTES_A_FRAME;
        self.survey_exposure(graph, steady);
        for droid in &mut self.droids {
            droid.suppressed = (droid.suppressed - dt).max(0.0);
            droid.looking_up = (droid.looking_up - dt).max(0.0);
            droid.planned += dt;
            // Shooting heard is gone to a while, and given up once there.
            if let Some((at, age)) = droid.heard {
                let there = flat(at - droid.position).norm() < HEARD_REACHED;
                droid.heard = (age + dt < HEARD_FOR && !there && droid.target.is_none()).then_some((at, age + dt));
            }
            // Firing blind until it sees whoever it was, or it is done.
            let sees_them = droid.target.is_some() && droid.target == droid.blind_at;
            droid.blind.step(dt, sees_them, !droid.alive());
            if droid.blind.at().is_none() {
                droid.blind_at = None;
            }
        }
        for i in 0..self.droids.len() {
            if let Some(left) = self.droids[i].dead_for {
                if left - dt <= 0.0 {
                    self.respawn(i);
                } else {
                    self.droids[i].dead_for = Some(left - dt);
                }
                continue;
            }
            self.droids[i].reload -= dt;
            self.droids[i].look_timer -= steady;
            if self.droids[i].look_timer <= 0.0 {
                self.droids[i].look_timer += LOOK_EVERY;
                self.look(graph, i);
            }
            self.decide(graph, i, dt);
            self.shoot(graph, i);
        }
    }

    /// Droid `i` looks about it: becomes the more aware of each enemy it can see, the quicker the
    /// more visible they are, and less of those it cannot (see [`Detection`]); picks a target
    /// among those it has spotted, and tells its team where they are.
    fn look(&mut self, graph: &Graph, i: usize) {
        let eagerness = self.eagerness(i);
        let seen: Vec<(usize, f32)> = self.enemies_of(i).map(|j| (j, self.visibility(graph, i, j))).collect();
        let mut visible = Vec::new();
        for (j, seen) in seen {
            let aware = &mut self.droids[i].aware[j];
            aware.step(&AI.detection, seen, eagerness, LOOK_EVERY);
            if seen > 0.0 && aware.spotted() {
                visible.push(j);
            }
        }
        let team = self.droids[i].team.index();
        for &j in &visible {
            self.intel[team].saw(j, self.droids[j].position);
            self.note_going(team, j);
        }
        let me = self.droids[i].position;
        if !self.smart(i) {
            self.droids[i].target = visible.into_iter().min_by(|&a, &b| {
                let d = |j: usize| (self.droids[j].position - me).norm();
                d(a).total_cmp(&d(b))
            });
            return;
        }
        // Whoever is shooting at it first, then the hurt, the exposed and the near; and it keeps
        // to the one it has unless another is clearly better.
        let worth = |j: usize| {
            let them = &self.droids[j];
            let in_the_open = !them.crouched && !them.prone && them.cover_spot().is_none();
            team::target_worth((them.position - me).norm(), them.health / HEALTH, them.target == Some(i), in_the_open)
        };
        self.droids[i].target = team::target(&visible, self.droids[i].target, worth);
    }

    /// Team `team` sees enemy `j` where it is: which way it is going, from where the team saw it
    /// last - a fifth of a second ago at least, or the step says nothing - the last way seen
    /// counting for half. Seen after a long while, it starts afresh.
    fn note_going(&mut self, team: usize, j: usize) {
        let at = self.droids[j].position;
        let Some(entry) = self.going[team].get_mut(j) else { return };
        *entry = match *entry {
            Some(seen @ (_, since, _)) if since < 0.2 => Some(seen),
            Some((was, since, way)) if since < 2.0 => Some((at, 0.0, way * 0.5 + flat(at - was) / since * 0.5)),
            _ => Some((at, 0.0, Vector3::zeros())),
        };
    }

    /// Each smart team's droids advancing with no target of their own take parts in going after
    /// the enemy each believes nearest, with the rest of the team going after the same one (see
    /// hydroxus-ai's [`roles`]): the nearest run at them, the rest - if it is on the move - cut it
    /// off ahead. Each knows only what its team has seen.
    fn assign_parts(&mut self) {
        let alive: Vec<bool> = self.droids.iter().map(|d| d.alive()).collect();
        // Who goes after whom: (team, enemy, droids).
        let mut groups: Vec<(usize, usize, Vec<usize>)> = Vec::new();
        for i in 0..self.droids.len() {
            let d = &self.droids[i];
            if !alive[i] || !self.smart(i) || d.task != Task::Advance || d.target.is_some() {
                self.droids[i].part = None;
                self.droids[i].after = None;
                continue;
            }
            let (t, me) = (d.team.index(), d.position);
            let nearest = self.intel[t].seen_within(AI.team.intel_time, |j| alive[j]).min_by(|a, b| (a.1 - me).norm().total_cmp(&(b.1 - me).norm()));
            let Some((j, _)) = nearest else {
                self.droids[i].part = None;
                self.droids[i].after = None;
                continue;
            };
            match groups.iter_mut().find(|g| g.0 == t && g.1 == j) {
                Some(g) => g.2.push(i),
                None => groups.push((t, j, vec![i])),
            }
        }
        for (t, j, members) in groups {
            let Some((_, them)) = self.intel[t].seen_within(AI.team.intel_time, |k| k == j).next() else { continue };
            let going = self.going[t].get(j).copied().flatten().map(|(_, _, way)| way).filter(|way| way.norm() >= AI.orchestrate.going_least);
            let chasers: Vec<Chaser> = members
                .iter()
                .map(|&i| Chaser { feet: self.droids[i].position, them, going, sees: false, aggression: 0.5, was: self.droids[i].part })
                .collect();
            for ((&i, part), chaser) in members.iter().zip(roles::parts(&AI.roles, &chasers)).zip(chasers) {
                if matches!(part, Part::CutOff { .. }) && !matches!(self.droids[i].part, Some(Part::CutOff { .. })) {
                    Log::info(format!(
                        "Team battle: {} droid {} goes to cut {} droid {} off",
                        self.droids[i].team.name(),
                        i % self.arena.per_team + 1,
                        self.droids[j].team.name(),
                        j % self.arena.per_team + 1
                    ));
                }
                self.droids[i].part = Some(part);
                self.droids[i].after = Some(chaser);
            }
        }
    }

    /// Whether droid `i`'s team fights as a team.
    fn smart(&self, i: usize) -> bool {
        self.smart[self.droids[i].team.index()]
    }

    /// Where each team's enemies, as it knows of them, can see: works out anew, for every coarse
    /// cell of the ground, how many of them could see a droid standing there.
    ///
    /// A slice of the ground each frame, for `dt` of the survey's time, so that each team's is
    /// worked out anew every [`AI`]'s `exposure_every` seconds as a whole, but never all at once:
    /// with many enemies on a big arena, all of it in one frame took tens of milliseconds.
    fn survey_exposure(&mut self, graph: &Graph, dt: f32) {
        let Some(mesh) = &self.mesh else { return };
        let polys = mesh.len();
        if polys == 0 {
            return;
        }
        let slice = ((polys as f32 * dt / AI.team.exposure_every.max(1.0e-3)).ceil() as usize).clamp(1, polys);
        for t in 0..self.arena.teams {
            if !self.smart[t] {
                continue;
            }
            let alive = |j: usize| self.droids.get(j).is_some_and(|d| d.alive());
            let seen = self.intel[t].known(&AI.team, alive);
            let from = self.exposure_at[t] % polys;
            let to = (from + slice).min(polys);
            self.exposure[t].survey_part(&AI.team, mesh, &seen, CHEST_STANDING, |enemy, at| {
                Self::clear(graph, enemy + Vector3::y() * EYES_STANDING, at)
            }, from..to);
            self.exposure_at[t] = to % polys;
        }
    }

    /// Where droid `i` believes the nearest enemy is, from what its team has seen lately.
    fn nearest_known_enemy(&self, i: usize) -> Option<Vector3<f32>> {
        let me = &self.droids[i];
        self.intel[me.team.index()].nearest(&AI.team, me.position, |j| self.droids[j].alive())
    }

    fn decide(&mut self, graph: &Graph, i: usize, dt: f32) {
        let threat = self.droids[i]
            .target
            .map(|j| self.droids[j].position)
            .or_else(|| self.nearest_known_enemy(i));
        let task = self.droids[i].task.clone();
        // Lying down only to hide behind low cover, or to fight where it is.
        if !matches!(task, Task::Hidden { .. } | Task::Standing { .. }) {
            self.droids[i].prone = false;
        }
        match task {
            Task::Advance => {
                // Pinned by fire it cannot answer, out in the open: to cover, or flat on the floor.
                let pinned = self.droids[i].suppressed > 0.0 && self.droids[i].target.is_none() && self.droids[i].air.is_none();
                if let (true, Some(threat)) = (pinned, threat) {
                    if !self.take_cover(graph, i, threat, false) {
                        let left = self.random((1.5, 3.0));
                        let droid = &mut self.droids[i];
                        droid.task = Task::Standing { left };
                        droid.prone = true;
                        droid.route.clear();
                        droid.going_to = None;
                    }
                    return;
                }
                if let (Some(_), Some(threat)) = (self.droids[i].target, threat) {
                    self.droids[i].crouched = false;
                    if !self.take_cover(graph, i, threat, false) {
                        // No cover to be had: most lie down to fight where they are - harder to
                        // see from far off, and to hit - and the rest stand, stepping aside.
                        let prone = self.random((0.0, 1.0)) < PRONE_CHANCE;
                        let left = self.random(if prone { (2.0, 4.0) } else { (1.0, 2.0) });
                        self.droids[i].task = Task::Standing { left };
                        self.droids[i].prone = prone;
                        if prone {
                            self.droids[i].route.clear();
                            self.droids[i].going_to = None;
                        } else if self.smart(i) {
                            self.sidestep(i, threat);
                        }
                    }
                    return;
                }
                // Move up: to where an enemy was seen - or, going round the side, off to one side
                // of it - or somewhere in the enemy half.
                let cut = match (self.droids[i].part, self.droids[i].after, &self.mesh) {
                    (Some(part), Some(after), Some(mesh)) => roles::cut_spot(&AI.roles, &AI.chase, &after, part, |p| mesh.poly_at(p).is_some()),
                    _ => None,
                };
                let goal = match threat {
                    // On the move, and its part to cut them off: ahead of them.
                    Some(_) if cut.is_some() => cut.unwrap(),
                    Some(threat) if self.smart(i) && self.droids[i].flank.is_some() => {
                        self.flank_point(i, threat).unwrap_or(threat)
                    }
                    // Holding the high ground: it stays up there.
                    Some(_) if self.droids[i].high.is_some() => self.droids[i].high.unwrap(),
                    Some(threat) => threat,
                    // Shooting heard, or shot at by someone unseen: that way.
                    None => match self.droids[i].blind.at().or(self.droids[i].heard.map(|(at, _)| at)) {
                        Some(at) => at,
                        None => self.objective(i),
                    },
                };
                self.walk_to(i, goal);
                if self.droids[i].route.is_empty() {
                    self.droids[i].objective = None;
                }
                // Getting nowhere: somewhere else.
                if self.smart(i) {
                    let here = self.droids[i].position;
                    if self.droids[i].stuck_from.step(&AI.team, here, dt) {
                        self.droids[i].objective = None;
                        self.droids[i].going_to = None;
                        self.droids[i].route.clear();
                    }
                }
            }
            Task::ToCover { spot, peek } => {
                let position = self.covers[spot].position;
                self.walk_to(i, position);
                if (self.droids[i].position - position).norm() < 0.3 {
                    // Behind cover too low to crouch behind, it lies down.
                    let prone = self.droids[i].cover_prone;
                    self.droids[i].crouched = !prone;
                    self.droids[i].prone = prone;
                    let left = self.random((0.6, 1.4));
                    self.droids[i].task = Task::Hidden { spot, peek, left };
                }
            }
            Task::Hidden { spot, peek, left } => {
                // Behind tall cover it stands with its back to it rather than crouching.
                let tall = Droid::tall_cover(spot, peek, &self.covers).is_some();
                let prone = !tall && self.droids[i].cover_prone;
                self.droids[i].crouched = !tall && !prone;
                self.droids[i].prone = prone;
                let position = self.covers[spot].position;
                if tall && (self.droids[i].position - position).norm() < 1.5 {
                    // Back in from leaning out round it.
                    let droid = &mut self.droids[i];
                    droid.position = position;
                    droid.was_at = position;
                    droid.route.clear();
                } else {
                    self.walk_to(i, position);
                }
                // Flanked: an enemy can see it where it crouches. Find better cover.
                let exposed = self.enemies_of(i).any(|j| {
                    (self.droids[j].position - position).norm() < 40.0
                        && self.sees_spot(graph, j, position, self.droids[i].eye_height())
                });
                if exposed {
                    if let Some(threat) = threat {
                        self.take_cover(graph, i, threat, self.droids[i].health < RETREAT_HEALTH);
                    }
                    return;
                }
                // Shot at, it keeps its head down.
                let down = self.smart(i) && self.droids[i].suppressed > 0.0;
                if left - dt > 0.0 || down {
                    self.droids[i].task = Task::Hidden { spot, peek, left: (left - dt).max(0.0) };
                    return;
                }
                if threat.is_none() {
                    // Nobody about: back to looking for them.
                    self.droids[i].task = Task::Advance;
                    self.droids[i].crouched = false;
                } else {
                    let left = self.random((1.2, 2.2));
                    self.droids[i].task = Task::Peeking { spot, peek, left };
                    self.droids[i].crouched = false;
                }
            }
            Task::Peeking { spot, peek, left } => {
                self.droids[i].crouched = false;
                if Droid::tall_cover(spot, peek, &self.covers).is_some() {
                    // Round tall cover it leans out from where it stands, rather than stepping out:
                    // its eyes, for what it sees and shoots, are out at `peek`, and its body is
                    // drawn behind the cover, leaning out (see `show_droids`).
                    let droid = &mut self.droids[i];
                    droid.position = peek;
                    droid.was_at = peek;
                    droid.route.clear();
                } else {
                    self.walk_to(i, peek);
                }
                let smart = self.smart(i);
                // With nobody in sight, it fires at where one was just seen, to keep them down.
                self.droids[i].suppress_at = if smart && self.droids[i].target.is_none() {
                    self.recently_seen(i)
                } else {
                    None
                };
                // Shot at, it ducks back down.
                if smart && self.droids[i].suppressed > 0.0 {
                    self.droids[i].suppress_at = None;
                    self.droids[i].crouched = true;
                    self.droids[i].task = Task::Hidden { spot, peek, left: 0.4 };
                    return;
                }
                if left - dt > 0.0 {
                    self.droids[i].task = Task::Peeking { spot, peek, left: left - dt };
                    return;
                }
                self.droids[i].suppress_at = None;
                if self.droids[i].target.is_none() && threat.is_none() {
                    self.droids[i].task = Task::Advance;
                    return;
                }
                if self.droids[i].target.is_none() {
                    // Nobody to shoot from here: push on to the next cover - fighting as a team,
                    // only while a teammate is firing to cover it, or it is on its own.
                    if smart && !self.covered(i) {
                        let left = self.random((0.6, 1.2));
                        self.droids[i].crouched = true;
                        self.droids[i].task = Task::Hidden { spot, peek, left };
                        return;
                    }
                    self.droids[i].task = Task::Advance;
                    return;
                }
                let left = self.random((0.5, 1.2));
                self.droids[i].crouched = true;
                self.droids[i].task = Task::Hidden { spot, peek, left };
            }
            Task::Standing { left } => {
                self.droids[i].route.clear();
                self.droids[i].going_to = None;
                // Lying flat, it stays down while it is being shot at.
                let pinned = self.droids[i].prone && self.droids[i].suppressed > 0.0;
                if left - dt > 0.0 && (self.droids[i].target.is_some() || pinned) {
                    self.droids[i].task = Task::Standing { left: left - dt };
                } else {
                    self.droids[i].task = Task::Advance;
                }
            }
        }
    }

    /// Where droid `i`, going round the side, makes for against a `threat` (see
    /// [`team::flank_point`]).
    fn flank_point(&self, i: usize, threat: Vector3<f32>) -> Option<Vector3<f32>> {
        let side = self.droids[i].flank?;
        let team = self.droids[i].team;
        let mates: Vec<Vector3<f32>> = self
            .droids
            .iter()
            .filter(|d| d.team == team && d.alive())
            .map(|d| d.position)
            .collect();
        let mesh = self.mesh.as_ref()?;
        team::flank_point(&AI.team, mesh, self.droids[i].position, &mates, threat, side)
    }

    /// Has droid `i`, with no cover to be had, step aside across the line from `threat`, one way
    /// or the other, rather than stand there.
    fn sidestep(&mut self, i: usize, threat: Vector3<f32>) {
        let side = if self.random((0.0, 1.0)) < 0.5 { 1.0 } else { -1.0 };
        let Some(mesh) = &self.mesh else { return };
        if let Some(to) = team::sidestep_to(&AI.team, mesh, self.droids[i].position, threat, side) {
            self.walk_to(i, to);
        }
    }

    /// The enemy droid `i`'s team last saw lately enough to fire at, and near enough, and where:
    /// the chest of one standing there.
    fn recently_seen(&self, i: usize) -> Option<(usize, Vector3<f32>)> {
        let alive = |j: usize| self.droids.get(j).is_some_and(|d| d.alive());
        self.intel[self.droids[i].team.index()]
            .recently_seen(&AI.team, self.droids[i].position, alive)
            .map(|(j, p)| (j, p + Vector3::y() * CHEST_STANDING))
    }

    /// Whether droid `i` has a teammate firing to cover it as it moves - or no teammate near
    /// enough to, or it has waited long enough (see [`team::covered`]).
    fn covered(&mut self, i: usize) -> bool {
        let team = self.droids[i].team;
        let mates: Vec<(Vector3<f32>, bool)> = self
            .droids
            .iter()
            .enumerate()
            .filter(|(j, d)| *j != i && d.team == team && d.alive())
            .map(|(_, d)| {
                let firing = matches!(d.task, Task::Peeking { .. } | Task::Standing { .. })
                    && (d.target.is_some() || d.suppress_at.is_some());
                (d.position, firing)
            })
            .collect();
        let roll = self.random((0.0, 1.0));
        team::covered(&AI.team, self.droids[i].position, mates, roll)
    }

    /// Whether droid `j`, from where it stands, could see eyes `height` over `spot`.
    fn sees_spot(&self, graph: &Graph, j: usize, spot: Vector3<f32>, height: f32) -> bool {
        let eyes = self.droids[j].eyes();
        Self::clear(graph, eyes, spot + Vector3::y() * height)
    }

    /// High ground for droid `i` to hold, if it is one to (see [`Droid::high`]): fighting as a
    /// team, holding the line with the rifle, and no more than [`MOST_HIGH`] of its team up
    /// there already - the best it can get to looking out over the middle of the arena.
    fn high_ground(&mut self, i: usize) -> Option<Vector3<f32>> {
        if let Some(high) = self.droids[i].high {
            return Some(high);
        }
        let me = &self.droids[i];
        if !self.smart(i) || me.flank.is_some() || !me.rifle {
            return None;
        }
        let team = me.team;
        let held: Vec<Vector3<f32>> = self
            .droids
            .iter()
            .enumerate()
            .filter(|&(j, d)| j != i && d.team == team && d.alive())
            .filter_map(|(_, d)| d.high)
            .collect();
        if held.len() >= MOST_HIGH {
            return None;
        }
        let mesh = self.mesh.as_ref()?;
        let (low, top) = mesh.bounds();
        let middle = Vector3::new(0.5 * (low.x + top.x), low.y, 0.5 * (low.z + top.z));
        let tuning = vantage::Tuning { near: HIGH_NEAR, reach: HIGH_REACH, ..AI.vantage };
        let high = vantage::high_ground(&tuning, mesh, me.position, middle, &held)?;
        Log::info(format!("Team battle: {} droid {i} takes the high ground, {:.0} m up", team.name(), high.y - middle.y));
        self.droids[i].high = Some(high);
        Some(high)
    }

    /// Somewhere to head for in the enemy's half, kept until it is reached - or, holding the
    /// line, the high ground (see [`Battle::high_ground`]).
    fn objective(&mut self, i: usize) -> Vector3<f32> {
        if let Some(objective) = self.droids[i].objective {
            return objective;
        }
        if let Some(high) = self.high_ground(i) {
            return high;
        }
        // One of the other teams, picked anew each time.
        let others: Vec<Team> = self.teams().into_iter().filter(|&t| t != self.droids[i].team).collect();
        if others.is_empty() {
            return self.droids[i].position;
        }
        let enemy = { let n = others.len(); others[self.rng().below(n)] };
        let (theirs, own) = (enemy.index(), self.droids[i].team.index());
        if self.spawns[theirs].is_empty() || self.spawns[own].is_empty() {
            return self.droids[i].position;
        }
        // Somewhere between the middle and the enemy's end.
        let a = { let n = self.spawns[theirs].len(); self.rng().below(n) };
        let b = { let n = self.spawns[own].len(); self.rng().below(n) };
        let t = self.random((0.4, 0.9));
        let (spawns, own) = (&self.spawns[theirs], &self.spawns[own]);
        let (Some((grid, origin)), Some(mesh)) = (&self.grid, &self.mesh) else { return self.droids[i].position };
        let (theirs, ours) = (grid.center(*origin, spawns[a]), grid.center(*origin, own[b]));
        let point = ours + (theirs - ours) * t;
        let objective = mesh.nearest(point).map_or(point, |n| mesh.closest_point(n, point));
        self.droids[i].objective = Some(objective);
        objective
    }

    /// Picks cover for droid `i` against a threat at `threat`: a spot the threat cannot see a
    /// crouched droid at, from which it can shoot standing or a step to the side. `retreat` takes
    /// cover further away rather than nearer.
    fn take_cover(&mut self, graph: &Graph, i: usize, threat: Vector3<f32>, retreat: bool) -> bool {
        let Some(mesh) = &self.mesh else { return false };
        let me = self.droids[i].position;
        if mesh.nearest(me).is_none() {
            return false;
        }
        let routes = mesh.entries_from(me, COVER_REACH);
        let taken: Vec<usize> = self
            .droids
            .iter()
            .enumerate()
            .filter(|(j, d)| *j != i && d.alive())
            .filter_map(|(_, d)| d.cover_spot())
            .collect();
        let threat_eyes = threat + Vector3::y() * EYES_STANDING;
        let now = self.droids[i].cover_spot();

        // Rank the reachable spots that face the threat, then check the best with rays.
        let mut candidates: Vec<(f32, usize)> = self
            .covers
            .iter()
            .enumerate()
            .filter(|(s, _)| !taken.contains(s) && Some(*s) != now)
            .filter_map(|(s, cover)| {
                let n = mesh.poly_at(cover.position)?;
                let (entry, cost) = routes.get(&n)?;
                let cost = cost + (cover.position.x - entry[0]).hypot(cover.position.z - entry[1]);
                let to_threat = flat(threat - cover.position);
                let distance = to_threat.norm();
                // Up over the threat, the better: it shoots down, and sees over what is between.
                let rank = team::cover_rank(cost, distance, retreat)? - vantage::cover_lift(&AI.vantage, cover.position.y - threat.y);
                team::shields_from(&cover.shields, to_threat / distance).then_some((rank, s))
            })
            .collect();
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));

        // Fighting as a team, the other enemies it knows of, and where its teammates are: cover
        // they can see into, or crowded by a teammate, counts for less.
        let smart = self.smart(i);
        let others: Vec<Vector3<f32>> = if smart {
            self.intel[self.droids[i].team.index()]
                .known(&AI.team, |j| self.droids.get(j).is_some_and(|d| d.alive()))
                .into_iter()
                .filter(|p| flat(p - threat).norm() > 2.0)
                .map(|p| p + Vector3::y() * EYES_STANDING)
                .collect()
        } else {
            Vec::new()
        };
        let mates: Vec<Vector3<f32>> = if smart {
            let team = self.droids[i].team;
            self.droids
                .iter()
                .enumerate()
                .filter(|(j, d)| *j != i && d.team == team && d.alive())
                .map(|(_, d)| d.cover_spot().map_or(d.position, |s| self.covers[s].position))
                .collect()
        } else {
            Vec::new()
        };
        let mut best: Option<(f32, usize, Vector3<f32>, bool)> = None;
        for &(rank, s) in candidates.iter().take(24) {
            let spot = self.covers[s].position;
            // Hidden crouched behind it - or, too low for that, lying down.
            let hides = |height: f32| Self::blocked(graph, spot + Vector3::y() * height, threat_eyes).is_some();
            let prone = if hides(EYES_CROUCHED) {
                false
            } else if hides(EYES_PRONE) {
                true
            } else {
                continue;
            };
            let crouched = spot + Vector3::y() * if prone { EYES_PRONE } else { EYES_CROUCHED };
            let seen = others.iter().filter(|eyes| Self::clear(graph, **eyes, crouched)).count();
            let crowded = mates.iter().any(|m| flat(m - spot).norm() < team::CROWDED);
            let worth = team::cover_worth(&AI.team, rank, seen, crowded);
            if best.is_some_and(|(b, _, _, _)| b <= worth) {
                continue;
            }
            // Low cover: shoot over it, standing.
            let standing = spot + Vector3::y() * EYES_STANDING;
            let peek = if Self::clear(graph, standing, threat_eyes) {
                Some(spot)
            } else {
                // High cover: step out to one side of it.
                let side = right_of(flat(threat - spot).normalize());
                [side, -side].into_iter().find_map(|side| {
                    let out = spot + side * 1.0;
                    let n = mesh.poly_at(out)?;
                    let out = Vector3::new(out.x, mesh.floor_of(n), out.z);
                    (Self::clear(graph, spot + Vector3::y() * EYES_STANDING, out + Vector3::y() * EYES_STANDING)
                        && Self::clear(graph, out + Vector3::y() * EYES_STANDING, threat_eyes))
                        .then_some(out)
                })
            };
            let Some(peek) = peek else { continue };
            best = Some((worth, s, peek, prone));
            // As it used to, the first that will do.
            if !smart {
                break;
            }
        }
        if let Some((_, s, peek, prone)) = best {
            self.droids[i].task = Task::ToCover { spot: s, peek };
            self.droids[i].cover_prone = prone;
            self.droids[i].retreating = retreat;
            // Now and then it jumps as it breaks for cover: a short hop or a high one.
            if self.droids[i].air.is_none() && self.random((0.0, 1.0)) < JUMP_CHANCE {
                let low = self.random((0.0, 1.0)) < 0.5;
                let speed = if low { JUMP_SPEED.0 } else { JUMP_SPEED.1 };
                let droid = &mut self.droids[i];
                droid.air = Some((0.0, speed));
                droid.jumped = true;
                droid.low_jump = low;
                droid.crouched = false;
            }
            return true;
        }
        false
    }

    /// Sets droid `i` on its way to `goal`, planning a route only when the goal moves.
    fn walk_to(&mut self, i: usize, goal: Vector3<f32>) {
        let droid = &self.droids[i];
        // Going there already, or near enough - a goal that drifts, an enemy on the move, is
        // routed to afresh only every so often, unless it has gone far.
        if let (Some(g), false) = (droid.going_to, droid.route.is_empty()) {
            let moved = (g - goal).norm();
            if moved < 1.0 || moved < REPLAN_FAR && droid.planned < REPLAN_EVERY {
                return;
            }
            // So many routes worked out this frame already: it keeps to the one it has a while.
            if self.routes_left == 0 {
                return;
            }
        }
        self.routes_left = self.routes_left.saturating_sub(1);
        if (droid.position - goal).norm() < 0.1 {
            self.droids[i].route.clear();
            return;
        }
        let Some(mesh) = &self.mesh else { return };
        // Fighting as a team, the way that keeps out of the enemy's sight where it can.
        let team = droid.team.index();
        let route = if self.smart[team] && !self.exposure[team].is_empty() {
            let exposure = &self.exposure[team];
            walk::route_to_weighted(mesh, droid.position, goal, f32::INFINITY, |n| exposure.cost(&AI.team, n))
        } else {
            walk::route_to(mesh, droid.position, goal, f32::INFINITY)
        };
        let droid = &mut self.droids[i];
        droid.route = route;
        droid.going_to = Some(goal);
        droid.planned = 0.0;
    }

    fn shoot(&mut self, graph: &mut Graph, i: usize) {
        // At its target, or with none in sight, at where an enemy was just seen, to keep them
        // down: that goes into their cover, or wide.
        let blind = self.droids[i].blind_at.zip(self.droids[i].blind.at());
        let (j, keeping_down) = match (self.droids[i].target, self.droids[i].suppress_at) {
            (Some(j), _) => (j, None),
            (None, Some((j, at))) => (j, Some(at)),
            // Shot at by someone it cannot see: blind, back where the bolts came from.
            (None, None) if blind.is_some() => {
                let (j, at) = blind.unwrap();
                (j, Some(blind::aim(&AI.blind, at, self.rng())))
            }
            (None, None) => {
                // Nothing to fire at: a burst under way stops.
                self.droids[i].burst_left = 0;
                return;
            }
        };
        // Not while its last shot has yet to leave the gun: each bolt goes as the gun fires.
        let hidden = matches!(self.droids[i].task, Task::Hidden { .. });
        if self.droids[i].reload > 0.0
            || self.droids[i].crouched
            || hidden
            || self.droids[i].avatar.changing_posture()
            || !self.droids[j].alive()
            || self.droids[i].pending.is_some()
        {
            return;
        }
        let (from, to) = (
            self.droids[i].eyes() - Vector3::y() * 0.35,
            keeping_down.unwrap_or_else(|| self.droids[j].chest()),
        );
        // Only while facing them - and, but for leaning out round tall cover, with its body turned
        // near enough to them for its aim to reach, not still stepping round to them.
        let droid = &self.droids[i];
        let body = droid.heading + droid.avatar.facing();
        let peeking = matches!(droid.task, Task::Peeking { .. });
        let facing = heading_of(to - from).is_some_and(|h| {
            let reach = if droid.prone { PRONE_BODY_AIM } else { BODY_AIM };
            angle_between(h, droid.heading) < 0.35 && (peeking || angle_between(h, body) < reach)
        });
        if !facing {
            return;
        }
        // Its gun up against what it is behind - seeing over or round what its gun cannot: not a
        // shot to take.
        if Self::blocked(graph, from, to).is_some_and(|along| along < OWN_COVER) {
            return;
        }
        // The pistol, a shot at a time; the rifle, a burst at a time, as fast as it fires, out of
        // its magazine, a pause after each - and, the magazine empty, as long as it takes to
        // charge again.
        let pause = FIRE_INTERVAL * self.random(BURST_PAUSE);
        let pistol_reload = FIRE_INTERVAL * self.random((0.8, 1.6));
        let droid = &mut self.droids[i];
        let rifle = droid.rifle;
        if rifle {
            if droid.burst_left == 0 {
                droid.burst_left = if droid.fire_mode == FireMode::Auto { AUTO_BURST } else { BURST_SHOTS };
            }
            droid.burst_left -= 1;
            droid.rounds = droid.rounds.saturating_sub(1);
            droid.reload = if droid.rounds == 0 {
                droid.rounds = pistol::MAGAZINE;
                droid.burst_left = 0;
                pistol::RECHARGE
            } else if droid.burst_left > 0 {
                RIFLE_GAP
            } else {
                pause
            };
        } else {
            droid.reload = pistol_reload;
        }
        let distance = (to - from).norm();
        let moving = !self.droids[i].route.is_empty();
        let reach = if rifle { RIFLE_REACH } else { PISTOL_REACH };
        let mut chance = 0.75 - distance / reach;
        if moving {
            chance *= 0.5;
        }
        if !self.droids[j].route.is_empty() {
            chance *= 0.75;
        }
        if self.droids[j].prone {
            chance *= PRONE_HARDER;
        }
        let hit_cover = Self::blocked(graph, from, to);
        let hit = keeping_down.is_none() && hit_cover.is_none() && self.random((0.0, 1.0)) < chance;
        // A miss goes a little wide.
        let end = match hit_cover {
            Some(along) => from + (to - from).normalize() * along,
            None if hit => to,
            None => {
                let wide = Vector3::new(self.random((-0.8, 0.8)), self.random((-0.4, 0.6)), self.random((-0.8, 0.8)));
                from + (to + wide - from).normalize() * (distance + 6.0)
            }
        };
        // It pulls the trigger; the bolt leaves when the pistol fires (see `fire_bolts`), and
        // does its harm when it gets there (see `bolts_arrive`).
        let damage = self.random(DAMAGE) * if rifle { RIFLE_HARM } else { 1.0 };
        self.droids[i].fired = true;
        self.droids[i].pending = Some(Shot { target: j, end, hits: hit, damage, waited: 0.0 });
    }

    /// Droid `i` fires from `from`: every enemy the shot carries to along the floor (see
    /// [`Heard`]) hears it - the more aware of `i` for it, the nearer - and, fighting no one it
    /// can see, heads for about where it was.
    fn shot_heard(&mut self, i: usize, from: Vector3<f32>) {
        let team = self.droids[i].team;
        let near = |d: &Droid| d.team != team && d.alive() && (d.position - from).norm() < SHOT_NOISE;
        if !self.droids.iter().any(near) {
            return;
        }
        let Some(noise) = self.mesh.as_ref().and_then(|mesh| Heard::at(mesh, from, SHOT_NOISE)) else { return };
        for m in 0..self.droids.len() {
            if !near(&self.droids[m]) {
                continue;
            }
            let Some(nearness) = self.mesh.as_ref().and_then(|mesh| noise.nearness(mesh, self.droids[m].position)) else {
                continue;
            };
            let eagerness = self.eagerness(m);
            let wrong = HEARD_WRONG * (1.0 - nearness);
            self.rng();
            let at = match (self.mesh.as_ref(), self.rng.as_mut()) {
                (Some(mesh), Some(rng)) if wrong > 0.1 => walk::spot_near(mesh, from, wrong, rng).unwrap_or(from),
                _ => from,
            };
            let them = &mut self.droids[m];
            them.aware[i].hear(&AI.detection, nearness, eagerness);
            if them.target.is_none() {
                them.heard = Some((at, 0.0));
            }
        }
    }

    /// `them` notices a bolt from droid `i`, fired from `from`: the more aware of `i`, as of a
    /// noise right beside it; and if it cannot see `i`, a bolt from someone unseen (see
    /// [`BlindFire`]).
    fn shot_at_by(them: &mut Droid, i: usize, from: Vector3<f32>, eagerness: f32) {
        them.aware[i].hear(&AI.detection, 1.0, eagerness);
        // Shot at from up above: it looks up.
        if AI.sight.overhead(them.position, from - Vector3::y() * HELD_AT) {
            them.looking_up = LOOKING_UP;
        }
        if them.target != Some(i) {
            them.blind.unseen_bolt(&AI.blind, from);
            if them.blind.at().is_some() {
                them.blind_at = Some(i);
            }
        }
    }

    /// Lets go the bolt of each droid whose pistol fired this frame, from its muzzle, and flies
    /// every bolt on for `dt`: into what it hits of the arena, or onto the droid it is to hit.
    fn fire_bolts(&mut self, graph: &mut Graph, dt: f32) {
        let Some(mut bolts) = self.bolts.take() else { return };
        for i in 0..self.droids.len() {
            let Some(mut shot) = self.droids[i].pending else { continue };
            // From the muzzle as the pistol fires, the way it points; if it never does, from about
            // where it is held, the way the droid faces.
            let (from, pointing) = match self.droids[i].avatar.shot() {
                Some(shot) => shot,
                None if shot.waited > 0.6 || !self.droids[i].alive() => {
                    let from = self.droids[i].eyes() - Vector3::y() * 0.35 + forward(self.droids[i].heading) * 0.4;
                    (from, (shot.end - from).try_normalize(1.0e-3).unwrap_or_else(|| forward(self.droids[i].heading)))
                }
                None => {
                    shot.waited += dt;
                    self.droids[i].pending = Some(shot);
                    continue;
                }
            };
            self.droids[i].pending = None;
            if !self.droids[i].alive() {
                continue;
            }
            if let Some(bench) = self.bench.as_mut() {
                bench.fired += 1;
            }
            self.shot_heard(i, from);
            // The bolt goes the way the pistol points. Pointed near enough at the droid it was to
            // hit, it goes onto them; pointed further off, it hits them only if that way passes
            // them, and nothing of the arena is in between.
            let Some(pointing) = pointing.try_normalize(1.0e-3) else { continue };
            // Its muzzle behind what it leans out round, or into what it shoots over: the bolt
            // leaves from where its shot was found clear (see `shoot`), about where the gun is held.
            let held = self.droids[i].eyes() - Vector3::y() * 0.35;
            let from = if Self::blocked(graph, from, from + pointing * OWN_COVER).is_some() || !Self::clear(graph, held, from) {
                held + pointing * 0.4
            } else {
                from
            };
            let chest = self.droids[shot.target].chest();
            let Some(aimed) = (chest - from).try_normalize(1.0e-3) else { continue };
            // How far over the barrel was, for it to ask that much less next time.
            if let (Some(pointed), Some(wanted)) = (heading_of(pointing), heading_of(aimed)) {
                let over = (
                    pointing.y.clamp(-1.0, 1.0).asin() - aimed.y.clamp(-1.0, 1.0).asin(),
                    wrap(pointed - wanted),
                );
                let bias = &mut self.droids[i].aim_bias;
                *bias = (
                    (bias.0 + AIM_LEARNING * over.0).clamp(-AIM_BIAS_MOST, AIM_BIAS_MOST),
                    (bias.1 + AIM_LEARNING * over.1).clamp(-AIM_BIAS_MOST, AIM_BIAS_MOST),
                );
            }
            let onto = |to: Vector3<f32>| Some((to - from).norm()).filter(|_| Self::clear(graph, from, to));
            let landing = if !shot.hits || !self.droids[shot.target].alive() {
                None
            } else if pointing.dot(&aimed) >= AIM_SNAP.cos() {
                onto(chest).map(|length| (aimed, length))
            } else {
                // Where that way passes nearest their chest, if it is near enough to hit them.
                let along = (chest - from).dot(&pointing);
                let nearest = from + pointing * along;
                (along > 0.0 && (nearest - chest).norm() < HIT_RADIUS)
                    .then(|| onto(nearest))
                    .flatten()
                    .map(|length| (pointing, length))
            };
            match landing {
                Some((way, length)) => {
                    // Onto them, glowing there as it lands.
                    if self.droids[i].rifle {
                        bolts.fire_rifle_for(graph, from, way, length, true);
                    } else {
                        bolts.fire_for(graph, from, way, length, true);
                    }
                    self.arrivals.push(Arrival {
                        shooter: i,
                        target: shot.target,
                        damage: shot.damage,
                        from,
                        to: from + way * length,
                        left: length / BOLT_SPEED,
                    });
                }
                None => {
                    // Wide by as much as the miss it was to be, or straight on if it was to hit:
                    // on till it hits the arena.
                    let wide = (shot.end - from).try_normalize(1.0e-3).map_or(Vector3::zeros(), |w| w - aimed);
                    let wide = if shot.hits { Vector3::zeros() } else { wide };
                    let way = (pointing + wide).try_normalize(1.0e-3).unwrap_or(pointing);
                    if self.droids[i].rifle {
                        bolts.fire_rifle(graph, from, way);
                    } else {
                        bolts.fire(graph, from, way);
                    }
                    // Anyone it passes or lands near keeps their head down a moment; and anyone that
                    // notices it go by - felt close, or seen in front - knows someone is shooting
                    // at them, and not seeing who, after a few fires blind back where it came from.
                    let reach = Self::blocked(graph, from, from + way * 100.0).unwrap_or(100.0);
                    let team = self.droids[i].team;
                    let sight = sight(self.arena.sight);
                    for m in 0..self.droids.len() {
                        let alert = self.alert(m);
                        let eagerness = self.eagerness(m);
                        let them = &mut self.droids[m];
                        if them.team == team || !them.alive() {
                            continue;
                        }
                        let chest = them.chest();
                        let along = (chest - from).dot(&way).clamp(0.0, reach);
                        let nearest = from + way * along;
                        let near = (nearest - chest).norm();
                        if near < AI.team.suppress_near {
                            them.suppressed = AI.team.suppressed_for;
                        }
                        if bolts::notices(&AI.bolts, &sight, alert, them.position, them.heading, near, nearest) {
                            Self::shot_at_by(them, i, from, eagerness);
                        }
                    }
                }
            }
        }
        // Only the arena stops a bolt: a droid it misses, it passes.
        bolts.fly(graph, dt, |graph, from, way, reach| {
            let end = from + way * reach;
            Self::blocked(graph, from, end).map(|along| (along, Handle::NONE))
        });
        self.bolts = Some(bolts);
    }

    /// Does the harm of each bolt that has got to the droid it hits.
    fn bolts_arrive(&mut self, graph: &mut Graph, dt: f32) {
        let mut arrived = Vec::new();
        self.arrivals.retain_mut(|arrival| {
            arrival.left -= dt;
            let here = arrival.left <= 0.0;
            if here {
                arrived.push(*arrival);
            }
            !here
        });
        for arrival in arrived {
            let Arrival { shooter: i, target: j, damage, from, to, .. } = arrival;
            if self.droids[j].alive() {
                self.hit(graph, i, j, damage, from, to);
            }
        }
    }

    /// Droid `i`'s bolt, fired from `from`, gets to droid `j` at `to`, for `damage`.
    fn hit(&mut self, graph: &mut Graph, i: usize, j: usize, damage: f32, from: Vector3<f32>, to: Vector3<f32>) {
        let distance = (to - from).norm();
        let eagerness = self.eagerness(j);
        let target = &mut self.droids[j];
        Self::shot_at_by(target, i, from, eagerness);
        target.health -= damage;
        target.hurt_flash = 0.12;
        if target.health <= 0.0 {
            target.dead_for = Some(RESPAWN_TIME);
            // It goes limp, knocked back by the shot, and may lose what it was hit in.
            let way = (to - from).normalize();
            target.avatar.set_eyes(Some(Color::BLACK));
            let going = if target.route.is_empty() { Vector3::zeros() } else { forward(target.heading) * 2.0 };
            target.ragdoll = Ragdoll::start(
                graph,
                target.avatar.root(),
                going,
                Some((way * ragdoll::STOPPING_BLOW, to)),
            );
            target.fallen = true;
            target.air = None;
            if let Some(ragdoll) = target.ragdoll.as_mut() {
                if let Some(body) = ragdoll.body_struck(graph, to, way) {
                    if let Some(part) = dismember::break_for(body) {
                        if target.avatar.break_off(graph, part) {
                            ragdoll.let_loose(graph, part);
                        }
                    }
                }
            }
            target.deaths += 1;
            target.route.clear();
            self.droids[i].kills += 1;
            self.droids[i].target = None;
            self.score[self.droids[i].team.index()] += 1;
            Log::info(format!(
                "Team battle: {} {} downs {} {} from {:.0} m, {} - {}",
                self.droids[i].team.name(),
                i % self.arena.per_team + 1,
                self.droids[j].team.name(),
                j % self.arena.per_team + 1,
                distance,
                self.droids[i].task.label(),
                score_line(&self.score, self.arena.teams)
            ));
            for intel in &mut self.intel {
                intel.forget(j);
            }
            for going in &mut self.going {
                if let Some(g) = going.get_mut(j) {
                    *g = None;
                }
            }
        } else {
            // Shot at out in the open: get to cover. Badly hurt anywhere: fall back.
            let threat = self.droids[i].position;
            let hurt = self.droids[j].health < RETREAT_HEALTH;
            let exposed = matches!(self.droids[j].task, Task::Advance | Task::Standing { .. });
            if exposed || hurt {
                self.take_cover(graph, j, threat, hurt);
            } else if let Task::Peeking { spot, peek, .. } = self.droids[j].task {
                self.droids[j].crouched = true;
                self.droids[j].task = Task::Hidden { spot, peek, left: 0.8 };
            }
        }
    }



    // ------------------------------------------------------------------ moving and showing

    fn move_droids(&mut self, dt: f32) {
        for i in 0..self.droids.len() {
            if !self.droids[i].alive() {
                continue;
            }
            // At the pace of what its feet are doing: jogging as it advances, running for cover,
            // and crouched, the crouch walk.
            // Walking with no enemy known of, jogging to one, running for cover, sprinting when
            // falling back hurt.
            let running = matches!(self.droids[i].task, Task::ToCover { .. });
            let knows = self.alert(i).is_some();
            let (posture, gait, fallback) = if self.droids[i].prone {
                (Posture::Crawling, Gait::Walking, CRAWL_SPEED)
            } else if self.droids[i].crouched {
                (Posture::Crouching, Gait::Walking, WALK_SPEED * 0.5)
            } else if running && self.droids[i].retreating {
                (Posture::Standing, Gait::Sprinting, RUN_SPEED * 1.4)
            } else if running {
                (Posture::Standing, Gait::Running, RUN_SPEED)
            } else if knows {
                (Posture::Standing, Gait::Jogging, WALK_SPEED)
            } else {
                (Posture::Standing, Gait::Walking, WALK_SPEED * 0.5)
            };
            let droid = &mut self.droids[i];
            droid.gait = gait;
            let speed = droid.avatar.pace(posture, gait).unwrap_or(fallback);
            let mut way = Vector3::zeros();
            // Getting down or up off the floor, it stops: its feet would slide.
            let mut step = if droid.avatar.changing_posture() { 0.0 } else { speed * dt };
            while let Some(&next) = droid.route.last() {
                let to = flat(next - droid.position);
                let distance = to.norm();
                if distance <= step {
                    droid.position = next;
                    droid.route.pop();
                    step -= distance;
                    way = to;
                } else {
                    droid.position += to / distance * step;
                    droid.position.y = next.y;
                    way = to;
                    break;
                }
            }
            // Face the target while there is one; otherwise, the way it is going. Behind tall
            // cover, along the wall toward the side it leans out from.
            let target = droid.target.map(|j| j);
            let along = match droid.task {
                Task::Hidden { spot, peek, .. } | Task::Peeking { spot, peek, .. } => {
                    Droid::tall_cover(spot, peek, &self.covers).map(|(_, along)| along)
                }
                _ => None,
            };
            let face = match (target, droid.suppress_at) {
                _ if along.is_some() => along,
                (Some(j), _) => heading_of(self.droids[j].position - self.droids[i].position),
                (None, Some((_, at))) => heading_of(at - self.droids[i].position),
                (None, None) => match droid.blind.at() {
                    Some(at) => heading_of(at - self.droids[i].position),
                    None => heading_of(way),
                },
            };
            let droid = &mut self.droids[i];
            if let Some(face) = face {
                let turn = wrap(face - droid.heading);
                droid.heading += turn.clamp(-TURN_SPEED * dt, TURN_SPEED * dt);
            }
        }
    }

    /// Takes away a droid's body - its model, and its limp bodies and what it lost, if it fell.
    fn clear_body(graph: &mut Graph, droid: &mut Droid) {
        if let Some(ragdoll) = droid.ragdoll.take() {
            ragdoll.remove(graph);
        }
        droid.avatar.sweep_up(graph);
        if graph.is_valid_handle(droid.root) {
            graph.remove_node(droid.root);
        }
    }

    /// Gives each droid that has come back, but still lies where it fell, a new body.
    fn raise_the_fallen(&mut self, scene: &mut Scene) {
        for i in 0..self.droids.len() {
            if !(self.droids[i].fallen && self.droids[i].alive()) {
                continue;
            }
            let team = self.droids[i].team;
            let Some(new) = self.make_droid(scene, team) else { continue };
            let droid = &mut self.droids[i];
            Self::clear_body(&mut scene.graph, droid);
            droid.root = new.root;
            droid.avatar = new.avatar;
            droid.shown = None;
            droid.fallen = false;
            droid.was_at = droid.position;
        }
    }

    fn show_droids(&mut self, graph: &mut Graph, dt: f32) {
        // Behind tall cover: which side the wall is on, where the droid stands behind it, and
        // whether it is leaning out round it.
        let tall: Vec<Option<(Wall, Vector3<f32>, bool)>> = self
            .droids
            .iter()
            .map(|d| match d.task {
                Task::Hidden { spot, peek, .. } | Task::Peeking { spot, peek, .. } => {
                    let (wall, _) = Droid::tall_cover(spot, peek, &self.covers)?;
                    Some((wall, self.covers[spot].position, matches!(d.task, Task::Peeking { .. })))
                }
                _ => None,
            })
            .collect();
        for (droid, tall) in self.droids.iter_mut().zip(tall) {
            // Lying where it fell, the physics has it.
            if let Some(ragdoll) = droid.ragdoll.as_mut().filter(|_| droid.fallen) {
                ragdoll.update(graph);
                continue;
            }
            droid.hurt_flash -= dt;
            // Its eyes in its team's colour, and white for a moment when hit.
            let eyes = if droid.hurt_flash > 0.0 { Color::WHITE } else { droid.team.color() };
            droid.avatar.set_eyes(Some(eyes));
            // In a jump, up and back down onto the floor.
            let mut falling = 0.0;
            if let Some((height, up)) = droid.air {
                let (height, up) = (height + up * dt, up - GRAVITY * dt);
                falling = (-up).max(0.0);
                droid.air = (height > 0.0).then_some((height, up));
            }
            let lift = droid.air.map_or(0.0, |(height, _)| height);
            // Its root turned the way it faces, as the player's body is turned the way they look:
            // the droid takes which way it goes, and faces, from there.
            // Leaning out round tall cover, it stands behind it: its eyes are out at `position`.
            let stands = tall.map_or(droid.position, |(_, behind, _)| behind);
            // The body goes on facing as it did across the world, and turns after the root.
            if let Some(shown) = droid.shown {
                droid.avatar.parent_turned(wrap(droid.heading - shown));
            }
            droid.shown = Some(droid.heading);
            graph[droid.root]
                .local_transform_mut()
                .set_position(stands + Vector3::y() * lift)
                .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), droid.heading));
            // How fast it went since last frame, which is what its feet keep pace with.
            let moved = flat(droid.position - droid.was_at);
            let speed = if dt > 0.0 { moved.norm() / dt } else { 0.0 };
            // Coming back to life is a jump across the arena, not a step.
            let (speed, moved) = if speed > RUN_SPEED * 2.0 { (0.0, Vector3::zeros()) } else { (speed, moved) };
            droid.was_at = droid.position;
            // The pistol out while fighting, raised and aimed at the target it has, and put away
            // a while after the last.
            droid.calm_for = if droid.target.is_some() { 0.0 } else { droid.calm_for + dt };
            let target = droid.target_at;
            let raised = target.is_some();
            // Up, and to the left of ahead, to the target.
            let look = target.map_or((0.0, 0.0), |at| {
                let to = at - droid.eyes();
                let up = (to.y / to.norm().max(1.0e-3)).clamp(-1.0, 1.0).asin();
                let left = heading_of(to).map_or(0.0, |h| wrap(h - droid.heading));
                // Less what its barrel has been pointing over by.
                (up - droid.aim_bias.0, left - droid.aim_bias.1)
            });
            // Facing a target, it strafes: going which way it goes, facing where it faces. Which
            // way it goes is from the way it faces.
            let strafing = raised && speed > 0.1;
            let way = heading_of(moved).map_or(0.0, |h| wrap(h - droid.heading));
            let going = Going {
                heading: Some(if strafing || speed > 0.1 { way } else { 0.0 }),
                speed,
                posture: droid.posture(),
                gait: droid.gait,
                grounded: droid.air.is_none(),
                jumped: std::mem::take(&mut droid.jumped),
                low: droid.low_jump,
                falling,
                lifted: 0.0,
                // Behind tall cover, its back to it at the side it leans out from - the corner -
                // and leaning out round it to shoot.
                cover: tall.map(|(wall, _, _)| wall),
                corner: tall.is_some(),
                peeking: tall.is_some_and(|(_, _, out)| out),
                pushing: speed > 0.1,
                strafing: strafing && tall.is_none(),
                armed: droid.calm_for < HOLSTER_AFTER,
                trigger: std::mem::take(&mut droid.fired),
                raised: raised && tall.is_none_or(|(_, _, out)| out),
                look,
                way,
                weapon: if droid.rifle { pistol::Weapon::Rifle } else { pistol::Weapon::Pistol },
            };
            droid.avatar.animate(graph, going, dt);
        }
    }

    fn move_camera(&mut self, graph: &mut Graph, dt: f32) {
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.yaw)
            * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), self.pitch);
        if let Some(i) = self.following.filter(|&i| i < self.droids.len()) {
            let droid = &self.droids[i];
            self.yaw = droid.heading;
            if self.first_person && droid.alive() {
                // Through its eyes - its face, its own body out of the way, as the player's is -
                // looking the way it faces, and up or down at whatever it is aiming at.
                let eyes = droid.avatar.face_at(graph).unwrap_or_else(|| droid.eyes()) + forward(droid.heading) * 0.1;
                let up = droid.target_at.map_or(0.0, |at| {
                    let to = at - eyes;
                    (-to.y).atan2(flat(to).norm())
                });
                self.pitch = up;
                self.camera_position = eyes;
            } else {
                // Behind and above the droid being followed, looking the way it faces.
                self.pitch = 20f32.to_radians();
                self.camera_position = droid.position + Vector3::y() * 3.0 - forward(droid.heading) * 5.0;
            }
        } else {
            let pressed = |key| self.keys.contains(&key);
            let speed = if pressed(KeyCode::ShiftLeft) || pressed(KeyCode::ShiftRight) { 30.0 } else { 12.0 };
            let ahead = forward(self.yaw);
            let right = -right_of(ahead);
            let mut way = Vector3::zeros();
            let axes = [
                (KeyCode::KeyW, ahead),
                (KeyCode::KeyS, -ahead),
                (KeyCode::KeyD, -right),
                (KeyCode::KeyA, right),
                (KeyCode::KeyE, Vector3::y()),
                (KeyCode::KeyQ, -Vector3::y()),
            ];
            for (key, dir) in axes {
                if pressed(key) {
                    way += dir;
                }
            }
            if way.norm() > 0.0 {
                self.camera_position += way.normalize() * speed * dt;
            }
        }
        for (i, droid) in self.droids.iter().enumerate() {
            let seen_through = self.first_person && self.following == Some(i) && droid.alive();
            droid.avatar.set_visible(graph, !seen_through);
        }
        graph[self.camera]
            .local_transform_mut()
            .set_position(self.camera_position)
            .set_rotation(rotation);
    }

    fn set_text(&self, ctx: &mut PluginContext, text: Handle<Text>, value: String) {
        ctx.user_interfaces.first_mut().send(text, TextMessage::Text(value));
    }

    fn update_text(&self, ctx: &mut PluginContext) {
        if matches!(self.phase, Phase::Loading | Phase::Settling(_)) {
            return;
        }
        let alive = |team: Team| self.droids.iter().filter(|d| d.team == team && d.alive()).count();
        let teams = self.teams();
        let board = if teams.len() == 2 {
            format!(
                "RED {}   —   {} CYAN\n{} / {} alive   first to {}",
                self.score[0],
                self.score[1],
                alive(Team::Red),
                alive(Team::Cyan),
                self.kill_limit()
            )
        } else {
            format!(
                "{}\n{} alive   first to {}",
                teams.iter().map(|t| format!("{} {}", t.name(), self.score[t.index()])).collect::<Vec<_>>().join("   "),
                teams.iter().map(|&t| alive(t).to_string()).collect::<Vec<_>>().join(" / "),
                self.kill_limit()
            )
        };
        self.set_text(ctx, self.scoreboard, board);
        let banner = match self.phase {
            Phase::Won(team, _) => format!("{} TEAM WINS", team.name()),
            _ if self.paused => "PAUSED".to_string(),
            _ => match self.following.and_then(|i| self.droids.get(i)) {
                Some(d) => format!(
                    "Following {} droid {}   {:.0} hp   {} kills {} deaths   {}",
                    d.team.name(),
                    self.following.unwrap() % self.arena.per_team + 1,
                    d.health.max(0.0),
                    d.kills,
                    d.deaths,
                    if !d.alive() {
                        "down"
                    } else if d.target.is_none() && d.blind.at().is_some() {
                        "firing blind"
                    } else if d.target.is_none() && d.heard.is_some() && d.task == Task::Advance {
                        "going to shooting heard"
                    } else if d.task == Task::Advance && matches!(d.part, Some(Part::CutOff { .. })) {
                        "cutting them off"
                    } else {
                        d.task.label()
                    }
                ),
                None => String::new(),
            },
        };
        self.set_text(ctx, self.banner, banner);
    }

    // ------------------------------------------------------------------ input

    fn on_event(&mut self, event: &Event<()>, ctx: &mut PluginContext) {
        match event {
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } if self.looking => {
                self.following = None;
                self.yaw -= delta.0 as f32 * 0.003;
                self.pitch = (self.pitch + delta.1 as f32 * 0.003).clamp(-1.5, 1.5);
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput { button: MouseButton::Right, state, .. } => {
                    self.looking = *state == ElementState::Pressed;
                }
                WindowEvent::KeyboardInput { event: input, .. } => {
                    let PhysicalKey::Code(code) = input.physical_key else { return };
                    if input.state == ElementState::Released {
                        self.keys.retain(|&k| k != code);
                        return;
                    }
                    if !self.keys.contains(&code) {
                        self.keys.push(code);
                    }
                    if input.repeat {
                        return;
                    }
                    match code {
                        KeyCode::Escape => ctx.loop_controller.exit(),
                        KeyCode::Space => self.paused = !self.paused,
                        KeyCode::KeyF => self.following = None,
                        // Following a droid: through its eyes, or from behind it.
                        KeyCode::KeyV => self.first_person = !self.first_person,
                        KeyCode::Tab if !self.droids.is_empty() => {
                            self.following = Some(self.following.map_or(0, |i| (i + 1) % self.droids.len()));
                        }
                        KeyCode::KeyR if self.grid.is_some() => {
                            self.start_match(&mut ctx.scenes[self.scene]);
                            self.phase = Phase::Playing;
                        }
                        _ => (),
                    }
                }
                WindowEvent::Focused(false) => {
                    self.keys.clear();
                    self.looking = false;
                }
                _ => (),
            },
            _ => (),
        }
    }
}

/// The colour a material is, if it says.
fn diffuse_color(material: &Material) -> Option<Color> {
    let key = ImmutableString::new("properties");
    let Some(MaterialResourceBinding::PropertyGroup(group)) = material.bindings().get(&key) else {
        return None;
    };
    match group.property_ref("diffuseColor") {
        Some(MaterialProperty::Color(color)) => Some(*color),
        _ => None,
    }
}

/// An angle brought into -π..π.
fn wrap(angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    (angle + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI
}

fn angle_between(a: f32, b: f32) -> f32 {
    wrap(a - b).abs()
}

/// The right-hand side of `ahead`, along the ground.
fn right_of(ahead: Vector3<f32>) -> Vector3<f32> {
    hydroxus_ai::navigation::steer::right_of(ahead)
}
