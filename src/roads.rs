//! Roads: where a city's droids keep off, but to cross.
//!
//! A map with streets says where they are in `<model>.roads.json` beside its model (see
//! `data/story/story_urbs.py`): rectangles of road, kerb to kerb, and of crossings over them,
//! each `[x0, z0, x1, z1]` in the world; and the height under which the floor is the road's
//! rather than the pavement's, so that nothing built over a street counts. On the walk grid
//! that comes of it, each cell of road but not of a crossing is dear to walk into for a droid
//! going about its business (see `hydroxus_ai::navigation::wander::ROAD_COST`): its routes keep to the pavements, and go over
//! the road only at a crossing - or where there is no other way at all. A droid after the
//! player pays no heed to it.

use crate::layout::WalkGrid;
use fyrox::core::{algebra::Vector3, log::Log};
use serde::Deserialize;


/// A rectangle on the ground, `[x0, z0, x1, z1]`.
type Rect = [f32; 4];

/// The roads and crossings a model's file has, before they are put on a grid.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RoadPlan {
    /// The floor under this height is the road's, in meters.
    level: f32,
    roads: Vec<Rect>,
    crossings: Vec<Rect>,
}

impl RoadPlan {
    /// The roads beside the model at `model_path`, if it has any.
    pub fn load(model_path: &str) -> Option<Self> {
        let stem = model_path.rsplit_once('.').map_or(model_path, |(stem, _)| stem);
        let path = format!("{stem}.roads.json");
        if !std::path::Path::new(&path).is_file() {
            return None;
        }
        let read = crate::platform::read_to_string(&path)
            .and_then(|text| serde_json::from_str::<Self>(&text).map_err(|e| e.to_string()));
        match read {
            Ok(plan) => {
                Log::info(format!(
                    "Roads: {} roads and {} crossings in {path}",
                    plan.roads.len(),
                    plan.crossings.len()
                ));
                Some(plan)
            }
            Err(error) => {
                Log::err(format!("Roads: could not read {path}: {error}"));
                None
            }
        }
    }

    /// These roads and `other`'s together: those of a model played alongside, which keeps to the
    /// height this one's road is under.
    pub fn with(mut self, other: Option<Self>) -> Self {
        if let Some(other) = other {
            self.roads.extend(other.roads);
            self.crossings.extend(other.crossings);
        }
        self
    }

    /// Every street's rectangle, kerb to kerb, `[x0, z0, x1, z1]`: for the traffic to drive (see
    /// [`crate::traffic`]).
    pub fn roads(&self) -> &[[f32; 4]] {
        &self.roads
    }

    /// Which cells of `grid`, whose corner is at `origin`, are road to keep off.
    pub fn on(&self, grid: &WalkGrid, origin: Vector3<f32>) -> Roads {
        let mut road = vec![false; grid.width * grid.rows()];
        // Each rectangle a cell's middle is in, on the plan, on every storey.
        let mut mark = |rect: &Rect, is: bool| {
            let first = |from: f32, corner: f32| ((from - corner) / grid.cell_size - 0.5).ceil().max(0.0) as usize;
            let (x0, z0) = (first(rect[0], origin.x), first(rect[1], origin.z));
            let (x1, z1) = (first(rect[2], origin.x).min(grid.width), first(rect[3], origin.z).min(grid.depth));
            for z in z0..z1 {
                for x in x0..x1 {
                    for (x, row) in grid.cells_over((x, z)) {
                        if !is || grid.floor(x, row) < self.level {
                            road[row * grid.width + x] = is;
                        }
                    }
                }
            }
        };
        for rect in &self.roads {
            mark(rect, true);
        }
        for rect in &self.crossings {
            mark(rect, false);
        }
        let roads = Roads { width: grid.width, road };
        Log::info(format!("Roads: {} cells of road to keep off", roads.road.iter().filter(|&&r| r).count()));
        roads
    }
}

/// Which cells of a walk grid are road, to keep off but to cross.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Roads {
    width: usize,
    road: Vec<bool>,
}

impl Roads {
    /// Whether cell `(x, row)` is road, and not a crossing.
    pub fn is_road(&self, x: usize, row: usize) -> bool {
        self.road.get(row * self.width + x).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A street 4 m wide down the middle of ground 20 m square, half a meter a cell, with a
    /// crossing over it 2 m wide at z 10.
    fn street() -> (WalkGrid, RoadPlan) {
        let mut grid = WalkGrid::new(40, 40, 0.5);
        for x in 0..40 {
            for z in 0..40 {
                grid.set(x, z, true);
                // The pavements stand a kerb's height over the road.
                let road = (16..24).contains(&x);
                grid.set_floor(x, z, if road { 0.0 } else { 0.12 });
            }
        }
        let plan = RoadPlan {
            level: 0.06,
            roads: vec![[8.0, 0.0, 12.0, 20.0]],
            crossings: vec![[8.0, 9.0, 12.0, 11.0]],
        };
        (grid, plan)
    }

    #[test]
    fn the_road_is_kept_off_but_its_crossing_and_the_pavements_are_not() {
        let (grid, plan) = street();
        let roads = plan.on(&grid, Vector3::zeros());
        assert!(roads.is_road(20, 4), "the road");
        assert!(!roads.is_road(20, 20), "the crossing");
        assert!(!roads.is_road(4, 4) && !roads.is_road(30, 4), "the pavements");
    }

    #[test]
    fn a_route_over_the_street_goes_by_the_crossing() {
        let (grid, plan) = street();
        let roads = plan.on(&grid, Vector3::zeros());
        use hydroxus_ai::navigation::{navmesh::NavMesh, walk::route_to_weighted, wander::ROAD_COST};
        let mesh = NavMesh::build(&grid, Vector3::zeros(), |x, z| u8::from(roads.is_road(x, z)));
        let dear = |n| if mesh.polygon(n).area == 1 { ROAD_COST } else { 0.0 };
        // From one pavement to the other, both well along from the crossing.
        let (from, to) = (Vector3::new(2.25, 0.12, 17.25), Vector3::new(18.25, 0.12, 17.25));
        let route = route_to_weighted(&mesh, from, to, f32::INFINITY, dear);
        assert!(!route.is_empty(), "a way over");
        // Every step of the way, as the crow flies between its corners, off the road.
        let mut at = from;
        for &next in route.iter().rev() {
            for k in 0..=20 {
                let p = at + (next - at) * (k as f32 / 20.0);
                assert!(!mesh.poly_at(p).is_some_and(|n| mesh.polygon(n).area == 1), "on the road at {p:?}: {route:?}");
            }
            at = next;
        }
    }

    #[test]
    fn floor_over_the_road_is_no_road() {
        let (mut grid, plan) = street();
        grid.set_floor(20, 4, 3.0);
        assert!(!plan.on(&grid, Vector3::zeros()).is_road(20, 4));
    }
}
