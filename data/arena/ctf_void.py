"""
Capture-the-flag arena in the void: platforms hanging in open space, joined by bridges, with
nothing under them, lit by street lights. Two mirrored bases, a raised hub in the middle, a pad
on each flank, and three ways across: the bridge straight down the middle, narrow and open, and
a lane round each flank, longer, with cover along it.

Builds data/arena/ctf_void.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/arena/ctf_void.py -- --seed 7 --out ~/Documents/blender/map/ctf_void.blend --glb data/arena/ctf_void.glb

Or open in Blender > Scripting tab > Run Script.

Layout (top view, x runs along the length, red base at -x and blue at +x; mirrored through the
middle, so neither side gets a better layout):

            +--pad--+ ================ north lane ================ +--pad--+
            |       |==+                  ||                  +==|       |
            +-------+  |                  ||                  |  +-------+
     +--------+     +==+              +--------+              +==+     +--------+
     |        |                       |  +--+  |                       |        |
     | C   F  |======= mid bridge ====| =|hub|= |==== mid bridge =======|  F   C |
     |        |                       |  +--+  |                       |        |
     +--------+     +==+              +--------+              +==+     +--------+
            +-------+  |                  ||                  |  +-------+
            |       |==+                  ||                  +==|       |
            +--pad--+ ================ south lane ================ +--pad--+

Made for Ruptura Systematis (MazeGame/maze), whose capture the flag reads the map's empties: the
game puts each side's flag in a firewall at `flag_red` / `flag_blue`, the computer that opens it
at `computer_red` / `computer_blue`, and its droids at `post_red_1` and on. The game knows this
map is in the void (its `Map::void`): it gives it a black sky, no floor of its own under it, and
surveys its floors under the open sky, where a cell's floor is the first surface down unless
something hangs over it with headroom under it, as a street light's head does. So every raised
floor is solid down to the platform, and reached by stairs of 0.25 m steps; every walkway's top
is at exactly 0, the ground the droids' walking starts from. Each street light's glass is pure
magenta, which the game lights with a lamp of its own.
"""

import bpy
import math
import random
import sys
import os

SLAB = 1.0           # platform and bridge thickness, under their tops at 0
BASE = (30.0, 48.0, -12.0, 12.0)    # blue's base platform (x0, x1, y0, y1); red's is mirrored
CENTRE = 9.0         # half the side of the centre platform
HUB = 3.5            # half the side of the raised hub on it
HUB_HEIGHT = 1.5
PAD = (14.0, 24.0, 14.0, 24.0)      # blue's north flank pad; the others are mirrored
LANE = (17.0, 21.0)  # |y| of each flank lane's edges
MID = 1.5            # half the width of the middle bridge
LINK = 1.5           # half the width of the bridge from the centre to each lane
LINK_AT = 6.0        # how far along the base's side, from its front, the way round to its pads leaves
STEP_RISE = 0.25
STEP_RUN = 0.35
STAIR_WIDTH = 3.0
RAIL = 1.0           # height of low cover walls
NUM_COVER = 30       # tries at cover blocks per half (mirrored to the other half)
MIN_GAP = 1.6        # walkable space kept around cover
EDGE = 1.2           # how far cover keeps in from a platform's edge
LAMP_HEIGHT = 4.6    # top of a street light's pole
ARM = 1.3            # how far its arm reaches out over the walkway


def get_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    seed = int(argv[argv.index("--seed") + 1]) if "--seed" in argv else random.randrange(1_000_000)
    glb = argv[argv.index("--glb") + 1] if "--glb" in argv else None
    out = argv[argv.index("--out") + 1] if "--out" in argv else None
    return seed, glb, out


def material(name, rgb):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    m.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*rgb, 1)
    m.diffuse_color = (*rgb, 1)
    return m


def collection(name):
    c = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(c)
    return c


def move_to(obj, col):
    for c in obj.users_collection:
        c.objects.unlink(obj)
    col.objects.link(obj)


def add_block(name, x, y, w, d, h, rot, col, mat, cover, z=0.0):
    """A box whose bottom sits at height z."""
    bpy.ops.mesh.primitive_cube_add(size=1, location=(x, y, z + h / 2))
    o = bpy.context.active_object
    o.name = name
    o.scale = (w, d, h)
    o.rotation_euler.z = rot
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    o.data.materials.append(mat)
    o["cover"] = cover
    move_to(o, col)
    return o


def add_box(name, x0, x1, y0, y1, z0, z1, col, mat, cover="none"):
    """An axis-aligned box given by its extents, in either order."""
    x0, x1 = sorted((x0, x1))
    y0, y1 = sorted((y0, y1))
    return add_block(name, (x0 + x1) / 2, (y0 + y1) / 2, x1 - x0, y1 - y0, z1 - z0, 0, col, mat, cover, z=z0)


def add_walkway(name, rect, col, mat, under, mat_under, keel):
    """A slab with its top at 0 over `rect` (x0, x1, y0, y1), and, if `keel`, a rough inverted
    pyramid of rock under it, its point `keel` meters down, so it reads as a thing that floats."""
    x0, x1, y0, y1 = rect
    add_box(name, x0, x1, y0, y1, -SLAB, 0, col, mat)
    if keel:
        w, d = abs(x1 - x0), abs(y1 - y0)
        bpy.ops.mesh.primitive_cone_add(vertices=4, radius1=0.0, radius2=1.0, depth=keel,
                                        location=((x0 + x1) / 2, (y0 + y1) / 2, -SLAB - keel / 2))
        o = bpy.context.active_object
        o.name = f"{name}_Keel"
        o.rotation_euler.z = math.pi / 4
        bpy.ops.object.transform_apply(location=False, rotation=True, scale=False)
        # the cone's square top is sqrt(2) across: stretch it to the slab, a little inside it
        o.scale = (0.92 * w / math.sqrt(2), 0.92 * d / math.sqrt(2), 1.0)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
        o.data.materials.append(mat_under)
        move_to(o, under)


def add_stairs(name, x, y, direction, rise, col, mat):
    """Solid stairs whose top step meets height `rise` at (x, y) and that descend toward
    `direction` ('+x', '-x', '+y' or '-y'). Returns the footprint (x0, x1, y0, y1)."""
    n = round(rise / STEP_RISE)
    sign = 1 if direction[0] == "+" else -1
    along_x = direction[1] == "x"
    w = STAIR_WIDTH / 2
    for i in range(n):
        top = rise - i * STEP_RISE                      # step i is i steps down from the top
        a, b = i * STEP_RUN * sign, (i + 1) * STEP_RUN * sign
        if along_x:
            add_box(f"{name}_{i}", x + a, x + b, y - w, y + w, 0, top, col, mat)
        else:
            add_box(f"{name}_{i}", x - w, x + w, y + a, y + b, 0, top, col, mat)
    run = n * STEP_RUN * sign
    if along_x:
        return (*sorted((x, x + run)), y - w, y + w)
    return (x - w, x + w, *sorted((y, y + run)))


def add_street_light(name, x, y, toward, col, mat_pole, mat_head, lights, mat_glass):
    """A street light: a pole standing at (x, y), its arm reaching out `toward` ('+x', '-x',
    '+y' or '-y') over the walkway, and a head at the end of it, glazed underneath. Returns the
    pole's footprint, to keep cover clear of it."""
    dx, dy = {"+x": (1, 0), "-x": (-1, 0), "+y": (0, 1), "-y": (0, -1)}[toward]
    p = 0.1
    add_box(f"{name}_Foot", x - 0.3, x + 0.3, y - 0.3, y + 0.3, 0, 0.12, col, mat_pole)
    add_box(f"{name}_Pole", x - p, x + p, y - p, y + p, 0.12, LAMP_HEIGHT, col, mat_pole)
    # the arm, from the pole out to the head
    ax, ay = x + dx * ARM, y + dy * ARM
    t = 0.06
    if dx:
        add_box(f"{name}_Arm", x, ax, y - t, y + t, LAMP_HEIGHT - 0.12, LAMP_HEIGHT, col, mat_pole)
    else:
        add_box(f"{name}_Arm", x - t, x + t, y, ay, LAMP_HEIGHT - 0.12, LAMP_HEIGHT, col, mat_pole)
    # the head: a housing longer along the arm, and its glass on the underside
    hl, hw = 0.45, 0.25                                # half its length and width
    hx, hy = (hl, hw) if dx else (hw, hl)
    cx, cy = ax + dx * 0.2, ay + dy * 0.2
    z1, z0 = LAMP_HEIGHT + 0.05, LAMP_HEIGHT - 0.2
    add_box(f"{name}_Head", cx - hx, cx + hx, cy - hy, cy + hy, z0, z1, col, mat_head)
    g = 0.05
    add_box(f"{name}_Glass", cx - hx + g, cx + hx - g, cy - hy + g, cy + hy - g, z0 - 0.04, z0,
            lights, mat_glass)
    return (x - 0.5, x + 0.5, y - 0.5, y + 0.5)


def marker(name, x, y, facing_x):
    """An empty where the game puts something, its +x turned to point along `facing_x` (+1 or -1)."""
    empty = bpy.data.objects.new(name, None)
    empty.location = (x, y, 0.0)
    empty.rotation_euler.z = 0.0 if facing_x > 0 else math.pi
    bpy.context.scene.collection.objects.link(empty)


def mirror_x(rect):
    x0, x1, y0, y1 = rect
    return (-x1, -x0, y0, y1)


def mirror_y(rect):
    x0, x1, y0, y1 = rect
    return (x0, x1, -y1, -y0)


def main():
    seed, glb, out = get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)

    col_walk = collection("Platforms")
    col_under = collection("Underside")
    col_struct = collection("Structures")
    col_cover = collection("Cover")
    col_lights = collection("Lights")

    mat_deck = material("Deck", (0.34, 0.35, 0.38))
    mat_bridge = material("Bridge", (0.28, 0.29, 0.32))
    mat_under = material("Rock", (0.16, 0.15, 0.17))
    mat_stairs = material("Stairs", (0.50, 0.48, 0.42))
    mat_low = material("CoverLow", (0.60, 0.45, 0.25))
    mat_high = material("CoverHigh", (0.40, 0.42, 0.45))
    mat_pole = material("LampPole", (0.12, 0.12, 0.14))
    mat_head = material("LampHead", (0.20, 0.20, 0.22))
    mat_glass = material("LightGlass", (1.0, 0.0, 1.0))  # the game's marker for fixture glass
    # the game's own team colours: blue's droids are cyan
    mat_team = {"red": material("TeamRed", (0.80, 0.07, 0.05)),
                "blue": material("TeamCyan", (0.00, 0.75, 0.85))}

    # where cover may go: on the platforms, in from their edges; and footprints it must stay
    # clear of: (x0, x1, y0, y1)
    platforms = []
    keep_clear = []

    # the centre: a square platform round a raised hub, with stairs up onto it from both ends
    c = CENTRE
    add_walkway("Centre", (-c, c, -c, c), col_walk, mat_deck, col_under, mat_under, 14.0)
    platforms.append((-c, c, -c, c))
    add_box("Hub", -HUB, HUB, -HUB, HUB, 0, HUB_HEIGHT, col_struct, mat_deck)
    keep_clear.append((-HUB, HUB, -HUB, HUB))
    for i, (x, d) in enumerate(((-HUB, "-x"), (HUB, "+x"))):
        keep_clear.append(add_stairs(f"HubStairs_{i}", x, 0, d, HUB_HEIGHT, col_struct, mat_stairs))
    # low walls along the hub's two open sides, broken in the middle to step through
    for sy in (1, -1):
        for sx in (1, -1):
            add_box(f"HubWall_{sy}{sx}", sx * 0.9, sx * HUB, sy * (HUB - 0.2), sy * HUB,
                    HUB_HEIGHT, HUB_HEIGHT + RAIL, col_struct, mat_low, "low")

    for team, sx in (("red", -1), ("blue", 1)):
        mat = mat_team[team]
        facing = -sx                                    # toward the middle
        base = BASE if sx > 0 else mirror_x(BASE)
        add_walkway(f"{team}Base", base, col_walk, mat_deck, col_under, mat_under, 18.0)
        platforms.append(base)
        bx0, bx1, by0, by1 = base
        # a band of the side's colour round the base's rim, just under its edge
        r = 0.06
        for i, (x0, x1, y0, y1) in enumerate([(bx0 - r, bx1 + r, by1, by1 + r), (bx0 - r, bx1 + r, by0 - r, by0),
                                              (bx0 - r, bx0, by0, by1), (bx1, bx1 + r, by0, by1)]):
            add_box(f"{team}Rim_{i}", x0, x1, y0, y1, -0.45, -0.05, col_struct, mat)

        # the middle bridge, from the base's front edge to the centre's
        front = BASE[0]
        mid = (c, front, -MID, MID) if sx > 0 else mirror_x((c, front, -MID, MID))
        add_walkway(f"{team}MidBridge", mid, col_walk, mat_bridge, col_under, mat_under, 0)

        for side, sy in (("N", 1), ("S", -1)):
            pad = PAD
            pad = pad if sy > 0 else mirror_y(pad)
            pad = pad if sx > 0 else mirror_x(pad)
            add_walkway(f"{team}Pad{side}", pad, col_walk, mat_deck, col_under, mat_under, 10.0)
            platforms.append(pad)
            # the flank lane's half, from the pad to the middle, where the other half meets it
            lane = (0, PAD[0], *LANE)
            lane = lane if sy > 0 else mirror_y(lane)
            lane = lane if sx > 0 else mirror_x(lane)
            add_walkway(f"{team}Lane{side}", lane, col_walk, mat_bridge, col_under, mat_under, 0)
            # low walls along it, on alternate edges, to duck behind on the way across
            for n, (x, outer) in enumerate(((3.5, True), (10.5, False))):
                y0 = LANE[1] - 0.4 if outer else LANE[0]
                wall = (x - 1.25, x + 1.25, y0, y0 + 0.4)
                wall = wall if sy > 0 else mirror_y(wall)
                wall = wall if sx > 0 else mirror_x(wall)
                add_box(f"{team}LaneWall{side}_{n}", *wall, 0, RAIL, col_struct, mat_low, "low")
            # from the base round to the pad: out of the base's side, along to the lane's height,
            # then across to the pad
            link_a = (BASE[0] + LINK_AT, BASE[0] + LINK_AT + 4.0, BASE[3], LANE[1])
            link_b = (PAD[1], BASE[0] + LINK_AT, *LANE)
            for n, link in enumerate((link_a, link_b)):
                link = link if sy > 0 else mirror_y(link)
                link = link if sx > 0 else mirror_x(link)
                add_walkway(f"{team}Link{side}_{n}", link, col_walk, mat_bridge, col_under, mat_under, 0)
            # a flight of stairs up to a perch on the pad, looking out over the lane
            px = (PAD[0] + PAD[1]) / 2
            perch = (px - 2.0, px + 2.0, PAD[3] - 3.5, PAD[3])
            perch = perch if sy > 0 else mirror_y(perch)
            perch = perch if sx > 0 else mirror_x(perch)
            add_box(f"{team}Perch{side}", *perch, 0, HUB_HEIGHT, col_struct, mat_deck)
            keep_clear.append(perch)
            stair_y = sy * (PAD[3] - 3.5)
            keep_clear.append(add_stairs(f"{team}PerchStairs{side}", sx * px, stair_y,
                                         "-y" if sy > 0 else "+y", HUB_HEIGHT, col_struct, mat_stairs))
            add_box(f"{team}PerchWall{side}", perch[0], perch[1], *sorted((sy * (PAD[3] - 0.2), sy * PAD[3])),
                    HUB_HEIGHT, HUB_HEIGHT + RAIL, col_struct, mat_low, "low")

        # the flag on a pad of the side's colour, toward the back of the base
        flag_x = sx * 40.0
        bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=2.5, depth=0.02, location=(flag_x, 0, 0.01))
        flag_pad = bpy.context.active_object
        flag_pad.name = f"{team}FlagPad"
        flag_pad.data.materials.append(mat)
        move_to(flag_pad, col_struct)
        keep_clear.append((flag_x - 4.5, flag_x + 4.5, -4.5, 4.5))

        # what the game puts here itself: the flag in its firewall, the computer that opens it
        # (by the back edge, its screen toward the middle), and each droid's post
        marker(f"flag_{team}", flag_x, 0.0, facing)
        marker(f"computer_{team}", sx * 45.5, -7.0, facing)
        keep_clear.append((sx * 45.5 - 2.5, sx * 45.5 + 2.5, -9.5, -4.5))
        px = (PAD[0] + PAD[1]) / 2
        for n, (x, y) in enumerate(((33.0, -5.0), (px, 17.0), (px, -17.0)), start=1):
            marker(f"post_{team}_{n}", sx * x, y, facing)
            keep_clear.append((sx * x - 1.5, sx * x + 1.5, y - 1.5, y + 1.5))

    # the bridges from the centre out to each lane
    for side, sy in (("N", 1), ("S", -1)):
        link = (-LINK, LINK, *sorted((sy * c, sy * LANE[0])))
        add_walkway(f"CentreLink{side}", link, col_walk, mat_bridge, col_under, mat_under, 0)

    # street lights: (x, y, which way the arm reaches), on the blue half, mirrored through the
    # middle onto the red half; each pole stands at a walkway's edge, its head out over it
    bx0, bx1, by0, by1 = BASE
    px0, px1, py0, py1 = PAD
    lights = [
        # the base: its corners and the middle of its back edge
        (bx0 + 0.6, by1 - 0.6, "-y"), (bx0 + 0.6, by0 + 0.6, "+y"),
        (bx1 - 0.6, by1 - 0.6, "-y"), (bx1 - 0.6, by0 + 0.6, "+y"),
        (bx1 - 0.6, 0.0 + 4.0, "-x"),
        # the middle bridge, on alternate sides
        (16.0, MID - 0.3, "-y"), (24.0, -MID + 0.3, "+y"),
        # the pads, at their outer corners
        (px1 - 0.6, py1 - 0.6, "-y"), (px1 - 0.6, -py1 + 0.6, "+y"),
        # the lanes, on their outer edge
        (7.0, LANE[1] - 0.3, "-y"), (7.0, -LANE[1] + 0.3, "+y"),
        # the centre's corners on this half
        (c - 0.6, c - 0.6, "-x"), (c - 0.6, -c + 0.6, "-x"),
    ]
    flip = {"+x": "-x", "-x": "+x", "+y": "-y", "-y": "+y"}
    n_lights = 0
    for x, y, toward in lights:
        for lx, ly, lt, tag in ((x, y, toward, "B"), (-x, -y, flip[toward], "R")):
            keep_clear.append(add_street_light(f"StreetLight_{n_lights}{tag}", lx, ly, lt, col_struct,
                                               mat_pole, mat_head, col_lights, mat_glass))
        n_lights += 2

    # random cover on the blue half, mirrored through the centre onto the red half: only on the
    # platforms, in from their edges
    placed = []

    def clear(x, y, r):
        if not any(x0 + r + EDGE <= x <= x1 - r - EDGE and y0 + r + EDGE <= y <= y1 - r - EDGE
                   for x0, x1, y0, y1 in platforms):
            return False
        for x0, x1, y0, y1 in keep_clear:
            dx, dy = max(x0 - x, 0, x - x1), max(y0 - y, 0, y - y1)
            if math.hypot(dx, dy) < r + MIN_GAP:
                return False
        return all(math.hypot(x - px, y - py) >= r + pr + MIN_GAP for px, py, pr in placed)

    n_cover = 0
    for i in range(NUM_COVER):
        low = rng.random() < 0.6
        w = rng.uniform(1.5, 3.5)
        d = rng.uniform(0.6, 1.2) if low else rng.uniform(0.8, 2.0)
        ht = rng.uniform(0.9, 1.2) if low else rng.uniform(2.0, 2.8)
        r = math.hypot(w, d) / 2
        rot = rng.choice([0, math.pi / 2, rng.uniform(0, math.pi)])
        for _ in range(800):
            x, y = rng.uniform(0.5, BASE[1]), rng.uniform(-PAD[3], PAD[3])
            if clear(x, y, r) and clear(-x, -y, r):
                placed += [(x, y, r), (-x, -y, r)]
                kind, mat = ("Low", mat_low) if low else ("High", mat_high)
                add_block(f"{kind}Cover_{i}_B", x, y, w, d, ht, rot, col_cover, mat, kind.lower())
                add_block(f"{kind}Cover_{i}_R", -x, -y, w, d, ht, rot, col_cover, mat, kind.lower())
                n_cover += 2
                break

    # light + camera, to look it over in Blender
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 30), rotation=(math.radians(45), 0, math.radians(30)))
    bpy.context.active_object.data.energy = 3
    bpy.ops.object.camera_add(location=(0, -80, 60), rotation=(math.radians(50), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.0, 0.0, 0.0, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "ctf_void.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} cover={n_cover} street lights={n_lights} -> {out}")
    if glb:
        export_glb(os.path.abspath(glb), [col_walk, col_under, col_struct, col_cover])


def join(col):
    """Joins a collection's meshes into one object (keeping their materials)."""
    objs = [o for o in col.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    objs[0].name = col.name


def export_glb(path, cols):
    """Exports for the game with the walkways, their undersides, structures and cover each
    joined into one mesh, the street lights' glass left a piece each, and the markers as
    empties. Runs after the .blend is saved, so the saved file keeps every object separate."""
    for col in cols:
        join(col)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_extras=True,
                              export_cameras=False, export_lights=False)
    print(f"[map] exported -> {path}")


if __name__ == "__main__":
    main()
