//! Heists: in a district of the city, crack the bank's vault and get out with what is in it.
//!
//! A heist map (see `data/arena/heist_aurum.py`) marks where the player comes in, `heist_start`;
//! the firewall doors between them and the vault, `door_1` and on, in the order they open, each
//! with the computer that opens it, `computer_door_1` and on; what there is to take, `loot`; and
//! where to get out with it, `extract_1` and on. Guards keep posts, `guard_1` and on - droids of
//! the side against the player, as capture the flag's red are, who watch for them, hunt them and
//! shoot - and drones patrol round `drone_1` and on. In the vault, `terminal_1` and on mark its
//! terminals; and `wave_1` and on, where security's waves come in from.
//!
//! A firewall door ([`DOOR_MODEL`], made by `data/heist/door.py`) is a doorway's frame with a pane
//! of a firewall's orange glass in it, flickering as a firewall does, that stands in the way until
//! its computer is hacked; then it fades, and lets the player through. Guards have the run of the
//! place, doors or none. Once the last door - the vault's - is down, the vault's terminals are
//! there to hack, each carrying far more credits than a computer elsewhere; and security sends
//! waves of guards after the player, each bigger than the last, with a drone called in each time,
//! and a lull between them. The player steals what they can between the waves, and gets out at an
//! extraction point whenever they like: only then are the credits theirs. Going down loses them.

use crate::{credits::Credits, fixtures::EMISSION_STRENGTH, level::Marker};
use fyrox::{
    asset::manager::ResourceManager,
    core::{
        algebra::{UnitQuaternion, Vector3},
        color::Color,
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{MaterialProperty, MaterialResource},
    resource::model::{Model, ModelResource, ModelResourceExtension},
    scene::{
        base::BaseBuilder,
        collider::{ColliderBuilder, ColliderShape},
        graph::Graph,
        light::{
            point::{PointLight, PointLightBuilder},
            BaseLightBuilder,
        },
        mesh::Mesh,
        node::Node,
        rigidbody::{RigidBodyBuilder, RigidBodyType},
        transform::TransformBuilder,
        Scene,
    },
};

pub const DOOR_MODEL: &str = "data/heist/firewall_door.glb";
/// The pane's part of the door's model, which goes when it opens.
const BARRIER: &str = "firewall_barrier";
/// The doorway, as `data/heist/door.py` makes it: how wide and how tall the way through is, how
/// wide each pillar is, and how deep the frame.
const DOORWAY: (f32, f32) = (2.4, 3.0);
const PILLAR: f32 = 0.4;
const DEPTH: f32 = 0.6;
/// The pane's glass, as a firewall's shell's: its colour, how strongly it colours what is seen
/// through it, and how brightly it glows; its light, how far and how bright.
const COLOUR: Color = Color::opaque(255, 46, 0);
const TINT: f32 = 0.4;
const GLOW: f32 = 0.35;
const LIGHT_RADIUS: f32 = 5.0;
const LIGHT: f32 = 0.8;
/// How long a door takes to fade once its computer is hacked, in seconds.
const FADE: f32 = 0.8;
/// How near an extraction point the player has to come to get out at it, in meters.
pub const EXTRACT_REACH: f32 = 3.0;
/// Security's waves, once the vault is open: how long until the first and between each after, in
/// seconds; how many guards the first is, each after one more, and the most there are in one.
const FIRST_WAVE: f32 = 30.0;
/// How long after a failed hack security's wave comes, in seconds.
const AFTER_FAILED_HACK: f32 = 5.0;
const WAVE_EVERY: f32 = 50.0;
const WAVE_FIRST: usize = 3;
const WAVE_MOST: usize = 8;
/// What each of the vault's terminals carries, in whole credits, at least and at most.
const TERMINAL_CREDITS: (u64, u64) = (300, 900);

/// A door as a heist map marks it: where it is, which way through it, and where its computer goes
/// and which way that faces, if the map says.
pub type DoorMark = (Vector3<f32>, f32, Option<(Vector3<f32>, f32)>);

/// What a heist map marks, read from its markers.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// Where the player comes in, and which way they face.
    pub start: (Vector3<f32>, f32),
    /// Each door, in the order they open: where, which way through it, and where its computer is
    /// marked to go, and which way it faces, if it is.
    pub doors: Vec<DoorMark>,
    /// The vault's middle, and its terminals: where each is marked to go, and which way it faces.
    pub vault: Vector3<f32>,
    pub terminals: Vec<(Vector3<f32>, f32)>,
    pub extracts: Vec<Vector3<f32>>,
    pub guards: Vec<Vector3<f32>>,
    pub drones: Vec<Vector3<f32>>,
    /// Where security's waves come in from.
    pub waves: Vec<Vector3<f32>>,
}

impl Plan {
    /// The heist the `markers` of a map mark out, if they do: a start, the vault and somewhere to
    /// get out, at least.
    pub fn from_markers(markers: &[Marker]) -> Option<Self> {
        let named = |name: &str| markers.iter().find(|m| m.name == name);
        // Those named `prefix` and a number, in its order.
        let numbered = |prefix: &str| {
            let mut found: Vec<(u32, &Marker)> = markers
                .iter()
                .filter_map(|m| Some((m.name.strip_prefix(prefix)?.parse::<u32>().ok()?, m)))
                .collect();
            found.sort_by_key(|&(n, _)| n);
            found
        };
        let start = named("heist_start")?;
        let vault = named("loot")?;
        let extracts: Vec<_> = numbered("extract_").into_iter().map(|(_, m)| m.position).collect();
        if extracts.is_empty() {
            return None;
        }
        let doors = numbered("door_")
            .into_iter()
            .map(|(n, m)| {
                let computer = named(&format!("computer_door_{n}")).map(|c| (c.position, c.yaw));
                (m.position, m.yaw, computer)
            })
            .collect();
        let guards: Vec<_> = numbered("guard_").into_iter().map(|(_, m)| m.position).collect();
        let mut waves: Vec<_> = numbered("wave_").into_iter().map(|(_, m)| m.position).collect();
        if waves.is_empty() {
            waves = guards.clone();
        }
        Some(Plan {
            start: (start.position, start.yaw),
            doors,
            vault: vault.position,
            terminals: numbered("terminal_").into_iter().map(|(_, m)| (m.position, m.yaw)).collect(),
            extracts,
            guards,
            drones: numbered("drone_").into_iter().map(|(_, m)| m.position).collect(),
            waves,
        })
    }

    /// The extraction point nearest `at`.
    pub fn nearest_extract(&self, at: Vector3<f32>) -> Vector3<f32> {
        let away = |e: &Vector3<f32>| flat(e - at).norm();
        self.extracts.iter().copied().min_by(|a, b| away(a).total_cmp(&away(b))).unwrap_or(self.vault)
    }

    /// Whether `at` is at one of the extraction points.
    pub fn at_extract(&self, at: Vector3<f32>) -> bool {
        self.extracts.iter().any(|e| flat(e - at).norm() < EXTRACT_REACH)
    }
}

fn flat(v: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(v.x, 0.0, v.z)
}

/// How far along a heist is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Getting through the doors: so many open of how many.
    Breaking { open: usize, of: usize },
    /// The vault open: its terminals to hack for credits, between security's waves, and out
    /// whenever the player likes.
    Take,
    /// Out, with what they stole.
    Done,
}

/// A heist under way.
#[derive(Debug, Clone, PartialEq)]
pub struct Heist {
    pub plan: Plan,
    pub stage: Stage,
    /// The credits stolen so far - the player's only once they get out with them - and from how
    /// many of the vault's terminals.
    pub stolen: Credits,
    pub hacked: u32,
    /// How many of security's waves have come, and how long until the next, in seconds; and how
    /// long until the one a failed hack brings, if one does.
    pub wave: u32,
    next_wave: f32,
    called: Option<f32>,
    /// How many times the alarm has gone up - droids on Alert after the player - and whether it is
    /// up now.
    pub alarms: u32,
    alarmed: bool,
}

impl Heist {
    pub fn new(plan: Plan) -> Self {
        let stage = Stage::Breaking { open: 0, of: plan.doors.len() };
        let mut heist =
            Heist { plan, stage, stolen: Credits::default(), hacked: 0, wave: 0, next_wave: FIRST_WAVE, called: None, alarms: 0, alarmed: false };
        heist.doors_open(0);
        heist
    }

    /// So many of the doors are open now: the vault, once they all are, and security on its way.
    pub fn doors_open(&mut self, open: usize) {
        if let Stage::Breaking { of, .. } = self.stage {
            self.stage = if open >= of { Stage::Take } else { Stage::Breaking { open, of } };
            if self.stage == Stage::Take {
                self.next_wave = FIRST_WAVE;
            }
        }
    }

    /// A hack has failed: security sends a wave [`AFTER_FAILED_HACK`] seconds from now, at any
    /// stage - unless one is coming sooner already.
    pub fn hack_failed(&mut self) {
        if self.stage != Stage::Done {
            self.called = Some(self.called.map_or(AFTER_FAILED_HACK, |left| left.min(AFTER_FAILED_HACK)));
        }
    }

    /// Moves on `dt` seconds: once the vault is open, security sends a wave now and then, each
    /// bigger than the last; and at any stage, one a failed hack brings. How many guards, if one
    /// is due now.
    pub fn update(&mut self, dt: f32) -> Option<usize> {
        if self.stage == Stage::Done {
            return None;
        }
        let called = self.called.map(|left| left - dt);
        self.called = called.filter(|&left| left > 0.0);
        let due_called = called.is_some_and(|left| left <= 0.0);
        let due_regular = self.stage == Stage::Take && {
            self.next_wave -= dt;
            self.next_wave <= 0.0
        };
        if !(due_called || due_regular) {
            return None;
        }
        self.wave += 1;
        // A called one puts the next off for as long as the lull after any other.
        self.next_wave = WAVE_EVERY;
        Some(wave_size(self.wave))
    }


    /// How long until the next wave, in seconds.
    pub fn next_wave(&self) -> f32 {
        self.next_wave.max(0.0)
    }

    /// Credits transferred from a hacked computer: stolen, to keep once out; from one of the
    /// vault's terminals, if `terminal`.
    pub fn steal(&mut self, credits: Credits, terminal: bool) {
        self.stolen += credits;
        self.hacked += u32::from(terminal);
    }

    /// Whether the droids hunting the player are on Alert now: counts each time it goes up.
    pub fn alarm(&mut self, up: bool) {
        if up && !self.alarmed {
            self.alarms += 1;
        }
        self.alarmed = up;
    }

    /// Gets out, if the vault is open and the player, at `at`, is at an extraction point: whether
    /// they did.
    pub fn extract(&mut self, at: Vector3<f32>) -> bool {
        let out = self.stage == Stage::Take && self.plan.at_extract(at);
        if out {
            self.stage = Stage::Done;
        }
        out
    }

    /// What there is to do next, for the status line.
    pub fn objective(&self) -> String {
        match self.stage {
            _ if self.called.is_some() => format!(
                "Trace complete: wave in {:.0} s",
                self.called.unwrap_or_default().ceil()
            ),
            Stage::Breaking { open, of } if open + 1 == of => {
                format!("Hack the vault door's computer ({}/{of})", open + 1)
            }
            Stage::Breaking { open, of } => format!("Hack the computer for door {} ({}/{of})", open + 1, open + 1),
            Stage::Take => format!(
                "Terminals {}/{}  {} stolen  Wave in {:.0} s",
                self.hacked,
                self.plan.terminals.len(),
                self.stolen,
                self.next_wave().ceil()
            ),
            Stage::Done => "Heist complete".to_string(),
        }
    }
}

/// How many guards security's `wave`th wave is, the first being 1.
pub fn wave_size(wave: u32) -> usize {
    (WAVE_FIRST + wave as usize - 1).min(WAVE_MOST)
}

/// What one of the vault's terminals carries, picked by `below`: far more than a computer
/// elsewhere.
pub fn terminal_credits(mut below: impl FnMut(usize) -> usize) -> Credits {
    let (least, most) = TERMINAL_CREDITS;
    Credits::new(least + below((most - least) as usize) as u64, below(100) as u64)
}

/// The firewall doors of a heist, and their model, as it loads.
#[derive(Debug, Default, PartialEq)]
pub struct Doors {
    model: Option<ModelResource>,
    pub doors: Vec<Door>,
}

#[derive(Debug, PartialEq)]
pub struct Door {
    pub position: Vector3<f32>,
    /// The computer that opens it, in the game's list of computers, once one is given it.
    pub computer: Option<usize>,
    /// Everything it put into the scene; of all that, the pane's meshes and their glass, its
    /// light, and the body that stands in the way while it is up.
    nodes: Vec<Handle<Node>>,
    barrier: Vec<Handle<Node>>,
    glass: MaterialResource,
    light: Handle<Node>,
    wall: Handle<Node>,
    /// Whether it is open, and how far faded, from 1 (up) to 0 (gone).
    open: bool,
    shown: f32,
}

impl Doors {
    pub fn request(resources: &ResourceManager) -> Self {
        Self { model: Some(resources.request::<Model>(DOOR_MODEL)), doors: Vec::new() }
    }

    /// The model, to wait for before the level is put together.
    pub fn models(&self) -> Vec<(String, ModelResource)> {
        self.model.iter().map(|m| (DOOR_MODEL.to_string(), m.clone())).collect()
    }

    /// Puts a door at each of `plan`'s, closed, in place of any from before.
    pub fn place(&mut self, scene: &mut Scene, plan: &Plan) {
        self.clear(&mut scene.graph);
        let Some(model) = self.model.clone().filter(|m| m.is_ok()) else {
            return;
        };
        for &(at, yaw, _) in &plan.doors {
            self.doors.push(Door::spawn(scene, &model, at, yaw));
        }
        Log::info(format!("Heist: {} doors", self.doors.len()));
    }

    /// Takes them all out of the scene.
    pub fn clear(&mut self, graph: &mut Graph) {
        for door in self.doors.drain(..) {
            for node in door.nodes {
                if graph.is_valid_handle(node) {
                    graph.remove_node(node);
                }
            }
        }
    }

    /// Opens each door whose computer is `cleared`, fading it out over [`FADE`] seconds, `dt` of
    /// them on; those still up flicker, `time` seconds in. How many are open.
    pub fn update(&mut self, graph: &mut Graph, cleared: impl Fn(usize) -> bool, time: f32, dt: f32) -> usize {
        for (n, door) in self.doors.iter_mut().enumerate() {
            let open = door.computer.is_none_or(&cleared);
            if open != door.open {
                door.set_open(graph, open);
            }
            door.shown = (door.shown + if open { -dt / FADE } else { dt / FADE }).clamp(0.0, 1.0);
            door.show(graph, flicker(time + n as f32 * 5.1) * door.shown);
        }
        self.doors.iter().filter(|d| d.open).count()
    }
}

impl Door {
    fn spawn(scene: &mut Scene, model: &ModelResource, at: Vector3<f32>, yaw: f32) -> Self {
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), yaw);
        let root = model.instantiate(scene);
        let graph = &mut scene.graph;
        graph[root].local_transform_mut().set_position(at).set_rotation(rotation);
        let barrier: Vec<_> = graph
            .traverse_handle_iter(root)
            .filter(|&node| graph[node].name().starts_with(BARRIER) && graph[node].is_mesh())
            .collect();
        if barrier.is_empty() {
            Log::err(format!("Heist: {DOOR_MODEL} has no {BARRIER}"));
        }
        let glass = fyrox_gfx::GlassMaterial {
            tint: COLOUR,
            tint_strength: TINT,
            emission: COLOUR,
            emission_strength: GLOW,
            // As a firewall's: no bending of what is behind it, close up, and no reflection.
            index_of_refraction: 1.0,
            reflectivity: 0.0,
            ..Default::default()
        }
        .build_resource();
        for &node in &barrier {
            graph[node].set_cast_shadows(false);
            if let Some(mesh) = graph[node].cast_mut::<Mesh>() {
                for surface in mesh.surfaces_mut() {
                    surface.set_material(glass.clone());
                }
            }
        }
        let light = PointLightBuilder::new(
            BaseLightBuilder::new(BaseBuilder::new().with_local_transform(
                TransformBuilder::new().with_local_position(at + Vector3::y() * (DOORWAY.1 * 0.6)).build(),
            ))
            .with_color(COLOUR),
        )
        .with_radius(LIGHT_RADIUS)
        .build(graph)
        .to_base();
        // In the model's own terms: through along x, across along z, up along y.
        let body = |graph: &mut Graph, middle: Vector3<f32>, half: Vector3<f32>| {
            let collider = ColliderBuilder::new(BaseBuilder::new().with_local_transform(
                TransformBuilder::new().with_local_position(middle).build(),
            ))
            .with_shape(ColliderShape::cuboid(half.x, half.y, half.z))
            .build(graph);
            RigidBodyBuilder::new(
                BaseBuilder::new().with_child(collider).with_local_transform(
                    TransformBuilder::new().with_local_position(at).with_local_rotation(rotation).build(),
                ),
            )
            .with_body_type(RigidBodyType::Static)
            .build(graph)
            .to_base()
        };
        // The pillars always stand in the way; the pane only while it is up.
        let (w, h) = (DOORWAY.0 / 2.0, DOORWAY.1);
        let pillars: Vec<Handle<Node>> = [-1.0, 1.0]
            .into_iter()
            .map(|s| {
                body(
                    graph,
                    Vector3::new(0.0, (h + PILLAR) / 2.0, s * (w + PILLAR / 2.0)),
                    Vector3::new(DEPTH / 2.0, (h + PILLAR) / 2.0, PILLAR / 2.0),
                )
            })
            .collect();
        let wall = body(graph, Vector3::new(0.0, h / 2.0, 0.0), Vector3::new(0.15, h / 2.0, w));
        let mut nodes = vec![root, light, wall];
        nodes.extend(pillars);
        Door { position: at, computer: None, nodes, barrier, glass, light, wall, open: false, shown: 1.0 }
    }

    fn set_open(&mut self, graph: &mut Graph, open: bool) {
        self.open = open;
        // Out of the way, far below, rather than gone: it comes back up for the next round.
        let y = if open { -1000.0 } else { 0.0 };
        graph[self.wall]
            .local_transform_mut()
            .set_position(Vector3::new(self.position.x, self.position.y + y, self.position.z));
    }

    /// Its glow and light at `level`, from 0 to 1; out of sight altogether at 0.
    fn show(&self, graph: &mut Graph, level: f32) {
        self.glass.data_ref().set_property(EMISSION_STRENGTH, MaterialProperty::Float(GLOW * level));
        self.glass.data_ref().set_property("tintStrength", MaterialProperty::Float(TINT * level.min(1.0)));
        for &node in &self.barrier {
            graph[node].set_visibility(self.shown > 0.0);
        }
        if let Some(light) = graph[self.light].cast_mut::<PointLight>() {
            light.base_light_mut().set_intensity(LIGHT * level);
        }
        graph[self.light].set_visibility(self.shown > 0.0);
    }
}

/// How brightly a door glows, `time` seconds in, from 0 to 1: as a firewall wavers, with now and
/// then a moment when it nearly goes out.
fn flicker(time: f32) -> f32 {
    let waver = 0.78 + 0.12 * (time * 23.0).sin() + 0.1 * (time * 57.3 + 1.7).sin();
    let tick = (time * 12.0).floor();
    let hash = ((tick * 12.9898).sin() * 43_758.547).rem_euclid(1.0);
    if hash > 0.92 { waver * 0.3 } else { waver }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(name: &str, x: f32, z: f32) -> Marker {
        Marker { name: name.to_string(), position: Vector3::new(x, 0.0, z), yaw: 0.0 }
    }

    fn plan() -> Plan {
        let markers = [
            marker("heist_start", 0.0, 0.0),
            marker("door_2", 20.0, 0.0),
            marker("door_1", 10.0, 0.0),
            marker("computer_door_1", 8.0, 1.0),
            marker("loot", 30.0, 0.0),
            marker("terminal_2", 31.0, 2.0),
            marker("terminal_1", 31.0, -2.0),
            marker("extract_1", 0.0, 50.0),
            marker("extract_2", 50.0, 0.0),
            marker("guard_1", 12.0, 0.0),
            marker("drone_1", 15.0, 0.0),
            marker("wave_1", -5.0, 0.0),
        ];
        Plan::from_markers(&markers).expect("a heist")
    }

    #[test]
    fn a_map_marks_a_heist_its_doors_in_order_each_with_its_computer() {
        let plan = plan();
        assert_eq!(plan.doors.len(), 2);
        assert_eq!(plan.doors[0].0.x, 10.0, "door_1 first");
        assert!(plan.doors[0].2.is_some() && plan.doors[1].2.is_none());
        assert_eq!(plan.terminals[0].0.z, -2.0, "terminal_1 first");
        assert_eq!((plan.extracts.len(), plan.guards.len(), plan.drones.len(), plan.waves.len()), (2, 1, 1, 1));
        assert_eq!(plan.nearest_extract(Vector3::new(40.0, 0.0, 0.0)).x, 50.0);
        // Without a vault, or nowhere to get out, it is no heist.
        assert!(Plan::from_markers(&[marker("heist_start", 0.0, 0.0), marker("loot", 1.0, 0.0)]).is_none());
    }

    #[test]
    fn the_doors_then_the_vault_and_out_any_time_after() {
        let mut heist = Heist::new(plan());
        assert_eq!(heist.stage, Stage::Breaking { open: 0, of: 2 });
        assert!(!heist.extract(Vector3::new(50.0, 0.0, 0.0)), "not before the vault is open");
        heist.doors_open(1);
        assert!(heist.objective().contains("vault door"));
        heist.doors_open(2);
        assert_eq!(heist.stage, Stage::Take);
        heist.steal(Credits::new(400, 0), true);
        heist.steal(Credits::new(5, 0), false);
        assert_eq!((heist.stolen, heist.hacked), (Credits::new(405, 0), 1));
        assert!(!heist.extract(Vector3::new(30.0, 0.0, 0.0)), "not at an extraction point");
        assert!(heist.extract(Vector3::new(51.0, 0.0, 1.0)));
        assert_eq!(heist.stage, Stage::Done);
    }

    #[test]
    fn security_sends_waves_once_the_vault_is_open_each_bigger() {
        let mut heist = Heist::new(plan());
        assert_eq!(heist.update(FIRST_WAVE * 2.0), None, "nothing while the doors are shut");
        heist.doors_open(2);
        assert_eq!(heist.update(FIRST_WAVE - 1.0), None);
        assert_eq!(heist.update(1.5), Some(WAVE_FIRST));
        assert_eq!(heist.update(WAVE_EVERY - 1.0), None, "a lull between them");
        assert_eq!(heist.update(1.5), Some(WAVE_FIRST + 1));
        assert_eq!(wave_size(100), WAVE_MOST, "never more than so many at once");
    }

    #[test]
    fn a_failed_hack_brings_a_wave_five_seconds_on_at_any_stage() {
        let mut heist = Heist::new(plan());
        heist.hack_failed();
        assert!(heist.objective().contains("Trace complete"));
        assert_eq!(heist.update(AFTER_FAILED_HACK - 0.5), None, "not yet");
        heist.hack_failed();
        assert_eq!(heist.update(0.6), Some(WAVE_FIRST), "a second failure does not put it off");
        assert_eq!(heist.update(10.0), None, "and it comes once");
        // In the vault, it puts the next regular one off by a whole lull.
        heist.doors_open(2);
        heist.update(FIRST_WAVE - 2.0);
        heist.hack_failed();
        assert_eq!(heist.update(AFTER_FAILED_HACK), Some(WAVE_FIRST + 1));
        assert_eq!(heist.update(WAVE_EVERY - 1.0), None);
        assert_eq!(heist.update(1.5), Some(WAVE_FIRST + 2));
    }

    #[test]
    fn a_vault_terminal_carries_far_more_than_a_computer_elsewhere() {
        let least = terminal_credits(|_| 0);
        assert!(least >= Credits::new(TERMINAL_CREDITS.0, 0));
        assert!(terminal_credits(|n| n - 1) <= Credits::new(TERMINAL_CREDITS.1, 99));
    }

    #[test]
    fn a_heist_with_no_doors_starts_at_the_open_vault() {
        let markers = [marker("heist_start", 0.0, 0.0), marker("loot", 1.0, 0.0), marker("extract_1", 9.0, 0.0)];
        let heist = Heist::new(Plan::from_markers(&markers).unwrap());
        assert_eq!(heist.stage, Stage::Take);
    }
}
