//! The sniper's sights: its scope, and its laser.
//!
//! Raised to the eye in first person, the sniper's scope fills the view: the view zooms in
//! [`SCOPE_ZOOM`] times, the gun is put out of sight, and in its place is the scope's own view - a
//! round window in black, with a violet reticle across it. The mouse turns the view that much
//! more finely, to match.
//!
//! Its laser sight is on whenever it is out: a thin violet beam from the laser on its handguard,
//! down the barrel's line, to the first thing in the way, with a glowing dot there that lights
//! what it is on. Through the scope only the dot shows, in the middle of the reticle.

use super::{
    pistol::{Weapon, SNIPER_COLOR, SNIPER_VIOLET},
    viewmodel::{SCOPE_IN, SCOPE_ZOOM},
    Player,
};
use fyrox::{
    core::{
        algebra::{Matrix4, UnitQuaternion, Vector2, Vector3},
        color::Color,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{Material, MaterialResource},
    resource::texture::{Texture, TextureKind, TexturePixelKind, TextureResource},
    scene::{
        base::BaseBuilder,
        camera::{Camera, Projection},
        graph::Graph,
        light::{point::PointLightBuilder, BaseLightBuilder},
        mesh::{
            surface::{SurfaceBuilder, SurfaceData, SurfaceResource},
            vertex::StaticVertex,
            MeshBuilder,
        },
        node::Node,
        pivot::PivotBuilder,
    },
    utils::raw_mesh::RawMeshBuilder,
};

/// How far the laser reaches, in meters.
const LASER_RANGE: f32 = 200.0;
/// How thick its beam is, and how big its dot, in meters; how brightly they glow; and how far
/// the dot's light reaches, in meters, and how bright it is.
const BEAM_WIDTH: f32 = 0.004;
const DOT_SIZE: f32 = 0.025;
const LASER_GLOW: f32 = 6.0;
const DOT_REACH: f32 = 0.6;
const DOT_BRIGHTNESS: f32 = 1.5;
/// How far in front of the eye the scope's view is drawn, in meters: past the near plane.
const SCOPE_DISTANCE: f32 = 0.12;
/// How much of the height of the view the scope's window takes up.
const SCOPE_WINDOW: f32 = 0.94;
/// How far up to aim the sniper is, from 0 to 1, as the view starts to zoom in; all the way in
/// at [`SCOPE_IN`].
const ZOOM_FROM: f32 = 0.55;
/// How thick the reticle's lines are, and the gap in their middle, as shares of the window's
/// radius.
const RETICLE_LINE: f32 = 0.006;
const RETICLE_GAP: f32 = 0.06;

/// The scope's view and the laser, made once with the player.
#[derive(Debug, Default, Clone, PartialEq)]
pub(super) struct Sight {
    /// The scope's black, with its window, and its reticle, carried by the camera.
    scope: Handle<Node>,
    /// The laser's beam, drawn out along its +Z, one meter long before it is scaled; and its dot.
    beam: Handle<Node>,
    dot: Handle<Node>,
}

/// A material that glows violet all over, `strength` times as bright as its colour.
fn violet(strength: f32) -> MaterialResource {
    let white = Texture::from_bytes(
        TextureKind::Rectangle {
            width: 1,
            height: 1,
        },
        TexturePixelKind::RGBA8,
        vec![255; 4],
    )
    .map(TextureResource::new_embedded);
    let mut material = Material::standard();
    material.set_property("diffuseColor", SNIPER_COLOR);
    material.set_property("emissionStrength", SNIPER_VIOLET * strength);
    material.bind("emissionTexture", white);
    MaterialResource::new_embedded(material)
}

/// Flat black.
fn black() -> MaterialResource {
    let mut material = Material::standard();
    material.set_property("diffuseColor", Color::opaque(0, 0, 0));
    MaterialResource::new_embedded(material)
}

/// Flat quads facing -Z (back toward the eye, which looks down +Z in the camera's terms), each
/// given by its four corners round in the XY plane.
fn quads(quads: &[[Vector2<f32>; 4]]) -> SurfaceData {
    let mut builder = RawMeshBuilder::<StaticVertex>::new(quads.len() * 6, quads.len() * 6);
    let normal = -Vector3::z();
    for quad in quads {
        for k in [0, 1, 2, 0, 2, 3] {
            let at = Vector3::new(quad[k].x, quad[k].y, 0.0);
            builder.insert(StaticVertex::from_pos_uv_normal(at, quad[k], normal));
        }
    }
    let mut data = SurfaceData::from_raw_mesh(builder.build());
    data.calculate_tangents().ok();
    data
}

/// The scope's black: everything from a circle of radius 1 out to well past the edges of any
/// screen.
fn scope_black() -> SurfaceData {
    const SIDES: usize = 64;
    const FAR: f32 = 40.0;
    let at = |k: usize, r: f32| {
        let a = std::f32::consts::TAU * k as f32 / SIDES as f32;
        Vector2::new(a.cos(), a.sin()) * r
    };
    let rings: Vec<[Vector2<f32>; 4]> = (0..SIDES)
        .map(|k| [at(k, 1.0), at(k, FAR), at(k + 1, FAR), at(k + 1, 1.0)])
        .collect();
    quads(&rings)
}

/// The scope's reticle in a window of radius 1: a cross with a gap in its middle, and small
/// marks along it.
fn reticle() -> SurfaceData {
    let (w, gap) = (RETICLE_LINE, RETICLE_GAP);
    let bar = |x0: f32, y0: f32, x1: f32, y1: f32| {
        [Vector2::new(x0, y0), Vector2::new(x1, y0), Vector2::new(x1, y1), Vector2::new(x0, y1)]
    };
    let mut parts = vec![
        bar(gap, -w, 1.0, w),
        bar(-1.0, -w, -gap, w),
        bar(-w, gap, w, 1.0),
        bar(-w, -1.0, w, -gap),
        // The middle, a dot.
        bar(-1.5 * w, -1.5 * w, 1.5 * w, 1.5 * w),
    ];
    for k in 1..=4 {
        let at = gap + 0.15 * k as f32;
        let tick = 0.025;
        parts.push(bar(at - w, -tick, at + w, tick));
        parts.push(bar(-at - w, -tick, -at + w, tick));
        parts.push(bar(-tick, -at - w, tick, -at + w));
    }
    quads(&parts)
}

impl Sight {
    /// Makes the scope's view, carried by `camera`, and the laser, all out of sight.
    pub(super) fn new(graph: &mut Graph, camera: Handle<Node>) -> Self {
        let surface = |data: SurfaceData, material: MaterialResource| {
            SurfaceBuilder::new(SurfaceResource::new_embedded(data))
                .with_material(material)
                .build()
        };
        let black: Handle<Node> = MeshBuilder::new(BaseBuilder::new().with_cast_shadows(false))
            .with_surfaces(vec![surface(scope_black(), black())])
            .build(graph)
            .to_base();
        let lines: Handle<Node> = MeshBuilder::new(BaseBuilder::new().with_cast_shadows(false))
            .with_surfaces(vec![surface(reticle(), violet(2.0))])
            .build(graph)
            .to_base();
        // A pivot, not a mesh: a mesh with no surfaces of its own has no bounds to speak of, and
        // the level's survey takes every mesh's for the map's.
        let scope: Handle<Node> = PivotBuilder::new(
            BaseBuilder::new()
                .with_cast_shadows(false)
                .with_visibility(false)
                .with_child(black)
                .with_child(lines),
        )
        .build(graph)
        .to_base();
        graph.link_nodes(scope, camera);
        // The reticle a hair in front of the black, so neither flickers through the other.
        graph[lines]
            .local_transform_mut()
            .set_position(Vector3::new(0.0, 0.0, -0.001));

        let beam_shape = SurfaceData::make_cube(
            Matrix4::new_translation(&Vector3::new(0.0, 0.0, 0.5))
                * Matrix4::new_nonuniform_scaling(&Vector3::new(BEAM_WIDTH, BEAM_WIDTH, 1.0)),
        );
        let beam: Handle<Node> = MeshBuilder::new(BaseBuilder::new().with_cast_shadows(false).with_visibility(false))
            .with_surfaces(vec![surface(beam_shape, violet(LASER_GLOW))])
            .build(graph)
            .to_base();
        let glow: Handle<Node> = PointLightBuilder::new(
            BaseLightBuilder::new(BaseBuilder::new())
                .with_color(SNIPER_COLOR)
                .with_intensity(DOT_BRIGHTNESS)
                .with_scatter_enabled(false),
        )
        .with_radius(DOT_REACH)
        .build(graph)
        .to_base();
        let dot_shape = SurfaceData::make_sphere(8, 8, 0.5 * DOT_SIZE, &Matrix4::identity());
        let dot: Handle<Node> = MeshBuilder::new(
            BaseBuilder::new()
                .with_cast_shadows(false)
                .with_visibility(false)
                .with_child(glow),
        )
        .with_surfaces(vec![surface(dot_shape, violet(LASER_GLOW))])
        .build(graph)
        .to_base();
        Self { scope, beam, dot }
    }
}

/// Shows `node`, or hides it, if it is not already so.
fn show(graph: &mut Graph, node: Handle<Node>, shown: bool) {
    if let Ok(node) = graph.try_get_mut(node) {
        if node.visibility() != shown {
            node.set_visibility(shown);
        }
    }
}

impl Player {
    /// Puts the sniper's sights where they are this frame: the view through its scope, zoomed in,
    /// as far as it is raised to the eye; and its laser, while it is out.
    pub(super) fn aim_sights(&mut self, graph: &mut Graph) {
        // The scope: zooming in as it comes up to the eye, its view showing once it is there.
        let scoped = if self.in_own_eyes { self.scoped(graph) } else { 0.0 };
        let t = ((scoped - ZOOM_FROM) / (SCOPE_IN - ZOOM_FROM)).clamp(0.0, 1.0);
        let zoom = 1.0 + (SCOPE_ZOOM - 1.0) * t * t * (3.0 - 2.0 * t);
        let through = scoped >= SCOPE_IN;
        let half = (0.5 * self.fov.to_radians()).tan() / zoom;
        // Only while it zooms, or as it stops: the view's width is otherwise others' to set (the
        // close-up talking, the settings).
        if zoom != self.zoom {
            self.zoom = zoom;
            if let Ok(camera) = graph.try_get_mut_of_type::<Camera>(self.camera.transmute::<Node>()) {
                if let Projection::Perspective(perspective) = camera.projection_mut() {
                    perspective.fov = 2.0 * half.atan();
                }
            }
        }
        show(graph, self.sight.scope, through);
        if through {
            let size = SCOPE_DISTANCE * half * SCOPE_WINDOW;
            graph[self.sight.scope]
                .local_transform_mut()
                .set_position(Vector3::new(0.0, 0.0, SCOPE_DISTANCE))
                .set_scale(Vector3::new(size, size, 1.0));
        }

        // The laser: from the gun held in view, or else the droid's, while the sniper is out.
        let laser = self
            .avatar
            .as_ref()
            .filter(|avatar| avatar.in_hand() == Weapon::Sniper && avatar.gun_out(graph, Weapon::Sniper))
            .and_then(|avatar| {
                let held = self.viewmodel_laser(graph);
                held.or_else(|| avatar.laser_node(Weapon::Sniper))
            })
            .and_then(|node| {
                let node = graph.try_get(node).ok()?;
                let from = node.global_position();
                let way = node.global_transform().transform_vector(&Vector3::x()).try_normalize(1.0e-6)?;
                Some((from, way))
            });
        let Some((from, way)) = laser else {
            show(graph, self.sight.beam, false);
            show(graph, self.sight.dot, false);
            return;
        };
        let hit = self.first_hit(graph, from, way, LASER_RANGE);
        let length = hit.map_or(LASER_RANGE, |(distance, _)| distance);
        // Through the scope the beam would streak across the view from the side: only its dot.
        show(graph, self.sight.beam, !through);
        if !through {
            graph[self.sight.beam]
                .local_transform_mut()
                .set_position(from)
                .set_rotation(UnitQuaternion::face_towards(&way, &Vector3::y()))
                .set_scale(Vector3::new(1.0, 1.0, length.max(1.0e-3)));
        }
        show(graph, self.sight.dot, hit.is_some());
        if hit.is_some() {
            // Its middle just short of what it is on, so it shows on the surface.
            let at = from + way * (length - 0.25 * DOT_SIZE).max(0.0);
            graph[self.sight.dot].local_transform_mut().set_position(at);
        }
    }
}
