//! A security drone, patrolling the maze. One is put down each round well away from where the
//! player starts, and hovers [`HOVER`] above the floor along the corridors, going from one spot a
//! trip away to the next and waiting a while at each - paying the player no heed, calm. More wait
//! out of sight, to be called in (see [`Drone::call_in`]): a failed hack brings one.
//!
//! What it does - patrolling, searching, on Alert, and its flight - is decided by its mind,
//! [`hydroxus_ai::tactics::patrol`], and its way through the air by
//! [`hydroxus_ai::navigation::flight`]; this is the drone in the scene, which sees, flies and fires.
//!
//! It turns hostile when the alarm is sounded (see [`Drone::alarm`]) or when the player's pistol
//! hits a droid - or the drone itself (see [`Drone::shot`]). Then it goes about the player as a
//! sentry does, but by air:
//!
//! - **Searching**: it flies to where the player was and scans there, then to one spot after
//!   another nearby, scanning at each, for [`SEARCH`](hydroxus_ai::tactics::patrol::SEARCH) seconds; and then it is calm again, and
//!   patrols as before. It sees in front of it as a sentry that is not on Alert does - less far
//!   the lower the player is, and with the lights off.
//! - **Alert**: it can see the player. It comes on to [`ENGAGE`](hydroxus_ai::tactics::patrol::ENGAGE) from them, keeps facing them and
//!   fires every [`FIRE_EVERY`] seconds - the quadcopter three times as often - while it can see them (see [`crate::drone_shot`]), each
//!   shot hurting them where it hits. Losing them, it searches.
//!
//! [`HITS`] bolts from the pistol bring it down: it sputters, its rings break into four pieces
//! each, which fly apart, and it drops with them to the floor, where it lies for the rest of the
//! round. Its body is solid, up and down: a ball the player cannot walk through, nor the droids
//! lying in the maze tumble through, nor bolts fly through, the wreck on the floor too. Flying, it
//! never pushes into anyone: someone in its way, it hovers where it is until they move.
//!
//! Its model, [`DRONE_MODEL`], is made in Blender: `drone_root`, which the animations move about,
//! with the eye, the shell and two rings under it, and the ring pieces beside it. The rings' spin
//! is in the animations, but not its glow, which is played here to go with each: what its green
//! materials glow with, over the animation's frames at 24 a second ([`glow`]); and what colour,
//! which is its mood's ([`Drone::mood`]), as a droid's eyes show theirs - its own green calm,
//! orange searching and red on Alert - turning from one to the next over a moment.
//!
//! It lights what is round it too, in its colour, as brightly as it glows: a lamp just in front
//! of its eye, whose shadows are traced as every light's are. Not inside it, where the drone's own
//! shell and eye, which are traced with the rest of the scene, would shut the light in.
//!
//! There is a faster kind too, the quadcopter (see [`Kind::Quad`]): it goes about the player as
//! this one does, but gets about at getting on for twice the speed.
//!
//! It speaks System Latin as things happen to it - hearing the alarm, spotting the player,
//! starting to search, giving up and going down, and why it was called in - as [`DRONE_LINES`]
//! has it, or in Urbs [`URBS_DRONE_LINES`], said as beeps, hums and buzzes (see
//! [`crate::formants::chirps`]), which no one would take for speech.

use crate::leaves::Leaves;
use crate::{
    ctf::Side,
    player::pistol::Pass,
    ragdoll::CHARACTERS,
    dialogue::{screen, Mood},
    fixtures::{glow_strength, DIFFUSE_COLOR, EMISSION_STRENGTH},
    formants::Curve,
    inhabitants::{self, Alert},
    layout::Rng,
    player::posture::Posture,
};
use fyrox::{
    core::{
        algebra::{Isometry3, Point3, UnitQuaternion, Vector3},
        color::Color,
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{MaterialProperty, MaterialResource},
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{
        animation::{Animation, AnimationPlayer},
        base::BaseBuilder,
        transform::TransformBuilder,
        collider::{BitMask, Collider, ColliderBuilder, ColliderShape, InteractionGroups, QueryFilterFlags},
        graph::{
            physics::{geometry::Ball, QueryFilter, RayCastOptions},
            Graph,
        },
        light::{
            point::{PointLight, PointLightBuilder},
            BaseLightBuilder,
        },
        mesh::Mesh,
        node::Node,
        rigidbody::{RigidBody, RigidBodyBuilder, RigidBodyType},
        Scene,
    },
};
use hydroxus_ai::{
    navigation::{
        navmesh::{NavMesh, Poly},
        walk,
    },
    perception::detection::Detection,
    tactics::patrol::{self, Patrol, Speeds},
};
use serde::Deserialize;
use std::{collections::HashMap, f32::consts::PI};

/// The drone's model, and what drones say.
pub const DRONE_MODEL: &str = "data/drone.glb";
/// The quadcopter's model: see [`Kind::Quad`].
pub const QUAD_MODEL: &str = "data/quad_drone.glb";
pub const DRONE_LINES: &str = "data/dialogue/drone.json";
/// What the drones of Urbs say: of its thief, its banks and its gold.
pub const URBS_DRONE_LINES: &str = "data/dialogue/drone_urbs.json";

/// Why a drone is called in, as what it says names it: to a bank's computer cleared, to a vault
/// opened, to a vault's gold taken, and to a failed hack's trace. With no lines for why, it says
/// its "alarm" lines.
pub const CALLED: [&str; 4] = ["bank", "vault", "gold", "trace"];

/// A line a drone says: in System Latin, and what that means in English.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Said {
    pub says: String,
    pub means: String,
}

/// What drones say, by what they say it about: see [`Drone::update`].
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct DroneLines {
    pub lines: HashMap<String, Vec<Said>>,
}

impl DroneLines {
    /// The lines in the file at `path`.
    pub fn load(path: &str) -> Result<Self, String> {
        let text = crate::platform::read_to_string(path)?;
        serde_json::from_str(&text).map_err(|error| format!("{path}: {error}"))
    }

    /// The `n`th line, going round, for what is called `name`; None for something with none. Why
    /// it was called in, with no lines of its own, has the alarm's.
    pub fn line(&self, name: &str, n: usize) -> Option<&Said> {
        let lines = match self.lines.get(name) {
            Some(lines) => lines,
            None if CALLED.contains(&name) => self.lines.get("alarm")?,
            None => return None,
        };
        lines.get(n % lines.len().max(1))
    }
}

/// How much the model is scaled: as made it is 6.5 m across its rings, and 1.3 m in the game.
const SCALE: f32 = 0.2;
/// How high it hovers, in meters above the floor. The death animation drops it this far.
pub const HOVER: f32 = 1.7;
/// How far from where the player starts it is put down, in meters of walking, at least: out of
/// sight.
const AWAY_FROM_PLAYER: f32 = 20.0;
/// How far off one called in is put down, in meters of walking, at most: out of sight, but near
/// enough to be heard coming, and to come.
const CALLED_FROM: f32 = 40.0;
/// How near where it is called to a patrolling drone has to be, in meters as the crow flies, to
/// answer the call itself, rather than one being called in from nearer.
pub const ANSWERS_WITHIN: f32 = 25.0;
/// How fast it flies, in meters per second: patrolling, and hostile; how quickly it gets up to
/// speed, in meters per second per second; and how quickly it turns, in radians a second.
type Flight = Speeds;

/// The security drone's flight; and the quadcopter's, getting on for twice as fast, quicker off
/// the mark and quicker to turn - outrunning a running droid, hostile.
const FLIGHT: Flight = Flight { patrol: 1.4, hurry: 3.5, acceleration: 3.0, turn_rate: 3.0 };
const QUAD_FLIGHT: Flight = Flight { patrol: 2.5, hurry: 6.0, acceleration: 6.0, turn_rate: 5.0 };

/// What kind of drone it is. Both go about the player alike - patrolling, searching, firing - and
/// say the same; the quadcopter only flies faster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The ball with its two rings, [`DRONE_MODEL`].
    Security,
    /// The quadcopter, [`QUAD_MODEL`], made in Blender in the drone's own terms - as big, as
    /// scaled, and dropping as far: `quad_root`, which the animations move about, with its body,
    /// four rotors and the gimbal under its nose under it, and the loose rotors its death sheds
    /// beside it. Its animations are the drone's, named `quad_` where the drone's are `drone_`,
    /// and its glow, the arm strips' and the eye's, goes along with them as the drone's does. Its
    /// shots leave from `quad_muzzle`, at the end of its barrels, and its lamp is at `quad_eye`,
    /// in front of its eye: both on the gimbal, which the animations turn. Brought down, it is
    /// not its death that drops it, but the physics: it falls, tumbling, bounces and comes to
    /// rest wherever it comes to rest, as a fallen droid does (see [`Drone::fall`]), while its
    /// death runs its rotors down, swings its gimbal limp and puts its glow out.
    Quad,
}

impl Kind {
    /// Its model.
    pub const fn model(self) -> &'static str {
        match self {
            Kind::Security => DRONE_MODEL,
            Kind::Quad => QUAD_MODEL,
        }
    }

    /// Its node that the animations move about, which the lamp goes along with.
    const fn body(self) -> &'static str {
        match self {
            Kind::Security => BODY,
            Kind::Quad => "quad_root",
        }
    }

    /// What its animations' names start with, where the drone's - which name them here - start
    /// with `drone_`.
    const fn prefix(self) -> &'static str {
        match self {
            Kind::Security => "drone_",
            Kind::Quad => "quad_",
        }
    }

    /// Its barrels' muzzles, which take turns to fire, a shot at a time, if it has more than the
    /// one; and how big its shots are, as a part of the drone's: the quadcopter's are small, as
    /// its barrels are.
    const fn barrels(self) -> &'static [&'static str] {
        match self {
            Kind::Security => &[],
            Kind::Quad => &["quad_muzzle_l", "quad_muzzle_r"],
        }
    }

    pub const fn shot_size(self) -> f32 {
        match self {
            Kind::Security => 1.0,
            Kind::Quad => 0.35,
        }
    }

    /// How much each of its shots hurts, as a part of the drone's; and how often it fires on
    /// Alert, in seconds: the quadcopter's hurt half as much, three times as often - half as much
    /// again over time.
    pub const fn shot_damage(self) -> f32 {
        match self {
            Kind::Security => 1.0,
            Kind::Quad => 0.5,
        }
    }

    const fn fire_every(self) -> f32 {
        match self {
            Kind::Security => FIRE_EVERY,
            Kind::Quad => FIRE_EVERY / 3.0,
        }
    }

    /// Its nodes where its shots leave from and where its lamp goes, if it has them; without, they
    /// are along the way its body faces (see [`MUZZLE`] and [`LAMP_AT`]).
    const fn muzzle_and_eye(self) -> Option<(&'static str, &'static str)> {
        match self {
            Kind::Security => None,
            Kind::Quad => Some(("quad_muzzle", "quad_eye")),
        }
    }

    /// Whether it flies free, at any height, through the air (see [`flight`]), rather than
    /// hovering its height over the floor along the walk grid's routes.
    const fn flies_free(self) -> bool {
        matches!(self, Kind::Quad)
    }

    const fn flight(self) -> Flight {
        match self {
            Kind::Security => FLIGHT,
            Kind::Quad => QUAD_FLIGHT,
        }
    }
}
/// On Alert, how often it fires, in seconds. How near the player it comes, how far off it fires
/// from, how soon after spotting them it first fires, and how long it searches, are its mind's
/// (see [`hydroxus_ai::tactics::patrol`]).
const FIRE_EVERY: f32 = 1.5;
/// How far it looks for a way anywhere, in meters of walking. With
/// no way there - the player up on a roof it has no way under, say - a search with no bound goes
/// over every cell it can reach in the whole city before it gives up, a tenth of a second and more.
const ROUTE_REACH: f32 = 200.0;
/// How many of the pistol's bolts bring it down.
pub const HITS: u32 = 3;
/// Its body: a ball this big across its middle, in meters.
const BODY_RADIUS: f32 = 0.3;
/// The quadcopter's wreck, which falls under physics, in meters from where it hovers from: a
/// thin disc out to its rotors' guards, as high as they are, and a box about its body, from its
/// skids up to the top of its hump, the heavier, the guards being light (see [`RIM_DENSITY`]). So
/// it comes to rest flat, on its skids or on its back: on its side it stands on the rim of the
/// disc, its weight well off to one side of it, and tips over. One shape, a box or a drum, stands
/// on its side as steadily as on its skids, and stays there.
const RIM_RADIUS: f32 = 0.66;
const RIM_HALF_HEIGHT: f32 = 0.03;
const RIM_AT: f32 = 0.084;
const CORE_HALF: (f32, f32, f32) = (0.2, 0.207, 0.3);
const CORE_AT: f32 = -0.078;
/// How heavy the guards are for their size, as a part of the body's.
const RIM_DENSITY: f32 = 0.3;
/// How it grips and how it bounces, each the average of its own and what it hits: the floor's are
/// nothing, so on the floor it is half as rough and bouncy as this.
const WRECK_FRICTION: f32 = 1.0;
const WRECK_BOUNCE: f32 = 0.25;
/// How quickly its turning dies away, as a part each second: the physics has nothing to slow a
/// drum rolling on its edge, which would go on down the corridor as a coin would on glass; slowed,
/// it stops rolling, and tips over.
const WRECK_DAMPING: f32 = 1.5;
/// How hard the bolt that brings it down knocks it on, away from whoever fired, in meters per
/// second, and up; how fast that tips it over away from them, in radians a second, and how fast
/// it slews round.
const KNOCK: f32 = 1.5;
const KNOCK_UP: f32 = 0.8;
const TUMBLE: f32 = 4.0;
const SLEW: f32 = 2.0;
/// Barely moving - in meters and radians a second - for this long, in seconds, the wreck is laid
/// to rest, still for the physics, as a fallen droid is.
const WRECK_STILL: (f32, f32) = (0.05, 0.1);
const WRECK_SETTLES: f32 = 0.5;
/// How near anyone it comes, flying, in meters between its body and theirs.
const CLEARANCE: f32 = 0.05;
/// How near a bolt has to fly past its body to be taken as shot at, in meters.
const NEAR_MISS: f32 = 1.0;
/// Its lamp: where it is, in the model's own terms along the way it faces - just clear of the
/// front of its eye, which reaches 1.4 out from the middle; how bright it is for each strength of
/// [`glow`]; and how far it reaches, in meters.
const LAMP_AT: f32 = 1.7;
const LAMP_BRIGHTNESS: f32 = 1.5;
const LAMP_REACH: f32 = 5.0;
/// Where its shots leave from, in the model's own terms along the way it faces: just clear of the
/// front of its eye and of its body; and the frame of `drone_fire` they leave on, as the eye
/// flashes.
const MUZZLE: f32 = 1.7;
const FIRE_AT: f32 = 2.0;
/// How long `drone_fire` and `drone_scan` last, in seconds.
const FIRE_LENGTH: f32 = 25.0 / FPS;
const SCAN_LENGTH: f32 = 97.0 / FPS;
/// How quickly its colour turns to its mood's: most of the way in a third of a second.
const MOOD_RATE: f32 = 6.0;
/// Its node that the animations move about, which the lamp goes along with.
const BODY: &str = "drone_root";
/// The material property that says how much like metal a surface is, from 0 to 1.
const METALLIC_FACTOR: &str = "metallicFactor";
/// The animations' frames a second.
const FPS: f32 = 24.0;
/// How many times as strong its glow is in the game as [`glow`] has it, as made in Blender: the
/// engine only glows round what is brighter than 1.01, and green light counts for less than
/// three quarters of how bright it is, so that as made its green would only be coloured.
const BRIGHTNESS: f32 = 3.0;
/// Its animations: the loops it hovers in, still and on the move; and the rest, each played once.
const IDLE: &str = "drone_idle";
const PATROL: &str = "drone_patrol";
const SCAN: &str = "drone_scan";
const FIRE: &str = "drone_fire";
const DEATH: &str = "drone_death";
const ANIMATIONS: [&str; 5] = [IDLE, PATROL, SCAN, FIRE, DEATH];

/// How strongly its green materials glow `frame` frames into the animation called `name`: as
/// many times as bright as green light of strength 1.
pub fn glow(name: &str, frame: f32) -> f32 {
    let curve = match name {
        // A slow breath, in and out once a loop of 97 frames.
        "drone_idle" => {
            return 1.2 - 0.3 * (frame / 97.0 * std::f32::consts::TAU).cos();
        }
        "drone_scan" => Curve(vec![
            [77.0, 1.0],
            [78.0, 5.0],
            [81.0, 2.0],
            [84.0, 5.0],
            [87.0, 2.0],
            [90.0, 3.0],
        ]),
        "drone_fire" => Curve(vec![[0.0, 1.0], [2.0, 8.0], [14.0, 1.0]]),
        // Flickering out, and then fading.
        "drone_death" => {
            let mut points = vec![[0.0, 1.0]];
            points.extend((0..=10).map(|k| [1.0 + 2.0 * k as f32, if k % 2 == 0 { 4.0 } else { 0.0 }]));
            points.push([40.0, 0.0]);
            Curve(points)
        }
        _ => return 1.0,
    };
    curve.at(frame)
}

/// The colour a glow whose own colour is `own` has in `mood`, strength 1 in its brightest part.
pub(crate) fn mood_colour(own: Vector3<f32>, mood: Mood) -> Vector3<f32> {
    screen::eyes(mood).map_or(own, |colour| {
        let colour = Vector3::new(colour.r, colour.g, colour.b).cast::<f32>();
        colour / colour.max().max(1.0)
    })
}

/// A colour of strength 1 at its brightest, as a colour to light with.
pub(crate) fn light_colour(colour: Vector3<f32>) -> Color {
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color::opaque(byte(colour.x), byte(colour.y), byte(colour.z))
}

/// `vector` along the ground.
fn flat(vector: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(vector.x, 0.0, vector.z)
}

/// How the drone is going about the player (see [`hydroxus_ai::tactics::patrol`]).
pub use hydroxus_ai::tactics::patrol::State;

/// Where the player is, for the drone to look for them: their feet and the middle of their body,
/// what their body is to anything that hits it, how they hold themselves, whether it is too
/// dark to see them far, and whether they have a weapon drawn, which catches the eye.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    pub feet: Vector3<f32>,
    pub middle: Vector3<f32>,
    pub collider: Handle<Collider>,
    pub posture: Posture,
    /// What there is to see them by: dark or light, and fog.
    pub seeing: hydroxus_ai::perception::sight::Seeing,
    pub armed: bool,
}

/// What the drone did this frame: what it has something to say about, as [`DRONE_LINES`] names it; and
/// what it fired, if it fired.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Doing {
    pub says: Option<&'static str>,
    pub fired: Option<Fired>,
}

/// A volley a drone fired: where it is aimed from, and where each shot leaves from - from there,
/// or from the barrel whose turn it was; how big they are and how much they hurt, as a part of the
/// drone's shots, and the colour they glow.
#[derive(Debug, Clone, PartialEq)]
pub struct Fired {
    pub from: Vector3<f32>,
    pub muzzles: Vec<Vector3<f32>>,
    pub size: f32,
    pub damage: f32,
    pub colour: Vector3<f32>,
}

/// The drone in the scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Drone {
    kind: Kind,
    root: Handle<Node>,
    player: Handle<Node>,
    /// What the animations move about, and its lamp.
    body: Handle<Node>,
    lamp: Handle<Node>,
    /// Where its shots leave from and where its lamp goes, for a kind that has them (see
    /// [`Kind::muzzle_and_eye`]); NONE for one that has not.
    muzzle: Handle<Node>,
    eye: Handle<Node>,
    /// Its barrels' muzzles, for a kind that fires from more than one (see [`Kind::barrels`]),
    /// and which fires next.
    barrels: Vec<Handle<Node>>,
    barrel: usize,
    /// Its body, and the ball that is, while it is in the round.
    hull: Handle<Node>,
    collider: Handle<Collider>,
    /// Its animations by name.
    animations: Vec<(String, Handle<Animation>)>,
    /// Its own copies of the materials that glow, and which way their glow's colour goes.
    glows: Vec<(MaterialResource, Vector3<f32>)>,
    /// The colour it glows now, on its way to its mood's, as [`mood_colour`] has it.
    colour: Vector3<f32>,
    /// The animation it is playing, and for how long it has, in seconds; and whether it has fired
    /// this time through `drone_fire`.
    playing: &'static str,
    since: f32,
    fired: bool,
    /// What it is doing, where it is and where it is going: patrolling, searching, on Alert (see
    /// [`hydroxus_ai::tactics::patrol`]).
    mind: Patrol,
    /// Whether it has been put down in this round's maze.
    placed: bool,
    /// How many of the pistol's bolts have hit it.
    hits: u32,
    /// Brought down, a quadcopter falls under physics (see [`Drone::fall`]): while its body is
    /// falling, for how long it has barely moved, in seconds; None while it flies, and once it
    /// has come to rest.
    falling: Option<f32>,
    /// Whether its body is the wreck's, not the ball it flies in.
    wrecked: bool,
    /// How aware it is of the player, watching for them (see [`Drone::think`]).
    detection: Detection,
    /// In capture the flag, its side; it patrols round its side's flag (its mind's post),
    /// watching all the while, and what it goes after is whichever of the other side the game
    /// shows it (see [`crate::ctf`]).
    side: Option<Side>,
}

impl Drone {
    /// Puts a drone of `kind` into `scene` from its `model`, out of sight until it is placed. None
    /// if the model has no animations.
    pub fn spawn(model: &ModelResource, scene: &mut Scene, kind: Kind) -> Option<Self> {
        let root = model.instantiate(scene);
        let graph = &mut scene.graph;
        graph[root]
            .local_transform_mut()
            .set_scale(Vector3::repeat(SCALE));
        graph[root].set_visibility(false);
        let nodes: Vec<Handle<Node>> = graph.traverse_handle_iter(root).collect();
        let Some(player) = nodes
            .iter()
            .copied()
            .find(|&node| graph[node].cast::<AnimationPlayer>().is_some())
        else {
            Log::err(format!("Drone: {} has no animations", kind.model()));
            graph.remove_node(root);
            return None;
        };
        // Its own shell and rings, just behind its lamp, would throw shadows across the corridor
        // from shadow maps. Traced shadows leave it out, as they are gathered once; so do these.
        for &node in &nodes {
            if graph[node].cast::<Mesh>().is_some() {
                graph[node].set_cast_shadows(false);
            }
        }
        let body = graph.find_by_name(root, kind.body()).map_or(root, |(body, _)| body);
        let (muzzle, eye) = kind.muzzle_and_eye().map_or((Handle::NONE, Handle::NONE), |(muzzle, eye)| {
            let find = |name| graph.find_by_name(root, name).map_or(Handle::NONE, |(node, _)| node);
            (find(muzzle), find(eye))
        });
        let barrels = kind
            .barrels()
            .iter()
            .filter_map(|&name| graph.find_by_name(root, name).map(|(node, _)| node))
            .collect();
        // Not scattering into a haze in the air: the drone is what glows.
        let lamp = PointLightBuilder::new(
            BaseLightBuilder::new(BaseBuilder::new().with_visibility(false))
                .with_intensity(LAMP_BRIGHTNESS)
                .with_scatter_enabled(false),
        )
        .with_radius(LAMP_REACH)
        .build(graph)
        .to_base();
        let glows = claim_glows(graph, &nodes);
        Log::info(format!("Drone: {} glowing materials", glows.len()));
        let animations = graph
            .try_get_mut_of_type::<AnimationPlayer>(player)
            .ok()?
            .animations_mut()
            .get_value_mut_silent()
            .pair_iter_mut()
            .map(|(handle, animation)| {
                animation.set_enabled(false);
                // Known by the drone's names, whatever its kind.
                let name = animation.name();
                let name = name.strip_prefix(kind.prefix()).map_or_else(|| name.to_owned(), |rest| format!("drone_{rest}"));
                (name, handle)
            })
            .collect::<Vec<_>>();
        for name in ANIMATIONS {
            if !animations.iter().any(|(had, _)| had == name) {
                Log::warn(format!("Drone: {} has no {name}", kind.model()));
            }
        }
        let mut drone = Self {
            kind,
            root,
            player,
            body,
            lamp,
            muzzle,
            eye,
            barrels,
            barrel: 0,
            // Given a body once it is put down.
            hull: Handle::NONE,
            collider: Handle::NONE,
            animations,
            colour: glows.first().map_or(Vector3::new(0.0, 1.0, 0.0), |(_, own)| *own),
            glows,
            playing: DEATH,
            since: 0.0,
            fired: false,
            mind: Patrol { tuning: crate::AI.patrol, ..Patrol::new(HOVER, kind.flies_free(), kind.flight(), SCAN_LENGTH) },
            placed: false,
            hits: 0,
            falling: None,
            wrecked: false,
            side: None,
            detection: Detection::default(),
        };
        drone.play(graph, IDLE);
        Some(drone)
    }

    /// Puts it down afresh, calm and whole, over the floor of `grid` whose corner is at `origin`,
    /// well away from the player's `feet`. Whether there was anywhere to put it.
    pub fn place(
        &mut self,
        graph: &mut Graph,
        mesh: &NavMesh,
        feet: Vector3<f32>,
        rng: &mut Rng,
    ) -> bool {
        self.side = None;
        self.mind.post = None;
        self.mind.watches = false;
        self.put_down(graph, mesh, feet, f32::INFINITY, rng)
    }

    /// Puts it down for `side` in capture the flag, by `post`, its side's flag, to patrol round
    /// it. Whether there was anywhere to put it.
    pub fn place_for(
        &mut self,
        graph: &mut Graph,
        mesh: &NavMesh,
        side: Side,
        post: Vector3<f32>,
        rng: &mut Rng,
    ) -> bool {
        let Some(at) = walk::spot_near(mesh, post, self.mind.tuning.post_reach, rng) else {
            return false;
        };
        // Put down anywhere, and then where it should be.
        if !self.put_down(graph, mesh, at, f32::INFINITY, rng) {
            return false;
        }
        self.side = Some(side);
        self.mind.post = Some(post);
        // On a side, it watches all the while.
        self.mind.watches = true;
        self.mind.at = at + Vector3::new(0.0, HOVER, 0.0);
        if let Ok(hull) = graph.try_get_mut_of_type::<RigidBody>(self.hull) {
            hull.local_transform_mut().set_position(self.mind.at);
        }
        self.pose(graph);
        Log::info(format!("Drone: {}'s, by its flag", side.name()));
        true
    }

    /// How aware of the player it is, if it is watching for them and not after them already (see
    /// [`hydroxus_ai::perception::detection`]).
    pub fn detection(&self) -> Option<&Detection> {
        (self.placed && self.mind.looks() && self.mind.state != State::Alert).then_some(&self.detection)
    }

    /// Whether it is after the player: on Alert.
    pub fn on_alert(&self) -> bool {
        self.placed && self.mind.state == State::Alert
    }

    /// Its side in capture the flag, if it has one.
    pub fn side(&self) -> Option<Side> {
        self.side
    }

    /// What it would go after, as a target, for another drone: its body, where it hovers.
    pub fn as_target(&self) -> Option<Target> {
        (self.placed && self.mind.state != State::Down).then(|| Target {
            feet: self.mind.at - Vector3::new(0.0, HOVER, 0.0),
            middle: self.mind.at,
            collider: self.collider,
            posture: Posture::Standing,
            seeing: hydroxus_ai::perception::sight::Seeing::default(),
            armed: true,
        })
    }

    /// Puts it down as [`Drone::place`] does, no more than `within` steps from the player's
    /// `feet` if there is room.
    fn put_down(
        &mut self,
        graph: &mut Graph,
        mesh: &NavMesh,
        feet: Vector3<f32>,
        within: f32,
        rng: &mut Rng,
    ) -> bool {
        if mesh.nearest(feet).is_none() {
            return false;
        }
        // Anywhere well away from the player: a spot picked at random will do - every polygon of
        // a surveyed level can be walked to, the survey having found it by walking - rather than
        // the way worked out to every one of a city's.
        if within.is_infinite() {
            if let Some(spot) = walk::far_spot(mesh, feet, AWAY_FROM_PLAYER, rng) {
                return self.put_at(graph, feet, spot, rng);
            }
        }
        // Within so far of them: the ways as far as that, and no further.
        let reach = within.min(1.0e4).max(AWAY_FROM_PLAYER);
        let mut reached: Vec<(Poly, f32)> = mesh.costs_from(feet, reach, |_| 0.0).into_iter().collect();
        reached.sort_unstable_by_key(|&(n, _)| n);
        let away: Vec<Poly> = reached.iter().filter(|&&(_, cost)| cost >= AWAY_FROM_PLAYER).map(|&(n, _)| n).collect();
        // Out of the way if there is room; in a small maze, as far off as it can be.
        let n = match away.is_empty() {
            false => away[rng.below(away.len())],
            true => match reached.iter().max_by(|a, b| a.1.total_cmp(&b.1)) {
                Some(&(n, _)) => n,
                None => return false,
            },
        };
        let spot = mesh.random_point(n, &mut |k| rng.below(k));
        self.put_at(graph, feet, spot, rng)
    }

    /// Puts it down over `spot`, on the floor, calm and whole, the player's `feet` where they are.
    fn put_at(
        &mut self,
        graph: &mut Graph,
        feet: Vector3<f32>,
        spot: Vector3<f32>,
        rng: &mut Rng,
    ) -> bool {
        let heading = inhabitants::between(rng, (-PI, PI));
        let resting = inhabitants::between(rng, self.mind.tuning.rest);
        self.mind.put_at(spot, heading, resting);
        self.detection.reset();
        self.placed = true;
        self.hits = 0;
        // A wreck's body is no good for flying in: it gets its ball back.
        if self.wrecked && graph.is_valid_handle(self.hull) {
            graph.remove_node(self.hull);
        }
        self.wrecked = false;
        self.falling = None;
        if !graph.is_valid_handle(self.hull) {
            (self.hull, self.collider) = hull(graph);
        }
        self.pose(graph);
        if let Ok(hull) = graph.try_get_mut_of_type::<RigidBody>(self.hull) {
            hull.local_transform_mut().set_position(self.mind.at);
        }
        self.play(graph, IDLE);
        graph[self.root].set_visibility(true);
        graph[self.lamp].set_visibility(true);
        Log::info(format!("Drone: patrolling from {:.1} m away", flat(self.mind.at - feet).norm()));
        true
    }

    /// Puts it down afresh as [`Drone::place`] does, well away from the player's `feet`, and sends
    /// it straight to search where they are, `at`, as the alarm would: called in. Whether there
    /// was anywhere to put it.
    pub fn call_in(
        &mut self,
        graph: &mut Graph,
        mesh: &NavMesh,
        feet: Vector3<f32>,
        at: Vector3<f32>,
        rng: &mut Rng,
    ) -> bool {
        if !self.put_down(graph, mesh, feet, CALLED_FROM, rng) {
            return false;
        }
        self.search(at);
        true
    }

    /// Takes it out of the round: out of sight, and out of the way of bolts, until it is put down
    /// again.
    pub fn hide(&mut self, graph: &mut Graph) {
        self.placed = false;
        self.wrecked = false;
        self.falling = None;
        self.mind.calm();
        graph[self.root].set_visibility(false);
        graph[self.lamp].set_visibility(false);
        if graph.is_valid_handle(self.hull) {
            graph.remove_node(self.hull);
        }
    }

    /// Whether it is in this round, hovering where it is.
    pub fn is_placed(&self) -> bool {
        self.placed
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn state(&self) -> State {
        self.mind.state
    }

    /// Where it is hovering.
    pub fn at(&self) -> Vector3<f32> {
        self.mind.at
    }

    /// What the animations move about: where it speaks from.
    pub fn body(&self) -> Handle<Node> {
        self.body
    }

    /// Its body, while it is in the round.
    pub fn collider(&self) -> Handle<Collider> {
        self.collider
    }

    /// Where its health bar goes - over the top of it - and how much health it has left, from 0
    /// to 1, while it is after the player or has been hit, and is up.
    pub fn health_bar(&self, graph: &Graph) -> Option<(Vector3<f32>, f32)> {
        let shown = match self.mind.state {
            State::Down => false,
            State::Alert | State::Search { .. } => true,
            State::Patrol => self.hits > 0,
        };
        (shown && self.placed).then(|| {
            let top = graph[self.body].global_position() + Vector3::new(0.0, BODY_RADIUS, 0.0);
            (top, 1.0 - self.hits as f32 / HITS as f32)
        })
    }

    /// How it feels, which colours its glow: calm, searching, or after the player.
    pub fn mood(&self) -> Mood {
        match self.mind.state {
            State::Patrol => Mood::Normal,
            State::Search { .. } | State::Down => Mood::Agitated,
            State::Alert => Mood::Hostile,
        }
    }

    /// Gives up on the player, the one it was after arrested: back to its patrol.
    pub fn stand_down(&mut self) {
        self.mind.stand_down();
    }

    /// The alarm sounded, or a droid shot, with the player's feet at `at`: calm or searching, it
    /// searches there. What it has to say about it, if anything.
    pub fn alarm(&mut self, at: Vector3<f32>) -> Option<&'static str> {
        match self.mind.state {
            State::Down | State::Alert => None,
            _ if !self.placed => None,
            // The player's own side's pays the alarm no heed.
            _ if self.side.is_some_and(Side::is_players) => None,
            _ => self.mind.alarm(at).map(said),
        }
    }

    /// Bolts that flew `passes` this frame, fired by `fired_by`'s side - or the player in the maze,
    /// with none: one that went within [`NEAR_MISS`] of its body, not its own side's, has it search
    /// where it was fired from, unless it is after the player already. What it says, if anything.
    pub fn near_miss(&mut self, passes: &[Pass], fired_by: Option<Side>) -> Option<&'static str> {
        if !self.placed || matches!(self.mind.state, State::Down | State::Alert) {
            return None;
        }
        if fired_by.is_some() && fired_by == self.side {
            return None;
        }
        let pass = passes.iter().find(|pass| pass.nearest(self.mind.at) <= BODY_RADIUS + NEAR_MISS)?;
        self.mind.shot_at(pass.fired_at).map(said)
    }

    /// A bolt that hit `collider`, fired by the player from `from`: if it hit the drone, it takes
    /// the hit, and searches where it came from if it was not after the player already. Whether it
    /// was the drone, and whether that brought it down.
    pub fn shot(&mut self, graph: &mut Graph, collider: Handle<Collider>, from: Vector3<f32>) -> Option<bool> {
        if collider != self.collider || self.mind.state == State::Down || !self.placed {
            return None;
        }
        self.hits += 1;
        if self.hits >= HITS {
            // Its body goes down with it (see `update`), and lies on the floor as solid as ever:
            // the drone's as its death has it, the quadcopter's falling.
            if self.kind == Kind::Quad {
                self.fall(graph, from);
            }
            self.mind.down();
            self.play(graph, DEATH);
            return Some(true);
        }
        self.mind.hit(from);
        Some(false)
    }

    /// Drops it out of the air, brought down by a bolt fired from `from`: its ball is swapped for
    /// a body the shape of it, which falls under physics - going on as it was flying, knocked on
    /// away from whoever fired and tipping over away from them - and tumbles, bounces and slides
    /// to wherever it comes to rest; the model goes along with it (see [`Drone::follow_wreck`]).
    fn fall(&mut self, graph: &mut Graph, from: Vector3<f32>) {
        if graph.is_valid_handle(self.hull) {
            graph.remove_node(self.hull);
        }
        let away = flat(self.mind.at - from).try_normalize(1.0e-4).unwrap_or_else(Vector3::z);
        (self.hull, self.collider) = wreck(graph, self.mind.at, self.mind.heading, self.mind.speed, away);
        self.wrecked = true;
        self.falling = Some(0.0);
    }

    /// Puts the model where its falling wreck is, and lays the wreck to rest once it has barely
    /// moved for a while.
    fn follow_wreck(&mut self, graph: &mut Graph, dt: f32) {
        let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(self.hull) else {
            return;
        };
        // Its own place, which the physics sets: it has no parent.
        let (at, rotation) = (**body.local_transform().position(), **body.local_transform().rotation());
        let still = body.lin_vel().norm() < WRECK_STILL.0 && body.ang_vel().norm() < WRECK_STILL.1;
        if let Some(stilled) = self.falling.as_mut() {
            *stilled = if still { *stilled + dt } else { 0.0 };
            if *stilled >= WRECK_SETTLES {
                body.set_body_type(RigidBodyType::Static);
                self.falling = None;
            }
        }
        self.mind.at = at;
        graph[self.root]
            .local_transform_mut()
            .set_position(at)
            .set_rotation(rotation);
    }

    /// Searches from `at`, going there first.
    fn search(&mut self, at: Vector3<f32>) {
        self.mind.search(at);
    }

    /// Starts the animation called `name` from its start, looping those it hovers in, unless it
    /// is playing it already.
    fn play(&mut self, graph: &mut Graph, name: &'static str) {
        if self.playing == name {
            return;
        }
        let find = |name: &str| {
            self.animations
                .iter()
                .find(|(had, _)| had == name)
                .map(|&(_, handle)| handle)
        };
        let (before, now) = (find(self.playing), find(name));
        self.playing = name;
        self.since = 0.0;
        self.fired = false;
        let Ok(player) = graph.try_get_mut_of_type::<AnimationPlayer>(self.player) else {
            return;
        };
        let animations = player.animations_mut().get_value_mut_silent();
        if let Some(before) = before.and_then(|h| animations.try_get_mut(h).ok()) {
            before.set_enabled(false);
        }
        if let Some(now) = now.and_then(|h| animations.try_get_mut(h).ok()) {
            now.set_loop(name == IDLE || name == PATROL)
                .set_enabled(true)
                .rewind();
        }
    }

    /// Where it is and which way it faces, as it is.
    fn pose(&self, graph: &mut Graph) {
        graph[self.root]
            .local_transform_mut()
            .set_position(self.mind.at)
            .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.mind.heading));
    }

    /// Whether it can see the `player`: in front of it, as a sentry sees - from as far as it sees
    /// at all on Alert - and with nothing in the way from its eye to the middle of their body.
    /// Whether it can see `player` - or whoever else it is shown as a target - from where it is.
    pub fn sees(&self, graph: &Graph, player: &Target) -> bool {
        self.visibility(graph, player) > 0.0
    }

    /// How visible the `player` - or whoever else it is shown as a target - is to it from where it
    /// is (see [`hydroxus_ai::perception::sight::Sight::visibility`]): 0 where it cannot see them, or something is in the way.
    pub fn visibility(&self, graph: &Graph, player: &Target) -> f32 {
        if !self.placed || self.mind.state == State::Down {
            return 0.0;
        }
        let alert = match self.mind.state {
            State::Alert => Alert::Alert,
            _ => Alert::Evasion,
        };
        let under = self.mind.at - Vector3::new(0.0, HOVER, 0.0);
        let stance = player.posture.into();
        let visible = crate::AI.sight.visibility(Some(alert), under, self.mind.heading, player.feet, stance, player.seeing);
        if visible <= 0.0 {
            return 0.0;
        }
        let from = graph[self.body].global_position();
        let way = player.middle - from;
        let length = way.norm();
        if length < 1.0e-3 {
            return visible;
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
        let clear = hits.iter().find(|hit| hit.collider != self.collider).is_some_and(|hit| hit.collider == player.collider);
        if clear { visible } else { 0.0 }
    }

    /// Goes about its business for another `dt` over the floor of `grid` whose corner is at
    /// `origin`, with the `player` where they are - if `live`, as while a round is being played;
    /// otherwise it only glows, hovering where it is. What it did.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        graph: &mut Graph,
        mesh: &NavMesh,
        player: &Target,
        reachable: &dyn Fn(Vector3<f32>) -> bool,
        leaves: Option<&Leaves>,
        rng: &mut Rng,
        dt: f32,
        live: bool,
    ) -> Doing {
        let mut doing = Doing::default();
        if live && self.placed {
            self.since += dt;
            let leaves = leaves.map(Leaves::graph);
            doing.says = self.think(graph, mesh, player, reachable, leaves, rng, dt);
            self.fly(graph, mesh, player, leaves, dt);
            match self.wrecked {
                true => self.follow_wreck(graph, dt),
                false => self.pose(graph),
            }
            // Its body goes where the animations take it: bobbing as it hovers, and down to the
            // floor when it is shot down. As of the last frame, which is as near as matters.
            let at = match self.mind.state {
                State::Down => graph[self.body].global_position(),
                _ => self.mind.at,
            };
            // Moved by hand, it would push anyone in its way with no end of force - down through
            // the floor, even, settling or falling onto them. So it never moves into anyone: it
            // stays put until they are out of the way.
            if graph.is_valid_handle(self.hull) && !self.wrecked {
                let from = graph[self.hull].global_position();
                if !blocked(graph, self.hull, from, at - from) {
                    if let Ok(hull) = graph.try_get_mut_of_type::<RigidBody>(self.hull) {
                        hull.set_next_kinematic_translation(at);
                    }
                }
            }
            // Which animation goes with what it is doing: firing and scanning play through.
            let next = match self.mind.state {
                State::Down => DEATH,
                State::Alert if self.mind.fire_in <= 0.0 => {
                    self.mind.fire_in = self.kind.fire_every();
                    // Firing again before the last has played through - the quadcopter does - it
                    // starts again from the start.
                    if self.playing == FIRE {
                        self.playing = IDLE;
                    }
                    FIRE
                }
                _ if self.playing == FIRE && self.since < FIRE_LENGTH => FIRE,
                State::Search { .. } if self.mind.scanning > 0.0 => SCAN,
                _ if self.mind.speed > 0.2 => PATROL,
                _ => IDLE,
            };
            self.play(graph, next);
        }
        self.glow(graph, dt);
        // It fires as the eye flashes, at wherever the player is by then.
        let time = self.time(graph);
        if self.playing == FIRE && !self.fired && time * FPS >= FIRE_AT && live {
            self.fired = true;
            let from = match graph.is_valid_handle(self.muzzle) {
                true => graph[self.muzzle].global_position(),
                false => graph[self.body]
                    .global_transform()
                    .transform_point(&Point3::new(0.0, 0.0, MUZZLE))
                    .coords,
            };
            // From its barrels in turn, if it has them.
            let barrel = self.barrels.get(self.barrel % self.barrels.len().max(1)).copied();
            self.barrel = self.barrel.wrapping_add(1);
            let muzzle = barrel.filter(|&barrel| graph.is_valid_handle(barrel)).map(|barrel| graph[barrel].global_position());
            doing.fired = Some(Fired {
                from,
                muzzles: vec![muzzle.unwrap_or(from)],
                size: self.kind.shot_size(),
                damage: self.kind.shot_damage(),
                colour: self.colour,
            });
        }
        doing
    }

    /// Where the animation it is playing is, in seconds.
    fn time(&self, graph: &Graph) -> f32 {
        let handle = self
            .animations
            .iter()
            .find(|(had, _)| had == self.playing)
            .map(|&(_, handle)| handle);
        handle
            .and_then(|handle| {
                let player = graph.try_get_of_type::<AnimationPlayer>(self.player).ok()?;
                Some(player.animations().try_get(handle).ok()?.time_position())
            })
            .unwrap_or(0.0)
    }

    /// Looks for the player and decides where to go (see [`Patrol::think`]). What it has to say,
    /// if anything.
    #[allow(clippy::too_many_arguments)]
    fn think(
        &mut self,
        graph: &Graph,
        mesh: &NavMesh,
        player: &Target,
        reachable: &dyn Fn(Vector3<f32>) -> bool,
        leaves: Option<&Graph>,
        rng: &mut Rng,
        dt: f32,
    ) -> Option<&'static str> {
        // On Alert it sees them at once; otherwise it becomes aware of them gradually, the more
        // quickly the more visible they are, and the more so searching.
        let visible = if self.mind.looks() { self.visibility(graph, player) } else { 0.0 };
        let sees = match self.mind.state {
            State::Alert if visible > 0.0 => {
                self.detection.alert();
                true
            }
            state => {
                let searching = if matches!(state, State::Search { .. }) { crate::AI.detection.searching } else { 1.0 };
                let eagerness = searching * if player.armed { crate::AI.detection.armed } else { 1.0 };
                self.detection.step(&crate::AI.detection, visible, eagerness, dt);
                self.detection.spotted() && visible > 0.0
            }
        };
        let air = Air { graph, hull: self.hull, leaves };
        let mut ways = Ways { mesh, air, reachable, rng, at: self.mind.at, flies_free: self.kind.flies_free() };
        self.mind.think(sees, player.feet, &mut ways, dt).map(said)
    }

    /// Flies on along its route for another `dt`, turning to face the way it goes - or the player,
    /// on Alert - and hovering over the floor (see [`Patrol::fly`]).
    fn fly(&mut self, graph: &Graph, mesh: &NavMesh, player: &Target, leaves: Option<&Graph>, dt: f32) {
        let around = Around { air: Air { graph, hull: self.hull, leaves }, mesh };
        self.mind.fly(player.feet, &around, dt);
    }

    /// Glows along with the animation it is playing, in its mood's colour, and lights what is
    /// round it as brightly.
    fn glow(&mut self, graph: &mut Graph, dt: f32) {
        let time = self.time(graph);
        let turned = 1.0 - (-MOOD_RATE * dt).exp();
        let own = self.glows.first().map_or(self.colour, |(_, own)| *own);
        self.colour += (mood_colour(own, self.mood()) - self.colour) * turned;
        let strength = glow(self.playing, time * FPS);
        // Its colour as well as its glow, as a droid's eyes: the eye and the inside of the shell
        // are made green, and would stay green under any glow.
        let colour = light_colour(self.colour);
        for (material, _) in &self.glows {
            let mut material = material.data_ref();
            material.set_property(DIFFUSE_COLOR, colour);
            material.set_property(
                EMISSION_STRENGTH,
                MaterialProperty::Vector3(self.colour * BRIGHTNESS * strength),
            );
        }
        // The lamp, in front of the eye wherever the animation has it, as bright as the glow.
        let at = match graph.is_valid_handle(self.eye) {
            true => graph[self.eye].global_position(),
            false => graph[self.body]
                .global_transform()
                .transform_point(&Point3::new(0.0, 0.0, LAMP_AT))
                .coords,
        };
        graph[self.lamp].local_transform_mut().set_position(at);
        if let Ok(lamp) = graph.try_get_mut_of_type::<PointLight>(self.lamp) {
            lamp.base_light_mut().set_intensity(LAMP_BRIGHTNESS * strength);
            lamp.base_light_mut().set_color(light_colour(self.colour));
        }
    }
}

/// Its body: a solid ball on a body moved by hand, which bumps into everyone and everything but
/// is not one of the characters, whom it looks out for as it flies (see [`blocked`]). The body,
/// and the ball.
fn hull(graph: &mut Graph) -> (Handle<Node>, Handle<Collider>) {
    let collider: Handle<Collider> = ColliderBuilder::new(BaseBuilder::new())
        .with_shape(ColliderShape::ball(BODY_RADIUS))
        .with_collision_groups(InteractionGroups::new(BitMask(!CHARACTERS), BitMask(u32::MAX)))
        .build(graph);
    let body = RigidBodyBuilder::new(BaseBuilder::new().with_child(collider))
        .with_body_type(RigidBodyType::KinematicPositionBased)
        .build(graph)
        .to_base();
    (body, collider)
}

/// The quadcopter's wreck: a body the shape of it, where it hovers `at`, facing `heading`, which
/// falls under physics - going on at `speed` the way it faces, knocked on along the level `away`
/// and up, and tipping over that way as it slews round. The body, and the box about its body.
fn wreck(graph: &mut Graph, at: Vector3<f32>, heading: f32, speed: f32, away: Vector3<f32>) -> (Handle<Node>, Handle<Collider>) {
    let ahead = Vector3::new(heading.sin(), 0.0, heading.cos());
    let part = |graph: &mut Graph, shape, y: f32, density: f32| -> Handle<Collider> {
        ColliderBuilder::new(BaseBuilder::new().with_local_transform(
            TransformBuilder::new().with_local_position(Vector3::new(0.0, y, 0.0)).build(),
        ))
        .with_shape(shape)
        .with_density(Some(density))
        .with_friction(WRECK_FRICTION)
        .with_restitution(WRECK_BOUNCE)
        .with_collision_groups(InteractionGroups::new(BitMask(!CHARACTERS), BitMask(u32::MAX)))
        .build(graph)
    };
    let (x, y, z) = CORE_HALF;
    let collider = part(graph, ColliderShape::cuboid(x, y, z), CORE_AT, 1.0);
    let rim = part(graph, ColliderShape::cylinder(RIM_HALF_HEIGHT, RIM_RADIUS), RIM_AT, RIM_DENSITY);
    let body = RigidBodyBuilder::new(
        BaseBuilder::new()
            .with_local_transform(
                TransformBuilder::new()
                    .with_local_position(at)
                    .with_local_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), heading))
                    .build(),
            )
            .with_child(collider)
            .with_child(rim),
    )
    .with_body_type(RigidBodyType::Dynamic)
    .with_ccd_enabled(true)
    .with_ang_damping(WRECK_DAMPING)
    .with_lin_vel(ahead * speed + away * KNOCK + Vector3::y() * KNOCK_UP)
    // About the level line square to the way it was knocked: its top goes over away.
    .with_ang_vel(Vector3::y().cross(&away) * TUMBLE + Vector3::y() * SLEW)
    .build(graph)
    .to_base();
    (body, collider)
}

/// Where a drone flies free: the scene, past its own body, `hull`; and the leaves it keeps out of
/// though they stop nothing else, if there are any (see [`Leaves`]).
#[derive(Clone, Copy)]
struct Air<'a> {
    graph: &'a Graph,
    hull: Handle<Node>,
    leaves: Option<&'a Graph>,
}

/// What a drone says about what came of what it did (see [`patrol::Event`]): the name of its
/// line for it in [`DRONE_LINES`].
fn said(event: patrol::Event) -> &'static str {
    match event {
        patrol::Event::Spotted => "spotted",
        patrol::Event::Searching => "searching",
        patrol::Event::Alarmed => "alarm",
        patrol::Event::Calm => "calm",
    }
}

/// A drone's ways about the level, for its mind (see [`hydroxus_ai::tactics::patrol::Ways`]):
/// along the floor of the `mesh`, where `reachable` says there is any way at all (see
/// [`ROUTE_REACH`]) - or, flying free, through the `air` from where it is, `at`.
struct Ways<'a, 'r> {
    mesh: &'a NavMesh,
    air: Air<'a>,
    reachable: &'a dyn Fn(Vector3<f32>) -> bool,
    rng: &'r mut Rng,
    at: Vector3<f32>,
    flies_free: bool,
}

impl Ways<'_, '_> {
    /// A way along the floor to `to`, if there is any: none looked for where there is none to be
    /// had.
    fn ground(&self, to: Vector3<f32>) -> Vec<Vector3<f32>> {
        match (self.reachable)(to) {
            true => walk::route_to(self.mesh, self.at - Vector3::new(0.0, HOVER, 0.0), to, ROUTE_REACH),
            false => Vec::new(),
        }
    }
}

impl patrol::Ways for Ways<'_, '_> {
    fn route_to(&mut self, to: Vector3<f32>) -> Vec<Vector3<f32>> {
        // Flying free, through the air to its height over `to`: there whether there is a way
        // along the floor or not.
        match self.flies_free {
            true => flight(self.air, self.at, to + Vector3::new(0.0, HOVER, 0.0), || self.ground(to)),
            false => self.ground(to),
        }
    }

    fn trip(&mut self, under: Vector3<f32>, reach: (f32, f32)) -> Vec<Vector3<f32>> {
        let route = walk::plan(self.mesh, under, reach, self.rng);
        // Flying free, through the air to its height over the end of it.
        match (self.flies_free, route.first()) {
            (true, Some(&end)) => flight(self.air, self.at, end + Vector3::new(0.0, HOVER, 0.0), || route),
            _ => route,
        }
    }

    fn spot_near(&mut self, post: Vector3<f32>, reach: f32) -> Option<Vector3<f32>> {
        walk::spot_near(self.mesh, post, reach, self.rng)
    }

    fn between(&mut self, range: (f32, f32)) -> f32 {
        inhabitants::between(self.rng, range)
    }
}

/// What is round a drone as it flies, for its mind (see [`hydroxus_ai::tactics::patrol::Air`]):
/// the `air` it flies through, and the floor of the `mesh` under it.
struct Around<'a> {
    air: Air<'a>,
    mesh: &'a NavMesh,
}

impl patrol::Air for Around<'_> {
    fn clear(&self, from: Vector3<f32>, to: Vector3<f32>) -> bool {
        clear(self.air, from, to, BODY_RADIUS)
    }

    fn floor_under(&self, at: Vector3<f32>) -> Option<f32> {
        walk::floor_under(self.mesh, at)
    }

    fn blocked(&self, from: Vector3<f32>, step: Vector3<f32>) -> bool {
        blocked(self.air.graph, self.air.hull, from, step)
    }
}

/// Whether a ball `radius` across its middle could fly from `from` to `to` through the `air`
/// without touching anything solid that stays where it is - the maze, the city - nor any leaves.
/// Not the characters, whom a drone looks out for as it flies (see [`blocked`]), nor anything
/// that moves: other drones, trains and lifts, the fallen.
fn clear(air: Air, from: Vector3<f32>, to: Vector3<f32>, radius: f32) -> bool {
    let way = to - from;
    if way.norm() < 1.0e-4 {
        return true;
    }
    let (ball, start) = (Ball::new(radius), Isometry3::translation(from.x, from.y, from.z));
    let through_leaves = air.leaves.is_none_or(|leaves| {
        leaves.physics.cast_shape(leaves, &ball, &start, &way, 1.0, false, Default::default()).is_none()
    });
    let Air { graph, hull, .. } = air;
    through_leaves
        && graph
        .physics
        .cast_shape(
            graph,
            &ball,
            &start,
            &way,
            1.0,
            false,
            QueryFilter {
                flags: QueryFilterFlags::EXCLUDE_KINEMATIC | QueryFilterFlags::EXCLUDE_DYNAMIC | QueryFilterFlags::EXCLUDE_SENSORS,
                exclude_rigid_body: Some(hull),
                predicate: Some(&|_, collider: &Collider| collider.collision_groups().memberships.0 != CHARACTERS),
                ..Default::default()
            },
        )
        .is_none()
}

/// The way a quadcopter flies from `from` to `to`, through the `air` at any height - past the
/// leaves, too - as a route, the last point first: [`hydroxus_ai::navigation::flight::flight`],
/// with [`clear`] saying what is in its way and `ground` its way along the floor, under a roof.
fn flight(
    air: Air,
    from: Vector3<f32>,
    to: Vector3<f32>,
    ground: impl FnOnce() -> Vec<Vector3<f32>>,
) -> Vec<Vector3<f32>> {
    hydroxus_ai::navigation::flight::flight(&crate::AI.flight, &|a, b, radius| clear(air, a, b, radius), from, to, HOVER, ground)
}

/// Whether the drone whose body is `hull`, its middle at `at`, would run into anyone - the player
/// or a droid - going `step`, or come nearer them than [`CLEARANCE`]. Going away from someone it
/// touches is never blocked, so it can always get clear.
fn blocked(graph: &Graph, hull: Handle<Node>, at: Vector3<f32>, step: Vector3<f32>) -> bool {
    let Some(way) = step.try_normalize(1.0e-6) else {
        return false;
    };
    graph
        .physics
        .cast_shape(
            graph,
            &Ball::new(BODY_RADIUS),
            &Isometry3::translation(at.x, at.y, at.z),
            &(way * (step.norm() + CLEARANCE)),
            1.0,
            false,
            QueryFilter {
                exclude_rigid_body: Some(hull),
                // The characters' capsules alone: the maze and most else are in every group,
                // the characters' among them.
                predicate: Some(&|_, collider: &Collider| {
                    collider.collision_groups().memberships.0 == CHARACTERS
                }),
                ..Default::default()
            },
        )
        .is_some()
}

/// Gives the drone its own copy of each material under `nodes` that glows, so that its glow can
/// change without changing the model's, with which way the colour of its glow goes, at strength 1.
pub(crate) fn claim_glows(graph: &mut Graph, nodes: &[Handle<Node>]) -> Vec<(MaterialResource, Vector3<f32>)> {
    let mut claimed: Vec<(u64, MaterialResource, Vector3<f32>)> = Vec::new();
    for &node in nodes {
        let Some(mesh) = graph[node].cast_mut::<Mesh>() else {
            continue;
        };
        for surface in mesh.surfaces_mut() {
            let original = surface.material().clone();
            let key = original.key();
            if let Some((_, copy, _)) = claimed.iter().find(|(had, _, _)| *had == key) {
                surface.set_material(copy.clone());
                continue;
            }
            let state = original.state();
            let Some(material) = state.data_ref() else {
                continue;
            };
            let colour = match glow_strength(material) {
                Some(MaterialProperty::Vector3(glow)) => glow / glow.max(),
                Some(_) => Vector3::new(0.0, 1.0, 0.0),
                None => continue,
            };
            let mut copy = material.clone();
            // Not metal: the engine lights a metal only by what it reflects, and puts out any glow
            // it has with the rest of its own colour. The model says nothing, which glTF takes
            // for all metal.
            copy.set_property(METALLIC_FACTOR, MaterialProperty::Float(0.0));
            let copy = MaterialResource::new_embedded(copy);
            drop(state);
            surface.set_material(copy.clone());
            claimed.push((key, copy, colour));
        }
    }
    claimed.into_iter().map(|(_, copy, colour)| (copy, colour)).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use hydroxus_ai::navigation::flight::{CLIMB_STEP, FLIGHT_CLEARANCE};
    use hydroxus_ai::tactics::patrol::wrap;

    #[test]
    fn its_green_glows_past_the_engines_threshold_whenever_it_is_lit() {
        // As the engine weighs green light (Rec. 709).
        let brightest = |strength: f32| 0.7152 * BRIGHTNESS * strength;
        for f in 0..97 {
            assert!(brightest(glow(IDLE, f as f32)) > 1.01, "idle at {f}");
        }
        assert!(brightest(glow(PATROL, 0.0)) > 1.01);
    }

    #[test]
    fn its_glow_follows_each_animation() {
        // Idle breathes between 0.9 and 1.5.
        let idle: Vec<f32> = (0..97).map(|f| glow(IDLE, f as f32)).collect();
        let (low, high) = idle.iter().fold((f32::MAX, f32::MIN), |(l, h), &g| (l.min(g), h.max(g)));
        assert!((low - 0.9).abs() < 0.01 && (high - 1.5).abs() < 0.01, "{low} {high}");
        assert_eq!(glow(PATROL, 30.0), 1.0);
        // Scan pulses at the end.
        assert_eq!(glow(SCAN, 40.0), 1.0);
        assert_eq!(glow(SCAN, 78.0), 5.0);
        assert_eq!(glow(SCAN, 81.0), 2.0);
        assert_eq!(glow(SCAN, 84.0), 5.0);
        assert_eq!(glow(SCAN, 96.0), 3.0);
        // Fire flashes.
        assert_eq!(glow(FIRE, 2.0), 8.0);
        assert_eq!(glow(FIRE, 14.0), 1.0);
        // Death flickers, then goes out.
        assert_eq!(glow(DEATH, 1.0), 4.0);
        assert_eq!(glow(DEATH, 3.0), 0.0);
        assert_eq!(glow(DEATH, 21.0), 4.0);
        assert_eq!(glow(DEATH, 40.0), 0.0);
        assert_eq!(glow(DEATH, 60.0), 0.0);
    }

    #[test]
    fn the_quadcopter_is_the_faster_drone() {
        let (security, quad) = (Kind::Security.flight(), Kind::Quad.flight());
        assert!(quad.patrol > security.patrol && quad.hurry > security.hurry);
        assert!(quad.acceleration > security.acceleration && quad.turn_rate > security.turn_rate);
        // Hostile, it outruns a running droid.
        assert!(quad.hurry > 3.5);
        // Its shots hurt half as much, three times as often: half as much again over time.
        assert_eq!(Kind::Quad.shot_damage(), 0.5 * Kind::Security.shot_damage());
        assert_eq!(Kind::Quad.fire_every(), 0.5);
        let over_time = |kind: Kind| kind.shot_damage() / kind.fire_every();
        assert!((over_time(Kind::Quad) / over_time(Kind::Security) - 1.5).abs() < 1.0e-4);
        assert_eq!(Kind::Quad.model(), QUAD_MODEL);
    }

    #[test]
    fn its_mood_colours_it_as_a_droids_eyes() {
        let green = Vector3::new(0.0, 1.0, 0.0);
        assert_eq!(mood_colour(green, Mood::Normal), green);
        let red = mood_colour(green, Mood::Hostile);
        assert!(red.x == 1.0 && red.y < 0.5 && red.z < 0.5, "{red}");
        let orange = mood_colour(green, Mood::Agitated);
        assert!(orange.x == 1.0 && orange.y > 0.3 && orange.y < 0.8, "{orange}");
        assert_eq!(light_colour(Vector3::new(1.0, 0.5, 0.0)), Color::opaque(255, 128, 0));
    }

    #[test]
    fn angles_come_round() {
        assert!((wrap(3.0 * PI / 2.0) + PI / 2.0).abs() < 1.0e-5);
        assert!((wrap(-3.0 * PI / 2.0) - PI / 2.0).abs() < 1.0e-5);
        assert!((wrap(0.5) - 0.5).abs() < 1.0e-6);
    }

    /// What it says, and when: hearing the alarm or a droid shot, spotting the player, starting
    /// to search where they were, giving up, and going down.
    const SAYS: [&str; 5] = ["alarm", "spotted", "searching", "calm", "down"];

    #[test]
    fn it_has_something_to_say_about_everything_it_does() {
        let lines = DroneLines::load(DRONE_LINES).expect("the drones' lines");
        for name in SAYS {
            let said = lines.line(name, 0).unwrap_or_else(|| panic!("nothing for {name}"));
            assert!(!said.says.is_empty() && !said.means.is_empty());
        }
        // Going round its lines in turn.
        let spotted = lines.lines["spotted"].len();
        assert_eq!(lines.line("spotted", spotted), lines.line("spotted", 0));
        assert_eq!(lines.line("nothing", 0), None);
        // Called in, it says what it says to the alarm.
        for why in CALLED {
            assert_eq!(lines.line(why, 0), lines.line("alarm", 0), "{why}");
        }
    }

    #[test]
    fn in_urbs_it_has_something_of_its_own_to_say_about_everything_and_why_it_was_called() {
        let urbs = DroneLines::load(URBS_DRONE_LINES).expect("Urbs's drones' lines");
        let maze = DroneLines::load(DRONE_LINES).unwrap();
        for name in SAYS.iter().chain(&CALLED) {
            let lines = &urbs.lines.get(*name).unwrap_or_else(|| panic!("nothing for {name}"));
            for said in lines.iter() {
                assert!(!said.says.is_empty() && !said.means.is_empty());
                let elsewhere = maze.lines.values().flatten().any(|other| other.says == said.says);
                assert!(!elsewhere, "{name}: {} is said in the maze too", said.says);
            }
        }
    }

    /// Someone standing with their feet at the origin, as the player's capsule is - 1.7 m tall,
    /// 0.35 m across - and a wall across the way 3 m ahead along z; and a drone's body hovering at `at`. The
    /// drone's body.
    fn standing_by(graph: &mut Graph, at: Vector3<f32>) -> Handle<Node> {
        use fyrox::scene::transform::TransformBuilder;
        let capsule = ColliderBuilder::new(BaseBuilder::new())
            .with_shape(ColliderShape::capsule_y(0.5, 0.35))
            .with_collision_groups(crate::ragdoll::character_groups())
            .build(graph);
        RigidBodyBuilder::new(
            BaseBuilder::new()
                .with_child(capsule)
                .with_local_transform(TransformBuilder::new().with_local_position(Vector3::new(0.0, 0.85, 0.0)).build()),
        )
        .with_body_type(RigidBodyType::KinematicPositionBased)
        .build(graph);
        let wall = ColliderBuilder::new(BaseBuilder::new()).with_shape(ColliderShape::cuboid(2.0, 2.0, 0.1)).build(graph);
        RigidBodyBuilder::new(
            BaseBuilder::new()
                .with_child(wall)
                .with_local_transform(TransformBuilder::new().with_local_position(Vector3::new(0.0, 1.0, 3.0)).build()),
        )
        .with_body_type(RigidBodyType::Static)
        .build(graph);
        let (body, _) = hull(graph);
        graph[body].local_transform_mut().set_position(at);
        // The physics has its colliders by the second update.
        for _ in 0..2 {
            graph.update(
                fyrox::core::algebra::Vector2::new(800.0, 600.0),
                1.0 / 60.0,
                Default::default(),
            );
        }
        body
    }

    #[test]
    fn a_drone_waits_for_someone_in_its_way_and_never_pushes_into_them() {
        let mut graph = Graph::new();
        let at = Vector3::new(-1.0, HOVER, 0.0);
        let body = standing_by(&mut graph, at);
        let toward = Vector3::new(0.05, 0.0, 0.0);
        // Far off, it flies on; up against them - 0.6 m from their middle, its body 4.5 cm from
        // the top of theirs, which curves down from the height it hovers at - it waits.
        assert!(!blocked(&graph, body, at, toward));
        assert!(blocked(&graph, body, Vector3::new(-0.6, HOVER, 0.0), toward));
        // It gets clear, and goes past them where there is room.
        assert!(!blocked(&graph, body, Vector3::new(-0.6, HOVER, 0.0), -toward));
        assert!(!blocked(&graph, body, Vector3::new(-0.6, HOVER, 0.0), Vector3::new(0.0, 0.0, -0.05)));
        // Over a crouching or crawling head it passes, and the maze is not someone.
        assert!(!blocked(&graph, body, Vector3::new(-0.6, 2.4, 0.0), toward));
        assert!(!blocked(&graph, body, Vector3::new(0.0, HOVER, 2.55), Vector3::new(0.0, 0.0, 0.05)));
    }

    #[test]
    fn a_downed_quadcopter_tumbles_down_and_comes_to_rest_flat_on_the_floor() {
        use fyrox::core::algebra::Vector2;
        let dt = 1.0 / 60.0;
        // Knocked every which way, still or flying flat out, facing any way.
        let falls = (0..24).map(|k| {
            let a = k as f32 * 0.55;
            (a * 2.3, if k % 2 == 0 { 0.0 } else { QUAD_FLIGHT.hurry }, Vector3::new(a.sin(), 0.0, a.cos()))
        });
        for (heading, speed, away) in falls {
            let mut graph = Graph::new();
            let floor = ColliderBuilder::new(BaseBuilder::new()).with_shape(ColliderShape::cuboid(30.0, 0.1, 30.0)).build(&mut graph);
            RigidBodyBuilder::new(
                BaseBuilder::new()
                    .with_local_transform(TransformBuilder::new().with_local_position(Vector3::new(0.0, -0.1, 0.0)).build())
                    .with_child(floor),
            )
            .with_body_type(RigidBodyType::Static)
            .build(&mut graph);
            let (body, _) = wreck(&mut graph, Vector3::new(0.0, HOVER, 0.0), heading, speed, away);
            let (mut tipped, mut lowest) = (0.0_f32, f32::MAX);
            for _ in 0..600 {
                graph.update(Vector2::new(800.0, 600.0), dt, Default::default());
                let body = &graph[body];
                let up = body.local_transform().rotation().transform_vector(&Vector3::y());
                tipped = tipped.max(up.y.abs().acos());
                lowest = lowest.min(body.local_transform().position().y);
            }
            let body = graph.try_get_of_type::<RigidBody>(body).unwrap();
            let up = body.local_transform().rotation().transform_vector(&Vector3::y());
            let at = **body.local_transform().position();
            // It tumbled, and lies flat, on its skids or on its back, still, on the floor.
            assert!(body.lin_vel().norm() < 0.05 && body.ang_vel().norm() < 0.1, "still moving: at {}, {} m/s, {} rad/s, up {up}", at.transpose(), body.lin_vel().transpose(), body.ang_vel().transpose());
            assert!(tipped > 0.5, "it fell without tumbling: {tipped}");
            assert!(up.y.abs() > 0.98, "it rests on its side: up {up}");
            // On its skids, or on its back on the top of its hump.
            let resting = if up.y > 0.0 { CORE_HALF.1 - CORE_AT } else { CORE_HALF.1 + CORE_AT };
            assert!((at.y - resting).abs() < 0.03, "at {at}, not on the floor: {resting}");
            // On its back is as low as it goes.
            assert!(lowest > CORE_HALF.1 + CORE_AT - 0.04, "it went through the floor: {lowest}");
        }
    }

    /// A scene of solid boxes, each its middle and its half-extents, as the physics has them.
    fn boxes(walls: &[(Vector3<f32>, Vector3<f32>)]) -> Graph {
        use fyrox::core::algebra::Vector2;
        let mut graph = Graph::new();
        for &(at, half) in walls {
            let wall = ColliderBuilder::new(BaseBuilder::new()).with_shape(ColliderShape::cuboid(half.x, half.y, half.z)).build(&mut graph);
            RigidBodyBuilder::new(
                BaseBuilder::new()
                    .with_local_transform(TransformBuilder::new().with_local_position(at).build())
                    .with_child(wall),
            )
            .with_body_type(RigidBodyType::Static)
            .build(&mut graph);
        }
        // The physics has its colliders by the second update.
        for _ in 0..2 {
            graph.update(Vector2::new(800.0, 600.0), 1.0 / 60.0, Default::default());
        }
        graph
    }

    /// Whether every leg of `route` - from `from`, the last point first - is clear.
    fn flown(graph: &Graph, from: Vector3<f32>, route: &[Vector3<f32>]) -> bool {
        let mut at = from;
        route.iter().rev().all(|&next| {
            let leg = clear(Air { graph, hull: Handle::NONE, leaves: None }, at, next, BODY_RADIUS);
            at = next;
            leg
        })
    }

    #[test]
    fn a_quadcopter_flies_straight_there_over_what_is_in_its_way_or_along_the_floor_under_a_roof() {
        let floor = (Vector3::new(0.0, -0.1, 0.0), Vector3::new(30.0, 0.1, 30.0));
        let (from, to) = (Vector3::new(0.0, HOVER, -8.0), Vector3::new(0.0, HOVER, 8.0));
        let nowhere = || -> Vec<Vector3<f32>> { panic!("no need of a way along the floor") };
        // Nothing in the way: straight there.
        let graph = boxes(&[floor]);
        assert_eq!(flight(Air { graph: &graph, hull: Handle::NONE, leaves: None }, from, to, nowhere), vec![to]);
        // A wall 5 m high across the way: up, over it with room to spare, and down.
        let wall = (Vector3::new(0.0, 2.5, 0.0), Vector3::new(20.0, 2.5, 0.2));
        let graph = boxes(&[floor, wall]);
        let route = flight(Air { graph: &graph, hull: Handle::NONE, leaves: None }, from, to, nowhere);
        assert_eq!(route.len(), 3, "{route:?}");
        assert_eq!(route[0], to);
        assert!(route[1].y > 5.0 + FLIGHT_CLEARANCE && route[1].y < 5.0 + FLIGHT_CLEARANCE + CLIMB_STEP + 0.01, "{route:?}");
        assert!(flown(&graph, from, &route));
        // A player up on a roof, with no way up to it along the floor: up onto it.
        let roof = (Vector3::new(0.0, 4.0, 8.0), Vector3::new(3.0, 4.0, 3.0));
        let graph = boxes(&[floor, roof]);
        let on_it = Vector3::new(0.0, 8.0 + HOVER, 8.0);
        let route = flight(Air { graph: &graph, hull: Handle::NONE, leaves: None }, from, on_it, nowhere);
        assert_eq!(route.first(), Some(&on_it));
        assert!(flown(&graph, from, &route), "{route:?}");
        // Under a roof 3 m up, the wall with a doorway at one end: along the floor, through it.
        let ceiling = (Vector3::new(0.0, 3.1, 0.0), Vector3::new(30.0, 0.1, 30.0));
        let wall = (Vector3::new(-2.0, 1.5, 0.0), Vector3::new(18.0, 1.5, 0.2));
        let graph = boxes(&[floor, ceiling, wall]);
        let door = Vector3::new(18.0, 0.0, 0.0);
        // As the walk grid has it: along the floor a step at a time, the last point first.
        let steps: Vec<Vector3<f32>> = (0..=36)
            .map(|k| k as f32 / 36.0 * 2.0)
            .map(|t| match t < 1.0 {
                true => (from - Vector3::new(0.0, HOVER, 0.0)).lerp(&door, t),
                false => door.lerp(&(to - Vector3::new(0.0, HOVER, 0.0)), t - 1.0),
            })
            .rev()
            .collect();
        let route = flight(Air { graph: &graph, hull: Handle::NONE, leaves: None }, from, to, || steps.clone());
        assert!(flown(&graph, from, &route), "{route:?}");
        // The wall ends 16 m along: the doorway is past it.
        assert!(route.iter().any(|at| at.x > 16.0 && at.z.abs() < 1.0), "not through the doorway: {route:?}");
        // Its corners cut: far fewer points than the steps along the floor.
        assert!(route.len() <= 8 && route.len() < steps.len() / 4, "{route:?}");
        assert_eq!(route.first().map(|at| (at - to).norm() < 1.0e-3), Some(true));
    }

    #[test]
    fn a_quadcopter_flies_over_leaves_that_stop_nothing_else() {
        let floor = (Vector3::new(0.0, -0.1, 0.0), Vector3::new(30.0, 0.1, 30.0));
        let (from, to) = (Vector3::new(0.0, HOVER, -8.0), Vector3::new(0.0, HOVER, 8.0));
        let nowhere = || -> Vec<Vector3<f32>> { panic!("no need of a way along the floor") };
        // A canopy across the way, from 1 m up to 6 m, in the leaves' world and not the scene's.
        let graph = boxes(&[floor]);
        let leaves = boxes(&[(Vector3::new(0.0, 3.5, 0.0), Vector3::new(4.0, 2.5, 4.0))]);
        let air = Air { graph: &graph, hull: Handle::NONE, leaves: Some(&leaves) };
        let route = flight(air, from, to, nowhere);
        assert_eq!(route.len(), 3, "straight through the leaves: {route:?}");
        assert!(route[1].y > 6.0 + FLIGHT_CLEARANCE, "{route:?}");
        // Without them, straight there.
        assert_eq!(flight(Air { leaves: None, ..air }, from, to, nowhere), vec![to]);
    }

    #[test]
    fn a_drone_is_solid() {
        let mut graph = Graph::new();
        let (_, collider) = hull(&mut graph);
        let ball = &graph[collider];
        assert!(!ball.is_sensor());
        let player = crate::ragdoll::character_groups();
        let groups = ball.collision_groups();
        // It and the player's capsule meet, but it is not one of the characters.
        assert!(groups.memberships.0 & player.filter.0 != 0 && player.memberships.0 & groups.filter.0 != 0);
        assert_eq!(groups.memberships.0 & CHARACTERS, 0);
    }

    /// How low the player's feet get, standing and maybe walking forward on a thin floor of the
    /// maze - a triangle mesh - for four seconds while a drone's body is moved by hand along
    /// `path` over time, holding still while it would run into anyone, as `Drone::update` moves
    /// it.
    fn lowest_feet(path: impl Fn(f32) -> Vector3<f32>, walk: bool) -> f32 {
        use crate::player::Player;
        use fyrox::core::algebra::{Matrix4, Vector2};
        use fyrox::scene::{
            collider::GeometrySource,
            mesh::{
                surface::{SurfaceBuilder, SurfaceData, SurfaceResource},
                MeshBuilder,
            },
            transform::TransformBuilder,
        };
        let mut graph = Graph::new();
        let mesh = MeshBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new().with_local_position(Vector3::new(0.0, -0.1, 0.0)).build(),
            ),
        )
        .with_surfaces(vec![SurfaceBuilder::new(SurfaceResource::new_embedded(SurfaceData::make_cube(
            Matrix4::new_nonuniform_scaling(&Vector3::new(20.0, 0.2, 20.0)),
        )))
        .build()])
        .build(&mut graph);
        graph.update_hierarchical_data();
        let floor = ColliderBuilder::new(BaseBuilder::new())
            .with_shape(ColliderShape::trimesh(vec![GeometrySource(mesh.to_base())]))
            .build(&mut graph);
        RigidBodyBuilder::new(BaseBuilder::new().with_child(floor))
            .with_body_type(RigidBodyType::Static)
            .build(&mut graph);
        let mut player = Player::spawn(&mut graph);
        player.teleport(&mut graph, Vector3::new(0.0, 0.86, 0.0), 0.0);
        if walk {
            player.on_key(fyrox::keyboard::KeyCode::KeyW, true);
        }
        let (hull, _) = hull(&mut graph);
        graph[hull].local_transform_mut().set_position(path(0.0));
        let dt = 1.0 / 60.0;
        let mut lowest = f32::MAX;
        for k in 0..240 {
            player.update(&mut graph, dt, true);
            let to = path(k as f32 * dt);
            let from = graph[hull].global_position();
            if !blocked(&graph, hull, from, to - from) {
                if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(hull) {
                    body.set_next_kinematic_translation(to);
                }
            }
            graph.update(Vector2::new(800.0, 600.0), dt, Default::default());
            lowest = lowest.min(player.feet(&graph).y);
        }
        lowest
    }

    #[test]
    fn a_drone_never_pushes_the_player_down_through_the_floor() {
        // Standing, with no drone near, the feet sink this far into the floor at most.
        let alone = lowest_feet(|_| Vector3::new(10.0, HOVER, 10.0), false);
        let cases: [(&str, Box<dyn Fn(f32) -> Vector3<f32>>, bool); 4] = [
            // Settling onto their head from above, as it would coming down to hover over a step,
            // or shot down overhead: moved by hand, it would press them down through the floor.
            ("sinking onto them", Box::new(|t| Vector3::new(0.0, (2.3 - t).max(0.3), 0.0)), false),
            ("flying through them", Box::new(|t| Vector3::new(-2.0 + 2.0 * t, HOVER, 0.0)), false),
            // Walking into it, under it, at its height and lower.
            ("walked into", Box::new(|_| Vector3::new(0.0, HOVER, 1.0)), true),
            ("walked into low", Box::new(|_| Vector3::new(0.0, 1.5, 0.6)), true),
        ];
        for (name, path, walk) in cases {
            let lowest = lowest_feet(path, walk);
            assert!(lowest > alone - 0.01, "{name}: down to {lowest} (alone {alone})");
        }
    }
}
