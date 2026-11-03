//! Credit chips: credits lying about in Story mode, to be picked up by walking into them. Each is
//! a hexagonal token on its edge ([`CHIP_MODEL`], made in Blender by
//! ~/Documents/blender/collectibles/credit_chip/build_credit_chip.py: a graphite frame round a
//! glass face, traces of light out from a raised CR, gold contacts round its edge), spinning and
//! bobbing over the floor, its lights the colour of what it is worth - cyan for a few credits,
//! gold for more, violet for a good deal - and bigger the more. Until the model has loaded, or
//! without it, a plain six-sided coin stands in for it.
//! Picked up, a chip's credits go straight into the player's, as a hacked computer's do: there is
//! nothing to carry, and no limit to how many.
//!
//! The city's own lie about everywhere there is floor to walk on - streets, rooftops, upper
//! storeys - a few in every [`CHUNK`]-meter square, always in the same places, worked out from
//! the square and the navmesh as the player comes near it ([`Chips::update`]); only those near
//! the player are in the scene. Taken, one stays taken until a new game, however many times the
//! round starts again.
//!
//! And more are dropped ([`Chips::drop_credits`]): a droid that goes down scatters what it was
//! carrying where it fell ([`Credits::purse`]) - if it was carrying any, as most are
//! ([`CARRIES`]); the player, deleted or arrested, drops half of
//! theirs where it happened ([`DROPPED`]), for them to go back for. Those lie there until they are
//! picked up, the next round too.

use crate::{
    credits::Credits,
    layout::Rng,
};
use hydroxus_ai::navigation::{grid::MAX_CLIMB, navmesh::NavMesh, walk};
use fyrox::{
    core::{
        algebra::{Matrix4, UnitQuaternion, Vector3},
        color::Color,
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{Material, MaterialProperty, MaterialResource},
    resource::model::ModelResource,
    resource::texture::{Texture, TextureKind, TexturePixelKind, TextureResource},
    scene::{
        base::BaseBuilder,
        graph::Graph,
        mesh::{
            surface::{Surface, SurfaceBuilder, SurfaceData, SurfaceResource},
            Mesh, MeshBuilder,
        },
        node::Node,
        sound::{SoundBufferResource, SoundBuilder, Status},
        transform::TransformBuilder,
    },
};
use std::collections::{HashMap, HashSet};

/// The chip's model: one mesh, its lights one material, `ChipGlow`, recoloured here by worth.
pub const CHIP_MODEL: &str = "data/credit_chip.glb";
/// How big the model is shown at its smallest, of how big it was made (0.26 m across).
const MODEL_SCALE: f32 = 0.85;
/// Where the chips' sound is described.
pub const CHIP_SOUNDS: &str = "data/sounds/credits_formants.json";
/// How big a square of the city has its own chips, in meters; how far from the player the city's
/// chips, and dropped ones, are in the scene.
pub const CHUNK: f32 = 48.0;
const SHOWN_WITHIN: f32 = 75.0;
/// How many chips a square has, by chance: none, one, two or three - one and a bit on average.
const IN_A_SQUARE: [(usize, usize); 4] = [(0, 20), (1, 40), (2, 30), (3, 10)];
/// How many cells a square tries for each chip, to find one with floor all round.
const TRIES: usize = 12;
/// How likely a droid is to have been carrying credits, to drop where it goes down, in hundredths:
/// the rest carried none.
pub const CARRIES: usize = 60;
/// Whether a droid that went down was carrying credits, for `roll`, a number from 0 to 99.
pub fn carries(roll: usize) -> bool {
    roll < CARRIES
}
/// How much of what they carry the player drops, deleted or arrested; and into how many chips at
/// most anything dropped scatters, how far round where it fell.
pub const DROPPED: f64 = 0.5;
const MOST_PIECES: usize = 5;
const SCATTER: f32 = 1.2;
/// How high a chip floats over the floor, how far it bobs and how fast it bobs and spins; how near
/// the player's middle it has to be to be picked up, across and up or down.
const HOVER: f32 = 0.55;
const BOB: f32 = 0.06;
const BOB_TIME: f32 = 1.6;
const TURN_TIME: f32 = 2.2;
const TAKE_REACH: f32 = 0.9;
const TAKE_HEIGHT: f32 = 1.4;
/// A chip's size, across and thick, at its smallest; how much bigger each worth is; and how
/// brightly it glows.
const SIZE: (f32, f32) = (0.22, 0.035);
const BIGGER: f32 = 1.3;
const GLOW: f32 = 2.5;
/// What it takes for a chip to be gold, and violet, in credits; and their colours, cyan first.
const GOLD_FROM: Credits = Credits::new(5, 0);
const VIOLET_FROM: Credits = Credits::new(25, 0);
const COLOURS: [Color; 3] = [Color::opaque(0, 217, 255), Color::opaque(255, 190, 25), Color::opaque(155, 50, 255)];

/// Which of the chips' worths `value` is: 0 cyan, 1 gold, 2 violet.
pub fn worth(value: Credits) -> usize {
    if value >= VIOLET_FROM {
        2
    } else if value >= GOLD_FROM {
        1
    } else {
        0
    }
}

/// `total` split into chips: as many as it is worth a credit each, no more than [`MOST_PIECES`],
/// at least one; the hundredths to the first.
pub fn pieces(total: Credits) -> Vec<Credits> {
    let n = (total.whole() as usize).clamp(1, MOST_PIECES);
    let each = total.0 / n as u64;
    let mut split = vec![Credits(each); n];
    split[0] = Credits(total.0 - each * (n as u64 - 1));
    split.retain(|c| c.0 > 0);
    split
}

/// Which chip: one of the city's, by its square and its number there, or one dropped, by its
/// number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    City(i64, i64, usize),
    Dropped(u64),
}

/// A chip in the scene.
#[derive(Debug, Clone, PartialEq)]
struct Shown {
    key: Key,
    node: Handle<Node>,
    at: Vector3<f32>,
    value: Credits,
    phase: f32,
}

/// The chips: the city's, square by square, which the player has taken, the dropped ones, and
/// those in the scene now.
#[derive(Debug, Default)]
pub struct Chips {
    /// Each square's chips, where and what each is worth, once worked out.
    squares: HashMap<(i64, i64), Vec<(Vector3<f32>, Credits)>>,
    taken: HashSet<Key>,
    dropped: Vec<(u64, Vector3<f32>, Credits)>,
    next_dropped: u64,
    shown: Vec<Shown>,
    /// The square the player was in when the chips were last brought in and out of the scene.
    around: Option<(i64, i64)>,
    time: f32,
    /// The model's surfaces for each worth, its lights in that worth's colour, once it has
    /// loaded; and whether the chips in the scene are to be made again from it.
    model: Option<[Vec<Surface>; 3]>,
    remake: bool,
    /// The plain coin's shape, each worth's material, and the sound of one picked up.
    shape: Option<SurfaceResource>,
    materials: Vec<MaterialResource>,
    sound: Option<(SoundBufferResource, f32)>,
    /// A number the city's squares are worked out from, the same for each square every time.
    seed: u64,
    /// Whether one of each worth has been put down in front of the player, with MAZE_CHIPS.
    shown_off: bool,
}

impl PartialEq for Chips {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Chips {
    /// Chips for a new game, from scratch: none taken, none dropped, the city's squares worked out
    /// afresh from `seed`. Whatever was in `graph` taken out of it.
    pub fn reset(&mut self, graph: &mut Graph, seed: u64) {
        self.clear_scene(graph);
        let (shape, materials, sound, model) = (self.shape.take(), std::mem::take(&mut self.materials), self.sound.take(), self.model.take());
        *self = Self { shape, materials, sound, model, seed, ..Default::default() };
    }

    /// Takes the chips out of the scene - for a new round - keeping which are taken and dropped,
    /// to come back as the player comes near them.
    pub fn clear_scene(&mut self, graph: &mut Graph) {
        for shown in self.shown.drain(..) {
            if graph.is_valid_handle(shown.node) {
                graph.remove_node(shown.node);
            }
        }
        self.around = None;
    }

    /// `total` dropped at `at`, on the floor, scattered round it in chips by `rng`.
    pub fn drop_credits(&mut self, at: Vector3<f32>, total: Credits, rng: &mut Rng) {
        if total.0 == 0 {
            return;
        }
        for value in pieces(total) {
            let angle = rng.below(360) as f32 * std::f32::consts::PI / 180.0;
            let out = SCATTER * (0.3 + 0.7 * rng.below(100) as f32 / 100.0);
            let spot = at + Vector3::new(angle.cos() * out, HOVER, angle.sin() * out);
            self.dropped.push((self.next_dropped, spot, value));
            self.next_dropped += 1;
        }
        // To be brought in, if the player is near.
        self.around = None;
    }

    /// With MAZE_CHIPS, to look at them: once a game, one of each worth - 2, 12 and 40 credits -
    /// put down in a row 2.5 m in front of the player, standing at `feet` facing `yaw`, a meter
    /// apart, the cheapest on their left.
    pub fn show_off(&mut self, feet: Vector3<f32>, yaw: f32) {
        if std::mem::replace(&mut self.shown_off, true) {
            return;
        }
        let ahead = Vector3::new(yaw.sin(), 0.0, yaw.cos());
        let left = Vector3::new(ahead.z, 0.0, -ahead.x);
        for (k, whole) in [2, 12, 40].into_iter().enumerate() {
            let at = feet + ahead * 2.5 + left * (1.0 - k as f32) + Vector3::new(0.0, HOVER, 0.0);
            self.dropped.push((self.next_dropped, at, Credits::new(whole, 0)));
            self.next_dropped += 1;
        }
        self.around = None;
        Log::info("Credit chips: one of each worth put down in front of the player (MAZE_CHIPS)");
    }

    /// The chips a square of the city has: where, on the floor of `mesh`, and what each is worth.
    /// The same every time for the same square.
    fn square(&mut self, mesh: &NavMesh, (sx, sz): (i64, i64)) -> &Vec<(Vector3<f32>, Credits)> {
        let seed = self.seed;
        self.squares.entry((sx, sz)).or_insert_with(|| {
            let mut rng = Rng::new(seed ^ (sx as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (sz as u64).wrapping_mul(0xc2b2_ae3d_27d4_eb4f));
            let roll = rng.below(100);
            let mut count = 0;
            let mut acc = 0;
            for &(n, chance) in &IN_A_SQUARE {
                acc += chance;
                if roll < acc {
                    count = n;
                    break;
                }
            }
            let cell = mesh.cell_size;
            let per = (CHUNK / cell).round() as i64;
            let (low, high) = mesh.bounds();
            let mut chips = Vec::new();
            for _ in 0..count {
                for _ in 0..TRIES {
                    let x = sx * per + rng.below(per as usize) as i64;
                    let z = sz * per + rng.below(per as usize) as i64;
                    let spot = low + Vector3::new((x as f32 + 0.5) * cell, 0.0, (z as f32 + 0.5) * cell);
                    if x < 0 || z < 0 || spot.x >= high.x || spot.z >= high.z {
                        continue;
                    }
                    let over = mesh.over_spot(spot);
                    if over.is_empty() {
                        continue;
                    }
                    let n = over[rng.below(over.len())];
                    let spot = Vector3::new(spot.x, mesh.floor_of(n), spot.z);
                    // Floor all round it, a step either way at most: out in the open, not in a
                    // corner or on a ledge.
                    let open = (-1..=1).all(|dx| {
                        (-1..=1).all(|dz| walk::floor_at(mesh, spot + Vector3::new(dx as f32, 0.0, dz as f32) * cell, MAX_CLIMB))
                    });
                    if !open {
                        continue;
                    }
                    let at = spot + Vector3::new(0.0, HOVER, 0.0);
                    chips.push((at, Credits::chip(|n| rng.below(n))));
                    break;
                }
            }
            chips
        })
    }

    /// The shape and materials, made the first time there is a chip to show.
    fn assets(&mut self) -> SurfaceResource {
        if self.materials.is_empty() {
            let white = Texture::from_bytes(TextureKind::Rectangle { width: 1, height: 1 }, TexturePixelKind::RGBA8, vec![255; 4])
                .map(TextureResource::new_embedded);
            self.materials = COLOURS
                .iter()
                .map(|&colour| {
                    let mut material = Material::standard();
                    material.set_property("diffuseColor", colour);
                    material.set_property("emissionStrength", Vector3::repeat(GLOW));
                    material.bind("emissionTexture", white.clone());
                    MaterialResource::new_embedded(material)
                })
                .collect();
        }
        self.shape
            .get_or_insert_with(|| {
                // A six-sided coin on its edge: a flat cylinder turned onto its side.
                let stand = Matrix4::from_axis_angle(&Vector3::x_axis(), std::f32::consts::FRAC_PI_2);
                SurfaceResource::new_embedded(SurfaceData::make_cylinder(6, SIZE.0 / 2.0, SIZE.1, true, &stand))
            })
            .clone()
    }

    /// Whether there is no sound for a chip picked up yet.
    pub fn sound_unset(&self) -> bool {
        self.sound.is_none()
    }

    /// Whether the chips are made from the model yet.
    pub fn has_model(&self) -> bool {
        self.model.is_some()
    }

    /// Makes the chips from `model`, its mesh's surfaces, from now on - those in the scene made
    /// again - each worth's lights in its colour: a copy of whatever material of it glows.
    pub fn set_model(&mut self, model: &ModelResource) {
        let surfaces: Vec<Surface> = {
            let data = model.data_ref();
            let graph = &data.get_scene().graph;
            graph.linear_iter().filter_map(|node| node.cast::<Mesh>()).flat_map(|mesh| mesh.surfaces().to_vec()).collect()
        };
        if surfaces.is_empty() {
            Log::warn(format!("Credit chips: {CHIP_MODEL} has no mesh; plain coins it is"));
            return;
        }
        let tiers = COLOURS.map(|colour| {
            surfaces
                .iter()
                .cloned()
                .map(|mut surface| {
                    let recoloured = {
                        let state = surface.material().state();
                        state.data_ref().and_then(|material| {
                            // The glTF importer gives a glow's strength per channel - the model's
                            // own cyan, times its strength - and the engine scales it by the
                            // surface's colour: made even, as strong as its strongest channel,
                            // so that the colour is the worth's alone.
                            let strength = match crate::fixtures::glow_strength(material)? {
                                MaterialProperty::Vector3(v) => v.max(),
                                MaterialProperty::Float(f) => f,
                                _ => GLOW,
                            };
                            let mut copy = material.clone();
                            copy.set_property("diffuseColor", colour);
                            copy.set_property(crate::fixtures::EMISSION_STRENGTH, Vector3::repeat(strength));
                            Some(MaterialResource::new_embedded(copy))
                        })
                    };
                    if let Some(material) = recoloured {
                        surface.set_material(material);
                    }
                    surface
                })
                .collect()
        });
        self.model = Some(tiers);
        self.remake = true;
        Log::info(format!("Credit chips: made from {CHIP_MODEL}, {} surfaces", surfaces.len()));
    }

    /// Sets the sound a chip makes picked up, and how far off it is heard at full volume.
    pub fn set_sound(&mut self, sound: Option<(SoundBufferResource, f32)>) {
        self.sound = sound;
    }

    /// The chips on `dt` seconds, the player at `player` - their middle - on `grid`: those near
    /// brought into the scene and those gone far taken out, each spinning and bobbing, and the
    /// ones the player walks into picked up - the city's own among them, if it is a `city`, or only
    /// those dropped. What those picked up were worth, all told, if any were.
    pub fn update(&mut self, graph: &mut Graph, dt: f32, player: Vector3<f32>, mesh: &NavMesh, city: bool) -> Option<Credits> {
        self.time += dt;
        if std::mem::take(&mut self.remake) {
            self.clear_scene(graph);
        }
        let here = ((player.x - mesh.origin.x) / CHUNK).floor() as i64;
        let here = (here, ((player.z - mesh.origin.z) / CHUNK).floor() as i64);
        if self.around != Some(here) {
            self.around = Some(here);
            self.bring_in(graph, player, mesh, here, city);
        }
        // Spinning and bobbing.
        for shown in &self.shown {
            let bob = BOB * ((self.time / BOB_TIME + shown.phase) * std::f32::consts::TAU).sin();
            let turn = (self.time / TURN_TIME + shown.phase) * std::f32::consts::TAU;
            if let Ok(node) = graph.try_get_mut(shown.node) {
                node.local_transform_mut()
                    .set_position(shown.at + Vector3::new(0.0, bob, 0.0))
                    .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), turn));
            }
        }
        // Picked up.
        let mut got = Credits::default();
        let mut k = 0;
        while k < self.shown.len() {
            let at = self.shown[k].at;
            let across = Vector3::new(at.x - player.x, 0.0, at.z - player.z).norm();
            if across < TAKE_REACH && (at.y - player.y).abs() < TAKE_HEIGHT {
                let shown = self.shown.swap_remove(k);
                got += shown.value;
                match shown.key {
                    Key::City(..) => {
                        self.taken.insert(shown.key);
                    }
                    Key::Dropped(id) => self.dropped.retain(|&(had, ..)| had != id),
                }
                if graph.is_valid_handle(shown.node) {
                    graph.remove_node(shown.node);
                }
            } else {
                k += 1;
            }
        }
        if got.0 == 0 {
            return None;
        }
        if let Some((buffer, reach)) = &self.sound {
            SoundBuilder::new(BaseBuilder::new().with_local_transform(TransformBuilder::new().with_local_position(player).build()))
                .with_buffer(Some(buffer.clone()))
                .with_radius(*reach)
                .with_play_once(true)
                .with_status(Status::Playing)
                .build(graph);
        }
        Some(got)
    }

    /// Brings into the scene the chips within [`SHOWN_WITHIN`] of `player`, in square `here`, that
    /// are not taken, and takes out those further off.
    fn bring_in(&mut self, graph: &mut Graph, player: Vector3<f32>, mesh: &NavMesh, here: (i64, i64), city: bool) {
        let near = |at: Vector3<f32>| Vector3::new(at.x - player.x, 0.0, at.z - player.z).norm() < SHOWN_WITHIN;
        // Out of it, the far ones.
        let mut k = 0;
        while k < self.shown.len() {
            if near(self.shown[k].at) {
                k += 1;
                continue;
            }
            let shown = self.shown.swap_remove(k);
            if graph.is_valid_handle(shown.node) {
                graph.remove_node(shown.node);
            }
        }
        // Into it, the near ones not in it yet.
        let reach = if city { (SHOWN_WITHIN / CHUNK).ceil() as i64 } else { -1 };
        let mut wanted: Vec<(Key, Vector3<f32>, Credits)> = Vec::new();
        for sx in here.0 - reach..=here.0 + reach {
            for sz in here.1 - reach..=here.1 + reach {
                let chips = self.square(mesh, (sx, sz)).clone();
                for (n, (at, value)) in chips.into_iter().enumerate() {
                    wanted.push((Key::City(sx, sz, n), at, value));
                }
            }
        }
        wanted.extend(self.dropped.iter().map(|&(id, at, value)| (Key::Dropped(id), at, value)));
        let shape = self.assets();
        for (key, at, value) in wanted {
            if !near(at) || self.taken.contains(&key) || self.shown.iter().any(|shown| shown.key == key) {
                continue;
            }
            let w = worth(value);
            let (surfaces, scale) = match &self.model {
                Some(tiers) => (tiers[w].clone(), MODEL_SCALE * BIGGER.powi(w as i32)),
                None => (vec![SurfaceBuilder::new(shape.clone()).with_material(self.materials[w].clone()).build()], BIGGER.powi(w as i32)),
            };
            let node = MeshBuilder::new(
                BaseBuilder::new()
                    .with_cast_shadows(false)
                    .with_local_transform(TransformBuilder::new().with_local_position(at).with_local_scale(Vector3::repeat(scale)).build()),
            )
            .with_surfaces(surfaces)
            .build(graph)
            .to_base();
            let phase = (at.x * 0.37 + at.z * 0.61).rem_euclid(1.0);
            self.shown.push(Shown { key, node, at, value, phase });
        }
        Log::info(format!("Credit chips: {} about the player, {} dropped about the city", self.shown.len(), self.dropped.len()));
    }

    /// Shows only the chips `can_see` says could be seen from where the player is.
    pub fn cull(&self, graph: &mut Graph, can_see: impl Fn(Vector3<f32>) -> bool) {
        for shown in &self.shown {
            if let Ok(node) = graph.try_get_mut(shown.node) {
                node.set_visibility(can_see(shown.at));
            }
        }
    }

    /// How many chips are in the scene, and how many have been dropped and lie about yet.
    #[cfg(test)]
    pub fn counts(&self) -> (usize, usize) {
        (self.shown.len(), self.dropped.len())
    }
}

/// Logs what the player picked up.
pub fn log_pickup(got: Credits, total: Credits) {
    Log::info(format!("Credits: +{got} in chips, {total} in all"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::WalkGrid;

    /// An open plain of floor, `cells` across, half a meter each.
    fn plain(cells: usize) -> NavMesh {
        let mut grid = WalkGrid::new(cells, cells, 0.5);
        for x in 0..cells {
            for z in 0..cells {
                grid.set(x, z, true);
            }
        }
        NavMesh::build(&grid, Vector3::zeros(), |_, _| 0)
    }

    #[test]
    fn dropped_credits_split_into_a_few_chips_that_add_up() {
        for total in [Credits(37), Credits::new(3, 50), Credits::new(120, 99)] {
            let split = pieces(total);
            assert!(!split.is_empty() && split.len() <= MOST_PIECES);
            assert_eq!(split.iter().map(|c| c.0).sum::<u64>(), total.0);
        }
        assert_eq!(pieces(Credits::new(3, 50)).len(), 3);
    }

    #[test]
    fn some_droids_carry_credits_and_some_do_not() {
        let carrying = (0..100).filter(|&roll| carries(roll)).count();
        assert_eq!(carrying, CARRIES);
        assert!(carrying > 0 && carrying < 100);
    }

    #[test]
    fn a_chip_glows_by_what_it_is_worth() {
        assert_eq!(worth(Credits::new(1, 0)), 0);
        assert_eq!(worth(Credits::new(5, 0)), 1);
        assert_eq!(worth(Credits::new(30, 0)), 2);
    }

    #[test]
    fn the_citys_chips_are_on_its_floor_in_the_same_places_every_time() {
        let grid = plain(400);
        let mut chips = Chips { seed: 7, ..Default::default() };
        let mut again = Chips { seed: 7, ..Default::default() };
        let mut all = 0;
        for sx in 0..4 {
            for sz in 0..4 {
                let first = chips.square(&grid, (sx, sz)).clone();
                assert_eq!(&first, again.square(&grid, (sx, sz)));
                for (at, value) in &first {
                    // In its square, over the floor.
                    assert!(at.x >= sx as f32 * CHUNK && at.x < (sx + 1) as f32 * CHUNK, "{at:?}");
                    assert!((at.y - HOVER).abs() < 1e-4 && value.0 > 0);
                }
                all += first.len();
            }
        }
        // One and a bit a square, on average.
        assert!((8..=40).contains(&all), "{all}");
        // Off the grid, none.
        assert!(chips.square(&grid, (-1, 0)).is_empty());
    }

    #[test]
    fn walking_into_a_chip_picks_it_up_once_and_it_stays_taken() {
        let grid = plain(400);
        let mut graph = Graph::new();
        let mut chips = Chips { seed: 3, ..Default::default() };
        let mut rng = Rng::new(1);
        chips.drop_credits(Vector3::new(50.0, 0.0, 50.0), Credits::new(2, 0), &mut rng);
        // Far off, nothing.
        let far = Vector3::new(10.0, 1.0, 10.0);
        let got = chips.update(&mut graph, 0.1, far, &grid, true);
        let city_got = got.unwrap_or_default();
        // Over to each dropped chip in turn.
        let spots: Vec<Vector3<f32>> = chips.dropped.iter().map(|&(_, at, _)| at).collect();
        assert_eq!(spots.len(), 2);
        let mut picked = Credits::default();
        for at in spots {
            picked += chips.update(&mut graph, 0.1, at, &grid, true).unwrap_or_default();
        }
        assert!(picked >= Credits::new(2, 0), "{picked} {city_got}");
        assert!(chips.dropped.is_empty());
        // A city chip taken stays taken, round after round.
        let (shown, _) = chips.counts();
        if let Some(first) = chips.shown.first().cloned() {
            let before = shown;
            chips.update(&mut graph, 0.1, first.at, &grid, true);
            assert!(chips.taken.contains(&first.key));
            chips.clear_scene(&mut graph);
            chips.update(&mut graph, 0.1, first.at + Vector3::new(0.0, 0.0, 0.01), &grid, true);
            assert!(chips.shown.iter().all(|shown| shown.key != first.key));
            assert!(chips.shown.len() < before + 1);
        }
    }
}
