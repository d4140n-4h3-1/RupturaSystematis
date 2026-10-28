//! The train on Urbs' elevated railway: three cars floating over the beam down the middle avenue,
//! from the station in the north through the one in the middle to the one in the south and back,
//! over and over. At each it stops with its doors open, both sides, for a while.
//!
//! Its model, [`TRAIN_MODEL`], is made in Blender (`~/Documents/blender/train/build_train.py`):
//! `train`, its cars `car_1` to `car_3`, and under each its door leaves,
//! `car_<n>_door_<side>_<k>_<leaf>` - side L or R, leaf a toward the train's back or b toward its
//! front. Its own clips are left off: the doors slide and the cars bob on their field here, in step
//! with where the train is.
//!
//! The cars are on a body of their own, moved by its velocity, as a ferry's deck is, so that whoever
//! stands in them is carried along (see [`crate::ferry`]). A shut doorway is closed by a collider
//! of its own, which nothing meets while the doors are open: the engine never moves a collider on
//! its body once it has made it, so it cannot be put out of the way instead. The survey of the level is made before
//! it is put in, so it is no part of the droids' grid: those that take it go by its [`Doors`] instead,
//! getting on at one station and off at another (see [`crate::inhabitants`]).

use fyrox::{
    core::{algebra::Vector3, log::Log, pool::Handle},
    graph::SceneGraph,
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{
        animation::AnimationPlayer,
        base::BaseBuilder,
        collider::{BitMask, Collider, ColliderBuilder, ColliderShape, GeometrySource, InteractionGroups},
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
/// Where along the line, in the world's z, the middle of each station is, where the train stands
/// at it, from the north end of the line to the south. (In Urbs' model they are 352 m north, 32 m
/// north and 352 m south of the middle, north being -z here.)
const STATIONS: [f32; 3] = [-352.0, -32.0, 352.0];
/// The stations it stands at in its round, in order: from the north to the south and back.
const ROUND: [usize; 4] = [0, 1, 2, 1];
/// How fast it goes at its fastest, half way between stations, in meters per second.
const TOP_SPEED: f32 = 18.0;
/// How long it stands at each station, in seconds; how long after it stops its doors start to
/// open, and how long before it leaves they start to shut.
const AT_STATION: f32 = 20.0;
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
/// beam's top, and half how thick, high and wide it is: thick enough that no one is pushed
/// through it.
const BLOCKER_OUT: f32 = 2.53;
const BLOCKER_MIDDLE: f32 = -0.25;
const BLOCKER_HALF: [f32; 3] = [0.2, 1.0, 0.75];
/// The floor in the cars, under the beam's top; and, from the middle, how far out the inside of
/// their walls is and how far along their ends are, in meters: within them, someone is aboard.
const FLOOR: f32 = -1.25;
const INSIDE: f32 = 2.45;
const HALF_LENGTH: f32 = 19.0;
/// The platforms either side of it at the stations, under the beam's top.
const PLATFORM: f32 = 8.0 - BEAM_TOP;

/// The train as the droids go by it: where it is, where its doorways are on it, and whether its
/// doors are open to get on and off by.
#[derive(Debug, Clone, PartialEq)]
pub struct Doors {
    /// The middle of the beam's top under it, where it will be at the end of this step.
    pub at: Vector3<f32>,
    /// The middle of each doorway, on the floor in it, from `at`.
    pub doorways: Vec<Vector3<f32>>,
    /// While its doors are wide open, at which station, and for how much longer, in seconds.
    pub open: Option<(usize, f32)>,
}

impl Doors {
    /// How many stations there are.
    pub fn stations(&self) -> usize {
        STATIONS.len()
    }

    /// Where the train stands at the `n`th station, its doors open.
    pub fn station(&self, n: usize) -> Vector3<f32> {
        Vector3::new(0.0, BEAM_TOP, STATIONS[n])
    }

    /// Whether `at` is in one of its cars, by the plan - not out on the platform beside it.
    pub fn within(&self, at: Vector3<f32>) -> bool {
        let off = at - self.at;
        off.x.abs() < INSIDE && off.z.abs() < HALF_LENGTH
    }

    /// The height of its floor.
    pub fn floor(&self) -> f32 {
        self.at.y + FLOOR
    }

    /// The platform beside the `n`th doorway at the `station`, `out` meters out from it.
    pub fn platform(&self, station: usize, n: usize, out: f32) -> Vector3<f32> {
        let doorway = self.doorways[n];
        self.station(station) + Vector3::new(doorway.x + doorway.x.signum() * out, PLATFORM, doorway.z)
    }
}

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
    /// The doorways' blockers.
    blockers: Vec<Handle<Node>>,
    /// The middle of each doorway, on the floor in it, from the beam's top under the train.
    doorways: Vec<Vector3<f32>>,
    /// Seconds into its round of the line.
    clock: f32,
}

impl Train {
    /// Puts the train from `model` on the line, at the station with its doors shut.
    pub fn spawn(scene: &mut Scene, model: &ModelResource) -> Self {
        let start = Vector3::new(0.0, BEAM_TOP, along_line(0.0));
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
        let doorways = blockers.iter().map(|at| Vector3::new(at.x.signum() * BLOCKER_OUT, FLOOR, at.z)).collect();
        let blockers: Vec<Handle<Node>> = blockers
            .into_iter()
            .map(|at| {
                let [x, y, z] = BLOCKER_HALF;
                ColliderBuilder::new(
                    BaseBuilder::new().with_local_transform(TransformBuilder::new().with_local_position(at).build()),
                )
                .with_shape(ColliderShape::cuboid(x, y, z))
                .build(graph)
                .to_base()
            })
            .collect();
        for &blocker in &blockers {
            body = body.with_child(blocker);
        }
        let body = RigidBodyBuilder::new(body)
            .with_body_type(RigidBodyType::KinematicVelocityBased)
            .build(graph)
            .to_base();
        graph.update_hierarchical_data();
        graph.link_nodes_keep_global_transform(root, body);
        Log::info(format!(
            "Train: {} cars, {} door leaves, {} doorways, {} stations, round the line in {:.0} s",
            cars.len(),
            leaves.len(),
            blockers.len(),
            STATIONS.len(),
            round()
        ));
        Self { body, cars, leaves, blockers, doorways, clock: 0.0 }
    }

    /// Takes the train out of the scene.
    pub fn remove(&self, graph: &mut Graph) {
        if graph.try_get(self.body).is_ok() {
            graph.remove_node(self.body);
        }
    }

    /// Back at the first station, as a round begins.
    pub fn reset(&mut self, graph: &mut Graph) {
        self.clock = 0.0;
        if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(self.body) {
            body.local_transform_mut().set_position(Vector3::new(0.0, BEAM_TOP, along_line(0.0)));
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

    /// The train as the droids go by it, as of the end of this step.
    pub fn doors(&self) -> Doors {
        Doors {
            at: Vector3::new(0.0, BEAM_TOP, along_line(self.clock)),
            doorways: self.doorways.clone(),
            open: wide_open(self.clock),
        }
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
        // Open, nothing meets the doorways' blockers; told only when that changes, as telling
        // the physics anything about a collider costs it working out its contacts afresh.
        let groups = match open > 0.0 {
            true => InteractionGroups::new(BitMask(0), BitMask(0)),
            false => InteractionGroups::default(),
        };
        for &blocker in &self.blockers {
            if let Ok(collider) = graph.try_get_mut_of_type::<Collider>(blocker) {
                if collider.collision_groups() != groups {
                    collider.set_collision_groups(groups);
                }
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

/// How long it takes to go from `from` to `to` along the line, in seconds: setting off gently and
/// slowing gently to a stop, at [`TOP_SPEED`] half way.
fn crossing(from: f32, to: f32) -> f32 {
    1.5 * (to - from).abs() / TOP_SPEED
}

/// Each station the train stands at in its round, and the one it goes on to from it.
fn legs() -> impl Iterator<Item = (usize, usize)> {
    (0..ROUND.len()).map(|k| (ROUND[k], ROUND[(k + 1) % ROUND.len()]))
}

/// How long its round takes, in seconds.
fn round() -> f32 {
    legs().map(|(from, to)| AT_STATION + crossing(STATIONS[from], STATIONS[to])).sum()
}

/// Where along the line, in the world's z, the train is `time` seconds into its round; and,
/// standing at a station, which, and how long it has stood there, in seconds.
fn on_round(time: f32) -> (f32, Option<(usize, f32)>) {
    let mut time = time.rem_euclid(round());
    for (from, to) in legs() {
        if time < AT_STATION {
            return (STATIONS[from], Some((from, time)));
        }
        time -= AT_STATION;
        let (a, b) = (STATIONS[from], STATIONS[to]);
        let cross = crossing(a, b);
        if time < cross {
            let u = time / cross;
            return (a + (b - a) * u * u * (3.0 - 2.0 * u), None);
        }
        time -= cross;
    }
    (STATIONS[ROUND[0]], Some((ROUND[0], 0.0)))
}

/// Where along the line, in the world's z, the train is `time` seconds into its round.
fn along_line(time: f32) -> f32 {
    on_round(time).0
}

/// How far open the doors are `time` seconds into the train's round, from 0, shut, to 1: open at
/// the stations only.
fn doors_open(time: f32) -> f32 {
    let Some((_, time)) = on_round(time).1 else {
        return 0.0;
    };
    let opening = ((time - DOORS_AFTER) / DOORS_TIME).clamp(0.0, 1.0);
    let shutting = ((time - (AT_STATION - DOORS_BEFORE)) / DOORS_TIME).clamp(0.0, 1.0);
    opening - shutting
}

/// While the doors are wide open `time` seconds into the train's round - from when they are all
/// the way open until they start to shut - at which station, and for how much longer, in seconds.
fn wide_open(time: f32) -> Option<(usize, f32)> {
    let (station, time) = on_round(time).1?;
    let (opened, shutting) = (DOORS_AFTER + DOORS_TIME, AT_STATION - DOORS_BEFORE);
    (opened..shutting).contains(&time).then_some((station, shutting - time))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_stands_at_each_station_from_north_to_south_and_back() {
        let mut stood = Vec::new();
        let mut t = 0.0;
        while t < round() {
            if let (z, Some((station, _))) = on_round(t) {
                assert_eq!(z, STATIONS[station]);
                if stood.last() != Some(&station) {
                    stood.push(station);
                }
            }
            t += 0.1;
        }
        assert_eq!(stood, ROUND);
        assert_eq!(along_line(0.0), STATIONS[0]);
        assert!((along_line(round() - 0.01) - STATIONS[0]).abs() < 0.01, "back where it began");
        assert_eq!(along_line(round() + 1.0), STATIONS[0]);
    }

    #[test]
    fn it_is_never_faster_than_its_top_speed() {
        let step = 0.05;
        let mut t = 0.0;
        while t < round() {
            let speed = (along_line(t + step) - along_line(t)).abs() / step;
            assert!(speed <= TOP_SPEED + 0.01, "{speed} at {t}");
            t += step;
        }
    }

    #[test]
    fn the_doors_open_only_while_it_stands_at_a_station() {
        assert_eq!(doors_open(0.0), 0.0);
        assert_eq!(doors_open(AT_STATION / 2.0), 1.0);
        assert_eq!(doors_open(AT_STATION - 0.5), 0.0);
        let mut t = 0.0;
        while t < 2.0 * round() {
            if doors_open(t) > 0.0 {
                assert!(STATIONS.contains(&along_line(t)), "open at {t} away from the stations");
            }
            t += 0.1;
        }
    }

    #[test]
    fn the_doors_are_wide_open_only_while_they_are_all_the_way_open() {
        let mut t = 0.0;
        let mut at = Vec::new();
        while t < round() {
            if let Some((station, left)) = wide_open(t) {
                assert_eq!(doors_open(t), 1.0, "not all the way open at {t}");
                assert_eq!(doors_open(t + left - 0.01), 1.0, "shutting before {left} s more at {t}");
                assert_eq!(along_line(t), STATIONS[station]);
                if at.last() != Some(&station) {
                    at.push(station);
                }
            }
            t += 0.1;
        }
        assert_eq!(at, ROUND, "open at every station");
        assert!(wide_open(AT_STATION + 1.0).is_none(), "gone");
    }

    #[test]
    fn the_platform_is_beside_the_doorway_out_from_it() {
        let doors = Doors {
            at: Vector3::new(0.0, BEAM_TOP, STATIONS[1]),
            doorways: vec![Vector3::new(BLOCKER_OUT, FLOOR, 2.75), Vector3::new(-BLOCKER_OUT, FLOOR, -8.5)],
            open: Some((1, 10.0)),
        };
        let right = doors.platform(1, 0, 1.0);
        assert!((right.x - (BLOCKER_OUT + 1.0)).abs() < 1e-5 && right.y == 8.0 && right.z == STATIONS[1] + 2.75);
        assert!(doors.platform(1, 1, 1.0).x < -BLOCKER_OUT, "the other side");
        assert_eq!(doors.platform(2, 0, 1.0).z, STATIONS[2] + 2.75, "at another station");
        assert!(!doors.within(right), "out on the platform");
        assert!(doors.within(doors.at + Vector3::new(1.7, FLOOR, 2.75)), "aboard");
        assert_eq!(doors.floor(), 8.25, "a step up");
    }
}
