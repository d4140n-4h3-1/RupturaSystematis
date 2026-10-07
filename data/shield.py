"""
The shield: a see-through egg of cyan glass that closes round the player's droid, with two rings
of light round it - one at its widest, one higher up - so that it reads as a field and not a
bubble. Raised with the 1 key, it stops what would hit the droid for a few seconds, then has to
recharge (see src/shield.rs).

Builds data/shield.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/shield.py -- --out ~/Documents/blender/shield.blend --glb data/shield.glb

Its origin is at the droid's feet, its +Z up: an egg HEIGHT tall, widest a third of the way up, at
WIDTH across, and narrower over the head, as an egg stood on its broad end is. The glass is cyan
with an alpha under 1: the engine takes a glTF surface for solid whatever its alpha, so the game
makes any see-through surface glass of its own colour, as it does the hearts'. The rings glow, and
are their own colour too, which the engine needs for them to glow at full strength.
"""

import bmesh
import bpy
import math
import os
import sys

HEIGHT = 2.3         # from the feet to the top
WIDTH = 1.7          # at its widest
BOTTOM = -0.02       # just under the floor, so that no gap shows at the feet
TAPER = 0.22         # how much narrower the top half is than the bottom: an egg, not a ball
SEGMENTS, RINGS = 64, 40
GLASS = (0.25, 0.8, 1.0, 0.28)   # cyan, mostly see-through
LIGHT = (0.2, 0.85, 1.0)
GLOW = 4.0


def get_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    glb = argv[argv.index("--glb") + 1] if "--glb" in argv else None
    out = argv[argv.index("--out") + 1] if "--out" in argv else None
    return glb, out


def egg_radius(t):
    """How far out the egg is at `t`, from -1 at its bottom to 1 at its top, as a share of its
    widest: a sphere's, narrowed above its middle and widened below it."""
    return math.sqrt(max(0.0, 1.0 - t * t)) * (1.0 - TAPER * t)


def glass_material():
    m = bpy.data.materials.new("ShieldGlass")
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = GLASS
    bsdf.inputs["Alpha"].default_value = GLASS[3]
    bsdf.inputs["Roughness"].default_value = 0.05
    m.diffuse_color = GLASS
    m.surface_render_method = "BLENDED"
    return m


def light_material():
    m = bpy.data.materials.new("ShieldLight")
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*LIGHT, 1.0)
    bsdf.inputs["Emission Color"].default_value = (*LIGHT, 1.0)
    bsdf.inputs["Emission Strength"].default_value = GLOW
    m.diffuse_color = (*LIGHT, 1.0)
    return m


def egg(material):
    """The shell: a UV sphere drawn out into an egg, its origin at the feet."""
    mesh = bpy.data.meshes.new("Shield")
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=SEGMENTS, v_segments=RINGS, radius=1.0)
    half = (HEIGHT - BOTTOM) / 2
    widest = max(egg_radius(i / 200 * 2 - 1) for i in range(201))
    for v in bm.verts:
        t = v.co.z
        r = math.hypot(v.co.x, v.co.y)
        out = egg_radius(t) / widest * (WIDTH / 2)
        scale = out / r if r > 1e-6 else 0.0
        v.co.x, v.co.y = v.co.x * scale, v.co.y * scale
        v.co.z = BOTTOM + (t + 1) * half
    for f in bm.faces:
        f.smooth = True
    bm.to_mesh(mesh)
    bm.free()
    mesh.materials.append(material)
    o = bpy.data.objects.new("Shield", mesh)
    bpy.context.scene.collection.objects.link(o)
    return o, half, widest


def ring(name, z, material, half, widest, thickness=0.025):
    """A ring of light round the egg at height `z`, just outside it."""
    t = (z - BOTTOM) / half - 1
    radius = egg_radius(t) / widest * (WIDTH / 2) + thickness
    bpy.ops.mesh.primitive_torus_add(major_radius=radius, minor_radius=thickness, major_segments=96,
                                     minor_segments=8, location=(0, 0, z))
    o = bpy.context.active_object
    o.name = name
    o.data.materials.append(material)
    bpy.ops.object.shade_smooth()
    return o


def main():
    glb, out = get_args()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    glass, light = glass_material(), light_material()
    shell, half, widest = egg(glass)
    # A ring round its broad lower part, and a thinner one higher up.
    low = ring("RingLow", BOTTOM + half * 0.62, light, half, widest)
    high = ring("RingHigh", BOTTOM + half * 1.35, light, half, widest, thickness=0.018)
    for o in (low, high):
        o.parent = shell
    bpy.ops.object.camera_add(location=(0, -5.5, 1.6), rotation=(math.radians(84), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "shield.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[shield] {HEIGHT} m tall, {WIDTH} m across -> {out}")
    if glb:
        bpy.ops.export_scene.gltf(filepath=os.path.abspath(glb), export_format="GLB",
                                  export_cameras=False, export_lights=False)
        print(f"[shield] exported -> {glb}")


if __name__ == "__main__":
    main()
