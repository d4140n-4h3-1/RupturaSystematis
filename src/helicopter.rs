//! Helicopters: a sentry flying a security helicopter round and round over the city, its rotors
//! turning. On the test level for now (see [`crate::ctf::DEMO_TOWN`]).
//!
//! The model, [`MODEL`], is made in Blender (~/Documents/blender/vehicles/helicopter): its nose
//! along -z as exported, so it is turned half round to face +z, the way the droid in it faces;
//! `helicopter_rotor` and `helicopter_tail_rotor` turn in its clip `rotors`, one turn of the main
//! rotor a second as made, played [`ROTOR_RATE`] times as fast. The pilot is a sentry - a droid in
//! security's livery, [`PILOT_MODEL`] - holding `droid_pilot`, the pose it was made sitting in the
//! pilot's seat in, hands on the cyclic and the collective, feet on the pedals (baked by
//! ~/Documents/blender/droid/build_ride.py). Both are put under one pivot, as a bike and its rider
//! are (see [`crate::race`]), and scaled alike, [`DROID_SCALE`].
//!
//! It flies a circle round the middle of the city's streets ([`Orbit`]), high enough to clear
//! every roof under the circle by [`CLEARANCE`]: nose a little down as a helicopter flies forward,
//! banked into the turn as fast a circle as it flies calls for (no more than [`MAX_BANK`]), and
//! bobbing a little.
//!
//! Security calls one in after the player too ([`Pursuit`]): a sentry with a radio that gets its
//! call through, the player wanted at enough stars, brings one in from far off. It never shoots.
//! It flies to the player, high enough over the roofs under it and under where it is going, and
//! circles them, its searchlight on them - on where it last saw them, sweeping, while it cannot -
//! and whenever it sees them, tells security where they are (see the game's own `chase_by_air`).
//! Wanted for nothing any more, or out of its sight long enough, it flies off and is gone.

use crate::player::DROID_SCALE;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{
        animation::AnimationPlayer,
        base::BaseBuilder,
        collider::Collider,
        graph::Graph,
        light::{spot::SpotLightBuilder, BaseLightBuilder},
        node::Node,
        pivot::PivotBuilder,
        Scene,
    },
};

/// The helicopter's model, and the sentry's who flies it.
pub const MODEL: &str = "data/helicopter.glb";
pub const PILOT_MODEL: &str = "data/droid_security.glb";
/// The pilot's pose, and the rotors' clip.
const POSE: &str = "droid_pilot";
const ROTORS: &str = "rotors";
/// How many times as fast as made the rotors' clip plays: the main rotor some five turns a second.
const ROTOR_RATE: f32 = 5.0;
/// How fast it flies round, in meters a second; how far over the highest roof under its circle;
/// how steeply it banks at most and how far its nose is down, in degrees; and how far and how
/// often it bobs, in meters and seconds.
const SPEED: f32 = 16.0;
pub const CLEARANCE: f32 = 25.0;
const MAX_BANK: f32 = 25.0;
const NOSE_DOWN: f32 = 5.0;
const BOB: f32 = 0.6;
const BOB_PERIOD: f32 = 5.0;
/// How far out from the middle of the streets it circles, as much of their reach.
const REACH: f32 = 0.8;
const GRAVITY: f32 = 9.81;

/// The circle a helicopter flies: round `centre` (x, z), `radius` out, `height` up, at `speed`,
/// anticlockwise seen from above.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit {
    pub centre: [f32; 2],
    pub radius: f32,
    pub height: f32,
    pub speed: f32,
}

impl Orbit {
    /// The circle over the streets `roads` (each `[x0, z0, x1, z1]`), at `height`: round their
    /// middle, [`REACH`] of the way out to their furthest edge. None with no streets.
    pub fn over(roads: &[[f32; 4]], height: f32) -> Option<Self> {
        let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
        for r in roads {
            lo = [lo[0].min(r[0].min(r[2])), lo[1].min(r[1].min(r[3]))];
            hi = [hi[0].max(r[0].max(r[2])), hi[1].max(r[1].max(r[3]))];
        }
        if roads.is_empty() {
            return None;
        }
        let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        let radius = REACH * ((hi[0] - lo[0]).max(hi[1] - lo[1]) / 2.0);
        Some(Self { centre, radius, height, speed: SPEED })
    }

    /// The point on the circle at `angle` radians round from +x, along the ground.
    pub fn at(&self, angle: f32) -> Vector3<f32> {
        Vector3::new(self.centre[0] + self.radius * angle.cos(), self.height, self.centre[1] + self.radius * angle.sin())
    }

    /// Where it is and how it is turned `time` seconds in: on the circle, nose along the way it
    /// goes and a little down, banked into the turn, bobbing.
    pub fn pose(&self, time: f32) -> (Vector3<f32>, UnitQuaternion<f32>) {
        let angle = time * self.speed / self.radius.max(1.0);
        let bob = BOB * (std::f32::consts::TAU * time / BOB_PERIOD).sin();
        let at = self.at(angle) + Vector3::new(0.0, bob, 0.0);
        let ahead = Vector3::new(-angle.sin(), 0.0, angle.cos());
        let inward = Vector3::new(-angle.cos(), 0.0, -angle.sin());
        let bank = (self.speed * self.speed / (GRAVITY * self.radius.max(1.0))).atan().min(MAX_BANK.to_radians());
        let up = Vector3::y() * bank.cos() + inward * bank.sin();
        let nose = ahead * NOSE_DOWN.to_radians().cos() - Vector3::y() * NOSE_DOWN.to_radians().sin();
        (at, UnitQuaternion::face_towards(&nose, &up))
    }
}

/// One helicopter in the scene: its pivot, its circle, and how long it has flown.
#[derive(Debug, PartialEq)]
pub struct Helicopter {
    root: Handle<Node>,
    orbit: Orbit,
    time: f32,
}

impl Helicopter {
    /// A helicopter from `model`, `pilot` flying it, put in `scene` on `orbit`, `time` seconds round.
    pub fn spawn(scene: &mut Scene, model: &ModelResource, pilot: Option<&ModelResource>, orbit: Orbit, time: f32) -> Self {
        let root = build(scene, model, pilot);
        let heli = Self { root, orbit, time };
        heli.place(&mut scene.graph);
        heli
    }

    /// Flies on another `dt`.
    pub fn fly(&mut self, graph: &mut Graph, dt: f32) {
        self.time += dt;
        self.place(graph);
    }

    fn place(&self, graph: &mut Graph) {
        let (at, turn) = self.orbit.pose(self.time);
        graph[self.root].local_transform_mut().set_position(at).set_rotation(turn);
    }

    /// Out of the scene.
    pub fn remove(&mut self, graph: &mut Graph) {
        if graph.is_valid_handle(self.root) {
            graph.remove_node(self.root);
        }
    }
}

/// A helicopter from `model`, `pilot` flying it, put in `scene`: its pivot, scaled as the droids are.
fn build(scene: &mut Scene, model: &ModelResource, pilot: Option<&ModelResource>) -> Handle<Node> {
    let body = model.instantiate(scene);
    let crew = pilot.map(|pilot| pilot.instantiate(scene));
    let graph = &mut scene.graph;
    let root: Handle<Node> = PivotBuilder::new(BaseBuilder::new()).build(graph).to_base();
    graph.link_nodes(body, root);
    // Made nose along -z; the droid in it faces +z.
    graph[body]
        .local_transform_mut()
        .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), std::f32::consts::PI));
    play_rotors(graph, body);
    if let Some(crew) = crew {
        graph.link_nodes(crew, root);
        // As a bike's rider: its pieces for coming apart out of sight, its guns put away.
        let _ = crate::dismember::Dismember::new(graph, crew);
        for gun in ["pistol", "rifle", "sniper"] {
            if let Some((node, _)) = graph.find_by_name(crew, gun) {
                graph[node].set_visibility(false);
            }
        }
        if !crate::race::hold_pose(graph, crew, POSE) {
            Log::warn(format!("Helicopter: the pilot has no {POSE}; it stands in the cockpit"));
        }
    }
    graph[root].local_transform_mut().set_scale(Vector3::repeat(DROID_SCALE));
    root
}

/// After the player: how fast it flies at most, in meters a second, and how quickly it gets up
/// to speed or changes way, in meters a second each second; how far out round them it circles,
/// and how fast; how near it slows down to come to where it is going.
pub const CHASE_SPEED: f32 = 30.0;
const CHASE_ACCEL: f32 = 8.0;
pub const CHASE_RADIUS: f32 = 30.0;
const CIRCLE_SPEED: f32 = 12.0;
const SLOWING: f32 = 0.6;
/// How far off it comes from, and goes off to before it is gone, in meters along the ground.
pub const CHASE_FROM: f32 = 260.0;
const GONE_AT: f32 = 320.0;
/// How quickly its height follows the roofs: up, fast, down, slowly, in meters a second.
const CLIMB: f32 = 12.0;
const DESCEND: f32 = 4.0;
/// Its searchlight: how far it reaches and how wide its cone is, in meters and degrees, how bright,
/// and how far and how fast it sweeps about where it last saw the player while it cannot see them.
const LIGHT_REACH: f32 = 170.0;
const LIGHT_CONE: f32 = 9.0;
const LIGHT_BRIGHT: f32 = 6.0;
const SWEEP: f32 = 9.0;
const SWEEP_RATE: f32 = 0.7;
/// How far below its pivot the searchlight hangs, in meters.
const LIGHT_BELOW: f32 = 1.2;

/// Where a helicopter after the player heads for, on its way to circle `over` (where the player
/// is, or was last seen), `round` radians round them: on its circle, at `height`.
pub fn circling(over: Vector3<f32>, round: f32, height: f32) -> Vector3<f32> {
    Vector3::new(over.x + CHASE_RADIUS * round.cos(), height, over.z + CHASE_RADIUS * round.sin())
}

/// One step of flying at `at`, going at `velocity`, towards `target` for `dt`: as fast as it may
/// fly ([`CHASE_SPEED`]), slowing as it comes near, no quicker to change speed or way than
/// [`CHASE_ACCEL`]. Where it is then, and how fast it is going.
pub fn steer(at: Vector3<f32>, velocity: Vector3<f32>, target: Vector3<f32>, dt: f32) -> (Vector3<f32>, Vector3<f32>) {
    let to = target - at;
    let distance = to.norm();
    let wanted = if distance > 1.0e-3 { to / distance * (distance * SLOWING).min(CHASE_SPEED) } else { Vector3::zeros() };
    let change = wanted - velocity;
    let most = CHASE_ACCEL * dt;
    let change = if change.norm() > most { change / change.norm() * most } else { change };
    let velocity = velocity + change;
    (at + velocity * dt, velocity)
}

/// A helicopter after the player.
#[derive(Debug, PartialEq)]
pub struct Pursuit {
    root: Handle<Node>,
    light: Handle<Node>,
    at: Vector3<f32>,
    velocity: Vector3<f32>,
    /// Which way its nose is, in radians round from +z, and how far it is banked, in radians.
    heading: f32,
    bank: f32,
    /// How far round the player it is on its circle, in radians.
    round: f32,
    /// How high it means to fly: over the roofs under it and under where it is going.
    height: f32,
    /// Where it is flying off to, once it is leaving.
    leaving: Option<Vector3<f32>>,
    time: f32,
    /// Where its searchlight is on.
    lit: Vector3<f32>,
}

impl Pursuit {
    /// A helicopter from `model`, `pilot` flying it, coming in at `at`, nose towards `towards`.
    pub fn spawn(scene: &mut Scene, model: &ModelResource, pilot: Option<&ModelResource>, at: Vector3<f32>, towards: Vector3<f32>) -> Self {
        let root = build(scene, model, pilot);
        let light = SpotLightBuilder::new(
            BaseLightBuilder::new(BaseBuilder::new().with_cast_shadows(false)).with_intensity(LIGHT_BRIGHT),
        )
        .with_distance(LIGHT_REACH)
        .with_hotspot_cone_angle(LIGHT_CONE.to_radians())
        .build(&mut scene.graph)
        .to_base();
        let to = towards - at;
        let round = (at.z - towards.z).atan2(at.x - towards.x);
        let chase = Self {
            root,
            light,
            at,
            velocity: Vector3::zeros(),
            heading: to.x.atan2(to.z),
            bank: 0.0,
            round,
            height: at.y,
            leaving: None,
            time: 0.0,
            lit: towards,
        };
        chase.show(&mut scene.graph);
        chase
    }

    /// Where its searchlight is, which is where it looks from.
    pub fn eye(&self) -> Vector3<f32> {
        self.at - Vector3::new(0.0, LIGHT_BELOW, 0.0)
    }

    /// Whether it is flying off.
    pub fn leaving(&self) -> bool {
        self.leaving.is_some()
    }

    /// Sets it flying off, away from `from`, and gone once far enough.
    pub fn leave(&mut self, from: Vector3<f32>) {
        if self.leaving.is_none() {
            let away = Vector3::new(self.at.x - from.x, 0.0, self.at.z - from.z).try_normalize(1.0e-3).unwrap_or_else(Vector3::x);
            self.leaving = Some(from + away * (GONE_AT + 50.0) + Vector3::new(0.0, self.height + 40.0 - from.y, 0.0));
        }
    }

    /// Whether, flying off, it has got far enough from `from` to be gone.
    pub fn gone(&self, from: Vector3<f32>) -> bool {
        self.leaving.is_some() && Vector3::new(self.at.x - from.x, 0.0, self.at.z - from.z).norm() > GONE_AT
    }

    /// Flies on another `dt`: circling `over`, where the player is or was last seen - or off, if it
    /// is leaving - as high as `roof` says clears what is under it and where it is going, by
    /// [`CLEARANCE`]; its searchlight on `seen`, the player, or sweeping about `over`. Shown there
    /// by [`Pursuit::show`].
    pub fn fly(
        &mut self,
        over: Vector3<f32>,
        seen: Option<Vector3<f32>>,
        roof: impl Fn(Vector3<f32>) -> f32,
        dt: f32,
    ) {
        if dt <= 0.0 {
            return;
        }
        self.time += dt;
        let going = match self.leaving {
            Some(away) => away,
            None => {
                // Round the circle at its own pace, once it is near it; on its way there, the
                // point on it nearest.
                let near = Vector3::new(self.at.x - over.x, 0.0, self.at.z - over.z).norm() < CHASE_RADIUS * 2.0;
                if near {
                    self.round += CIRCLE_SPEED / CHASE_RADIUS * dt;
                } else {
                    self.round = (self.at.z - over.z).atan2(self.at.x - over.x);
                }
                circling(over, self.round, self.height)
            }
        };
        let wanted = roof(self.at).max(roof(going)) + CLEARANCE;
        let rate = if wanted > self.height { CLIMB } else { DESCEND };
        self.height += (wanted - self.height).clamp(-rate * dt, rate * dt);
        let target = Vector3::new(going.x, self.height, going.z);
        let was = self.heading;
        let (at, velocity) = steer(self.at, self.velocity, target, dt);
        self.at = at;
        self.velocity = velocity;
        // Nose the way it goes, once it goes anywhere; banked into the turn as a turn that quick at
        // that speed calls for; nose down the faster it goes.
        let flat_speed = Vector3::new(velocity.x, 0.0, velocity.z).norm();
        if flat_speed > 1.0 {
            self.heading = velocity.x.atan2(velocity.z);
        }
        let turning = wrap(self.heading - was) / dt;
        let bank = (flat_speed * turning / GRAVITY).atan().clamp(-MAX_BANK.to_radians(), MAX_BANK.to_radians());
        self.bank += (bank - self.bank) * (1.0 - (-3.0 * dt).exp());
        self.lit = seen.unwrap_or_else(|| {
            let sweep = SWEEP_RATE * self.time;
            over + Vector3::new(SWEEP * sweep.cos(), 0.0, SWEEP * (1.7 * sweep).sin())
        });
    }

    /// Puts it where it is in the scene, turned as it flies, its searchlight on what it lights.
    pub fn show(&self, graph: &mut Graph) {
        self.place(graph);
        self.aim_light(graph, self.lit);
    }

    fn place(&self, graph: &mut Graph) {
        let speed = Vector3::new(self.velocity.x, 0.0, self.velocity.z).norm();
        let nose_down = NOSE_DOWN.to_radians() * (0.5 + speed / CHASE_SPEED);
        let ahead = Vector3::new(self.heading.sin(), 0.0, self.heading.cos());
        let right = Vector3::new(-ahead.z, 0.0, ahead.x);
        // Banked to the left for a turn to the left, as heading grows that way.
        let up = Vector3::y() * self.bank.cos() - right * self.bank.sin();
        let nose = ahead * nose_down.cos() - Vector3::y() * nose_down.sin();
        let bob = BOB * (std::f32::consts::TAU * self.time / BOB_PERIOD).sin();
        graph[self.root]
            .local_transform_mut()
            .set_position(self.at + Vector3::new(0.0, bob, 0.0))
            .set_rotation(UnitQuaternion::face_towards(&nose, &up));
    }

    /// Points its searchlight, under it, at `at`.
    fn aim_light(&self, graph: &mut Graph, at: Vector3<f32>) {
        let from = self.eye();
        let Some(way) = (at - from).try_normalize(1.0e-3) else {
            return;
        };
        // The light shines down its own -y.
        let turn = UnitQuaternion::rotation_between(&-Vector3::y(), &way).unwrap_or_else(|| {
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), std::f32::consts::PI)
        });
        if let Ok(light) = graph.try_get_mut(self.light) {
            light.local_transform_mut().set_position(from).set_rotation(turn);
        }
    }

    /// Out of the scene, and its light with it.
    pub fn remove(&mut self, graph: &mut Graph) {
        for node in [self.root, self.light] {
            if graph.is_valid_handle(node) {
                graph.remove_node(node);
            }
        }
    }
}

fn wrap(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// Spins the rotors of the helicopter under `body`: its `rotors` clip on, looping, at
/// [`ROTOR_RATE`]; every other clip it has off.
fn play_rotors(graph: &mut Graph, body: Handle<Node>) {
    let nodes: Vec<Handle<Node>> = graph.traverse_handle_iter(body).collect();
    let Some(player) = nodes.into_iter().find(|&node| graph[node].cast::<AnimationPlayer>().is_some()) else {
        Log::warn("Helicopter: its rotors have no clip; they stand still".to_string());
        return;
    };
    let Ok(player) = graph.try_get_mut_of_type::<AnimationPlayer>(player) else {
        return;
    };
    for (_, animation) in player.animations_mut().get_value_mut_silent().pair_iter_mut() {
        let rotors = animation.name() == ROTORS;
        animation.set_enabled(rotors);
        if rotors {
            animation.set_loop(true).set_speed(ROTOR_RATE);
        }
    }
}

/// How high a circle `orbit` has to fly to clear every roof under it by [`CLEARANCE`]: looked
/// down on from far above at `samples` points round it with `cast` - the first thing hit, going
/// `way` from a point, within a reach - and never lower than `floor` plus the clearance.
pub fn clear_height(
    orbit: &Orbit,
    samples: usize,
    floor: f32,
    mut cast: impl FnMut(Vector3<f32>, Vector3<f32>, f32) -> Option<Vector3<f32>>,
) -> f32 {
    const FROM: f32 = 600.0;
    let top = (0..samples)
        .filter_map(|k| {
            let above = orbit.at(std::f32::consts::TAU * k as f32 / samples as f32) + Vector3::new(0.0, FROM, 0.0);
            cast(above, -Vector3::y(), 2.0 * FROM).map(|hit| hit.y)
        })
        .fold(floor, f32::max);
    top + CLEARANCE
}

/// A circle over `roads` clear of everything `graph` has under it (see [`clear_height`]),
/// looking past the player's `skip`; above `floor`, the ground the player starts on.
pub fn orbit_over(graph: &Graph, roads: &[[f32; 4]], floor: f32, skip: Handle<Collider>) -> Option<Orbit> {
    let orbit = Orbit::over(roads, floor)?;
    let height = clear_height(&orbit, 48, floor, |from, way, reach| {
        crate::race::first_hit(graph, from, way, reach, skip).map(|hit| hit.point)
    });
    Some(Orbit { height, ..orbit })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEMO: [[f32; 4]; 2] = [[-74.0, -74.0, -58.0, 74.0], [-74.0, 58.0, 74.0, 74.0]];

    #[test]
    fn after_the_player_it_flies_in_no_faster_than_it_may_and_comes_to_a_stop_there() {
        let (mut at, mut velocity) = (Vector3::new(0.0, 60.0, -CHASE_FROM), Vector3::zeros());
        let target = Vector3::new(0.0, 60.0, 0.0);
        let mut fastest: f32 = 0.0;
        for _ in 0..(60.0 / 0.02) as usize {
            (at, velocity) = steer(at, velocity, target, 0.02);
            fastest = fastest.max(velocity.norm());
        }
        assert!(fastest <= CHASE_SPEED + 1e-3 && fastest > 0.9 * CHASE_SPEED, "up to speed: {fastest}");
        assert!((at - target).norm() < 1.0 && velocity.norm() < 0.5, "there, and still: {at:?}");
        // Gets up to speed no quicker than it may.
        let (_, v) = steer(Vector3::zeros(), Vector3::zeros(), Vector3::new(500.0, 0.0, 0.0), 0.5);
        assert!((v.norm() - CHASE_ACCEL * 0.5).abs() < 1e-3);
    }

    #[test]
    fn it_circles_the_player_out_round_them() {
        let over = Vector3::new(10.0, 0.0, -5.0);
        for round in [0.0, 1.0, 4.0] {
            let at = circling(over, round, 70.0);
            assert!(((at - over).xz().norm() - CHASE_RADIUS).abs() < 1e-3);
            assert_eq!(at.y, 70.0);
        }
    }

    #[test]
    fn it_circles_the_middle_of_the_streets() {
        let orbit = Orbit::over(&DEMO, 50.0).unwrap();
        assert_eq!(orbit.centre, [0.0, 0.0]);
        assert!((orbit.radius - 0.8 * 74.0).abs() < 1e-4);
        for t in [0.0, 3.0, 11.5] {
            let (at, _) = orbit.pose(t);
            let out = (at.x.powi(2) + at.z.powi(2)).sqrt();
            assert!((out - orbit.radius).abs() < 1e-3, "on the circle");
            assert!((at.y - 50.0).abs() <= BOB + 1e-4, "at its height, bobbing");
        }
        assert!(Orbit::over(&[], 0.0).is_none());
    }

    #[test]
    fn it_flies_nose_first_a_little_down_banked_into_the_turn() {
        let orbit = Orbit { centre: [0.0, 0.0], radius: 60.0, height: 40.0, speed: SPEED };
        let dt = 0.01;
        for t in [0.0, 7.0, 20.0] {
            let (a, turn) = orbit.pose(t);
            let (b, _) = orbit.pose(t + dt);
            let along = Vector3::new(b.x - a.x, 0.0, b.z - a.z).normalize();
            let nose = turn * Vector3::z();
            assert!(nose.dot(&along) > 0.99, "nose the way it goes");
            assert!(nose.y < 0.0 && nose.y > -0.15, "nose a little down: {}", nose.y);
            let up = turn * Vector3::y();
            let inward = -Vector3::new(a.x, 0.0, a.z).normalize();
            let bank = up.dot(&inward).asin().to_degrees();
            assert!(bank > 5.0 && bank <= MAX_BANK + 0.01, "banked into the turn: {bank}");
        }
    }

    #[test]
    fn it_flies_clear_of_the_highest_roof_under_it() {
        let orbit = Orbit { centre: [0.0, 0.0], radius: 60.0, height: 0.0, speed: SPEED };
        // A tower 70 m tall where the circle crosses +x; the rest of the city 20 m.
        let height = clear_height(&orbit, 48, 0.0, |from, _, _| {
            Some(Vector3::new(from.x, if from.x > 55.0 { 70.0 } else { 20.0 }, from.z))
        });
        assert_eq!(height, 70.0 + CLEARANCE);
        // Nothing under it at all: over the ground.
        assert_eq!(clear_height(&orbit, 48, 3.0, |_, _, _| None), 3.0 + CLEARANCE);
    }
}
