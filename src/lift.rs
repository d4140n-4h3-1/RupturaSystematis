//! Lifts: up the office towers of a city, a car that stops at every floor and the roof, up and
//! back down again, and a gate before its shaft at each floor that stands open only while the car
//! is there.
//!
//! A map marks one with a mesh named `lift_<id>`, its car, its origin the middle of its deck where
//! it starts, at the bottom; empties named `lift_<id>_<k>` where it stops at floor k, from 1 up;
//! and a mesh for each floor's gate, `lift_<id>_gate_<k>`, from 0, the bottom, up. The car is
//! taken out of the level's static collider and put on a body of its own, moved by its velocity,
//! which the player reads to be carried up and down with it, as on a ferry (see
//! [`crate::ferry`]); the gates are taken out too, each onto a body of its own, switched in and out
//! of the way by its collision groups, as the engine never moves a collider once it is made.
//!
//! The droids keep off the bottom of the shafts, where the car comes down; and the floors up above
//! are no part of their grid, which only stairs lead up to.

use crate::layout::WalkGrid;
use fyrox::{
    core::{algebra::Vector3, log::Log, pool::Handle},
    graph::SceneGraph,
    scene::{
        base::BaseBuilder,
        collider::{BitMask, Collider, ColliderBuilder, ColliderShape, GeometrySource, InteractionGroups},
        graph::Graph,
        node::Node,
        rigidbody::{RigidBody, RigidBodyBuilder, RigidBodyType},
        transform::TransformBuilder,
    },
};

/// What a map names a lift's car, its stops and its gates after.
const PREFIX: &str = "lift_";
const GATE: &str = "gate_";
/// How fast a car goes at its fastest, half way between two floors, in meters per second.
const TOP_SPEED: f32 = 3.5;
/// How long a car stands at each floor, in seconds; and how long after it gets there its gate
/// opens, and how long before it goes its gate shuts.
const WAIT: f32 = 4.0;
const OPENS: f32 = 0.3;
const SHUTS: f32 = 0.6;
/// How near a gate someone has to be, along the floor and up from it, to be standing in its
/// doorway, which keeps it open and the car waiting.
const DOORWAY: f32 = 0.9;
const DOORWAY_HIGH: f32 = 2.0;
/// How far from the player a lift is still drawn, in meters: its car and its gates are pieces of
/// their own, and there are many.
const SEEN_WITHIN: f32 = 150.0;
/// How far apart the lifts' rounds are started, in seconds, so that they do not all go together.
const STAGGER: f32 = 7.3;

/// The level's lifts.
#[derive(Debug, Default, PartialEq)]
pub struct Lifts {
    lifts: Vec<Lift>,
}

#[derive(Debug, PartialEq)]
struct Lift {
    car: Handle<Node>,
    timetable: Timetable,
    /// Each floor's gate, from the bottom up: its body, its collider, and whether it stands open.
    gates: Vec<Gate>,
    /// Seconds into its round, and where it starts its round, to start again from.
    clock: f32,
    start: f32,
    /// Whether it is drawn: not far off.
    seen: bool,
}

#[derive(Debug, PartialEq)]
struct Gate {
    body: Handle<Node>,
    collider: Handle<Node>,
    /// Where the middle of its doorway is, on the floor.
    at: Vector3<f32>,
    open: bool,
}

/// Where a lift stops, from the bottom up, and when it is where.
#[derive(Debug, Clone, PartialEq)]
struct Timetable(Vec<Vector3<f32>>);

impl Timetable {
    /// The floors it calls at in a round, in order: up to the top, and back down to the one over
    /// the bottom, before it starts again from the bottom.
    fn calls(&self) -> Vec<usize> {
        let top = self.0.len() - 1;
        (0..=top).chain((1..top).rev()).collect()
    }

    /// How long it takes from one floor to another, setting off gently and slowing gently to a
    /// stop, at [`TOP_SPEED`] half way.
    fn between(&self, from: usize, to: usize) -> f32 {
        1.5 * self.0[from].metric_distance(&self.0[to]) / TOP_SPEED
    }

    /// How long a round takes, in seconds.
    fn round(&self) -> f32 {
        let calls = self.calls();
        calls.iter().enumerate().map(|(i, &floor)| WAIT + self.between(floor, calls[(i + 1) % calls.len()])).sum()
    }

    /// Where the car is `time` seconds into its round, which repeats; and the floor it stands at,
    /// if it is standing at one, and for how long it has been.
    fn at(&self, time: f32) -> (Vector3<f32>, Option<(usize, f32)>) {
        let calls = self.calls();
        let mut time = time.rem_euclid(self.round());
        for (i, &floor) in calls.iter().enumerate() {
            if time < WAIT {
                return (self.0[floor], Some((floor, time)));
            }
            time -= WAIT;
            let next = calls[(i + 1) % calls.len()];
            let between = self.between(floor, next);
            if time < between {
                let u = time / between.max(f32::EPSILON);
                return (self.0[floor].lerp(&self.0[next], u * u * (3.0 - 2.0 * u)), None);
            }
            time -= between;
        }
        (self.0[0], Some((0, 0.0)))
    }

    /// The floor whose gate stands open `time` seconds into the round, if any: while the car is
    /// standing there, but for a moment after it gets there and before it goes.
    fn open(&self, time: f32) -> Option<usize> {
        match self.at(time).1 {
            Some((floor, stood)) if (OPENS..WAIT - SHUTS).contains(&stood) => Some(floor),
            _ => None,
        }
    }
}

/// The `<id>` and the rest after [`PREFIX`] in a lift's name: `lift_u3_gate_2` is ("u3",
/// "gate_2").
fn named(name: &str) -> Option<(&str, &str)> {
    let rest = name.strip_prefix(PREFIX)?;
    Some(rest.split_once('_').unwrap_or((rest, "")))
}

/// Shut, in everyone's way; or open, in no one's.
fn groups(open: bool) -> InteractionGroups {
    if open {
        InteractionGroups::new(BitMask(0), BitMask(0))
    } else {
        InteractionGroups::default()
    }
}

/// A body of its own for `mesh`, of `kind`, where the mesh is, with a collider of its shape; the
/// mesh put on it, which stands straight, as the map makes it, where its origin is. The body and
/// its collider. (Put on it as it is rather than keeping where it is across the world, which would
/// want the whole scene's transforms brought up to date for every one of them.)
fn body_for(graph: &mut Graph, mesh: Handle<Node>, name: &str, kind: RigidBodyType) -> (Handle<Node>, Handle<Node>) {
    let at = graph[mesh].global_position();
    let collider = ColliderBuilder::new(BaseBuilder::new())
        .with_shape(ColliderShape::trimesh(vec![GeometrySource(mesh)]))
        .build(graph)
        .to_base();
    let body = RigidBodyBuilder::new(
        BaseBuilder::new()
            .with_name(name)
            .with_local_transform(TransformBuilder::new().with_local_position(at).build())
            .with_child(collider),
    )
    .with_body_type(kind)
    .build(graph)
    .to_base();
    graph.link_nodes(mesh, body);
    graph[mesh]
        .local_transform_mut()
        .set_position(Vector3::zeros())
        .set_rotation(Default::default())
        .set_scale(Vector3::repeat(1.0));
    (body, collider)
}

impl Lifts {
    /// Takes the lifts a model marks under `root` out of it: each car onto a body of its own,
    /// where it starts, and each gate onto one of its own, shut. From here on they are no part of
    /// the level's shape. The bodies are returned too, to be removed with the level.
    pub fn claim(graph: &mut Graph, root: Handle<Node>) -> (Self, Vec<Handle<Node>>) {
        let named_nodes: Vec<(Handle<Node>, String, bool)> = graph
            .traverse_handle_iter(root)
            .filter(|&h| graph[h].name().starts_with(PREFIX))
            .map(|h| (h, graph[h].name().to_string(), graph[h].is_mesh()))
            .collect();
        let mut lifts = Vec::new();
        let mut bodies = Vec::new();
        for (car_mesh, name, _) in named_nodes.iter().filter(|(_, name, mesh)| *mesh && named(name).is_some_and(|(_, rest)| rest.is_empty())) {
            let Some((id, _)) = named(name) else { continue };
            let mut stops: Vec<(usize, Vector3<f32>)> = vec![(0, graph[*car_mesh].global_position())];
            let mut gate_meshes: Vec<(usize, Handle<Node>)> = Vec::new();
            for (h, other, mesh) in &named_nodes {
                let Some((other_id, rest)) = named(other) else { continue };
                if other_id != id || rest.is_empty() {
                    continue;
                }
                match (rest.strip_prefix(GATE), mesh) {
                    (Some(k), true) => gate_meshes.extend(k.parse().ok().map(|k| (k, *h))),
                    (None, false) => stops.extend(rest.parse().ok().map(|k| (k, graph[*h].global_position()))),
                    _ => (),
                }
            }
            stops.sort_by_key(|&(k, _)| k);
            gate_meshes.sort_by_key(|&(k, _)| k);
            let floors = stops.len();
            if floors < 2 || gate_meshes.len() != floors || stops.iter().enumerate().any(|(i, &(k, _))| i != k) {
                Log::warn(format!("Maze: lift {name} has {floors} stops and {} gates, not one at every floor", gate_meshes.len()));
                continue;
            }
            let timetable = Timetable(stops.into_iter().map(|(_, at)| at).collect());
            let (car, _) = body_for(graph, *car_mesh, name, RigidBodyType::KinematicVelocityBased);
            bodies.push(car);
            let gates = gate_meshes
                .into_iter()
                .map(|(_, mesh)| {
                    let at = graph[mesh].global_position();
                    let gate_name = graph[mesh].name().to_string();
                    let (body, collider) = body_for(graph, mesh, &gate_name, RigidBodyType::Static);
                    bodies.push(body);
                    Gate { body, collider, at, open: false }
                })
                .collect();
            let start = (lifts.len() as f32 * STAGGER).rem_euclid(timetable.round());
            lifts.push(Lift { car, timetable, gates, clock: start, start, seen: true });
        }
        if !lifts.is_empty() {
            Log::info(format!(
                "Maze: {} lifts, {} floors in all",
                lifts.len(),
                lifts.iter().map(|lift| lift.gates.len()).sum::<usize>()
            ));
        }
        (Self { lifts }, bodies)
    }

    /// Takes the bottom of every shaft, where the cars come down, and a cell round it, out of
    /// `grid`, whose corner is at `origin`.
    pub fn keep_out(&self, grid: &mut WalkGrid, origin: Vector3<f32>) {
        const REACH: f32 = 1.9;
        for lift in &self.lifts {
            let bottom = lift.timetable.0[0];
            let step = grid.cell_size * 0.5;
            let mut x = -REACH;
            while x <= REACH {
                let mut z = -REACH;
                while z <= REACH {
                    if let Some((cx, cz)) = grid.cell_at(origin, bottom + Vector3::new(x, 0.1, z)) {
                        grid.set(cx, cz, false);
                    }
                    z += step;
                }
                x += step;
            }
        }
    }

    /// Puts every car back where its round starts, standing still, and every gate as it stands
    /// then, as a round begins.
    pub fn reset(&mut self, graph: &mut Graph) {
        for lift in &mut self.lifts {
            lift.clock = lift.start;
            let (at, _) = lift.timetable.at(lift.clock);
            if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(lift.car) {
                body.local_transform_mut().set_position(at);
                body.set_lin_vel(Vector3::zeros());
            }
            lift.set_gates(graph);
        }
    }

    /// Moves the cars on by `dt`, each given the velocity that takes it to where it should be by
    /// the end of this step, and opens and shuts the gates as they come and go. A car waits, its
    /// gate open, while the player stands in its doorway, with their feet at `feet`.
    pub fn update(&mut self, graph: &mut Graph, dt: f32, feet: Vector3<f32>) {
        if dt <= 0.0 {
            return;
        }
        for lift in &mut self.lifts {
            let held = lift.timetable.open(lift.clock).is_some_and(|floor| {
                let gate = &lift.gates[floor];
                (feet - gate.at).xz().norm() < DOORWAY && (feet.y - gate.at.y).abs() < DOORWAY_HIGH
            }) && lift.timetable.open(lift.clock + dt).is_none();
            if !held {
                lift.clock += dt;
            }
            let (to, _) = lift.timetable.at(lift.clock);
            if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(lift.car) {
                let here = **body.local_transform().position();
                body.set_lin_vel((to - here) / dt);
            }
            lift.set_gates(graph);
        }
    }

    /// Draws only the lifts near `viewer`, and their gates while they are shut.
    pub fn show_near(&mut self, graph: &mut Graph, viewer: Vector3<f32>) {
        for lift in &mut self.lifts {
            let seen = (lift.timetable.0[0] - viewer).xz().norm() < SEEN_WITHIN;
            if seen == lift.seen {
                continue;
            }
            lift.seen = seen;
            graph[lift.car].set_visibility(seen);
            for gate in &lift.gates {
                graph[gate.body].set_visibility(seen && !gate.open);
            }
        }
    }
}

impl Lift {
    /// Each gate open or shut as the car's place in its round has it: switched, and shown or
    /// hidden, only as that changes.
    fn set_gates(&mut self, graph: &mut Graph) {
        let open = self.timetable.open(self.clock);
        for (floor, gate) in self.gates.iter_mut().enumerate() {
            let now = open == Some(floor);
            if now == gate.open {
                continue;
            }
            gate.open = now;
            if let Ok(collider) = graph.try_get_mut_of_type::<Collider>(gate.collider) {
                collider.set_collision_groups(groups(now));
            }
            graph[gate.body].set_visibility(self.seen && !now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timetable() -> Timetable {
        Timetable((0..4).map(|k| Vector3::new(0.0, 4.0 * k as f32, 0.0)).collect())
    }

    #[test]
    fn it_calls_at_every_floor_up_and_back_down() {
        assert_eq!(timetable().calls(), vec![0, 1, 2, 3, 2, 1]);
        let between = timetable().between(0, 1);
        assert!((between - 1.5 * 4.0 / TOP_SPEED).abs() < 1e-5);
        assert!((timetable().round() - 6.0 * (WAIT + between)).abs() < 1e-3);
    }

    #[test]
    fn it_stands_at_each_floor_with_its_gate_open_then_goes_on() {
        let table = timetable();
        let between = table.between(0, 1);
        assert_eq!(table.at(1.0), (Vector3::zeros(), Some((0, 1.0))));
        assert_eq!(table.open(1.0), Some(0));
        assert_eq!(table.open(0.1), None, "not open the moment it gets there");
        assert_eq!(table.open(WAIT - 0.1), None, "shut before it goes");
        let (half, standing) = table.at(WAIT + between / 2.0);
        assert!(standing.is_none() && (half.y - 2.0).abs() < 1e-4, "half way up: {half:?}");
        assert_eq!(table.open(WAIT + between / 2.0), None);
        let (at, standing) = table.at(WAIT + between + 1.0);
        assert_eq!(standing.map(|(floor, _)| floor), Some(1));
        assert!((at.y - 4.0).abs() < 1e-4);
        // Down from the top: past it at 3, the next floor it stands at is 2.
        let to_top = 3.0 * (WAIT + between);
        assert_eq!(table.at(to_top + 1.0).1.map(|(floor, _)| floor), Some(3));
        assert_eq!(table.at(to_top + WAIT + between + 1.0).1.map(|(floor, _)| floor), Some(2));
        assert_eq!(table.at(table.round() + 1.0).1.map(|(floor, _)| floor), Some(0), "round again");
    }

    #[test]
    fn it_is_never_faster_than_its_top_speed() {
        let table = timetable();
        let dt = 1.0 / 60.0;
        let fastest = (0..(table.round() / dt) as usize)
            .map(|i| table.at((i + 1) as f32 * dt).0.metric_distance(&table.at(i as f32 * dt).0) / dt)
            .fold(0.0, f32::max);
        assert!(fastest <= TOP_SPEED * 1.01, "{fastest} m/s");
    }

    #[test]
    fn a_lift_is_known_by_its_names() {
        assert_eq!(named("lift_u3"), Some(("u3", "")));
        assert_eq!(named("lift_u3_2"), Some(("u3", "2")));
        assert_eq!(named("lift_c10_gate_0"), Some(("c10", "gate_0")));
        assert_eq!(named("ferry_1"), None);
    }
}
