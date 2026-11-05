//! Traffic: droids on hover bikes going about the cities' streets in the story, and stop lights at
//! the junctions of the main streets.
//!
//! The traffic's AI - the streets and junctions, the lights' cycles, and where each bike goes - is
//! [`hydroxus_ai::navigation::traffic`]'s; this puts the bikes and the stop lights in the scene
//! and moves them there.
//!
//! The streets are those of each city's road plan (see [`crate::roads`]): straight strips 16 m
//! wide, kerb to kerb, a junction wherever two cross. Traffic keeps to the right, two lanes each
//! way, [`LANE`](hydroxus_ai::navigation::traffic::LANE) wide - the lines between them painted by `data/story/story_urbs.py`'s `streets`.
//! What can be driven is found by looking (see [`Network::survey`]): a lane from one junction to the
//! next with anything in it - a light cycle parked at the kerb, a station's stairs - is not driven,
//! nor a turn through a junction that would meet something; nor a street with no road under it.
//!
//! A bike goes from junction to junction along its lane, and at each turns left or right or goes on
//! across, as it planned on the way there - in the right-hand lane to turn right and the left-hand
//! to turn left - round a curve into its lane on the way out. It keeps its distance from the bike
//! in front, and stops for anyone in the road before it: the player, or a droid on foot. It is
//! solid: it cannot be walked through.
//!
//! The junctions where two main streets meet ([`main_street`](hydroxus_ai::navigation::traffic::main_street): every other street) have stop lights, one on each corner
//! for the traffic coming in on its right, its two heads up over the street, one over each lane
//! (`data/stop_light.glb`, from ~/Documents/blender/street/stop_light) - if there is room for them all
//! (see [`Network::survey`]). Each junction lets one street's traffic through at a time: green, then
//! amber, then a moment of red both ways ([`CYCLE`]); a bike stops at the line on red, and on amber
//! if it can. The crossing signal on each corner's mast, facing across the street its light holds,
//! shows the walking figure while that street's traffic has red. At a junction with no lights a bike
//! goes in only while nothing is crossing it the other way.

use crate::race::Bike;
pub use hydroxus_ai::navigation::traffic::{Light, Network, CYCLE};
use hydroxus_ai::navigation::traffic::{signal_place, way};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{
        base::BaseBuilder,
        collider::{ColliderBuilder, ColliderShape},
        graph::Graph,
        node::Node,
        rigidbody::{RigidBodyBuilder, RigidBodyType},
        Scene,
    },
};

/// The stop light's model: its foot at the origin, its heads facing +z (towards the traffic it
/// holds), its arm reaching out along -x over the street.
pub const STOP_LIGHT_MODEL: &str = "data/stop_light.glb";
/// Its lenses, each a mesh of its own: red, amber, green, and the crossing's wait and walk.
const LENSES: [&str; 5] = ["stop_light_red", "stop_light_amber", "stop_light_green", "walk_stop", "walk_go"];

/// How many bikes go about the streets, at most.
pub const BIKES: usize = 64;
/// How far from the player a bike or a stop light is still drawn; how many stop lights there are
/// in the scene at most, put up on the lit corners nearest the player; and how often they are
/// moved round to follow them, in seconds.
const SHOWN: f32 = 280.0;
const POOL: usize = 200;
const REPOST: f32 = 0.5;
/// How high over the road a bike hovers, and the box it is to whatever runs into it.
const HOVER: f32 = 0.15;
const BODY: [f32; 3] = [0.45, 0.75, 1.3];

/// The traffic in the scene: the bikes where its riders are, and the stop lights at the
/// junctions (the traffic itself, [`hydroxus_ai::navigation::traffic::Traffic`]).
#[derive(Debug, Default, PartialEq)]
pub struct Traffic {
    pub drive: hydroxus_ai::navigation::traffic::Traffic,
    /// Each rider's bike in the scene, as [`Traffic::spawn`] put it there: its body and the bike.
    bikes: Vec<Option<(Handle<Node>, Bike)>>,
    /// Every lit corner: its junction, and the way the traffic its light holds comes in from.
    corners: Vec<(usize, usize)>,
    /// The stop lights in the scene, each up on one of the corners nearest the player, or none:
    /// its root and its lenses (see [`LENSES`]), its corner, and what it shows as of the last step.
    signals: Vec<(Handle<Node>, [Handle<Node>; 5], Option<usize>, Option<Light>)>,
    /// The stop lights' model, and how long until they are next moved round.
    signal_model: Option<ModelResource>,
    repost: f32,
}

impl Traffic {
    /// Traffic on `network`, `count` bikes put down at random on its lanes - `pick` giving numbers
    /// from 0 to 1 - away from `player`.
    pub fn new(network: Network, count: usize, player: Vector3<f32>, pick: &mut dyn FnMut() -> f32) -> Self {
        Self { drive: hydroxus_ai::navigation::traffic::Traffic::new(network, count, player, pick), ..Default::default() }
    }

    /// The bikes and the stop lights put into `scene`: each bike from `bike`, ridden by a droid
    /// from `riders` in turn, and a stop light from `signal` on each corner of each lit junction.
    pub fn spawn(&mut self, scene: &mut Scene, bike: &ModelResource, riders: &[ModelResource], signal: Option<&ModelResource>) {
        self.bikes = Vec::with_capacity(self.drive.riders.len());
        for n in 0..self.drive.riders.len() {
            let droid = (!riders.is_empty()).then(|| &riders[n % riders.len()]);
            let built = crate::race::build_bike(scene, bike, droid);
            let graph = &mut scene.graph;
            let collider = ColliderBuilder::new(BaseBuilder::new())
                .with_shape(ColliderShape::cuboid(BODY[0], BODY[1], BODY[2]))
                .build(graph);
            graph[collider].local_transform_mut().set_position(Vector3::new(0.0, BODY[1] + 0.2, 0.0));
            let body = RigidBodyBuilder::new(BaseBuilder::new().with_child(collider))
                .with_body_type(RigidBodyType::KinematicPositionBased)
                .build(graph)
                .to_base();
            graph.link_nodes(built.root, body);
            self.bikes.push(Some((body, built)));
        }
        self.corners = (0..self.drive.network.junctions.len())
            .filter(|&j| self.drive.network.junctions[j].lit)
            .flat_map(|j| (0..4).map(move |w| (j, w)))
            .filter(|&(j, w)| self.drive.network.junctions[j].out[w].is_some())
            .collect();
        if signal.is_none() {
            Log::err(format!("Traffic: could not load {STOP_LIGHT_MODEL}; the lights work, unseen"));
        }
        self.signal_model = signal.cloned();
        self.repost = 0.0;
        Log::info(format!(
            "Traffic: {} bikes; {} of {} junctions lit, {} stop lights, up to {POOL} of them shown at a time",
            self.drive.riders.len(),
            self.drive.network.junctions.iter().filter(|j| j.lit).count(),
            self.drive.network.junctions.len(),
            self.corners.len()
        ));
    }

    /// Takes the bikes and the stop lights out of the scene.
    pub fn remove(&mut self, graph: &mut Graph) {
        for bike in &mut self.bikes {
            if let Some((body, _)) = bike.take() {
                graph.remove_node(body);
            }
        }
        for (root, _, _, _) in self.signals.drain(..) {
            graph.remove_node(root);
        }
    }

    /// Runs the traffic on `dt` seconds, the bikes stopping for `people` on foot.
    pub fn step(&mut self, dt: f32, people: &[Vector3<f32>]) {
        self.drive.step(dt, people);
    }

    /// The stop lights moved round onto the lit corners nearest `near`, as many as there are of
    /// them: each staying on its corner while that is still among the nearest, the rest put up on
    /// those that have come near, and more made while there are fewer than [`POOL`].
    fn post_signals(&mut self, scene: &mut Scene, near: Vector3<f32>) {
        let Some(model) = self.signal_model.clone() else {
            return;
        };
        let mut wanted: Vec<(f32, usize)> = self
            .corners
            .iter()
            .enumerate()
            .map(|(c, &(j, w))| ((signal_place(&self.drive.network.tuning, self.drive.network.junctions[j].at, w).0 - near).norm(), c))
            .filter(|&(d, _)| d < SHOWN)
            .collect();
        wanted.sort_by(|a, b| a.0.total_cmp(&b.0));
        wanted.truncate(POOL);
        let wanted: std::collections::HashSet<usize> = wanted.into_iter().map(|(_, c)| c).collect();
        // Off the corners no longer near.
        for signal in &mut self.signals {
            if signal.2.is_some_and(|c| !wanted.contains(&c)) {
                signal.2 = None;
                scene.graph[signal.0].set_visibility(false);
            }
        }
        let held: std::collections::HashSet<usize> = self.signals.iter().filter_map(|s| s.2).collect();
        for corner in wanted.into_iter().filter(|c| !held.contains(c)) {
            let free = match self.signals.iter().position(|s| s.2.is_none()) {
                Some(free) => free,
                None if self.signals.len() < POOL => {
                    let root = model.instantiate(scene);
                    let lenses = LENSES.map(|name| scene.graph.find_by_name(root, name).map_or(Handle::NONE, |(node, _)| node));
                    self.signals.push((root, lenses, None, None));
                    self.signals.len() - 1
                }
                None => break,
            };
            let (j, w) = self.corners[corner];
            let at = self.drive.network.junctions[j].at;
            let (mast, _) = signal_place(&self.drive.network.tuning, at, w);
            // Its heads (+z) face the traffic coming in, back along the street (way `w`).
            let back = way(w);
            let root = self.signals[free].0;
            let node = &mut scene.graph[root];
            node.set_visibility(true);
            node.local_transform_mut()
                .set_position(Vector3::new(mast.x, at.y + 0.12, mast.z))
                .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), back.x.atan2(back.z)));
            self.signals[free].2 = Some(corner);
            self.signals[free].3 = None;
        }
    }

    /// Puts the bikes where they are, and the stop lights up on the corners nearest `near` - the
    /// player - showing what they show; hiding whatever is far off.
    pub fn place(&mut self, scene: &mut Scene, near: Vector3<f32>, dt: f32) {
        self.repost -= dt;
        if self.repost <= 0.0 {
            self.repost = REPOST;
            self.post_signals(scene, near);
        }
        let graph = &mut scene.graph;
        for (rider, bike) in self.drive.riders.iter().zip(&self.bikes) {
            let Some((body, _)) = bike else {
                continue;
            };
            let shown = (rider.at - near).norm() < SHOWN;
            let node = &mut graph[*body];
            if node.visibility() != shown {
                node.set_visibility(shown);
            }
            let bob = (self.drive.time * 2.6 + rider.cruise).sin() * 0.02;
            let rotation = UnitQuaternion::face_towards(&rider.ahead, &Vector3::y())
                * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -rider.lean);
            graph[*body]
                .local_transform_mut()
                .set_position(rider.at + Vector3::y() * (HOVER + bob))
                .set_rotation(rotation);
        }
        for s in 0..self.signals.len() {
            let (_, lenses, corner, was) = self.signals[s];
            let Some(corner) = corner else {
                continue;
            };
            let (j, w) = self.corners[corner];
            // This light holds the traffic coming in from way `w`: going the other way.
            let now = self.drive.light_for(j, (w + 2) % 4);
            if now == was {
                continue;
            }
            self.signals[s].3 = now;
            let lit = [
                now == Some(Light::Red),
                now == Some(Light::Amber),
                now == Some(Light::Green),
                now != Some(Light::Red),
                now == Some(Light::Red),
            ];
            for (lens, on) in lenses.into_iter().zip(lit) {
                if let Ok(node) = graph.try_get_mut(lens) {
                    node.set_visibility(on);
                }
            }
        }
    }
}
