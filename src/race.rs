//! Racing: three laps of a circuit on a hover bike, against four droids on bikes of their own
//! (see [`RIVALS`]) and the clock.
//!
//! A map is raced when it marks a start line, `race_start`, and checkpoints, `checkpoint_1` on
//! round the track in order (see `data/arena/race_circuit.py`): a lap is every checkpoint in turn
//! and then the line. The player's droid rides the bike - its own model, [`BIKE_MODEL`], made in
//! Blender (~/Documents/blender/vehicles/hover_bike), with a droid on it holding `droid_ride`, the
//! pose made for it - and is seen from behind, the camera following on a little way back.
//!
//! The bike hovers [`HOVER`] over whatever is under it, tilting with the road as it banks and
//! climbs, leaning into its turns. W goes, S brakes and then backs up, A and D steer, Shift boosts
//! for as long as the boost lasts ([`BOOST_FOR`] seconds, filling again over [`BOOST_REFILL`]),
//! and R puts the bike back on the track at the last checkpoint passed. Speed pads on the road
//! (`race_pad_N`, amber chevrons lit one at a time from the back, [`PadLights`]) kick a bike over
//! one up to [`PAD_SPEED`] and hold it there for [`PAD_FOR`] seconds. Off the end of a crest it
//! flies, falling, until it is over the road again. It cannot go through the walls: it glances off
//! them, losing speed by how square on it hits.
//!
//! A countdown of [`COUNTDOWN`] seconds holds the bike on the line; then the clock runs until the
//! last lap is done. With MAZE_RACE_AUTOPILOT set, the bike drives itself round, steering for the
//! checkpoints ahead - for trying the track out, and the beginnings of droids to race against.

use crate::level::Marker;
use fyrox::{
    core::{
        algebra::{Point3, UnitQuaternion, Vector3},
        color::Color,
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    keyboard::KeyCode,
    material::{MaterialProperty, MaterialResource},
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{
        animation::AnimationPlayer,
        base::BaseBuilder,
        collider::Collider,
        mesh::Mesh,
        graph::{physics::RayCastOptions, Graph},
        node::Node,
        pivot::PivotBuilder,
        Scene,
    },
};

/// The bike's model: its body under `hover_bike`, its front +z once turned round (the model's own
/// front is -z, as it was made facing the droid that rides it).
pub const BIKE_MODEL: &str = "data/hover_bike.glb";
/// The pose the droid rides in.
const RIDE: &str = "droid_ride";
/// How many laps a race is.
pub const LAPS: u32 = 3;
/// How long the bike is held on the line before the off, in seconds.
pub const COUNTDOWN: f32 = 3.0;

/// How high the bike's origin - the ground under its middle - rides over the road, in meters:
/// its lift pads a little off it.
pub const HOVER: f32 = 0.15;
/// How fast it goes flat out, and boosting, in meters per second.
pub const TOP_SPEED: f32 = 38.0;
pub const BOOST_SPEED: f32 = 50.0;
/// How quickly it picks up speed, flat out and boosting, from a standstill, in m/s².
const ACCEL: f32 = 15.0;
const BOOST_ACCEL: f32 = 22.0;
/// How hard it brakes, how fast it backs up, and how quickly it slows of itself with nothing held.
const BRAKE: f32 = 32.0;
const REVERSE_SPEED: f32 = 8.0;
const COAST: f32 = 4.0;
/// How quickly what it was going sideways dies away, on the road and in the air: how well it holds
/// the line it is pointed along.
const GRIP: f32 = 7.0;
const AIR_GRIP: f32 = 0.4;
/// How fast it turns at a crawl and flat out, in radians a second: slower the faster it goes.
const STEER_SLOW: f32 = 2.3;
const STEER_FAST: f32 = 1.05;
/// How much harder it falls than things do of themselves, off a crest: it is heavy.
const FALL: f32 = 1.6;
const GRAVITY: f32 = 9.81;
/// How long the boost lasts, full, and how long it takes to fill again from empty, in seconds.
pub const BOOST_FOR: f32 = 3.0;
pub const BOOST_REFILL: f32 = 12.0;
/// A speed pad: how fast a bike crossing one is kicked up to - by [`PAD_KICK`] at the most - and
/// for how long, in seconds, it is held up to that before it bleeds back down to its top speed.
pub const PAD_SPEED: f32 = 56.0;
const PAD_KICK: f32 = 16.0;
pub const PAD_FOR: f32 = 1.2;
/// How far a pad reaches from its middle, along the road and across it, and up and down; and how
/// nearly the bike has to be pointing its way for it to count.
const PAD_HALF: (f32, f32) = (4.0, 2.6);
const PAD_HEIGHT: f32 = 2.0;
const PAD_FACING: f32 = 0.3;
/// How far ahead a droid sees a speed pad and makes for it, and how far off its line it goes to.
const PAD_SEEN: f32 = 45.0;
const PAD_DETOUR: f32 = 6.0;
/// How many chevrons a pad has, each its own mesh in the track (`SpeedPadChevron1`, the hindmost,
/// on: see `data/arena/race_circuit.py`); how long each is lit, in seconds, before the next one on
/// is; and how bright an unlit one's colour is, of a lit one's.
const PAD_CHEVRONS: usize = 3;
const CHEVRON_LIT: f32 = 0.12;
const CHEVRON_DARK: f32 = 0.3;
/// How far from its middle the bike keeps off a wall: to either side, and ahead or behind.
const RADIUS: f32 = 0.6;
const HALF_LENGTH: f32 = 1.3;
/// How high over the road it meets the walls, in meters: under their tops, over the kerbs.
const BODY: f32 = 0.6;
/// How much of what it was going into a wall it keeps, bounced back off it.
const BOUNCE: f32 = 0.25;
/// How far over the road the ground under it is looked for, and how far below: further while it is
/// on the road, so that it follows it down over a crest rather than taking off at any dip.
const PROBE_UP: f32 = 1.5;
const STICK: f32 = 0.6;
const LIFT_OFF: f32 = 0.15;
/// How steep a surface can be and still be road under the bike: steeper is a wall.
const FLOOR_UP: f32 = 0.6;
/// How far it leans into a turn at full lock and speed, in radians.
const LEAN: f32 = 0.4;

/// How near the middle of a checkpoint the bike has to come for it to count: across, the whole
/// of the widest road (the Grand Circuit's, 24 m) and then some; up and down, not the road over or
/// under it.
const GATE_REACH: f32 = 15.0;
const GATE_HEIGHT: f32 = 4.0;

/// The camera: how far behind the bike and over it, and how far ahead of it it looks; further back
/// the faster it goes; and how quickly it follows.
const CHASE_BACK: f32 = 4.2;
const CHASE_BACK_FAST: f32 = 5.4;
const CHASE_UP: f32 = 1.8;
const CHASE_AHEAD: f32 = 6.0;
const CHASE_LOOK_UP: f32 = 0.9;
const CHASE_FOLLOW: f32 = 5.0;
/// How far the camera keeps in front of anything between it and the bike.
const CHASE_CLEARANCE: f32 = 0.4;

/// The circuit, from the map's markers.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    /// The middle of the start line.
    pub start: Vector3<f32>,
    /// The checkpoints round the track in order, and then the start line: a lap is all of them.
    pub gates: Vec<Vector3<f32>>,
    /// Where the bikes line up, and which way they face, from the front of the grid back.
    pub grid: Vec<(Vector3<f32>, f32)>,
    /// The speed pads: the middle of each, and the way it kicks along the ground (length 1).
    pub pads: Vec<(Vector3<f32>, Vector3<f32>)>,
}

impl Track {
    /// The track the map's `markers` lay out, if they mark a start line and checkpoints.
    pub fn from_markers(markers: &[Marker]) -> Option<Self> {
        let start = markers.iter().find(|m| m.name == "race_start")?.position;
        let numbered = |prefix: &str| {
            let mut found: Vec<(u32, Vector3<f32>)> = markers
                .iter()
                .filter_map(|m| Some((m.name.strip_prefix(prefix)?.parse::<u32>().ok()?, m.position)))
                .collect();
            found.sort_by_key(|&(n, _)| n);
            found.into_iter().map(|(_, at)| at).collect::<Vec<_>>()
        };
        let mut gates = numbered("checkpoint_");
        if gates.is_empty() {
            return None;
        }
        gates.push(start);
        // Facing the first checkpoint from the line, whichever way the markers are turned.
        let first = gates[0];
        let ahead = yaw_towards(start, first);
        let mut grid: Vec<_> = numbered("race_grid_").into_iter().map(|at| (at, ahead)).collect();
        if grid.is_empty() {
            grid.push((start, ahead));
        }
        // A pad's +x is the way it kicks: the marker's yaw is measured from it (see `level`).
        let pads = markers
            .iter()
            .filter(|m| m.name.strip_prefix("race_pad_").is_some_and(|n| n.parse::<u32>().is_ok()))
            .map(|m| (m.position, Vector3::new(m.yaw.cos(), 0.0, -m.yaw.sin())))
            .collect();
        Some(Self { start, gates, grid, pads })
    }

    /// The speed pad under a bike at `at` pointing `ahead`, if it is on one and going its way.
    pub fn pad_under(&self, at: Vector3<f32>, ahead: Vector3<f32>) -> Option<usize> {
        self.pads.iter().position(|&(middle, way)| {
            let to = at - middle;
            let across = Vector3::new(way.z, 0.0, -way.x);
            to.dot(&way).abs() < PAD_HALF.0
                && to.dot(&across).abs() < PAD_HALF.1
                && to.y.abs() < PAD_HEIGHT
                && ahead.dot(&way) > PAD_FACING
        })
    }

    /// Whether `at` is at gate `n`: near its middle, at its height.
    pub fn at_gate(&self, n: usize, at: Vector3<f32>) -> bool {
        let gate = self.gates[n % self.gates.len()];
        let flat = Vector3::new(at.x - gate.x, 0.0, at.z - gate.z).norm();
        flat < GATE_REACH && (at.y - gate.y).abs() < GATE_HEIGHT
    }
}

/// The way from `from` to `to`, as a yaw: left positive from the world's +z, as the player's is.
pub fn yaw_towards(from: Vector3<f32>, to: Vector3<f32>) -> f32 {
    (to.x - from.x).atan2(to.z - from.z)
}

/// `angle` brought round into -π..π.
fn wrap(angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    (angle + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI
}

/// How the bike is getting on round the track.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Progress {
    /// The gate it is making for next.
    pub next: usize,
    /// The lap it is on, from 1; past [`LAPS`] once it has finished.
    pub lap: u32,
    /// When the lap under way started, on the race's clock.
    pub lap_started: f32,
    /// How long each lap done took.
    pub laps: Vec<f32>,
}

/// What passing a gate came to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Passed {
    Checkpoint,
    /// A lap done, in this long; and the race, if it was the last.
    Lap(f32),
    Finished(f32),
}

impl Progress {
    pub fn new() -> Self {
        Self { lap: 1, ..Default::default() }
    }

    /// Whether the bike at `at` passes the gate it is making for at `time`, and what that
    /// comes to. Only that gate counts: they are passed in order or not at all.
    pub fn pass(&mut self, track: &Track, at: Vector3<f32>, time: f32) -> Option<Passed> {
        if self.lap > LAPS || !track.at_gate(self.next, at) {
            return None;
        }
        self.next += 1;
        if self.next < track.gates.len() {
            return Some(Passed::Checkpoint);
        }
        self.next = 0;
        let took = time - self.lap_started;
        self.laps.push(took);
        self.lap_started = time;
        self.lap += 1;
        Some(if self.lap > LAPS { Passed::Finished(time) } else { Passed::Lap(took) })
    }

    pub fn finished(&self) -> bool {
        self.lap > LAPS
    }

    pub fn best_lap(&self) -> Option<f32> {
        self.laps.iter().copied().reduce(f32::min)
    }

    /// The last gate passed: where the bike goes back to, put back on the track.
    pub fn last_gate(&self, track: &Track) -> Option<usize> {
        match self.next {
            0 if self.laps.is_empty() => None,
            0 => Some(track.gates.len() - 1),
            n => Some(n - 1),
        }
    }
}

/// What the rider asks of the bike this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Controls {
    /// Forward (1) or brake and back (-1).
    pub throttle: f32,
    /// Left (1) or right (-1).
    pub steer: f32,
    pub boost: bool,
}

/// Something a ray met: how far along, where, and which way the surface faces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub distance: f32,
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
}

/// How the bike moves: where it is, which way it points and how it is going, and what is under it.
#[derive(Debug, Clone, PartialEq)]
pub struct Handling {
    /// The ground under the bike's middle, as it hovers.
    pub position: Vector3<f32>,
    pub yaw: f32,
    pub velocity: Vector3<f32>,
    /// Which way is up from the road under it, as it last was.
    pub up: Vector3<f32>,
    pub grounded: bool,
    /// How much boost is left, from 1 down to 0.
    pub boost: f32,
    /// How far it is leaning into its turn right now, in radians: left positive.
    pub lean: f32,
    /// How long it is held up to [`PAD_SPEED`] yet, off a speed pad, in seconds.
    pub surge: f32,
}

impl Handling {
    pub fn new(position: Vector3<f32>, yaw: f32) -> Self {
        Self {
            position,
            yaw,
            velocity: Vector3::zeros(),
            up: Vector3::y(),
            grounded: true,
            boost: 1.0,
            lean: 0.0,
            surge: 0.0,
        }
    }

    /// Which way it points, along the ground.
    pub fn ahead(&self) -> Vector3<f32> {
        Vector3::new(self.yaw.sin(), 0.0, self.yaw.cos())
    }

    /// How fast it is going the way it points, in meters per second: backing up, less than 0.
    pub fn speed(&self) -> f32 {
        self.velocity.dot(&self.ahead())
    }

    /// Over a speed pad: kicked up towards [`PAD_SPEED`] the way it points, and held there a while.
    pub fn surge(&mut self) {
        let ahead = self.ahead();
        let along = self.velocity.dot(&ahead);
        let kicked = (along + PAD_KICK).min(PAD_SPEED).max(along);
        self.velocity += ahead * (kicked - along);
        self.surge = PAD_FOR;
    }

    /// Moves it on `dt` seconds as `controls` ask, `cast` saying what a ray from a point, along a
    /// direction (of length 1), meets within a reach.
    pub fn step(
        &mut self,
        controls: Controls,
        dt: f32,
        cast: &mut dyn FnMut(Vector3<f32>, Vector3<f32>, f32) -> Option<Hit>,
    ) {
        if dt <= 0.0 {
            return;
        }
        let up = if self.grounded { self.up } else { Vector3::y() };
        // Along the road and across it.
        let ahead = flat_on(self.ahead(), up);
        let side = up.cross(&ahead).normalize();
        let mut along = self.velocity.dot(&ahead);
        let mut across = self.velocity.dot(&side);
        let mut vertical = self.velocity.dot(&up);

        // Steering: slower the faster it goes, the other way round backing up, hardly at all at a
        // standstill.
        let speed = along.abs();
        let fast = (speed / TOP_SPEED).clamp(0.0, 1.0);
        let rate = STEER_SLOW + (STEER_FAST - STEER_SLOW) * fast;
        let crawl = (speed / 6.0).clamp(0.3, 1.0);
        let backing = if along < -0.5 { -1.0 } else { 1.0 };
        let air = if self.grounded { 1.0 } else { 0.5 };
        self.yaw = wrap(self.yaw + controls.steer.clamp(-1.0, 1.0) * rate * crawl * backing * air * dt);
        self.lean += (controls.steer * fast * LEAN - self.lean) * (1.0 - (-6.0 * dt).exp());

        // The boost, while there is some left and the rider wants it.
        let boosting = controls.boost && self.boost > 0.0 && controls.throttle > 0.0;
        self.boost = if boosting {
            (self.boost - dt / BOOST_FOR).max(0.0)
        } else {
            (self.boost + dt / BOOST_REFILL).min(1.0)
        };
        let (mut top, accel) = if boosting { (BOOST_SPEED, BOOST_ACCEL) } else { (TOP_SPEED, ACCEL) };
        // Off a speed pad, nothing holds it back from what the pad gave it a while.
        if self.surge > 0.0 {
            self.surge = (self.surge - dt).max(0.0);
            top = top.max(PAD_SPEED);
        }

        if self.grounded {
            let throttle = controls.throttle.clamp(-1.0, 1.0);
            if throttle > 0.0 {
                if along < 0.0 {
                    along = (along + BRAKE * throttle * dt).min(0.0);
                } else {
                    // Less and less to be had nearing the top speed; past it, held back to it.
                    let room = (1.0 - (along / top).powi(2)).max(0.0);
                    along += accel * throttle * room * dt;
                }
            } else if throttle < 0.0 {
                if along > 0.0 {
                    along = (along + BRAKE * throttle * dt).max(0.0);
                } else {
                    along = (along + ACCEL * 0.5 * throttle * dt).max(-REVERSE_SPEED);
                }
            } else {
                let slow = (COAST + along.abs() * 0.02) * dt;
                along -= along.signum() * slow.min(along.abs());
            }
            if along > top {
                along -= (along - top) * (1.0 - (-1.5 * dt).exp());
            }
            across *= (-GRIP * dt).exp();
            vertical = 0.0;
        } else {
            across *= (-AIR_GRIP * dt).exp();
        }
        self.velocity = ahead * along + side * across + up * vertical;
        // Downhill it gathers speed, and uphill loses it; in the air, it falls.
        let gravity = Vector3::new(0.0, -GRAVITY, 0.0);
        if self.grounded {
            self.velocity += (gravity - up * gravity.dot(&up)) * dt;
        } else {
            self.velocity += gravity * FALL * dt;
        }

        // Along, sliding off whatever wall is in the way.
        let mut moving = self.velocity * dt;
        for _ in 0..3 {
            let length = moving.norm();
            if length < 1e-5 {
                break;
            }
            let way = moving / length;
            let flat = Vector3::new(way.x, 0.0, way.z);
            if flat.norm() < 1e-3 {
                break;
            }
            let body = self.position + up * BODY;
            let reach = length + RADIUS;
            let Some(hit) = cast(body, way, reach).filter(|hit| hit.normal.y.abs() < FLOOR_UP) else {
                break;
            };
            let wall = Vector3::new(hit.normal.x, 0.0, hit.normal.z).normalize();
            // Up to it, and the rest of the way along it.
            let free = (hit.distance - RADIUS).max(0.0);
            self.position += way * free;
            let rest = moving - way * free;
            moving = rest - wall * rest.dot(&wall).min(0.0);
            self.scrape(wall, BOUNCE);
        }
        self.position += moving;
        self.keep_off_walls(cast);

        // Onto the road under it, or into the air off it.
        let reach = PROBE_UP + HOVER + if self.grounded { STICK } else { LIFT_OFF };
        let under = cast(self.position + Vector3::y() * PROBE_UP, -Vector3::y(), reach)
            .filter(|hit| hit.normal.y >= FLOOR_UP);
        match under {
            Some(hit) if self.velocity.y <= 0.5 || self.grounded => {
                let floor = hit.point.y + HOVER;
                // Up a slope at once; down it with the road, as it is followed down.
                self.position.y = floor;
                self.up = (self.up + (hit.normal - self.up) * (1.0 - (-10.0 * dt).exp())).normalize();
                if !self.grounded {
                    // Landing: what it was falling at goes into the road.
                    let into = self.velocity.dot(&hit.normal);
                    if into < 0.0 {
                        self.velocity -= hit.normal * into;
                    }
                }
                self.grounded = true;
            }
            _ => {
                self.grounded = false;
                self.up = (self.up + (Vector3::y() - self.up) * (1.0 - (-3.0 * dt).exp())).normalize();
            }
        }
    }

    /// Pushes it back out of any wall it has come into, turning or sliding: ahead and behind, to
    /// either side, and at its corners.
    fn keep_off_walls(&mut self, cast: &mut dyn FnMut(Vector3<f32>, Vector3<f32>, f32) -> Option<Hit>) {
        let ahead = self.ahead();
        let side = Vector3::new(ahead.z, 0.0, -ahead.x);
        let probes = [
            (ahead, HALF_LENGTH),
            (-ahead, HALF_LENGTH),
            (side, RADIUS),
            (-side, RADIUS),
            ((ahead * HALF_LENGTH + side * RADIUS).normalize(), (HALF_LENGTH.powi(2) + RADIUS.powi(2)).sqrt()),
            ((ahead * HALF_LENGTH - side * RADIUS).normalize(), (HALF_LENGTH.powi(2) + RADIUS.powi(2)).sqrt()),
            ((-ahead * HALF_LENGTH + side * RADIUS).normalize(), (HALF_LENGTH.powi(2) + RADIUS.powi(2)).sqrt()),
            ((-ahead * HALF_LENGTH - side * RADIUS).normalize(), (HALF_LENGTH.powi(2) + RADIUS.powi(2)).sqrt()),
        ];
        for (way, reach) in probes {
            let body = self.position + Vector3::y() * BODY;
            if let Some(hit) = cast(body, way, reach).filter(|hit| hit.normal.y.abs() < FLOOR_UP) {
                let wall = Vector3::new(hit.normal.x, 0.0, hit.normal.z).normalize();
                let depth = reach - hit.distance;
                let out = -way.dot(&wall);
                if out > 1e-3 {
                    self.position += wall * depth * out;
                }
                self.scrape(wall, 0.0);
            }
        }
    }

    /// Against a wall facing `wall`: what it was going into it taken out of how it goes - and
    /// `bounce` of that back off it - losing most of its speed square on and little glancing off;
    /// and its nose brought round along the wall, which it scrapes along rather than grinding
    /// into.
    fn scrape(&mut self, wall: Vector3<f32>, bounce: f32) {
        let into = self.velocity.dot(&wall);
        if into >= 0.0 {
            return;
        }
        let square = -into / self.velocity.norm().max(1e-3);
        self.velocity -= wall * into * (1.0 + bounce);
        self.velocity *= 1.0 - 0.5 * square;
        let along = self.velocity - wall * self.velocity.dot(&wall);
        if along.norm() > 1.0 {
            let turn = wrap(along.x.atan2(along.z) - self.yaw);
            if turn.abs() < std::f32::consts::FRAC_PI_2 {
                self.yaw = wrap(self.yaw + turn * 0.3);
            }
        }
    }

    /// How the bike is turned: up off the road, pointing its way, leaning into its turn.
    pub fn rotation(&self) -> UnitQuaternion<f32> {
        let ahead = flat_on(self.ahead(), self.up);
        UnitQuaternion::face_towards(&ahead, &self.up) * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -self.lean)
    }
}

/// `v` laid flat on the surface whose up is `up`, its length 1.
fn flat_on(v: Vector3<f32>, up: Vector3<f32>) -> Vector3<f32> {
    let flat = v - up * v.dot(&up);
    if flat.norm() < 1e-4 {
        v
    } else {
        flat.normalize()
    }
}

/// The droids racing the player, in the colours of their kinds (see `data/recolour_droids.py`):
/// red, orange, green and blue; how far off the middle of the road each keeps, left positive; and
/// how hard each takes the corners, the player's autopilot's being 1.
pub const RIVALS: [(&str, &str, f32, f32); 4] = [
    ("Red", "data/droid_hostile.glb", -3.0, 1.03),
    ("Orange", "data/droid_maintenance.glb", 3.0, 0.98),
    ("Green", "data/droid_explorer.glb", 0.0, 0.94),
    ("Blue", "data/droid_security.glb", -1.5, 1.0),
];
/// Which place on the grid the player starts from, from the front: behind the droids' front row.
const PLAYER_GRID: usize = 3;
/// How close two bikes come before they bump, side by side, in meters; and how much of what they
/// were coming together at they keep, bounced apart.
const CONTACT: f32 = 1.5;
const CONTACT_BOUNCE: f32 = 0.3;
/// How far ahead a droid looks for a bike in its way, and how far it pulls out to pass one.
const TRAFFIC_AHEAD: f32 = 12.0;
const TRAFFIC_SWERVE: f32 = 3.5;

/// What the race says to the game as it goes.
#[derive(Debug, Clone, PartialEq)]
pub enum RaceNews {
    Go,
    Lap { lap: u32, took: f32, best: bool },
    /// The player over the line for the last time: in this long, in what place of how many.
    Finished { time: f32, best_lap: Option<f32>, place: usize, of: usize },
}

/// What the HUD shows of the race.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaceStatus {
    pub time: f32,
    pub lap: u32,
    pub laps: u32,
    pub last_lap: Option<f32>,
    pub best_lap: Option<f32>,
    /// In meters per second.
    pub speed: f32,
    pub boost: f32,
    /// The player's place, and how many are racing.
    pub place: usize,
    pub racers: usize,
}

/// The keys held for the bike.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Keys {
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    boost: bool,
    reset: bool,
}

/// A bike in the scene: its root, which goes where the handling says, the bike under it, and
/// the droid riding it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Bike {
    pub root: Handle<Node>,
    pub rider: Option<Handle<Node>>,
}

/// One of those racing: the player's, first, or a droid's.
#[derive(Debug, Clone, PartialEq)]
pub struct Racer {
    pub name: &'static str,
    pub handling: Handling,
    pub progress: Progress,
    bike: Option<Bike>,
    /// How far off the middle of the road it keeps, and how hard it takes the corners, driving
    /// itself (see [`autopilot`]).
    line: f32,
    pace: f32,
    /// When it crossed the line for the last time, on the race's clock.
    pub finished: Option<f32>,
    /// The speed pad it is over, if any: each pad kicks once as it is crossed.
    on_pad: Option<usize>,
    /// Where it started on the grid, and which way it faced: where it goes back to, put back on
    /// the track before it has passed a gate.
    grid: (Vector3<f32>, f32),
}

impl Racer {
    fn new(name: &'static str, (at, yaw): (Vector3<f32>, f32), line: f32, pace: f32) -> Self {
        Self {
            grid: (at, yaw),
            name,
            handling: Handling::new(at + Vector3::y() * HOVER, yaw),
            progress: Progress::new(),
            bike: None,
            line,
            pace,
            finished: None,
            on_pad: None,
        }
    }

    /// How far round the race it has got: gates passed, then how near the next one it is - more
    /// is further on. Finished, by how soon.
    fn how_far(&self, track: &Track) -> (u32, f32) {
        if let Some(at) = self.finished {
            return (u32::MAX, -at);
        }
        let gates = track.gates.len() as u32;
        let passed = (self.progress.lap - 1) * gates + self.progress.next as u32;
        let next = track.gates[self.progress.next % track.gates.len()];
        (passed, -(next - self.handling.position).norm())
    }

    /// Back at the last gate passed - or its place on the grid, before the first - facing the
    /// next, stopped.
    fn back_on_track(&mut self, track: &Track) {
        let (at, yaw) = match self.progress.last_gate(track) {
            Some(n) => {
                let at = track.gates[n];
                let next = track.gates[(n + 1) % track.gates.len()];
                (at, yaw_towards(at, next))
            }
            None => self.grid,
        };
        let boost = self.handling.boost;
        self.handling = Handling::new(at + Vector3::y() * HOVER, yaw);
        self.handling.boost = boost;
    }

    /// Puts its bike where the handling has it, at `time` on the race's clock.
    fn place(&self, graph: &mut Graph, time: f32) {
        if let Some(bike) = &self.bike {
            let bob = (time * 2.6 + self.line).sin() * 0.025;
            graph[bike.root]
                .local_transform_mut()
                .set_position(self.handling.position + self.handling.up * bob)
                .set_rotation(self.handling.rotation());
        }
    }
}

/// Which of a pad's chevrons is lit, `clock` seconds on: one at a time, from the hindmost to the
/// front, and round again.
pub fn lit_chevron(clock: f32) -> usize {
    (clock.max(0.0) / CHEVRON_LIT) as usize % PAD_CHEVRONS
}

/// The speed pads' chevrons, lit one at a time: a material of its own for each of the pads'
/// chevrons - the hindmost of every pad sharing one - and how brightly it glows lit.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PadLights {
    chevrons: Vec<(MaterialResource, MaterialProperty)>,
    clock: f32,
    lit: Option<usize>,
}

impl PadLights {
    /// The chevrons of the track in `graph`, each given its own copy of its material.
    pub fn claim(graph: &mut Graph) -> Self {
        let mut chevrons = Vec::new();
        for k in 1..=PAD_CHEVRONS {
            let name = format!("SpeedPadChevron{k}");
            let nodes: Vec<Handle<Node>> =
                graph.pair_iter().filter(|(_, node)| node.name() == name).map(|(handle, _)| handle).collect();
            let mut copy: Option<(MaterialResource, MaterialProperty)> = None;
            for node in nodes {
                let Some(mesh) = graph[node].cast_mut::<Mesh>() else {
                    continue;
                };
                for surface in mesh.surfaces_mut() {
                    if copy.is_none() {
                        let original = surface.material().clone();
                        let state = original.state();
                        let Some(material) = state.data_ref() else {
                            continue;
                        };
                        let Some(strength) = crate::fixtures::glow_strength(material) else {
                            continue;
                        };
                        copy = Some((MaterialResource::new_embedded(material.clone()), strength));
                    }
                    if let Some((material, _)) = &copy {
                        surface.set_material(material.clone());
                    }
                }
            }
            match copy {
                Some(copy) => chevrons.push(copy),
                None => break,
            }
        }
        Log::info(format!("Race: {} speed pad chevrons to light", chevrons.len()));
        if chevrons.len() != PAD_CHEVRONS && !chevrons.is_empty() {
            Log::warn(format!("Race: the speed pads have {} glowing chevrons, not {PAD_CHEVRONS}", chevrons.len()));
        }
        Self { chevrons, clock: 0.0, lit: None }
    }

    /// The chevrons on `dt` seconds: the one lit now glowing, the others dark.
    pub fn update(&mut self, dt: f32) {
        self.clock += dt;
        let lit = lit_chevron(self.clock);
        if self.lit != Some(lit) {
            self.light(|k| k == lit);
            self.lit = Some(lit);
        }
    }

    /// Every chevron lit, as they were before the race: for the next race to find them glowing.
    pub fn restore(&mut self) {
        self.light(|_| true);
        self.lit = None;
    }

    /// Lights the chevrons that `on` says, the others dark.
    fn light(&self, on: impl Fn(usize) -> bool) {
        for (k, (material, strength)) in self.chevrons.iter().enumerate() {
            let on = on(k);
            let glow = match strength {
                MaterialProperty::Vector3(_) if !on => MaterialProperty::Vector3(Vector3::zeros()),
                MaterialProperty::Float(_) if !on => MaterialProperty::Float(0.0),
                other => other.clone(),
            };
            let shade = if on { 1.0 } else { CHEVRON_DARK };
            let mut material = material.data_ref();
            material.set_property(crate::fixtures::EMISSION_STRENGTH, glow);
            material.set_property("diffuseColor", Color::from(Vector3::repeat(shade)));
        }
    }
}

/// A race under way.
#[derive(Debug, Clone, PartialEq)]
pub struct Race {
    pub track: Track,
    /// The player's bike, and then the droids'.
    pub racers: Vec<Racer>,
    /// How long is left of the countdown, in seconds; then how long the race has gone.
    countdown: f32,
    pub time: f32,
    keys: Keys,
    autopilot: bool,
    /// Where the camera is and what it looks at, from the player's bike, as it follows: carried
    /// along with it, so that only its swing round after the bike's turns lags.
    eye: Vector3<f32>,
    target: Vector3<f32>,
    /// The speed pads' chevrons, blinking.
    lights: PadLights,
}

impl Race {
    /// A race on `track`, the player against `rivals` droids (as many of [`RIVALS`]), on the grid.
    pub fn new(track: Track, autopilot: bool, rivals: usize) -> Self {
        let slot = |n: usize| track.grid[n % track.grid.len()];
        let mut racers = vec![Racer::new("You", slot(PLAYER_GRID.min(rivals)), 0.0, 1.0)];
        for (n, &(name, _, line, pace)) in RIVALS.iter().take(rivals).enumerate() {
            let n = if n >= PLAYER_GRID.min(rivals) { n + 1 } else { n };
            racers.push(Racer::new(name, slot(n), line, pace));
        }
        let (eye, target) = chase(&racers[0].handling, 0.0);
        let at = racers[0].handling.position;
        Self {
            track,
            racers,
            countdown: COUNTDOWN,
            time: 0.0,
            keys: Keys::default(),
            autopilot,
            eye: eye - at,
            target: target - at,
            lights: PadLights::default(),
        }
    }

    /// Puts the bikes into `scene` from their `model`: the player's with the droid on it from
    /// `droid`, and each droid's with its own from `rivals`, as far as they have loaded.
    pub fn spawn(&mut self, scene: &mut Scene, model: &ModelResource, droid: Option<&ModelResource>, rivals: &[Option<ModelResource>]) {
        for (n, racer) in self.racers.iter_mut().enumerate() {
            let rider = if n == 0 { droid } else { rivals.get(n - 1).and_then(Option::as_ref) };
            racer.bike = Some(build_bike(scene, model, rider));
            racer.place(&mut scene.graph, 0.0);
        }
        self.lights = PadLights::claim(&mut scene.graph);
    }

    /// Takes the bikes out of the scene.
    pub fn remove(&mut self, graph: &mut Graph) {
        self.lights.restore();
        for racer in &mut self.racers {
            if let Some(bike) = racer.bike.take() {
                graph.remove_node(bike.root);
            }
        }
    }

    pub fn on_key(&mut self, code: KeyCode, pressed: bool) {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => self.keys.forward = pressed,
            KeyCode::KeyS | KeyCode::ArrowDown => self.keys.back = pressed,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.keys.left = pressed,
            KeyCode::KeyD | KeyCode::ArrowRight => self.keys.right = pressed,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.keys.boost = pressed,
            KeyCode::KeyR if pressed => self.keys.reset = true,
            _ => (),
        }
    }

    /// Lets go of every key: the window lost focus, or the menu opened.
    pub fn release_keys(&mut self) {
        self.keys = Keys::default();
    }

    /// What the keys ask of the player's bike.
    fn keyed(&self) -> Controls {
        let k = self.keys;
        let axis = |plus: bool, minus: bool| (plus as i32 - minus as i32) as f32;
        Controls {
            throttle: axis(k.forward, k.back),
            steer: axis(k.left, k.right),
            boost: k.boost,
        }
    }

    /// Runs the race on `dt` seconds: the countdown, then every bike round the track, `skip` -
    /// the player's own body, carried along with theirs - left out of what they meet. With
    /// `steering` off, nothing the player holds counts.
    pub fn update(&mut self, graph: &mut Graph, dt: f32, skip: Handle<Collider>, steering: bool) -> Vec<RaceNews> {
        let mut news = Vec::new();
        let started = self.countdown <= 0.0;
        if !started {
            self.countdown -= dt;
            if self.countdown <= 0.0 {
                news.push(RaceNews::Go);
            }
        } else if self.racers[0].finished.is_none() {
            self.time += dt;
        }
        let reset = std::mem::take(&mut self.keys.reset);
        let keyed = self.keyed();
        let bikes: Vec<(Vector3<f32>, Vector3<f32>)> =
            self.racers.iter().map(|r| (r.handling.position, r.handling.ahead())).collect();
        let racers = self.racers.len();
        for n in 0..racers {
            let racer = &mut self.racers[n];
            let driving = started && !racer.progress.finished();
            let controls = match (driving, n == 0 && !self.autopilot) {
                (false, _) => Controls::default(),
                (true, true) if steering => keyed,
                (true, true) => Controls::default(),
                (true, false) => {
                    let line = racer.line + traffic(&racer.handling, n, &bikes);
                    autopilot(&racer.handling, &self.track, &racer.progress, line, racer.pace)
                }
            };
            if n == 0 && reset && driving {
                racer.back_on_track(&self.track);
            }
            let mut cast = |from: Vector3<f32>, way: Vector3<f32>, reach: f32| first_hit(graph, from, way, reach, skip);
            racer.handling.step(controls, dt, &mut cast);
            let pad = if racer.handling.grounded {
                self.track.pad_under(racer.handling.position, racer.handling.ahead())
            } else {
                None
            };
            if let Some(on) = pad.filter(|_| pad != racer.on_pad) {
                let before = racer.handling.speed();
                racer.handling.surge();
                if n == 0 {
                    Log::info(format!(
                        "Race: speed pad {}, {:.0} to {:.0} km/h",
                        on + 1,
                        before * 3.6,
                        racer.handling.speed() * 3.6
                    ));
                }
            }
            racer.on_pad = pad;
            // Fallen off the world - or a droid stuck, a long way off its way round: back on the
            // track.
            if racer.handling.position.y < self.track.start.y - 40.0 {
                racer.back_on_track(&self.track);
            }
            if !driving {
                continue;
            }
            let best = racer.progress.best_lap();
            match racer.progress.pass(&self.track, racer.handling.position, self.time) {
                Some(Passed::Lap(took)) if n == 0 => news.push(RaceNews::Lap {
                    lap: racer.progress.lap - 1,
                    took,
                    best: best.is_none_or(|b| took < b),
                }),
                Some(Passed::Finished(time)) => {
                    racer.finished = Some(time);
                    if n == 0 {
                        let best_lap = racer.progress.best_lap();
                        let place = self.place_of(0);
                        news.push(RaceNews::Finished { time, best_lap, place, of: racers });
                    }
                }
                _ => (),
            }
        }
        bump(&mut self.racers);
        self.lights.update(dt);
        self.follow(graph, dt, skip);
        for racer in &self.racers {
            racer.place(graph, self.time);
        }
        news
    }

    /// The racers in the order they stand, first first: by how far round they have got.
    pub fn standings(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.racers.len()).collect();
        order.sort_by(|&a, &b| {
            let (a, b) = (self.racers[a].how_far(&self.track), self.racers[b].how_far(&self.track));
            b.0.cmp(&a.0).then(b.1.total_cmp(&a.1))
        });
        order
    }

    /// Which place racer `n` is in, from 1.
    pub fn place_of(&self, n: usize) -> usize {
        self.standings().iter().position(|&r| r == n).map_or(self.racers.len(), |p| p + 1)
    }

    /// Brings the camera on after the player's bike, in front of anything between them.
    fn follow(&mut self, graph: &Graph, dt: f32, skip: Handle<Collider>) {
        let handling = &self.racers[0].handling;
        let at = handling.position;
        let (eye, target) = chase(handling, handling.speed());
        let k = 1.0 - (-CHASE_FOLLOW * dt).exp();
        self.eye += (eye - at - self.eye) * k;
        self.target += (target - at - self.target) * (1.0 - (-CHASE_FOLLOW * 2.0 * dt).exp());
        // In front of whatever is between it and the bike: a wall, the tunnel's roof.
        let from = at + Vector3::y() * (CHASE_LOOK_UP + 0.3);
        let way = at + self.eye - from;
        let length = way.norm();
        if length > 1e-3 {
            if let Some(hit) = first_hit(graph, from, way / length, length, skip) {
                self.eye = from + way / length * (hit.distance - CHASE_CLEARANCE).max(0.5) - at;
            }
        }
    }

    /// Where the body that carries the camera goes, and where the camera is and looks.
    pub fn view(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let at = self.racers[0].handling.position;
        (at, at + self.eye, at + self.target)
    }

    pub fn status(&self) -> RaceStatus {
        let you = &self.racers[0];
        RaceStatus {
            time: self.time,
            lap: you.progress.lap.min(LAPS),
            laps: LAPS,
            last_lap: you.progress.laps.last().copied(),
            best_lap: you.progress.best_lap(),
            speed: you.handling.speed().max(0.0),
            boost: you.handling.boost,
            place: self.place_of(0),
            racers: self.racers.len(),
        }
    }

    /// The final order as it stands, a line each: who, and their time if they have finished.
    pub fn results(&self) -> String {
        self.standings()
            .iter()
            .enumerate()
            .map(|(p, &n)| {
                let racer = &self.racers[n];
                let time = racer.finished.map_or("racing".to_string(), crate::hud::format_time);
                format!("{}. {}  {time}", p + 1, racer.name)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The countdown's number, or GO for a moment after it, to show over the track.
    pub fn countdown_text(&self) -> Option<String> {
        if self.countdown > 0.0 {
            Some(format!("{}", self.countdown.ceil() as u32))
        } else if self.time < 1.0 && self.racers[0].finished.is_none() {
            Some("GO!".to_string())
        } else {
            None
        }
    }
}

/// A bike in `scene` from its `model`, with the droid on it from `droid`, if there is one.
pub(crate) fn build_bike(scene: &mut Scene, model: &ModelResource, droid: Option<&ModelResource>) -> Bike {
    let bike = model.instantiate(scene);
    let rider = droid.map(|droid| droid.instantiate(scene));
    let graph = &mut scene.graph;
    let root: Handle<Node> = PivotBuilder::new(BaseBuilder::new()).build(graph).to_base();
    graph.link_nodes(bike, root);
    // Made facing the droid on it; the droid faces +z.
    graph[bike]
        .local_transform_mut()
        .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), std::f32::consts::PI));
    if let Some(rider) = rider {
        graph.link_nodes(rider, root);
        // As the player's droid is: its pieces and broken ends, which it carries for coming
        // apart, out of sight - or its green insides would show through its shell - and its
        // guns put away.
        let _ = crate::dismember::Dismember::new(graph, rider);
        for gun in ["pistol", "rifle"] {
            if let Some((node, _)) = graph.find_by_name(rider, gun) {
                graph[node].set_visibility(false);
            }
        }
        if !hold_pose(graph, rider, RIDE) {
            Log::warn(format!("Race: the droid has no {RIDE}; it stands on the bike"));
        }
    }
    // The droid is scaled down to the size it is played at, and the bike it was posed on with it.
    graph[root]
        .local_transform_mut()
        .set_scale(Vector3::repeat(crate::player::DROID_SCALE));
    Bike { root, rider }
}

/// How far racer `n` pulls out of its line to get by a bike just ahead of it in `bikes` (where each
/// is and which way it points): away from it, the side it is on.
fn traffic(handling: &Handling, n: usize, bikes: &[(Vector3<f32>, Vector3<f32>)]) -> f32 {
    let ahead = handling.ahead();
    let left = Vector3::new(ahead.z, 0.0, -ahead.x);
    let mut swerve = 0.0;
    for (m, &(at, _)) in bikes.iter().enumerate() {
        if m == n {
            continue;
        }
        let to = at - handling.position;
        let (front, side) = (to.dot(&ahead), to.dot(&left));
        if front > 0.0 && front < TRAFFIC_AHEAD && side.abs() < 2.5 && to.y.abs() < 3.0 {
            swerve = if side > 0.0 { -TRAFFIC_SWERVE } else { TRAFFIC_SWERVE };
        }
    }
    swerve
}

/// Bikes that have come together pushed apart, bouncing off each other.
fn bump(racers: &mut [Racer]) {
    for a in 0..racers.len() {
        for b in a + 1..racers.len() {
            let (pa, pb) = (racers[a].handling.position, racers[b].handling.position);
            let apart = Vector3::new(pb.x - pa.x, 0.0, pb.z - pa.z);
            let distance = apart.norm();
            if distance >= CONTACT || (pb.y - pa.y).abs() > 1.5 || distance < 1e-4 {
                continue;
            }
            let normal = apart / distance;
            let push = normal * (CONTACT - distance) / 2.0;
            racers[a].handling.position -= push;
            racers[b].handling.position += push;
            let closing = (racers[a].handling.velocity - racers[b].handling.velocity).dot(&normal);
            if closing > 0.0 {
                let impulse = normal * closing * (1.0 + CONTACT_BOUNCE) / 2.0;
                racers[a].handling.velocity -= impulse;
                racers[b].handling.velocity += impulse;
            }
        }
    }
}

/// Where the camera wants to be behind the bike going at `speed`, and what it looks at.
fn chase(handling: &Handling, speed: f32) -> (Vector3<f32>, Vector3<f32>) {
    let ahead = handling.ahead();
    let fast = (speed / BOOST_SPEED).clamp(0.0, 1.0);
    let back = CHASE_BACK + (CHASE_BACK_FAST - CHASE_BACK) * fast;
    let at = handling.position;
    let eye = at - ahead * back + Vector3::y() * CHASE_UP;
    let target = at + ahead * CHASE_AHEAD + Vector3::y() * CHASE_LOOK_UP;
    (eye, target)
}

/// Steers for the checkpoints ahead, `line` meters off their middles (left positive): flat out
/// where the way on is straight, easing off and braking into the turns - the harder `pace` is
/// over 1, the less.
pub fn autopilot(handling: &Handling, track: &Track, progress: &Progress, line: f32, pace: f32) -> Controls {
    let n = track.gates.len();
    let near = track.gates[progress.next % n];
    let far = track.gates[(progress.next + 1) % n];
    // Its line: off the middle of the road, square across the way on.
    let way = (far - near).try_normalize(1e-5).unwrap_or_else(Vector3::z);
    let left = Vector3::new(way.z, 0.0, -way.x);
    let (near_on, far_on) = (near + left * line, far + left * line);
    // Through the next gate towards the one after, a little, the nearer it is.
    let to_near = Vector3::new(near_on.x - handling.position.x, 0.0, near_on.z - handling.position.z).norm();
    let blend = (1.0 - to_near / 25.0).clamp(0.0, 1.0);
    let mut aim = near_on + (far_on - near_on).try_normalize(1e-5).unwrap_or_default() * 8.0 * blend;
    // A speed pad just ahead, not far off its line and pointing its way: over it.
    let heading = handling.ahead();
    let side = Vector3::new(heading.z, 0.0, -heading.x);
    let pad = track
        .pads
        .iter()
        .filter(|&&(at, way)| {
            let to = at - handling.position;
            let front = to.dot(&heading);
            front > PAD_HALF.0 && front < PAD_SEEN && to.dot(&side).abs() < PAD_DETOUR && to.y.abs() < PAD_HEIGHT * 2.0 && way.dot(&heading) > 0.8
        })
        .min_by(|a, b| (a.0 - handling.position).norm().total_cmp(&(b.0 - handling.position).norm()));
    if let Some(&(at, way)) = pad {
        // Through its middle along it, rather than across it.
        aim = at + way * PAD_HALF.0;
    }
    let off = wrap(yaw_towards(handling.position, aim) - handling.yaw);
    // How sharp the way on turns at the next gate, and at the one after it, as the bike nears
    // each: slow enough for the sharper, in time to make it.
    let after = track.gates[(progress.next + 2) % n];
    let at_near = wrap(yaw_towards(near, far) - yaw_towards(handling.position, near)).abs();
    let at_far = wrap(yaw_towards(far, after) - yaw_towards(near, far)).abs();
    let to_far = to_near + (far - near).norm();
    let soon = |bend: f32, distance: f32| bend * (1.0 - distance / 70.0).clamp(0.0, 1.0);
    let bend = soon(at_near, to_near).max(soon(at_far, to_far));
    let speed = handling.speed();
    let wanted = TOP_SPEED * (1.0 - bend * 0.9 / pace.max(0.1)).clamp(0.3, 1.0);
    // Pointed well off, it slows and comes round at a crawl, rather than backing up - which
    // would steer it the other way.
    let throttle = if off.abs() > 1.2 {
        if speed > 10.0 { -1.0 } else { 0.4 }
    } else if speed > wanted + 3.0 {
        -1.0
    } else if speed > wanted {
        0.0
    } else {
        1.0
    };
    Controls {
        throttle,
        steer: (off * 2.5).clamp(-1.0, 1.0),
        boost: off.abs() < 0.08 && bend < 0.1 * pace,
    }
}

/// The first thing a ray from `from` along `way` meets within `reach`, but `skip`.
pub(crate) fn first_hit(graph: &Graph, from: Vector3<f32>, way: Vector3<f32>, reach: f32, skip: Handle<Collider>) -> Option<Hit> {
    let mut hits = Vec::new();
    graph.physics.cast_ray(
        RayCastOptions {
            ray_origin: Point3::from(from),
            ray_direction: way,
            max_len: reach,
            groups: Default::default(),
            sort_results: true,
        },
        &mut hits,
    );
    hits.into_iter().find(|hit| hit.collider != skip).map(|hit| {
        let normal = if hit.normal.dot(&way) > 0.0 { -hit.normal } else { hit.normal };
        Hit {
            distance: hit.toi,
            point: hit.position.coords,
            normal,
        }
    })
}

/// Holds the droid under `root` in its pose `name`, every other animation off: whether it has it.
fn hold_pose(graph: &mut Graph, root: Handle<Node>, name: &str) -> bool {
    let nodes: Vec<Handle<Node>> = graph.traverse_handle_iter(root).collect();
    let Some(player) = nodes.into_iter().find(|&node| graph[node].cast::<AnimationPlayer>().is_some()) else {
        return false;
    };
    let Ok(player) = graph.try_get_mut_of_type::<AnimationPlayer>(player) else {
        return false;
    };
    let mut found = false;
    for (_, animation) in player.animations_mut().get_value_mut_silent().pair_iter_mut() {
        let this = animation.name() == name;
        animation.set_enabled(this);
        if this {
            animation.set_loop(true).rewind();
            found = true;
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(name: &str, x: f32, z: f32) -> Marker {
        Marker { name: name.to_string(), position: Vector3::new(x, 0.0, z), yaw: 0.0 }
    }

    /// A square track 100 m on a side, its checkpoints at the other three corners.
    fn square() -> Track {
        Track::from_markers(&[
            marker("race_start", 0.0, 0.0),
            marker("checkpoint_2", 100.0, 100.0),
            marker("checkpoint_1", 0.0, 100.0),
            marker("checkpoint_3", 100.0, 0.0),
            marker("race_grid_1", 0.0, -10.0),
        ])
        .unwrap()
    }

    #[test]
    fn a_track_is_its_checkpoints_in_order_and_then_the_line() {
        let track = square();
        assert_eq!(
            track.gates,
            vec![Vector3::new(0.0, 0.0, 100.0), Vector3::new(100.0, 0.0, 100.0), Vector3::new(100.0, 0.0, 0.0), Vector3::zeros()]
        );
        // The grid faces the first checkpoint: along +z.
        assert!(track.grid[0].1.abs() < 1e-5);
        assert!(Track::from_markers(&[marker("race_start", 0.0, 0.0)]).is_none());
        assert!(Track::from_markers(&[marker("checkpoint_1", 0.0, 0.0)]).is_none());
    }

    #[test]
    fn a_gate_counts_only_at_its_own_height() {
        let track = square();
        assert!(track.at_gate(0, Vector3::new(5.0, 1.0, 98.0)));
        assert!(!track.at_gate(0, Vector3::new(20.0, 0.0, 100.0)));
        // The viaduct over the straight is not the straight.
        assert!(!track.at_gate(0, Vector3::new(0.0, 10.0, 100.0)));
    }

    #[test]
    fn laps_are_every_gate_in_turn_and_the_race_is_three_of_them() {
        let track = square();
        let mut progress = Progress::new();
        // The line first is nothing: a lap is the checkpoints and then the line.
        assert_eq!(progress.pass(&track, Vector3::zeros(), 0.0), None);
        // Out of order is nothing either.
        assert_eq!(progress.pass(&track, Vector3::new(100.0, 0.0, 100.0), 1.0), None);
        let mut time = 0.0;
        let mut results = Vec::new();
        for _ in 0..LAPS {
            for gate in track.gates.clone() {
                time += 10.0;
                results.push(progress.pass(&track, gate, time));
            }
        }
        assert_eq!(results[0], Some(Passed::Checkpoint));
        assert_eq!(results[3], Some(Passed::Lap(40.0)));
        assert_eq!(results[7], Some(Passed::Lap(40.0)));
        assert_eq!(results[11], Some(Passed::Finished(120.0)));
        assert!(progress.finished());
        assert_eq!(progress.laps, vec![40.0; 3]);
        // Finished, nothing more counts.
        assert_eq!(progress.pass(&track, track.gates[0], 130.0), None);
    }

    #[test]
    fn put_back_on_the_track_it_goes_to_the_last_gate_passed() {
        let track = square();
        let mut progress = Progress::new();
        assert_eq!(progress.last_gate(&track), None);
        progress.pass(&track, track.gates[0], 1.0);
        assert_eq!(progress.last_gate(&track), Some(0));
        for gate in track.gates[1..].to_vec() {
            progress.pass(&track, gate, 2.0);
        }
        assert_eq!(progress.last_gate(&track), Some(3));
    }

    /// A flat road at `y`, walled at x = ±`half` - as a ray caster for the handling.
    fn road(half: f32) -> impl FnMut(Vector3<f32>, Vector3<f32>, f32) -> Option<Hit> {
        move |from: Vector3<f32>, way: Vector3<f32>, reach: f32| {
            let mut best: Option<Hit> = None;
            let mut consider = |t: f32, normal: Vector3<f32>| {
                if t >= 0.0 && t <= reach && best.is_none_or(|b| t < b.distance) {
                    best = Some(Hit { distance: t, point: from + way * t, normal });
                }
            };
            if way.y < -1e-6 {
                consider(-from.y / way.y, Vector3::y());
            }
            if way.x > 1e-6 {
                consider((half - from.x) / way.x, -Vector3::x());
            }
            if way.x < -1e-6 {
                consider((-half - from.x) / way.x, Vector3::x());
            }
            best
        }
    }

    fn run(handling: &mut Handling, controls: Controls, seconds: f32, cast: &mut dyn FnMut(Vector3<f32>, Vector3<f32>, f32) -> Option<Hit>) {
        let dt = 1.0 / 60.0;
        for _ in 0..(seconds / dt) as usize {
            handling.step(controls, dt, cast);
        }
    }

    #[test]
    fn flat_out_it_reaches_its_top_speed_and_no_more_hovering_all_the_while() {
        let mut cast = road(100.0);
        let mut bike = Handling::new(Vector3::new(0.0, HOVER, 0.0), 0.0);
        let go = Controls { throttle: 1.0, ..Default::default() };
        run(&mut bike, go, 4.0, &mut cast);
        let at_four = bike.speed();
        assert!(at_four > 20.0 && at_four < TOP_SPEED, "{at_four}");
        run(&mut bike, go, 10.0, &mut cast);
        assert!((bike.speed() - TOP_SPEED).abs() < 2.0, "{}", bike.speed());
        assert!((bike.position.y - HOVER).abs() < 1e-3 && bike.grounded);
        // Straight on, along +z.
        assert!(bike.position.x.abs() < 1e-3 && bike.position.z > 300.0);
        // Boosting it goes faster, as long as the boost lasts.
        run(&mut bike, Controls { boost: true, ..go }, BOOST_FOR, &mut cast);
        assert!(bike.speed() > TOP_SPEED + 5.0, "{}", bike.speed());
        assert!(bike.boost < 0.05);
        // Braking stops it, and holding on, it backs up.
        run(&mut bike, Controls { throttle: -1.0, ..Default::default() }, 4.0, &mut cast);
        assert!(bike.speed() < 0.0 && bike.speed() >= -REVERSE_SPEED - 0.1, "{}", bike.speed());
    }

    #[test]
    fn it_turns_the_way_it_is_steered_and_holds_its_line() {
        let mut cast = road(1000.0);
        let mut bike = Handling::new(Vector3::new(0.0, HOVER, 0.0), 0.0);
        run(&mut bike, Controls { throttle: 1.0, ..Default::default() }, 3.0, &mut cast);
        run(&mut bike, Controls { throttle: 1.0, steer: 1.0, ..Default::default() }, 0.5, &mut cast);
        // Left is +x, turning from +z.
        assert!(bike.yaw > 0.3, "{}", bike.yaw);
        assert!(bike.lean > 0.1);
        // Its velocity comes round after it.
        let heading = bike.velocity.x.atan2(bike.velocity.z);
        run(&mut bike, Controls { throttle: 1.0, ..Default::default() }, 1.0, &mut cast);
        let after = bike.velocity.x.atan2(bike.velocity.z);
        assert!((after - bike.yaw).abs() < (heading - bike.yaw).abs() + 1e-3);
        assert!((after - bike.yaw).abs() < 0.05, "{after} {}", bike.yaw);
    }

    #[test]
    fn it_glances_off_a_wall_rather_than_going_through_it() {
        let mut cast = road(8.0);
        // Flat out at 45 degrees into the wall at x = 8.
        let mut bike = Handling::new(Vector3::new(0.0, HOVER, 0.0), std::f32::consts::FRAC_PI_4);
        bike.velocity = bike.ahead() * TOP_SPEED;
        for _ in 0..120 {
            bike.step(Controls { throttle: 1.0, ..Default::default() }, 1.0 / 60.0, &mut cast);
            assert!(bike.position.x < 8.0 - RADIUS * 0.5, "through the wall at {:?}", bike.position);
        }
        // It goes on along the wall, slower.
        assert!(bike.velocity.z > 5.0 && bike.velocity.norm() < TOP_SPEED, "{:?} at {:?} yaw {}", bike.velocity, bike.position, bike.yaw);
    }

    #[test]
    fn off_the_edge_it_flies_and_falls_and_lands_on_what_is_below() {
        // A road ending at z = 20, with another 3 m lower beyond it.
        let mut cast = |from: Vector3<f32>, way: Vector3<f32>, reach: f32| {
            if way.y > -1e-6 {
                return None;
            }
            let floor = if from.z < 20.0 { 3.0 } else { 0.0 };
            let t = (floor - from.y) / way.y;
            (t >= 0.0 && t <= reach).then(|| Hit { distance: t, point: from + way * t, normal: Vector3::y() })
        };
        let mut bike = Handling::new(Vector3::new(0.0, 3.0 + HOVER, 0.0), 0.0);
        bike.velocity = Vector3::new(0.0, 0.0, 30.0);
        let mut flew = false;
        for _ in 0..90 {
            bike.step(Controls { throttle: 1.0, ..Default::default() }, 1.0 / 60.0, &mut cast);
            flew |= !bike.grounded;
            assert!(bike.position.y >= HOVER - 1e-3);
        }
        assert!(flew && bike.grounded && (bike.position.y - HOVER).abs() < 1e-3, "{:?}", bike.position);
    }

    #[test]
    fn a_speed_pad_kicks_a_bike_going_its_way_once_and_holds_it_there_a_while() {
        // A pad half way up the square's first side, kicking along +z: its marker's +x is +z.
        let mut markers = vec![
            marker("race_start", 0.0, 0.0),
            marker("checkpoint_1", 0.0, 100.0),
            marker("checkpoint_2", 100.0, 100.0),
        ];
        markers.push(Marker { yaw: -std::f32::consts::FRAC_PI_2, ..marker("race_pad_1", 0.0, 50.0) });
        let track = Track::from_markers(&markers).unwrap();
        assert_eq!(track.pads.len(), 1);
        assert!((track.pads[0].1 - Vector3::z()).norm() < 1e-5, "{:?}", track.pads[0].1);
        assert_eq!(track.pad_under(Vector3::new(1.0, 0.0, 52.0), Vector3::z()), Some(0));
        // Off it, or going the other way over it, nothing.
        assert_eq!(track.pad_under(Vector3::new(4.0, 0.0, 50.0), Vector3::z()), None);
        assert_eq!(track.pad_under(Vector3::new(0.0, 0.0, 50.0), -Vector3::z()), None);
        assert_eq!(track.pad_under(Vector3::new(0.0, 10.0, 50.0), Vector3::z()), None);

        // Raced over: flat out it comes off the pad well over its top speed, and it keeps most of
        // that for a while after.
        let mut race = Race::new(track, false, 0);
        race.countdown = 0.0;
        let mut cast = road(100.0);
        let racer = &mut race.racers[0];
        racer.handling = Handling::new(Vector3::new(0.0, HOVER, 30.0), 0.0);
        racer.handling.velocity = Vector3::z() * TOP_SPEED;
        let go = Controls { throttle: 1.0, ..Default::default() };
        let mut kicks = 0;
        for _ in 0..30 {
            racer.handling.step(go, 1.0 / 60.0, &mut cast);
            let pad = race.track.pad_under(racer.handling.position, racer.handling.ahead());
            if pad.is_some() && pad != racer.on_pad {
                racer.handling.surge();
                kicks += 1;
            }
            racer.on_pad = pad;
        }
        assert_eq!(kicks, 1);
        assert!(racer.handling.speed() > PAD_SPEED - 2.0, "{}", racer.handling.speed());
        run(&mut racer.handling, go, PAD_FOR * 0.5, &mut cast);
        assert!(racer.handling.speed() > TOP_SPEED + 10.0, "{}", racer.handling.speed());
        // And then it bleeds back down to its top speed.
        run(&mut racer.handling, go, 6.0, &mut cast);
        assert!((racer.handling.speed() - TOP_SPEED).abs() < 2.0, "{}", racer.handling.speed());
    }

    #[test]
    fn a_pads_chevrons_light_one_at_a_time_from_the_back() {
        let lit: Vec<usize> = (0..7).map(|k| lit_chevron((k as f32 + 0.5) * CHEVRON_LIT)).collect();
        assert_eq!(lit, vec![0, 1, 2, 0, 1, 2, 0]);
    }

    #[test]
    fn a_droid_makes_for_a_speed_pad_just_ahead_of_it() {
        let mut markers = vec![marker("race_start", 0.0, 0.0), marker("checkpoint_1", 0.0, 200.0)];
        markers.push(Marker { yaw: -std::f32::consts::FRAC_PI_2, ..marker("race_pad_1", 4.0, 40.0) });
        let track = Track::from_markers(&markers).unwrap();
        let progress = Progress::new();
        let bike = Handling::new(Vector3::new(0.0, HOVER, 10.0), 0.0);
        // Left is +x: it steers left, for the pad, not straight on for the checkpoint.
        assert!(autopilot(&bike, &track, &progress, 0.0, 1.0).steer > 0.1);
        // Too far off its line, it goes for the checkpoint as if there were no pad.
        let plain = Track { pads: Vec::new(), ..track.clone() };
        let far = Handling::new(Vector3::new(-8.0, HOVER, 10.0), 0.0);
        assert_eq!(autopilot(&far, &track, &progress, 0.0, 1.0), autopilot(&far, &plain, &progress, 0.0, 1.0));
    }

    #[test]
    fn the_standings_go_by_gates_passed_then_by_how_near_the_next() {
        let mut race = Race::new(square(), false, 2);
        // All on the grid, none past a gate: whoever is nearest the first is ahead.
        race.racers[0].handling.position = Vector3::new(0.0, 0.0, 90.0);
        race.racers[1].handling.position = Vector3::new(0.0, 0.0, 50.0);
        race.racers[2].handling.position = Vector3::new(0.0, 0.0, 70.0);
        assert_eq!(race.standings(), vec![0, 2, 1]);
        // A gate passed counts for more than being near the next.
        race.racers[1].progress.next = 1;
        assert_eq!(race.standings(), vec![1, 0, 2]);
        assert_eq!(race.place_of(0), 2);
        // Finished, first over the line first.
        race.racers[2].finished = Some(100.0);
        race.racers[0].finished = Some(90.0);
        assert_eq!(race.standings(), vec![0, 2, 1]);
        assert!(race.results().starts_with("1. You"));
    }

    #[test]
    fn bikes_that_come_together_bump_apart() {
        let mut race = Race::new(square(), false, 1);
        race.racers[0].handling.position = Vector3::new(0.0, 0.0, 0.0);
        race.racers[0].handling.velocity = Vector3::new(5.0, 0.0, 0.0);
        race.racers[1].handling.position = Vector3::new(1.0, 0.0, 0.0);
        race.racers[1].handling.velocity = Vector3::zeros();
        bump(&mut race.racers);
        let apart = race.racers[1].handling.position - race.racers[0].handling.position;
        assert!((apart.norm() - CONTACT).abs() < 1e-4);
        // The one that ran into the other has given it some of its speed.
        assert!(race.racers[0].handling.velocity.x < 5.0 && race.racers[1].handling.velocity.x > 0.0);
        // Over and under, as on the viaduct, they pass.
        race.racers[1].handling.position = Vector3::new(0.5, 10.0, 0.0);
        let before = race.racers[1].handling.position;
        bump(&mut race.racers);
        assert_eq!(race.racers[1].handling.position, before);
    }

    #[test]
    fn the_autopilot_drives_round_the_track() {
        // Round the square on an open plain.
        let mut cast = road(1000.0);
        let track = square();
        let mut progress = Progress::new();
        let (at, yaw) = track.grid[0];
        let mut bike = Handling::new(at + Vector3::y() * HOVER, yaw);
        let dt = 1.0 / 60.0;
        let mut time = 0.0;
        while !progress.finished() && time < 120.0 {
            let controls = autopilot(&bike, &track, &progress, 0.0, 1.0);
            bike.step(controls, dt, &mut cast);
            time += dt;
            progress.pass(&track, bike.position, time);
        }
        assert!(progress.finished(), "lap {} gate {} at {:?}", progress.lap, progress.next, bike.position);
    }
}
