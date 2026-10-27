//! The escape tunnels, under a city (see `data/story/story_urbs.py`): a warren hung out of sight in
//! the void well under it, reached only by hatches, as Skyrim's interiors are by their doors.
//!
//! A map marks each hatch in the city with an empty, `hatch_<name>` - in each precinct's tunnel,
//! each bank's strongroom, and a few manholes in the street - and its other end in the tunnels
//! with another, `hatch_<name>_below`. E at either takes the player to the other, the screen going
//! dark and coming up again on the far side; no droid follows, there being no way between them on
//! foot. In the tunnels, `den_<k>` is where a gang keeps, and `fence` the counter where a fence
//! will wipe the player's record clean, for a price; and `cache_<k>` a cache of credits, its box
//! the mesh `cache_box_<k>`, taken by walking up to it.

use crate::{credits::Credits, level::Marker};
use fyrox::{
    core::{algebra::Vector3, log::Log, pool::Handle},
    graph::SceneGraph,
    scene::{graph::Graph, node::Node},
};

/// How near a hatch, the fence or a cache the player has to be to use it, along the ground and up
/// or down, in meters.
const REACH: (f32, f32) = (1.4, 1.8);
/// How long going through a hatch takes, in seconds: the screen goes dark over the first half, and
/// comes up again, on the far side, over the second.
pub const PASSING: f32 = 0.8;
/// What the fence asks to wipe a record clean, in credits: so much, and so much more a count.
const FENCE: u64 = 60;
const FENCE_PER_COUNT: u64 = 40;

/// A hatch and its other end.
#[derive(Debug, Clone, PartialEq)]
pub struct Hatch {
    pub name: String,
    pub above: Vector3<f32>,
    pub below: Vector3<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cache {
    pub at: Vector3<f32>,
    box_: Handle<Node>,
    pub taken: bool,
}

/// A trip through a hatch, under way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Passing {
    /// Where to, and which way to face there.
    pub to: Vector3<f32>,
    pub yaw: f32,
    /// How long it has gone on, in seconds, and whether the player has been put there yet.
    pub time: f32,
    pub moved: bool,
}

impl Passing {
    /// How dark the screen is, from 0 to 1.
    pub fn dark(&self) -> f32 {
        let half = PASSING / 2.0;
        (1.0 - (self.time - half).abs() / half).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Tunnels {
    pub hatches: Vec<Hatch>,
    pub caches: Vec<Cache>,
    pub fence: Option<Vector3<f32>>,
    pub den: Vec<Vector3<f32>>,
}

/// Whether `feet` are near enough `at` to use what is there.
fn near(feet: Vector3<f32>, at: Vector3<f32>) -> bool {
    (feet - at).xz().norm() < REACH.0 && (feet.y - at.y).abs() < REACH.1
}

impl Tunnels {
    /// The tunnels, from `markers`, and the caches' boxes from under `root`: each hatch that has
    /// both its ends.
    pub fn find(graph: &Graph, root: Handle<Node>, markers: &[Marker]) -> Self {
        let at = |name: &str| markers.iter().find(|m| m.name == name).map(|m| m.position);
        let hatches: Vec<Hatch> = markers
            .iter()
            .filter_map(|m| {
                let name = m.name.strip_prefix("hatch_")?.strip_suffix("_below")?;
                Some(Hatch { name: name.to_string(), above: at(&format!("hatch_{name}"))?, below: m.position })
            })
            .collect();
        let caches = markers
            .iter()
            .filter_map(|m| {
                let k = m.name.strip_prefix("cache_")?;
                let box_ = graph.find_by_name(root, &format!("cache_box_{k}")).map(|(h, _)| h).unwrap_or_default();
                Some(Cache { at: m.position, box_, taken: false })
            })
            .collect::<Vec<_>>();
        let mut den: Vec<(String, Vector3<f32>)> =
            markers.iter().filter(|m| m.name.starts_with("den_")).map(|m| (m.name.clone(), m.position)).collect();
        den.sort_by(|a, b| a.0.cmp(&b.0));
        let tunnels = Self { hatches, caches, fence: at("fence"), den: den.into_iter().map(|(_, at)| at).collect() };
        if !tunnels.hatches.is_empty() {
            Log::info(format!(
                "Escape tunnels: {} hatches, {} caches, {} of the den",
                tunnels.hatches.len(),
                tunnels.caches.len(),
                tunnels.den.len()
            ));
        }
        tunnels
    }

    /// Every cache full again, for a new round.
    pub fn reset(&mut self, graph: &mut Graph) {
        for cache in &mut self.caches {
            cache.taken = false;
            if graph.is_valid_handle(cache.box_) {
                graph[cache.box_].set_visibility(true);
            }
        }
    }

    /// The hatch at the player's `feet`, if they are at one: where it goes, and whether that is
    /// down into the tunnels.
    pub fn hatch_at(&self, feet: Vector3<f32>) -> Option<(&Hatch, bool)> {
        self.hatches.iter().find_map(|hatch| {
            if near(feet, hatch.above) {
                Some((hatch, true))
            } else if near(feet, hatch.below) {
                Some((hatch, false))
            } else {
                None
            }
        })
    }

    /// Where the player comes out through `hatch`, going `down` or up, facing `yaw` as they went
    /// in: a little off its other end, so as not to stand on it and go straight back - down, out
    /// of its dead end toward the den, facing that way; up, on the way they were facing.
    pub fn arrival(&self, hatch: &Hatch, down: bool, yaw: f32) -> (Vector3<f32>, f32) {
        let (to, yaw) = match down {
            true => {
                let middle = self.den.iter().fold(Vector3::zeros(), |sum, p| sum + p) / self.den.len().max(1) as f32;
                let way = (middle - hatch.below).xz();
                (hatch.below, if way.norm() > 1.0e-3 { way.x.atan2(way.y) } else { yaw })
            }
            false => (hatch.above, yaw),
        };
        (to + Vector3::new(yaw.sin(), 0.0, yaw.cos()) * 1.6 + Vector3::new(0.0, 0.1, 0.0), yaw)
    }

    /// Whether the player's `feet` are at the fence's counter.
    pub fn at_fence(&self, feet: Vector3<f32>) -> bool {
        self.fence.is_some_and(|fence| near(feet, fence))
    }

    /// Takes any cache the player's `feet` are at: its box gone, and what was in it, by `below`.
    pub fn take(&mut self, graph: &mut Graph, feet: Vector3<f32>, below: impl FnMut(usize) -> usize) -> Option<Credits> {
        let cache = self.caches.iter_mut().find(|cache| !cache.taken && near(feet, cache.at))?;
        cache.taken = true;
        if graph.is_valid_handle(cache.box_) {
            graph[cache.box_].set_visibility(false);
        }
        Some(Credits::store(below))
    }
}

impl Tunnels {
    /// Whether `at` is down in the tunnels, which hang far below where falling off the city counts
    /// as falling into the void: within a few meters of what their markers cover.
    pub fn holds(&self, at: Vector3<f32>) -> bool {
        let points = self.hatches.iter().map(|h| h.below).chain(self.den.iter().copied()).chain(self.caches.iter().map(|c| c.at));
        let (lo, hi) = points.fold((Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN)), |(lo, hi), p| (lo.inf(&p), hi.sup(&p)));
        lo.x <= hi.x
            && at.x > lo.x - 8.0
            && at.x < hi.x + 8.0
            && at.z > lo.z - 8.0
            && at.z < hi.z + 8.0
            && at.y > lo.y - 3.0
            && at.y < hi.y + 8.0
    }
}

/// What the fence asks to wipe clean a record of `counts`.
pub fn fence_price(counts: u32) -> u64 {
    FENCE + FENCE_PER_COUNT * counts as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(name: &str, x: f32, y: f32) -> Marker {
        Marker { name: name.to_string(), position: Vector3::new(x, y, 0.0), yaw: 0.0 }
    }

    #[test]
    fn a_hatch_goes_between_its_ends_and_only_one_with_both_counts() {
        let markers = [
            marker("hatch_bank_1", 10.0, 0.1),
            marker("hatch_bank_1_below", 0.0, -120.0),
            marker("hatch_street_1_below", 5.0, -120.0),
        ];
        let tunnels = Tunnels::find(&Graph::new(), Handle::NONE, &markers);
        assert_eq!(tunnels.hatches.len(), 1, "street_1 has no end above");
        let (hatch, down) = tunnels.hatch_at(Vector3::new(10.3, 0.1, 0.2)).unwrap();
        assert!(down && hatch.name == "bank_1");
        assert!(!tunnels.hatch_at(Vector3::new(0.0, -120.0, 0.5)).unwrap().1);
        assert!(tunnels.holds(Vector3::new(1.0, -119.0, 0.0)), "down in them");
        assert!(!tunnels.holds(Vector3::new(1.0, -60.0, 0.0)), "falling past them");
    }

    #[test]
    fn the_screen_goes_dark_and_comes_up_again() {
        let at = |time| Passing { to: Vector3::zeros(), yaw: 0.0, time, moved: false }.dark();
        assert_eq!(at(0.0), 0.0);
        assert!((at(PASSING / 2.0) - 1.0).abs() < 1.0e-6);
        assert_eq!(at(PASSING), 0.0);
        assert!(fence_price(3) > fence_price(0));
    }
}
