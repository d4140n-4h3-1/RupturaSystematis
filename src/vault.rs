//! Banks' vaults: a strongroom of gold at the back of each bank in a city, behind a door that one
//! of the bank's computers opens (see [`crate::computer::Kind::Bank`]).
//!
//! A map marks a vault's door with a mesh named `vault_door_<n>`, and anything that swings open
//! with it with meshes named after it, `vault_door_<n>_wheel` say; and the middle of the
//! strongroom behind it with an empty, `vault_<n>`. The bank's computer that opens it is marked
//! `bank_<n>_vault`. The door is taken out of the level's static collider and hung on a body of
//! its own, at its hinge, which turns to swing it open into the strongroom once it is opened from
//! that computer, its collider going with it. Walking into the strongroom takes what is in it.
//!
//! The survey of the level counts only the level's own collider, so the doorway is taken out of
//! the droids' grid by hand, open or shut: nobody but the player goes in.

use crate::{layout::WalkGrid, level::Marker};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        log::Log,
        pool::Handle,
    },
    graph::SceneGraph,
    scene::{
        base::BaseBuilder,
        collider::{ColliderBuilder, ColliderShape},
        graph::Graph,
        node::Node,
        rigidbody::{RigidBodyBuilder, RigidBodyType},
        transform::TransformBuilder,
    },
};

/// What a map names a vault's door, and the empty in the middle of its strongroom.
const DOOR: &str = "vault_door_";
const MIDDLE: &str = "vault_";
/// How far the door swings open, in radians, and how long it takes, in seconds.
const SWING: f32 = 1.75;
const OPENING: f32 = 3.0;
/// How near the middle of the strongroom the player's feet have to come to take what is in it,
/// across and up, in meters.
const TAKE_WITHIN: (f32, f32) = (1.6, 1.5);

#[derive(Debug, Default, PartialEq)]
pub struct Vaults {
    pub vaults: Vec<Vault>,
}

#[derive(Debug, PartialEq)]
pub struct Vault {
    /// Which bank's it is, as the map numbers them.
    pub bank: u32,
    /// The computer that opens it, in the game's list of computers, once one is given it.
    pub computer: Option<usize>,
    /// The body the door hangs on, at its hinge, and which way it turns to swing into the
    /// strongroom; the door's own extent, for the grid.
    body: Handle<Node>,
    hinge: Vector3<f32>,
    turn: f32,
    bounds: (Vector3<f32>, Vector3<f32>),
    /// The middle of the strongroom, if the map marks it.
    middle: Option<Vector3<f32>>,
    /// Whether it has been opened, how far open it is from 0 to 1, and whether what is in it has
    /// been taken.
    open: bool,
    swung: f32,
    pub taken: bool,
}

impl Vaults {
    /// Takes each vault door out from under `root`, before the level's shape is taken from what
    /// is left, and hangs it on a body of its own; the strongrooms' middles from `markers`. The
    /// bodies, for the level to remove with it.
    pub fn claim(graph: &mut Graph, root: Handle<Node>, markers: &[Marker]) -> (Self, Vec<Handle<Node>>) {
        let named: Vec<(Handle<Node>, String)> = graph
            .traverse_handle_iter(root)
            .filter(|&h| graph[h].name().starts_with(DOOR) && graph[h].is_mesh())
            .map(|h| (h, graph[h].name().to_string()))
            .collect();
        let mut vaults = Vec::new();
        for (door, name) in &named {
            let Some(bank) = name.strip_prefix(DOOR).and_then(|n| n.parse::<u32>().ok()) else {
                continue;
            };
            let parts: Vec<Handle<Node>> = named
                .iter()
                .filter(|(_, other)| other.strip_prefix(name.as_str()).is_some_and(|rest| rest.starts_with('_')))
                .map(|(h, _)| *h)
                .collect();
            let bounds = graph[*door].world_bounding_box();
            let (min, max) = (bounds.min, bounds.max);
            let middle = markers
                .iter()
                .find(|m| m.name.strip_prefix(MIDDLE) == Some(bank.to_string().as_str()))
                .map(|m| m.position);
            // Hung by one end of its width, turning about the upright there.
            let along_x = max.x - min.x >= max.z - min.z;
            let centre = (min + max) * 0.5;
            let hinge = if along_x {
                Vector3::new(min.x, min.y, centre.z)
            } else {
                Vector3::new(centre.x, min.y, min.z)
            };
            // Turned the way that brings its other end toward the strongroom.
            let free = if along_x { Vector3::new(max.x, min.y, centre.z) } else { Vector3::new(centre.x, min.y, max.z) };
            let turn = match middle {
                Some(middle) => {
                    let swung = |sign: f32| {
                        UnitQuaternion::from_axis_angle(&Vector3::y_axis(), sign * SWING) * (free - hinge) + hinge
                    };
                    if (swung(1.0) - middle).xz().norm() < (swung(-1.0) - middle).xz().norm() { 1.0 } else { -1.0 }
                }
                None => 1.0,
            };
            let half = (max - min) * 0.5;
            let collider = ColliderBuilder::new(BaseBuilder::new().with_local_transform(
                TransformBuilder::new().with_local_position(centre - hinge).build(),
            ))
            .with_shape(ColliderShape::cuboid(half.x, half.y, half.z))
            .build(graph);
            let body = RigidBodyBuilder::new(
                BaseBuilder::new()
                    .with_name(name.as_str())
                    .with_local_transform(TransformBuilder::new().with_local_position(hinge).build())
                    .with_child(collider),
            )
            .with_body_type(RigidBodyType::Static)
            .build(graph)
            .to_base();
            graph.update_hierarchical_data();
            graph.link_nodes_keep_global_transform(*door, body);
            for &part in &parts {
                graph.link_nodes_keep_global_transform(part, body);
            }
            if middle.is_none() {
                Log::warn(format!("Maze: {name} has no {MIDDLE}{bank} in its strongroom"));
            }
            vaults.push(Vault {
                bank,
                computer: None,
                body,
                hinge,
                turn,
                bounds: (min, max),
                middle,
                open: false,
                swung: 0.0,
                taken: false,
            });
        }
        if !vaults.is_empty() {
            Log::info(format!("Maze: {} vaults", vaults.len()));
        }
        let bodies = vaults.iter().map(|vault| vault.body).collect();
        (Self { vaults }, bodies)
    }

    /// Takes each doorway out of `grid`, whose corner is at `origin`.
    pub fn keep_out(&self, grid: &mut WalkGrid, origin: Vector3<f32>) {
        for vault in &self.vaults {
            let (min, max) = vault.bounds;
            let step = grid.cell_size * 0.5;
            let mut x = min.x - grid.cell_size;
            while x <= max.x + grid.cell_size {
                let mut z = min.z - grid.cell_size;
                while z <= max.z + grid.cell_size {
                    if let Some((cx, cz)) = grid.cell_at(origin, Vector3::new(x, min.y + 0.1, z)) {
                        grid.set(cx, cz, false);
                    }
                    z += step;
                }
                x += step;
            }
        }
    }

    /// Shuts every door again, with what is in each strongroom back in it, for a new round.
    pub fn reset(&mut self, graph: &mut Graph) {
        for vault in &mut self.vaults {
            vault.open = false;
            vault.swung = 0.0;
            vault.taken = false;
            vault.computer = None;
            vault.pose(graph);
        }
    }

    /// Opens the vault that the `computer`th computer opens. Where its door is, if there is one.
    pub fn open_from(&mut self, computer: usize) -> Option<Vector3<f32>> {
        let vault = self.vaults.iter_mut().find(|vault| vault.computer == Some(computer))?;
        vault.open = true;
        Log::info(format!("Vault {}: opened", vault.bank));
        Some(vault.hinge)
    }

    /// The middle of the first strongroom, if any: for trying them out.
    pub fn first_middle(&self) -> Option<Vector3<f32>> {
        self.vaults.iter().find_map(|vault| vault.middle)
    }

    /// Swings each door on that has been opened, `dt` seconds on; and, with the player's `feet`
    /// in a strongroom that is open, what is in it is taken: where that was, the first time.
    pub fn update(&mut self, graph: &mut Graph, feet: Vector3<f32>, dt: f32) -> Option<Vector3<f32>> {
        let mut taken = None;
        for vault in &mut self.vaults {
            if vault.open && vault.swung < 1.0 {
                vault.swung = (vault.swung + dt / OPENING).min(1.0);
                vault.pose(graph);
            }
            if let Some(middle) = vault.middle.filter(|_| vault.open && !vault.taken) {
                let off = feet - middle;
                if off.xz().norm() < TAKE_WITHIN.0 && off.y.abs() < TAKE_WITHIN.1 {
                    vault.taken = true;
                    taken = Some(middle);
                    Log::info(format!("Vault {}: emptied", vault.bank));
                }
            }
        }
        taken
    }
}

impl Vault {
    /// Puts the door where it is now, as far open as it has swung: slowly at first, as a heavy
    /// door starts, and slowly at the last.
    fn pose(&self, graph: &mut Graph) {
        let eased = self.swung * self.swung * (3.0 - 2.0 * self.swung);
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.turn * SWING * eased);
        graph[self.body].local_transform_mut().set_rotation(rotation);
    }
}

/// Which bank's vault a computer's marker, `bank_<n>_vault`, says it opens, if any.
pub fn opened_by(marker: &str) -> Option<u32> {
    marker.strip_prefix("bank_")?.strip_suffix("_vault")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_banks_vault_computer_is_named_for_its_bank() {
        assert_eq!(opened_by("bank_3_vault"), Some(3));
        assert_eq!(opened_by("bank_3_2"), None);
        assert_eq!(opened_by("store_3"), None);
    }
}
