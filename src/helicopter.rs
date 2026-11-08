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

use crate::player::DROID_SCALE;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{animation::AnimationPlayer, base::BaseBuilder, collider::Collider, graph::Graph, node::Node, pivot::PivotBuilder, Scene},
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
            for gun in ["pistol", "rifle"] {
                if let Some((node, _)) = graph.find_by_name(crew, gun) {
                    graph[node].set_visibility(false);
                }
            }
            if !crate::race::hold_pose(graph, crew, POSE) {
                Log::warn(format!("Helicopter: the pilot has no {POSE}; it stands in the cockpit"));
            }
        }
        graph[root].local_transform_mut().set_scale(Vector3::repeat(DROID_SCALE));
        let heli = Self { root, orbit, time };
        heli.place(graph);
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
