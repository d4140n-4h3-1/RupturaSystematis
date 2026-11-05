//! Hearts: health items floating in the maze's corridors, each slowly spinning and bobbing up and
//! down. Walking into one with health to make up picks it up (see [`Hearts::take`]), and gives
//! back a good part of it at once (see [`crate::health`]).
//!
//! The model, [`HEART_MODEL`], is made in Blender from `health.blend` and exported as it is; it
//! is scaled here to [`SIZE`] however big it was made. The engine takes a glTF model's surfaces
//! for solid whatever their alpha, so a see-through one - the heart's outer layer - is made glass
//! here, of its own colour ([`Hearts::glaze`]), as the ceiling's panes are.
//!
//! As it glows, each lights what is round it too, red, from a lamp in its middle, whose shadows
//! are traced as every light's are. The heart itself casts none, or it would shut its own light
//! in.
//!
//! A few are scattered afresh each maze ([`count`]), each on open floor away from the start and
//! from the others ([`spots`]) - the first a little way from the start, so there is one to come
//! across early.
//!
//! Shield cells - spare charges for the shield (see [`crate::shield`]) - float about the same
//! way, from their own model, [`SHIELD_PICKUP_MODEL`], made in Blender from `shield.blend`, with
//! a blue lamp. They are rarer: about one for every three hearts ([`shield_count`]). Stamina
//! cells, which freeze the player's breath for a while (see [`crate::player::breath::FREEZE`]),
//! the same, from [`STAMINA_PICKUP_MODEL`], made from `stamina.blend`, with a green lamp: rarer
//! than hearts too, but commoner than shield cells ([`stamina_count`]).

use crate::{
    fixtures::{property, DIFFUSE_COLOR},
    layout::Rng,
};
use hydroxus_ai::navigation::{
    navmesh::{NavMesh, Poly},
    walk,
};
use fyrox_gfx::{replace_materials, GlassMaterial};
use fyrox::{
    core::{
        algebra::{Matrix4, Point3, UnitQuaternion, Vector3},
        color::Color,
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{Material, MaterialProperty, MaterialResource},
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{
        base::BaseBuilder,
        graph::Graph,
        light::{point::PointLightBuilder, BaseLightBuilder},
        mesh::{
            buffer::{VertexAttributeUsage, VertexReadTrait},
            Mesh,
        },
        node::Node,
        Scene,
    },
};

/// The heart's model, and the shield cell's.
pub const HEART_MODEL: &str = "data/health.glb";
pub const SHIELD_PICKUP_MODEL: &str = "data/shield_pickup.glb";
pub const STAMINA_PICKUP_MODEL: &str = "data/stamina_pickup.glb";
/// The shield cells' lamp, and the stamina cells'.
pub const SHIELD_LAMP: Color = Color::opaque(60, 150, 255);
pub const STAMINA_LAMP: Color = Color::opaque(60, 255, 110);
/// How near the middle of the player's body a heart has to be to be picked up, in meters.
const TAKE_REACH: f32 = 0.9;
/// How big a heart is across its biggest side, in meters.
const SIZE: f32 = 0.45;
/// How high the middle of a heart floats above the floor, in meters; how far it bobs up and
/// down from there, and how long a bob takes, in seconds; and how long it takes to turn round
/// once, in seconds.
const HOVER: f32 = 1.1;
const BOB: f32 = 0.05;
const BOB_TIME: f32 = 2.0;
const TURN_TIME: f32 = 4.0;
/// Its lamp: its colour, how bright it is, and how far it reaches, in meters.
const LAMP_COLOUR: Color = Color::opaque(255, 40, 40);
const LAMP_BRIGHTNESS: f32 = 1.0;
const LAMP_REACH: f32 = 2.5;
/// How many hearts a maze gets: one for so many cells of floor, between the fewest and the most.
const FLOOR_PER_HEART: usize = 5000;
const FEWEST: usize = 3;
const MOST: usize = 12;
/// And how many shield cells: rarer, a third as many.
const FLOOR_PER_SHIELD: usize = 15000;
const FEWEST_SHIELDS: usize = 1;
const MOST_SHIELDS: usize = 4;
/// And how many stamina cells: between the two, half as many as hearts.
const FLOOR_PER_STAMINA: usize = 10000;
const FEWEST_STAMINA: usize = 2;
const MOST_STAMINA: usize = 6;
/// How many cells of walking from the start a heart has to be at least, and the first at most;
/// how far apart two hearts are at first asked to be; and how far out all round a heart has to
/// have floor, so that it floats in a corridor rather than against a wall - all in meters.
const AWAY_FROM_START: f32 = 8.0;
const NEAR_START: f32 = 20.0;
const APART: f32 = 20.0;
const ROOM: f32 = 1.0;

/// How many hearts a maze with `floor` cells of floor gets.
pub fn count(floor: usize) -> usize {
    (floor / FLOOR_PER_HEART).clamp(FEWEST, MOST)
}

/// How many shield cells a maze with `floor` cells of floor gets: fewer than hearts.
pub fn shield_count(floor: usize) -> usize {
    (floor / FLOOR_PER_SHIELD).clamp(FEWEST_SHIELDS, MOST_SHIELDS)
}

/// How many stamina cells a maze with `floor` cells of floor gets: fewer than hearts, more than
/// shield cells.
pub fn stamina_count(floor: usize) -> usize {
    (floor / FLOOR_PER_STAMINA).clamp(FEWEST_STAMINA, MOST_STAMINA)
}

/// How many spots picked at random [`spots`] tries for each, before asking for less room.
const TRIES: usize = 4000;

/// Where `count` hearts go on `mesh`, whose player starts at `start`: on open floor (see [`ROOM`])
/// far enough from the start to walk - the first no further than [`NEAR_START`], where there is
/// room - as far apart as there is room for, each picked at random by `rng` from the floor that
/// can be walked to, as likely anywhere of it as anywhere else. Fewer if there is no room for
/// them all.
pub fn spots(mesh: &NavMesh, start: Vector3<f32>, count: usize, rng: &mut Rng) -> Vec<Vector3<f32>> {
    let reached = mesh.entries_from(start, f32::INFINITY);
    let mut polygons: Vec<(Poly, [f32; 2], f32)> = reached.iter().map(|(&n, &(entry, cost))| (n, entry, cost)).collect();
    polygons.sort_unstable_by_key(|&(n, _, _)| n);
    // Each by how much floor it has, for picking somewhere as likely anywhere as anywhere else.
    let mut floor = 0.0;
    let upto: Vec<f32> = polygons
        .iter()
        .map(|&(n, _, _)| {
            floor += mesh.area_of(n);
            floor
        })
        .collect();
    if floor <= 0.0 {
        return Vec::new();
    }
    // Floor all round, at its height: a stair's step up or down at most.
    let open = |at: Vector3<f32>| {
        (0..8).all(|k| {
            let angle = k as f32 * std::f32::consts::FRAC_PI_4;
            walk::floor_at(mesh, at + Vector3::new(angle.sin(), 0.0, angle.cos()) * ROOM, crate::AI.grid.max_climb)
        })
    };
    // Somewhere picked at random, and how far it is to walk there from the start.
    let pick = |rng: &mut Rng| {
        let roll = rng.below(1 << 20) as f32 / (1 << 20) as f32 * floor;
        let k = upto.partition_point(|&u| u < roll).min(polygons.len() - 1);
        let (n, entry, cost) = polygons[k];
        let at = mesh.random_point(n, &mut |m| rng.below(m));
        (at, cost + (at.x - entry[0]).hypot(at.z - entry[1]))
    };
    let mut spots: Vec<Vector3<f32>> = Vec::new();
    if count > 0 {
        let near = (0..TRIES)
            .map(|_| pick(rng))
            .find(|&(at, walk)| (AWAY_FROM_START..=NEAR_START).contains(&walk) && open(at));
        spots.extend(near.map(|(at, _)| at));
    }
    let mut apart = APART;
    while spots.len() < count && apart >= 0.5 {
        let far_enough = |at: Vector3<f32>, spots: &[Vector3<f32>]| spots.iter().all(|s| (s.x - at.x).hypot(s.z - at.z) >= apart);
        match (0..TRIES).map(|_| pick(rng)).find(|&(at, walk)| walk >= AWAY_FROM_START && open(at) && far_enough(at, &spots)) {
            Some((at, _)) => spots.push(at),
            // No room that far apart: nearer, then.
            None => apart /= 2.0,
        }
    }
    spots
}

/// The hearts in the scene.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Hearts {
    /// Each heart and its lamp, where it floats - above the middle of its cell - and how far into
    /// its bob and its turn it started, as a part of each.
    hearts: Vec<(Handle<Node>, Handle<Node>, Vector3<f32>, f32)>,
    /// How big the model is made, to be [`SIZE`] across.
    scale: f32,
    /// The glass its see-through surfaces are made of, shared by every heart.
    glass: Option<MaterialResource>,
    /// The colour of each one's lamp - a heart's red, without - and what they are called.
    lamp: Option<Color>,
    name: Option<&'static str>,
    /// How long they have been floating, in seconds.
    time: f32,
}

impl Hearts {
    /// Floating things called `name`, lit by lamps of `colour`, rather than hearts' red.
    pub fn with_lamp(name: &'static str, colour: Color) -> Self {
        Self { lamp: Some(colour), name: Some(name), ..Default::default() }
    }

    /// Takes the hearts of the last maze out of `scene`, and puts one from `model` over each of
    /// `spots`, on the floor.
    pub fn place(&mut self, model: &ModelResource, scene: &mut Scene, spots: &[Vector3<f32>]) {
        self.clear(&mut scene.graph);
        for (n, &spot) in spots.iter().enumerate() {
            let root = model.instantiate(scene);
            if n == 0 {
                self.scale = scale_for(&scene.graph, root);
            }
            self.glaze(&mut scene.graph, root);
            let meshes: Vec<Handle<Node>> = scene.graph.traverse_handle_iter(root).collect();
            for node in meshes {
                if scene.graph[node].cast::<Mesh>().is_some() {
                    scene.graph[node].set_cast_shadows(false);
                }
            }
            // Not scattering into a haze in the air: the heart is what glows.
            let lamp = PointLightBuilder::new(
                BaseLightBuilder::new(BaseBuilder::new())
                    .with_color(self.lamp.unwrap_or(LAMP_COLOUR))
                    .with_intensity(LAMP_BRIGHTNESS)
                    .with_scatter_enabled(false),
            )
            .with_radius(LAMP_REACH)
            .build(&mut scene.graph)
            .to_base();
            let at = spot + Vector3::new(0.0, HOVER, 0.0);
            // Each out of step with the others.
            let phase = n as f32 * 0.37 % 1.0;
            self.hearts.push((root, lamp, at, phase));
        }
        Log::info(format!("{}: {} in the maze", self.name.unwrap_or("Hearts"), self.hearts.len()));
        self.update(&mut scene.graph, 0.0);
    }

    /// Makes the see-through surfaces of the heart under `root` glass, of their own colour.
    fn glaze(&mut self, graph: &mut Graph, root: Handle<Node>) {
        let see_through = |material: &Material| match property(material, DIFFUSE_COLOR) {
            Some(MaterialProperty::Color(colour)) if colour.a < 255 => Some(colour),
            _ => None,
        };
        if self.glass.is_none() {
            // The first see-through colour there is, which is the one the heart has.
            let mut colour = None;
            for node in graph.traverse_handle_iter(root) {
                if let Some(mesh) = graph[node].cast::<Mesh>() {
                    for surface in mesh.surfaces() {
                        let state = surface.material().state();
                        colour = colour.or_else(|| state.data_ref().and_then(see_through));
                    }
                }
            }
            let Some(colour) = colour else {
                return;
            };
            self.glass = Some(
                GlassMaterial {
                    tint: Color::opaque(colour.r, colour.g, colour.b),
                    // As much as the surface hides of what is behind it.
                    tint_strength: colour.a as f32 / 255.0,
                    ..Default::default()
                }
                .build_resource(),
            );
        }
        if let Some(glass) = &self.glass {
            replace_materials(graph, root, |material| see_through(material).is_some(), glass);
        }
    }

    /// Takes them all out of `graph`.
    pub fn clear(&mut self, graph: &mut Graph) {
        for (root, lamp, ..) in self.hearts.drain(..) {
            for node in [root, lamp] {
                if graph.is_valid_handle(node) {
                    graph.remove_node(node);
                }
            }
        }
    }

    /// Bobs and turns each another `dt` seconds on.
    pub fn update(&mut self, graph: &mut Graph, dt: f32) {
        self.time += dt;
        for &(root, lamp, at, phase) in &self.hearts {
            let bob = BOB * ((self.time / BOB_TIME + phase) * std::f32::consts::TAU).sin();
            let turn = (self.time / TURN_TIME + phase) * std::f32::consts::TAU;
            let here = at + Vector3::new(0.0, bob, 0.0);
            if let Ok(node) = graph.try_get_mut(root) {
                node.local_transform_mut()
                    .set_position(here)
                    .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), turn))
                    .set_scale(Vector3::repeat(self.scale));
            }
            if let Ok(node) = graph.try_get_mut(lamp) {
                node.local_transform_mut().set_position(here);
            }
        }
    }

    /// Picks up the heart within [`TAKE_REACH`] of the player's middle at `player`, if there is
    /// one: it is gone from the maze. Whether there was.
    pub fn take(&mut self, graph: &mut Graph, player: Vector3<f32>) -> bool {
        let Some(n) = self
            .hearts
            .iter()
            .position(|&(_, _, at, _)| (at - player).norm() < TAKE_REACH)
        else {
            return false;
        };
        let (root, lamp, ..) = self.hearts.swap_remove(n);
        for node in [root, lamp] {
            if graph.is_valid_handle(node) {
                graph.remove_node(node);
            }
        }
        true
    }

    /// Shows only the hearts `can_see` says could be seen from where the player is.
    pub fn cull(&self, graph: &mut Graph, can_see: impl Fn(Vector3<f32>) -> bool) {
        for &(root, lamp, at, _) in &self.hearts {
            // Its lamp too: a light is only worked out while it is showing, and it lights no
            // further than the heart can be seen from.
            for node in [root, lamp] {
                if let Ok(node) = graph.try_get_mut(node) {
                    node.set_visibility(can_see(at));
                }
            }
        }
    }
}

/// How much the model under `root` has to be scaled to be [`SIZE`] across its biggest side, as
/// its meshes are placed under it.
fn scale_for(graph: &Graph, root: Handle<Node>) -> f32 {
    let (mut low, mut high) = (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN));
    for node in graph.traverse_handle_iter(root) {
        let Some(mesh) = graph[node].cast::<Mesh>() else {
            continue;
        };
        // Its place under the root, from its own transform and each parent's up to the root.
        let mut transform: Matrix4<f32> = graph[node].local_transform().matrix();
        let mut above = graph[node].parent();
        while above.is_some() && above != root {
            transform = graph[above].local_transform().matrix() * transform;
            above = graph[above].parent();
        }
        for surface in mesh.surfaces() {
            let data = surface.data();
            let data = data.data_ref();
            for vertex in data.vertex_buffer.iter() {
                if let Ok(position) = vertex.read_3_f32(VertexAttributeUsage::Position) {
                    let point = transform.transform_point(&Point3::from(position)).coords;
                    low = low.inf(&point);
                    high = high.sup(&point);
                }
            }
        }
    }
    let biggest = (high - low).max();
    if biggest.is_finite() && biggest > 0.0 {
        SIZE / biggest
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::WalkGrid;

    #[test]
    fn shield_cells_are_rarer_than_hearts() {
        for floor in [0, 5_000, 20_000, 60_000, 150_000, 1_000_000] {
            assert!(shield_count(floor) >= 1, "at least one");
            assert!(shield_count(floor) < count(floor), "fewer than hearts in {floor}");
        }
    }

    #[test]
    fn stamina_cells_are_rarer_than_hearts_and_commoner_than_shield_cells() {
        for floor in [0, 5_000, 20_000, 60_000, 150_000, 1_000_000] {
            assert!(stamina_count(floor) < count(floor), "fewer than hearts in {floor}");
            assert!(stamina_count(floor) > shield_count(floor), "more than shield cells in {floor}");
        }
    }

    #[test]
    fn a_few_hearts_for_any_maze() {
        assert_eq!(count(0), FEWEST);
        assert_eq!(count(5_000), FEWEST);
        assert_eq!(count(20_000), 4);
        assert_eq!(count(1_000_000), MOST);
    }

    #[test]
    fn hearts_go_on_open_floor_away_from_the_start_and_from_each_other() {
        // A room 60 cells wide and deep, walled all round.
        let mut grid = WalkGrid::new(62, 62, crate::survey::CELL_SIZE);
        for x in 1..61 {
            for z in 1..61 {
                grid.set(x, z, true);
            }
        }
        let mesh = NavMesh::build(&grid, Vector3::zeros(), |_, _| 0);
        let start = Vector3::new(2.75, 0.0, 2.75);
        let spots = spots(&mesh, start, 4, &mut Rng::new(3));
        assert_eq!(spots.len(), 4);
        // In the open, as the crow flies is how far it is to walk.
        let away = |at: Vector3<f32>| (at.x - start.x).hypot(at.z - start.z);
        assert!(away(spots[0]) <= NEAR_START + 0.5, "the first near the start");
        for (n, &at) in spots.iter().enumerate() {
            assert!(away(at) >= AWAY_FROM_START - 0.5, "{at:?}");
            // The walls are the first and last half meter each way, the room 30 m across.
            let wall = 0.5 + ROOM;
            assert!(at.x >= wall && at.z >= wall && at.x <= 30.5 - ROOM && at.z <= 30.5 - ROOM, "not against a wall: {at:?}");
            for &other in &spots[n + 1..] {
                assert!((other - at).norm() > 1.0, "two in one place");
            }
        }
    }

    #[test]
    fn a_maze_with_no_room_gets_none() {
        let grid = WalkGrid::new(10, 10, crate::survey::CELL_SIZE);
        let mesh = NavMesh::build(&grid, Vector3::zeros(), |_, _| 0);
        assert!(spots(&mesh, Vector3::new(0.5, 0.0, 0.5), 3, &mut Rng::new(1)).is_empty());
    }
}
