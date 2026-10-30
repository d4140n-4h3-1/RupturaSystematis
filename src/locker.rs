//! The precincts' evidence lockers (see [`crate::precinct`]): a bank of steel lockers against the
//! hall's side wall, a model of their own, put where the map marks each with `precinct_<n>_lockers`
//! - its back to the wall, facing out along the marker's +x.
//!
//! They go into the level before its shape is taken, so that they are part of it: walked into,
//! surveyed round, and put out with its lights. One door of each bank is the evidence locker's
//! own, which swings open as the player takes their pistol back out of it, and shuts again when
//! the next is put in. Its collider stays shut, flush with the bank's front.

use crate::level::Marker;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{graph::Graph, node::Node, Scene},
};

/// The lockers' model: made in Blender by ~/Documents/blender/precinct/build_locker.py, which
/// saves locker.blend beside it. It faces its +Z, from the foot of the wall under the middle of
/// its back.
pub const LOCKER_MODEL: &str = "data/locker.glb";
/// The evidence locker's door, its origin on its hinge, on its left as it is faced.
const DOOR: &str = "locker_door";
const MARKER: (&str, &str) = ("precinct_", "_lockers");
/// How far the door swings open, in radians (`OPEN` in the build script), and how long that
/// takes, in seconds.
const OPEN: f32 = 105.0 * std::f32::consts::PI / 180.0;
const SWING: f32 = 0.6;
/// How near the player a bank has to be to be drawn at all, in meters: it stands inside its
/// precinct, out of sight from further off.
const SEEN_WITHIN: f32 = 60.0;

#[derive(Debug, PartialEq)]
struct Locker {
    /// The precinct's number.
    number: u32,
    root: Handle<Node>,
    door: Handle<Node>,
    /// The door's turn shut, as the model has it.
    shut: UnitQuaternion<f32>,
    /// How far open it is, from 0 to 1, and whether it is opening or shutting.
    opened: f32,
    open: bool,
    at: Vector3<f32>,
    seen: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct Lockers {
    lockers: Vec<Locker>,
}

impl Lockers {
    /// Puts a bank of lockers under `root` where each of `markers` marks one.
    pub fn place(scene: &mut Scene, root: Handle<Node>, model: &ModelResource, markers: &[Marker]) -> Self {
        let mut lockers = Vec::new();
        for marker in markers {
            let Some(number) = marker
                .name
                .strip_prefix(MARKER.0)
                .and_then(|rest| rest.strip_suffix(MARKER.1))
                .and_then(|n| n.parse().ok())
            else {
                continue;
            };
            let instance = model.instantiate(scene);
            let graph = &mut scene.graph;
            // The model's +Z along the marker's +x.
            graph[instance]
                .local_transform_mut()
                .set_position(marker.position)
                .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), marker.yaw + std::f32::consts::FRAC_PI_2));
            graph.link_nodes(instance, root);
            let Some((door, _)) = graph.find_by_name(instance, DOOR) else {
                Log::err(format!("Lockers: {LOCKER_MODEL} has no {DOOR}"));
                continue;
            };
            let shut = **graph[door].local_transform().rotation();
            lockers.push(Locker { number, root: instance, door, shut, opened: 0.0, open: false, at: marker.position, seen: true });
        }
        if !lockers.is_empty() {
            Log::info(format!("Lockers: {} banks", lockers.len()));
        }
        Self { lockers }
    }

    /// Opens or shuts precinct `number`'s evidence locker, swinging there.
    pub fn set_open(&mut self, number: u32, open: bool) {
        for locker in self.lockers.iter_mut().filter(|l| l.number == number) {
            locker.open = open;
        }
    }

    /// Shuts every evidence locker at once.
    pub fn reset(&mut self, graph: &mut Graph) {
        for locker in &mut self.lockers {
            locker.open = false;
            locker.opened = 0.0;
            locker.turn(graph);
        }
    }

    /// Swings each door on towards open or shut, over `dt`.
    pub fn update(&mut self, graph: &mut Graph, dt: f32) {
        for locker in &mut self.lockers {
            let to = if locker.open { 1.0 } else { 0.0 };
            if locker.opened == to {
                continue;
            }
            let step = dt / SWING;
            locker.opened = if locker.open { (locker.opened + step).min(1.0) } else { (locker.opened - step).max(0.0) };
            locker.turn(graph);
        }
    }

    /// Draws only the banks near `viewer`.
    pub fn show_near(&mut self, graph: &mut Graph, viewer: Vector3<f32>) {
        for locker in &mut self.lockers {
            let seen = (locker.at - viewer).norm() < SEEN_WITHIN;
            if seen != locker.seen {
                locker.seen = seen;
                graph[locker.root].set_visibility(seen);
            }
        }
    }
}

impl Locker {
    fn turn(&self, graph: &mut Graph) {
        // Eased, so the door does not start or stop with a jolt.
        let eased = self.opened * self.opened * (3.0 - 2.0 * self.opened);
        let swing = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -OPEN * eased);
        if graph.is_valid_handle(self.door) {
            graph[self.door].local_transform_mut().set_rotation(self.shut * swing);
        }
    }
}
