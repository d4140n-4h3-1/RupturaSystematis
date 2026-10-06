"""
Battle royale town: a walled town of streets and blocks, its buildings from one to four floors,
every one of them open to go into - doors on the ground floor, windows on every floor to see and
shoot through, and stairs up inside to each floor, and to the roof of the tallest.

Builds data/arena/br_town.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/arena/br_town.py -- --seed 7 --out ~/Documents/blender/map/br_town.blend --glb data/arena/br_town.glb

Layout (top view): BLOCKS x BLOCKS blocks, BLOCK meters square, with streets ROAD wide between
them and round them, inside a wall. The middle block is an open square, where the closing ring
tends to end; the others each hold a building of one of the kinds below, picked by the seed,
with cover in the yard round it and in the streets.

- House: one or two floors, a door at the front and back.
- Office: three floors, doors on two sides, stairs to the roof.
- Apartment: four floors - the tallest anything is - doors on two sides, stairs to the roof.
- Warehouse: one tall floor, big doors at each end, stacks of crates inside to fight among.
- Square: no building, but low walls, crates and pillars.

It is night: street lights stand along every street, at the middle of a block's side, their arms
out over the road, and a light at each corner of the square.

Every floor above the ground is FLOOR meters over the one below, reached by switchback stairs
of 0.25 m steps in the building's back corner, through an opening in the floor above each
flight; a rail keeps anyone from walking into the opening from beside it. Each floor has a
light in its ceiling: its glass is pure magenta, which the game lights with a lamp of its own.

Made for Ruptura Systematis (MazeGame/maze). The game puts the players down at the empties
`spawn_1` to `spawn_16`, spread round the town.
"""

import bpy
import math
import random
import sys
import os

BLOCKS = 5           # blocks along each side
BLOCK = 28.0         # a block's side, meters
ROAD = 8.0           # a street's width
HALF = (BLOCKS * BLOCK + (BLOCKS + 1) * ROAD) / 2   # half the town's side, inside its wall
TOWN_WALL = 4.0      # how high the wall round the town is
FLOOR = 3.5          # from one floor to the next: 14 steps
SLAB = 0.3           # a floor's thickness
WALL = 0.3           # a building's walls
DOOR = (1.6, 2.4)    # a doorway's width and height
WINDOW = (1.4, 1.0, 2.2)   # a window's width, and its sill and top over its floor
STEP_RISE = 0.25
STEP_RUN = 0.35
FLIGHT = 1.4         # a flight's width
LANDING = 1.6        # the landing at either end of a flight
RAIL = 1.0           # rails, and low cover, are crouched behind
PARAPET = 1.1        # the wall round a roof
LAMP_HEIGHT = 4.6    # top of a street light's pole, over its foot
ARM = 1.3            # how far its arm reaches out over the street
MIN_GAP = 1.8        # walkable space kept round cover
SPAWNS = 16

KINDS = {            # kind: floors, footprint range (x, y), and how likely it is
    "house": ((1, 2), (9.0, 12.0), (8.0, 10.0), 4),
    "office": ((3, 3), (14.0, 18.0), (12.0, 14.0), 3),
    "apartment": ((4, 4), (14.0, 16.0), (12.0, 14.0), 2),
    "warehouse": ((1, 1), (18.0, 22.0), (14.0, 18.0), 2),
}
WAREHOUSE_HEIGHT = 7.0


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
    """A box whose bottom sits at height z, `w` by `d` and turned `rot` about its middle. Made
    from its mesh data straight off: an operator for each of the town's thousands of boxes takes
    longer the more there are already."""
    c, s_ = math.cos(rot), math.sin(rot)
    corners = []
    for zz in (0.0, h):
        for px, py in ((-w / 2, -d / 2), (w / 2, -d / 2), (w / 2, d / 2), (-w / 2, d / 2)):
            corners.append((px * c - py * s_, px * s_ + py * c, zz - h / 2))
    faces = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(corners, [], faces)
    mesh.materials.append(mat)
    mesh.update()
    o = bpy.data.objects.new(name, mesh)
    o.location = (x, y, z + h / 2)
    o["cover"] = cover
    col.objects.link(o)
    return o


def add_box(name, x0, x1, y0, y1, z0, z1, col, mat, cover="none"):
    """An axis-aligned box given by its extents, in either order; nothing if it is empty."""
    x0, x1 = sorted((x0, x1))
    y0, y1 = sorted((y0, y1))
    z0, z1 = sorted((z0, z1))
    if x1 - x0 < 1e-3 or y1 - y0 < 1e-3 or z1 - z0 < 1e-3:
        return None
    return add_block(name, (x0 + x1) / 2, (y0 + y1) / 2, x1 - x0, y1 - y0, z1 - z0, 0, col, mat, cover, z=z0)


def marker(name, x, y, z, facing):
    """An empty where the game puts something, its +x turned `facing` radians round from +x."""
    empty = bpy.data.objects.new(name, None)
    empty.location = (x, y, z)
    empty.rotation_euler.z = facing
    bpy.context.scene.collection.objects.link(empty)


class Town:
    """What is built so far: the collections and materials it goes in, and where there is to
    put cover."""

    def __init__(self):
        self.ground = collection("Ground")
        self.struct = collection("Buildings")
        self.cover = collection("Cover")
        self.lights = collection("Lights")
        self.grass = material("Grass", (0.20, 0.30, 0.16))
        self.road = material("Road", (0.18, 0.18, 0.19))
        self.pavement = material("Pavement", (0.42, 0.41, 0.38))
        self.walls = [material(f"Wall{i}", rgb) for i, rgb in enumerate(
            [(0.62, 0.55, 0.45), (0.50, 0.52, 0.56), (0.58, 0.40, 0.32), (0.66, 0.64, 0.58)])]
        self.floor_mat = material("Floor", (0.36, 0.33, 0.30))
        self.roof = material("Roof", (0.25, 0.25, 0.27))
        self.stairs = material("Stairs", (0.50, 0.48, 0.42))
        self.rail = material("Rail", (0.12, 0.12, 0.14))
        self.low = material("CoverLow", (0.60, 0.45, 0.25))
        self.high = material("CoverHigh", (0.40, 0.42, 0.45))
        self.town_wall = material("TownWall", (0.30, 0.29, 0.28))
        self.glass = material("LightGlass", (1.0, 0.0, 1.0))  # the game's marker for fixture glass
        self.n_lights = 0
        self.n_buildings = 0
        self.n_cover = 0
        # (x0, x1, y0, y1) kept clear of cover: buildings, their doorways, the spawns
        self.keep_clear = []

    def light(self, x, y, ceiling):
        """A light fixture's glass just under a ceiling."""
        add_box(f"Light_{self.n_lights}", x - 0.6, x + 0.6, y - 0.3, y + 0.3, ceiling - 0.08, ceiling,
                self.lights, self.glass)
        self.n_lights += 1

    def street_light(self, x, y, toward):
        """A street light: a pole standing at (x, y), its arm reaching out `toward` ('+x', '-x',
        '+y' or '-y') over the street, and a head at the end of it, glazed underneath."""
        name = f"StreetLight_{self.n_lights}"
        self.n_lights += 1
        dx, dy = {"+x": (1, 0), "-x": (-1, 0), "+y": (0, 1), "-y": (0, -1)}[toward]
        p, t, top = 0.1, 0.06, LAMP_HEIGHT
        add_box(f"{name}_Foot", x - 0.3, x + 0.3, y - 0.3, y + 0.3, 0.0, 0.12, self.struct, self.rail)
        add_box(f"{name}_Pole", x - p, x + p, y - p, y + p, 0.12, top, self.struct, self.rail)
        ax, ay = x + dx * ARM, y + dy * ARM
        if dx:
            add_box(f"{name}_Arm", x, ax, y - t, y + t, top - 0.12, top, self.struct, self.rail)
        else:
            add_box(f"{name}_Arm", x - t, x + t, y, ay, top - 0.12, top, self.struct, self.rail)
        hl, hw = 0.45, 0.25
        hx, hy = (hl, hw) if dx else (hw, hl)
        cx, cy = ax + dx * 0.2, ay + dy * 0.2
        z1, z0 = top + 0.05, top - 0.2
        add_box(f"{name}_Head", cx - hx, cx + hx, cy - hy, cy + hy, z0, z1, self.struct, self.roof)
        g = 0.05
        add_box(f"{name}_Glass", cx - hx + g, cx + hx - g, cy - hy + g, cy + hy - g, z0 - 0.04, z0,
                self.lights, self.glass)
        self.keep_clear.append((x - 0.6, x + 0.6, y - 0.6, y + 0.6))


def wall(town, name, along_x, at, a0, a1, z0, z1, openings, mat):
    """A wall `WALL` thick along x (or along y) at `at` across, from `a0` to `a1`, from `z0` up to
    `z1`, with openings - (middle, width, bottom, top) along it - cut through it."""
    def piece(n, b0, b1, c0, c1):
        if along_x:
            add_box(f"{name}_{n}", b0, b1, at - WALL / 2, at + WALL / 2, c0, c1, town.struct, mat)
        else:
            add_box(f"{name}_{n}", at - WALL / 2, at + WALL / 2, b0, b1, c0, c1, town.struct, mat)
    openings = sorted(openings)
    n, edge = 0, a0
    for middle, width, bottom, top in openings:
        o0, o1 = middle - width / 2, middle + width / 2
        piece(n, edge, o0, z0, z1)                    # the wall up to the opening
        piece(n + 1, o0, o1, z0, bottom)              # under it
        piece(n + 2, o0, o1, top, z1)                 # over it
        n, edge = n + 3, o1
    piece(n, edge, a1, z0, z1)


def slab(town, name, x0, x1, y0, y1, top, hole, mat):
    """A floor slab from x0..x1, y0..y1, its top at `top`, with an opening `hole` (x0, x1, y0,
    y1) through it, if any."""
    if hole is None:
        add_box(name, x0, x1, y0, y1, top - SLAB, top, town.struct, mat)
        return
    hx0, hx1, hy0, hy1 = hole
    add_box(f"{name}_0", x0, hx0, y0, y1, top - SLAB, top, town.struct, mat)
    add_box(f"{name}_1", hx1, x1, y0, y1, top - SLAB, top, town.struct, mat)
    add_box(f"{name}_2", hx0, hx1, y0, hy0, top - SLAB, top, town.struct, mat)
    add_box(f"{name}_3", hx0, hx1, hy1, y1, top - SLAB, top, town.struct, mat)


def flight(town, name, x, y0, y1, bottom, way):
    """Solid stairs one floor up, from `bottom`, across y0..y1, rising along x from `x` in the
    direction `way` (+1 or -1). Returns where along x they end."""
    n = round(FLOOR / STEP_RISE)
    for i in range(n):
        a = x + way * i * STEP_RUN
        b = x + way * (i + 1) * STEP_RUN
        add_box(f"{name}_{i}", a, b, y0, y1, bottom, bottom + (i + 1) * STEP_RISE, town.struct, town.stairs)
    return x + way * n * STEP_RUN


def building(town, rng, kind, cx, cy, w, d, rot_quarter, name):
    """A building of `kind`, `w` by `d`, its middle at (cx, cy). Its front faces the street the
    way `rot_quarter` says (0: -y, 1: +x, 2: +y, 3: -x)."""
    floors_range = KINDS[kind][0]
    floors = rng.randint(*floors_range)
    tall = WAREHOUSE_HEIGHT if kind == "warehouse" else FLOOR
    mat = rng.choice(town.walls)
    x0, x1, y0, y1 = cx - w / 2, cx + w / 2, cy - d / 2, cy + d / 2
    town.keep_clear.append((x0 - 1.0, x1 + 1.0, y0 - 1.0, y1 + 1.0))
    top = floors * tall
    roof_access = kind in ("office", "apartment")
    # The stairs, in the back corner along the back (+y) wall, in two lanes: even flights in the
    # outer lane rising along +x, odd ones in the inner lane coming back along -x.
    run = round(FLOOR / STEP_RISE) * STEP_RUN
    sx = x0 + WALL / 2 + LANDING            # where the flights start along x
    outer = (y1 - WALL / 2 - FLIGHT, y1 - WALL / 2)
    inner = (outer[0] - FLIGHT, outer[0])
    flights = floors - 1 + (1 if roof_access else 0)
    # Each floor's slab, with an opening over the flight coming up through it.
    for k in range(1, floors + 1):
        z = k * tall
        lane = (outer if (k - 1) % 2 == 0 else inner) if k - 1 < flights else None
        hole = (sx - 0.1, sx + run + 0.1, lane[0], lane[1]) if lane else None
        roof = k == floors
        slab(town, f"{name}_Slab{k}", x0, x1, y0, y1, z + (SLAB if roof else 0.0), hole,
             town.roof if roof else town.floor_mat)
    # Walls, floor by floor: doors on the ground floor, windows on every floor.
    sides = [  # along_x, at, from, to, outward: -y, +x, +y, -x
        (True, y0, x0, x1), (False, x1, y0, y1), (True, y1, x0, x1), (False, x0, y0, y1)]
    front = rot_quarter
    for k in range(floors):
        z0, z1 = k * tall, (k + 1) * tall
        for s, (along_x, at, a0, a1) in enumerate(sides):
            length = a1 - a0
            openings = []
            if k == 0 and s in (0, 1):
                # A door in the middle of the front and the side - never the back, which the
                # stairs run along.
                middle = (a0 + a1) / 2
                width = 3.2 if kind == "warehouse" else DOOR[0]
                openings.append((middle, width, z0, z0 + (4.0 if kind == "warehouse" else DOOR[1])))
            # Windows spaced along the wall, but not across the stairs on the back wall.
            n = max(1, int(length // 4.0))
            for i in range(n):
                middle = a0 + length * (i + 0.5) / n
                if any(abs(middle - o[0]) < (o[1] + WINDOW[0]) / 2 + 0.4 for o in openings):
                    continue
                if s == 2 and middle < sx + run + 0.5 and k > 0:
                    continue
                if kind == "warehouse":
                    openings.append((middle, WINDOW[0], z0 + 3.0, z0 + 4.4))
                else:
                    openings.append((middle, WINDOW[0], z0 + WINDOW[1], z0 + WINDOW[2]))
            # The walls run past each other at the corners, overlapping there.
            reach = WALL / 2 if along_x else -WALL / 2
            wall(town, f"{name}_W{k}{s}", along_x, at, a0 - reach, a1 + reach, z0, z1, openings, mat)
        # A light in the middle of each floor's ceiling, clear of the stairs.
        town.light(cx + w * 0.15, cy - d * 0.15, z1)
    # The roof's parapet, broken where the stairs come up.
    if roof_access or kind == "apartment":
        rz = top + SLAB
        for s, (along_x, at, a0, a1) in enumerate(sides):
            reach = WALL / 2 if along_x else -WALL / 2
            wall(town, f"{name}_Parapet{s}", along_x, at, a0 - reach, a1 + reach, rz, rz + PARAPET, [], town.roof)
    # The flights, and the rails along the openings they come up through.
    for f in range(flights):
        bottom = f * tall
        lane = outer if f % 2 == 0 else inner
        if f % 2 == 0:
            flight(town, f"{name}_Flight{f}", sx, lane[0], lane[1], bottom, +1)
        else:
            flight(town, f"{name}_Flight{f}", sx + run, lane[0], lane[1], bottom, -1)
        # On the floor above, a rail along the opening's open side, and across its far end.
        above = bottom + tall + (SLAB if f + 1 == floors else 0.0)
        edge = lane[0] if f % 2 == 0 else lane[1]
        if f % 2 == 0:
            # the opening is in the outer lane: its open side faces the inner lane - but the next
            # flight runs up that side, so the rail only goes where that flight is still low
            add_box(f"{name}_Rail{f}", sx + run * 0.5, sx + run, edge - 0.05, edge + 0.05,
                    above, above + RAIL, town.struct, town.rail)
            add_box(f"{name}_RailEnd{f}", sx - 0.1, sx, lane[0], lane[1], above, above + RAIL, town.struct, town.rail)
        else:
            add_box(f"{name}_Rail{f}", sx, sx + run * 0.5, edge - 0.05, edge + 0.05,
                    above, above + RAIL, town.struct, town.rail)
            # the opening in the inner lane is open to the room on its other side
            add_box(f"{name}_RailRoom{f}", sx - 0.1, sx + run + 0.1, lane[0] - 0.1, lane[0],
                    above, above + RAIL, town.struct, town.rail)
            add_box(f"{name}_RailEnd{f}", sx + run, sx + run + 0.1, lane[0], lane[1], above, above + RAIL,
                    town.struct, town.rail)
    # Inside a warehouse: stacks of crates to fight among.
    if kind == "warehouse":
        for i in range(6):
            bx = rng.uniform(x0 + 3, x1 - 3)
            by = rng.uniform(y0 + 3, y1 - 4)
            h = rng.choice((1.1, 2.4))
            add_block(f"{name}_Crate{i}", bx, by, 2.0, 2.0, h, 0, town.cover,
                      town.low if h < 2 else town.high, "low" if h < 2 else "high")
    # Inside the others: a little cover on the ground floor.
    elif w > 10:
        for i in range(2):
            bx = rng.uniform(x0 + 2.5, x1 - 2.5)
            by = rng.uniform(y0 + 2.5, inner[0] - 1.5)
            add_block(f"{name}_Table{i}", bx, by, 1.8, 0.9, 0.9, 0, town.cover, town.low, "low")
    town.n_buildings += 1
    return floors


def square(town, rng, cx, cy, name):
    """An open square: a ring of low walls and pillars round a raised middle."""
    add_box(f"{name}_Paving", cx - BLOCK / 2, cx + BLOCK / 2, cy - BLOCK / 2, cy + BLOCK / 2, -0.1, 0.0,
            town.ground, town.pavement)
    for i in range(8):
        a = i * math.pi / 4
        r = 8.0
        x, y = cx + r * math.cos(a), cy + r * math.sin(a)
        if i % 2:
            add_block(f"{name}_Pillar{i}", x, y, 1.2, 1.2, 3.0, 0, town.cover, town.high, "high")
        else:
            add_block(f"{name}_Low{i}", x, y, 3.0, 0.6, RAIL, a + math.pi / 2, town.cover, town.low, "low")
    town.keep_clear.append((cx - 10, cx + 10, cy - 10, cy + 10))


def scatter_cover(town, rng, tries):
    """Crates, low walls and parked cars - boxes of a car's size - in the yards and streets, clear
    of the buildings, each other and the spawns."""
    placed = []

    def clear(x, y, r):
        for x0, x1, y0, y1 in town.keep_clear:
            dx, dy = max(x0 - x, 0, x - x1), max(y0 - y, 0, y - y1)
            if math.hypot(dx, dy) < r + MIN_GAP:
                return False
        return all(math.hypot(x - px, y - py) >= r + pr + MIN_GAP for px, py, pr in placed)

    for i in range(tries):
        kind = rng.choice(("crate", "low", "car"))
        w, d, h = {"crate": (1.4, 1.4, 1.2), "low": (3.0, 0.6, RAIL), "car": (4.2, 1.9, 1.5)}[kind]
        r = math.hypot(w, d) / 2
        rot = rng.choice((0.0, math.pi / 2))
        for _ in range(60):
            x, y = rng.uniform(-HALF + 2, HALF - 2), rng.uniform(-HALF + 2, HALF - 2)
            if clear(x, y, r):
                placed.append((x, y, r))
                mat = town.high if kind == "car" else town.low
                add_block(f"Cover_{kind}_{i}", x, y, w, d, h, rot, town.cover, mat, "low")
                town.n_cover += 1
                break


def main():
    seed, glb, out = get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    town = Town()

    # The ground: streets everywhere, and a yard of grass on each block.
    add_box("Streets", -HALF, HALF, -HALF, HALF, -1.0, -0.002, town.ground, town.road)
    pitch = BLOCK + ROAD
    first = -HALF + ROAD + BLOCK / 2
    middle = BLOCKS // 2
    # The wall round the town.
    t = 1.0
    for n, (x0, x1, y0, y1) in enumerate([(-HALF - t, HALF + t, HALF, HALF + t), (-HALF - t, HALF + t, -HALF - t, -HALF),
                                           (-HALF - t, -HALF, -HALF, HALF), (HALF, HALF + t, -HALF, HALF)]):
        add_box(f"TownWall_{n}", x0, x1, y0, y1, 0.0, TOWN_WALL, town.struct, town.town_wall)

    kinds, weights = zip(*[(k, v[3]) for k, v in KINDS.items()])
    tallest = 0
    for bi in range(BLOCKS):
        for bj in range(BLOCKS):
            cx, cy = first + bi * pitch, first + bj * pitch
            name = f"Block{bi}{bj}"
            if bi == middle and bj == middle:
                square(town, rng, cx, cy, name)
                continue
            # The yard: grass a hair over the street's top, so that the two never fight.
            add_box(f"{name}_Yard", cx - BLOCK / 2, cx + BLOCK / 2, cy - BLOCK / 2, cy + BLOCK / 2, -0.1, 0.0,
                    town.ground, town.grass)
            kind = rng.choices(kinds, weights)[0]
            _, wr, dr, _ = KINDS[kind]
            w, d = rng.uniform(*wr), rng.uniform(*dr)
            # Moved a little off the block's middle.
            front = 0
            ox = rng.uniform(-(BLOCK - w) / 2 + 2, (BLOCK - w) / 2 - 2)
            oy = rng.uniform(-(BLOCK - d) / 2 + 2, (BLOCK - d) / 2 - 2)
            floors = building(town, rng, kind, cx + ox, cy + oy, w, d, front, name)
            tallest = max(tallest, floors)

    # Street lights at the middle of each block's south and east sides, on the street just off
    # the block, so that every stretch of street between two corners has one; and one at each
    # corner of the square.
    for bi in range(BLOCKS):
        for bj in range(BLOCKS):
            cx, cy = first + bi * pitch, first + bj * pitch
            if bi == middle and bj == middle:
                for sx, sy in ((1, 1), (1, -1), (-1, 1), (-1, -1)):
                    town.street_light(cx + sx * (BLOCK / 2 + 0.4), cy + sy * (BLOCK / 2 + 0.4), "-x" if sx > 0 else "+x")
                continue
            town.street_light(cx, cy - BLOCK / 2 - 0.4, "-y")
            town.street_light(cx + BLOCK / 2 + 0.4, cy, "+x")
    # And along the town wall, where no block's lights reach: its west and north streets.
    for k in range(BLOCKS):
        c = first + k * pitch
        town.street_light(-HALF + 0.6, c, "+x")
        town.street_light(c, HALF - 0.6, "-y")

    # The spawns: round the streets, on a ring well out from the middle, each facing in.
    for i in range(SPAWNS):
        a = 2 * math.pi * i / SPAWNS
        r = HALF - ROAD / 2
        # Along the outer street, square to the town.
        x = max(-r, min(r, r * 1.5 * math.cos(a)))
        y = max(-r, min(r, r * 1.5 * math.sin(a)))
        marker(f"spawn_{i + 1}", x, y, 0.0, math.atan2(-y, -x))
        town.keep_clear.append((x - 2, x + 2, y - 2, y + 2))
    scatter_cover(town, rng, 160)

    # A sun and a camera, to look it over in Blender.
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 60), rotation=(math.radians(50), 0, math.radians(30)))
    bpy.context.active_object.data.energy = 3
    bpy.ops.object.camera_add(location=(0, -HALF * 2.2, HALF * 1.6), rotation=(math.radians(50), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.55, 0.65, 0.8, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "br_town.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {HALF * 2:.0f} m across, buildings={town.n_buildings} tallest={tallest} floors, "
          f"lights={town.n_lights} cover={town.n_cover} -> {out}")
    if glb:
        export_glb(os.path.abspath(glb), [town.ground, town.struct, town.cover])


def join(col):
    """Joins a collection's meshes into one object (keeping their materials)."""
    objs = [o for o in col.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    objs[0].name = col.name
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def export_glb(path, cols):
    """Exports for the game with the ground, the buildings and the cover each joined into one
    mesh, the lights' glass left a piece each, and the spawns as empties. Runs after the .blend
    is saved, so the saved file keeps every object separate."""
    for col in cols:
        join(col)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_extras=True,
                              export_cameras=False, export_lights=False)
    print(f"[map] exported -> {path}")


if __name__ == "__main__":
    main()
