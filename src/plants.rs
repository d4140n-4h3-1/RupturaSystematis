//! The Wilderness's trees and undergrowth (plant_life's models, CC0, copied into `data/forest`),
//! planted where a map marks them.
//!
//! A map built with them (`data/arena/tb_forest.py`) has no trees of its own: an empty named
//! `Plant_<model>_<n>` ([`MARKER`]), with [`FAR`] after it out of reach, stands where each grows, turned and scaled as it grows, and a
//! copy of `<model>.glb` is planted under it. A tree's trunk is in the map as a plain post, in the
//! meshes named [`TRUNK_POSTS`]: solid, as the level's collider is made from the map before
//! anything is planted, and hidden behind the model's own bark. The planted meshes are named
//! `Foliage_<model>`, as leaves are (see [`crate::level::FOLIAGE`]): they stop nothing.
//!
//! Each model is made ready once, as the Wilderness makes them (its `models.rs`): every copy
//! shares its surface data. Surfaces meant to be seen from both sides - the glTF materials'
//! `doubleSided`, which the importer drops - get a reversed twin of every triangle, since the
//! renderer culls faces seen from behind and a leaf card is a single sheet. The wind's morph
//! targets, and the clip baked from them, are dropped: the trees stand still. The undergrowth -
//! [`MODELS`] under `plants` - casts no shadows: low and many, its shadows hardly showed, and
//! drawing it again into every shadow map cost more than drawing it once. Nor does a tree out of
//! reach, past the wall round a map - most of a small map's trees - whose shadow falls past it.

use crate::level::FOLIAGE;
use fyrox::{
    asset::manager::ResourceManager,
    core::{algebra::Vector4, log::Log, math::TriangleDefinition, pool::Handle},
    fxhash::FxHashSet,
    graph::SceneGraph,
    resource::model::{Model, ModelResource, ModelResourceExtension},
    scene::{
        animation::AnimationPlayer,
        graph::Graph,
        mesh::{
            buffer::{VertexAttributeUsage, VertexReadTrait, VertexWriteTrait},
            surface::SurfaceData,
            Mesh,
        },
        node::Node,
        Scene,
    },
};

/// What a map's markers for plants are named, starting: then the model's file name, without
/// `.glb`, and a number.
pub const MARKER: &str = "Plant_";
/// What a map's posts for its trees' trunks are named, starting.
pub const TRUNK_POSTS: &str = "TreeTrunks";
/// What a marker's name ends in for a plant out of reach, which casts no shadow.
pub const FAR: &str = "_far";

/// The models there are to plant.
pub const MODELS: [&str; 12] = [
    "data/forest/trees/norway-spruce.glb",
    "data/forest/trees/pine.glb",
    "data/forest/trees/silver-fir.glb",
    "data/forest/trees/european-larch.glb",
    "data/forest/trees/silver-birch.glb",
    "data/forest/trees/rowan.glb",
    "data/forest/trees/english-oak.glb",
    "data/forest/plants/common-hawthorn.glb",
    "data/forest/plants/common-broom.glb",
    "data/forest/plants/bracken.glb",
    "data/forest/plants/hard-fern.glb",
    "data/forest/plants/bilberry.glb",
];

/// The models, asked for, to plant once they have loaded.
#[derive(Debug, Default)]
pub struct Plants {
    /// Each model's name (its file's, without `.glb`), the model, which of its surfaces, in
    /// order, are seen from both sides, and whether it casts shadows.
    models: Vec<(String, ModelResource, Vec<bool>, bool)>,
    /// The surface data made ready already.
    prepared: FxHashSet<u64>,
}

impl Plants {
    /// Asks for every one of [`MODELS`].
    pub fn request(resources: &ResourceManager) -> Self {
        let models = MODELS
            .iter()
            .map(|&path| {
                let name = path.rsplit('/').next().unwrap_or(path).trim_end_matches(".glb").to_string();
                (name, resources.request::<Model>(path), double_sided(path), !path.contains("/plants/"))
            })
            .collect();
        Self { models, prepared: Default::default() }
    }

    /// Whether every model has loaded, or failed to.
    pub fn loaded(&self) -> bool {
        self.models.iter().all(|(_, model, _, _)| model.is_ok() || model.is_failed_to_load())
    }

    /// Plants a copy of its model under every marker under `root`, and hides the trunks' posts.
    /// Returns how many were planted.
    pub fn plant(&mut self, scene: &mut Scene, root: Handle<Node>) -> usize {
        let graph = &scene.graph;
        let markers: Vec<(Handle<Node>, String, bool)> = graph
            .traverse_handle_iter(root)
            .filter_map(|h| {
                let rest = graph[h].name().strip_prefix(MARKER)?;
                let (rest, far) = rest.strip_suffix(FAR).map_or((rest, false), |rest| (rest, true));
                Some((h, rest.rsplit_once('_').map_or(rest, |(name, _)| name).to_string(), far))
            })
            .collect();
        let posts: Vec<Handle<Node>> = graph
            .traverse_handle_iter(root)
            .filter(|&h| graph[h].is_mesh() && graph[h].name().starts_with(TRUNK_POSTS))
            .collect();
        for post in posts {
            scene.graph[post].set_visibility(false);
        }
        let mut planted = 0;
        let mut missing = FxHashSet::default();
        for (marker, name, far) in markers {
            let Some((_, model, double_sided, shadows)) = self.models.iter().find(|(n, model, _, _)| *n == name && model.is_ok()) else {
                missing.insert(name);
                continue;
            };
            let (model, double_sided, shadows) = (model.clone(), double_sided.clone(), *shadows && !far);
            let copy = model.instantiate(scene);
            scene.graph.link_nodes(copy, marker);
            self.prepare(&mut scene.graph, copy, &double_sided, &name, shadows);
            planted += 1;
        }
        if !missing.is_empty() {
            Log::warn(format!("Plants: no model for {missing:?}"));
        }
        planted
    }

    /// Makes the copy under `root` ready (see the module's description).
    fn prepare(&mut self, graph: &mut Graph, root: Handle<Node>, double_sided: &[bool], name: &str, shadows: bool) {
        let nodes: Vec<Handle<Node>> = graph.traverse_handle_iter(root).collect();
        for &handle in &nodes {
            // The baked wind clip's player: with the wind's targets gone it has nothing to move.
            if graph[handle].cast::<AnimationPlayer>().is_some() {
                graph.remove_node(handle);
            }
        }
        for handle in nodes {
            let Some(mesh) = graph.try_get_node_mut(handle).ok().and_then(|n| n.cast_mut::<Mesh>()) else {
                continue;
            };
            mesh.set_name(format!("{FOLIAGE}_{name}"));
            mesh.set_cast_shadows(shadows);
            for (index, surface) in mesh.surfaces().iter().enumerate() {
                let resource = surface.data();
                if !self.prepared.insert(resource.key()) {
                    continue;
                }
                let mut data = resource.data_ref();
                data.blend_shapes_container = None;
                if double_sided.get(index).copied().unwrap_or(false) {
                    double(&mut data);
                }
            }
        }
    }
}

/// Which of the glTF model's primitives at `path`, in order, are meant to be seen from both
/// sides. Read from the file itself, as the importer drops it. Empty if it cannot be read.
pub fn double_sided(path: &str) -> Vec<bool> {
    let json = std::fs::read(path).ok().and_then(|bytes| {
        // A binary glTF: a 12-byte header, then the JSON chunk's length, type, and the JSON.
        let length = u32::from_le_bytes(bytes.get(12..16)?.try_into().ok()?) as usize;
        serde_json::from_slice::<serde_json::Value>(bytes.get(20..20 + length)?).ok()
    });
    let Some(json) = json else {
        Log::warn(format!("Could not read the glTF of {path}: no surface is made double-sided"));
        return Vec::new();
    };
    let materials = json["materials"].as_array().cloned().unwrap_or_default();
    json["meshes"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|mesh| mesh["primitives"].as_array().cloned().unwrap_or_default())
        .map(|primitive| {
            primitive["material"]
                .as_u64()
                .and_then(|m| materials.get(m as usize))
                .and_then(|m| m["doubleSided"].as_bool())
                .unwrap_or(false)
        })
        .collect()
}

/// Gives every triangle of `data` a reversed twin, with its own copies of the vertices, normals
/// turned over.
fn double(data: &mut SurfaceData) {
    let original_count = data.vertex_buffer.vertex_count();
    let raw = data.vertex_buffer.raw_data().to_vec();
    let vertex_size = data.vertex_buffer.vertex_size() as usize;
    {
        let mut vertices = data.vertex_buffer.modify();
        for vertex in raw.chunks_exact(vertex_size) {
            let _ = vertices.push_vertex_raw(vertex);
        }
        for mut vertex in vertices.iter_mut().skip(original_count as usize) {
            if let Ok(normal) = vertex.read_3_f32(VertexAttributeUsage::Normal) {
                let _ = vertex.write_3_f32(VertexAttributeUsage::Normal, -normal);
            }
            // The tangent follows the texture's u across the card, which the flip leaves as it
            // was; with the normal turned over, the bitangent (normal x tangent, times w) would
            // turn over too, so w is flipped to keep it following v.
            if let Ok(t) = vertex.read_4_f32(VertexAttributeUsage::Tangent) {
                let _ = vertex.write_4_f32(VertexAttributeUsage::Tangent, Vector4::new(t.x, t.y, t.z, -t.w));
            }
        }
    }
    let reversed: Vec<TriangleDefinition> = data
        .geometry_buffer
        .iter()
        .map(|t| TriangleDefinition([t.0[0] + original_count, t.0[2] + original_count, t.0[1] + original_count]))
        .collect();
    let mut triangles = data.geometry_buffer.modify();
    for triangle in reversed {
        triangles.push(triangle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_is_there_and_has_leaves_seen_from_both_sides() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in MODELS {
            assert!(dir.join(path).is_file(), "no model at {path}");
            let sided = double_sided(dir.join(path).to_str().unwrap());
            assert!(sided.iter().any(|&both| both), "{path}: no surface seen from both sides, {sided:?}");
        }
    }
}
