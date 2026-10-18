//! Roads: where a city's droids keep off, but to cross.
//!
//! A map with streets says where they are in `<model>.roads.json` beside its model (see
//! `data/story/story_urbs.py`): rectangles of road, kerb to kerb, and of crossings over them,
//! each `[x0, z0, x1, z1]` in the world; and the height under which the floor is the road's
//! rather than the pavement's, so that nothing built over a street counts. On the walk grid
//! that comes of it, each cell of road but not of a crossing is dear to walk into for a droid
//! going about its business (see [`ROAD_COST`]): its routes keep to the pavements, and go over
//! the road only at a crossing - or where there is no other way at all. A droid after the
//! player pays no heed to it.

use crate::layout::WalkGrid;
use fyrox::core::{algebra::Vector3, log::Log};
use serde::Deserialize;

/// How much more a step onto the road costs than one along the pavement: enough that a droid
/// walks to the nearest crossing rather than straight over, a block's length out of its way.
pub const ROAD_COST: f32 = 20.0;

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

    /// What a step into cell `(x, row)` costs over a step along the pavement, for a route (see
    /// [`WalkGrid::routes_from_weighted`]).
    pub fn extra(&self, x: usize, row: usize) -> f32 {
        if self.is_road(x, row) {
            ROAD_COST
        } else {
            0.0
        }
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
        // From one pavement to the other, both well along from the crossing.
        let path = grid
            .route_between_weighted((4, 34), (36, 34), f32::INFINITY, |x, z| roads.extra(x, z))
            .expect("a way over");
        let on_road: Vec<_> = path.iter().filter(|&&(x, z)| (16..24).contains(&x) && !(18..22).contains(&z)).collect();
        assert!(on_road.is_empty(), "off the road but at the crossing: {on_road:?}");
    }

    #[test]
    fn floor_over_the_road_is_no_road() {
        let (mut grid, plan) = street();
        grid.set_floor(20, 4, 3.0);
        assert!(!plan.on(&grid, Vector3::zeros()).is_road(20, 4));
    }
}
