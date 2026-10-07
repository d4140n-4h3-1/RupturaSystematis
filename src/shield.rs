//! The shield: a power-up the player raises with the 1 key. A see-through egg of glass closes
//! round their droid, two rings of light round it, and stops what would hit them - droids' bolts,
//! drones' shots, a droid catching them - but not the closing ring of battle royale.
//!
//! It has health of its own, [`STRENGTH`] hits, and its glow shows how much is left: cyan when it
//! is whole, yellow by half, red when one more hit will break it ([`colour_of`]). It flashes as it
//! takes each. Once raised it stays up until it breaks - there is no taking it down early - and
//! only then does it start to charge again, over [`RECHARGE`] seconds: so it cannot be raised and
//! let go again and again. While it is up, the health bar is its, blue, named SHIELD ACTIVE.
//!
//! The model, [`SHIELD_MODEL`], is made in Blender by `data/shield.py`. The engine takes a glTF
//! surface for solid whatever its alpha, so the see-through shell is made glass here, as the
//! hearts are; and its rings are given materials of their own, to colour as its health goes.

use crate::fixtures::{property, DIFFUSE_COLOR};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        color::Color,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{Material, MaterialProperty, MaterialResource},
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{graph::Graph, mesh::Mesh, node::Node, Scene},
};
use fyrox_gfx::GlassMaterial;

pub const SHIELD_MODEL: &str = "data/shield.glb";

/// How long the shield takes to charge again once it has broken, in seconds.
pub const RECHARGE: f32 = 18.0;
/// How many hits it stops before it breaks.
pub const STRENGTH: u32 = 4;
/// How many spare charges - shield cells picked up - can be carried at once.
pub const MOST_SPARES: u32 = 2;
/// How long it takes to grow round the droid, and to fade away, in seconds.
const GROW: f32 = 0.25;
/// How long a hit makes it flash, in seconds.
const FLASH: f32 = 0.3;
/// How brightly the glass glows - only faintly, to stay see-through - and the rings, as a share
/// of how they were made to; and how much brighter a hit makes them.
const GLASS_GLOW: f32 = 0.12;
const RING_GLOW: f32 = 4.0;
const FLASH_GLOW: f32 = 3.0;
/// How far it breathes in and out while it is up, as a share of its glow, and how fast, per second.
const PULSE: (f32, f32) = (0.15, 1.6);
/// Its colour whole, at half its health, and about to break.
const WHOLE: Color = Color::opaque(60, 215, 255);
const HALF: Color = Color::opaque(255, 210, 40);
const BREAKING: Color = Color::opaque(255, 40, 40);

/// What the shield is doing.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Charge {
    /// Charged, ready to raise.
    #[default]
    Ready,
    /// Up, with so many hits it can still take; it stays up until they are gone.
    Up { health: u32 },
    /// Charging again, with so long `left`.
    Recharging { left: f32 },
}

/// What happened when the shield was hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Took {
    /// It was not up: the hit goes through.
    Nothing,
    /// It stopped the hit.
    Stopped,
    /// It stopped the hit, and broke.
    Broke,
}

impl Charge {
    /// Raises it, if it is charged: whether it did.
    pub fn raise(&mut self) -> bool {
        let ready = *self == Charge::Ready;
        if ready {
            *self = Charge::Up { health: STRENGTH };
        }
        ready
    }

    /// Takes a hit, if it is up.
    pub fn take(&mut self) -> Took {
        match self {
            Charge::Up { health } if *health > 1 => {
                *health -= 1;
                Took::Stopped
            }
            Charge::Up { .. } => {
                *self = Charge::Recharging { left: RECHARGE };
                Took::Broke
            }
            _ => Took::Nothing,
        }
    }

    /// Moves on `dt` seconds: broken, it charges again. Up, it stays up, however long.
    pub fn update(&mut self, dt: f32) {
        *self = match *self {
            Charge::Recharging { left } if left <= dt => Charge::Ready,
            Charge::Recharging { left } => Charge::Recharging { left: left - dt },
            other => other,
        };
    }

    pub fn is_up(&self) -> bool {
        matches!(self, Charge::Up { .. })
    }

    /// How much of its health it has, from 0 to 1; whole while it is not up.
    pub fn health(&self) -> f32 {
        match self {
            Charge::Up { health } => *health as f32 / STRENGTH as f32,
            _ => 1.0,
        }
    }
}

/// The egg's size, as `data/shield.py` makes it: how tall, from the feet, and how wide.
const EGG: (f32, f32) = (2.3, 1.7);

/// Whether a point `from_feet` away from the droid's feet is inside the egg round it, or near
/// enough its shell to be as good as.
fn inside_egg(from_feet: Vector3<f32>) -> bool {
    let (height, width) = EGG;
    let (half_up, half_across) = (height / 2.0, width / 2.0);
    let across = (from_feet.x * from_feet.x + from_feet.z * from_feet.z).sqrt() / half_across;
    let up = (from_feet.y - half_up) / half_up;
    across * across + up * up < 1.15
}

/// The shield's colour with `health` left, from 1 (whole) to 0: cyan, through yellow, to red.
pub fn colour_of(health: f32) -> Color {
    let mix = |a: Color, b: Color, t: f32| {
        let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
        Color::opaque(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b))
    };
    let health = health.clamp(0.0, 1.0);
    // A hit short of breaking is red already.
    let left = ((health * STRENGTH as f32 - 1.0) / (STRENGTH as f32 - 1.0)).clamp(0.0, 1.0);
    if left >= 0.5 {
        mix(HALF, WHOLE, (left - 0.5) * 2.0)
    } else {
        mix(BREAKING, HALF, left * 2.0)
    }
}

/// The shield round the player's droid: its charge, and the egg it shows as.
#[derive(Debug, Default, PartialEq)]
pub struct Shield {
    pub charge: Charge,
    /// Spare charges, from shield cells picked up: each raises it at once, charging or not.
    pub spares: u32,
    root: Handle<Node>,
    /// The shell's glass, and the rings' materials, its own to colour; and the rings, hidden
    /// while the camera is inside it, where they would cut across the view.
    glass: Option<MaterialResource>,
    rings: Vec<MaterialResource>,
    ring_nodes: Vec<Handle<Node>>,
    /// How far grown round the droid it is, 0 to 1; how long is left of a hit's flash, and of
    /// its breaking; and how long it has been up, for its pulse.
    grown: f32,
    flash: f32,
    time: f32,
}

impl Shield {
    /// The shield from `model`, in `scene`, out of sight until it is raised.
    pub fn place(scene: &mut Scene, model: &ModelResource) -> Self {
        let root = model.instantiate(scene);
        let graph = &mut scene.graph;
        graph[root].set_visibility(false);
        let mut shield = Shield { root, ..Default::default() };
        let see_through = |material: &Material| {
            matches!(property(material, DIFFUSE_COLOR), Some(MaterialProperty::Color(colour)) if colour.a < 255)
        };
        let handles: Vec<Handle<Node>> = graph.traverse_handle_iter(root).collect();
        for handle in handles {
            let Some(mesh) = graph[handle].cast_mut::<Mesh>() else {
                continue;
            };
            // It casts no shadow, or it would put the droid in the dark.
            mesh.set_cast_shadows(false);
            for surface in mesh.surfaces_mut() {
                let original = surface.material().clone();
                let glassy = original.state().data_ref().is_some_and(see_through);
                if glassy {
                    let glass = shield.glass.get_or_insert_with(|| {
                        GlassMaterial {
                            tint: WHOLE,
                            tint_strength: 0.12,
                            index_of_refraction: 1.05,
                            distortion: 0.05,
                            reflectivity: 0.6,
                            emission: WHOLE,
                            emission_strength: GLASS_GLOW,
                            // Light goes through it as if it were not there.
                            tints_light: false,
                            ..Default::default()
                        }
                        .build_resource()
                    });
                    surface.set_material(glass.clone());
                } else {
                    let copy = original.state().data_ref().map(|m| MaterialResource::new_embedded(m.clone()));
                    if let Some(copy) = copy {
                        surface.set_material(copy.clone());
                        shield.rings.push(copy);
                        if !shield.ring_nodes.contains(&handle) {
                            shield.ring_nodes.push(handle);
                        }
                    }
                }
            }
        }
        shield
    }

    /// Raises it, if it is charged, or with a spare charge if it is still charging: whether it
    /// did. Up already, it stays as it is.
    pub fn raise(&mut self) -> bool {
        let mut raised = self.charge.raise();
        if !raised && matches!(self.charge, Charge::Recharging { .. }) && self.spares > 0 {
            self.spares -= 1;
            self.charge = Charge::Up { health: STRENGTH };
            raised = true;
        }
        if raised {
            self.time = 0.0;
        }
        raised
    }

    /// Takes a shield cell as a spare charge, if there is room for one: whether there was.
    pub fn add_spare(&mut self) -> bool {
        let room = self.spares < MOST_SPARES;
        if room {
            self.spares += 1;
        }
        room
    }

    /// Takes a hit, if it is up, flashing as it does.
    pub fn take(&mut self) -> Took {
        let took = self.charge.take();
        if took != Took::Nothing {
            self.flash = FLASH;
        }
        took
    }

    /// Charged and down again, for a new round, or the player back in after going down.
    pub fn reset(&mut self, graph: &mut Graph) {
        self.charge = Charge::Ready;
        self.spares = 0;
        self.grown = 0.0;
        self.flash = 0.0;
        if let Ok(node) = graph.try_get_mut(self.root) {
            node.set_visibility(false);
        }
    }

    /// Moves it on `dt` seconds, round the droid standing at `feet` facing `yaw`, seen from
    /// `eye`: growing as it is raised, shrinking away as it goes down, its colour that of its
    /// health.
    pub fn update(&mut self, graph: &mut Graph, dt: f32, feet: Vector3<f32>, yaw: f32, eye: Vector3<f32>) {
        self.charge.update(dt);
        self.time += dt;
        self.flash = (self.flash - dt).max(0.0);
        let up = self.charge.is_up();
        self.grown = (self.grown + if up { dt / GROW } else { -dt / GROW }).clamp(0.0, 1.0);
        let Ok(node) = graph.try_get_mut(self.root) else {
            return;
        };
        let showing = self.grown > 0.0;
        node.set_visibility(showing);
        if !showing {
            return;
        }
        // Grows out from the middle, eased.
        let size = 1.0 - (1.0 - self.grown).powi(3);
        node.local_transform_mut()
            .set_position(feet)
            .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), yaw))
            .set_scale(Vector3::new(size, size.max(0.3), size));
        // From inside it - aiming over the shoulder - the glass is out of sight, but the rings
        // would cut across the view: they go.
        let inside = inside_egg(eye - feet);
        for &ring in &self.ring_nodes {
            if let Ok(ring) = graph.try_get_mut(ring) {
                ring.set_visibility(!inside);
            }
        }
        // Its health's colour, breathing, and bright for a moment as it is hit; faded with it
        // as it grows and goes.
        let colour = colour_of(self.charge.health());
        let pulse = 1.0 + PULSE.0 * (self.time * PULSE.1 * std::f32::consts::TAU).sin();
        let flash = 1.0 + FLASH_GLOW * self.flash / FLASH;
        let glow = pulse * flash * self.grown;
        if let Some(glass) = &self.glass {
            let mut glass = glass.data_ref();
            glass.set_property("tint", colour);
            glass.set_property("emission", colour);
            glass.set_property("emissionStrength", GLASS_GLOW * glow);
        }
        let light = Vector3::new(colour.r as f32, colour.g as f32, colour.b as f32) / 255.0;
        for ring in &self.rings {
            let mut ring = ring.data_ref();
            // The engine lights what a surface gives off by its own colour: the same as its glow.
            ring.set_property(DIFFUSE_COLOR, colour);
            ring.set_property("emissionStrength", light * (RING_GLOW * glow));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raised_it_stops_hits_until_it_breaks_then_charges_again() {
        let mut charge = Charge::Ready;
        assert_eq!(charge.take(), Took::Nothing, "not up, it stops nothing");
        assert!(charge.raise());
        assert!(!charge.raise(), "raised once at a time");
        for _ in 1..STRENGTH {
            assert_eq!(charge.take(), Took::Stopped);
        }
        assert_eq!(charge.take(), Took::Broke);
        assert_eq!(charge, Charge::Recharging { left: RECHARGE });
        assert!(!charge.raise(), "not while charging");
        charge.update(RECHARGE + 0.01);
        assert_eq!(charge, Charge::Ready);
    }

    #[test]
    fn it_stays_up_until_it_breaks_and_only_then_charges() {
        let mut charge = Charge::Ready;
        charge.raise();
        charge.update(RECHARGE * 10.0);
        assert!(charge.is_up(), "no running out: only breaking takes it down");
        assert!(!charge.raise(), "and no raising it afresh while it is up");
        charge.take();
        charge.update(RECHARGE * 10.0);
        assert_eq!(charge, Charge::Up { health: STRENGTH - 1 }, "worn down, it does not mend");
    }

    #[test]
    fn a_spare_raises_it_again_while_it_charges_but_not_while_it_is_up() {
        let mut shield = Shield::default();
        assert!(shield.add_spare() && shield.add_spare());
        assert!(!shield.add_spare(), "no room for more than {MOST_SPARES}");
        assert!(shield.raise());
        assert_eq!(shield.spares, MOST_SPARES, "charged, the charge is used, not a spare");
        assert!(!shield.raise(), "up, it stays as it is");
        assert_eq!(shield.spares, MOST_SPARES);
        while shield.take() != Took::Broke {}
        assert!(shield.raise(), "broken and charging, a spare raises it at once");
        assert_eq!((shield.spares, shield.charge), (MOST_SPARES - 1, Charge::Up { health: STRENGTH }));
    }

    #[test]
    fn the_camera_over_the_shoulder_is_inside_it_and_behind_it_is_not() {
        assert!(inside_egg(Vector3::new(0.4, 1.6, -0.5)), "aiming over the shoulder");
        assert!(!inside_egg(Vector3::new(0.0, 2.0, -3.0)), "held back behind");
        assert!(!inside_egg(Vector3::new(0.0, 2.8, 0.0)), "above it");
    }

    #[test]
    fn its_colour_goes_from_cyan_through_yellow_to_red_as_it_is_worn_down() {
        assert_eq!(colour_of(1.0), WHOLE);
        assert_eq!(colour_of(1.0 / STRENGTH as f32), BREAKING, "a hit from breaking, red");
        let mid = colour_of(0.5 + 0.5 / STRENGTH as f32);
        assert!(mid.r > 200 && mid.g > 150, "yellow between: {mid:?}");
        // Redder with every hit.
        let reds: Vec<u8> = (1..=STRENGTH).map(|h| colour_of(h as f32 / STRENGTH as f32).r).collect();
        assert!(reds.windows(2).all(|w| w[0] >= w[1]), "{reds:?}");
    }
}
