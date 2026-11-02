//! The leaves of a map's trees and bushes, to the quadcopter (see [`crate::drone`]). The meshes
//! named [`FOLIAGE`] are no part of the level's collider - they hide whoever is in them but stop
//! nothing, and give the walk grid no floor in the treetops - so a quadcopter flying free would
//! fly straight through them. It keeps clear of them in a world of their own: a copy of the
//! level's meshes, apart from the scene, whose physics has the leaves alone, for it to look for a
//! way past them in as it looks in the scene's for a way past what is solid. Nothing else in the
//! game - the player, the droids, a bolt, a line of sight - ever meets them.

use crate::level::FOLIAGE;
use fyrox::{
    core::{algebra::Vector2, log::Log, pool::Handle},
    graph::SceneGraph,
    scene::{
        base::BaseBuilder,
        collider::{ColliderBuilder, ColliderShape, GeometrySource},
        graph::Graph,
        node::Node,
        rigidbody::{RigidBodyBuilder, RigidBodyType},
    },
};
use std::sync::Arc;

/// The world of a level's leaves.
#[derive(Clone)]
pub struct Leaves(Arc<Graph>);

impl Leaves {
    /// The leaves of the level whose model is under `root` in `graph`, its place in the world
    /// worked out already; None if it has none.
    pub fn of(graph: &Graph, root: Handle<Node>) -> Option<Self> {
        let leafy = |graph: &Graph, node: Handle<Node>| graph[node].is_mesh() && graph[node].name().starts_with(FOLIAGE);
        if !graph.traverse_handle_iter(root).any(|node| leafy(graph, node)) {
            return None;
        }
        // The whole model, so that every mesh is where it is in the scene, and a collider of the
        // leaves in it.
        let mut leaves = Graph::new();
        let (copy, _) = graph.copy_node(root, &mut leaves, false, &mut |_, _| true, &mut |_, _| (), &mut |_, _, _| ());
        leaves.update_hierarchical_data();
        let meshes: Vec<GeometrySource> = leaves
            .traverse_handle_iter(copy)
            .filter(|&node| leafy(&leaves, node))
            .map(GeometrySource)
            .collect();
        Log::info(format!("Leaves: {} meshes, to keep the quadcopters out of", meshes.len()));
        let collider = ColliderBuilder::new(BaseBuilder::new())
            .with_shape(ColliderShape::trimesh(meshes))
            .build(&mut leaves);
        RigidBodyBuilder::new(BaseBuilder::new().with_child(collider))
            .with_body_type(RigidBodyType::Static)
            .build(&mut leaves);
        // The physics has its colliders by the second update.
        for _ in 0..2 {
            leaves.update(Vector2::new(1.0, 1.0), 0.0, Default::default());
        }
        Some(Self(Arc::new(leaves)))
    }

    /// Their world, to look for a way past them in.
    pub fn graph(&self) -> &Graph {
        &self.0
    }
}

impl std::fmt::Debug for Leaves {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Leaves")
    }
}

impl PartialEq for Leaves {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fyrox::{
        core::algebra::{Isometry3, Matrix4, Vector3},
        scene::{
            graph::physics::geometry::Ball,
            mesh::{
                surface::{SurfaceBuilder, SurfaceData, SurfaceResource},
                MeshBuilder,
            },
            pivot::PivotBuilder,
            transform::TransformBuilder,
        },
    };

    /// A cube 2 m across named `name`, with its middle at `at`, under `root`.
    fn cube(graph: &mut Graph, root: Handle<Node>, name: &str, at: Vector3<f32>) {
        let cube = MeshBuilder::new(
            BaseBuilder::new()
                .with_name(name)
                .with_local_transform(TransformBuilder::new().with_local_position(at).build()),
        )
        .with_surfaces(vec![SurfaceBuilder::new(SurfaceResource::new_embedded(SurfaceData::make_cube(
            Matrix4::new_scaling(2.0),
        )))
        .build()])
        .build(graph);
        graph.link_nodes(cube, root);
    }

    /// Whether a ball 0.3 m across its middle flies from `from` 10 m along x untouched.
    fn through(leaves: &Leaves, from: Vector3<f32>) -> bool {
        let graph = leaves.graph();
        let start = Isometry3::translation(from.x, from.y, from.z);
        graph.physics.cast_shape(graph, &Ball::new(0.3), &start, &Vector3::new(10.0, 0.0, 0.0), 1.0, false, Default::default()).is_none()
    }

    #[test]
    fn only_the_leaves_are_in_their_world_where_they_are() {
        let mut graph = Graph::new();
        let root = PivotBuilder::new(
            BaseBuilder::new().with_local_transform(TransformBuilder::new().with_local_position(Vector3::new(0.0, 0.0, 20.0)).build()),
        )
        .build(&mut graph)
        .to_base();
        assert!(Leaves::of(&graph, root).is_none(), "leaves in a level with none");
        cube(&mut graph, root, "Foliage_0_0", Vector3::new(5.0, 6.0, 0.0));
        cube(&mut graph, root, "Trunks_0_0", Vector3::new(5.0, 1.0, 0.0));
        graph.update_hierarchical_data();
        let leaves = Leaves::of(&graph, root).expect("its leaves");
        // Where the model puts them, under its root moved 20 m along z.
        assert!(!through(&leaves, Vector3::new(0.0, 6.0, 20.0)), "through the leaves");
        assert!(through(&leaves, Vector3::new(0.0, 6.0, 0.0)), "leaves where they are not");
        // The trunks are no part of it: they are in the level's collider.
        assert!(through(&leaves, Vector3::new(0.0, 1.0, 20.0)), "the trunks among the leaves");
    }
}
