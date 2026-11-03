//! Where the player starts and where the exit goes.
//!
//! The maze comes from a model, so nothing says which parts of it are corridors. The game
//! samples the level on a grid instead: a cell is walkable when a ray dropped into it lands on the
//! floor rather than on top of a wall. Paths between cells then come from a breadth-first search,
//! which is what places the exit at the far end of the maze by walking distance, not by straight
//! line distance through the walls. The navmesh made from the grid (`hydroxus-ai`'s, as the grid
//! is) is what is kept of it, and what this goes by, and the inhabitants find their way about by.

pub use hydroxus_ai::navigation::grid::{Rng, WalkGrid};
use hydroxus_ai::navigation::navmesh::NavMesh;

/// Picks a start and an exit for a round on `mesh`: the start is chosen by `pick` among the
/// cells of the largest connected area, and the exit is as far from it as the maze allows.
pub fn plan_round(
    mesh: &NavMesh,
    mut pick: impl FnMut(usize) -> usize,
) -> Option<((usize, usize), (usize, usize))> {
    // Stray samples (a ledge, a gap outside the walls) form small islands of their own; the
    // round is played in the biggest connected area.
    let mut best: Vec<(usize, usize)> = Vec::new();
    let width = mesh.width();
    let mut seen = vec![false; width * mesh.depth() * mesh.storeys()];
    let cells: Vec<(usize, usize)> = mesh.walkable_cells().collect();
    for (x, z) in cells {
        if seen[z * width + x] {
            continue;
        }
        let area: Vec<(usize, usize)> = mesh
            .distances_from((x, z))
            .iter()
            .enumerate()
            .filter(|(_, d)| d.is_some())
            .map(|(i, _)| (i % width, i / width))
            .collect();
        for &(ax, az) in &area {
            seen[az * width + ax] = true;
        }
        if area.len() > best.len() {
            best = area;
        }
    }
    if best.len() < 2 {
        return None;
    }

    // A start in a random spot can land in the middle of the maze, which halves the walk;
    // starting from one end of the longest route keeps every round a full crossing.
    let seed = best[pick(best.len())];
    let (start, _) = mesh.farthest_from(seed)?;
    let (exit, _) = mesh.farthest_from(start)?;
    Some((start, exit))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a grid from rows of `#` (wall) and `.` (floor).
    fn mesh(grid: &WalkGrid) -> NavMesh {
        NavMesh::build(grid, hydroxus_ai::nalgebra::Vector3::zeros(), |_, _| 0)
    }

    fn grid(rows: &[&str]) -> WalkGrid {
        let mut grid = WalkGrid::new(rows[0].len(), rows.len(), 1.0);
        for (z, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                grid.set(x, z, c == '.');
            }
        }
        grid
    }

    #[test]
    fn a_round_spans_the_longest_route() {
        let maze = grid(&[
            ".....", //
            "####.", //
            ".....", //
        ]);
        let (start, exit) = plan_round(&mesh(&maze), |n| n / 2).unwrap();
        let mut ends = [start, exit];
        ends.sort();
        assert_eq!(ends, [(0, 0), (0, 2)]);
    }

    #[test]
    fn a_round_ignores_small_islands() {
        let maze = grid(&[
            ".#....", //
            "##....", //
        ]);
        for i in 0..9 {
            let (start, exit) = plan_round(&mesh(&maze), |n| i % n).unwrap();
            assert!(start.0 >= 2 && exit.0 >= 2, "{start:?} {exit:?}");
        }
    }

    #[test]
    fn nothing_to_play_on_gives_no_round() {
        assert_eq!(plan_round(&mesh(&grid(&["#.#"])), |_| 0), None);
    }
}
