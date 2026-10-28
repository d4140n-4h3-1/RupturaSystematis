//! The train on Urbs' elevated railway: three cars floating over the beam down the middle avenue,
//! from the station in the north to the far end of the line in the south and back, over and over.
//! At the station it stops with its doors open, both sides, for a while; at the far end, where
//! there is no platform, it waits with them shut.
//!
//! Its model, [`TRAIN_MODEL`], is made in Blender (`~/Documents/blender/train/build_train.py`):
//! `train`, its cars `car_1` to `car_3`, and under each its door leaves,
//! `car_<n>_door_<side>_<k>_<leaf>` - side L or R, leaf a toward the train's back or b toward its
//! front. Its own clips are left off: the doors slide and the cars bob on their field here, in step
//! with where the train is.
//!
//! The cars are on a body of their own, moved by its velocity, as a ferry's deck is, so that whoever
//! stands in them is carried along (see [`crate::ferry`]). A shut doorway is closed by a collider
//! of its own, put out of the way while the doors are open. The droids keep off it: the survey of
//! the level is made before it is put in.

use fyrox::{
    core::{algebra::Vector3, log::Log, pool::Handle},
    graph::SceneGraph,
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{
        animation::AnimationPlayer,
        base::BaseBuilder,
        collider::{ColliderBuilder, ColliderShape, GeometrySource},
        graph::Graph,
        node::Node,
        rigidbody::{RigidBody, RigidBodyBuilder, RigidBodyType},
        transform::TransformBuilder,
        Scene,
    },
};

pub const TRAIN_MODEL: &str = "data/train.glb";

/// How high the beam's top is, which the train's origin rides on, in meters.
const BEAM_TOP: f32 = 9.5;
/// Where along the line, in the world's z, the train stops at the station, and where it turns
/// back at its far end. (In Urbs' model the station is 332 to 372 m north, which is -z here.)
const STATION: f32 = -352.0;
const FAR_END: f32 = 440.0;
/// How fast it goes at its fastest, half way, in meters per second.
const TOP_SPEED: f32 = 18.0;
/// How long it stands at the station and at the far end, in seconds; and, at the station, how
/// long after it stops its doors start to open, and how long before it leaves they start to shut.
const AT_STATION: f32 = 25.0;
const AT_END: f32 = 8.0;
const DOORS_AFTER: f32 = 1.0;
const DOORS_BEFORE: f32 = 3.0;
/// How long the doors take to open or shut, in seconds; and how far a leaf comes out from the
/// side first, and then slides along it, in meters.
const DOORS_TIME: f32 = 1.25;
const DOOR_OUT: f32 = 0.06;
const DOOR_SLIDE: f32 = 0.78;
/// How far the cars bob up and down on their field, in meters, and how long a bob takes.
const HOVER: f32 = 0.012;
const HOVER_TIME: f32 = 2.0;
/// A doorway's blocker: how far out from the middle it stands, how high its middle is over the
/// beam's top, and half how thick, high and wide it is.
const BLOCKER_OUT: f32 = 2.53;
const BLOCKER_MIDDLE: f32 = -0.25;
const BLOCKER_HALF: [f32; 3] = [0.06, 1.0, 0.75];
/// Where a doorway's blocker goes while the doors are open: well up out of anyone's way.
const OUT_OF_THE_WAY: f32 = 50.0;

/// A door leaf: its node, where it is shut, and which way it moves - out to its side, and along.
#[derive(Debug, PartialEq)]
struct Leaf {
    node: Handle<Node>,
    shut: Vector3<f32>,
    out: f32,
    along: f32,
}

#[derive(Debug, Default, PartialEq)]
pub struct Train {
    /// The body the model rides on.
    body: Handle<Node>,
    /// The cars, and where each is at rest.
    cars: Vec<(Handle<Node>, Vector3<f32>)>,
    leaves: Vec<Leaf>,
    /// The doorways' blockers, and where each stands with the doors shut.
    blockers: Vec<(Handle<Node>, Vector3<f32>)>,
    /// Seconds into its round of the line.
    clock: f32,
}

impl Train {
    /// Puts the train from `model` on the line, at the station with its doors shut.
    pub fn spawn(scene: &mut Scene, model: &ModelResource) -> Self {
        let start = Vector3::new(0.0, BEAM_TOP, STATION);
        let root = model.instantiate(scene);
        let graph = &mut scene.graph;
        graph[root].local_transform_mut().set_position(start);
        graph.update_hierarchical_data();
        let nodes: Vec<Handle<Node>> = graph.traverse_handle_iter(root).collect();
        // Its own clips are left off.
        for &node in &nodes {
            if let Ok(player) = graph.try_get_mut_of_type::<AnimationPlayer>(node) {
                for (_, animation) in player.animations_mut().get_value_mut_silent().pair_iter_mut() {
                    animation.set_enabled(false);
                }
            }
        }
        let mut cars = Vec::new();
        let mut leaves = Vec::new();
        let mut blockers = Vec::new();
        for &node in &nodes {
            let name = graph[node].name().to_string();
            if let Some(door) = name.split_once("_door_").map(|(_, door)| door.to_string()) {
                // L or R, which doorway along the car, and the leaf a or b.
                let parts: Vec<&str> = door.split('_').collect();
                let [side, _, leaf] = parts[..] else { continue };
                let out = if side == "R" { 1.0 } else { -1.0 };
                // Blender's +y, the train's front, is the world's -z.
                let along = if leaf == "b" { -1.0 } else { 1.0 };
                let shut = **graph[node].local_transform().position();
                leaves.push(Leaf { node, shut, out, along });
                if leaf == "a" {
                    let at = graph[node].global_position() - start;
                    blockers.push(Vector3::new(out * BLOCKER_OUT, BLOCKER_MIDDLE, at.z));
                }
            } else if name.starts_with("car_") && graph[node].is_mesh() {
                cars.push((node, **graph[node].local_transform().position()));
            }
        }
        // The cars' shape, all but the doors, which move.
        let shape = ColliderBuilder::new(BaseBuilder::new())
            .with_shape(ColliderShape::trimesh(cars.iter().map(|&(car, _)| GeometrySource(car)).collect()))
            .build(graph);
        let mut body = BaseBuilder::new()
            .with_name("train")
            .with_local_transform(TransformBuilder::new().with_local_position(start).build())
            .with_child(shape);
        let blockers: Vec<(Handle<Node>, Vector3<f32>)> = blockers
            .into_iter()
            .map(|at| {
                let [x, y, z] = BLOCKER_HALF;
                let blocker = ColliderBuilder::new(
                    BaseBuilder::new().with_local_transform(TransformBuilder::new().with_local_position(at).build()),
                )
                .with_shape(ColliderShape::cuboid(x, y, z))
                .build(graph)
                .to_base();
                (blocker, at)
            })
            .collect();
        for &(blocker, _) in &blockers {
            body = body.with_child(blocker);
        }
        let body = RigidBodyBuilder::new(body)
            .with_body_type(RigidBodyType::KinematicVelocityBased)
            .build(graph)
            .to_base();
        graph.update_hierarchical_data();
        graph.link_nodes_keep_global_transform(root, body);
        Log::info(format!(
            "Train: {} cars, {} door leaves, {} doorways, {:.0} m of line",
            cars.len(),
            leaves.len(),
            blockers.len(),
            FAR_END - STATION
        ));
        Self { body, cars, leaves, blockers, clock: 0.0 }
    }

    /// Takes the train out of the scene.
    pub fn remove(&self, graph: &mut Graph) {
        if graph.try_get(self.body).is_ok() {
            graph.remove_node(self.body);
        }
    }

    /// Back at the station, as a round begins.
    pub fn reset(&mut self, graph: &mut Graph) {
        self.clock = 0.0;
        if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(self.body) {
            body.local_transform_mut().set_position(Vector3::new(0.0, BEAM_TOP, STATION));
            body.set_lin_vel(Vector3::zeros());
        }
        self.pose(graph);
    }

    /// Moves the train on by `dt`: given the velocity that takes it where it should be by the end
    /// of this step, its doors and cars where they are then.
    pub fn update(&mut self, graph: &mut Graph, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        self.clock += dt;
        if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(self.body) {
            let here = **body.local_transform().position();
            let there = Vector3::new(0.0, BEAM_TOP, along_line(self.clock));
            body.set_lin_vel((there - here) / dt);
        }
        self.pose(graph);
    }

    fn pose(&self, graph: &mut Graph) {
        let open = doors_open(self.clock);
        let out = DOOR_OUT * (open / 0.2).min(1.0);
        let slide = DOOR_SLIDE * ((open - 0.2) / 0.8).clamp(0.0, 1.0);
        for leaf in &self.leaves {
            if let Ok(node) = graph.try_get_mut(leaf.node) {
                let at = leaf.shut + Vector3::new(leaf.out * out, 0.0, leaf.along * slide);
                node.local_transform_mut().set_position(at);
            }
        }
        let up = Vector3::new(0.0, OUT_OF_THE_WAY, 0.0);
        for &(blocker, at) in &self.blockers {
            if let Ok(node) = graph.try_get_mut(blocker) {
                node.local_transform_mut().set_position(if open > 0.0 { at + up } else { at });
            }
        }
        for (k, &(car, rest)) in self.cars.iter().enumerate() {
            let phase = self.clock / HOVER_TIME * std::f32::consts::TAU + k as f32 * 0.9;
            if let Ok(node) = graph.try_get_mut(car) {
                node.local_transform_mut().set_position(rest + Vector3::new(0.0, HOVER * phase.sin(), 0.0));
            }
        }
    }
}

/// How long it takes to go from one end of the line to the other, in seconds: setting off gently
/// and slowing gently to a stop, at [`TOP_SPEED`] half way.
fn crossing() -> f32 {
    1.5 * (FAR_END - STATION).abs() / TOP_SPEED
}

/// Where along the line, in the world's z, the train is `time` seconds into its round: standing
/// at the station, going to the far end, standing there, and coming back.
fn along_line(time: f32) -> f32 {
    let cross = crossing();
    let time = time.rem_euclid(AT_STATION + AT_END + 2.0 * cross);
    let eased = |t: f32| {
        let u = (t / cross).clamp(0.0, 1.0);
        u * u * (3.0 - 2.0 * u)
    };
    let share = if time < AT_STATION {
        0.0
    } else if time < AT_STATION + cross {
        eased(time - AT_STATION)
    } else if time < AT_STATION + cross + AT_END {
        1.0
    } else {
        1.0 - eased(time - AT_STATION - cross - AT_END)
    };
    STATION + (FAR_END - STATION) * share
}

/// How far open the doors are `time` seconds into the train's round, from 0, shut, to 1: open at
/// the station only.
fn doors_open(time: f32) -> f32 {
    let cross = crossing();
    let time = time.rem_euclid(AT_STATION + AT_END + 2.0 * cross);
    if time >= AT_STATION {
        return 0.0;
    }
    let opening = ((time - DOORS_AFTER) / DOORS_TIME).clamp(0.0, 1.0);
    let shutting = ((time - (AT_STATION - DOORS_BEFORE)) / DOORS_TIME).clamp(0.0, 1.0);
    opening - shutting
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_stands_at_the_station_goes_to_the_far_end_and_comes_back() {
        assert_eq!(along_line(0.0), STATION);
        assert_eq!(along_line(AT_STATION - 0.1), STATION);
        let there = AT_STATION + crossing() + AT_END / 2.0;
        assert!((along_line(there) - FAR_END).abs() < 1e-3);
        let round = AT_STATION + AT_END + 2.0 * crossing();
        assert!((along_line(round - 0.01) - STATION).abs() < 0.01);
        assert_eq!(along_line(round + 1.0), STATION);
    }

    #[test]
    fn it_is_never_faster_than_its_top_speed() {
        let step = 0.05;
        let round = AT_STATION + AT_END + 2.0 * crossing();
        let mut t = 0.0;
        while t < round {
            let speed = (along_line(t + step) - along_line(t)).abs() / step;
            assert!(speed <= TOP_SPEED + 0.01, "{speed} at {t}");
            t += step;
        }
    }

    #[test]
    fn the_doors_open_only_while_it_stands_at_the_station() {
        assert_eq!(doors_open(0.0), 0.0);
        assert_eq!(doors_open(AT_STATION / 2.0), 1.0);
        assert_eq!(doors_open(AT_STATION - 0.5), 0.0);
        let mut t = 0.0;
        while t < 300.0 {
            if doors_open(t) > 0.0 {
                assert_eq!(along_line(t), STATION, "open at {t} away from the station");
            }
            t += 0.1;
        }
    }
}
