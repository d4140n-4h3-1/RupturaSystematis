//! The maze's inhabitants: droids like the player's, going about the maze by themselves.
//!
//! Each one wanders. It picks somewhere a good walk away, follows the cheapest route there over
//! the survey's grid - which keeps to the middle of the corridors (see
//! [`WalkGrid::routes_from`]) - and when it gets there stands idling a while before setting off
//! again. It walks at the droid's own walking pace, so its feet keep to the floor, and slows down
//! to turn.
//!
//! It keeps to floor it can walk: up and down stairs a step at a time, never off the edge of a
//! platform or a walkway, and where one floor is over another, on the one it is on - a point it
//! is going to is on the floor at that point's own height (see [`hydroxus_ai::trip`]). Where a ferry
//! gets it there sooner than walking - or there is no way round - it takes the ferry: waits for it
//! at one end, gets on as it waits, rides it across and gets off at the other end.
//!
//! Now and then one near one of the railway's stations takes a train (see [`crate::train`]) to another,
//! a few at a time: it climbs to the platform, waits beside a doorway no one else is waiting at,
//! and gets on once the doors are open; stands inside, looking out, as it rides; and gets off at
//! the station it is going to, and goes about its business there.
//!
//! They have bodies, so the player cannot walk through them, and they make way for each other.
//! One walking veers to its right round anyone ahead of it, so that two meeting head on pass
//! each other on the left, and to the left instead if a wall is in the way; one standing about
//! that sees another coming straight at it steps out of its way. Only someone right in front of
//! it stops it, and kept waiting too long, it goes somewhere else instead.
//!
//! Only those the player could see are drawn, but all of them cast their shadows, even round a
//! corner out of sight.
//!
//! Each is one of the kinds of droid in the conversations (see [`crate::dialogue`]), with a code
//! of its own, and can be talked to by a player close by and facing it. While it is, it stands
//! still and turns to face them, and afterwards stands a moment before going on its way. They
//! all start out of the player's sight, and wander - all but one, which stands a little way in
//! front of where the player starts, facing them, for a while before it sets off too.
//!
//! A conversation can turn a droid hostile (see [`Inhabitants::set_hostile`]), and a hostile
//! droid hunts the player the way Metal Gear's guards do, through its [`Alert`] phases:
//!
//! - **Alert**: it can see the player, and after a first moment runs at them, breaking into a
//!   sprint now and then (see [`hydroxus_ai::pace::SPRINT_EVERY`]), on breath spent and got back as the player's
//!   is: out of it, it walks until enough is back. A bar over its head shows how much it has
//!   left, as the player's does (see [`Inhabitants::breath`]). If it gets close enough to touch them, it
//!   has caught them.
//! - **Evasion**: it has lost them. It runs to where they were going when it last saw them, and
//!   looks about; then walks to one spot after another nearby, looking about at each, until
//!   [`EVASION`] seconds are up.
//! - **Caution**: it has given up, and wanders as before, but watching for the player still,
//!   for [`CAUTION`] seconds; then it is calm again, and can be talked to once more.
//!
//! A droid sees clearly what is in front of it, and out of the corner of its eye what is near off
//! to the side, in every phase - but nothing behind it, even right behind it: it can be crept up
//! on from behind, or slipped round (see [`Sight`]). Out of Alert it does not see as far
//! as it could: a player crouched is seen from half as far, and one crawling from less than a
//! third. With the lights off it sees a good deal less far in any phase - unless the player's
//! flashlight is on. Seeing the player again, it is back on
//! Alert, and a bolt from the pistol has it search where the shot came from.
//!
//! Out of Alert, it also listens (see [`Inhabitants::hear`]): a noise that carries as far as it
//! is, along the corridors, has it run to where the noise was and search from there.
//!
//! A sentry that sees another sentry after the player goes after them too (see
//! [`Inhabitants::join_chases`]), calm or not: the one it sees shows it where the player is. It
//! sees the other as it would see the player standing, and a sentry that sees one that has joined
//! in joins in as well, so a chase draws in every sentry that catches sight of it.
//!
//! Hostile, it cannot be talked to any more. After [`HITS`] bolts - hostile or not; a sentry,
//! tougher, [`SENTRY_HITS`] - it goes down, its eyes
//! dark: it goes limp and falls, knocked back by the last bolt, and lies where it falls (see
//! [`crate::ragdoll`]) - or, without a ragdoll, crouches. Lying there, a bolt still shoves it.
//! It stays there, for the rest of the round; the others step over it rather than round it.
//!
//! Those about that see the player shoot one - it, or the player - are witnesses (see
//! [`Inhabitants::witnesses`]), but only those that saw the player near enough, within
//! [`IDENTIFY_REACH`], know who did it: the rest saw a shooting, and no more. A sentry that knows
//! goes after them, and one that does not searches where it was done; a droid that would sound
//! the alarm runs to the nearest sentry instead, to tell it - stopping to be questioned on the way,
//! if the player stops it - and stands to be questioned by it, which then searches where it was
//! done with the sentries near it (see [`Inhabitants::report`]). Shot down before it gets there,
//! it tells no one. One the player shot down that no one saw is found by the first to see it lying there
//! (see [`Inhabitants::find_bodies`]): a sentry searches from there, and any other runs to tell
//! one. Shooting one that is after the player is self-defense: no one minds seeing it, nor finding
//! it lying there - unless it is a sentry, which the game takes as assault, whatever it was doing.
//!
//! A droid that is not hostile minds having the player's pistol pointed at it (see
//! [`Inhabitants::feel_aimed_at`]): for as long as its kind stands for it, and then it stops,
//! faces the player and warns them. Each warning then holds for [`hydroxus_ai::threat::WARNING`] seconds more, time
//! to say it and for the player to take it in, before the next: the last warning, and then
//! being provoked. What it does then is up to its kind: it turns hostile, or sounds the alarm
//! (see [`Inhabitants::raise_alarm`]) for those near it that would. Looking away only holds the
//! next stage off; with the pistol off it for [`CALM`] seconds, it calms down again.

pub use hydroxus_ai::alert::{Alert, CAUTION, EVASION};
pub use hydroxus_ai::threat::Threat;
pub(crate) use hydroxus_ai::route::{between, plan, route_to, spot_near};
use crate::{ferry::Crossing, roads::Roads, train::Doors};
use hydroxus_ai::{
    flat, forward, heading_of,
    group::{about_leader, ABOUT_LEADER, BESIDE},
    hearing::Heard,
    join::{could_see_other, joiners, Watch},
    pace::{pace, sprints},
    route::{plan_avoiding, spot_on_any_floor_near},
    search::SearchMap,
    sight::Sight,
    steer::{make_way, step_aside},
    threat::{pointed_at, stage_after, CALM},
    trip::{self, Doing, Trip},
};
use crate::{
    ctf::{self, Side},
    dismember,
    layout::{Rng, WalkGrid},
    level::Level,
    player::{
        avatar::{self, Avatar, Going},
        breath,
        pistol::Pass,
        posture::{Gait, Posture},
        Strike,
    },
    ragdoll::{self, Ragdoll},
    survey,
};
use std::collections::HashMap;
use fyrox::{
    core::{
        algebra::{Point3, Vector3},
        color::Color,
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    material::MaterialResource,
    resource::model::ModelResource,
    scene::{
        base::BaseBuilder,
        collider::{Collider, ColliderBuilder, ColliderShape},
        graph::{physics::RayCastOptions, Graph},
        mesh::Mesh,
        node::Node,
        rigidbody::{RigidBody, RigidBodyBuilder, RigidBodyType},
        transform::TransformBuilder,
        Scene,
    },
};

/// The droid in the colours any kind of droid takes on while it is after the player.
pub const HOSTILE_MODEL: &str = "data/droid_hostile.glb";
/// How many droids live in a maze, unless the conversations or MAZE_INHABITANTS say otherwise.
const COUNT: usize = 6;
/// A pair walking about together (see [`BESIDE`]): how far off where it keeps that the one
/// keeping beside its partner hurries to catch up; and how far behind the one leading lets it
/// get before waiting for it.
const CATCH_UP: f32 = 3.0;
/// How far ahead of where its partner is, in seconds of its walking, the one keeping beside it
/// makes for: where it will be by the time it gets there.
const LEAD: f32 = 1.0;
const WAIT_FOR: f32 = 6.0;
/// How often a droid is moved and posed in the scene, by how far it is from the player: within
/// each distance in meters, at most this often in seconds - every frame, near. A city has a
/// hundred droids or more, nearly all of them a long way off and small, and a droid's every bone
/// and part moving with it each frame is most of what they cost. Where it is goes on all the same.
const POSE_EVERY: [(f32, f32); 4] = [(35.0, 0.0), (80.0, 0.1), (160.0, 0.25), (f32::INFINITY, 0.5)];
/// How near the player, in meters, a droid's eyes light what is round them: further off, the
/// light they cast is too small to see, and a crowd's would be a hundred lights or more.
const EYE_LAMP_NEAR: f32 = 30.0;
/// How big the squares of the ground are, in meters, that droids are found near one another by:
/// no smaller than anyone makes way for another from (see [`hydroxus_ai::steer`]), so that all
/// of those near enough are in the square one is in or those round it.
const NEIGHBOURS: f32 = 3.0;
/// Their bodies: a capsule this far out from its middle line, with its middle this far above the
/// feet - 1.7 m tall, like the player's.
const RADIUS: f32 = 0.3;
const MIDDLE: f32 = 0.85;
/// How far from the player they are put down, in steps across the grid's half-meter cells
/// (see [`WalkGrid::routes_from`]): out of sight to begin with.
const AWAY_FROM_PLAYER: f32 = 24.0;
/// How far from the player the one put down near them is, from least to most, in the same steps;
/// how far to either side of the way they face, in radians, if there is room there; and how long
/// it stands there, in seconds, before it sets off.
const NEAR_PLAYER: (f32, f32) = (8.0, 14.0);
const NEAR_CONE: f32 = 30.0 * std::f32::consts::PI / 180.0;
const NEAR_REST: f32 = 20.0;
/// How far a droid goes each time it sets off, in steps across the grid, from least to most:
/// never just round the corner, never across the whole maze.
const TRIP: (f32, f32) = (30.0, 160.0);
/// How long it stands still between trips, in seconds, from least to most.
const REST: (f32, f32) = (2.0, 10.0);
/// How near the next point on its route it has to get, in meters, before making for the one
/// after: that much of every corner is cut.
const REACHED: f32 = 0.6;
/// How near the end of its route counts as there, in meters.
const ARRIVED: f32 = 0.1;
/// How quickly it gets up to speed and slows down, in meters per second per second.
const ACCELERATION: f32 = 3.0;
/// How long, in seconds, a droid waits for someone in its way before going somewhere else.
const PATIENCE: f32 = 3.0;
/// How quickly its feet follow the floor up and down, like
/// [`EYE_EASING`](crate::player::posture::EYE_EASING).
const FLOOR_EASING: f32 = 10.0;
/// Its walking pace, in meters per second, if the droid has no walk to go by.
const FALLBACK_PACE: f32 = 1.4;
/// How near the player's feet a droid's have to be, in meters, and how far off where the
/// player looks it can be, in radians, for the player to talk to it.
const TALK_REACH: f32 = 2.5;
const TALK_CONE: f32 = 40.0 * std::f32::consts::PI / 180.0;
/// Where a droid's face is above its feet, in meters, if its model has no head to go by.
const FACE_HEIGHT: f32 = 1.6;
/// Its jogging, running and sprinting paces, in meters per second, if the droid has no jog, run
/// or sprint to go by.
const FALLBACK_RUN: f32 = 2.0;
const FALLBACK_RUNNING: f32 = 2.8;
const FALLBACK_SPRINT: f32 = 3.5;
/// How long a droid that has just turned hostile stands before it goes after the player, in
/// seconds: long enough to finish its threat, and for the player to start running.
const WINDUP: f32 = 1.0;
/// How long after hearing something a droid pays no heed to another noise, in seconds, but to
/// go on to where that one was.
const HEARING_REST: f32 = 3.0;
/// How near a bolt has to fly past a droid's chest for it to take it as being shot at, in meters.
const NEAR_MISS: f32 = 1.0;
/// How far ahead of where it last saw the player it looks for them first, in meters, the way
/// they were going.
const GUESS: f32 = 4.0;
/// How long it looks about at each spot it searches, in seconds, and how far either way it turns
/// looking, in radians.
const LOOK_ABOUT: f32 = 3.0;
const LOOK_SWEEP: f32 = 70.0 * std::f32::consts::PI / 180.0;
/// How far each spot it searches is from the last, in steps across the grid, from least to most.
const SEARCH_TRIP: (f32, f32) = (10.0, 40.0);
/// How far above its feet, in meters, the middle of the view is taken to be pointed at a droid.
const CHEST: f32 = 1.2;
/// How far off, in meters, droids that answer an alarm hear it.
const ALARM_RANGE: f32 = 40.0;
/// How often a hostile droid that can see the player works out its way to them again, in
/// seconds.
const REPLAN: f32 = 0.4;
/// How far it looks for a way to the player, in steps across the grid's half-meter cells.
const CHASE_REACH: f32 = 200.0;
/// How far above or below where a droid is the floor can be and still be the floor it is on, in
/// meters: a stair's step, and a little more for its feet still coming down onto it.
const ON_FLOOR: f32 = 0.7;
/// In battle royale, how far from where it is a droid wanders, at most, in meters: near enough to
/// find the way there, upstairs in another building as likely as not.
const RING_WANDER: f32 = 35.0;
/// In capture the flag: how far from its post a droid wanders, and is put down, in meters; how long it keeps after one of the other side it has lost sight of, and
/// how long it keeps its pistol out once there is no one to shoot at, in seconds.
const POST_REACH: f32 = 3.0;
const SPAWN_REACH: f32 = 1.5;
const FOE_MEMORY: f32 = 6.0;
/// In capture the flag, how near one of the other side has to be for a droid to notice it even
/// out of sight of its eyes, in meters: a droid's footsteps, and a drone's hum, which carries.
const NOTICE_DROID: f32 = 3.0;
const NOTICE_RIVAL: f32 = 8.0;
const ARMED_FOR: f32 = 5.0;
/// In capture the flag, a droid shoots at whoever it sees of the other side - the player too,
/// for the enemy's - from as far as [`FIRE_RANGE`], in meters, coming on to [`ENGAGE`] and standing
/// there; once every so many seconds, as [`FIRE_EVERY`] says; wide by up to [`SPREAD`] meters
/// every ten meters away; and only facing within [`FIRE_CONE`] radians of them.
const FIRE_RANGE: f32 = 24.0;
const ENGAGE: f32 = 11.0;
const FIRE_EVERY: (f32, f32) = (0.7, 1.3);
/// A gang in a shootout is wilder: it comes on to [`GANG_ENGAGE`] meters before it stands to
/// shoot, and fires as often as [`GANG_FIRE_EVERY`] says.
const GANG_ENGAGE: f32 = 6.0;
const GANG_FIRE_EVERY: (f32, f32) = (0.45, 0.9);
const SPREAD: f32 = 0.55;
const FIRE_CONE: f32 = 0.3;
/// How high over the player's feet a droid aims.
const PLAYER_CHEST: f32 = 1.0;
/// How far out from the middle of the player's body it goes, standing, in meters (see
/// `crate::player::posture`).
const PLAYER_RADIUS: f32 = 0.35;
/// How near the player's feet, in meters, a hostile droid's have to get to hit them - their
/// bodies touching, give or take a few centimeters - and how far above or below; and how long,
/// in seconds, it stands after a hit before it comes on again.
const HIT_PAUSE: f32 = 1.2;
const CATCH: f32 = RADIUS + PLAYER_RADIUS + 0.05;
const CATCH_HEIGHT: f32 = 1.0;
/// How much breath a sentry has, to the player's 1: it is trained; and how fast it gets it back,
/// to the player's 1: it is heavier.
const SENTRY_STAMINA: f32 = 1.5;
const SENTRY_RECOVERY: f32 = 2.0 / 3.0;
/// A sentry paces itself after someone (see [`hydroxus_ai::pace`]) unless whoever it is after is
/// within [`BURST`] meters, near enough for all it has left to catch them.
const BURST: f32 = 6.0;
/// How many of the pistol's bolts it takes to stop a hostile droid, and a sentry, armoured.
pub const HITS: u32 = 3;
const SENTRY_HITS: u32 = 5;
/// The ragdoll body a good headshot has to go through, and how near the middle of it, as a share
/// of how far round it is: through the middle of the skull, not just clipping it.
const HEAD: &str = "head";
const SQUARELY: f32 = 0.6;
/// How tall a droid that has been stopped is, crouched, in meters.
const DOWN_HEIGHT: f32 = 1.1;

/// How far off, in meters, a droid sees the player shoot one of the others, to be a witness to
/// it (see [`Inhabitants::witnesses`]); and how far off one going about its business sees one
/// lying where the player shot it down, and how often they look, in seconds (see
/// [`Inhabitants::find_bodies`]).
const WITNESS_REACH: f32 = 30.0;
/// How near the player, in meters, a witness has to be to see who it is that shot: further off,
/// it sees a shooting but not who.
pub const IDENTIFY_REACH: f32 = 15.0;
const BODY_SIGHT: f32 = 15.0;
const BODY_LOOK_EVERY: f32 = 0.5;
/// How far off, in meters, a witness looks for a sentry to tell, and how near it has to get to
/// tell it; how long it hides, with none about, looking for one now and then, before it gives up,
/// and how often it looks, in seconds.
const REPORT_REACH: f32 = 150.0;
const REPORT_NEAR: f32 = 2.5;
const HIDE_FOR: f32 = 30.0;
const REPORT_LOOK_EVERY: f32 = 1.0;

/// Whether a witness at `feet` knows who it was that shot from `shooter`: whether it `sees` them,
/// near enough to tell who it is (see [`IDENTIFY_REACH`]).
fn knows_who(sees: bool, feet: Vector3<f32>, shooter: Vector3<f32>) -> bool {
    sees && flat(shooter - feet).norm() <= IDENTIFY_REACH
}

/// A droid that has seen what the player did, on its way to tell a sentry of it (see
/// [`Inhabitants::report`]). Shot down before it gets there, it tells no one.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Witness {
    /// Where it was done: where the player stood, or the one they shot down lies, for the sentry
    /// to search from.
    at: Vector3<f32>,
    /// Whether it knows who did it.
    knows: bool,
    /// The sentry it is making for, as an index, if it has found one; and how long until it
    /// looks for one again, or gives up, in seconds.
    to: Option<usize>,
    look_in: f32,
    left: f32,
}

/// What came of the witnesses' going to tell a sentry (see [`Inhabitants::report`]).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Reports {
    /// Those that have told a sentry: the witness, the sentry, where it was done, and whether
    /// the witness knows who did it.
    pub told: Vec<(usize, usize, Vector3<f32>, bool)>,
    /// Those that gave up, with no sentry to be found, and go about their business again.
    pub gave_up: Vec<usize>,
}

/// What came of moving everyone along.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct News {
    /// The droid that hit the player, if one did.
    pub hit: Option<usize>,
    /// The droids whose phase changed, as indices, and what to: none for calm again.
    pub alerts: Vec<(usize, Option<Alert>)>,
    /// The droids that heard the player, and are searching where.
    pub heard: Vec<usize>,
    /// The droids that answered an alarm, and are searching where the player was.
    pub alarmed: Vec<usize>,
    /// In capture the flag, the droids that have just gone after one of the other side; and the
    /// shots fired: from where, which way, one meter long, and by which side.
    pub engaged: Vec<usize>,
    pub shots: Vec<(Vector3<f32>, Vector3<f32>, Side)>,
}

/// How much breath a droid has left, for its stamina bar: where its feet and face are, how much
/// of its most it has left, from 0 to 1, and whether it has run out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Breath {
    pub feet: Vector3<f32>,
    pub face: Vector3<f32>,
    pub left: f32,
    pub winded: bool,
    /// How much of its health it has left, from 0 to 1: bolts take it.
    pub health: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct Inhabitant {
    body: Handle<Node>,
    collider: Handle<Collider>,
    avatar: Avatar,
    /// Where its feet are.
    feet: Vector3<f32>,
    /// How fast it is going along the ground, in meters per second.
    speed: f32,
    /// Which way it is making for, in radians, left positive from the world's +z.
    heading: f32,
    /// The rest of its route, as points on the floor, the next one last.
    route: Vec<Vector3<f32>>,
    /// Its way across by ferry, if it is taking one (see [`hydroxus_ai::trip`]).
    trip: Option<Trip>,
    /// Its ride on the train, if it is taking it.
    ride: Option<Ride>,
    /// How long it has left to stand still, in seconds.
    resting: f32,
    /// How long it has been kept waiting by someone in its way, in seconds.
    waiting: f32,
    /// Where its feet were the last time the graphics effects were told.
    last_seen: Option<Vector3<f32>>,
    /// Which kind of droid it is, as an index into the conversations' characters, and its code.
    character: usize,
    code: u32,
    /// Whether it is in the hostile droid's colours.
    looks_hostile: bool,
    /// Whether the player is talking to it - or, with `with`, the droid it is talking with: a
    /// witness questioned by a sentry, and the sentry. It stands where it is, facing whoever it
    /// is talking to.
    talking: bool,
    with: Option<usize>,
    /// How it is going about the player, once it has turned on them.
    alert: Option<Alert>,
    /// Whether, being hostile, it can see the player, as of this frame.
    sees_player: bool,
    /// Where it last saw the player's feet, and which way they were going then, roughly.
    lost_at: Vector3<f32>,
    lost_going: Vector3<f32>,
    /// How long it has left to search, or to stay wary, in seconds.
    search_left: f32,
    /// How many spots it has set off to search since it lost the player.
    searched: u32,
    /// How long it has left looking about where it is, in seconds, while it is, and which way it
    /// faced as it started.
    looking: Option<f32>,
    look_from: f32,
    /// How long before it heeds another noise, in seconds.
    deaf: f32,
    /// How long it has had the pistol pointed at it since it last warned the player, or since it
    /// first had, in seconds; how many times it has warned them; and how long the pistol has been
    /// off it since, in seconds.
    threat: f32,
    warned: u8,
    unaimed: f32,
    /// How long until it works out its way to the player again, in seconds.
    replan: f32,
    /// Whether there was no way there the last time it worked one out: it waits for `replan` to
    /// try again, rather than searching the whole level for it every frame.
    no_way: bool,
    /// How long, having just turned hostile, it stands before going after the player, in seconds.
    windup: f32,
    /// On Alert, how long until it sprints, in seconds - below 0 before it has been picked - and
    /// how long it has left sprinting.
    sprint_in: f32,
    sprinting: f32,
    /// Its breath, from 1 - or a sentry's [`SENTRY_STAMINA`] - down to 0, spent and got back as
    /// the player's is, but a sentry's back more slowly; and whether it has run out of it, and
    /// can only walk until enough is back.
    stamina: f32,
    winded: bool,
    /// How much breath it has at most: 1, or a sentry's more.
    most: f32,
    /// Whether, a sentry low on breath, it has eased off to a jog to get it back (see
    /// [`hydroxus_ai::pace::EASE_OFF`]).
    easing: bool,
    /// Whether it is a sentry.
    sentry: bool,
    /// How many of the pistol's bolts have hit it while hostile.
    hits: u32,
    /// Whether it has been stopped, and stays where it went down.
    down: bool,
    /// Its body gone limp, once it has been stopped, if it has a ragdoll.
    ragdoll: Option<Ragdoll>,
    /// Whether, stopped, it was the player who stopped it; and whether anyone has seen it done,
    /// or come across it lying there since, for whoever comes across it next to make nothing of.
    murdered: bool,
    found: bool,
    /// What it has seen the player do, if anything, on its way to tell a sentry.
    witness: Option<Witness>,
    /// In capture the flag, its side, and where it keeps to (see [`crate::ctf`]); whom of the
    /// other side it is after, if anyone, and how long since it last saw them, in seconds; and
    /// where it was last shot at from by someone it did not see, to go after, and for how much
    /// longer, in seconds.
    side: Option<Side>,
    post: Option<Vector3<f32>>,
    foe: Option<Foe>,
    foe_unseen: f32,
    shot_from: Option<(Vector3<f32>, f32)>,
    /// How long it has left speaking to the player, its head turned to them, in seconds.
    speaking: f32,
    /// How long before it can fire again, and before it puts its pistol away, in seconds; and
    /// what it last aimed at.
    reload: f32,
    armed_left: f32,
    aimed_at: Option<Vector3<f32>>,
    /// The droid it walks about with, if it is one of a pair, as an index; and whether it leads
    /// the way, or keeps beside the other.
    partner: Option<usize>,
    leads: bool,
    /// Keeping about the one leading its group, where (see [`ABOUT_LEADER`]).
    slot: usize,
    /// Its part in a foot chase, if it is in one (see [`Chase`]).
    pursuit: Option<Pursuit>,
    /// Where the shooting it is taking cover from is, a bystander to a shootout, and whether it
    /// has picked where to take cover yet (see [`Inhabitants::shootouts`]).
    cover: Option<(Vector3<f32>, bool)>,
    /// One of a group keeping to turf of its own: the spot it stands about, and how far round it
    /// is theirs, in meters. The player on it is as the pistol pointed at it, and the others
    /// keep off it.
    turf: Option<(Vector3<f32>, f32)>,
    /// How long since it was last posed, in seconds (see [`POSE_EVERY`]).
    unposed: f32,
}

impl Inhabitant {
    /// Whether it is getting on the train, on it or getting off it: off the grid's floor.
    fn aboard(&self) -> bool {
        self.ride.is_some_and(|ride| ride.aboard())
    }
}

impl Inhabitant {
    /// How many bolts it takes to stop it (see [`HITS`]).
    fn stopped_by(&self) -> u32 {
        match self.sentry {
            true => SENTRY_HITS,
            false => HITS,
        }
    }

    /// Whether it has anything to do with the player's being found: not one of the player's
    /// own side in capture the flag.
    fn hunts_player(&self) -> bool {
        self.side.is_none_or(|side| !side.is_players())
    }

    /// Puts it down afresh as another droid of its kind, as a crowd's droids turn up again (see
    /// [`Crowd`]): with its feet at `feet`, facing `heading`, to stand `resting` seconds before
    /// it sets off, under a code of its own, and with nothing of where it was or of the player.
    fn put_down(&mut self, graph: &mut Graph, feet: Vector3<f32>, heading: f32, resting: f32, rng: &mut Rng) {
        self.feet = feet;
        self.heading = heading;
        self.speed = 0.0;
        self.route.clear();
        self.ride = None;
        self.resting = resting;
        self.waiting = 0.0;
        // Not taken by the effects for having moved all that way in a frame.
        self.last_seen = None;
        self.code = 10 + rng.below(90) as u32;
        self.lost_at = feet;
        self.lost_going = Vector3::zeros();
        self.searched = 0;
        self.looking = None;
        self.deaf = 0.0;
        self.threat = 0.0;
        self.unaimed = 0.0;
        self.no_way = false;
        self.shot_from = None;
        self.cover = None;
        self.witness = None;
        // Posed where it is now on the next update, however far off: it is as long since it was
        // as the longest wait there is between.
        self.unposed = self.unposed.max(POSE_EVERY[POSE_EVERY.len() - 1].1);
        if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(self.body) {
            let middle = feet + Vector3::new(0.0, MIDDLE, 0.0);
            body.local_transform_mut().set_position(middle);
            body.set_next_kinematic_translation(middle);
        }
    }
}

impl Inhabitant {
    /// Breaks off the part of it lying there that a bolt struck the ragdoll body `body` of, if
    /// a hit there breaks anything off: its head, an arm at the shoulder or the elbow, a leg at
    /// the hip or the knee - a hand at the elbow, a foot at the knee.
    fn break_off(&mut self, graph: &mut Graph, body: &str) {
        let (Some(ragdoll), Some(part)) = (self.ragdoll.as_mut(), dismember::break_for(body))
        else {
            return;
        };
        if self.avatar.break_off(graph, part) {
            ragdoll.let_loose(graph, part);
        }
    }

    /// Where its head is and how far round, if a bolt going `way` that struck it at `at` went
    /// squarely through it: a good headshot.
    fn headshot(
        &self,
        graph: &Graph,
        at: Vector3<f32>,
        way: Vector3<f32>,
    ) -> Option<(Vector3<f32>, f32)> {
        ragdoll::struck_squarely(graph, self.avatar.root(), HEAD, at, way, SQUARELY)
    }

    /// Blows its head apart into voxels, wherever it is - on it or lying broken off - after a
    /// good headshot going `way`, its head `middle` and `radius` round. Whether it was there to.
    fn blow_head_up(
        &mut self,
        graph: &mut Graph,
        (middle, radius): (Vector3<f32>, f32),
        way: Vector3<f32>,
    ) -> bool {
        if !self.avatar.blow_up(graph, HEAD, middle, radius, way) {
            return false;
        }
        if let Some(ragdoll) = self.ragdoll.as_mut() {
            ragdoll.let_loose(graph, HEAD);
        }
        Log::info(format!("Headshot: head blown apart at {middle:?}"));
        true
    }
}

/// What a kind of droid looks like: the model it is made from, and which of that model's
/// materials the hostile droid's model has others in place of, with those, for while it is after
/// the player.
#[derive(Debug, Clone, PartialEq)]
pub struct Livery {
    model: ModelResource,
    hostile: Vec<(MaterialResource, MaterialResource)>,
    /// The colour its eyes are while it is hostile, in place of the usual red.
    hostile_eyes: Option<Color>,
}

impl Livery {
    /// A kind of droid made from `model`, loaded, that looks like the droid made from `hostile`
    /// while it is after the player - the same droid in other colours - or, without, like itself.
    pub fn new(model: ModelResource, hostile: Option<&ModelResource>) -> Self {
        let hostile = hostile.map_or_else(Vec::new, |hostile| changed_materials(&model, hostile));
        Self { model, hostile, hostile_eyes: None }
    }

    /// The same kind of droid, its eyes `colour` while it is hostile rather than the usual red:
    /// a droid already red all over shows up better with eyes of another colour.
    pub fn with_hostile_eyes(mut self, colour: Option<Color>) -> Self {
        self.hostile_eyes = colour;
        self
    }
}

/// How the droids of a kind walk about: in twos, as often as `in_pairs` says, from 0 to 1; or in
/// groups of two up to `groups_of`; and how often one of the kind is put down against the others,
/// its `share`. Of its groups, the share `turf` says keep to turf of their own, as far round as
/// it says, in meters (see [`Inhabitant::turf`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grouping {
    pub in_pairs: f32,
    pub groups_of: usize,
    pub share: f32,
    pub turf: Option<(f32, f32)>,
}

impl Default for Grouping {
    fn default() -> Self {
        Self { in_pairs: 0.0, groups_of: 0, share: 1.0, turf: None }
    }
}

/// Each of `from`'s materials that `to` has a different one in place of, mesh by mesh by name and
/// surface by surface, with the one in its place.
fn changed_materials(
    from: &ModelResource,
    to: &ModelResource,
) -> Vec<(MaterialResource, MaterialResource)> {
    let (from, to) = (from.data_ref(), to.data_ref());
    let (from, to) = (&from.get_scene().graph, &to.get_scene().graph);
    let mut changed: Vec<(MaterialResource, MaterialResource)> = Vec::new();
    for node in from.linear_iter() {
        let Some(mesh) = node.cast::<Mesh>() else {
            continue;
        };
        let Some((_, other)) = to.find_by_name_from_root(node.name()) else {
            continue;
        };
        let Some(other) = other.cast::<Mesh>() else {
            continue;
        };
        for (mine, theirs) in mesh.surfaces().iter().zip(other.surfaces()) {
            let (mine, theirs) = (mine.material(), theirs.material());
            if changed.iter().any(|(had, _)| had.key() == mine.key()) {
                continue;
            }
            let (mine_state, theirs_state) = (mine.state(), theirs.state());
            let differs = match (mine_state.data_ref(), theirs_state.data_ref()) {
                (Some(a), Some(b)) => a != b,
                _ => false,
            };
            if differs {
                changed.push((mine.clone(), theirs.clone()));
            }
        }
    }
    changed
}

/// Puts the droid whose model is under `root` in `livery`'s hostile colours, or back in its own.
fn recolour(graph: &mut Graph, root: Handle<Node>, livery: &Livery, hostile: bool) {
    let nodes: Vec<Handle<Node>> = graph.traverse_handle_iter(root).collect();
    for node in nodes {
        let Some(mesh) = graph[node].cast_mut::<Mesh>() else {
            continue;
        };
        for surface in mesh.surfaces_mut() {
            let key = surface.material().key();
            let swap = livery
                .hostile
                .iter()
                .find_map(|(own, theirs)| match hostile {
                    true => (own.key() == key).then_some(theirs),
                    false => (theirs.key() == key).then_some(own),
                });
            if let Some(material) = swap {
                surface.set_material(material.clone());
            }
        }
    }
}

/// What a droid walking about with a partner goes by, of the other.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Partner {
    feet: Vector3<f32>,
    heading: f32,
    speed: f32,
    /// Whether it is going about its business - calm, or wary - rather than down or hunting.
    about: bool,
    /// Whether it is kept where it is by the player: talking to them, or warning them off.
    held: bool,
}

/// Everyone who lives in the maze.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Inhabitants {
    droids: Vec<Inhabitant>,
    /// What each kind of droid looks like, by its index into the conversations' characters.
    liveries: Vec<Livery>,
    /// How each kind walks about, alone, in pairs or in groups, by the same index.
    groupings: Vec<Grouping>,
    /// The phases changed since the last update, as [`News::alerts`] has them, and those that
    /// heard the player.
    alerts: Vec<(usize, Option<Alert>)>,
    heard: Vec<usize>,
    alarmed: Vec<usize>,
    /// Whether they have been put into this round's maze yet.
    populated: bool,
    /// In battle royale, the ring everyone has to keep inside: its middle and how far it reaches,
    /// in meters (see [`crate::royale`]).
    ring: Option<(Vector3<f32>, f32)>,
    /// Where the player could be, since they were last seen, shared by everyone searching for
    /// them (see [`hydroxus_ai::search`]); and where and which way they were going when last seen,
    /// while a droid after them can see them.
    search: Option<SearchMap>,
    last_sight: Option<(Vector3<f32>, Vector3<f32>)>,
    /// Where a noise or an alarm has sent droids to search, to search from.
    search_from: Option<Vector3<f32>>,
    /// How long till the searchers next look over the map, in seconds.
    look_in: f32,
    /// In a city, where its droids can turn up again (see [`Inhabitants::recycle`]).
    crowd: Option<Crowd>,
    /// The foot chase going on, if there is one, and how long until the next is looked for, in
    /// seconds (see [`Inhabitants::foot_chases`]).
    chase: Option<Chase>,
    chase_in: f32,
    /// The shootout going on, if there is one, and how long until the next is looked for, in
    /// seconds (see [`Inhabitants::shootouts`]).
    shootout: Option<Shootout>,
    shootout_in: f32,
    /// How long till those going about their business next look for anyone the player has shot
    /// down, in seconds.
    body_look: f32,
    /// The trains, for those that take them.
    trains: Vec<Doors>,
    /// In a city, which of the grid's cells can be walked to from where the player started, a
    /// bit each by index (see [`Reach`]): where those going about their business may set off for.
    /// An office tower's floors, up its lift, are no part of it - nor anything else walled off -
    /// and a droid that picked a spot there would look for the way over every cell for a long
    /// way round before it gave up, a tenth of a second or more. Empty, anywhere.
    reachable: Reach,
}

/// Which of a grid's cells can be got to, a bit each by index - or, empty, all of them.
#[derive(Debug, Default, Clone, PartialEq)]
struct Reach(Vec<u64>);

impl Reach {
    fn of(cells: impl Iterator<Item = bool>) -> Self {
        let mut bits = Vec::new();
        for (i, can) in cells.enumerate() {
            if i % 64 == 0 {
                bits.push(0);
            }
            if can {
                *bits.last_mut().unwrap() |= 1 << (i % 64);
            }
        }
        Self(bits)
    }

    /// Whether the cell `index` can be got to.
    fn can(&self, index: usize) -> bool {
        self.0.is_empty() || self.0.get(index / 64).is_some_and(|bits| bits & (1 << (index % 64)) != 0)
    }

    /// Whether there is any way to `to` over `grid`, whose corner is at `origin`, at all: if not,
    /// a search for one would go over every cell it could within its bound before it gave up -
    /// for a droid's, the tenth part of a second and more.
    fn to(&self, (grid, origin): (&WalkGrid, Vector3<f32>), to: Vector3<f32>) -> bool {
        self.0.is_empty() || grid.walkable_cell(origin, to).is_some_and(|(x, z)| self.can(z * grid.width + x))
    }
}

/// A foot chase in a city: one of its security after one of a gang, which runs for it - and if
/// it is caught, stands for being held at pistol point a while.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Chase {
    guard: usize,
    thief: usize,
    /// How long it has gone on, or how long the thief has been held, in seconds.
    time: f32,
    /// What the thief says back to the guard, and in how long, in seconds: one after the other,
    /// never both at once.
    reply: Option<(f32, &'static str)>,
}

/// What came of seeing to the foot chases for a frame (see [`Inhabitants::foot_chases`]).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChaseNews {
    /// What those in a chase called out, by who, as the name of the bark - if the player is near
    /// enough to hear it: `chase` as it starts, and the one running `flee` back; `caught` when
    /// the one running is caught, and `caught` back; and `escaped` when it gets away.
    pub said: Vec<(usize, &'static str)>,
    /// A chase begun, and one over, however it ended: the one after the other, and the one
    /// running.
    pub started: Option<(usize, usize)>,
    pub ended: Option<(usize, usize)>,
}

/// A droid's part in a [`Chase`], with the other's index.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Pursuit {
    /// After the other, which runs from it.
    Chasing(usize),
    /// Running from the other, which is after it.
    Fleeing(usize),
    /// Holding the other, caught, at pistol point.
    Holding(usize),
    /// Caught by the other, standing for it.
    Held(usize),
}

/// How long between one foot chase and the next, in seconds, from least to most; and how soon
/// another is looked for when there was no one about for one.
const CHASE_EVERY: (f32, f32) = (40.0, 100.0);
const CHASE_RETRY: f32 = 5.0;
/// How long into a round the first foot chase is looked for, in seconds.
const CHASE_FIRST: f32 = 20.0;
/// How near the player both of a chase are when it starts, in meters along the ground, for it to
/// be seen; and how far apart they are, from least to most.
const CHASE_SEEN: f32 = 60.0;
const CHASE_APART: (f32, f32) = (8.0, 30.0);
/// How near the one after the other has to get, in meters, to catch it; how far off it has to
/// get to be away; and how long the chase goes on at most, in seconds, before it is given up.
const CHASE_CAUGHT: f32 = 1.5;
const CHASE_ESCAPED: f32 = 40.0;
const CHASE_FOR: f32 = 45.0;
/// How far ahead the one running makes for, in meters, each time it looks again where to go:
/// away from the one after it.
const FLEE_AHEAD: f32 = 25.0;
/// How long one caught is held at pistol point, in seconds.
const HELD_FOR: f32 = 6.0;
/// How long after the guard the thief says what it has to say back, in seconds.
const REPLY_AFTER: f32 = 1.8;
/// How near the player, in meters, what those in a chase call out is heard.
const CHASE_HEARD: f32 = 40.0;

/// A shootout in a city: six of its security against a gang of six, where the gang keeps to.
#[derive(Debug, Clone, PartialEq)]
struct Shootout {
    /// Where the gang keeps to, and the security called in comes to - once it has met up where
    /// the nearest of it is, out of the gang's sight, if it has not gone in yet.
    site: Vector3<f32>,
    rally: Option<Vector3<f32>>,
    guards: Vec<usize>,
    gang: Vec<usize>,
    /// How long it has gone on, in seconds.
    time: f32,
    /// Whether each side, security's and then the gang's, has called out going after the other.
    engaged: [bool; 2],
}

/// What came of seeing to the shootouts for a frame (see [`Inhabitants::shootouts`]).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ShootoutNews {
    /// What those in it called out, by who, as the name of the bark - if the player is near
    /// enough to hear it: `shootout` as security calls the rest of it in; `engaged` as the first
    /// of each side goes after the other; and `won` by one of the side left standing.
    pub said: Vec<(usize, &'static str)>,
    /// Everyone in a shootout begun, and those left standing of one over - and those taking
    /// cover from it, once it is - and those that have just run for cover.
    pub started: Vec<usize>,
    pub ended: Vec<usize>,
    pub sheltering: Vec<usize>,
}

/// How many of each side are in a shootout.
const SHOOTOUT_SIDE: usize = 6;
/// A group keeping to its turf: how far from the spot the one leading it wanders, in meters; how
/// long it stands each time, in seconds, from least to most; and how much more a step onto it
/// costs the others than one along the pavement, so that their routes go round it.
const TURF_KEEP: f32 = 2.5;
const TURF_REST: (f32, f32) = (6.0, 15.0);
const TURF_COST: f32 = 3.0;

/// How long between one shootout and the next, in seconds, from least to most; how soon another
/// is looked for when there was no one about for one; and how long into a round the first is.
const SHOOTOUT_EVERY: (f32, f32) = (150.0, 300.0);
const SHOOTOUT_RETRY: f32 = 10.0;
const SHOOTOUT_FIRST: f32 = 60.0;
/// How far from the player, in meters along the ground, the gang keeps to: near enough to be
/// heard, not so near the player is in the middle of it.
const SHOOTOUT_SEEN: (f32, f32) = (25.0, 70.0);
/// How far from there, in meters, those in it are called from at most; and how near any of
/// security can already be, so that they come to it rather than start in the middle of it.
const SHOOTOUT_CALLED: f32 = 90.0;
const SHOOTOUT_APART: f32 = 35.0;
/// How near where security meets up all of it has to be, in meters, for it to go in together,
/// rather than one at a time to be picked off; and how long it waits for the rest at most, in
/// seconds.
const RALLY_NEAR: f32 = 6.0;
const RALLY_FOR: f32 = 30.0;
/// How near the shooting, in meters, those going about their business run for cover from it; how
/// far they run, about; and how many ways off they try for somewhere it cannot be seen from.
const COVER_NEAR: f32 = 45.0;
const COVER_RUN: f32 = 20.0;
const COVER_TRIES: usize = 8;
/// How long it goes on at most, in seconds, and how far off the player can go, in meters, before
/// it is over all the same.
const SHOOTOUT_FOR: f32 = 120.0;
const SHOOTOUT_GONE: f32 = 130.0;

/// A city's crowd: only so many droids, all of them about the player. One that has gone far off,
/// out of sight, turns up again as another nearer by, somewhere the player cannot see, so that
/// wherever they go the streets are as busy - busier than the whole city's hundreds spread over
/// it would make them - and only the droids near enough to matter cost anything.
#[derive(Debug, Default, Clone, PartialEq)]
struct Crowd {
    /// Whether each cell of the walk grid, row by row, is somewhere a droid can turn up: off the
    /// roads, and where the player started from could be walked to.
    pavement: Vec<bool>,
    /// How long till the crowd is next looked over, in seconds.
    next: f32,
}

impl Crowd {
    /// Somewhere on the floor of `grid` whose corner is at `origin` that one of the crowd could
    /// have walked to, a little way off the `player`'s feet, where `hidden` says the player cannot
    /// see it turn up; and for a group, room for the rest about it, keeping to `slots` of
    /// [`ABOUT_LEADER`]: where, which way it faces, and where the rest are.
    #[allow(clippy::type_complexity)]
    fn spot(
        &self,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        player: Vector3<f32>,
        slots: &[usize],
        rng: &mut Rng,
        hidden: impl Fn(Vector3<f32>) -> bool,
    ) -> Option<(Vector3<f32>, f32, Vec<Vector3<f32>>)> {
        (0..CROWD_TRIES).find_map(|_| {
            let angle = between(rng, (-std::f32::consts::PI, std::f32::consts::PI));
            let away = between(rng, CROWD_BACK);
            let at = player + forward(angle) * away;
            let cell = survey::cell_at(grid, origin, at)?;
            let pavement = |(x, z): (usize, usize)| self.pavement.get(z * grid.width + x).copied().unwrap_or(false);
            if !pavement(cell) {
                return None;
            }
            let feet = grid.on_floor(origin, cell);
            if !hidden(feet) {
                return None;
            }
            let heading = between(rng, (-std::f32::consts::PI, std::f32::consts::PI));
            let about: Option<Vec<Vector3<f32>>> = slots
                .iter()
                .map(|&slot| {
                    let cell = survey::cell_at(grid, origin, about_leader(feet, heading, slot)).filter(|&cell| pavement(cell))?;
                    Some(grid.on_floor(origin, cell))
                })
                .collect();
            Some((feet, heading, about?))
        })
    }
}

/// How far from the player, in meters along the ground, a droid of a crowd that is going about
/// its business and cannot be seen is taken out of the way (see [`Crowd`]).
const CROWD_GONE: f32 = 110.0;
/// How far from the player, in meters along the ground, it turns up again, from least to most -
/// and how far the crowd is put down to begin with, at most.
const CROWD_BACK: (f32, f32) = (45.0, 85.0);
/// How often the crowd is looked over, in seconds, and how many of them turn up again each time
/// at most, so that a crowd left behind all at once comes back over a few looks.
const CROWD_EVERY: f32 = 0.5;
const CROWD_AT_ONCE: usize = 6;
/// How long one of a crowd stands still between trips, in seconds, from least to most: not as
/// long as others (see [`REST`]), so that more of them are on the move.
const CROWD_REST: (f32, f32) = (1.0, 4.0);
/// How many spots it tries for each droid turning up again before leaving it for the next look.
const CROWD_TRIES: usize = 16;
/// How far off where the player looks, in radians, is out of view whatever is in the way: a
/// little wider than half of the widest view.
const OUT_OF_VIEW: f32 = 75.0 * std::f32::consts::PI / 180.0;

/// How fast the player is taken to get away, in meters per second, for where they could be.
const PLAYER_SPEED: f32 = 4.0;
/// How far a searcher is taken to see, in meters, for clearing the map - no further than it sees
/// at all - and how often it looks it over, in seconds.
const SEARCH_SIGHT: f32 = 16.0;
const LOOK_EVERY: f32 = 0.25;
/// How far apart, in meters, the searchers keep where they are going.
const SEARCH_APART: f32 = 4.0;
/// How far ahead of the player, at most, a droid after them that is not the nearest makes for, to
/// cut them off, in meters; and how near it has to be to go straight at them instead.
const CUT_OFF: f32 = 10.0;
const CLOSE_IN: f32 = 4.0;
/// Taking a train: how near a station a droid has to be, setting off somewhere, to think of it,
/// and how likely it is to then, one in so many; how many can be taking each train at once; and how
/// long one waits on the platform for it at most, in seconds.
const TRAIN_REACH: f32 = 150.0;
const TRAIN_CHANCE: usize = 3;
const RIDERS: usize = 8;
const TRAIN_WAIT: f32 = 200.0;
/// How far out on the platform from a doorway it waits, and how far in from it it stands aboard,
/// in meters; and how near where it waits it has to be to be there.
const PLATFORM_OUT: f32 = 1.1;
const STAND_IN: f32 = 0.8;
const AT_PLATFORM: f32 = 1.2;
/// How far out from the middle of the train one getting off has to be, in meters, to be clear of
/// its doorway: off the sill, its side past the car's.
const CLEAR_OF_DOORS: f32 = 3.0;
/// How fast it gets on and off, at least, in meters per second, and how long the doors have to
/// stay open for it to set off to, on top of the time that takes, in seconds.
const BOARDING_PACE: f32 = 1.0;
const TIME_TO_SPARE: f32 = 1.0;

/// A droid's ride on a train: from one station to another.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ride {
    /// Which train it takes, as the game counts them.
    train: usize,
    /// Which stations it gets on at and off at, as the train counts them.
    from: usize,
    to: usize,
    /// Which of the train's doorways it gets on and off by, and where it stands in the car, from
    /// the middle of the beam's top under the train.
    doorway: usize,
    spot: Vector3<f32>,
    /// Where it waits on the platform it gets on from, and where it goes on the one it gets off on.
    wait: Vector3<f32>,
    off: Vector3<f32>,
    stage: Riding,
    /// How long it has been waiting on the platform, in seconds.
    waited: f32,
}

/// Where along the way a droid taking the train is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Riding {
    /// On its way up to the platform.
    ToPlatform,
    /// Waiting there for the doors to open.
    Waiting,
    /// Getting on, the doors open.
    Boarding,
    /// On, standing where it stands.
    Aboard,
    /// Getting off, at the station it is going to.
    Leaving,
}

/// What a droid taking the train does this frame.
#[derive(Debug, Clone, PartialEq)]
enum Rides {
    /// Goes its own way, on to the platform.
    Own,
    /// Goes where the route says, the next point last: getting on or off, or nowhere.
    Go(Vec<Vector3<f32>>),
    /// Is done with the ride, or gives it up.
    Off,
}

impl Ride {
    /// Whether it is off the grid's floor for the train: getting on, on, or getting off.
    fn aboard(&self) -> bool {
        matches!(self.stage, Riding::Boarding | Riding::Aboard | Riding::Leaving)
    }

    /// Moves the ride on for a droid at `feet` with `route` ahead of it - on its way to the
    /// platform - the train as `train` has it, after `dt`.
    fn step(&mut self, feet: Vector3<f32>, route: &[Vector3<f32>], train: &Doors, dt: f32) -> Rides {
        let doorway = train.at + train.doorways[self.doorway];
        let spot = train.at + self.spot;
        let within = train.within(feet);
        // Whether the doors are open at `station`, and stay open long enough for it to get to `to`.
        let in_time = |station: usize, to: Vector3<f32>| {
            let takes = flat(to - feet).norm() / BOARDING_PACE + TIME_TO_SPARE;
            train.open.is_some_and(|(at, left)| at == station && left > takes)
        };
        let shut = train.open.is_none();
        let waiting = flat(self.wait - feet).norm() < AT_PLATFORM;
        // Kept aboard, the doors shut: it stands where it is till they open again.
        let stay = |ride: &mut Self| {
            ride.spot = Vector3::new(feet.x - train.at.x, train.doorways[ride.doorway].y, feet.z - train.at.z);
            ride.stage = Riding::Aboard;
            Rides::Go(Vec::new())
        };
        match self.stage {
            Riding::ToPlatform if waiting => {
                self.stage = Riding::Waiting;
                Rides::Go(Vec::new())
            }
            Riding::ToPlatform if route.is_empty() => Rides::Off,
            Riding::ToPlatform => Rides::Own,
            Riding::Waiting => {
                self.waited += dt;
                if self.waited > TRAIN_WAIT {
                    Rides::Off
                } else if in_time(self.from, spot) {
                    self.stage = Riding::Boarding;
                    Rides::Go(vec![spot, doorway])
                } else {
                    Rides::Go(if waiting { Vec::new() } else { vec![self.wait] })
                }
            }
            Riding::Boarding if within && flat(spot - feet).norm() < 2.0 * ARRIVED => {
                self.stage = Riding::Aboard;
                Rides::Go(Vec::new())
            }
            Riding::Boarding if shut && within => stay(self),
            Riding::Boarding if shut => {
                // Too late: back to waiting for it to come round again.
                self.stage = Riding::Waiting;
                Rides::Go(vec![self.wait])
            }
            Riding::Boarding => Rides::Go(if within { vec![spot] } else { vec![spot, doorway] }),
            Riding::Aboard if in_time(self.to, self.off) => {
                self.stage = Riding::Leaving;
                Rides::Go(vec![self.off, doorway])
            }
            Riding::Aboard => Rides::Go(Vec::new()),
            Riding::Leaving
                if train.out(feet).abs() > CLEAR_OF_DOORS && flat(self.off - feet).norm() < AT_PLATFORM =>
            {
                Rides::Off
            }
            // Not off in time: on round again.
            Riding::Leaving if within && shut => stay(self),
            Riding::Leaving => Rides::Go(if within { vec![self.off, doorway] } else { vec![self.off] }),
        }
    }
}

/// A ride on one of `trains` for a droid at `feet`, over `grid` whose corner is at `origin`: on the
/// train with a station nearest it, near enough to think of, that has room - fewer than [`RIDERS`]
/// and a doorway no one is taking it by, those `taken` as (train, doorway) - from that station to
/// another of its stations: the way up to the platform beside the doorway, and the ride.
fn take_train(
    (grid, origin): (&WalkGrid, Vector3<f32>),
    feet: Vector3<f32>,
    trains: &[Doors],
    taken: &[(usize, usize)],
    rng: &mut Rng,
) -> Option<(Vec<Vector3<f32>>, Ride)> {
    let away = |train: &Doors, n: usize| flat(train.station(n) - feet).norm();
    let room = |k: usize| {
        let riding = taken.iter().filter(|&&(t, _)| t == k).count();
        riding < RIDERS && (0..trains[k].doorways.len()).any(|n| !taken.contains(&(k, n))) && trains[k].stations() >= 2
    };
    // Each train's station nearest it, nearest first; the first of those with room.
    let mut near: Vec<(usize, usize, f32)> = (0..trains.len())
        .filter_map(|k| {
            let train = &trains[k];
            let n = (0..train.stations()).min_by(|&a, &b| away(train, a).total_cmp(&away(train, b)))?;
            Some((k, n, away(train, n)))
        })
        .filter(|&(_, _, d)| d <= TRAIN_REACH)
        .collect();
    near.sort_by(|a, b| a.2.total_cmp(&b.2));
    let (which, from, _) = near.into_iter().find(|&(k, _, _)| room(k))?;
    let train = &trains[which];
    let free: Vec<usize> = (0..train.doorways.len()).filter(|&n| !taken.contains(&(which, n))).collect();
    let to = (from + 1 + rng.below(train.stations() - 1)) % train.stations();
    let doorway = free[rng.below(free.len())];
    let wait = trip::floor_near(grid, origin, train.platform(from, doorway, PLATFORM_OUT))?;
    let off = trip::floor_near(grid, origin, train.platform(to, doorway, PLATFORM_OUT))?;
    let route = route_to((grid, origin), feet, wait, CHASE_REACH);
    if route.is_empty() {
        return None;
    }
    let inward = -train.doorways[doorway].dot(&train.across).signum();
    let spot = train.doorways[doorway] + train.across * inward * STAND_IN + train.along * between(rng, (-0.5, 0.5));
    Some((route, Ride { train: which, from, to, doorway, spot, wait, off, stage: Riding::ToPlatform, waited: 0.0 }))
}

/// The square of the ground `at` is in (see [`NEIGHBOURS`]).
fn square_of(at: Vector3<f32>) -> (i32, i32) {
    ((at.x / NEIGHBOURS).floor() as i32, (at.z / NEIGHBOURS).floor() as i32)
}

/// As [`plan`], a trip for a droid going about its business from `feet` - but keeping off the
/// `roads`, if there are any, and off the `turfs` of the groups that keep to one, each a cell and
/// how many cells round it is theirs: to somewhere off them a `trip` away as the crow flies, the
/// way that crosses them least (see [`Roads::extra`], [`plan_avoiding`]). One that finds itself
/// on a road, as it might after a chase, gets off it the shortest way. Empty, with nowhere found
/// to go.
fn wander(
    (grid, origin): (&WalkGrid, Vector3<f32>),
    feet: Vector3<f32>,
    trip: (f32, f32),
    roads: Option<&Roads>,
    turfs: &[((usize, usize), f32)],
    reachable: &Reach,
    rng: &mut Rng,
) -> Vec<Vector3<f32>> {
    let Some(roads) = roads else {
        return plan((grid, origin), feet, trip, rng);
    };
    let on_road = survey::cell_at(grid, origin, feet).is_some_and(|(x, z)| roads.is_road(x, z));
    // On a group's turf: within as many cells as it reaches of the cell its spot is in.
    let on_turf = |x: usize, z: usize| {
        turfs.iter().any(|&((tx, tz), reach)| {
            let (dx, dz) = (x as f32 - tx as f32, z as f32 - tz as f32);
            dx * dx + dz * dz < reach * reach
        })
    };
    plan_avoiding(
        (grid, origin),
        feet,
        trip,
        |x, z| roads.is_road(x, z) || on_turf(x, z) || !reachable.can(z * grid.width + x),
        |x, z| {
            let road = if on_road { 0.0 } else { roads.extra(x, z) };
            road + if on_turf(x, z) { TURF_COST } else { 0.0 }
        },
        rng,
    )
}

/// Whether nothing is in the way of `watcher` seeing `seen`: a ray from its face to the other's
/// chest meets the other before anything else - aside from its own body, which the ray starts in.
fn in_sight_of(graph: &Graph, watcher: &Inhabitant, seen: &Inhabitant) -> bool {
    let from = watcher.feet + Vector3::new(0.0, FACE_HEIGHT, 0.0);
    let way = seen.feet + Vector3::new(0.0, CHEST, 0.0) - from;
    let length = way.norm();
    if length < 1.0e-3 {
        return true;
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
    hits.iter()
        .find(|hit| hit.collider != watcher.collider)
        .is_none_or(|hit| hit.collider == seen.collider)
}

/// Whether nothing is in the way from `from` to `to` but what `passes`.
fn clear_between(
    graph: &Graph,
    from: Vector3<f32>,
    to: Vector3<f32>,
    passes: impl Fn(Handle<Collider>) -> bool,
) -> bool {
    let way = to - from;
    let length = way.norm();
    if length < 1.0e-3 {
        return true;
    }
    let mut hits = Vec::new();
    graph.physics.cast_ray(
        RayCastOptions {
            ray_origin: Point3::from(from),
            ray_direction: way / length,
            max_len: length,
            groups: Default::default(),
            sort_results: false,
        },
        &mut hits,
    );
    hits.iter().all(|hit| passes(hit.collider))
}

/// Someone of the other side in capture the flag who is not one of the droids - a drone: where it
/// is over the floor, the middle of its body, and its body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rival {
    pub side: Side,
    pub feet: Vector3<f32>,
    pub middle: Vector3<f32>,
    pub collider: Handle<Collider>,
}

/// Whom of the other side a droid in capture the flag is after: one of the droids, or a
/// [`Rival`], by its body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Foe {
    Droid(usize),
    Rival(Handle<Collider>),
}

/// Whether nothing is in the way from `watcher`'s eyes to `at`, but the body `seen`.
fn in_sight_of_point(graph: &Graph, watcher: &Inhabitant, at: Vector3<f32>, seen: Handle<Collider>) -> bool {
    let from = watcher.feet + Vector3::new(0.0, FACE_HEIGHT, 0.0);
    let way = at - from;
    let length = way.norm();
    if length < 1.0e-3 {
        return true;
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
    hits.iter()
        .find(|hit| hit.collider != watcher.collider)
        .is_none_or(|hit| hit.collider == seen)
}

impl Inhabitant {
    /// Puts it into `alert`, or with none calms it down, and gets it ready to go about it; and
    /// tells of it in `news`, as the `n`th droid.
    fn enter(&mut self, n: usize, alert: Option<Alert>, news: &mut Vec<(usize, Option<Alert>)>) {
        if self.alert == alert {
            return;
        }
        self.alert = alert;
        news.push((n, alert));
        match alert {
            Some(Alert::Alert) => self.replan = 0.0,
            Some(Alert::Evasion) => self.search_afresh(),
            Some(Alert::Caution) | None => {
                self.route.clear();
                self.looking = None;
                self.resting = REST.0;
                self.search_left = CAUTION;
            }
        }
    }

    /// Starts searching from where it last saw the player.
    fn search_afresh(&mut self) {
        self.route.clear();
        self.looking = None;
        self.searched = 0;
        self.search_left = EVASION;
    }
}

impl Inhabitants {
    /// Whether they have been put into this round's maze yet.
    pub fn is_populated(&self) -> bool {
        self.populated
    }

    /// Takes everyone out of the scene, to be put into the next round afresh.
    pub fn clear(&mut self, graph: &mut Graph) {
        for mut droid in self.droids.drain(..) {
            droid.avatar.sweep_up(graph);
            if graph.is_valid_handle(droid.body) {
                graph.remove_node(droid.body);
            }
            if let Some(ragdoll) = &droid.ragdoll {
                ragdoll.remove(graph);
            }
        }
        self.populated = false;
        self.reachable = Reach::default();
        self.chase = None;
        self.chase_in = 0.0;
        self.shootout = None;
        self.shootout_in = 0.0;
        self.search = None;
        self.last_sight = None;
        self.search_from = None;
    }

    /// Puts the maze's inhabitants into `scene` on the floor of `grid` whose corner is at
    /// `origin`, well away from the `player`'s feet - all but one, a little way in front of them
    /// as they face `facing` - and off the `roads`, if it has any. They take turns being each of
    /// the kinds of droid there are to talk to, each looking as its one of `liveries` has it:
    /// `people` of them, or [`COUNT`] without - or as MAZE_INHABITANTS says. Of each kind, the
    /// share `in_pairs` says walk about in twos, one keeping beside the other.
    #[allow(clippy::too_many_arguments)]
    pub fn populate(
        &mut self,
        scene: &mut Scene,
        liveries: Vec<Livery>,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        (player, facing): (Vector3<f32>, f32),
        (people, grouping): (Option<usize>, impl Fn(usize) -> Grouping),
        crowd: bool,
        roads: Option<&Roads>,
        rng: &mut Rng,
    ) {
        self.clear(&mut scene.graph);
        self.populated = true;
        self.liveries = liveries;
        let characters = self.liveries.len();
        if characters == 0 {
            return;
        }
        self.groupings = (0..characters).map(&grouping).collect();
        ragdoll::prepare(&mut scene.graph);
        let count = crate::platform::var("MAZE_INHABITANTS")
            .and_then(|n| n.trim().parse().ok())
            .or(people)
            .unwrap_or(COUNT);
        let on_road = |i: usize| roads.is_some_and(|roads| roads.is_road(i % grid.width, i / grid.width));
        let Some(start) = survey::cell_at(grid, origin, player)
            .filter(|&(x, z)| grid.is_walkable(x, z))
            .or_else(|| survey::nearest_walkable(grid, origin, player))
        else {
            return;
        };
        let routes = grid.routes_from(start, f32::INFINITY);
        self.reachable = Reach::of((0..grid.width * grid.rows()).map(|i| routes.cost(i).is_some()));
        // A crowd turns up again anywhere it could have walked to, off the roads; and is about
        // the player from the start.
        self.chase_in = CHASE_FIRST;
        self.shootout_in = SHOOTOUT_FIRST;
        self.crowd = crowd.then(|| Crowd {
            pavement: (0..grid.width * grid.rows()).map(|i| routes.cost(i).is_some() && !on_road(i)).collect(),
            next: CROWD_EVERY,
        });
        let about = |cell: (usize, usize)| !crowd || flat(grid.on_floor(origin, cell) - player).norm() <= CROWD_BACK.1;
        // Where they can be put down, away from the player and near them, in one look over a
        // grid that can be a whole city's.
        let (mut places, mut near) = (Vec::new(), Vec::new());
        for (i, cost) in routes.reached() {
            let far = cost >= AWAY_FROM_PLAYER;
            let cell = (i % grid.width, i / grid.width);
            if (far || (NEAR_PLAYER.0..=NEAR_PLAYER.1).contains(&cost)) && !on_road(i) && about(cell) {
                match far {
                    true => places.push(cell),
                    false => near.push(cell),
                }
            }
        }
        // In front of them if there is room, where they will see it.
        let ahead: Vec<(usize, usize)> = near
            .iter()
            .copied()
            .filter(|&cell| {
                let to = flat(grid.on_floor(origin, cell) - player).try_normalize(1.0e-3);
                to.is_some_and(|to| to.dot(&forward(facing)) >= NEAR_CONE.cos())
            })
            .collect();
        let near = if ahead.is_empty() { near } else { ahead };
        let first = rng.below(characters);
        // Each kind in turn - or, with some more common than others, each picked by its share.
        let shares: Vec<f32> = (0..characters).map(|c| grouping(c).share.max(0.0)).collect();
        let even = shares.iter().all(|&share| share == shares[0]);
        let total: f32 = shares.iter().sum();
        for n in 0.. {
            if self.droids.len() >= count {
                break;
            }
            // The first near the player, if there is room; the rest away.
            let close = n == 0 && !near.is_empty();
            let place = match close {
                true => near[rng.below(near.len())],
                false if places.is_empty() => break,
                false => places.swap_remove(rng.below(places.len())),
            };
            let feet = grid.on_floor(origin, place);
            let heading = match close {
                true => {
                    let to_player = flat(player - feet);
                    to_player.x.atan2(to_player.z)
                }
                false => between(rng, (-std::f32::consts::PI, std::f32::consts::PI)),
            };
            // Not all setting off at once, and the one near the player not for a while.
            let resting = match close {
                true => NEAR_REST,
                false => between(rng, REST),
            };
            let character = match even || total <= 0.0 {
                true => (first + n) % characters,
                false => {
                    let mut pick = between(rng, (0.0, total));
                    shares.iter().position(|&share| {
                        pick -= share;
                        pick < 0.0
                    })
                    .unwrap_or(characters - 1)
                }
            };
            if !self.spawn_one(scene, feet, heading, resting, character, rng) {
                break;
            }
            // Those walking about with it, about it, if it is one of a pair or a group.
            let Grouping { in_pairs, groups_of, turf, .. } = grouping(character);
            let size = if groups_of >= 2 {
                self.group_size(character, rng)
            } else if (rng.below(1000) as f32) < in_pairs * 1000.0 {
                2
            } else {
                1
            };
            let leader = self.droids.len() - 1;
            let mut slot = 0;
            for _ in 1..size {
                if self.droids.len() >= count {
                    break;
                }
                let floor = (slot..ABOUT_LEADER.len()).find_map(|s| {
                    let at = about_leader(feet, heading, s);
                    survey::cell_at(grid, origin, at)
                        .filter(|&(x, z)| grid.is_walkable(x, z) && !roads.is_some_and(|roads| roads.is_road(x, z)))
                        .map(|cell| (s, grid.on_floor(origin, cell)))
                });
                let Some((s, at)) = floor else {
                    break;
                };
                if !self.spawn_one(scene, at, heading, resting, character, rng) {
                    break;
                }
                let other = self.droids.len() - 1;
                if self.droids[leader].partner.is_none() {
                    self.droids[leader].partner = Some(other);
                }
                self.droids[leader].leads = true;
                self.droids[other].partner = Some(leader);
                self.droids[other].slot = s;
                slot = s + 1;
            }
            // Some groups keep to where they are, as their turf.
            if let Some((share, reach)) = turf.filter(|_| self.droids[leader].leads) {
                if (rng.below(1000) as f32) < share * 1000.0 {
                    for m in self.group_of(leader) {
                        self.droids[m].turf = Some((feet, reach));
                    }
                }
            }
        }
        let groups = self.droids.iter().filter(|droid| droid.leads).count();
        let turfs = self.droids.iter().filter(|droid| droid.leads && droid.turf.is_some()).count();
        Log::info(format!(
            "Maze: {} inhabitants, {groups} pairs or groups of them, {turfs} keeping to turf of their own",
            self.droids.len()
        ));
    }

    /// In a city, now and then, has one of its security - a kind `guards` says is one - go after
    /// one of a gang - a kind `thieves` says is one - near it, which runs for it, both near
    /// enough the `player`'s feet to be seen (see [`Chase`]); and sees each chase through, `dt`
    /// seconds on from the last time: caught, the one running stands to be held at pistol point a
    /// while; well ahead, or after long enough, it has got away. What came of it (see
    /// [`ChaseNews`]).
    pub fn foot_chases(
        &mut self,
        player: Vector3<f32>,
        rng: &mut Rng,
        dt: f32,
        guards: impl Fn(usize) -> bool,
        thieves: impl Fn(usize) -> bool,
    ) -> ChaseNews {
        let mut news = ChaseNews::default();
        let mut said = Vec::new();
        if self.crowd.is_none() {
            return news;
        }
        let Some(mut chase) = self.chase else {
            self.chase_in -= dt;
            if self.chase_in > 0.0 {
                return news;
            }
            let calm = |droid: &Inhabitant| {
                !droid.down
                    && droid.ragdoll.is_none()
                    && droid.alert.is_none()
                    && !droid.talking
                    && droid.warned == 0
                    && droid.trip.is_none()
                    && droid.ride.is_none()
                    && droid.pursuit.is_none()
                    && droid.side.is_none()
                    && droid.cover.is_none()
                    && droid.witness.is_none()
                    && droid.post.is_none()
                    && droid.speaking == 0.0
                    && flat(droid.feet - player).norm() < CHASE_SEEN
            };
            // One of a gang - not one leading the rest of it - and one of security near enough.
            let droids = &self.droids;
            let mut pairs: Vec<(usize, usize)> = Vec::new();
            for (t, thief) in droids.iter().enumerate() {
                if !thieves(thief.character) || thief.leads || !calm(thief) {
                    continue;
                }
                for (g, guard) in droids.iter().enumerate() {
                    let apart = flat(guard.feet - thief.feet).norm();
                    if guards(guard.character) && calm(guard) && (CHASE_APART.0..=CHASE_APART.1).contains(&apart) {
                        pairs.push((g, t));
                    }
                }
            }
            if pairs.is_empty() {
                self.chase_in = CHASE_RETRY;
                return news;
            }
            self.chase_in = between(rng, CHASE_EVERY);
            let (guard, thief) = pairs[rng.below(pairs.len())];
            // The one running leaves its group behind, for good.
            self.leave_group(thief);
            for (n, pursuit) in [(guard, Pursuit::Chasing(thief)), (thief, Pursuit::Fleeing(guard))] {
                let droid = &mut self.droids[n];
                droid.pursuit = Some(pursuit);
                droid.route.clear();
                droid.replan = 0.0;
                droid.resting = 0.0;
            }
            self.chase = Some(Chase { guard, thief, time: 0.0, reply: Some((REPLY_AFTER, "flee")) });
            Log::info(format!("Foot chase: droid {guard} after droid {thief}"));
            said.push((guard, "chase"));
            news.started = Some((guard, thief));
            news.said = self.heard_near(said, player);
            return news;
        };
        chase.time += dt;
        if let Some((left, name)) = chase.reply {
            chase.reply = match left - dt {
                left if left > 0.0 => Some((left, name)),
                _ => {
                    said.push((chase.thief, name));
                    None
                }
            };
        }
        let (guard, thief) = (&self.droids[chase.guard], &self.droids[chase.thief]);
        let apart = flat(guard.feet - thief.feet).norm();
        // Anything else coming up - the player, say - and it is over, with nothing more said.
        let interrupted = [guard, thief]
            .iter()
            .any(|droid| droid.down || droid.ragdoll.is_some() || droid.alert.is_some() || droid.talking);
        let holding = matches!(guard.pursuit, Some(Pursuit::Holding(_)));
        let over = if interrupted {
            true
        } else if holding {
            chase.time > HELD_FOR
        } else if apart < CHASE_CAUGHT && (guard.feet.y - thief.feet.y).abs() < CATCH_HEIGHT {
            // Caught: it stops where it is, and is held there a while.
            self.droids[chase.guard].pursuit = Some(Pursuit::Holding(chase.thief));
            self.droids[chase.thief].pursuit = Some(Pursuit::Held(chase.guard));
            for n in [chase.guard, chase.thief] {
                self.droids[n].speed = 0.0;
                self.droids[n].route.clear();
            }
            chase.time = 0.0;
            said.push((chase.guard, "caught"));
            chase.reply = Some((REPLY_AFTER, "caught"));
            Log::info(format!("Foot chase: droid {} caught droid {}", chase.guard, chase.thief));
            false
        } else if apart > CHASE_ESCAPED || chase.time > CHASE_FOR {
            said.push((chase.guard, "escaped"));
            Log::info(format!("Foot chase: droid {} got away from droid {}", chase.thief, chase.guard));
            true
        } else {
            false
        };
        if over {
            for n in [chase.guard, chase.thief] {
                let droid = &mut self.droids[n];
                droid.pursuit = None;
                droid.route.clear();
                droid.armed_left = 0.0;
                droid.resting = between(rng, REST);
            }
            self.chase = None;
            news.ended = Some((chase.guard, chase.thief));
        } else {
            self.chase = Some(chase);
        }
        news.said = self.heard_near(said, player);
        news
    }

    /// In a city, now and then, has six of its security - kinds `guards` says are - fight it out
    /// with a gang of six - kinds `thieves` says are - near enough the `player`'s feet to be
    /// heard: the gang gathers where one of it is, and security is called in to it - meeting up out
/// of its sight first, to go in together - each side
    /// shooting at the other as capture the flag's do (see [`Side::Security`]); and sees it
    /// through, `dt` seconds on from the last time, until one side is all down, it has gone on
    /// long enough, or the player has gone far off. Those left standing go about their business
    /// again, each by itself. What came of it (see [`ShootoutNews`]).
    pub fn shootouts(
        &mut self,
        player: Vector3<f32>,
        rng: &mut Rng,
        dt: f32,
        guards: impl Fn(usize) -> bool,
        thieves: impl Fn(usize) -> bool,
    ) -> ShootoutNews {
        let mut news = ShootoutNews::default();
        let mut said = Vec::new();
        if self.crowd.is_none() {
            return news;
        }
        let Some(mut shootout) = self.shootout.take() else {
            self.shootout_in -= dt;
            if self.shootout_in > 0.0 {
                return news;
            }
            let calm = |droid: &Inhabitant| {
                !droid.down
                    && droid.ragdoll.is_none()
                    && droid.alert.is_none()
                    && !droid.talking
                    && droid.warned == 0
                    && droid.trip.is_none()
                    && droid.ride.is_none()
                    && droid.pursuit.is_none()
                    && droid.side.is_none()
                    && droid.cover.is_none()
                    && droid.speaking == 0.0
                    // Not one keeping a post: a precinct's guard, or the gang in its hideout.
                    && droid.post.is_none()
            };
            let droids = &self.droids;
            // The nearest of those `of` kind, calm, to `site` - and far enough off - as many as
            // there are to be, if there are as many.
            let nearest = |site: Vector3<f32>, of: &dyn Fn(usize) -> bool, apart: f32| {
                let mut near: Vec<(usize, f32)> = droids
                    .iter()
                    .enumerate()
                    .filter(|(_, droid)| of(droid.character) && calm(droid))
                    .map(|(n, droid)| (n, flat(droid.feet - site).norm()))
                    .filter(|&(_, away)| (apart..SHOOTOUT_CALLED).contains(&away))
                    .collect();
                near.sort_by(|a, b| a.1.total_cmp(&b.1));
                (near.len() >= SHOOTOUT_SIDE).then(|| near.into_iter().take(SHOOTOUT_SIDE).map(|(n, _)| n).collect::<Vec<_>>())
            };
            // Where one of a gang is, near enough to be heard, and enough of both about it.
            let mut sites: Vec<usize> = (0..droids.len())
                .filter(|&n| thieves(droids[n].character) && calm(&droids[n]))
                .filter(|&n| (SHOOTOUT_SEEN.0..SHOOTOUT_SEEN.1).contains(&flat(droids[n].feet - player).norm()))
                .collect();
            let first = rng.below(sites.len().max(1));
            sites.rotate_left(first);
            // A gang on its turf first: security raids it there.
            sites.sort_by_key(|&n| droids[n].turf.is_none());
            let found = sites.into_iter().find_map(|n| {
                let site = droids[n].feet;
                Some((site, nearest(site, &guards, SHOOTOUT_APART)?, nearest(site, &thieves, 0.0)?))
            });
            let Some((site, guards, gang)) = found else {
                self.shootout_in = SHOOTOUT_RETRY;
                return news;
            };
            self.shootout_in = between(rng, SHOOTOUT_EVERY);
            let rally = self.droids[guards[0]].feet;
            for (side, members, post) in [(Side::Security, &guards, rally), (Side::Gang, &gang, site)] {
                for &n in members {
                    // Its turf it keeps, to go back to after, if it is left standing.
                    let turf = self.droids[n].turf;
                    self.leave_group(n);
                    let droid = &mut self.droids[n];
                    droid.turf = turf;
                    droid.side = Some(side);
                    droid.post = Some(post);
                    droid.foe = None;
                    droid.foe_unseen = 0.0;
                    droid.hits = 0;
                    droid.route.clear();
                    droid.replan = 0.0;
                    droid.resting = 0.0;
                }
            }
            Log::info(format!("Shootout: security {guards:?} against gang {gang:?}"));
            // Security's nearest calls the rest of it in.
            said.push((guards[0], "shootout"));
            news.started = guards.iter().chain(&gang).copied().collect();
            news.said = self.heard_near(said, player);
            self.shootout = Some(Shootout { site, rally: Some(rally), guards, gang, time: 0.0, engaged: [false; 2] });
            return news;
        };
        shootout.time += dt;
        // Met up, or done waiting, security goes in together.
        if let Some(rally) = shootout.rally {
            let met = shootout
                .guards
                .iter()
                .all(|&n| self.droids[n].down || flat(self.droids[n].feet - rally).norm() < RALLY_NEAR);
            if met || shootout.time > RALLY_FOR {
                shootout.rally = None;
                for &n in &shootout.guards {
                    let droid = &mut self.droids[n];
                    droid.post = Some(shootout.site);
                    droid.route.clear();
                    droid.replan = 0.0;
                    droid.resting = 0.0;
                }
                Log::info(format!("Shootout: security goes in, {:.0} seconds after it was called", shootout.time));
            }
        }
        // The first of each side to go after the other calls it out.
        for (side, members) in [&shootout.guards, &shootout.gang].into_iter().enumerate() {
            if shootout.engaged[side] {
                continue;
            }
            if let Some(&n) = members.iter().find(|&&n| self.droids[n].foe.is_some() && !self.droids[n].down) {
                shootout.engaged[side] = true;
                said.push((n, "engaged"));
            }
        }
        // Once the shooting starts, those about it going about their business run for cover;
        // and those that have something else to do - the player, say - think better of it.
        let shooting = shootout.engaged.iter().any(|&engaged| engaged);
        for (n, droid) in self.droids.iter_mut().enumerate() {
            if droid.cover.is_some() && (droid.down || droid.alert.is_some() || droid.talking || droid.trip.is_some() || droid.aboard()) {
                droid.cover = None;
                // Its eyes its own again, but for one down, or after the player, whose show that.
                if !droid.down && droid.alert.is_none() {
                    news.ended.push(n);
                }
            }
            let bystander = shooting
                && droid.cover.is_none()
                && droid.side.is_none()
                && droid.pursuit.is_none()
                && !droid.down
                && droid.alert.is_none()
                && !droid.talking
                && droid.warned == 0
                && droid.trip.is_none()
                && !droid.aboard()
                && flat(droid.feet - shootout.site).norm() < COVER_NEAR;
            if bystander {
                droid.cover = Some((shootout.site, false));
                droid.route.clear();
                droid.resting = 0.0;
                news.sheltering.push(n);
            }
        }
        if !news.sheltering.is_empty() {
            Log::info(format!("Shootout: {} about it run for cover", news.sheltering.len()));
        }
        let standing = |members: &[usize]| members.iter().copied().filter(|&n| !self.droids[n].down).collect::<Vec<_>>();
        let (guards, gang) = (standing(&shootout.guards), standing(&shootout.gang));
        let over = guards.is_empty()
            || gang.is_empty()
            || shootout.time > SHOOTOUT_FOR
            || flat(shootout.site - player).norm() > SHOOTOUT_GONE;
        if !over {
            self.shootout = Some(shootout);
            news.said = self.heard_near(said, player);
            return news;
        }
        // One side left standing says so.
        match (guards.first(), gang.first()) {
            (Some(&n), None) | (None, Some(&n)) => said.push((n, "won")),
            _ => (),
        }
        Log::info(format!("Shootout over: {} of security and {} of the gang standing", guards.len(), gang.len()));
        for &n in shootout.guards.iter().chain(&shootout.gang) {
            let droid = &mut self.droids[n];
            droid.side = None;
            droid.post = None;
            droid.foe = None;
            droid.shot_from = None;
            if !droid.down && droid.alert.is_none() {
                droid.route.clear();
                droid.trip = None;
                droid.resting = between(rng, REST);
            }
        }
        // The gang left standing goes on together, as one group.
        let calm: Vec<usize> = gang.iter().copied().filter(|&n| self.droids[n].alert.is_none()).collect();
        if let Some((&leader, rest)) = calm.split_first() {
            self.join_group(leader, rest);
        }
        news.ended.extend(guards.into_iter().chain(gang));
        // The danger past, those that took cover come out again.
        for (n, droid) in self.droids.iter_mut().enumerate() {
            if droid.cover.take().is_some() {
                droid.route.clear();
                droid.resting = between(rng, REST);
                if !droid.down && droid.alert.is_none() {
                    news.ended.push(n);
                }
            }
        }
        news.said = self.heard_near(said, player);
        news
    }

    /// Has the `n`th droid leave its pair or group, for good: another of those keeping about it
    /// leads the rest, if it led them.
    fn leave_group(&mut self, n: usize) {
        let Some(partner) = self.droids[n].partner.take() else {
            return;
        };
        let kept = |m: usize, leader: usize| m != leader && !self.droids[m].leads && self.droids[m].partner == Some(leader);
        if self.droids[n].leads {
            let rest: Vec<usize> = (0..self.droids.len()).filter(|&m| kept(m, n)).collect();
            if let Some((&leader, others)) = rest.split_first() {
                self.droids[leader].partner = others.first().copied();
                self.droids[leader].leads = !others.is_empty();
                self.droids[leader].slot = 0;
                for &m in others {
                    self.droids[m].partner = Some(leader);
                }
            }
        } else if self.droids[partner].partner == Some(n) {
            let next = (0..self.droids.len()).find(|&m| kept(m, partner));
            self.droids[partner].partner = next;
            self.droids[partner].leads = next.is_some();
        }
        self.droids[n].leads = false;
        self.droids[n].slot = 0;
        self.droids[n].turf = None;
    }

    /// How many a group of the `character`th kind is to be: from two up to as many as its
    /// grouping says, and no more than can keep about the one leading it (see [`ABOUT_LEADER`]);
    /// one, for a kind that does not walk about in groups.
    fn group_size(&self, character: usize, rng: &mut Rng) -> usize {
        let most = self.groupings.get(character).map_or(0, |g| g.groups_of).min(ABOUT_LEADER.len() + 1);
        match most >= 2 {
            true => 2 + rng.below(most - 1),
            false => 1,
        }
    }

    /// Has each of `joining`, walking about by itself, keep about the `leader` - in the places
    /// about it not already kept, as many as there are - and to its turf, if it has one.
    fn join_group(&mut self, leader: usize, joining: &[usize]) {
        let mut kept: Vec<usize> = self
            .group_of(leader)
            .into_iter()
            .skip(1)
            .filter(|&m| !self.droids[m].down)
            .map(|m| self.droids[m].slot)
            .collect();
        let turf = self.droids[leader].turf;
        for &m in joining {
            let Some(slot) = (0..ABOUT_LEADER.len()).find(|s| !kept.contains(s)) else {
                break;
            };
            kept.push(slot);
            let droid = &mut self.droids[m];
            droid.partner = Some(leader);
            droid.leads = false;
            droid.slot = slot;
            droid.turf = turf;
            let leader = &mut self.droids[leader];
            leader.partner = leader.partner.or(Some(m));
            leader.leads = true;
        }
    }

    /// Those of `said` near enough the `player`'s feet to be heard (see [`CHASE_HEARD`]).
    fn heard_near(&self, said: Vec<(usize, &'static str)>, player: Vector3<f32>) -> Vec<(usize, &'static str)> {
        said.into_iter()
            .filter(|&(n, _)| flat(self.droids[n].feet - player).norm() < CHASE_HEARD)
            .collect()
    }

    /// The nearest pair or group to the `player`'s feet, within `reach` meters, of a kind
    /// `talks` says has things to say among itself, that is going about its business - all of
    /// it calm, and minding nothing - to talk among itself: up to three of it, the one leading
    /// it first.
    pub fn talkers(&self, player: Vector3<f32>, reach: f32, talks: impl Fn(usize) -> bool) -> Option<Vec<usize>> {
        (0..self.droids.len())
            .filter(|&n| self.droids[n].leads && talks(self.droids[n].character))
            .map(|n| self.group_of(n).into_iter().filter(|&m| !self.droids[m].down).take(3).collect::<Vec<_>>())
            .filter(|members| members.len() >= 2 && self.at_ease(members, player, reach))
            .min_by(|a, b| {
                let away = |members: &Vec<usize>| flat(self.droids[members[0]].feet - player).norm();
                away(a).total_cmp(&away(b))
            })
    }

    /// Whether all of `members` are going about their business within `reach` meters of the
    /// `player`'s feet - standing, calm, and minding nothing - to talk among themselves.
    pub fn at_ease(&self, members: &[usize], player: Vector3<f32>, reach: f32) -> bool {
        members.iter().all(|&n| {
            self.droids.get(n).is_some_and(|droid| {
                !droid.down
                    && droid.alert.is_none()
                    && !droid.talking
                    && droid.warned == 0
                    && droid.side.is_none()
                    && droid.pursuit.is_none()
                    && droid.cover.is_none()
                    && droid.trip.is_none()
                    && !droid.aboard()
                    && flat(droid.feet - player).norm() < reach
            })
        })
    }

    /// Whether the `n`th droid speaks for its pair or group: the first of them still standing -
    /// the one leading it, unless that is down - or one walking about by itself.
    pub fn speaks_for_group(&self, n: usize) -> bool {
        self.group_of(n).into_iter().find(|&m| !self.droids[m].down) == Some(n)
    }

    /// The `n`th droid's group: the one leading it first, then those keeping about it - or the
    /// droid alone, walking about by itself.
    fn group_of(&self, n: usize) -> Vec<usize> {
        let Some(droid) = self.droids.get(n) else {
            return Vec::new();
        };
        let leader = match droid.leads {
            true => n,
            false => droid.partner.unwrap_or(n),
        };
        std::iter::once(leader)
            .chain((0..self.droids.len()).filter(|&m| {
                m != leader && !self.droids[m].leads && self.droids[m].partner == Some(leader)
            }))
            .collect()
    }

    /// In a city, has those of its crowd that have got far from the `player`'s feet, going about
    /// their business, turn up again as others nearer by, on the floor of `grid` whose corner is
    /// at `origin` (see [`Crowd`]): wherever `seen` says the player cannot see from where they
    /// are, or off to the side of the way they look, `ahead`. One lying where it went down, as
    /// far off and out of sight, is taken away, and another of its kind turns up in its place, in
    /// `scene`: or a city would empty, a shootout at a time. `dt` seconds on from the last time.
    #[allow(clippy::too_many_arguments)]
    pub fn recycle(
        &mut self,
        scene: &mut Scene,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        player: Vector3<f32>,
        ahead: Vector3<f32>,
        rng: &mut Rng,
        dt: f32,
        seen: impl Fn(&Graph, Vector3<f32>) -> bool,
    ) {
        let Some(crowd) = self.crowd.as_mut() else {
            return;
        };
        crowd.next -= dt;
        if crowd.next > 0.0 {
            return;
        }
        crowd.next = CROWD_EVERY;
        let graph = &mut scene.graph;
        let ahead = flat(ahead).try_normalize(1.0e-3);
        // Out of the player's sight: behind something, or off to the side of the view.
        let hidden = |graph: &Graph, at: Vector3<f32>| {
            let off_view = match (ahead, flat(at - player).try_normalize(1.0e-3)) {
                (Some(ahead), Some(to)) => to.dot(&ahead) < OUT_OF_VIEW.cos(),
                _ => false,
            };
            off_view || !seen(graph, at)
        };
        // Going about its business, with nothing to do with the player or anyone else.
        let idle = |droid: &Inhabitant| {
            !droid.down
                && droid.ragdoll.is_none()
                && droid.alert.is_none()
                && !droid.talking
                && droid.warned == 0
                && droid.side.is_none()
                && droid.post.is_none()
                && droid.trip.is_none()
                && !droid.aboard()
                && droid.foe.is_none()
                && droid.armed_left == 0.0
                && droid.speaking == 0.0
                && droid.pursuit.is_none()
                && droid.cover.is_none()
                && droid.witness.is_none()
        };
        let mut moved = 0;
        for n in 0..self.droids.len() {
            if moved >= CROWD_AT_ONCE {
                break;
            }
            let droid = &self.droids[n];
            // A pair or a group goes together, the one leading taking the rest along; one whose
            // leader is down goes on its own from now.
            if !droid.leads && droid.partner.is_some_and(|p| !self.droids[p].down) {
                continue;
            }
            let members: Vec<usize> = match droid.leads {
                true => self.group_of(n).into_iter().filter(|&m| !self.droids[m].down).collect(),
                false => vec![n],
            };
            let gone = |droid: &Inhabitant| {
                idle(droid) && flat(droid.feet - player).norm() > CROWD_GONE && hidden(graph, droid.feet)
            };
            if !members.iter().all(|&m| gone(&self.droids[m])) {
                continue;
            }
            if members.len() == 1 && self.droids[n].partner.is_some() {
                self.droids[n].partner = None;
                self.droids[n].leads = false;
            }
            // One of a kind that walks about in groups, left by itself - the others of its group
            // down, or itself gone off from them - makes a group afresh with others of its kind
            // gone off by themselves; and if none can join it, waits, out of the way, for some
            // that can.
            let kind = self.droids[n].character;
            let mut members = members;
            let size = self.group_size(kind, rng);
            if members.len() == 1 && size >= 2 {
                let joining: Vec<usize> = (0..self.droids.len())
                    .filter(|&m| {
                        let droid = &self.droids[m];
                        !members.contains(&m)
                            && droid.character == kind
                            && droid.partner.is_none()
                            && !droid.leads
                            && gone(droid)
                    })
                    .take(size - members.len())
                    .collect();
                self.join_group(n, &joining);
                // Some of a group made afresh keep to turf of their own.
                let turf = self.groupings.get(kind).and_then(|g| g.turf);
                if let Some((share, reach)) = turf.filter(|_| !joining.is_empty() && self.droids[n].turf.is_none()) {
                    if (rng.below(1000) as f32) < share * 1000.0 {
                        for &m in std::iter::once(&n).chain(&joining) {
                            self.droids[m].turf = Some((self.droids[n].feet, reach));
                        }
                    }
                }
                members.extend(joining);
            }
            if size >= 2 && members.len() < 2 {
                continue;
            }
            let Some(crowd) = self.crowd.as_ref() else {
                return;
            };
            let slots: Vec<usize> = members.iter().skip(1).map(|&m| self.droids[m].slot).collect();
            let spot = crowd.spot((grid, origin), player, &slots, rng, |at| hidden(graph, at));
            let Some((feet, heading, about)) = spot else {
                continue;
            };
            let resting = between(rng, CROWD_REST);
            // Turf of its own goes with it, where it turns up.
            for &m in &members {
                if let Some((_, reach)) = self.droids[m].turf {
                    self.droids[m].turf = Some((feet, reach));
                }
            }
            self.droids[n].put_down(graph, feet, heading, resting, rng);
            for (&m, at) in members.iter().skip(1).zip(about) {
                self.droids[m].put_down(graph, at, heading, resting, rng);
            }
            moved += 1;
        }
        if moved > 0 && crate::platform::var("MAZE_DEBUG").is_some() {
            Log::info(format!("Crowd: {moved} turned up again nearer by"));
        }
        // Those lying where they went down, far off out of sight, make way for others.
        let mut renewed = 0;
        for n in 0..self.droids.len() {
            if moved + renewed >= CROWD_AT_ONCE {
                break;
            }
            let droid = &self.droids[n];
            let gone = droid.down
                && droid.side.is_none()
                && droid.pursuit.is_none()
                && flat(droid.feet - player).norm() > CROWD_GONE
                && hidden(&scene.graph, droid.feet);
            if !gone {
                continue;
            }
            let Some(crowd) = self.crowd.as_ref() else {
                return;
            };
            let graph = &scene.graph;
            let character = self.droids[n].character;
            // One of a kind that walks about in groups turns up where it went down, out of the
            // way, to make a group afresh with others of its kind there (see above), rather than
            // nearer by by itself.
            let grouped = self.groupings.get(character).is_some_and(|g| g.groups_of >= 2);
            let there = survey::cell_at(grid, origin, droid.feet)
                .filter(|&(x, z)| grouped && grid.is_walkable(x, z))
                .map(|cell| (grid.on_floor(origin, cell), between(rng, (-std::f32::consts::PI, std::f32::consts::PI)), Vec::new()));
            let Some((feet, heading, _)) = there.or_else(|| crowd.spot((grid, origin), player, &[], rng, |at| hidden(graph, at))) else {
                continue;
            };
            if !self.spawn_one(scene, feet, heading, between(rng, CROWD_REST), character, rng) {
                continue;
            }
            self.leave_group(n);
            let Some(fresh) = self.droids.pop() else {
                continue;
            };
            let mut old = std::mem::replace(&mut self.droids[n], fresh);
            old.avatar.sweep_up(&mut scene.graph);
            if scene.graph.is_valid_handle(old.body) {
                scene.graph.remove_node(old.body);
            }
            if let Some(ragdoll) = &old.ragdoll {
                ragdoll.remove(&mut scene.graph);
            }
            renewed += 1;
        }
        if renewed > 0 {
            Log::info(format!("Crowd: {renewed} that went down made way for others"));
        }
    }

    /// Puts one droid, of kind `character`, into `scene` with its feet at `feet`, facing
    /// `heading`, to stand `resting` seconds before it sets off. Whether it could be.
    fn spawn_one(
        &mut self,
        scene: &mut Scene,
        feet: Vector3<f32>,
        heading: f32,
        resting: f32,
        character: usize,
        rng: &mut Rng,
    ) -> bool {
        let collider: Handle<Collider> = ColliderBuilder::new(BaseBuilder::new())
            .with_shape(ColliderShape::capsule_y(MIDDLE - RADIUS, RADIUS))
            .with_collision_groups(ragdoll::character_groups())
            .build(&mut scene.graph);
        let body = RigidBodyBuilder::new(
            BaseBuilder::new()
                .with_child(collider)
                .with_local_transform(
                    TransformBuilder::new()
                        .with_local_position(feet + Vector3::new(0.0, MIDDLE, 0.0))
                        .build(),
                ),
        )
        .with_body_type(RigidBodyType::KinematicPositionBased)
        .build(&mut scene.graph)
        .to_base();
        let model = &self.liveries[character].model;
        let Some(avatar) = Avatar::spawn(model, scene, body, -MIDDLE, true) else {
            scene.graph.remove_node(body);
            return false;
        };
        self.droids.push(Inhabitant {
            body,
            collider,
            avatar,
            feet,
            speed: 0.0,
            heading,
            route: Vec::new(),
            trip: None,
            ride: None,
            resting,
            waiting: 0.0,
            last_seen: None,
            character,
            code: 10 + rng.below(90) as u32,
            looks_hostile: false,
            talking: false,
            with: None,
            alert: None,
            sees_player: false,
            lost_at: feet,
            lost_going: Vector3::zeros(),
            search_left: 0.0,
            searched: 0,
            looking: None,
            look_from: 0.0,
            deaf: 0.0,
            threat: 0.0,
            warned: 0,
            unaimed: 0.0,
            replan: 0.0,
            no_way: false,
            windup: 0.0,
            sprint_in: -1.0,
            sprinting: 0.0,
            stamina: SENTRY_STAMINA,
            winded: false,
            most: SENTRY_STAMINA,
            easing: false,
            sentry: false,
            hits: 0,
            down: false,
            ragdoll: None,
            murdered: false,
            found: false,
            witness: None,
            side: None,
            post: None,
            foe: None,
            foe_unseen: 0.0,
            shot_from: None,
            speaking: 0.0,
            reload: 0.0,
            armed_left: 0.0,
            aimed_at: None,
            partner: None,
            leads: false,
            slot: 0,
            pursuit: None,
            cover: None,
            turf: None,
            // Not all posed on the same frame, far off.
            unposed: between(rng, (0.0, POSE_EVERY[POSE_EVERY.len() - 1].1)),
        });
        true
    }

    /// Puts each side's droids into `scene` for capture the flag, on the floor of `grid` whose
    /// corner is at `origin`, each at its post in `bases`, to keep to (see [`ctf::Bases::post`]).
    /// Each looks
    /// as its side's kind of droid does, as `kind` says, of `liveries`. The enemy's watch for
    /// the player from the start.
    pub fn populate_sides(
        &mut self,
        scene: &mut Scene,
        liveries: Vec<Livery>,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        bases: &ctf::Bases,
        kind: impl Fn(Side) -> usize,
        rng: &mut Rng,
    ) {
        self.clear(&mut scene.graph);
        self.populated = true;
        self.liveries = liveries;
        if self.liveries.is_empty() {
            return;
        }
        ragdoll::prepare(&mut scene.graph);
        for side in Side::BOTH {
            let facing = flat(bases.flag(side.other()) - bases.flag(side));
            for n in 0..ctf::DROIDS {
                let Some(feet) = spot_near((grid, origin), bases.post(side, n), SPAWN_REACH, rng) else {
                    continue;
                };
                let character = kind(side).min(self.liveries.len() - 1);
                let heading = facing.x.atan2(facing.z);
                if !self.spawn_one(scene, feet, heading, between(rng, (0.5, 2.0)), character, rng) {
                    break;
                }
                let m = self.droids.len() - 1;
                let droid = &mut self.droids[m];
                droid.side = Some(side);
                droid.post = Some(bases.post(side, n));
                if !side.is_players() {
                    droid.alert = Some(Alert::Caution);
                    droid.search_left = CAUTION;
                }
            }
        }
        Log::info(format!("Capture the flag: {} droids", self.droids.len()));
    }

    /// Puts a guard into `scene` for a heist at each of `posts`, on the floor of `grid` whose corner
    /// is at `origin`: droids of the side against the player, each keeping to its post and
    /// watching for them from the start, the kinds of droid of `liveries` in turn.
    pub fn populate_guards(
        &mut self,
        scene: &mut Scene,
        liveries: Vec<Livery>,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        posts: &[Vector3<f32>],
        rng: &mut Rng,
    ) {
        self.clear(&mut scene.graph);
        self.populated = true;
        self.liveries = liveries;
        if self.liveries.is_empty() {
            return;
        }
        ragdoll::prepare(&mut scene.graph);
        for (n, &post) in posts.iter().enumerate() {
            let Some(feet) = spot_near((grid, origin), post, SPAWN_REACH, rng) else {
                continue;
            };
            let heading = between(rng, (0.0, std::f32::consts::TAU));
            if !self.spawn_one(scene, feet, heading, between(rng, (0.5, 2.0)), n % self.liveries.len(), rng) {
                break;
            }
            let droid = self.droids.last_mut().expect("just put down");
            droid.side = Some(ctf::PLAYERS.other());
            droid.post = Some(post);
            droid.alert = Some(Alert::Caution);
            droid.search_left = CAUTION;
        }
        Log::info(format!("Heist: {} guards", self.droids.len()));
    }

    /// Sends a wave of `count` guards after the player, at `player`, in a heist: into `scene`, at
    /// the `entries` in turn, on the floor of `grid` whose corner is at `origin`, each already
    /// searching where the player is. How many came.
    pub fn spawn_wave(
        &mut self,
        scene: &mut Scene,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        entries: &[Vector3<f32>],
        count: usize,
        player: Vector3<f32>,
        rng: &mut Rng,
    ) -> usize {
        if self.liveries.is_empty() || entries.is_empty() {
            return 0;
        }
        let mut came = 0;
        for k in 0..count {
            let at = entries[k % entries.len()];
            let Some(feet) = spot_near((grid, origin), at, SPAWN_REACH, rng) else {
                continue;
            };
            let to = flat(player - feet);
            let character = (self.droids.len() + k) % self.liveries.len();
            if !self.spawn_one(scene, feet, to.x.atan2(to.z), between(rng, (0.1, 0.5)), character, rng) {
                break;
            }
            let m = self.droids.len() - 1;
            let droid = &mut self.droids[m];
            droid.side = Some(ctf::PLAYERS.other());
            droid.lost_at = player;
            droid.lost_going = Vector3::zeros();
            droid.alert = None;
            droid.enter(m, Some(Alert::Evasion), &mut self.alerts);
            self.search_from = Some(player);
            came += 1;
        }
        came
    }

    /// Puts `count` droids into `scene` for battle royale, each on a side of its own - side
    /// `Lone(1)` and on - one at each of `spots` (where, and which way it faces), on the floor of
    /// `grid` whose corner is at `origin`. Each is the kind of droid of `liveries` its number
    /// says, round them all in turn, and watches for everyone from the start.
    pub fn populate_lone(
        &mut self,
        scene: &mut Scene,
        liveries: Vec<Livery>,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        spots: &[(Vector3<f32>, f32)],
        rng: &mut Rng,
    ) {
        self.clear(&mut scene.graph);
        self.populated = true;
        self.liveries = liveries;
        if self.liveries.is_empty() {
            return;
        }
        ragdoll::prepare(&mut scene.graph);
        for (n, &(at, heading)) in spots.iter().enumerate() {
            let side = Side::Lone(n as u8 + 1);
            if !self.spawn_lone((grid, origin), scene, side, at, heading, rng) {
                break;
            }
        }
        Log::info(format!("Battle royale: {} droids", self.droids.len()));
    }

    /// Puts one droid of `side` into `scene` for battle royale, at `at` on the floor of `grid`,
    /// facing `heading`, watching for everyone: a new one, for a side whose droid went down and
    /// has a life left, while the old one lies where it fell. Whether it could.
    pub fn spawn_lone(
        &mut self,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        scene: &mut Scene,
        side: Side,
        at: Vector3<f32>,
        heading: f32,
        rng: &mut Rng,
    ) -> bool {
        if self.liveries.is_empty() {
            return false;
        }
        let feet = spot_near((grid, origin), at, SPAWN_REACH, rng).unwrap_or(at);
        let number = match side {
            Side::Lone(n) => n as usize,
            _ => 0,
        };
        let character = number % self.liveries.len();
        if !self.spawn_one(scene, feet, heading, between(rng, (0.5, 2.0)), character, rng) {
            return false;
        }
        let droid = self.droids.last_mut().expect("just put down");
        droid.side = Some(side);
        droid.alert = Some(Alert::Caution);
        droid.search_left = CAUTION;
        true
    }

    /// The sides that have a droid standing, in battle royale.
    pub fn sides_standing(&self) -> Vec<Side> {
        let mut sides: Vec<Side> = self.droids.iter().filter(|droid| !droid.down).filter_map(|droid| droid.side).collect();
        sides.dedup();
        sides
    }

    /// In battle royale, the ring everyone has to keep inside: its middle, and how far it reaches.
    pub fn set_ring(&mut self, ring: Option<(Vector3<f32>, f32)>) {
        self.ring = ring;
    }

    /// Whether there is any way at all to `to` over `grid`, whose corner is at `origin`, from where
    /// the player started - for the drones, to look for a way only where there is one.
    pub fn can_reach(&self, (grid, origin): (&WalkGrid, Vector3<f32>), to: Vector3<f32>) -> bool {
        self.reachable.to((grid, origin), to)
    }

    /// The trains, as of the end of this step, for those that take them.
    pub fn set_trains(&mut self, trains: Vec<Doors>) {
        self.trains = trains;
    }

    /// The droids standing outside `ring`, its middle and reach, as which they are.
    pub fn outside(&self, (middle, reach): (Vector3<f32>, f32)) -> Vec<usize> {
        self.droids
            .iter()
            .enumerate()
            .filter(|(_, droid)| !droid.down && flat(droid.feet - middle).norm() > reach)
            .map(|(n, _)| n)
            .collect()
    }

    /// Hurts the `n`th droid as a bolt would, but from nowhere and with no knock: the ring's
    /// doing. Whether it went down.
    pub fn hurt(&mut self, graph: &mut Graph, n: usize) -> bool {
        let Some(droid) = self.droids.get(n).filter(|droid| !droid.down) else {
            return false;
        };
        let strike = Strike {
            collider: droid.collider,
            at: droid.feet + Vector3::new(0.0, CHEST, 0.0),
            way: -Vector3::y(),
        };
        let feet = droid.feet;
        self.shot(graph, strike, feet) == Some(n)
    }

    /// Has the `n`th droid speak to the player for `seconds`: it turns its head to them, as far
    /// as a head turns, without turning round.
    pub fn speak_to_player(&mut self, n: usize, seconds: f32) {
        if let Some(droid) = self.droids.get_mut(n) {
            droid.speaking = seconds;
        }
    }

    /// The side of the `n`th droid, in capture the flag.
    pub fn side(&self, n: usize) -> Option<Side> {
        self.droids.get(n).and_then(|droid| droid.side)
    }

    /// Each droid standing, as a target for a drone: which it is, its side, its feet, the middle
    /// of its body, and its collider.
    pub fn standing(&self) -> Vec<(usize, Option<Side>, Vector3<f32>, Vector3<f32>, Handle<Collider>)> {
        self.droids
            .iter()
            .enumerate()
            .filter(|(_, droid)| !droid.down)
            .map(|(n, droid)| (n, droid.side, droid.feet, droid.feet + Vector3::new(0.0, CHEST, 0.0), droid.collider))
            .collect()
    }

    /// Moves everyone along for another `dt`, over `grid` whose corner is at `origin` - and across
    /// on the `crossings`, the ferries, where that is quicker - making way for each other and -
    /// unless they are after them - for the `player`'s feet, sentries as `sentry` says of their
    /// kind with more breath than the rest. Whether anyone caught the player, and whose phase
    /// changed.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        graph: &mut Graph,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        crossings: &[Crossing],
        roads: Option<&Roads>,
        player: Vector3<f32>,
        rivals: &[Rival],
        rng: &mut Rng,
        dt: f32,
        sentry: impl Fn(usize) -> bool,
    ) -> News {
        let mut caught = None;
        let mut alerts = std::mem::take(&mut self.alerts);
        let heard = std::mem::take(&mut self.heard);
        let alarmed = std::mem::take(&mut self.alarmed);
        // Floor that can be walked at the height of `spot`: not a drop off the edge, nor the floor
        // under a walkway, nor the walkway over it.
        let floor_at = |spot: Vector3<f32>| {
            survey::cell_at(grid, origin, spot)
                .is_some_and(|(x, z)| grid.is_walkable(x, z) && (grid.floor(x, z) - spot.y).abs() <= ON_FLOOR)
        };
        // A way to `to`, if there is one at all: none looked for where there is none to be had.
        let reachable = &self.reachable;
        let go_to = |feet: Vector3<f32>, to: Vector3<f32>| match reachable.to((grid, origin), to) {
            true => trip::go_to((grid, origin), feet, to, CHASE_REACH, crossings),
            false => (Vec::new(), None),
        };
        let route_to = |feet: Vector3<f32>, to: Vector3<f32>| match reachable.to((grid, origin), to) {
            true => route_to((grid, origin), feet, to, CHASE_REACH),
            false => Vec::new(),
        };
        // Where everyone is, and which way those walking are going; the player last.
        let everyone: Vec<(Vector3<f32>, Option<Vector3<f32>>)> = self
            .droids
            .iter()
            .map(|droid| {
                let walking = (droid.speed > 0.1).then(|| forward(droid.heading));
                (droid.feet, walking)
            })
            .chain(std::iter::once((player, None)))
            .collect();
        let the_player = everyone.len() - 1;
        // Who is where, by squares of the ground: each makes way only for those near it (see
        // [`NEIGHBOURS`]), and in a city of hundreds, looking through everyone for them would
        // cost more than all the rest of moving them along.
        let mut squares: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (n, (at, _)) in everyone.iter().enumerate() {
            squares.entry(square_of(*at)).or_default().push(n);
        }
        // Those lying where they went down are in no one's way.
        let down: Vec<bool> = self.droids.iter().map(|droid| droid.down).collect();
        // Each one's partner, if it walks about with one, as of the start of this frame.
        let partners: Vec<Option<Partner>> = self
            .droids
            .iter()
            .map(|droid| {
                let other = &self.droids[droid.partner?];
                Some(Partner {
                    feet: other.feet,
                    heading: other.heading,
                    speed: other.speed,
                    about: !other.down && matches!(other.alert, None | Some(Alert::Caution)),
                    held: other.talking || other.warned > 0,
                })
            })
            .collect();
        // In capture the flag, whom of the other side each is after, and whether it sees them:
        // the nearest it sees - or, near enough, hears - or else the one it was after, for a while
        // after losing sight of them. The other side's droids and drones alike.
        // Where each droid is standing, as of the start of this frame.
        let placed: Vec<Option<(Vector3<f32>, Vector3<f32>)>> = self
            .droids
            .iter()
            .map(|droid| (!droid.down).then(|| (droid.feet, droid.feet + Vector3::new(0.0, CHEST, 0.0))))
            .collect();
        let foe_place = |foe: Foe| -> Option<(Vector3<f32>, Vector3<f32>)> {
            match foe {
                Foe::Droid(n) => placed.get(n).copied().flatten(),
                Foe::Rival(body) => rivals.iter().find(|r| r.collider == body).map(|r| (r.feet, r.middle)),
            }
        };
        let foes: Vec<(Option<Foe>, bool)> = (0..self.droids.len())
            .map(|m| {
                let droid = &self.droids[m];
                let Some(side) = droid.side.filter(|_| !droid.down) else {
                    return (None, false);
                };
                let enemy = |foe: Foe| match foe {
                    Foe::Droid(n) => !self.droids[n].down && self.droids[n].side.is_some_and(|them| side.against(them)),
                    Foe::Rival(body) => rivals.iter().any(|r| r.collider == body && side.against(r.side)),
                };
                let candidates = (0..self.droids.len())
                    .map(Foe::Droid)
                    .chain(rivals.iter().map(|r| Foe::Rival(r.collider)))
                    .filter(|&foe| enemy(foe));
                let seen = candidates
                    .filter_map(|foe| {
                        let (feet, middle) = foe_place(foe)?;
                        let away = (middle - (droid.feet + Vector3::new(0.0, CHEST, 0.0))).norm();
                        let (body, notice) = match foe {
                            Foe::Droid(n) => (self.droids[n].collider, NOTICE_DROID),
                            Foe::Rival(body) => (body, NOTICE_RIVAL),
                        };
                        let looks = could_see_other(Some(Alert::Alert), droid.feet, droid.heading, feet, false);
                        ((looks || away <= notice) && in_sight_of_point(graph, droid, middle, body)).then_some((foe, away))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(foe, _)| foe);
                match (seen, droid.foe.filter(|&foe| enemy(foe))) {
                    (Some(foe), _) => (Some(foe), true),
                    (None, Some(foe)) if droid.foe_unseen < FOE_MEMORY => (Some(foe), false),
                    _ => (None, false),
                }
            })
            .collect();
        let mut engaged = Vec::new();
        let mut shots = Vec::new();

        // The search: lost from sight, the player could be anywhere they could have got to since,
        // and wherever the searchers look and do not see them, they are not.
        let seeing = self
            .droids
            .iter()
            .find(|droid| droid.alert == Some(Alert::Alert) && droid.sees_player && !droid.down);
        let cell_of = |at: Vector3<f32>| survey::cell_at(grid, origin, at).filter(|&(x, z)| grid.is_walkable(x, z));
        let mut search = self.search.take().unwrap_or_else(|| SearchMap::new(grid));
        if let Some(seeing) = seeing {
            self.last_sight = Some((seeing.lost_at, seeing.lost_going));
            self.search_from = None;
        } else if let Some((at, going)) = self.last_sight.take() {
            if let Some(cell) = cell_of(at) {
                let going = (going.norm() > 1.0e-3).then_some((going.x, going.z));
                search.lose(grid, cell, going);
            }
        } else if let Some(at) = self.search_from.take() {
            // Something new about where they are: search from there.
            if let Some(cell) = cell_of(at) {
                search.lose(grid, cell, None);
            }
        }
        let searching = self.droids.iter().any(|d| d.alert == Some(Alert::Evasion) && !d.down);
        if searching && !search.is_empty() {
            search.spread(grid, PLAYER_SPEED / grid.cell_size * dt);
            self.look_in -= dt;
            if self.look_in <= 0.0 {
                self.look_in = LOOK_EVERY;
                let sight = Sight::default();
                for droid in self.droids.iter().filter(|d| d.alert == Some(Alert::Evasion) && !d.down) {
                    if let Some(cell) = cell_of(droid.feet) {
                        let reach = SEARCH_SIGHT.min(sight.range) / grid.cell_size;
                        search.look(grid, cell, droid.heading, (reach, sight.cone, 2.0));
                    }
                }
            }
        }
        // Where each searcher is making for, to keep the others clear of it.
        let goals: Vec<Option<(usize, usize)>> = self
            .droids
            .iter()
            .map(|d| (d.alert == Some(Alert::Evasion)).then(|| d.route.first().and_then(|&g| cell_of(g))).flatten())
            .collect();
        // The droid after the player that is nearest them goes straight at them; the others make
        // for somewhere ahead of them, to cut them off.
        let nearest_chaser = self
            .droids
            .iter()
            .enumerate()
            .filter(|(_, d)| d.alert == Some(Alert::Alert) && !d.down && d.windup == 0.0)
            .min_by(|(_, a), (_, b)| flat(a.feet - player).norm().total_cmp(&flat(b.feet - player).norm()))
            .map(|(n, _)| n);
        let player_going = seeing.map(|d| d.lost_going);
        // Which of those leading have one of their pair or group to wait for: kept talking, or
        // fallen behind.
        let mut waits = vec![false; self.droids.len()];
        for droid in &self.droids {
            let Some(leader) = droid.partner.filter(|_| !droid.leads) else {
                continue;
            };
            let about = !droid.down && matches!(droid.alert, None | Some(Alert::Caution));
            let held = droid.talking || droid.warned > 0;
            if about && (held || flat(droid.feet - self.droids[leader].feet).norm() > WAIT_FOR) {
                waits[leader] = true;
            }
        }
        // A crowd is busier: it stands about less between trips.
        let rest = match self.crowd {
            Some(_) => CROWD_REST,
            None => REST,
        };
        // Each group's turf, for the others to keep off: the cell its spot is in, and how many
        // cells round it reaches.
        let turfs: Vec<((usize, usize), f32)> = self
            .droids
            .iter()
            .filter(|droid| !droid.down && (droid.leads || droid.partner.is_none()))
            .filter_map(|droid| {
                let (at, reach) = droid.turf?;
                Some((survey::cell_at(grid, origin, at)?, reach / grid.cell_size))
            })
            .collect();
        // The droids' own bodies, which are no cover.
        let bodies: Vec<Handle<Collider>> = self.droids.iter().map(|droid| droid.collider).collect();
        // The trains' doorways those taking them are getting on and off by.
        let mut taken: Vec<(usize, usize)> =
            self.droids.iter().filter_map(|droid| droid.ride.map(|ride| (ride.train, ride.doorway))).collect();
        for (me, droid) in self.droids.iter_mut().enumerate() {
            droid.avatar.fly_voxels(graph, dt);
            // In the hostile droid's colours while it is after the player; it goes down in
            // whichever it had on.
            let hostile = droid.alert.is_some();
            if !droid.down && droid.side.is_none() && hostile != droid.looks_hostile {
                if let Some(livery) = self.liveries.get(droid.character) {
                    recolour(graph, droid.avatar.root(), livery, hostile);
                }
                droid.looks_hostile = hostile;
            }
            // Limp, the physics has it: its bones follow its bodies, its capsule is gone, and it
            // is wherever its pelvis has got to.
            if let Some(ragdoll) = droid.ragdoll.as_mut() {
                ragdoll.update(graph);
                if ragdoll.is_limp() {
                    if graph.is_valid_handle(droid.collider) {
                        graph.remove_node(droid.collider);
                    }
                    if let Some(pelvis) = ragdoll.pelvis(graph) {
                        droid.feet.x = pelvis.x;
                        droid.feet.z = pelvis.z;
                    }
                    droid.speed = 0.0;
                    droid.route.clear();
                    continue;
                }
            }
            let (foe, sees_foe) = foes[me];
            // Shot at by someone it has not seen, it goes after where from for a while.
            if let Some((_, left)) = droid.shot_from.as_mut() {
                *left -= dt;
            }
            if sees_foe || droid.shot_from.is_some_and(|(_, left)| left <= 0.0) {
                droid.shot_from = None;
            }
            if foe.is_some() && droid.foe.is_none() {
                engaged.push(me);
                if matches!(foe, Some(Foe::Rival(_))) {
                    Log::info(format!("Capture the flag: droid {me} goes after a drone"));
                }
            }
            droid.foe = foe;
            droid.foe_unseen = if sees_foe { 0.0 } else { droid.foe_unseen + dt };
            droid.reload = (droid.reload - dt).max(0.0);
            droid.armed_left = (droid.armed_left - dt).max(0.0);
            // After the player, it goes straight for them rather than round them; after one of
            // the other side, straight for it - the player coming first.
            let hunting = droid.alert == Some(Alert::Alert);
            let fighting = droid.foe.filter(|_| !hunting);
            // After one running from it, it goes straight for it, to catch it.
            let chased = match droid.pursuit {
                Some(Pursuit::Chasing(other)) => Some(other),
                _ => None,
            };
            let (x, z) = square_of(droid.feet);
            let near: Vec<usize> = (-1..=1)
                .flat_map(|dx| (-1..=1).map(move |dz| (x + dx, z + dz)))
                .filter_map(|square| squares.get(&square))
                .flatten()
                .copied()
                .collect();
            let others = near
                .iter()
                .copied()
                .filter(|&other| {
                    other != me
                        && !down.get(other).copied().unwrap_or(false)
                        && !(hunting && other == the_player)
                        && Some(Foe::Droid(other)) != fighting
                        && Some(other) != chased
                })
                .map(|other| everyone[other]);
            // In capture the flag, whoever of the other side it sees to shoot at: the player
            // first, for the enemy's; and near enough, it stands to shoot.
            let target = match droid.side {
                Some(_) if hunting && droid.sees_player => Some(player + Vector3::new(0.0, PLAYER_CHEST, 0.0)),
                Some(_) if sees_foe => fighting.and_then(foe_place).map(|(_, middle)| middle),
                _ => None,
            }
            .filter(|&at| flat(at - droid.feet).norm() < FIRE_RANGE);
            let gang = droid.side == Some(Side::Gang);
            let engage = if gang { GANG_ENGAGE } else { ENGAGE };
            let standing_to_shoot = target.is_some_and(|at| flat(at - droid.feet).norm() < engage);
            if target.is_some() {
                droid.armed_left = ARMED_FOR;
                droid.aimed_at = target;
            }
            droid.resting = (droid.resting - dt).max(0.0);
            droid.windup = (droid.windup - dt).max(0.0);
            droid.deaf = (droid.deaf - dt).max(0.0);
            if droid.sees_player {
                // Where the player is, and roughly which way they are going, from how they have
                // moved lately.
                let moved = flat(player - droid.lost_at);
                droid.lost_going = droid.lost_going * (-4.0 * dt).exp() + moved;
                droid.lost_at = player;
            }
            // Seeing the player or not, a hostile droid goes into the phase that follows - once it
            // is done standing, having just turned hostile.
            if let Some(alert) = droid.alert.filter(|_| droid.windup == 0.0 && !droid.down) {
                if alert != Alert::Alert {
                    droid.search_left -= dt;
                }
                let next = alert.next(droid.sees_player, droid.search_left);
                // One guarding its side's end never calms down.
                let next = match next {
                    None if droid.side.is_some() => {
                        droid.search_left = CAUTION;
                        Some(Alert::Caution)
                    }
                    next => next,
                };
                droid.enter(me, next, &mut alerts);
            }
            // In battle royale, going about its business, it gives up on anywhere outside the
            // ring, for somewhere inside it.
            if let Some((middle, reach)) = self.ring.filter(|_| droid.alert != Some(Alert::Alert) && droid.foe.is_none()) {
                let out = |at: Vector3<f32>| flat(at - middle).norm() > reach;
                if droid.trip.is_none() && droid.route.first().map_or(out(droid.feet), |&end| out(end)) {
                    droid.route.clear();
                    droid.resting = 0.0;
                }
            }
            // Taking a ferry, it does as the trip says until it is across, whatever else it is
            // about: on its way to the ferry, it still works out its way again as it goes.
            let mut on_trip = false;
            // Keeping beside its partner, whether it has fallen behind.
            let mut catching_up = false;
            if let Some(trip) = droid.trip.as_mut().filter(|_| !droid.down) {
                match trip.step(droid.feet, &droid.route, crossings) {
                    Doing::Go(route) => {
                        if trip.stage != trip::Stage::ToDock {
                            droid.route = route;
                            on_trip = true;
                        }
                    }
                    Doing::Across => {
                        let to = trip.to;
                        droid.trip = None;
                        droid.route = route_to(droid.feet, to);
                        on_trip = !droid.route.is_empty();
                    }
                    Doing::GiveUp => {
                        droid.trip = None;
                        droid.route.clear();
                    }
                }
            }
            // Taking the train, it does as the ride says from the platform until it is off again,
            // whatever else it is about - but on its way up, or waiting, it gives it up for
            // anything else that comes up.
            if let Some(mut ride) = droid.ride.filter(|_| !droid.down) {
                let about_its_business = droid.alert.is_none()
                    && droid.pursuit.is_none()
                    && droid.cover.is_none()
                    && droid.witness.is_none()
                    && droid.foe.is_none()
                    && !droid.talking
                    && droid.warned == 0;
                let train = self.trains.get(ride.train).filter(|_| about_its_business || ride.aboard());
                match train.map(|train| (ride.step(droid.feet, &droid.route, train, dt), train)) {
                    Some((Rides::Own, _)) => droid.ride = Some(ride),
                    Some((Rides::Go(route), train)) => {
                        droid.route = route;
                        on_trip = true;
                        // Waiting, it faces the doorway; aboard, it looks out of it.
                        let doorway = train.at + train.doorways[ride.doorway];
                        let facing = match ride.stage {
                            Riding::Waiting => Some(flat(doorway - droid.feet)),
                            Riding::Aboard => Some(Vector3::new(doorway.x - train.at.x, 0.0, 0.0)),
                            _ => None,
                        };
                        if let Some(to) = facing.filter(|to| to.norm() > 1.0e-3 && droid.route.is_empty()) {
                            droid.heading = to.x.atan2(to.z);
                        }
                        droid.ride = Some(ride);
                    }
                    rides => {
                        // The train gone from under it, it is left where it is.
                        if rides.is_none() && ride.aboard() {
                            Log::warn(format!("Maze: droid {me} left off the grid, the train gone"));
                        }
                        droid.ride = None;
                        droid.route.clear();
                        taken.retain(|&n| n != (ride.train, ride.doorway));
                        if about_its_business {
                            droid.resting = between(rng, REST);
                        }
                    }
                }
            }
            // Stopped, it stays where it went down - aboard the train, going along with it.
            if droid.down {
                droid.route.clear();
                droid.trip = None;
                droid.ride = droid.ride.filter(|ride| ride.stage == Riding::Aboard);
            } else if on_trip {
                // Waiting for the ferry, it faces where it will come.
                if let Some(crossing) = droid.trip.filter(|trip| trip.stage == trip::Stage::Waiting).and_then(|trip| crossings.get(trip.ferry).map(|c| c.ends[trip.from])) {
                    let to = flat(crossing - droid.feet);
                    if to.norm() > 1.0e-3 {
                        droid.heading = to.x.atan2(to.z);
                    }
                }
            // Talking, or warning them off, it stands and faces the player, and rests a moment
            // once they are done.
            } else if let Some(pursuit) = droid.pursuit {
                droid.replan -= dt;
                match pursuit {
                    // After the other, the way to it worked out again every so often.
                    Pursuit::Chasing(other) => {
                        if let Some((feet, _)) = placed.get(other).copied().flatten() {
                            // With no way there last time, only once it is time to look again.
                            if droid.replan <= 0.0 || droid.route.is_empty() && !droid.no_way {
                                droid.replan = REPLAN;
                                droid.route = route_to(droid.feet, feet);
                                droid.no_way = droid.route.is_empty();
                            }
                        }
                    }
                    // Running from the other: somewhere well ahead, away from it.
                    Pursuit::Fleeing(other) => {
                        let away = placed
                            .get(other)
                            .copied()
                            .flatten()
                            .and_then(|(feet, _)| flat(droid.feet - feet).try_normalize(1.0e-3));
                        if let Some(away) = away.filter(|_| droid.replan <= 0.0 || droid.route.is_empty() && !droid.no_way) {
                            droid.replan = REPLAN * 4.0;
                            let ahead = droid.feet + away * FLEE_AHEAD;
                            let to = spot_near((grid, origin), ahead, FLEE_AHEAD * 0.4, rng).unwrap_or(ahead);
                            droid.route = route_to(droid.feet, to);
                            droid.no_way = droid.route.is_empty();
                        }
                    }
                    // Caught, or holding the one caught: standing, facing the other.
                    Pursuit::Holding(other) | Pursuit::Held(other) => {
                        droid.route.clear();
                        if let Some(to) = placed
                            .get(other)
                            .copied()
                            .flatten()
                            .map(|(feet, _)| flat(feet - droid.feet))
                            .filter(|to| to.norm() > 1.0e-3)
                        {
                            droid.heading = to.x.atan2(to.z);
                        }
                    }
                }
            } else if droid.talking || droid.warned > 0 {
                droid.route.clear();
                droid.waiting = 0.0;
                droid.resting = droid.resting.max(REST.0);
                let them = droid.with.and_then(|k| everyone.get(k)).map_or(player, |&(feet, _)| feet);
                let to_them = flat(them - droid.feet);
                if to_them.norm() > 1.0e-3 {
                    droid.heading = to_them.x.atan2(to_them.z);
                }
            } else if droid.alert.is_some() && droid.windup > 0.0 {
                // Just turned hostile, it stands a moment where it is.
                droid.route.clear();
            } else if droid.alert == Some(Alert::Alert) {
                // After the player, the way to them worked out again every so often as they move.
                droid.replan -= dt;
                // Not while it stands to shoot, which leaves it no route; and with no way there
                // last time, only once it is time to look again.
                if !standing_to_shoot && (droid.replan <= 0.0 || droid.route.is_empty() && !droid.no_way) {
                    droid.replan = REPLAN;
                    let away = flat(player - droid.feet).norm();
                    let cut_off = player_going
                        .filter(|_| Some(me) != nearest_chaser && away > CLOSE_IN)
                        .and_then(|going| going.try_normalize(1.0e-3))
                        .map(|going| player + going * (away * 0.6).min(CUT_OFF))
                        .filter(|&ahead| (1..=4).all(|i| floor_at(player + (ahead - player) * (i as f32 / 4.0))));
                    (droid.route, droid.trip) = go_to(droid.feet, cut_off.unwrap_or(player));
                    droid.no_way = droid.route.is_empty();
                }
            } else if let Some((feet, _)) = fighting.and_then(foe_place) {
                // After one of the other side, the way to it worked out again every so often.
                droid.replan -= dt;
                // As after the player: not while it stands to shoot, nor before it is time again.
                if !standing_to_shoot && (droid.replan <= 0.0 || droid.route.is_empty() && !droid.no_way) {
                    droid.replan = REPLAN;
                    (droid.route, droid.trip) = go_to(droid.feet, feet);
                    droid.no_way = droid.route.is_empty();
                }
            } else if let Some((from, _)) = droid.shot_from.filter(|_| droid.side.is_some()) {
                // Shot at by someone it did not see: after them, where the shot came from.
                droid.replan -= dt;
                // As after the player: not while it stands to shoot, nor before it is time again.
                if !standing_to_shoot && (droid.replan <= 0.0 || droid.route.is_empty() && !droid.no_way) {
                    droid.replan = REPLAN;
                    (droid.route, droid.trip) = go_to(droid.feet, from);
                    droid.no_way = droid.route.is_empty();
                }
            } else if droid.alert == Some(Alert::Evasion) {
                if droid.route.is_empty() {
                    match droid.looking {
                        // Looking about where it is, one way and then the other.
                        Some(left) if left > 0.0 => {
                            droid.looking = Some(left - dt);
                            let turn = std::f32::consts::TAU * (LOOK_ABOUT - left) / LOOK_ABOUT;
                            droid.heading = droid.look_from + LOOK_SWEEP * turn.sin();
                        }
                        // Done looking: on to where the player is likeliest to be, of where it
                        // can get to, clear of where the others are going - or, with nowhere
                        // likelier than anywhere else, another spot nearby.
                        Some(_) => {
                            droid.looking = None;
                            droid.searched += 1;
                            let taken: Vec<(usize, usize)> = goals
                                .iter()
                                .enumerate()
                                .filter(|&(n, _)| n != me)
                                .filter_map(|(_, g)| *g)
                                .collect();
                            let next = cell_of(droid.feet).filter(|_| !search.is_empty()).and_then(|from| {
                                search.best(grid, from, &taken, SEARCH_APART / grid.cell_size, CHASE_REACH)
                            });
                            (droid.route, droid.trip) = match next {
                                Some(cell) => go_to(droid.feet, grid.on_floor(origin, cell)),
                                None => (Vec::new(), None),
                            };
                            if droid.route.is_empty() {
                                droid.route = plan((grid, origin), droid.feet, SEARCH_TRIP, rng);
                            }
                        }
                        // First where the player was going when it last saw them, if there is
                        // floor all the way there, or else where it saw them.
                        None if droid.searched == 0 => {
                            droid.searched = 1;
                            let guess = droid
                                .lost_going
                                .try_normalize(1.0e-3)
                                .map(|going| droid.lost_at + going * GUESS)
                                .filter(|&guess| {
                                    let (from, way) = (droid.lost_at, guess - droid.lost_at);
                                    (1..=4).all(|i| floor_at(from + way * (i as f32 / 4.0)))
                                })
                                .unwrap_or(droid.lost_at);
                            (droid.route, droid.trip) = go_to(droid.feet, guess);
                        }
                        // Got there, or has nowhere to go: it looks about.
                        None => {
                            droid.looking = Some(LOOK_ABOUT);
                            droid.look_from = droid.heading;
                        }
                    }
                }
            } else if let Some(sentry) = droid.witness.and_then(|witness| witness.to) {
                // A witness, on its way to tell a sentry: the way to it worked out again every
                // so often as it walks about.
                droid.replan -= dt;
                if let Some((feet, _)) = placed.get(sentry).copied().flatten() {
                    if droid.replan <= 0.0 || droid.route.is_empty() {
                        droid.replan = REPLAN * 2.0;
                        (droid.route, droid.trip) = go_to(droid.feet, feet);
                    }
                }
            } else if let Some((danger, picked)) = droid.cover {
                // A bystander to a shootout: running from it to somewhere it cannot be seen
                // from, if there is anywhere about, or else as far off as it can; and there,
                // crouched, it faces the way of it.
                if !picked {
                    let away = flat(droid.feet - danger)
                        .try_normalize(1.0e-3)
                        .map_or(droid.heading, |away| away.x.atan2(away.z));
                    let open = |at: Vector3<f32>| {
                        let (from, to) = (danger + Vector3::new(0.0, CHEST, 0.0), at + Vector3::new(0.0, CHEST, 0.0));
                        let way = to - from;
                        let length = way.norm();
                        let mut hits = Vec::new();
                        graph.physics.cast_ray(
                            RayCastOptions {
                                ray_origin: Point3::from(from),
                                ray_direction: way / length.max(1.0e-3),
                                max_len: length,
                                groups: Default::default(),
                                sort_results: false,
                            },
                            &mut hits,
                        );
                        !hits.iter().any(|hit| !bodies.contains(&hit.collider))
                    };
                    // Straight away from it first, and then further and further to either side.
                    let spots: Vec<Vector3<f32>> = (0..COVER_TRIES)
                        .filter_map(|i| {
                            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
                            let turn = side * (i as f32 / 2.0).ceil() * std::f32::consts::FRAC_PI_4 * 0.75;
                            spot_near((grid, origin), droid.feet + forward(away + turn) * COVER_RUN, COVER_RUN * 0.3, rng)
                        })
                        .filter(|&spot| flat(spot - danger).norm() > flat(droid.feet - danger).norm())
                        .collect();
                    let to = spots.iter().copied().find(|&spot| !open(spot)).or_else(|| {
                        spots.iter().copied().max_by(|a, b| flat(*a - danger).norm().total_cmp(&flat(*b - danger).norm()))
                    });
                    droid.route = to.map_or_else(Vec::new, |to| route_to(droid.feet, to));
                    droid.cover = Some((danger, true));
                } else if droid.route.is_empty() {
                    let to = flat(danger - droid.feet);
                    if to.norm() > 1.0e-3 {
                        droid.heading = to.x.atan2(to.z);
                    }
                }
            } else if let Some(leader) = partners[me].filter(|leader| leader.about && !droid.leads) {
                // Keeping beside its partner, on its right and a step behind - or, one of a
                // group, wherever about the one leading it it keeps (see [`ABOUT_LEADER`]) - where
                // that is going to be in a moment, as it walks; or else behind it, with no floor
                // off the road there.
                let ahead = forward(leader.heading) * leader.speed * LEAD;
                let keeps_off = |at: Vector3<f32>| {
                    floor_at(at)
                        && !survey::cell_at(grid, origin, at).is_some_and(|(x, z)| roads.is_some_and(|r| r.is_road(x, z)))
                };
                let behind = leader.feet + ahead - forward(leader.heading) * (BESIDE.0 + 0.8 * droid.slot as f32);
                let slot = [about_leader(leader.feet + ahead, leader.heading, droid.slot), behind]
                    .into_iter()
                    .find(|&slot| keeps_off(slot))
                    .unwrap_or(behind);
                droid.replan -= dt;
                let off = flat(slot - droid.feet).norm();
                if off < REACHED && leader.speed < 0.1 {
                    // There, and the other standing: it stands too, facing the same way.
                    droid.route.clear();
                    droid.heading = leader.heading;
                } else if droid.replan <= 0.0 || droid.route.is_empty() && !droid.no_way {
                    droid.replan = REPLAN;
                    droid.route = route_to(droid.feet, slot);
                    droid.no_way = droid.route.is_empty();
                }
                catching_up = off > CATCH_UP;
            } else if droid.route.is_empty() {
                if let Some(aside) = step_aside(droid.feet, others.clone(), floor_at) {
                    droid.route = vec![aside];
                } else if droid.resting == 0.0 {
                    (droid.route, droid.trip) = match droid.post {
                        // In capture the flag, about where it keeps to.
                        Some(post) => spot_near((grid, origin), post, POST_REACH, rng)
                            .map(|to| go_to(droid.feet, to))
                            .unwrap_or_default(),
                        // In battle royale, somewhere inside the ring, well in from its edge.
                        None => match self.ring {
                            // Not too far to find the way, up in a building as likely as not:
                            // round where it is, if that is in the ring; or else on its way in.
                            Some((middle, reach)) => {
                                let to_middle = flat(middle - droid.feet);
                                let off = to_middle.norm();
                                let around = if off < reach * 0.8 {
                                    droid.feet
                                } else {
                                    droid.feet + to_middle * ((off - reach * 0.5).min(RING_WANDER) / off)
                                };
                                (0..6)
                                    .filter_map(|_| spot_on_any_floor_near((grid, origin), around, RING_WANDER, rng))
                                    .find(|&to| flat(to - middle).norm() < reach * 0.9)
                                    .map(|to| go_to(droid.feet, to))
                                    .unwrap_or_default()
                            }
                            // Keeping to its turf, about its spot; or else anywhere off the
                            // others' turf.
                            None => match droid.turf {
                                Some((at, _)) => (
                                    spot_near((grid, origin), at, TURF_KEEP, rng)
                                        .map_or_else(Vec::new, |to| route_to(droid.feet, to)),
                                    None,
                                ),
                                // Now and then, near the station, taking the train.
                                // Not one of a kind that keeps in groups, gone off by itself:
                                // it is to make a group afresh with others of its kind.
                                None => match (!self.trains.is_empty()
                                    && droid.partner.is_none()
                                    && self.groupings.get(droid.character).is_none_or(|g| g.groups_of < 2)
                                    && droid.side.is_none()
                                    && !droid.sentry
                                    && rng.below(TRAIN_CHANCE) == 0)
                                    .then(|| take_train((grid, origin), droid.feet, &self.trains, &taken, rng))
                                    .flatten()
                                {
                                    Some((route, ride)) => {
                                        taken.push((ride.train, ride.doorway));
                                        droid.ride = Some(ride);
                                        (route, None)
                                    }
                                    None => (wander((grid, origin), droid.feet, TRIP, roads, &turfs, &self.reachable, rng), None),
                                },
                            },
                        },
                    };
                    if droid.route.is_empty() {
                        droid.resting = REST.0;
                    }
                }
            }
            // Near enough to shoot, it stands and faces them.
            if standing_to_shoot && !droid.down {
                droid.route.clear();
                if let Some(to) = target.map(|at| flat(at - droid.feet)).filter(|to| to.norm() > 1.0e-3) {
                    droid.heading = to.x.atan2(to.z);
                }
            }
            // Running after the player, and jogging to where it lost them.
            let running = matches!(droid.pursuit, Some(Pursuit::Chasing(_) | Pursuit::Fleeing(_)));
            // Taking cover, it runs for it, and once there crouches in it.
            let sheltering = droid.cover.is_some() && droid.alert.is_none() && !droid.talking;
            let sheltered = sheltering && droid.route.is_empty();
            let reporting = droid.witness.is_some_and(|witness| witness.to.is_some());
            let after = droid.alert == Some(Alert::Alert) || fighting.is_some() || running || sheltering || reporting;
            // Called to a shootout, it hurries there.
            let called = matches!(droid.side, Some(Side::Security | Side::Gang))
                && droid.post.is_some_and(|post| flat(post - droid.feet).norm() > POST_REACH * 2.0);
            let hurrying = after
                || called
                || catching_up
                || (droid.alert == Some(Alert::Evasion) && droid.searched <= 1)
                || droid.trip.is_some_and(|trip| trip.hurrying())
                || droid.ride.is_some_and(|ride| matches!(ride.stage, Riding::Boarding | Riding::Leaving));
            // After the player, it sprints now and then, when it is not to be told.
            let chasing = (droid.alert == Some(Alert::Alert) && droid.windup == 0.0 || running) && !droid.down;
            (droid.sprint_in, droid.sprinting) =
                sprints(droid.sprint_in, droid.sprinting, chasing, rng, dt);
            // A sentry paces itself: its sprints kept for while it has breath to spare, easing
            // off to get some back when it is low - all it has left spent only on a last burst,
            // whoever it is after near enough to catch.
            let quarry = match droid.pursuit {
                Some(Pursuit::Chasing(other)) => placed.get(other).copied().flatten().map(|(feet, _)| feet),
                _ => (hunting && droid.sees_player).then_some(player),
            };
            let burst = quarry.is_some_and(|at| flat(at - droid.feet).norm() < BURST);
            let saving;
            (droid.easing, saving) = pace(droid.sentry && !burst, droid.easing, droid.stamina / droid.most);
            if saving {
                droid.sprinting = 0.0;
            }
            // In a foot chase, both flat out for as long as they have the breath - or, a sentry,
            // as long as it has breath to spare.
            if running && !droid.winded && !saving {
                droid.sprinting = droid.sprinting.max(dt * 2.0);
            }
            // Out of breath, it walks until enough is back, and a sprint it was due waits.
            let gait = match (hurrying, droid.sprinting > 0.0) {
                _ if droid.winded => Gait::Walking,
                (true, _) if droid.easing => Gait::Jogging,
                (true, true) => Gait::Sprinting,
                (true, false) if after => Gait::Running,
                (true, false) => Gait::Jogging,
                (false, _) => Gait::Walking,
            };
            let moving = droid.speed > 0.1 && !droid.down;
            let breathing = if moving { gait } else { Gait::Walking };
            droid.sentry = sentry(droid.character);
            let (most, recovery) = match droid.sentry {
                true => (SENTRY_STAMINA, SENTRY_RECOVERY),
                false => (1.0, 1.0),
            };
            droid.most = most;
            droid.stamina = droid.stamina.min(most);
            (droid.stamina, droid.winded) =
                breath::breathe(droid.stamina, droid.winded, breathing, dt, most, recovery);
            if droid.winded {
                droid.sprinting = 0.0;
            }
            // Past each point on the way, bar the last, as soon as it is near - or already
            // behind, having been put off course making way for someone.
            while let [.., after, next] = droid.route[..] {
                if flat(next - droid.feet).norm() < REACHED
                    || flat(after - droid.feet).norm() < flat(after - next).norm()
                {
                    droid.route.pop();
                } else {
                    break;
                }
            }

            // Leading the way, it waits for its partner or group, kept talking or fallen behind.
            let waiting_for_partner = droid.leads && matches!(droid.alert, None | Some(Alert::Caution)) && waits[me];
            let mut wanted = 0.0;
            if let Some(&next) = droid.route.last().filter(|_| !waiting_for_partner) {
                let to = flat(next - droid.feet);
                if droid.route.len() == 1 && to.norm() < ARRIVED {
                    droid.route.clear();
                    droid.resting = between(rng, if droid.turf.is_some() { TURF_REST } else { rest });
                } else {
                    let (way, blocked) = make_way(droid.feet, to.normalize(), others, floor_at);
                    droid.heading = way.x.atan2(way.z);
                    if blocked {
                        droid.waiting += dt;
                        if droid.waiting > PATIENCE {
                            droid.route.clear();
                            droid.waiting = 0.0;
                            droid.resting = between(rng, (0.5, 2.0));
                        }
                    } else {
                        droid.waiting = 0.0;
                        let fallback = match gait {
                            Gait::Sprinting => FALLBACK_SPRINT,
                            Gait::Running => FALLBACK_RUNNING,
                            Gait::Walking => FALLBACK_PACE,
                            Gait::Jogging => FALLBACK_RUN,
                        };
                        let pace = droid
                            .avatar
                            .pace(Posture::Standing, gait)
                            .unwrap_or(fallback);
                        // Slower the further it has yet to turn, and slowing down to stop at the
                        // end of the way.
                        let off = droid.heading - droid.avatar.facing();
                        // How much of the way is left, as far as it matters: no further than it
                        // takes to stop from its pace, from the next point on.
                        let stopping = pace * pace / (2.0 * ACCELERATION);
                        let mut left = to.norm();
                        for pair in droid.route.windows(2).rev() {
                            if left >= stopping {
                                break;
                            }
                            left += flat(pair[0] - pair[1]).norm();
                        }
                        wanted =
                            (pace * off.cos().max(0.0)).min((2.0 * ACCELERATION * left).sqrt());
                    }
                }
            }
            let step = ACCELERATION * dt;
            droid.speed += (wanted - droid.speed).clamp(-step, step);
            // On a ferry, it goes along with it.
            let carried = crossings
                .iter()
                .find(|crossing| crossing.carries(droid.feet))
                .map_or(Vector3::zeros(), |crossing| crossing.velocity);
            droid.feet += carried * dt;
            // It keeps to floor it can walk: never off an edge, nor up or down more than a stair's
            // step - along the edge instead, if it can, or not at all. Getting on a ferry or off
            // it, it goes where the trip takes it.
            let step = forward(droid.heading) * (droid.speed * dt);
            let off_the_grid = droid.trip.is_some_and(|trip| trip.off_the_grid()) || droid.aboard();
            let fits = |step: Vector3<f32>| off_the_grid || trip::can_step(grid, origin, droid.feet, droid.feet + step);
            match [step, Vector3::new(step.x, 0.0, 0.0), Vector3::new(0.0, 0.0, step.z)].into_iter().find(|&step| fits(step)) {
                Some(step) => droid.feet += step,
                // Nowhere to go from here: kept waiting, as by someone in its way, it goes
                // somewhere else in the end.
                None => {
                    droid.speed = 0.0;
                    droid.waiting += dt;
                    if droid.waiting > PATIENCE && droid.trip.is_none() {
                        droid.route.clear();
                        droid.waiting = 0.0;
                    }
                }
            }
            // Aboard the train, it goes along with it, standing where it stands, on its floor.
            let train = droid.ride.and_then(|ride| self.trains.get(ride.train)).filter(|_| droid.aboard());
            if let (Some(train), Some(ride)) = (train, droid.ride.filter(|ride| ride.stage == Riding::Aboard)) {
                droid.feet = train.at + ride.spot;
                droid.speed = 0.0;
            }
            match crossings.iter().find(|crossing| crossing.carries(droid.feet)) {
                Some(crossing) => droid.feet.y = crossing.deck.y,
                None => match train.filter(|train| train.within(droid.feet)) {
                    Some(train) => droid.feet.y = train.floor(),
                    None => {
                        if let Some((x, z)) = survey::cell_at(grid, origin, droid.feet) {
                            if grid.is_walkable(x, z) {
                                let floor = grid.floor(x, z);
                                droid.feet.y += (floor - droid.feet.y) * (1.0 - (-FLOOR_EASING * dt).exp());
                            }
                        }
                    }
                },
            }

            let to_player = player - droid.feet;
            if droid.alert == Some(Alert::Alert)
                && droid.side.is_none()
                && droid.windup == 0.0
                && droid.sees_player
                && flat(to_player).norm() < CATCH
                && to_player.y.abs() < CATCH_HEIGHT
            {
                caught = caught.or(Some(me));
                // Having hit them, it stands a moment before coming on again.
                droid.windup = HIT_PAUSE;
                droid.speed = 0.0;
            }

            // Far off, and going about its business, it is moved and posed in the scene only now
            // and then (see [`POSE_EVERY`]): every node of a droid that moves has to be worked out
            // afresh, and in a city of hundreds that costs the engine more than all else. Where it
            // is goes on all the same. Near, hunting, or fighting, every frame.
            droid.unposed += dt;
            let away = flat(player - droid.feet).norm();
            let busy = target.is_some()
                || droid.armed_left > 0.0
                || droid.alert.is_some()
                || droid.talking
                || droid.pursuit.is_some()
                || droid.cover.is_some()
                || droid.aboard();
            let every = match busy {
                true => 0.0,
                false => POSE_EVERY.iter().find(|&&(within, _)| away < within).map_or(0.0, |&(_, every)| every),
            };
            let posing = droid.unposed >= every;
            droid.avatar.set_eye_lamp(graph, away < EYE_LAMP_NEAR);
            if posing {
                if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(droid.body) {
                    body.set_next_kinematic_translation(droid.feet + Vector3::new(0.0, MIDDLE, 0.0));
                }
            }
            // Speaking to the player, its head turned to them.
            droid.speaking = (droid.speaking - dt).max(0.0);
            let to_player = flat(player - droid.feet);
            let head = (droid.speaking > 0.0 && !droid.down && to_player.norm() > 1.0e-3)
                .then(|| to_player.x.atan2(to_player.z) - droid.heading);
            if posing {
                droid.avatar.turn_head(head);
            }
            // Firing when it can, facing them near enough; looking up or down at them.
            let facing_them = target.and_then(|at| {
                let to = flat(at - droid.feet);
                let off = (to.x.atan2(to.z) - droid.heading + std::f32::consts::PI)
                    .rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                (off.abs() < FIRE_CONE).then_some(at)
            });
            let trigger = facing_them.is_some() && droid.reload == 0.0 && !droid.down;
            if trigger {
                droid.reload = between(rng, if gang { GANG_FIRE_EVERY } else { FIRE_EVERY });
            }
            // Its body is never turned - only the model's root is, by the heading - so the way to
            // them is in the world's terms, as the heading is, not ahead of where it faces.
            // Holding one caught, its pistol on it.
            let held = match droid.pursuit {
                Some(Pursuit::Holding(other)) => placed.get(other).copied().flatten().map(|(_, chest)| chest),
                _ => None,
            };
            if held.is_some() {
                droid.armed_left = droid.armed_left.max(dt * 2.0);
            }
            let look = target.or(held).map_or((0.0, droid.heading), |at| {
                let from = droid.feet + Vector3::new(0.0, FACE_HEIGHT, 0.0);
                let to = at - from;
                (to.y.atan2(flat(to).norm()), to.x.atan2(to.z))
            });
            let going = Going {
                heading: Some(droid.heading),
                speed: droid.speed,
                posture: match droid.down || sheltered {
                    true => Posture::Crouching,
                    false => Posture::Standing,
                },
                gait,
                grounded: true,
                jumped: false,
                low: false,
                falling: 0.0,
                lifted: carried.y * dt,
                cover: None,
                corner: false,
                peeking: false,
                pushing: true,
                strafing: false,
                armed: droid.armed_left > 0.0 && !droid.down,
                rifle: false,
                trigger,
                raised: target.is_some() || held.is_some(),
                look,
                way: 0.0,
            };
            if posing {
                droid.avatar.animate(graph, going, droid.unposed);
                droid.unposed = 0.0;
            }
            // A shot leaves the muzzle as the pistol fires, at what it aimed at, give or take.
            if let (Some((from, _)), Some(at), Some(side)) = (droid.avatar.shot(), droid.aimed_at, droid.side) {
                let wide = SPREAD * (at - from).norm() / 10.0;
                let miss = Vector3::new(
                    between(rng, (-wide, wide)),
                    between(rng, (-0.6 * wide, 0.6 * wide)),
                    between(rng, (-wide, wide)),
                );
                if let Some(way) = (at + miss - from).try_normalize(1.0e-3) {
                    shots.push((from, way, side));
                }
            }
        }
        self.search = Some(search);
        News {
            hit: caught,
            alerts,
            heard,
            alarmed,
            engaged,
            shots,
        }
    }

    /// Has each hostile droid look for the player, at `player` in `posture`, whom it can see
    /// wherever `in_sight` says nothing is in the way from the player to it, and [`Sight`] says
    /// they are in front of it and near enough - `in_the_dark` or not.
    pub fn look_for_player(
        &mut self,
        player: Vector3<f32>,
        posture: Posture,
        in_the_dark: bool,
        in_sight: impl Fn(Vector3<f32>) -> bool,
    ) {
        for droid in &mut self.droids {
            droid.sees_player = match droid.alert {
                Some(alert) if !droid.down && droid.hunts_player() => {
                    Sight::default().could_see(
                        Some(alert),
                        droid.feet,
                        droid.heading,
                        player,
                        posture.into(),
                        in_the_dark,
                    ) && in_sight(droid.feet)
                }
                _ => false,
            };
        }
    }

    /// Has each sentry - each droid whose kind, as an index into the conversations' characters,
    /// `sentry` says is one - that sees another sentry after the player go after them too, as if
    /// it saw them itself. It sees the other as [`could_see_other`] has it, `in_the_dark` or not,
    /// and as long as nothing is in the way. Calm, it stands a moment first, as one does that has
    /// just turned hostile. Call it after [`Self::look_for_player`]: those after the player are the
    /// ones that can see them, or see another that can.
    pub fn join_chases(
        &mut self,
        graph: &Graph,
        in_the_dark: bool,
        sentry: impl Fn(usize) -> bool,
    ) {
        let watches: Vec<Watch> = self
            .droids
            .iter()
            .map(|droid| Watch {
                sentry: sentry(droid.character),
                alert: droid.alert,
                sees_player: droid.sees_player,
                busy: droid.down || droid.talking || !droid.hunts_player(),
                windup: droid.windup,
                feet: droid.feet,
                heading: droid.heading,
            })
            .collect();
        let droids = &self.droids;
        let joining = joiners(&watches, in_the_dark, |m, n| {
            in_sight_of(graph, &droids[m], &droids[n])
        });
        for m in joining {
            let droid = &mut self.droids[m];
            if droid.alert.is_none() {
                droid.windup = WINDUP;
                droid.threat = 0.0;
                droid.warned = 0;
            }
            droid.sees_player = true;
            droid.enter(m, Some(Alert::Alert), &mut self.alerts);
        }
    }

    /// Has each droid that is not hostile feel the pistol pointed at it for another `dt`, or
    /// not: by the player looking from `eye` along `ahead` with it out, if they are - and only
    /// if it can see them do it. They are at `watched`: their feet, how they hold themselves and
    /// whether it is too dark to see them far; it sees them as a calm droid sees anyone (see
    /// [`Sight`]), facing them, as long as `in_sight` says nothing is in the way from the player
    /// to the droid. Its back to them, it never knows. `patience` says how
    /// long each kind of droid - as an index into the conversations' characters - stands for
    /// it; none, it pays it no heed. Some kinds take the player standing over them the same way:
    /// within as many meters as `crowded` says, or on their turf (see [`Inhabitant::turf`]), as
    /// long as nothing is in the way. Which droids went on to another stage.
    pub fn feel_aimed_at(
        &mut self,
        aim: Option<(Vector3<f32>, Vector3<f32>)>,
        watched: (Vector3<f32>, Posture, bool),
        in_sight: impl Fn(Vector3<f32>) -> bool,
        patience: impl Fn(usize) -> Option<f32>,
        crowded: impl Fn(usize) -> Option<f32>,
        dt: f32,
    ) -> Vec<(usize, Threat)> {
        let (player, posture, in_the_dark) = watched;
        // A pair or group takes the player standing over any of it as one, through the one that
        // speaks for it: none of the rest warn them as well.
        let mut crowding = vec![false; self.droids.len()];
        for (m, droid) in self.droids.iter().enumerate() {
            // Standing over it, or on its turf, where it can see them - but for one keeping to a
            // post, in its den or its precinct, which has visitors.
            let over = droid.post.is_none()
                && crowded(droid.character).is_some_and(|reach| flat(player - droid.feet).norm() < reach)
                && (player.y - droid.feet.y).abs() < CATCH_HEIGHT;
            let trespassing = droid
                .turf
                .is_some_and(|(at, reach)| flat(player - at).norm() < reach && (player.y - at.y).abs() < CATCH_HEIGHT);
            let near = !droid.down && (over || trespassing) && in_sight(droid.feet);
            if let Some(voice) = near.then(|| self.group_of(m).into_iter().find(|&v| !self.droids[v].down)).flatten() {
                crowding[voice] = true;
            }
        }
        let mut stages = Vec::new();
        for (n, droid) in self.droids.iter_mut().enumerate() {
            let Some(patience) = patience(droid.character).filter(|_| droid.hunts_player()) else {
                continue;
            };
            let minding = droid.alert.is_none() && !droid.down && !droid.talking;
            let chest = droid.feet + Vector3::new(0.0, CHEST, 0.0);
            let aimed = minding
                && aim.is_some_and(|(eye, ahead)| pointed_at(eye, ahead, chest))
                && Sight::default().could_see(None, droid.feet, droid.heading, player, posture.into(), in_the_dark)
                && in_sight(droid.feet);
            let aimed = aimed || (minding && crowding[n]);
            if !minding {
                droid.threat = 0.0;
                droid.warned = 0;
                continue;
            }
            // Worse only while it is pointed at, and the same while it is not - until it has been
            // off it long enough to calm down.
            if aimed {
                droid.threat += dt;
                droid.unaimed = 0.0;
            } else {
                droid.unaimed += dt;
            }
            if droid.threat >= stage_after(droid.warned, patience) {
                droid.threat = 0.0;
                droid.warned += 1;
                let stage = match droid.warned {
                    1 => Threat::Warned,
                    2 => Threat::WarnedAgain,
                    _ => Threat::Provoked,
                };
                if stage == Threat::Provoked {
                    droid.warned = 0;
                }
                stages.push((n, stage));
            } else if droid.unaimed >= CALM && (droid.warned > 0 || droid.threat > 0.0) {
                droid.threat = 0.0;
                if droid.warned > 0 {
                    droid.warned = 0;
                    stages.push((n, Threat::Calmed));
                }
            }
        }
        stages
    }

    /// Provokes the `n`th droid at once, as if it had stood for the pistol as long as it will:
    /// shot at, say. True if it was one that minds.
    pub fn provoke(&mut self, n: usize) -> bool {
        match self.droids.get_mut(n) {
            Some(droid) if droid.alert.is_none() && !droid.down => {
                droid.threat = 0.0;
                droid.warned = 0;
                true
            }
            _ => false,
        }
    }

    /// The `n`th droid sounds the alarm, with the player at `player`: every droid near it that
    /// `answers`, by its kind, and is not already after them, searches where they are.
    pub fn raise_alarm(&mut self, n: usize, player: Vector3<f32>, answers: impl Fn(usize) -> bool) {
        let Some(from) = self.droids.get(n).map(|droid| droid.feet) else {
            return;
        };
        for (m, droid) in self.droids.iter_mut().enumerate() {
            let answering = m != n
                && droid.hunts_player()
                && answers(droid.character)
                && !droid.down
                && matches!(droid.alert, None | Some(Alert::Caution))
                && (droid.feet - from).norm() < ALARM_RANGE;
            if !answering {
                continue;
            }
            droid.talking = false;
            droid.warned = 0;
            droid.threat = 0.0;
            droid.lost_at = player;
            droid.lost_going = Vector3::zeros();
            droid.alert = None;
            droid.enter(m, Some(Alert::Evasion), &mut self.alerts);
            self.alarmed.push(m);
            // Where the player was, as the alarm had it, is where the search starts from.
            self.search_from = Some(player);
        }
    }

    /// The player at `player`, their body `them`, has shot the `n`th droid. Those about it that
    /// could see it done - the one shot, or the player - within [`WITNESS_REACH`], `in_the_dark`
    /// or not, are witnesses to it: going about their business, or searching, but not after the
    /// player already, nor busy with a shootout or a chase, and of a kind that `minds`, by its
    /// kind and the one shot's. Them, as indices, each with whether it knows who did it: whether
    /// it saw the player within [`IDENTIFY_REACH`] of it. One shot down is the player's doing,
    /// for whoever comes across it lying there (see [`Inhabitants::find_bodies`]) - unless a
    /// witness has seen it done already.
    pub fn witnesses(
        &mut self,
        graph: &Graph,
        n: usize,
        (player, them): (Vector3<f32>, Handle<Collider>),
        in_the_dark: bool,
        minds: impl Fn(usize, usize) -> bool,
    ) -> Vec<(usize, bool)> {
        let Some(shot) = self.droids.get(n) else {
            return Vec::new();
        };
        let (at, character) = (shot.feet, shot.character);
        // Nothing in the way but the one shot, its limbs flying, the player and the witness.
        let passes = |watcher: &Inhabitant, collider: Handle<Collider>| {
            collider == watcher.collider
                || collider == shot.collider
                || collider == them
                || shot.ragdoll.as_ref().is_some_and(|ragdoll| ragdoll.owns(collider))
        };
        let witnesses: Vec<(usize, bool)> = self
            .droids
            .iter()
            .enumerate()
            .filter(|&(m, droid)| {
                m != n
                    && !droid.down
                    && !droid.talking
                    && droid.side.is_none()
                    && droid.pursuit.is_none()
                    && droid.witness.is_none()
                    && droid.alert != Some(Alert::Alert)
                    && droid.hunts_player()
                    && flat(droid.feet - at).norm() < WITNESS_REACH
                    && minds(droid.character, character)
            })
            .filter_map(|(m, droid)| {
                let eyes = droid.feet + Vector3::new(0.0, FACE_HEIGHT, 0.0);
                let sees = |feet: Vector3<f32>, chest: f32| {
                    could_see_other(droid.alert, droid.feet, droid.heading, feet, in_the_dark)
                        && clear_between(graph, eyes, feet + Vector3::new(0.0, chest, 0.0), |c| passes(droid, c))
                };
                let sees_player = sees(player, PLAYER_CHEST);
                (sees_player || sees(at, CHEST)).then(|| (m, knows_who(sees_player, droid.feet, player)))
            })
            .collect();
        let shot = &mut self.droids[n];
        if shot.down {
            shot.murdered = true;
            shot.found |= !witnesses.is_empty();
        }
        if !witnesses.is_empty() {
            let knew = witnesses.iter().filter(|&&(_, knows)| knows).count();
            Log::info(format!("Witnesses: {} saw droid {n} shot, {knew} of them who did it", witnesses.len()));
        }
        witnesses
    }

    /// Every [`BODY_LOOK_EVERY`] seconds of `dt`, those going about their business look for
    /// anyone the player has shot down that no one has come across yet: each that one of them
    /// can see lying within [`BODY_SIGHT`], `in_the_dark` or not, of a kind that `minds` by its
    /// own kind and the one lying there's, has been found, by the first to see it. Who found
    /// one, and where it lies.
    pub fn find_bodies(
        &mut self,
        graph: &Graph,
        in_the_dark: bool,
        dt: f32,
        minds: impl Fn(usize, usize) -> bool,
    ) -> Vec<(usize, Vector3<f32>)> {
        self.body_look -= dt;
        if self.body_look > 0.0 {
            return Vec::new();
        }
        self.body_look = BODY_LOOK_EVERY;
        let mut found = Vec::new();
        for n in 0..self.droids.len() {
            let body = &self.droids[n];
            if !body.down || !body.murdered || body.found {
                continue;
            }
            // Low down, where it lies: its pelvis, or crouched, its middle.
            let lies = body
                .ragdoll
                .as_ref()
                .and_then(|ragdoll| ragdoll.pelvis(graph))
                .unwrap_or(body.feet + Vector3::new(0.0, 0.5 * DOWN_HEIGHT, 0.0));
            let finder = self.droids.iter().enumerate().find(|&(m, droid)| {
                m != n
                    && !droid.down
                    && !droid.talking
                    && droid.side.is_none()
                    && droid.pursuit.is_none()
                    && droid.witness.is_none()
                    && matches!(droid.alert, None | Some(Alert::Caution))
                    && droid.hunts_player()
                    && flat(droid.feet - lies).norm() < BODY_SIGHT
                    && minds(droid.character, body.character)
                    && could_see_other(droid.alert, droid.feet, droid.heading, body.feet, in_the_dark)
                    && clear_between(graph, droid.feet + Vector3::new(0.0, FACE_HEIGHT, 0.0), lies, |collider| {
                        collider == droid.collider
                            || collider == body.collider
                            || body.ragdoll.as_ref().is_some_and(|ragdoll| ragdoll.owns(collider))
                    })
            });
            if let Some((m, _)) = finder {
                Log::info(format!("Body found: droid {n}, by droid {m}"));
                found.push((m, lies));
                self.droids[n].found = true;
            }
        }
        found
    }

    /// Has the `n`th droid, having seen what the player did at `at` - and who did it, if it
    /// `knows` - go and tell a sentry of it (see [`Inhabitants::report`]).
    pub fn witness(&mut self, n: usize, at: Vector3<f32>, knows: bool) {
        if let Some(droid) = self.droids.get_mut(n).filter(|droid| !droid.down) {
            droid.witness = Some(Witness { at, knows, to: None, look_in: 0.0, left: HIDE_FOR });
            droid.warned = 0;
            droid.threat = 0.0;
            droid.route.clear();
            droid.resting = 0.0;
        }
    }

    /// Sees the witnesses on, `dt` seconds on from the last time: each makes for the nearest
    /// sentry - of the kinds `sentry` says are - standing, within
    /// [`REPORT_REACH`], and tells it once it is within [`REPORT_NEAR`] of it. With none about,
    /// it hides, running from where it was done, and looks again now and then, till
    /// [`HIDE_FOR`] is up. What came of it.
    pub fn report(&mut self, dt: f32, sentry: impl Fn(usize) -> bool) -> Reports {
        let mut reports = Reports::default();
        let sentries: Vec<(usize, Vector3<f32>)> = self
            .droids
            .iter()
            .enumerate()
            .filter(|(_, droid)| {
                // Not one after the player already, nor one questioning another witness.
                !droid.down
                    && sentry(droid.character)
                    && droid.side.is_none()
                    && droid.witness.is_none()
                    && !droid.talking
                    && droid.alert != Some(Alert::Alert)
            })
            .map(|(n, droid)| (n, droid.feet))
            .collect();
        for (n, droid) in self.droids.iter_mut().enumerate() {
            let Some(mut witness) = droid.witness else {
                continue;
            };
            // Gone after the player itself, or caught up in a shootout, it has more to think of.
            if droid.down || droid.alert.is_some() || droid.side.is_some() {
                droid.witness = None;
                continue;
            }
            // Stopped by the player, to be questioned, it goes on once they are done.
            if droid.talking {
                continue;
            }
            // Its sentry, if it still stands; or, now and then, the nearest there is.
            let to = witness.to.and_then(|to| sentries.iter().find(|&&(s, _)| s == to)).copied();
            witness.look_in -= dt;
            let to = match to {
                Some(to) => Some(to),
                None if witness.look_in <= 0.0 => {
                    witness.look_in = REPORT_LOOK_EVERY;
                    sentries
                        .iter()
                        .copied()
                        .filter(|&(_, feet)| flat(feet - droid.feet).norm() < REPORT_REACH)
                        .min_by(|a, b| flat(a.1 - droid.feet).norm().total_cmp(&flat(b.1 - droid.feet).norm()))
                }
                None => None,
            };
            if let Some((s, feet)) = to {
                if witness.to != Some(s) {
                    droid.route.clear();
                    droid.replan = 0.0;
                }
                witness.to = Some(s);
                droid.cover = None;
                if flat(feet - droid.feet).norm() < REPORT_NEAR {
                    droid.witness = None;
                    droid.route.clear();
                    droid.resting = REST.1;
                    Log::info(format!("Witness: droid {n} told sentry {s}"));
                    reports.told.push((n, s, witness.at, witness.knows));
                    continue;
                }
            } else {
                witness.to = None;
                witness.left -= dt;
                if witness.left <= 0.0 {
                    droid.witness = None;
                    droid.cover = None;
                    droid.route.clear();
                    droid.resting = REST.0;
                    reports.gave_up.push(n);
                    continue;
                }
                if droid.cover.is_none() {
                    droid.cover = Some((witness.at, false));
                    droid.route.clear();
                }
            }
            droid.witness = Some(witness);
        }
        reports
    }

    /// Has the `n`th droid, if it hunts the player at all, search from `at` - where it found
    /// one they shot down, say - unless it is after them already.
    pub fn search_at(&mut self, n: usize, at: Vector3<f32>) {
        let Some(droid) = self.droids.get_mut(n) else {
            return;
        };
        if droid.down || !droid.hunts_player() || droid.alert == Some(Alert::Alert) {
            return;
        }
        droid.lost_at = at;
        droid.lost_going = Vector3::zeros();
        droid.alert = None;
        droid.enter(n, Some(Alert::Evasion), &mut self.alerts);
        self.alarmed.push(n);
        self.search_from = Some(at);
    }

    /// Has everyone after the player, searching for them or wary of them, or on their way to tell
    /// a sentry of them, give it up: the player has been arrested. Those that did, as indices.
    pub fn stand_down(&mut self) -> Vec<usize> {
        let mut calmed = Vec::new();
        for (n, droid) in self.droids.iter_mut().enumerate() {
            if droid.down || droid.side.is_some() {
                continue;
            }
            let was = droid.alert.is_some() || droid.witness.is_some() || droid.warned > 0;
            droid.witness = None;
            droid.warned = 0;
            droid.threat = 0.0;
            droid.sees_player = false;
            if droid.alert.is_some() {
                droid.alert = None;
                droid.route.clear();
                droid.trip = None;
                droid.resting = REST.0;
            }
            if was {
                calmed.push(n);
            }
        }
        self.search = None;
        self.last_sight = None;
        self.search_from = None;
        calmed
    }

    /// Has everyone after the player, or searching for them, lose them: the player has gone where
    /// none can follow. Witnesses still go to tell a sentry what they saw. Those that did, as
    /// indices.
    pub fn lose_player(&mut self) -> Vec<usize> {
        let mut lost = Vec::new();
        for (n, droid) in self.droids.iter_mut().enumerate() {
            if droid.down || droid.side.is_some() {
                continue;
            }
            droid.threat = 0.0;
            droid.sees_player = false;
            if droid.alert.is_some() {
                droid.alert = None;
                droid.route.clear();
                droid.trip = None;
                droid.resting = REST.0;
                lost.push(n);
            }
        }
        self.search = None;
        self.last_sight = None;
        self.search_from = None;
        lost
    }

    /// Puts a droid of the `character`th kind down to keep about `post`, on the floor of `grid`
    /// whose corner is at `origin`, facing `heading`: one of a precinct's guards. It keeps to it for
    /// good, never turning up again elsewhere as the crowd does, nor called off to a chase or a
    /// shootout. Which it is, if it was put down.
    pub fn post_guard(
        &mut self,
        scene: &mut Scene,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        post: Vector3<f32>,
        heading: f32,
        character: usize,
        rng: &mut Rng,
    ) -> Option<usize> {
        let feet = grid.cell_at(origin, post).filter(|&(x, z)| grid.is_walkable(x, z)).map(|cell| grid.on_floor(origin, cell))?;
        if !self.spawn_one(scene, feet, heading, between(rng, (1.0, 4.0)), character, rng) {
            return None;
        }
        let n = self.droids.len() - 1;
        self.droids[n].post = Some(post);
        Some(n)
    }

    /// Whether a droid of a kind `sentry` says is one is after the player within `reach` of `at`.
    pub fn hunted_near(&self, at: Vector3<f32>, reach: f32, sentry: impl Fn(usize) -> bool) -> bool {
        self.droids.iter().any(|droid| {
            !droid.down
                && sentry(droid.character)
                && matches!(droid.alert, Some(Alert::Alert | Alert::Evasion))
                && flat(droid.feet - at).norm() < reach
        })
    }

    /// Whether the `n`th droid is after the player already.
    pub fn is_hostile(&self, n: usize) -> bool {
        self.droids.get(n).is_some_and(|droid| droid.alert == Some(Alert::Alert))
    }

    /// Moves where the `n`th droid keeps to, to `post`.
    pub fn set_post(&mut self, n: usize, post: Vector3<f32>) {
        if let Some(droid) = self.droids.get_mut(n) {
            droid.post = Some(post);
            droid.route.clear();
            droid.resting = 0.0;
        }
    }

    /// The first standing droid of a kind that `sentry` says is one, calm or not, that sees the
    /// player at `at` - their body `them` - standing or as `posture` says, `in_the_dark` or not,
    /// with nothing in the way but what is `see_through`, such as a firewall: as it would see them
    /// at all.
    #[allow(clippy::too_many_arguments)]
    pub fn sentry_seeing(
        &self,
        graph: &Graph,
        at: Vector3<f32>,
        them: Handle<Collider>,
        posture: Posture,
        in_the_dark: bool,
        see_through: &[Handle<Collider>],
        sentry: impl Fn(usize) -> bool,
    ) -> Option<usize> {
        self.droids.iter().position(|droid| {
            !droid.down
                && droid.side.is_none()
                && sentry(droid.character)
                && Sight::default().could_see(droid.alert, droid.feet, droid.heading, at, posture.into(), in_the_dark)
                && clear_between(
                    graph,
                    droid.feet + Vector3::new(0.0, FACE_HEIGHT, 0.0),
                    at + Vector3::new(0.0, PLAYER_CHEST, 0.0),
                    |collider| collider == droid.collider || collider == them || see_through.contains(&collider),
                )
        })
    }

    /// The nearest standing droid, calm, of a kind that `sentry` says is one, within `reach` of
    /// `at` along the ground: which it is, and whether it has its back to `at`.
    pub fn sentry_near(&self, at: Vector3<f32>, reach: f32, sentry: impl Fn(usize) -> bool) -> Option<(usize, bool)> {
        self.droids
            .iter()
            .enumerate()
            .filter(|(_, droid)| !droid.down && droid.alert.is_none() && droid.side.is_none() && sentry(droid.character))
            .map(|(n, droid)| (n, flat(droid.feet - at).norm(), droid))
            .filter(|&(_, off, _)| off < reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(n, _, droid)| (n, forward(droid.heading).dot(&flat(at - droid.feet)) < 0.0))
    }

    /// Whether the `n`th droid has been stopped.
    pub fn is_down(&self, n: usize) -> bool {
        self.droids.get(n).is_some_and(|droid| droid.down)
    }

    /// Whether the `n`th droid is after the player, standing: shooting it is self-defense, no
    /// crime to be witnessed, and no body to report (see [`Inhabitants::witnesses`]) - unless the
    /// game holds its kind above that, as it does sentries.
    pub fn attacking_player(&self, n: usize) -> bool {
        self.droids
            .get(n)
            .is_some_and(|droid| !droid.down && droid.hunts_player() && droid.alert == Some(Alert::Alert))
    }

    /// The droid whose body `collider` is, if it is one.
    pub fn hit(&self, collider: Handle<Collider>) -> Option<usize> {
        self.droids
            .iter()
            .position(|droid| droid.collider == collider)
    }

    /// A noise at `at`, that carries `loudness` meters along the corridors of `grid`, whose
    /// corner is at `origin`. Every hostile droid within earshot that is not on Alert - and has
    /// not just heard something else, or been shot - runs to where it was, and searches from
    /// there.
    pub fn hear(
        &mut self,
        (grid, origin): (&WalkGrid, Vector3<f32>),
        at: Vector3<f32>,
        loudness: f32,
    ) {
        let listening = |droid: &Inhabitant| {
            !droid.down && droid.hunts_player() && matches!(droid.alert, Some(Alert::Evasion | Alert::Caution))
        };
        if !self.droids.iter().any(listening) {
            return;
        }
        let Some(noise) = Heard::at((grid, origin), at, loudness) else {
            return;
        };
        for (n, droid) in self.droids.iter_mut().enumerate() {
            if !listening(droid) {
                continue;
            }
            if !noise.by((grid, origin), droid.feet) {
                continue;
            }
            if droid.deaf > 0.0 {
                continue;
            }
            droid.deaf = HEARING_REST;
            droid.lost_at = at;
            droid.lost_going = Vector3::zeros();
            droid.alert = None;
            droid.enter(n, Some(Alert::Evasion), &mut self.alerts);
            self.heard.push(n);
            // Where the noise was is where the search starts from.
            self.search_from = Some(at);
        }
    }

    /// Bolts that flew `passes` this frame, fired by `fired_by`'s side - or the player in the
    /// maze, with none - and struck the bodies in `struck`. A droid one went within [`NEAR_MISS`]
    /// of, that it did not strike, and not of its own side's, takes it as being shot at, missed:
    /// in capture the flag it turns to where it was fired from, and goes after whoever fired it
    /// unless it sees them, unless it is fighting already; a
    /// hostile one searches from there, unless it is after the player already, as one hit
    /// does (see [`Inhabitants::shot`]). Every droid shot at like that is handed back: those not
    /// hostile yet are to be provoked as if hit (see [`Inhabitants::provoke`]).
    pub fn near_miss(
        &mut self,
        passes: &[Pass],
        fired_by: Option<Side>,
        struck: &[Handle<Collider>],
    ) -> Vec<usize> {
        let mut missed = Vec::new();
        if passes.is_empty() {
            return missed;
        }
        for (n, droid) in self.droids.iter_mut().enumerate() {
            if droid.down || droid.talking || struck.contains(&droid.collider) {
                continue;
            }
            if droid.side.is_some() && droid.side == fired_by {
                continue;
            }
            let chest = droid.feet + Vector3::new(0.0, CHEST, 0.0);
            let Some(pass) = passes.iter().find(|pass| pass.nearest(chest) <= NEAR_MISS) else {
                continue;
            };
            missed.push(n);
            if droid.alert == Some(Alert::Alert) {
                continue;
            }
            if droid.side.is_some() {
                if let Some(heading) = heading_of(pass.fired_at - droid.feet) {
                    droid.heading = heading;
                }
                droid.shot_from = Some((pass.fired_at, FOE_MEMORY));
            } else if droid.alert.is_some() && droid.hunts_player() {
                droid.deaf = HEARING_REST;
                droid.lost_at = pass.fired_at;
                droid.lost_going = Vector3::zeros();
                droid.alert = None;
                droid.enter(n, Some(Alert::Evasion), &mut self.alerts);
            }
        }
        missed
    }

    /// The most urgent phase any droid is in, and the longest any of them in it has left to
    /// search or stay wary, in seconds.
    pub fn alarm(&self) -> Option<(Alert, f32)> {
        // A guard's standing watch in capture the flag is no alarm.
        let raised = |droid: &&Inhabitant| !(droid.side.is_some() && droid.alert == Some(Alert::Caution));
        let alert = self.droids.iter().filter(raised).filter_map(|droid| droid.alert).max()?;
        let left = self
            .droids
            .iter()
            .filter(raised)
            .filter(|droid| droid.alert == Some(alert))
            .map(|droid| droid.search_left)
            .fold(0.0, f32::max);
        Some((alert, left))
    }

    /// How much breath each sentry that is hostile, or has spent some, has left.
    pub fn breath(&self, graph: &Graph) -> Vec<Breath> {
        self.droids
            .iter()
            .enumerate()
            .filter(|(_, droid)| droid.sentry && !droid.down)
            .filter(|(_, droid)| droid.alert.is_some() || droid.stamina < droid.most || droid.hits > 0)
            .filter_map(|(n, droid)| {
                Some(Breath {
                    feet: droid.feet,
                    face: self.face(graph, n)?,
                    left: droid.stamina / droid.most,
                    winded: droid.winded,
                    health: 1.0 - droid.hits as f32 / droid.stopped_by() as f32,
                })
            })
            .collect()
    }

    /// Turns the `n`th droid on the player - and the rest of its pair or group with it:
    /// after a moment they hunt them, and cannot be talked to any more.
    pub fn set_hostile(&mut self, n: usize) {
        for m in self.group_of(n) {
            if let Some(droid) = self.droids.get_mut(m).filter(|droid| !droid.down && droid.hunts_player()) {
                droid.alert = Some(Alert::Alert);
                droid.windup = WINDUP;
                droid.replan = 0.0;
            }
        }
    }

    /// Shoots down the droid nearest the `player`'s feet, as bolts from them would, [`HITS`]
    /// times over, for MAZE_KNOCKDOWN, or once through the head with MAZE_HEADSHOT; and with MAZE_DISMEMBER=<bodies>, a comma-separated list
    /// of ragdoll bodies such as head,forearm.L,shin.R, breaks those off it too. Which one, if any
    /// is standing.
    pub fn knock_down(&mut self, graph: &mut Graph, player: Vector3<f32>) -> Option<usize> {
        let n = (0..self.droids.len())
            .filter(|&n| !self.droids[n].down)
            .min_by(|&a, &b| {
                let distance = |n: usize| (self.droids[n].feet - player).norm();
                distance(a).total_cmp(&distance(b))
            })?;
        let droid = &self.droids[n];
        let mut at = droid.feet + Vector3::new(0.0, 1.2, 0.0);
        let mut from = player + Vector3::new(0.0, 1.5, 0.0);
        // With MAZE_HEADSHOT, one bolt level through the middle of its skull.
        let headshot = crate::platform::var("MAZE_HEADSHOT").is_some();
        if headshot {
            if let Some((head, _)) = graph.find_by_name(droid.avatar.root(), "DEF-spine.004") {
                at = graph[head].global_position() + Vector3::new(0.0, 0.19, 0.0);
                from = Vector3::new(player.x, at.y, player.z);
            }
        }
        let strike = Strike {
            collider: droid.collider,
            at,
            way: (at - from).try_normalize(1.0e-6).unwrap_or_else(Vector3::z),
        };
        let n = (0..SENTRY_HITS.max(HITS)).find_map(|_| self.shot(graph, strike, player))?;
        if let Some(bodies) = crate::platform::var("MAZE_DISMEMBER") {
            for body in bodies
                .split(',')
                .map(str::trim)
                .filter(|body| !body.is_empty())
            {
                self.droids[n].break_off(graph, body);
                Log::info(format!("MAZE_DISMEMBER: {body} broken off droid {n}"));
            }
        }
        Some(n)
    }

    /// A bolt from the pistol of the player at `player` has struck something. If it is a hostile
    /// droid, that is another hit on it, and at [`HITS`] it goes down, its eyes dark: limp,
    /// knocked back by the bolt, or without a ragdoll crouched, and no taller than it is
    /// crouched. Short of that, if it has not seen who fired, it searches for them where they
    /// fired from. A droid already lying there is just shoved. The droid that went down, if one
    /// did.
    pub fn shot(
        &mut self,
        graph: &mut Graph,
        strike: Strike,
        player: Vector3<f32>,
    ) -> Option<usize> {
        let collider = strike.collider;
        if let Some(droid) = self.droids.iter_mut().find(|droid| {
            droid
                .ragdoll
                .as_ref()
                .is_some_and(|ragdoll| ragdoll.owns(collider))
        }) {
            let ragdoll = droid.ragdoll.as_mut()?;
            ragdoll.shove(graph, collider, strike.way * ragdoll::SHOVE, strike.at);
            let body = ragdoll.body_of(collider);
            let headshot = body
                .filter(|&body| body == HEAD)
                .and_then(|_| droid.headshot(graph, strike.at, strike.way));
            if !headshot.is_some_and(|head| droid.blow_head_up(graph, head, strike.way)) {
                if let Some(body) = body {
                    droid.break_off(graph, body);
                }
            }
            return None;
        }
        let (n, droid) = self
            .droids
            .iter_mut()
            .enumerate()
            .find(|(_, droid)| droid.collider == collider)?;
        if droid.down {
            return None;
        }
        // A good headshot stops it there and then.
        let headshot = droid.headshot(graph, strike.at, strike.way);
        droid.hits += 1;
        if droid.hits < droid.stopped_by() && headshot.is_none() {
            if droid.side.is_some() {
                // In capture the flag, it turns to whoever it was, to see them, and goes after
                // them if it does not.
                let to = flat(player - droid.feet);
                if to.norm() > 1.0e-3 && droid.alert != Some(Alert::Alert) {
                    droid.heading = to.x.atan2(to.z);
                    droid.shot_from = Some((player, FOE_MEMORY));
                }
            } else if droid.alert.is_some_and(|alert| alert != Alert::Alert) {
                // After whoever fired, paying no heed to the bolt's own noise hitting it. One
                // going about its business does what its kind does about it (see
                // [`Inhabitants::provoke`]).
                droid.deaf = HEARING_REST;
                droid.lost_at = player;
                droid.lost_going = Vector3::zeros();
                droid.alert = None;
                droid.enter(n, Some(Alert::Evasion), &mut self.alerts);
            }
            return None;
        }
        droid.down = true;
        droid.alert = None;
        droid.sees_player = false;
        droid.witness = None;
        droid.cover = None;
        droid.avatar.set_eyes(Some(Color::BLACK));
        droid.ragdoll = Ragdoll::start(
            graph,
            droid.avatar.root(),
            forward(droid.heading) * droid.speed,
            Some((strike.way * ragdoll::STOPPING_BLOW, strike.at)),
        );
        if let Some(head) = headshot {
            droid.blow_head_up(graph, head, strike.way);
        }
        if let Some(ragdoll) = &droid.ragdoll {
            if headshot.is_none() {
                if let Some(body) = ragdoll.body_struck(graph, strike.at, strike.way) {
                    droid.break_off(graph, body);
                }
            }
            return Some(n);
        }
        if let Ok(shape) = graph.try_get_mut(droid.collider) {
            shape.set_shape(ColliderShape::capsule_y(0.5 * DOWN_HEIGHT - RADIUS, RADIUS));
            shape.local_transform_mut().set_position(Vector3::new(
                0.0,
                0.5 * DOWN_HEIGHT - MIDDLE,
                0.0,
            ));
        }
        Some(n)
    }

    /// The droid the player could talk to, standing at `feet` and looking `ahead` along the
    /// ground, if there is one: the nearest close by and in front of them, as long as `in_sight`
    /// says nothing is in the way from the player to where it is.
    pub fn to_talk_to(
        &self,
        feet: Vector3<f32>,
        ahead: Vector3<f32>,
        in_sight: impl Fn(Vector3<f32>) -> bool,
    ) -> Option<usize> {
        within_talking(self.droids.iter().map(|droid| droid.feet), feet, ahead)
            .into_iter()
            .filter(|&i| self.droids[i].alert.is_none() && !self.droids[i].down)
            // Being questioned by a sentry, or questioning a witness, it is busy.
            .filter(|&i| self.droids[i].with.is_none())
            // On its way to tell a sentry what the player did, it stops to be questioned - but not
            // while it is in hiding, with no sentry to make for.
            .filter(|&i| self.droids[i].witness.is_none_or(|witness| witness.to.is_some()))
            // In the middle of a shootout, it has no time for them.
            .filter(|&i| !matches!(self.droids[i].side, Some(Side::Security | Side::Gang)))
            .find(|&i| in_sight(self.droids[i].feet))
    }

    /// Which kind of droid the `n`th is, as an index into the conversations' characters, and its
    /// code.
    pub fn who(&self, n: usize) -> Option<(usize, u32)> {
        self.droids
            .get(n)
            .map(|droid| (droid.character, droid.code))
    }

    /// Where the `n`th droid's feet are.
    pub fn feet(&self, n: usize) -> Option<Vector3<f32>> {
        self.droids.get(n).map(|droid| droid.feet)
    }

    /// Where the middle of the `n`th droid's face is, as of the last frame.
    pub fn face(&self, graph: &Graph, n: usize) -> Option<Vector3<f32>> {
        let droid = self.droids.get(n)?;
        Some(
            droid
                .avatar
                .face_at(graph)
                .unwrap_or(droid.feet + Vector3::new(0.0, FACE_HEIGHT, 0.0)),
        )
    }

    /// Has the player talking to the `n`th droid, or done talking to it; done, it stands a
    /// moment before going on its way.
    pub fn set_talking(&mut self, n: usize, talking: bool) {
        if let Some(droid) = self.droids.get_mut(n) {
            droid.talking = talking;
            droid.with = None;
            if !talking {
                droid.resting = REST.0;
            }
        }
    }

    /// Has the `n`th droid and the `m`th stand talking with each other, each facing the other,
    /// until [`Inhabitants::set_talking`] lets them go.
    pub fn talk_with(&mut self, n: usize, m: usize) {
        for (a, b) in [(n, m), (m, n)] {
            if let Some(droid) = self.droids.get_mut(a) {
                droid.talking = true;
                droid.with = Some(b);
            }
        }
    }

    /// Whether the `n`th droid is on its way to tell a sentry of what the player did, and if it
    /// is, whether it knows who did it.
    pub fn witness_knows(&self, n: usize) -> Option<bool> {
        self.droids.get(n)?.witness.map(|witness| witness.knows)
    }

    /// Has the `n`th droid, a witness, keep quiet about what it saw: it tells no one, and goes
    /// about its business again.
    pub fn silence(&mut self, n: usize) {
        if let Some(droid) = self.droids.get_mut(n) {
            droid.witness = None;
            droid.cover = None;
            droid.route.clear();
        }
    }

    /// Has the `n`th droid, a witness, know the player for who did it.
    pub fn identify(&mut self, n: usize) {
        if let Some(witness) = self.droids.get_mut(n).and_then(|droid| droid.witness.as_mut()) {
            witness.knows = true;
        }
    }

    /// Has the `n`th droid's eyes glow `colour`, or their own colour with none. Hostile red is its
    /// kind's own hostile colour, if it has one (see [`Livery::with_hostile_eyes`]).
    pub fn set_eyes(&self, n: usize, colour: Option<Color>) {
        if let Some(droid) = self.droids.get(n) {
            // Hostile, in its kind's own colour if it has one.
            let own = self.liveries.get(droid.character).and_then(|livery| livery.hostile_eyes);
            let colour = match (colour, own) {
                (Some(colour), Some(own)) if Some(colour) == crate::dialogue::screen::eyes(crate::dialogue::Mood::Hostile) => Some(own),
                _ => colour,
            };
            droid.avatar.set_eyes(colour);
        }
    }

    /// Draws only those the player could see from where they are in `level`.
    pub fn show(&self, graph: &mut Graph, level: &Level) {
        for droid in &self.droids {
            droid
                .avatar
                .set_visible(graph, level.can_see(droid.feet));
        }
    }

    /// The droid nearest `to` of those that can be seen, for the graphics effects: a capsule
    /// round it, and how far it has moved since the last time this was asked.
    pub fn moving(&mut self, graph: &Graph, to: Vector3<f32>) -> Option<fyrox_gfx::MovingThing> {
        let mut nearest: Option<(f32, fyrox_gfx::MovingThing)> = None;
        for droid in &mut self.droids {
            let moved = droid
                .last_seen
                .map_or(Vector3::zeros(), |last| droid.feet - last);
            droid.last_seen = Some(droid.feet);
            let distance = (droid.feet - to).norm();
            if droid.avatar.is_visible(graph) && nearest.is_none_or(|(d, _)| distance < d) {
                nearest = Some((distance, avatar::capsule(droid.feet, moved)));
            }
        }
        nearest.map(|(_, thing)| thing)
    }
}

/// Those of the droids with their feet at `droids` that a player at `feet`, looking `ahead`
/// along the ground, is close enough to and facing enough to talk to, nearest first.
fn within_talking(
    droids: impl Iterator<Item = Vector3<f32>>,
    feet: Vector3<f32>,
    ahead: Vector3<f32>,
) -> Vec<usize> {
    let mut near: Vec<(f32, usize)> = droids
        .enumerate()
        .filter_map(|(i, there)| {
            let to_them = flat(there - feet);
            let distance = to_them.norm();
            let off = if distance < 1.0e-4 {
                0.0
            } else {
                (to_them.dot(&ahead) / distance).clamp(-1.0, 1.0).acos()
            };
            (distance < TALK_REACH && off < TALK_CONE).then_some((distance, i))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    near.into_iter().map(|(_, i)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_witness_knows_who_shot_only_if_it_saw_them_near_enough() {
        let feet = Vector3::new(0.0, 0.0, 0.0);
        assert!(knows_who(true, feet, Vector3::new(10.0, 0.0, 5.0)), "near, and seen");
        assert!(!knows_who(true, feet, Vector3::new(40.0, 0.0, 0.0)), "a shot from far off");
        assert!(!knows_who(false, feet, Vector3::new(3.0, 0.0, 0.0)), "near, but not seen");
        assert!(knows_who(true, feet, Vector3::new(0.0, 6.0, IDENTIFY_REACH)), "however high up");
    }

    const AHEAD: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);

    #[test]
    fn the_squares_droids_are_found_by_reach_as_far_as_any_makes_way() {
        use hydroxus_ai::steer::{AVOID_RANGE, YIELD_RANGE};
        assert!(NEIGHBOURS >= AVOID_RANGE.max(YIELD_RANGE));
    }

    #[test]
    fn going_about_its_business_it_keeps_off_the_road_but_to_cross() {
        // A street 4 m wide down the middle of ground 20 m square, a crossing over it at z 10.
        let mut grid = WalkGrid::new(40, 40, 0.5);
        for x in 0..40 {
            for z in 0..40 {
                grid.set(x, z, true);
                grid.set_floor(x, z, if (16..24).contains(&x) { 0.0 } else { 0.12 });
            }
        }
        let plan: crate::roads::RoadPlan = serde_json::from_str(
            r#"{ "level": 0.06, "roads": [[8, 0, 12, 20]], "crossings": [[8, 9, 12, 11]] }"#,
        )
        .unwrap();
        let roads = plan.on(&grid, Vector3::zeros());
        let mut rng = Rng::new(11);
        let mut crossed = 0;
        for _ in 0..40 {
            let route = wander((&grid, Vector3::zeros()), Vector3::new(2.0, 0.12, 2.0), (10.0, 30.0), Some(&roads), &[], &Reach::default(), &mut rng);
            assert!(!route.is_empty());
            for at in &route {
                let (x, z) = survey::cell_at(&grid, Vector3::zeros(), *at).unwrap();
                assert!(!roads.is_road(x, z), "on the road at {at:?}");
            }
            crossed += route.iter().any(|at| at.x > 12.0) as u32;
        }
        assert!(crossed > 0, "it does cross, at the crossing");
        // On the road, it gets off it.
        let route = wander((&grid, Vector3::zeros()), Vector3::new(10.0, 0.0, 2.0), (10.0, 30.0), Some(&roads), &[], &Reach::default(), &mut rng);
        let end = route.first().unwrap();
        let (x, z) = survey::cell_at(&grid, Vector3::zeros(), *end).unwrap();
        assert!(!roads.is_road(x, z));
    }

    #[test]
    fn going_about_its_business_it_keeps_off_a_gangs_turf() {
        // Ground 20 m square, with a street at its edge, and a gang's turf 3 m round in the
        // middle of the pavement.
        let mut grid = WalkGrid::new(40, 40, 0.5);
        for x in 0..40 {
            for z in 0..40 {
                grid.set(x, z, true);
                grid.set_floor(x, z, 0.12);
            }
        }
        let plan: crate::roads::RoadPlan =
            serde_json::from_str(r#"{ "level": 0.06, "roads": [[19, 0, 20, 20]], "crossings": [] }"#).unwrap();
        let roads = plan.on(&grid, Vector3::zeros());
        let spot = Vector3::new(10.0, 0.12, 10.0);
        let turfs = [(survey::cell_at(&grid, Vector3::zeros(), spot).unwrap(), 3.0 / 0.5)];
        let mut rng = Rng::new(5);
        for _ in 0..40 {
            let route = wander((&grid, Vector3::zeros()), Vector3::new(2.0, 0.12, 2.0), (10.0, 30.0), Some(&roads), &turfs, &Reach::default(), &mut rng);
            if let Some(end) = route.first() {
                // Give or take the half-meter cells it is reckoned in.
                assert!(flat(end - spot).norm() >= 2.5, "makes for {end:?}, on the turf");
            }
            for at in &route {
                assert!(flat(at - spot).norm() > 1.5, "goes by {at:?}, through the middle of the turf");
            }
        }
    }

    #[test]
    fn only_those_close_by_and_in_front_can_be_talked_to_nearest_first() {
        let droids = [
            Vector3::new(0.0, 0.0, 2.0),  // ahead
            Vector3::new(0.3, 0.0, 1.0),  // nearer, a little to one side
            Vector3::new(0.0, 0.0, -1.0), // behind
            Vector3::new(2.0, 0.0, 0.5),  // well off to the side
            Vector3::new(0.0, 0.0, 4.0),  // too far
        ];
        assert_eq!(
            within_talking(droids.into_iter(), Vector3::zeros(), AHEAD),
            [1, 0]
        );
    }

    #[test]
    fn what_can_be_got_to_is_kept_a_bit_a_cell_and_empty_is_anywhere() {
        let reach = Reach::of((0..130).map(|i| i % 3 == 0));
        assert!(reach.can(0) && reach.can(63) && reach.can(129));
        assert!(!reach.can(1) && !reach.can(64) && !reach.can(128));
        assert!(!reach.can(10_000), "past the end");
        assert!(Reach::default().can(10_000), "anywhere, with nothing known");
    }

    #[test]
    fn a_droid_going_about_its_business_does_not_set_off_for_where_it_cannot_get_to() {
        // Two pavements, a path between them; the far one, as if walled off, not to be got to.
        let mut grid = WalkGrid::new(40, 10, 1.0);
        for z in 0..10 {
            for x in (0..15).chain(25..40) {
                grid.set(x, z, true);
            }
        }
        for x in 15..25 {
            grid.set(x, 0, true);
        }
        let plan: crate::roads::RoadPlan =
            serde_json::from_str(r#"{ "level": 0.06, "roads": [], "crossings": [] }"#).unwrap();
        let roads = plan.on(&grid, Vector3::zeros());
        let reach = Reach::of((0..grid.width * grid.rows()).map(|i| i % grid.width < 25));
        let (mut rng, mut went) = (Rng::new(3), 0);
        for _ in 0..60 {
            let route = wander((&grid, Vector3::zeros()), Vector3::new(2.0, 0.0, 5.0), (10.0, 30.0), Some(&roads), &[], &reach, &mut rng);
            assert!(route.iter().all(|at| at.x < 25.0), "set off for the far side: {route:?}");
            went += !route.is_empty() as u32;
        }
        assert!(went > 0, "it goes somewhere");
        // Not kept off it, it does go there now and then.
        let anywhere = (0..60).any(|_| {
            wander((&grid, Vector3::zeros()), Vector3::new(2.0, 0.0, 5.0), (10.0, 30.0), Some(&roads), &[], &Reach::default(), &mut rng)
                .iter()
                .any(|at| at.x >= 25.0)
        });
        assert!(anywhere, "the far side is there to go to");
    }

    /// The train at the first station with one doorway, on its right, its doors open there for as
    /// long as `open_for` says; and a ride by it to the last station, waiting on the platform.
    fn at_the_station(open_for: Option<f32>) -> (Doors, Ride) {
        let mut train = Doors {
            line: crate::train::LINES[0],
            at: Vector3::zeros(),
            doorways: vec![Vector3::new(2.53, -1.25, 2.75)],
            across: Vector3::x(),
            along: Vector3::z(),
            open: None,
        };
        train.at = train.station(0);
        train.open = open_for.map(|left| (0, left));
        let to = train.stations() - 1;
        let (wait, off) = (train.platform(0, 0, PLATFORM_OUT), train.platform(to, 0, PLATFORM_OUT));
        let spot = train.doorways[0] + Vector3::new(-STAND_IN, 0.0, 0.0);
        (train, Ride { train: 0, from: 0, to, doorway: 0, spot, wait, off, stage: Riding::Waiting, waited: 0.0 })
    }

    /// Moves the ride on a step of `dt`, the droid walking where it says as it would: on to each
    /// point of the way, past it once near; aboard, where it stands.
    fn ride_a_step(ride: &mut Ride, train: &Doors, feet: &mut Vector3<f32>, dt: f32) -> Rides {
        let rides = ride.step(*feet, &[], train, dt);
        if let Rides::Go(route) = &rides {
            let mut route = route.clone();
            while route.len() > 1 && flat(route[route.len() - 1] - *feet).norm() < REACHED {
                route.pop();
            }
            if let Some(&next) = route.last() {
                let to = flat(next - *feet);
                if to.norm() > 1.0e-4 {
                    *feet += to.normalize() * (FALLBACK_PACE * dt).min(to.norm());
                }
            }
        }
        if ride.stage == Riding::Aboard {
            *feet = train.at + ride.spot;
        }
        rides
    }

    #[test]
    fn a_ride_on_the_train_gets_on_at_one_station_and_off_at_another() {
        let (mut train, mut ride) = at_the_station(None);
        let station = train.at;
        let mut feet = ride.wait;
        let dt = 0.1;
        // The doors shut, it waits.
        for _ in 0..20 {
            assert_eq!(ride_a_step(&mut ride, &train, &mut feet, dt), Rides::Go(Vec::new()));
        }
        assert_eq!(ride.stage, Riding::Waiting);
        // They open, it gets on.
        train.open = Some((0, 15.0));
        for _ in 0..100 {
            ride_a_step(&mut ride, &train, &mut feet, dt);
        }
        assert_eq!(ride.stage, Riding::Aboard);
        assert!(train.within(feet), "{feet:?}");
        // Off it goes, and it with it.
        train.open = None;
        let there = train.station(ride.to);
        for k in 0..=50 {
            train.at = station.lerp(&there, k as f32 / 50.0);
            ride_a_step(&mut ride, &train, &mut feet, dt);
            assert_eq!(feet, train.at + ride.spot);
        }
        // At a station on the way, it stays on.
        train.open = Some((1, 15.0));
        for _ in 0..20 {
            ride_a_step(&mut ride, &train, &mut feet, dt);
        }
        assert_eq!(ride.stage, Riding::Aboard);
        // At the one it is going to, the doors open: off it gets, onto its platform.
        train.open = Some((ride.to, 15.0));
        let mut off = false;
        for _ in 0..100 {
            if ride_a_step(&mut ride, &train, &mut feet, dt) == Rides::Off {
                off = true;
                break;
            }
        }
        assert!(off, "{:?} at {feet:?}", ride.stage);
        assert!((feet.x - train.at.x).abs() > CLEAR_OF_DOORS && flat(feet - ride.off).norm() < AT_PLATFORM);
    }

    #[test]
    fn a_ride_on_the_train_gets_on_only_with_time_to_and_waits_for_the_next_if_too_late() {
        // Too little time left to get on: it goes on waiting.
        let (train, mut ride) = at_the_station(Some(0.5));
        let mut feet = ride.wait;
        ride_a_step(&mut ride, &train, &mut feet, 0.1);
        assert_eq!(ride.stage, Riding::Waiting);
        // Shut on it before it is on: back to waiting.
        let (mut train, mut ride) = at_the_station(Some(10.0));
        let mut feet = ride.wait;
        ride_a_step(&mut ride, &train, &mut feet, 0.1);
        assert_eq!(ride.stage, Riding::Boarding);
        train.open = None;
        ride_a_step(&mut ride, &train, &mut feet, 0.1);
        assert_eq!(ride.stage, Riding::Waiting);
        // Kept waiting too long, it gives up.
        let (train, mut ride) = at_the_station(None);
        let mut feet = ride.wait;
        let steps = (TRAIN_WAIT / 1.0) as usize + 1;
        let gave_up = (0..steps).any(|_| ride_a_step(&mut ride, &train, &mut feet, 1.0) == Rides::Off);
        assert!(gave_up);
    }
}
