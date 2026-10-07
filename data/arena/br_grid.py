"""
Battle royale on the Grid: a city of dark blocks on a platform floating in the void, every edge
of it traced in light. Its buildings are the town's (see br_town.py) - one to four floors, doors,
windows, stairs up inside each and to the roofs of the tallest - but black, and trimmed: lines of
light up every corner, round every floor, along every roof and round every door. Every coloured
trim glows, in a colour of the PALETTE: each block a district of its own colour, its building
and its outline alike; each pad and its bridge and tower another; the cover and the pylons all
sorts; and the square's ring every colour in turn. The colours come from a stream of their own,
so that the seed makes the same city whatever they are.

Builds data/arena/br_grid.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/arena/br_grid.py -- --seed 11 --out ~/Documents/blender/map/br_grid.blend --glb data/arena/br_grid.glb

Layout (top view): BLOCKS x BLOCKS blocks, BLOCK meters square, with streets ROAD wide between
them and round them, on a platform with nothing under it. The floor is ruled with a faint grid of
light, every GRID meters, and every block is outlined brighter. A low barrier, its top lit, runs
round the platform's edge but for a gap in the middle of each side, where a bridge, lit along
both edges and with no rail, goes out over the void to a pad with a tower on it: the highest
ground there is, and a long way out if the ring closes elsewhere. The middle block is an open
square, a ring of light on its floor.

It is night in the void: light pylons stand at every other corner, on the pads and at the
square, and each building has a light in every ceiling - their glass pure magenta, which the
game lights with a lamp of its own.

Made for Ruptura Systematis (MazeGame/maze). The game puts the players down at the empties
`spawn_1` to `spawn_16`, round the platform's outer street.
"""

import bpy
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import br_town  # noqa: E402  - the buildings, walls, floors and stairs, shared with the town
from br_town import add_box, add_block, marker, collection, building, KINDS, WALL, SLAB, PARAPET, RAIL, \
    WAREHOUSE_HEIGHT, FLOOR, DOOR, MIN_GAP  # noqa: E402

BLOCKS = 5
BLOCK = 24.0
ROAD = 8.0
HALF = (BLOCKS * BLOCK + (BLOCKS + 1) * ROAD) / 2   # half the platform's side
DEPTH = 2.5          # how thick the platform is, over the void
BARRIER = 1.0        # the barrier round its edge: crouched behind, jumped over at your peril
GAP = 8.0            # the gap in it where each bridge leaves
BRIDGE = (18.0, 4.0)  # a bridge's length and width
PAD = 22.0           # a pad's side
GRID = 6.0           # the floor's grid of light
LINE = 0.06          # a grid line's width
EDGE_LINE = 0.14     # a block's outline, a bridge's edges
TRIM = 0.12          # a building's trim: how wide each line of light is
PROUD = 0.05         # how far trim stands out from what it is on
PYLON = 6.0          # a light pylon's height
SPAWNS = 16

CYAN = (0.0, 0.85, 1.0)
ORANGE = (1.0, 0.38, 0.02)
WHITE = (0.75, 0.9, 1.0)
# The lines of light come in these, cyan the commonest. Never pure magenta, which marks lamp glass.
PALETTE = {
    "cyan": CYAN,
    "orange": ORANGE,
    "pink": (1.0, 0.08, 0.55),
    "yellow": (1.0, 0.82, 0.05),
    "green": (0.15, 1.0, 0.3),
    "violet": (0.5, 0.15, 1.0),
    "red": (1.0, 0.05, 0.05),
}
SHADES = ("cyan", "cyan", "orange", "pink", "yellow", "green", "violet", "red")


def material(name, rgb, roughness=0.35, metallic=0.4):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*rgb, 1)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    m.diffuse_color = (*rgb, 1)
    return m


def glow(name, rgb, strength):
    """A line of light, glowing `rgb` at `strength`. Its own colour is `rgb` too: the engine lights
    what a surface gives off as it lights a lightmap, by the surface's colour, so a dark line
    would glow only dimly however strong."""
    m = material(name, rgb, roughness=0.5, metallic=0.0)
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Emission Color"].default_value = (*rgb, 1)
    bsdf.inputs["Emission Strength"].default_value = strength
    m.diffuse_color = (*rgb, 1)
    return m


class Grid(br_town.Town):
    """The town's builder, in the Grid's colours, with a collection for the trim."""

    def __init__(self):
        super().__init__()
        self.trim = collection("Trim")
        self.platform = material("Platform", (0.025, 0.03, 0.045))
        self.road = material("Road", (0.015, 0.018, 0.028), roughness=0.15, metallic=0.6)
        self.pavement = material("Pavement", (0.035, 0.04, 0.055))
        self.walls = [material(f"Panel{i}", rgb) for i, rgb in enumerate(
            [(0.03, 0.035, 0.05), (0.045, 0.05, 0.065), (0.02, 0.025, 0.04)])]
        self.floor_mat = material("Floor", (0.05, 0.055, 0.07))
        self.roof = material("Roof", (0.035, 0.04, 0.055))
        self.stairs = material("Stairs", (0.06, 0.065, 0.08))
        self.rail = material("Rail", (0.02, 0.02, 0.03))
        self.low = material("CoverLow", (0.05, 0.05, 0.065))
        self.high = material("CoverHigh", (0.04, 0.045, 0.06))
        self.grid_glow = glow("GridLine", CYAN, 1.0)
        self.lines = {name: glow(f"Trim{name.title()}", rgb, 3.0) for name, rgb in PALETTE.items()}
        self.white = glow("TrimWhite", WHITE, 2.0)
        self.n_trim = 0

    def line(self, x0, x1, y0, y1, z0, z1, mat):
        """A line of light, a box given by its extents."""
        self.n_trim += 1
        return add_box(f"Trim_{self.n_trim}", x0, x1, y0, y1, z0, z1, self.trim, mat)

    def floor_line(self, x0, x1, y0, y1, mat, z=0.0):
        """A line of light set into the floor at `z`, just proud of it."""
        return self.line(x0, x1, y0, y1, z, z + 0.012, mat)

    def outline(self, x0, x1, y0, y1, z0, z1, width, mat, out=0.0):
        """Four lines round the rectangle x0..x1, y0..y1, from z0 to z1, `width` wide, standing `out`
        outside it."""
        w = width
        self.line(x0 - out - w, x1 + out + w, y0 - out - w, y0 - out, z0, z1, mat)
        self.line(x0 - out - w, x1 + out + w, y1 + out, y1 + out + w, z0, z1, mat)
        self.line(x0 - out - w, x0 - out, y0 - out, y1 + out, z0, z1, mat)
        self.line(x1 + out, x1 + out + w, y0 - out, y1 + out, z0, z1, mat)

    def pylon(self, x, y, z=0.0, colour="cyan"):
        """A light pylon: a dark mast, lit up two sides in `colour`, a crossbar at the top, and
        under it the lamp's glass."""
        name = f"Pylon_{self.n_lights}"
        self.n_lights += 1
        p, top = 0.12, z + PYLON
        add_box(f"{name}_Foot", x - 0.35, x + 0.35, y - 0.35, y + 0.35, z, z + 0.15, self.struct, self.rail)
        add_box(f"{name}_Mast", x - p, x + p, y - p, y + p, z + 0.15, top, self.struct, self.rail)
        self.line(x - p - 0.03, x - p, y - 0.03, y + 0.03, z + 0.15, top, self.lines[colour])
        self.line(x + p, x + p + 0.03, y - 0.03, y + 0.03, z + 0.15, top, self.lines[colour])
        add_box(f"{name}_Bar", x - 0.9, x + 0.9, y - 0.25, y + 0.25, top, top + 0.18, self.struct, self.roof)
        self.line(x - 0.9, x + 0.9, y - 0.27, y - 0.25, top, top + 0.18, self.white)
        self.line(x - 0.9, x + 0.9, y + 0.25, y + 0.27, top, top + 0.18, self.white)
        add_box(f"{name}_Glass", x - 0.8, x + 0.8, y - 0.2, y + 0.2, top - 0.04, top, self.lights, self.glass)
        self.keep_clear.append((x - 0.6, x + 0.6, y - 0.6, y + 0.6))


def trim_building(g, kind, cx, cy, w, d, floors, colour):
    """Lines of light on a building: up each corner, round it at every floor and along the top of
    its roof or parapet, and round its doors."""
    mat = g.lines[colour]
    tall = WAREHOUSE_HEIGHT if kind == "warehouse" else FLOOR
    x0, x1, y0, y1 = cx - w / 2, cx + w / 2, cy - d / 2, cy + d / 2
    # The outside faces of its walls, which run past each other at the corners.
    ox0, ox1, oy0, oy1 = x0 - WALL / 2, x1 + WALL / 2, y0 - WALL / 2, y1 + WALL / 2
    roof = floors * tall + SLAB
    parapet = kind in ("office", "apartment")
    top = roof + (PARAPET if parapet else 0.0)
    # Up each corner.
    for cx_, cy_ in ((ox0, oy0), (ox1, oy0), (ox0, oy1), (ox1, oy1)):
        sx = -1 if cx_ == ox0 else 1
        sy = -1 if cy_ == oy0 else 1
        g.line(*sorted((cx_ - sx * TRIM, cx_ + sx * PROUD)), *sorted((cy_ - sy * TRIM, cy_ + sy * PROUD)), 0.0, top, mat)
    # Round it at every floor above the ground, and along its top.
    for k in range(1, floors):
        z = k * tall
        g.outline(ox0, ox1, oy0, oy1, z - TRIM / 2, z + TRIM / 2, PROUD, mat)
    g.outline(ox0, ox1, oy0, oy1, top - TRIM, top, PROUD, mat)
    # Round the doors: in the middle of the front (-y) and the side (+x).
    width = 3.2 if kind == "warehouse" else DOOR[0]
    height = 4.0 if kind == "warehouse" else DOOR[1]
    mx, my = (x0 + x1) / 2, (y0 + y1) / 2
    t = 0.08
    for a in (mx - width / 2 - t, mx + width / 2):
        g.line(a, a + t, oy0 - PROUD, oy0, 0.0, height + t, mat)
    g.line(mx - width / 2 - t, mx + width / 2 + t, oy0 - PROUD, oy0, height, height + t, mat)
    for a in (my - width / 2 - t, my + width / 2):
        g.line(ox1, ox1 + PROUD, a, a + t, 0.0, height + t, mat)
    g.line(ox1, ox1 + PROUD, my - width / 2 - t, my + width / 2 + t, height, height + t, mat)


def square(g, cx, cy, name):
    """The open square: the town's ring of low walls and pillars, its edges lit, round a ring of
    light on the floor."""
    add_box(f"{name}_Paving", cx - BLOCK / 2, cx + BLOCK / 2, cy - BLOCK / 2, cy + BLOCK / 2, -0.1, 0.0,
            g.ground, g.pavement)
    for i in range(8):
        a = i * math.pi / 4
        x, y = cx + 8.0 * math.cos(a), cy + 8.0 * math.sin(a)
        if i % 2:
            add_block(f"{name}_Pillar{i}", x, y, 1.2, 1.2, 3.0, 0, g.cover, g.high, "high")
            g.outline(x - 0.6, x + 0.6, y - 0.6, y + 0.6, 2.9, 3.0, 0.04, g.lines["yellow"])
        else:
            add_block(f"{name}_Low{i}", x, y, 3.0, 0.6, RAIL, a + math.pi / 2, g.cover, g.low, "low")
    # The ring on the floor, in short straight pieces, going round every colour there is.
    shades = list(PALETTE)
    n, r = 48, 4.5
    for i in range(n):
        a = 2 * math.pi * (i + 0.5) / n
        length = 2 * math.pi * r / n + 0.02
        add_block(f"{name}_Ring{i}", cx + r * math.cos(a), cy + r * math.sin(a), 0.18, length, 0.012,
                  a, g.trim, g.lines[shades[i * len(shades) // n]], "none")
    g.keep_clear.append((cx - 10, cx + 10, cy - 10, cy + 10))


def scatter_cover(g, rng, paint, tries):
    """Crates, low walls and parked light cycles - boxes of a car's size - in the yards and
    streets, clear of the buildings, each other and the spawns; each with a line of light along
    its top or its side."""
    placed = []

    def clear(x, y, r):
        for x0, x1, y0, y1 in g.keep_clear:
            dx, dy = max(x0 - x, 0, x - x1), max(y0 - y, 0, y - y1)
            if math.hypot(dx, dy) < r + MIN_GAP:
                return False
        return all(math.hypot(x - px, y - py) >= r + pr + MIN_GAP for px, py, pr in placed)

    for i in range(tries):
        kind = rng.choice(("crate", "low", "cycle"))
        w, d, h = {"crate": (1.4, 1.4, 1.2), "low": (3.0, 0.6, RAIL), "cycle": (4.2, 1.9, 1.5)}[kind]
        r = math.hypot(w, d) / 2
        turned = rng.random() < 0.5
        for _ in range(60):
            x, y = rng.uniform(-HALF + 2, HALF - 2), rng.uniform(-HALF + 2, HALF - 2)
            if not clear(x, y, r):
                continue
            placed.append((x, y, r))
            add_block(f"Cover_{kind}_{i}", x, y, w, d, h, math.pi / 2 if turned else 0.0, g.cover,
                      g.high if kind == "cycle" else g.low, "low")
            hw, hd = (d, w) if turned else (w, d)
            colour = g.lines[paint.choice(SHADES)]
            if kind == "cycle":
                # A stripe along each side, halfway up.
                g.outline(x - hw / 2, x + hw / 2, y - hd / 2, y + hd / 2, h * 0.45, h * 0.45 + 0.1, 0.03, colour)
            else:
                g.outline(x - hw / 2, x + hw / 2, y - hd / 2, y + hd / 2, h - 0.06, h, 0.03, colour)
            g.n_cover += 1
            break


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    # The colours come from a stream of their own, so that the layout is the seed's whatever they are.
    paint = random.Random(seed + 1)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    br_town.BLOCK = BLOCK
    g = Grid()

    # The platform, with nothing under it, its sides lit just under the floor.
    add_box("Platform", -HALF, HALF, -HALF, HALF, -DEPTH, -0.002, g.ground, g.road)
    g.outline(-HALF, HALF, -HALF, HALF, -0.35, -0.2, 0.0, g.lines["cyan"], out=0.0)
    g.outline(-HALF, HALF, -HALF, HALF, -DEPTH + 0.1, -DEPTH + 0.2, 0.0, g.lines["cyan"], out=0.0)
    # The floor's grid of light.
    steps = int(HALF // GRID)
    for k in range(-steps, steps + 1):
        c = k * GRID
        g.floor_line(c - LINE / 2, c + LINE / 2, -HALF, HALF, g.grid_glow)
        g.floor_line(-HALF, HALF, c - LINE / 2, c + LINE / 2, g.grid_glow)

    # The barrier round the edge, but for a gap in the middle of each side, its top lit.
    for n, (along_x, at) in enumerate(((True, -HALF), (True, HALF), (False, -HALF), (False, HALF))):
        inward = 1 if at < 0 else -1
        for part, (a0, a1) in enumerate(((-HALF, -GAP / 2), (GAP / 2, HALF))):
            b0, b1 = sorted((at, at + inward * 0.4))
            if along_x:
                add_box(f"Barrier_{n}{part}", a0, a1, b0, b1, 0.0, BARRIER, g.struct, g.rail)
                g.line(a0, a1, b0 - 0.02, b1 + 0.02, BARRIER, BARRIER + 0.05, g.lines["cyan"])
            else:
                add_box(f"Barrier_{n}{part}", b0, b1, a0, a1, 0.0, BARRIER, g.struct, g.rail)
                g.line(b0 - 0.02, b1 + 0.02, a0, a1, BARRIER, BARRIER + 0.05, g.lines["cyan"])

    # The bridges, out over the void to the pads, each with a tower on it.
    length, width = BRIDGE
    for n, (dx, dy) in enumerate(((0, -1), (1, 0), (0, 1), (-1, 0))):
        shade = ("orange", "pink", "green", "violet")[n]
        far = HALF + length
        if dx:
            bx0, bx1 = sorted((dx * HALF, dx * far))
            by0, by1 = -width / 2, width / 2
        else:
            bx0, bx1 = -width / 2, width / 2
            by0, by1 = sorted((dy * HALF, dy * far))
        add_box(f"Bridge_{n}", bx0, bx1, by0, by1, -0.5, -0.002, g.ground, g.road)
        if dx:
            g.floor_line(bx0, bx1, by0, by0 + EDGE_LINE, g.lines[shade])
            g.floor_line(bx0, bx1, by1 - EDGE_LINE, by1, g.lines[shade])
        else:
            g.floor_line(bx0, bx0 + EDGE_LINE, by0, by1, g.lines[shade])
            g.floor_line(bx1 - EDGE_LINE, bx1, by0, by1, g.lines[shade])
        px, py = dx * (far + PAD / 2), dy * (far + PAD / 2)
        add_box(f"Pad_{n}", px - PAD / 2, px + PAD / 2, py - PAD / 2, py + PAD / 2, -1.5, -0.002, g.ground, g.road)
        g.outline(px - PAD / 2 + EDGE_LINE, px + PAD / 2 - EDGE_LINE, py - PAD / 2 + EDGE_LINE, py + PAD / 2 - EDGE_LINE,
                  0.0, 0.012, EDGE_LINE, g.lines[shade])
        g.outline(px - PAD / 2, px + PAD / 2, py - PAD / 2, py + PAD / 2, -0.35, -0.2, 0.0, g.lines[shade])
        # The tower: three floors and the roof, toward the pad's far side, its doors turned to the
        # bridge (they face -y and +x, so they are put where those face it, or face along it).
        floors = building(g, rng, "office", px + dx * 3.0, py + dy * 3.0, 12.0, 11.0, 0, f"Tower{n}")
        trim_building(g, "office", px + dx * 3.0, py + dy * 3.0, 12.0, 11.0, floors, shade)
        for sx, sy in ((1, 1), (-1, -1)):
            g.pylon(px + sx * (PAD / 2 - 1.2) if dy else px - dx * (PAD / 2 - 1.2),
                    py + sy * (PAD / 2 - 1.2) if dx else py - dy * (PAD / 2 - 1.2), colour=shade)

    pitch = BLOCK + ROAD
    first = -HALF + ROAD + BLOCK / 2
    middle = BLOCKS // 2
    kinds, weights = zip(*[(k, v[3]) for k, v in KINDS.items()])
    tallest = 0
    for bi in range(BLOCKS):
        for bj in range(BLOCKS):
            cx, cy = first + bi * pitch, first + bj * pitch
            name = f"Block{bi}{bj}"
            # Each block outlined in light, in its building's colour: a district of its own.
            shade = "yellow" if (bi, bj) == (middle, middle) else paint.choice(SHADES)
            g.outline(cx - BLOCK / 2, cx + BLOCK / 2, cy - BLOCK / 2, cy + BLOCK / 2, 0.0, 0.012, EDGE_LINE,
                      g.lines[shade])
            if bi == middle and bj == middle:
                square(g, cx, cy, name)
                continue
            kind = rng.choices(kinds, weights)[0]
            _, wr, dr, _ = KINDS[kind]
            w, d = min(rng.uniform(*wr), BLOCK - 5), min(rng.uniform(*dr), BLOCK - 5)
            ox = rng.uniform(-(BLOCK - w) / 2 + 2, (BLOCK - w) / 2 - 2)
            oy = rng.uniform(-(BLOCK - d) / 2 + 2, (BLOCK - d) / 2 - 2)
            floors = building(g, rng, kind, cx + ox, cy + oy, w, d, 0, name)
            trim_building(g, kind, cx + ox, cy + oy, w, d, floors, shade)
            tallest = max(tallest, floors)

    # Light pylons at every other street corner, and at the square's corners.
    for i in range(BLOCKS + 1):
        for j in range(BLOCKS + 1):
            if (i + j) % 2:
                continue
            x = -HALF + ROAD / 2 + i * pitch
            y = -HALF + ROAD / 2 + j * pitch
            g.pylon(x + 2.6, y + 2.6, colour=paint.choice(SHADES))
    cx = cy = first + middle * pitch
    for sx, sy in ((1, 1), (1, -1), (-1, 1), (-1, -1)):
        g.pylon(cx + sx * (BLOCK / 2 - 1.0), cy + sy * (BLOCK / 2 - 1.0), colour="yellow")

    # The spawns: round the outer street, each facing in.
    for i in range(SPAWNS):
        a = 2 * math.pi * i / SPAWNS
        r = HALF - ROAD / 2
        x = max(-r, min(r, r * 1.5 * math.cos(a)))
        y = max(-r, min(r, r * 1.5 * math.sin(a)))
        marker(f"spawn_{i + 1}", x, y, 0.0, math.atan2(-y, -x))
        g.keep_clear.append((x - 2, x + 2, y - 2, y + 2))
    scatter_cover(g, rng, paint, 140)

    # A camera to look it over in Blender, in the dark.
    bpy.ops.object.camera_add(location=(0, -HALF * 2.4, HALF * 1.4), rotation=(math.radians(58), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.0, 0.0, 0.004, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "br_grid.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {HALF * 2:.0f} m across, buildings={g.n_buildings} tallest={tallest} floors, "
          f"lights={g.n_lights} cover={g.n_cover} trim={g.n_trim} -> {out}")
    if glb:
        br_town.export_glb(os.path.abspath(glb), [g.ground, g.struct, g.cover, g.trim])


if __name__ == "__main__":
    main()
