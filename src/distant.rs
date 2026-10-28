//! A city far off: in place of its full detail, a simplified copy of each block of it.
//!
//! A city's model gathers its geometry into tiles, a mesh for each of its collections - named
//! `<collection>_<x>_<y>` - and the renderer pays by the mesh and the material, not by the
//! triangle. Far off, the model's simplified blocks stand in for them instead: a mesh for each
//! collection of each block of [`BLOCK`] meters, named `Far<collection>_<x>_<y>`, all the tiles'
//! pieces too small to make out at that distance left out (see `far_off` in
//! `data/story/story_urbs.py`). Its lit windows, gathered by block apart from the rest, are shown
//! near and far alike.
//!
//! A block is shown in full within [`NEAR`] of the player, and its simplified copy further out;
//! the lamps' glass in it goes with its full detail. The simplified copies are taken out of the
//! level, so that they are no part of its shape: the full detail stays in it, shown or not.

use crate::fixtures;
use fyrox::{
    core::{algebra::Vector3, log::Log, pool::Handle},
    graph::SceneGraph,
    scene::{base::BaseBuilder, graph::Graph, mesh::Mesh, node::Node, pivot::PivotBuilder},
};
use std::collections::BTreeMap;

/// How far across a block is, and a tile, in meters, as the model has them (`FAR_TILE` and
/// `TILE` in `data/story/story_urbs.py`).
const BLOCK: f32 = 256.0;
const TILE: f32 = 128.0;
/// The collections the model gathers into tiles.
const COLLECTIONS: [&str; 4] = ["Ground", "Structures", "Cover", "Trim"];
/// What a simplified block's meshes are named for.
const FAR: &str = "Far";
/// How near the player a block has to be, at its nearest, to be shown in full, in meters; and how
/// much further it has to go, or nearer to come, to change, so that one on the edge does not
/// flicker between the two.
const NEAR: f32 = 220.0;
const MARGIN: f32 = 15.0;

#[derive(Debug, Default, PartialEq)]
pub struct Distant {
    blocks: Vec<Block>,
}

#[derive(Debug, PartialEq)]
struct Block {
    /// Which, counted as the model counts them, east and north.
    at: (i32, i32),
    /// Its full detail, and its simplified copy.
    near: Vec<Handle<Node>>,
    far: Vec<Handle<Node>>,
    /// Whether it is showing its simplified copy.
    far_off: bool,
}

impl Distant {
    /// Takes the simplified blocks of the model under `root` out of it, under a node of their own,
    /// returned to be removed with the level; and finds the tiles each stands in for, and the
    /// `lamp_glass` in it. All of it is shown in full to begin with.
    pub fn claim(graph: &mut Graph, root: Handle<Node>, lamp_glass: &[Handle<Node>]) -> (Self, Option<Handle<Node>>) {
        let mut blocks: BTreeMap<(i32, i32), Block> = BTreeMap::new();
        fn block(blocks: &mut BTreeMap<(i32, i32), Block>, at: (i32, i32)) -> &mut Block {
            blocks.entry(at).or_insert_with(|| Block { at, near: Vec::new(), far: Vec::new(), far_off: false })
        }
        let meshes: Vec<Handle<Node>> = graph.traverse_handle_iter(root).filter(|&h| graph[h].is_mesh()).collect();
        for handle in meshes {
            let name = graph[handle].name();
            if let Some((collection, at)) = tile_of(name) {
                if collection.starts_with(FAR) {
                    block(&mut blocks, at).far.push(handle);
                } else {
                    // A tile is in the block its corner is in: tiles are a whole number to a block.
                    let per = (BLOCK / TILE) as i32;
                    block(&mut blocks, (at.0.div_euclid(per), at.1.div_euclid(per))).near.push(handle);
                }
            }
        }
        for &glass in lamp_glass {
            if graph.try_get(glass).is_ok() {
                block(&mut blocks, block_at(graph[glass].world_bounding_box().center())).near.push(glass);
            }
        }
        // Only blocks the model simplified change; without a copy, a block is shown in full.
        let blocks: Vec<Block> = blocks.into_values().filter(|block| !block.far.is_empty()).collect();
        if blocks.is_empty() {
            return (Self::default(), None);
        }
        let far_root = PivotBuilder::new(BaseBuilder::new().with_name("far off")).build(graph).to_base();
        for block in &blocks {
            for &far in &block.far {
                graph.link_nodes_keep_global_transform(far, far_root);
                graph[far].set_visibility(false);
            }
        }
        Log::info(format!(
            "Maze: {} blocks simplified far off, {} meshes in full, {} simplified",
            blocks.len(),
            blocks.iter().map(|block| block.near.len()).sum::<usize>(),
            blocks.iter().map(|block| block.far.len()).sum::<usize>()
        ));
        (Self { blocks }, Some(far_root))
    }

    /// Shows each block in full or simplified, by how far it is from `viewer`.
    pub fn update(&mut self, graph: &mut Graph, viewer: Vector3<f32>) {
        for block in &mut self.blocks {
            let away = distance_to(block.at, viewer);
            let far_off = if block.far_off { away > NEAR - MARGIN } else { away > NEAR + MARGIN };
            if far_off == block.far_off {
                continue;
            }
            block.far_off = far_off;
            for &near in &block.near {
                graph[near].set_visibility(!far_off);
            }
            for &far in &block.far {
                graph[far].set_visibility(far_off);
            }
        }
    }
}

/// The collection a tile's or a block's mesh is of, and which it is, from its name:
/// `<collection>_<x>_<y>`. None for any other mesh.
fn tile_of(name: &str) -> Option<(&str, (i32, i32))> {
    let mut parts = name.rsplitn(3, '_');
    let y = parts.next()?.parse().ok()?;
    let x = parts.next()?.parse().ok()?;
    let collection = parts.next()?;
    COLLECTIONS
        .contains(&collection.strip_prefix(FAR).unwrap_or(collection))
        .then_some((collection, (x, y)))
}

/// The block `at` is in. The model's north, +y, is the world's -z.
fn block_at(at: Vector3<f32>) -> (i32, i32) {
    ((at.x / BLOCK).floor() as i32, (-at.z / BLOCK).floor() as i32)
}

/// How far `viewer` is from the nearest of the block `at`, along the ground, in meters: 0 in it.
fn distance_to((x, y): (i32, i32), viewer: Vector3<f32>) -> f32 {
    let (x0, x1) = (x as f32 * BLOCK, (x + 1) as f32 * BLOCK);
    let (z0, z1) = (-(y + 1) as f32 * BLOCK, -y as f32 * BLOCK);
    let dx = (x0 - viewer.x).max(viewer.x - x1).max(0.0);
    let dz = (z0 - viewer.z).max(viewer.z - z1).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

/// The lamps' glass of the model under `root`, before it is glazed.
pub fn lamp_glass(graph: &Graph, root: Handle<Node>) -> Vec<Handle<Node>> {
    graph
        .traverse_handle_iter(root)
        .filter(|&h| graph[h].cast::<Mesh>().is_some_and(fixtures::is_lamp_glass))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tile_and_a_block_are_known_by_their_names() {
        assert_eq!(tile_of("Trim_3_-2"), Some(("Trim", (3, -2))));
        assert_eq!(tile_of("FarStructures_-1_0"), Some(("FarStructures", (-1, 0))));
        assert_eq!(tile_of("Windows_1_1"), None, "shown whatever the distance");
        assert_eq!(tile_of("O123"), None);
        assert_eq!(tile_of("cell_wall_2_3"), None);
    }

    #[test]
    fn the_distance_to_a_block_is_to_its_nearest() {
        // The block (0, 0) is x 0 to 256 and the model's y 0 to 256: the world's z 0 to -256.
        assert_eq!(distance_to((0, 0), Vector3::new(100.0, 0.0, -100.0)), 0.0, "in it");
        assert_eq!(distance_to((0, 0), Vector3::new(300.0, 5.0, -100.0)), 44.0);
        assert_eq!(distance_to((0, 0), Vector3::new(100.0, 0.0, 10.0)), 10.0, "to its south");
        assert_eq!(block_at(Vector3::new(100.0, 0.0, -100.0)), (0, 0));
        assert_eq!(block_at(Vector3::new(-1.0, 0.0, 1.0)), (-1, -1));
    }
}
