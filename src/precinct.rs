//! Precincts: where a city's security takes those it arrests (see `data/story/story_urbs.py`).
//!
//! A map marks a precinct with empties: `precinct_<n>` before its desk, where those released are
//! let go; `precinct_<n>_desk` behind it, where a guard keeps; `precinct_<n>_locker`, the evidence
//! locker, where what is taken off those arrested is kept; `precinct_<n>_guard`, the middle of the
//! corridor along the cells, where another keeps; and `cell_<n>_<c>`, the middle of each cell.
//! Each cell is shut by a firewall, a mesh named `cell_wall_<n>_<c>`, and the cell block by
//! another, `precinct_<n>_gate`; the game makes them glow as capture the flag's firewalls do (see
//! [`crate::firewall`]) and drops them when they are opened. The first cell's back wall has a
//! loose panel, `tunnel_panel_<n>`, before which `tunnel_<n>` is, onto a stair down to a tunnel
//! that comes out at `tunnel_<n>_exit`.
//!
//! The firewalls and the panel are taken out of the level's static collider, as a vault's door is
//! (see [`crate::vault`]), each on a body of its own; and out of the droids' grid, so that none of
//! them walks into a cell, or out of the tunnel - but for the gate while it is open.

use crate::{firewall, layout::WalkGrid, level::Marker};
use fyrox::{
    core::{algebra::Vector3, log::Log, pool::Handle},
    graph::SceneGraph,
    material::{MaterialProperty, MaterialResource},
    scene::{
        base::BaseBuilder,
        collider::{Collider, ColliderBuilder, ColliderShape},
        graph::Graph,
        mesh::Mesh,
        node::Node,
        rigidbody::{RigidBodyBuilder, RigidBodyType},
        transform::TransformBuilder,
    },
};

const WALL: &str = "cell_wall_";
const PANEL: &str = "tunnel_panel_";
/// How far round a tunnel's way out the floor is taken out of the grid, in meters.
const EXIT_TAKEN: f32 = 1.3;
/// How far a precinct reaches round its markers, in meters, for whether someone is in it.
const AROUND: f32 = 3.0;

/// A firewall, or the tunnel's panel: in the way until it is opened.
#[derive(Debug, PartialEq)]
pub struct Barrier {
    body: Handle<Node>,
    collider: Handle<Collider>,
    meshes: Vec<Handle<Node>>,
    /// Its extent, shut.
    pub bounds: (Vector3<f32>, Vector3<f32>),
    /// Whether it is a firewall, which goes when opened, or the panel, which slides aside.
    firewall: bool,
    open: bool,
    /// The cells of the grid it took, to give back when it opens.
    taken: Vec<(usize, usize)>,
}

#[derive(Debug, PartialEq)]
pub struct Cell {
    /// The middle of its floor.
    pub middle: Vector3<f32>,
    pub wall: Barrier,
}

#[derive(Debug, PartialEq)]
pub struct Tunnel {
    /// Before the panel, in the cell, and outside where it comes out.
    pub at: Vector3<f32>,
    pub exit: Vector3<f32>,
    pub panel: Barrier,
    /// How far the panel has been pried loose, from 0 to 1.
    pub pried: f32,
}

#[derive(Debug, PartialEq)]
pub struct Precinct {
    pub number: u32,
    pub front: Vector3<f32>,
    pub desk: Vector3<f32>,
    pub locker: Vector3<f32>,
    pub guard: Vector3<f32>,
    pub cells: Vec<Cell>,
    pub gate: Barrier,
    pub tunnel: Option<Tunnel>,
    /// What it covers, from its markers: its least and most corners.
    extent: (Vector3<f32>, Vector3<f32>),
}

#[derive(Debug, Default, PartialEq)]
pub struct Precincts {
    pub precincts: Vec<Precinct>,
    /// The firewalls' glass, all one, to flicker.
    glass: Option<MaterialResource>,
}

impl Barrier {
    /// Hangs `mesh` on a body of its own, in the way as a box of its extent; a firewall glows.
    fn claim(graph: &mut Graph, mesh: Handle<Node>, glass: Option<&MaterialResource>) -> Self {
        let bounds = graph[mesh].world_bounding_box();
        let (min, max) = (bounds.min, bounds.max);
        let centre = (min + max) * 0.5;
        let half = ((max - min) * 0.5).map(|h| h.max(0.02));
        let collider = ColliderBuilder::new(BaseBuilder::new())
            .with_shape(ColliderShape::cuboid(half.x, half.y, half.z))
            .build(graph);
        let name = graph[mesh].name().to_string();
        let body = RigidBodyBuilder::new(
            BaseBuilder::new()
                .with_name(name)
                .with_local_transform(TransformBuilder::new().with_local_position(centre).build())
                .with_child(collider),
        )
        .with_body_type(RigidBodyType::Static)
        .build(graph)
        .to_base();
        graph.update_hierarchical_data();
        graph.link_nodes_keep_global_transform(mesh, body);
        if let Some(glass) = glass {
            graph[mesh].set_cast_shadows(false);
            if let Some(mesh) = graph[mesh].cast_mut::<Mesh>() {
                for surface in mesh.surfaces_mut() {
                    surface.set_material(glass.clone());
                }
            }
        }
        Self { body, collider, meshes: vec![mesh], bounds: (min, max), firewall: glass.is_some(), open: false, taken: Vec::new() }
    }

    pub fn middle(&self) -> Vector3<f32> {
        (self.bounds.0 + self.bounds.1) * 0.5
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// How far `at` is from it along the ground, in meters: 0 inside it.
    pub fn distance(&self, at: Vector3<f32>) -> f32 {
        let (min, max) = self.bounds;
        let dx = (min.x - at.x).max(at.x - max.x).max(0.0);
        let dz = (min.z - at.z).max(at.z - max.z).max(0.0);
        (dx * dx + dz * dz).sqrt()
    }

    /// Opens it or shuts it: a firewall goes or comes back, and the panel slides aside along the
    /// wall it is in, by its own width, or back; and gives the grid back what it took, or takes it
    /// again.
    fn set_open(&mut self, graph: &mut Graph, open: bool, grid: Option<&mut WalkGrid>) {
        self.open = open;
        let (min, max) = self.bounds;
        let centre = (min + max) * 0.5;
        let offset = match (open, self.firewall) {
            (false, _) => Vector3::zeros(),
            // Out of the way, far below, rather than gone: it comes back for the next round.
            (true, true) => Vector3::new(0.0, -1000.0, 0.0),
            (true, false) => {
                let size = max - min;
                if size.x >= size.z { Vector3::new(size.x, 0.0, 0.0) } else { Vector3::new(0.0, 0.0, size.z) }
            }
        };
        graph[self.body].local_transform_mut().set_position(centre + offset);
        if self.firewall {
            for &mesh in &self.meshes {
                graph[mesh].set_visibility(!open);
            }
        }
        if let Some(grid) = grid {
            for &(x, z) in &self.taken {
                grid.set(x, z, open);
            }
        }
    }

    /// Takes the floor under it, and a cell round, out of `grid`, whose corner is at `origin`.
    fn keep_out(&mut self, grid: &mut WalkGrid, origin: Vector3<f32>) {
        let (min, max) = self.bounds;
        let step = grid.cell_size * 0.5;
        let mut x = min.x - grid.cell_size;
        while x <= max.x + grid.cell_size {
            let mut z = min.z - grid.cell_size;
            while z <= max.z + grid.cell_size {
                if let Some((cx, cz)) = grid.cell_at(origin, Vector3::new(x, min.y + 0.1, z)) {
                    if grid.is_walkable(cx, cz) && !self.taken.contains(&(cx, cz)) {
                        grid.set(cx, cz, false);
                        self.taken.push((cx, cz));
                    }
                }
                z += step;
            }
            x += step;
        }
    }
}

/// The `n` and the rest after `prefix` in `name`: `cell_wall_2_3` after `cell_wall_` is (2, "3").
fn numbered<'a>(name: &'a str, prefix: &str) -> Option<(u32, &'a str)> {
    let rest = name.strip_prefix(prefix)?;
    let (n, rest) = rest.split_once('_').unwrap_or((rest, ""));
    Some((n.parse().ok()?, rest))
}

impl Precincts {
    /// Takes each precinct's firewalls and panel out from under `root`, before the level's shape
    /// is taken from what is left, and hangs each on a body of its own; where things are, from
    /// `markers`. The bodies, for the level to remove with it.
    pub fn claim(graph: &mut Graph, root: Handle<Node>, markers: &[Marker]) -> (Self, Vec<Handle<Node>>) {
        let marker = |name: String| markers.iter().find(|m| m.name == name).map(|m| m.position);
        let named: Vec<(Handle<Node>, String)> = graph
            .traverse_handle_iter(root)
            .filter(|&h| graph[h].is_mesh())
            .map(|h| (h, graph[h].name().to_string()))
            .filter(|(_, name)| name.starts_with(WALL) || name.starts_with(PANEL) || name.starts_with("precinct_"))
            .collect();
        let glass = firewall::glass();
        let mut precincts = Vec::new();
        for (gate, name) in &named {
            let Some((n, "gate")) = numbered(name, "precinct_") else {
                continue;
            };
            let (Some(front), Some(desk), Some(locker), Some(guard)) = (
                marker(format!("precinct_{n}")),
                marker(format!("precinct_{n}_desk")),
                marker(format!("precinct_{n}_locker")),
                marker(format!("precinct_{n}_guard")),
            ) else {
                Log::warn(format!("Precinct {n}: its markers are not all there"));
                continue;
            };
            let gate = Barrier::claim(graph, *gate, Some(&glass));
            let mut walls: Vec<(u32, Handle<Node>)> = named
                .iter()
                .filter_map(|(h, name)| match numbered(name, WALL) {
                    Some((m, c)) if m == n => Some((c.parse().ok()?, *h)),
                    _ => None,
                })
                .collect();
            walls.sort_by_key(|&(c, _)| c);
            let cells: Vec<Cell> = walls
                .into_iter()
                .filter_map(|(c, wall)| {
                    let middle = marker(format!("cell_{n}_{c}"))?;
                    Some(Cell { middle, wall: Barrier::claim(graph, wall, Some(&glass)) })
                })
                .collect();
            let tunnel = named
                .iter()
                .find(|(_, name)| numbered(name, PANEL).is_some_and(|(m, _)| m == n))
                .and_then(|&(panel, _)| {
                    let at = marker(format!("tunnel_{n}"))?;
                    let exit = marker(format!("tunnel_{n}_exit"))?;
                    Some(Tunnel { at, exit, panel: Barrier::claim(graph, panel, None), pried: 0.0 })
                });
            let corners = [front, desk, locker, guard].into_iter().chain(cells.iter().map(|c| c.middle));
            let extent = corners.fold((Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN)), |(lo, hi), p| {
                (lo.inf(&p), hi.sup(&p))
            });
            Log::info(format!("Precinct {n}: {} cells{}", cells.len(), if tunnel.is_some() { ", a tunnel" } else { "" }));
            precincts.push(Precinct { number: n, front, desk, locker, guard, cells, gate, tunnel, extent });
        }
        precincts.sort_by_key(|p| p.number);
        let bodies = precincts
            .iter()
            .flat_map(|p| {
                p.cells.iter().map(|c| c.wall.body).chain([p.gate.body]).chain(p.tunnel.as_ref().map(|t| t.panel.body))
            })
            .collect();
        (Self { precincts, glass: Some(glass) }, bodies)
    }

    /// Takes the firewalls, the panels and the tunnels' ways out out of `grid`, whose corner is
    /// at `origin`: no droid walks into a cell, through the gate while it is shut, or into a
    /// tunnel.
    pub fn keep_out(&mut self, grid: &mut WalkGrid, origin: Vector3<f32>) {
        for precinct in &mut self.precincts {
            for cell in &mut precinct.cells {
                cell.wall.keep_out(grid, origin);
            }
            precinct.gate.keep_out(grid, origin);
            if let Some(tunnel) = precinct.tunnel.as_mut() {
                tunnel.panel.keep_out(grid, origin);
                let reach = Vector3::new(EXIT_TAKEN, 0.0, EXIT_TAKEN);
                let (Some(low), Some(high)) = (grid.cell_at(origin, tunnel.exit - reach), grid.cell_at(origin, tunnel.exit + reach))
                else {
                    continue;
                };
                for z in low.1..=high.1 {
                    for x in low.0..=high.0 {
                        let middle = crate::survey::cell_center(origin, x, z);
                        if (middle - tunnel.exit).xz().norm() < EXIT_TAKEN {
                            grid.set(x, z, false);
                        }
                    }
                }
            }
        }
    }

    /// Shuts everything again, the panels back in place, for a new round.
    pub fn reset(&mut self, graph: &mut Graph, mut grid: Option<&mut WalkGrid>) {
        for precinct in &mut self.precincts {
            for cell in &mut precinct.cells {
                cell.wall.set_open(graph, false, grid.as_deref_mut());
            }
            precinct.gate.set_open(graph, false, grid.as_deref_mut());
            if let Some(tunnel) = precinct.tunnel.as_mut() {
                tunnel.panel.set_open(graph, false, grid.as_deref_mut());
                tunnel.pried = 0.0;
            }
        }
    }

    /// Opens the `c`th cell of the `p`th precinct, or shuts it.
    pub fn set_cell(&mut self, graph: &mut Graph, (p, c): (usize, usize), open: bool, grid: Option<&mut WalkGrid>) {
        if let Some(cell) = self.precincts.get_mut(p).and_then(|p| p.cells.get_mut(c)) {
            cell.wall.set_open(graph, open, grid);
        }
    }

    /// Opens the `p`th precinct's gate, or shuts it.
    pub fn set_gate(&mut self, graph: &mut Graph, p: usize, open: bool, grid: Option<&mut WalkGrid>) {
        if let Some(precinct) = self.precincts.get_mut(p) {
            precinct.gate.set_open(graph, open, grid);
        }
    }

    /// Opens the `p`th precinct's tunnel, its panel slid aside.
    pub fn open_tunnel(&mut self, graph: &mut Graph, p: usize) {
        if let Some(tunnel) = self.precincts.get_mut(p).and_then(|p| p.tunnel.as_mut()) {
            tunnel.pried = 1.0;
            tunnel.panel.set_open(graph, true, None);
        }
    }

    /// The firewalls' glow, `time` seconds in, as capture the flag's flicker.
    pub fn flicker(&self, time: f32) {
        if let Some(glass) = &self.glass {
            glass
                .data_ref()
                .set_property(crate::fixtures::EMISSION_STRENGTH, MaterialProperty::Float(firewall::GLOW * firewall::flicker(time)));
        }
    }

    /// The `p`th precinct's firewalls' bodies: they are seen through.
    pub fn glass(&self, p: usize) -> Vec<Handle<Collider>> {
        self.precincts
            .get(p)
            .map(|precinct| precinct.cells.iter().map(|c| c.wall.collider).chain([precinct.gate.collider]).collect())
            .unwrap_or_default()
    }

    /// The precinct nearest `at`, if there is one.
    pub fn nearest(&self, at: Vector3<f32>) -> Option<usize> {
        (0..self.precincts.len()).min_by(|&a, &b| {
            let off = |p: usize| (self.precincts[p].front - at).norm();
            off(a).total_cmp(&off(b))
        })
    }

    /// Whether `at` is in the `p`th precinct: within [`AROUND`] of what its markers cover, and
    /// near the height of its floor.
    pub fn inside(&self, p: usize, at: Vector3<f32>) -> bool {
        self.precincts.get(p).is_some_and(|precinct| {
            let (lo, hi) = precinct.extent;
            at.x > lo.x - AROUND
                && at.x < hi.x + AROUND
                && at.z > lo.z - AROUND
                && at.z < hi.z + AROUND
                && at.y > lo.y - 1.0
                && at.y < hi.y + 2.5
        })
    }

    /// Whether `at` is in the `c`th cell of the `p`th precinct: behind its firewall, on its floor.
    pub fn in_cell(&self, (p, c): (usize, usize), at: Vector3<f32>) -> bool {
        let Some(cell) = self.precincts.get(p).and_then(|p| p.cells.get(c)) else {
            return false;
        };
        let wall = cell.wall.middle();
        let inward = (cell.middle - wall).xz();
        let off = (at - wall).xz();
        let depth = inward.norm() * 2.0;
        let along = off.dot(&inward) / inward.norm().max(1.0e-3);
        let across = (off - inward.normalize() * along).norm();
        along > 0.0 && along < depth && across < 1.5 && (at.y - cell.middle.y).abs() < 1.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_precincts_pieces_are_numbered_by_their_names() {
        assert_eq!(numbered("cell_wall_2_3", WALL), Some((2, "3")));
        assert_eq!(numbered("precinct_1_gate", "precinct_"), Some((1, "gate")));
        assert_eq!(numbered("tunnel_panel_3", PANEL), Some((3, "")));
        assert_eq!(numbered("precinct_x_gate", "precinct_"), None);
    }
}
