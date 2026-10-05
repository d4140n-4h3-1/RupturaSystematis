//! Ferries: platforms that carry whoever stands on them between two places over the void, waiting
//! a while at each end.
//!
//! A map marks one with a mesh named `ferry_...`, its origin where its deck is to be, and empties
//! named after it - `<name>_1`, `<name>_2` and on - at the places it goes through, in order, the
//! last its other end. The ferry is taken out of the level's static collider and put on a body of
//! its own, moved by its velocity, which the player reads to be carried along with it (see
//! `Player::drive`). The survey of the level counts only the level's own collider, so the droids
//! never take a ferry: they keep to the ground that stays where it is.

use fyrox::{
    core::{algebra::Vector3, log::Log, pool::Handle},
    graph::SceneGraph,
    scene::{
        base::BaseBuilder,
        collider::{ColliderBuilder, ColliderShape, GeometrySource},
        graph::Graph,
        node::Node,
        rigidbody::{RigidBody, RigidBodyBuilder, RigidBodyType},
        transform::TransformBuilder,
    },
};

/// What a map names a ferry's mesh, and its stops after it.
const PREFIX: &str = "ferry_";
/// How fast a ferry goes at its fastest, half way across, in meters per second.
const TOP_SPEED: f32 = 4.0;
/// How long a ferry waits at each end, in seconds.
const WAIT: f32 = 5.0;

/// The level's ferries, and how far into their round trip they are.
#[derive(Debug, Default, PartialEq)]
pub struct Ferries {
    ferries: Vec<Ferry>,
    /// Seconds since the round began: every ferry keeps the same time.
    clock: f32,
}

#[derive(Debug, PartialEq)]
struct Ferry {
    body: Handle<Node>,
    route: Route,
}

/// The places a ferry goes through, from one end to the other.
#[derive(Debug, Clone, PartialEq)]
struct Route(Vec<Vector3<f32>>);

impl Route {
    fn length(&self) -> f32 {
        self.0.windows(2).map(|leg| leg[0].metric_distance(&leg[1])).sum()
    }

    /// The point `distance` along the route from its start.
    fn along(&self, mut distance: f32) -> Vector3<f32> {
        for leg in self.0.windows(2) {
            let length = leg[0].metric_distance(&leg[1]);
            if distance <= length {
                return leg[0].lerp(&leg[1], distance / length.max(f32::EPSILON));
            }
            distance -= length;
        }
        *self.0.last().unwrap()
    }

    /// How long it takes to cross from end to end, in seconds: setting off gently and slowing
    /// gently to a stop, at [`TOP_SPEED`] half way.
    fn crossing(&self) -> f32 {
        1.5 * self.length() / TOP_SPEED
    }

    /// Where the ferry is `time` seconds into its round trip, which repeats: waiting at the start,
    /// across, waiting at the far end, and back.
    fn at(&self, time: f32) -> Vector3<f32> {
        let crossing = self.crossing();
        let leg = WAIT + crossing;
        let time = time.rem_euclid(2.0 * leg);
        let (time, back) = if time < leg { (time, false) } else { (time - leg, true) };
        let u = ((time - WAIT) / crossing).clamp(0.0, 1.0);
        let eased = u * u * (3.0 - 2.0 * u);
        let share = if back { 1.0 - eased } else { eased };
        self.along(share * self.length())
    }
}

impl Ferries {
    /// Takes the ferries a model marks under `root` out of it, each onto a body of its own, where
    /// it starts: from here on it is no part of the level's shape. The bodies are returned too,
    /// to be removed with the level.
    pub fn claim(graph: &mut Graph, root: Handle<Node>) -> (Self, Vec<Handle<Node>>) {
        let named: Vec<(Handle<Node>, String, bool)> = graph
            .traverse_handle_iter(root)
            .filter(|&h| graph[h].name().starts_with(PREFIX))
            .map(|h| (h, graph[h].name().to_string(), graph[h].is_mesh()))
            .collect();
        let mut ferries = Vec::new();
        for (mesh, name, _) in named.iter().filter(|(_, _, mesh)| *mesh) {
            let mut stops: Vec<(u32, Vector3<f32>)> = named
                .iter()
                .filter(|(_, _, mesh)| !mesh)
                .filter_map(|(h, stop, _)| {
                    let n = stop.strip_prefix(name.as_str())?.strip_prefix('_')?.parse().ok()?;
                    Some((n, graph[*h].global_position()))
                })
                .collect();
            stops.sort_by_key(|&(n, _)| n);
            let start = graph[*mesh].global_position();
            let route = Route(std::iter::once(start).chain(stops.into_iter().map(|(_, at)| at)).collect());
            if route.0.len() < 2 {
                Log::warn(format!("Maze: ferry {name} has nowhere to go"));
                continue;
            }
            let collider = ColliderBuilder::new(BaseBuilder::new())
                .with_shape(ColliderShape::trimesh(vec![GeometrySource(*mesh)]))
                .build(graph);
            let body = RigidBodyBuilder::new(
                BaseBuilder::new()
                    .with_name(name.as_str())
                    .with_local_transform(TransformBuilder::new().with_local_position(start).build())
                    .with_child(collider),
            )
            .with_body_type(RigidBodyType::KinematicVelocityBased)
            .build(graph)
            .to_base();
            graph.update_hierarchical_data();
            graph.link_nodes_keep_global_transform(*mesh, body);
            Log::info(format!(
                "Maze: ferry {name}, {} stops, {:.0} m across in {:.0} s",
                route.0.len(),
                route.length(),
                route.crossing()
            ));
            ferries.push(Ferry { body, route });
        }
        let bodies = ferries.iter().map(|ferry| ferry.body).collect();
        (Self { ferries, clock: 0.0 }, bodies)
    }

    /// Puts every ferry back where it starts, standing still, as a round begins.
    pub fn reset(&mut self, graph: &mut Graph) {
        self.clock = 0.0;
        for ferry in &self.ferries {
            if let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(ferry.body) {
                body.local_transform_mut().set_position(ferry.route.0[0]);
                body.set_lin_vel(Vector3::zeros());
            }
        }
    }

    /// Moves the ferries on by `dt`: each is given the velocity that takes it from where it is to
    /// where it should be by the end of this step, so it never drifts off its route.
    pub fn update(&mut self, graph: &mut Graph, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        self.clock += dt;
        for ferry in &self.ferries {
            let Ok(body) = graph.try_get_mut_of_type::<RigidBody>(ferry.body) else {
                continue;
            };
            let here = **body.local_transform().position();
            body.set_lin_vel((ferry.route.at(self.clock) - here) / dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route() -> Route {
        // Up 3 m, then 4 m along: 7 m in all.
        Route(vec![Vector3::zeros(), Vector3::new(0.0, 3.0, 0.0), Vector3::new(4.0, 3.0, 0.0)])
    }

    #[test]
    fn it_waits_at_each_end() {
        let route = route();
        let crossing = route.crossing();
        assert_eq!(route.at(0.0), Vector3::zeros());
        assert_eq!(route.at(WAIT), Vector3::zeros(), "still waiting");
        let there = Vector3::new(4.0, 3.0, 0.0);
        assert!(route.at(WAIT + crossing).metric_distance(&there) < 1e-4);
        assert!(route.at(2.0 * WAIT + crossing).metric_distance(&there) < 1e-4, "waiting there");
        let round = 2.0 * (WAIT + crossing);
        assert!(route.at(round).norm() < 1e-4, "back again");
    }

    #[test]
    fn it_goes_through_every_stop() {
        let route = route();
        let crossing = route.crossing();
        // Easing is symmetric, so the middle of the time is the middle of the way: 3.5 m along,
        // half a meter past the corner.
        let middle = route.at(WAIT + crossing / 2.0);
        assert!(middle.metric_distance(&Vector3::new(0.5, 3.0, 0.0)) < 1e-4, "{middle:?}");
    }

    #[test]
    fn it_is_never_faster_than_its_top_speed() {
        let route = route();
        let dt = 1.0 / 60.0;
        let round = 2.0 * (WAIT + route.crossing());
        let fastest = (0..(round / dt) as usize)
            .map(|i| route.at((i + 1) as f32 * dt).metric_distance(&route.at(i as f32 * dt)) / dt)
            .fold(0.0, f32::max);
        assert!(fastest <= TOP_SPEED * 1.01, "{fastest} m/s");
        assert!(fastest > TOP_SPEED * 0.95, "{fastest} m/s");
    }
}
