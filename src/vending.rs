//! Vending machines: each sells one of the power-ups the Mutators sell (see [`crate::powerups`]),
//! at its own price, to anyone with the credits - a machine thinks nothing of the player. Walk up
//! to one, face it, and E buys one.
//!
//! The model, [`VENDING_MODEL`], is made in Blender by
//! `~/Documents/blender/street/vending/build_vending.py`: three machines, `vending_repair`,
//! `vending_stamina` and `vending_shield`, each with its glass, `vending_<kind>_glass`, under it.
//! A map marks where one stands with `vending_<kind>_<n>`: its back there, facing the marker's
//! +x. Each is put into the level before its shape is taken, so that it is solid and surveyed
//! round, and its glow put out with the lights.
//!
//! The engine takes a glTF surface for solid whatever its alpha, so the machines' glass - the one
//! see-through material in the model - is made glass here, as the hearts' outer layer is.

use crate::{credits::Credits, fixtures::{property, DIFFUSE_COLOR}, level::Marker, powerups::PowerUp};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        color::Color,
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{Material, MaterialProperty},
    resource::model::{ModelResource, ModelResourceExtension},
    scene::{node::Node, Scene},
};
use fyrox_gfx::{replace_materials, GlassMaterial};

/// The machines' model.
pub const VENDING_MODEL: &str = "data/vending.glb";
/// How far in front of a machine's face the player can be to use it, in meters, and how far to
/// either side of its middle; and how nearly they have to face it, as the cosine of the angle off.
const REACH: f32 = 1.4;
const ASIDE: f32 = 0.6;
const FACING: f32 = 0.5;
/// How deep a machine is, from its back to its face, in meters (`DEPTH` in the build script).
const DEPTH: f32 = 0.85;

/// What a power-up costs from a machine: as a Mutator asks for it before what it thinks of the
/// player comes into it.
pub fn price(power: PowerUp) -> Credits {
    match power {
        PowerUp::Repair => Credits::new(40, 0),
        PowerUp::Stamina => Credits::new(30, 0),
        PowerUp::Shield => Credits::new(60, 0),
    }
}

/// What a machine selling `power` is called after, in the model and on its marker.
fn name_of(power: PowerUp) -> &'static str {
    match power {
        PowerUp::Repair => "repair",
        PowerUp::Stamina => "stamina",
        PowerUp::Shield => "shield",
    }
}

/// The kind a machine's name says it sells.
fn kind(name: &str) -> Option<PowerUp> {
    match name {
        "repair" => Some(PowerUp::Repair),
        "stamina" => Some(PowerUp::Stamina),
        "shield" => Some(PowerUp::Shield),
        _ => None,
    }
}

/// The machine a marker named `name` marks, if it marks one: what it sells.
pub fn of_marker(name: &str) -> Option<PowerUp> {
    let rest = name.strip_prefix("vending_")?;
    let (sells, number) = rest.split_once('_')?;
    number.parse::<u32>().ok()?;
    kind(sells)
}

#[derive(Debug, PartialEq)]
struct Machine {
    sells: PowerUp,
    root: Handle<Node>,
    /// The middle of its face, on the floor, and which way it faces.
    face: Vector3<f32>,
    ahead: Vector3<f32>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Vending {
    machines: Vec<Machine>,
}

impl Vending {
    /// Puts a machine under `root` where each of `markers` marks one, of the kind it says.
    pub fn place(scene: &mut Scene, root: Handle<Node>, model: &ModelResource, markers: &[Marker]) -> Self {
        let mut machines = Vec::new();
        for marker in markers {
            let Some(sells) = of_marker(&marker.name) else {
                continue;
            };
            let instance = model.instantiate(scene);
            let graph = &mut scene.graph;
            // Only the one it sells, brought back to the model's middle: the others away.
            let Some((kept, _)) = graph.find_by_name(instance, &format!("vending_{}", name_of(sells))) else {
                Log::err(format!("Vending: {VENDING_MODEL} has no machine for {}", name_of(sells)));
                graph.remove_node(instance);
                continue;
            };
            for other in [PowerUp::Repair, PowerUp::Stamina, PowerUp::Shield].into_iter().filter(|&p| p != sells) {
                if let Some((handle, _)) = graph.find_by_name(instance, &format!("vending_{}", name_of(other))) {
                    graph.remove_node(handle);
                }
            }
            graph[kept].local_transform_mut().set_position(Vector3::zeros());
            // The model's +Z along the marker's +x.
            let ahead = Vector3::new(marker.yaw.cos(), 0.0, -marker.yaw.sin());
            graph[instance]
                .local_transform_mut()
                .set_position(marker.position)
                .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), ahead.x.atan2(ahead.z)));
            graph.link_nodes(instance, root);
            Self::glaze(scene, instance);
            machines.push(Machine { sells, root: instance, face: marker.position + ahead * DEPTH, ahead });
        }
        if !machines.is_empty() {
            Log::info(format!("Vending: {} machines", machines.len()));
        }
        Self { machines }
    }

    /// Its see-through material made glass, of its own colour.
    fn glaze(scene: &mut Scene, root: Handle<Node>) {
        let see_through = |material: &Material| matches!(
            property(material, DIFFUSE_COLOR),
            Some(MaterialProperty::Color(colour)) if colour.a < 255
        );
        let glass = GlassMaterial { tint: Color::opaque(140, 178, 204), tint_strength: 0.18, ..Default::default() }.build_resource();
        replace_materials(&mut scene.graph, root, see_through, &glass);
    }

    /// The machine the player at `feet`, looking along `ahead`, is at, if one: which, and what it
    /// sells.
    pub fn at(&self, feet: Vector3<f32>, ahead: Vector3<f32>) -> Option<(usize, PowerUp)> {
        let flat = |v: Vector3<f32>| Vector3::new(v.x, 0.0, v.z);
        let looking = flat(ahead).try_normalize(1.0e-3)?;
        self.machines.iter().enumerate().find_map(|(n, m)| {
            let off = flat(feet - m.face);
            let out = off.dot(&m.ahead);
            let aside = (off - m.ahead * out).norm();
            let facing = -looking.dot(&m.ahead);
            (out > 0.0 && out < REACH && aside < ASIDE && facing > FACING).then_some((n, m.sells))
        })
    }

    /// Where each stands, its face's middle on the floor: for the townsfolk to go and use.
    pub fn faces(&self) -> Vec<Vector3<f32>> {
        self.machines.iter().map(|m| m.face + m.ahead * 0.8).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_marker_says_what_its_machine_sells() {
        assert_eq!(of_marker("vending_repair_1"), Some(PowerUp::Repair));
        assert_eq!(of_marker("vending_shield_12"), Some(PowerUp::Shield));
        assert_eq!(of_marker("vending_stamina_x"), None);
        assert_eq!(of_marker("vending_snacks_1"), None);
        assert_eq!(of_marker("store_1"), None);
    }

    #[test]
    fn it_is_used_from_in_front_facing_it() {
        let v = Vending {
            machines: vec![Machine {
                sells: PowerUp::Stamina,
                root: Handle::NONE,
                face: Vector3::new(0.0, 0.0, 1.0),
                ahead: Vector3::new(0.0, 0.0, 1.0),
            }],
        };
        let at = |x: f32, z: f32, look_z: f32| v.at(Vector3::new(x, 0.0, z), Vector3::new(0.0, 0.0, look_z));
        assert_eq!(at(0.0, 1.8, -1.0), Some((0, PowerUp::Stamina)));
        assert_eq!(at(0.0, 1.8, 1.0), None, "back to it");
        assert_eq!(at(0.0, 3.0, -1.0), None, "too far");
        assert_eq!(at(1.0, 1.8, -1.0), None, "off to the side");
        assert_eq!(at(0.0, 0.5, -1.0), None, "behind its face");
        assert!(price(PowerUp::Shield) > price(PowerUp::Repair));
    }
}
