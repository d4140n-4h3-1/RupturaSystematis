"""
A firewall door, for heists: a doorway's frame - two pillars and a lintel, dark, traced in
glowing orange - with a pane of the firewall's orange glass in it, which stands in the way until
the computer it answers to is hacked (see src/heist.rs). It is made in the firewall's look (see
data/ctf/firewall.glb): the pane's meshes are named `firewall_barrier`, and the game makes them
glass and has them flicker as a firewall's shell does.

Builds data/heist/firewall_door.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/heist/door.py -- --out ~/Documents/blender/heist/firewall_door.blend --glb data/heist/firewall_door.glb

Its origin is on the floor in the middle of the doorway. The way through it is along +X, the
pane across it along Y and up Z - in the game's terms, through along +x, across z, up y, as the
door's marker in a map turns it. A map leaves an opening OPENING wide and FRAME_HEIGHT tall in its
wall for it, which the frame fills.
"""

import bmesh
import bpy
import math
import os
import sys

DOORWAY = (2.4, 3.0)       # the way through: how wide and how tall
PILLAR = 0.4               # each pillar's width, and the lintel's height
DEPTH = 0.6                # how deep the frame is, along the way through
PANE = 0.08                # how thick the pane is
OPENING = DOORWAY[0] + 2 * PILLAR          # the opening a wall leaves for the whole frame
FRAME_HEIGHT = DOORWAY[1] + PILLAR
TRIM = 0.06
ORANGE = (1.0, 0.18, 0.0)
DARK = (0.03, 0.03, 0.04)


def get_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    glb = argv[argv.index("--glb") + 1] if "--glb" in argv else None
    out = argv[argv.index("--out") + 1] if "--out" in argv else None
    return glb, out


def material(name, rgb, alpha=1.0, emit=0.0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*rgb, alpha)
    b.inputs["Alpha"].default_value = alpha
    b.inputs["Metallic"].default_value = 0.5 if emit == 0.0 else 0.0
    b.inputs["Roughness"].default_value = 0.35
    if emit:
        b.inputs["Emission Color"].default_value = (*rgb, 1)
        b.inputs["Emission Strength"].default_value = emit
    m.diffuse_color = (*rgb, alpha)
    if alpha < 1.0:
        m.surface_render_method = "BLENDED"
    return m


def box(name, x0, x1, y0, y1, z0, z1, mat):
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co.x = x0 + (v.co.x + 0.5) * (x1 - x0)
        v.co.y = y0 + (v.co.y + 0.5) * (y1 - y0)
        v.co.z = z0 + (v.co.z + 0.5) * (z1 - z0)
    bm.to_mesh(mesh)
    bm.free()
    mesh.materials.append(mat)
    o = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(o)
    return o


def main():
    glb, out = get_args()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    dark, orange = material("FrameDark", DARK), material("FrameTrim", ORANGE, emit=4.0)
    glass = material("FirewallPane", ORANGE, alpha=0.4)
    w, h, d = DOORWAY[0] / 2, DOORWAY[1], DEPTH / 2
    # The frame: two pillars and a lintel over them.
    for s in (-1, 1):
        y0, y1 = sorted((s * w, s * (w + PILLAR)))
        box(f"Pillar{'L' if s < 0 else 'R'}", -d, d, y0, y1, 0.0, FRAME_HEIGHT, dark)
    box("Lintel", -d, d, -w, w, h, FRAME_HEIGHT, dark)
    # Lines of light round the doorway, on both faces, and up the pillars' outer edges.
    for x in (-d - TRIM / 2, d + TRIM / 2):
        for s in (-1, 1):
            box("TrimSide", x - TRIM / 2, x + TRIM / 2, s * w - TRIM / 2, s * w + TRIM / 2, 0.0, h, orange)
            box("TrimEdge", x - TRIM / 2, x + TRIM / 2, s * (w + PILLAR) - TRIM / 2, s * (w + PILLAR) + TRIM / 2,
                0.0, FRAME_HEIGHT, orange)
        box("TrimTop", x - TRIM / 2, x + TRIM / 2, -w, w, h - TRIM / 2, h + TRIM / 2, orange)
        box("TrimCrown", x - TRIM / 2, x + TRIM / 2, -(w + PILLAR), w + PILLAR, FRAME_HEIGHT - TRIM,
            FRAME_HEIGHT, orange)
    # The pane, the firewall's glass, in the middle of the frame's depth.
    box("firewall_barrier", -PANE / 2, PANE / 2, -w, w, 0.0, h, glass)
    # Bars of light across it, part of the barrier too, so they go with it.
    for k in range(1, 6):
        z = h * k / 6
        box(f"firewall_barrier_bar{k}", -PANE, PANE, -w, w, z - 0.015, z + 0.015, glass)

    bpy.ops.object.camera_add(location=(5.5, -3.5, 2.2), rotation=(math.radians(80), 0, math.radians(57)))
    bpy.context.scene.camera = bpy.context.active_object
    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "firewall_door.blend")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[door] {OPENING:.1f} m by {FRAME_HEIGHT:.1f} m -> {out}")
    if glb:
        bpy.ops.export_scene.gltf(filepath=os.path.abspath(glb), export_format="GLB",
                                  export_cameras=False, export_lights=False)
        print(f"[door] exported -> {glb}")


if __name__ == "__main__":
    main()
