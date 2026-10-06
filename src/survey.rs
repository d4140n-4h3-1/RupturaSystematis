//! Finding the walkable ground of a level that is already in the scene.
//!
//! This is the scene side of [`layout`](crate::layout): it samples the level's colliders on a
//! grid of cells, and turns cells back into places in the world.
//!
//! A cell's floor is the highest in it with headroom over it, under a ceiling, and room around:
//! the ground, or a stair, or a floor up above. A cell holds one floor, so ground under a floor
//! up above is left out. A level out in the open, with no ceiling over it, has the sky for its
//! ceiling: a cell's floor there is the first surface down. Of those, the walkable cells are the ground and whatever can be climbed
//! to from it a stair's step at a time ([`hydroxus_ai::grid::MAX_CLIMB`]) - not the tops of
//! crates or walls, which no stairs lead up to.

use crate::layout::WalkGrid;
use hydroxus_ai::grid::MAX_CLIMB;
use std::collections::VecDeque;
use fyrox::{
    core::{
        algebra::{Point3, Vector3},
        log::Log,
        pool::Handle,
    },
    scene::{
        collider::Collider,
        graph::{physics::RayCastOptions, Graph},
        node::Node,
    },
};

/// Spacing of the walkability samples, in meters.
pub(crate) const CELL_SIZE: f32 = 0.5;

/// How much room a floor needs over it to be stood on.
const HEADROOM: f32 = 2.0;
/// How high a floor can be and still be the ground, which the climbing starts from.
const GROUND: f32 = 0.5;
/// The most floors over one another the grid keeps: a four-floor building with its roof, and
/// some to spare.
const MOST_STOREYS: usize = 6;

/// Samples the maze for walkable ground, returning the grid and the world position of its corner.
/// `maze` is the level's collider, which must exist already, and `ignore` a mesh that is not part
/// of the maze. `open_sky` is for a level with no ceiling over it.
pub fn survey(
    graph: &Graph,
    maze: Handle<Collider>,
    ignore: Handle<Node>,
    open_sky: bool,
) -> Option<(WalkGrid, Vector3<f32>)> {
    let started = crate::platform::nanos_now();
    // The footprint of every mesh in the scene is the footprint of the maze.
    let mut min = Vector3::repeat(f32::MAX);
    let mut max = Vector3::repeat(f32::MIN);
    for node in graph.linear_iter() {
        if node.is_mesh() && node.handle() != ignore {
            let aabb = node.world_bounding_box();
            min = min.inf(&aabb.min);
            max = max.sup(&aabb.max);
        }
    }
    if min.x > max.x {
        return None;
    }
    let width = ((max.x - min.x) / CELL_SIZE).ceil() as usize;
    let depth = ((max.z - min.z) / CELL_SIZE).ceil() as usize;
    let origin = Vector3::new(min.x, 0.0, min.z);
    // Each cell's floors, if it has any, bottom up: looked for from over everything down to under
    // it all, and never less far down than a meter under the ground.
    let bottom = (min.y - 1.0).min(-1.0);
    let mut floors: Vec<Vec<f32>> = Vec::with_capacity(width * depth);
    for z in 0..depth {
        for x in 0..width {
            let spot = cell_center(origin, x, z);
            floors.push(floors_at(graph, maze, spot, (max.y + 1.0, bottom), open_sky));
        }
    }
    let storeys = floors.iter().map(Vec::len).max().unwrap_or(1).clamp(1, MOST_STOREYS);
    let mut grid = WalkGrid::with_storeys(width, depth, storeys, CELL_SIZE);
    for (i, cell) in floors.iter().enumerate() {
        for (storey, &floor) in cell.iter().enumerate().take(storeys) {
            let (x, row) = grid.on_storey((i % width, i / width), storey);
            grid.set_floor(x, row, floor);
        }
    }
    // Out from the ground, onto every floor a step from one already walkable: on whichever storey
    // it is, up stairs to the floors up above.
    // The climbing starts from the ground: where the first floor down - the one there is always
    // open air over - is low enough to be ground. Floors further down, if they are any, are found
    // by walking to them.
    let mut queue = VecDeque::new();
    for (i, cell) in floors.iter().enumerate() {
        if let Some((storey, &floor)) = cell.iter().enumerate().take(storeys).last() {
            if floor <= GROUND {
                queue.push_back(grid.on_storey((i % width, i / width), storey));
            }
        }
    }
    for &(x, row) in &queue {
        grid.set(x, row, true);
    }
    let mut walkable = queue.len();
    let ground = walkable;
    let mut tried = vec![false; width * grid.rows()];
    while let Some(cell) = queue.pop_front() {
        let (x, z) = grid.plan(cell);
        let here = grid.floor(cell.0, cell.1);
        // The first floor down at each spot - the last, bottom up - has open air over it; only a
        // floor under another could be the inside of something solid, and needs a clear way in.
        let first = |plan: (usize, usize), storey: usize| storey + 1 == floors[plan.1 * width + plan.0].len().min(storeys);
        let here_first = first((x, z), grid.storey(cell));
        let neighbours = [(x.wrapping_sub(1), z), (x + 1, z), (x, z.wrapping_sub(1)), (x, z + 1)];
        for (nx, nz) in neighbours {
            if nx >= width || nz >= depth {
                continue;
            }
            for (storey, &floor) in floors[nz * width + nx].iter().enumerate().take(storeys) {
                let (cx, crow) = grid.on_storey((nx, nz), storey);
                if !grid.is_walkable(cx, crow)
                    && (floor - here).abs() <= MAX_CLIMB
                    && !tried[crow * width + cx]
                    && ((here_first && first((nx, nz), storey))
                        || passage(graph, grid.on_floor(origin, cell), grid.on_floor(origin, (cx, crow))))
                {
                    // Room round it, asked once.
                    tried[crow * width + cx] = true;
                    if !roomy(graph, cell_center(origin, nx, nz), floor) {
                        continue;
                    }
                    grid.set(cx, crow, true);
                    walkable += 1;
                    queue.push_back((cx, crow));
                }
            }
        }
    }
    let upper = grid.walkable_cells().filter(|&cell| grid.storey(cell) > 0).count();
    Log::info(format!(
        "Maze: {width}x{depth} cells over {:.1}x{:.1} m, {walkable} walkable, {} of them up off the ground, \
         {upper} on {} storeys over others, in {:.1} s",
        max.x - min.x,
        max.z - min.z,
        walkable - ground,
        storeys - 1,
        crate::platform::nanos_now().saturating_sub(started) as f32 / 1.0e9
    ));
    Some((grid, origin))
}

/// The middle of cell `(x, z)` of the survey's grid, whose corner is at `origin`.
pub fn cell_center(origin: Vector3<f32>, x: usize, z: usize) -> Vector3<f32> {
    hydroxus_ai::grid::cell_center(origin, CELL_SIZE, x, z)
}

/// The cell of `grid` that `position` is in, if it is on the grid at all.
pub fn cell_at(
    grid: &WalkGrid,
    origin: Vector3<f32>,
    position: Vector3<f32>,
) -> Option<(usize, usize)> {
    grid.cell_at(origin, position)
}

/// The walkable cell nearest to a point, which is where the player can stand to reach it.
pub fn nearest_walkable(
    grid: &WalkGrid,
    origin: Vector3<f32>,
    point: Vector3<f32>,
) -> Option<(usize, usize)> {
    grid.nearest_walkable(origin, point)
}

/// The direction from `cell` towards the walkable floor around it, reached by walking.
pub fn open_direction(grid: &WalkGrid, origin: Vector3<f32>, cell: (usize, usize)) -> Vector3<f32> {
    const REACH: u32 = 8;
    let center = cell_center(origin, cell.0, cell.1);
    let distances = grid.distances_from(cell);
    let mut sum = Vector3::zeros();
    for (i, distance) in distances.iter().enumerate() {
        if distance.is_some_and(|d| d > 0 && d <= REACH) {
            sum += cell_center(origin, i % grid.width, i / grid.width) - center;
        }
    }
    if sum.norm_squared() > 0.0 {
        sum
    } else {
        Vector3::z()
    }
}

/// The walkable map as text, with the start and the exit marked: the quickest way to see what the
/// survey made of a new model. Every other row is left out, so that the map is about as tall as
/// it is wide in a terminal.
pub fn draw_map(grid: &WalkGrid, start: (usize, usize), exit: (usize, usize)) -> String {
    let mut text = String::new();
    for z in (0..grid.depth).step_by(2) {
        for x in 0..grid.width {
            let c = if (x, z) == start || (x, z + 1) == start {
                'S'
            } else if (x, z) == exit || (x, z + 1) == exit {
                'E'
            } else if grid.is_walkable(x, z) {
                '.'
            } else {
                '#'
            };
            text.push(c);
        }
        text.push('\n');
    }
    text
}

/// The floors at `spot`, bottom up: every surface with headroom over it (see [`roomy`] for room
/// round it, which is asked later), found
/// by dropping a ray from `top`, over everything, down to `bottom`, under everything, through all
/// of the maze, surface by surface - a cast meets only the first surface of a collider. The first
/// floor down is the first surface with [`HEADROOM`] or more of open air above it - under
/// `open_sky`, the sky is the ceiling - as it always was. Under that, a floor under a floor up
/// above - the ground floor of a house under its first floor - is one too, but only where the
/// air over it is open, not the inside of something solid (see [`floors_of`]).
fn floors_at(
    graph: &Graph,
    maze: Handle<Collider>,
    spot: Vector3<f32>,
    (top, bottom): (f32, f32),
    open_sky: bool,
) -> Vec<f32> {
    /// Below a surface, to cast on from past it; and the most surfaces a cell is looked through.
    const PAST: f32 = 0.01;
    const MOST: usize = 32;
    let mut heights = Vec::new();
    let mut from = top;
    while heights.len() < MOST {
        let below = Vector3::new(spot.x, from, spot.z);
        // The maze's surfaces only: the floor under everything is solid, and a ray starting in
        // it meets it where it starts - a centimeter at a time through it, before.
        let Some(hit) = maze_hit(graph, maze, below, -Vector3::y(), from - bottom) else {
            break;
        };
        heights.push(hit.y);
        from = hit.y - PAST;
    }
    // The first floor down is the topmost with room round it, as it always was; those under it
    // are asked for room only if the survey walks to them.
    let mut floors = floors_of(&heights, open_sky);
    let first = floors.iter().position(|&floor| roomy(graph, spot, floor)).unwrap_or(floors.len());
    floors.drain(..first);
    floors.reverse();
    floors
}

/// Whether there is room round the middle of a cell at `spot` to stand on `floor`: nothing
/// within reach at chest height. Asked only of floors the survey walks to - most are never.
fn roomy(graph: &Graph, spot: Vector3<f32>, floor: f32) -> bool {
    let chest = Vector3::new(spot.x, floor + 1.0, spot.z);
    [Vector3::x(), -Vector3::x(), Vector3::z(), -Vector3::z()]
        .iter()
        .all(|dir| first_hit(graph, chest, *dir, 0.45).is_none())
}

/// Whether someone can walk from the middle of the cell at `from` to the middle of its neighbour
/// at `to`, each on its floor: nothing in the way at chest height. A floor inside something solid
/// - the ground under stairs built solid down to it - has the solid's sides all round it.
fn passage(graph: &Graph, from: Vector3<f32>, to: Vector3<f32>) -> bool {
    let (from, to) = (from + Vector3::new(0.0, 1.0, 0.0), to + Vector3::new(0.0, 1.0, 0.0));
    let way = to - from;
    let length = way.norm();
    length < 1.0e-4 || first_hit(graph, from, way / length, length).is_none()
}

/// The floors among `heights`, the surfaces a ray dropped from over everything meets, top down:
/// the first with [`HEADROOM`] under the surface over it, as [`floor_of`] has it, and every one
/// under that with as much headroom. Those further down may be the inside of something solid -
/// stairs or a raised floor built solid down to the ground meet the ground in one surface - which
/// the survey never walks into (see [`survey`]).
fn floors_of(heights: &[f32], open_sky: bool) -> Vec<f32> {
    let Some(first) = floor_of(heights, open_sky) else {
        return Vec::new();
    };
    let Some(at) = heights.iter().position(|&h| h == first) else {
        return vec![first];
    };
    let mut floors = vec![first];
    floors.extend(
        heights[at..]
            .windows(2)
            .filter(|pair| pair[0] - pair[1] >= HEADROOM)
            .map(|pair| pair[1]),
    );
    floors
}

/// The floor among `heights`, the surfaces a ray dropped from over everything meets, top down:
/// the first with [`HEADROOM`] under the surface over it. That is never the first surface down,
/// the top of the roof, which has nothing over it - unless the level is under `open_sky`, with no
/// roof, when the sky is the ceiling and the first surface down is the floor. Only the gap under
/// a ceiling is sure to be open air: the gaps further down may be the inside of something solid.
fn floor_of(heights: &[f32], open_sky: bool) -> Option<f32> {
    let sky = open_sky.then_some(f32::INFINITY);
    let heights: Vec<f32> = sky.into_iter().chain(heights.iter().copied()).collect();
    heights
        .windows(2)
        .find(|pair| pair[0] - pair[1] >= HEADROOM)
        .map(|pair| pair[1])
}


/// Where a ray from `origin` along `direction` first meets the maze, `maze`, within `length`, if
/// it does: past anything else.
fn maze_hit(
    graph: &Graph,
    maze: Handle<Collider>,
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    length: f32,
) -> Option<Vector3<f32>> {
    HITS.with(|hits| {
        let mut hits = hits.borrow_mut();
        graph.physics.cast_ray(
            RayCastOptions {
                ray_origin: Point3::from(origin),
                ray_direction: direction,
                max_len: length,
                groups: Default::default(),
                sort_results: true,
            },
            &mut *hits,
        );
        hits.iter().find(|hit| hit.collider == maze).map(|hit| hit.position.coords)
    })
}

thread_local! {
    /// One list of hits for every ray of the survey: a new one for each of its rays was much of
    /// what the survey cost.
    static HITS: std::cell::RefCell<Vec<fyrox::scene::graph::physics::Intersection>> = Default::default();
}

fn first_hit(
    graph: &Graph,
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    length: f32,
) -> Option<(Vector3<f32>, Handle<Collider>)> {
    HITS.with(|hits| {
        let mut hits = hits.borrow_mut();
        graph.physics.cast_ray(
            RayCastOptions {
                ray_origin: Point3::from(origin),
                ray_direction: direction,
                max_len: length,
                groups: Default::default(),
                sort_results: true,
            },
            &mut *hits,
        );
        hits.first().map(|hit| (hit.position.coords, hit.collider))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_floor_under_a_roof_is_the_one_with_headroom() {
        // roof top, roof underside, floor, floor underside
        let hall = [7.3, 7.0, 0.0, -0.2];
        assert_eq!(floor_of(&hall, false), Some(0.0));
    }

    #[test]
    fn in_the_open_the_floor_is_the_first_surface_down() {
        let platform = [0.0, -1.0];
        assert_eq!(floor_of(&platform, false), None, "with no roof, no floor");
        assert_eq!(floor_of(&platform, true), Some(0.0));
        let hub = [1.5, 0.0, -1.0];
        assert_eq!(floor_of(&hub, true), Some(1.5));
    }

    #[test]
    fn in_the_open_something_solid_is_floored_on_top() {
        // a tower standing over the void, and a platform over the rock hanging under it: the
        // gaps below their tops are the inside of them, and the void under them
        assert_eq!(floor_of(&[3.0, -1.0], true), Some(3.0));
        assert_eq!(floor_of(&[0.0, -1.0, -15.0], true), Some(0.0));
    }

    #[test]
    fn floors_over_one_another_are_each_a_floor() {
        // A house of two floors under a roof, in the open: roof, first floor, ground.
        let house = [7.3, 7.0, 3.5, 3.2, 0.0, -1.0];
        assert_eq!(floors_of(&house, true), vec![7.3, 3.5, 0.0]);
        // Under a ceiling, the roof is no floor.
        assert_eq!(floors_of(&house, false), vec![3.5, 0.0]);
    }

    #[test]
    fn under_something_solid_is_a_floor_only_to_look_at() {
        // Stairs 2.5 m high built down to the ground, which they meet in one surface: the ground
        // inside them has the headroom, and is only kept from being walked by their sides.
        let stairs = [2.5, 0.0, -1.0];
        assert_eq!(floors_of(&stairs, true), vec![2.5, 0.0]);
        // A crate is too low to stand under.
        let crate_ = [1.2, 0.0, -1.0];
        assert_eq!(floors_of(&crate_, true), vec![1.2]);
    }

    #[test]
    fn nothing_under_the_void_is_a_floor() {
        assert_eq!(floor_of(&[], true), None);
    }
}
