"""
Capture-the-flag arena in the void: platforms hanging in open space at five heights, joined by
bridges and stairs, with nothing under them, lit by street lights. Mirrored through the middle,
so neither side gets a better layout.

Builds data/arena/ctf_void.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/arena/ctf_void.py -- --seed 7 --out ~/Documents/blender/map/ctf_void.blend --glb data/arena/ctf_void.glb

Or open in Blender > Scripting tab > Run Script.

Layout (top view, x runs along the length, red base at -x and blue at +x):

                                  +--sky--+
                                  |island |  +6
                                  +--||---+
            +--pad--+ ========= high road ========= +--pad--+      +3
            |       |=+  .           ||          .  +=|       |
            +-------+ |   .  stones  ||  stones .     | +-------+
     +--------+       |    +-------------------+      |       +--------+
     |        |==:::::+    |    centre  +---+  |      +:::::==|        |
     | C   F  |  sunken plaza  ===catwalk===|hub|=== sunken plaza  |  F   C |   0 / -3
     |        |==:::::+    |            +---+  |      +:::::==|        |
     +--------+       |    +-------------------+      |       +--------+
            +-------+ |   .  stones  ||  stones .     | +-------+
            |       |=+  .           ||          .  +=|       |
            +--pad--+ ========= high road ========= +--pad--+      +3
                                  +--||---+
                                  |island |  +6
                                  +--sky--+

- The bases (0): the flag toward the back, the computer that opens its firewall by the back
  edge, a watchtower (+3) in each back corner, up a flight of stairs.
- The middle: a catwalk (0) straight from each base to the centre, narrow and with no rail, over
  a sunken plaza (-3) that stairs lead down into from the base and up out of to the centre, on
  either side of the catwalk: the quick way over, or the covered way under.
- The centre (0), round a raised hub (+1.5).
- The flanks: from each base's side, a bridge round and up a flight of stairs onto the high
  road (+3), which runs from pad to pad, each pad with a perch (+4.5) over the road; a flight up
  from the centre meets the road in the middle, and from there another up to a sky island (+6),
  the highest and most open ground there is.
- From each pad's inner corner, stepping stones drop a meter at a time to the centre's corner:
  a way down for the player, too far apart for the droids, who walk round.

Made for Ruptura Systematis (MazeGame/maze), whose capture the flag reads the map's empties: the
game puts each side's flag in a firewall at `flag_red` / `flag_blue`, the computer that opens it
at `computer_red` / `computer_blue`, and its droids at `post_red_1` and on. The game knows this
map is in the void (its `Map::void`): it gives it a black sky, no floor of its own under it, and
surveys its floors with the sky for their ceiling, so a spot's floor, for the droids, is the
first surface down - the catwalk, not the plaza under it, and not the walkway under a street
light's head either, which they walk round. Every raised floor is reached by stairs of 0.25 m
steps. Each street light's
glass is pure magenta, which the game lights with a lamp of its own.
"""

import bpy
import math
import random
import sys
import os

SLAB = 1.0           # thickness of a platform, under its top
DECK = 0.5           # thickness of a bridge
BASE = (30.0, 48.0, -12.0, 12.0)    # blue's base platform (x0, x1, y0, y1); red's is mirrored
CENTRE = 9.0         # half the side of the centre platform
HUB = 3.5            # half the side of the raised hub on it
LOW = -3.0           # the sunken plazas
HIGH = 3.0           # the high road and its pads
SKY = 6.0            # the sky islands
PERCH = 1.5          # how far a perch, the hub, or a watchtower's floor is over what it is on
TOWER = 3.0          # how high a base's watchtowers are
PAD = (14.0, 24.0, 14.0, 24.0)      # blue's north pad; the others are mirrored
LANE = (17.0, 21.0)  # |y| of the high road's edges
ISLAND = (-6.0, 6.0, 30.0, 38.0)    # the north sky island; the south one is mirrored
CATWALK = 0.8        # half the width of the catwalk
LINK = 1.5           # half the width of the narrower bridges and their stairs
LINK_AT = 6.0        # how far along the base's side, from its front, the way round leaves
STEP_RISE = 0.25
STEP_RUN = 0.35
RAIL = 1.0           # height of low cover walls
NUM_COVER = 40       # tries at cover blocks per half (mirrored to the other half)
MIN_GAP = 1.6        # walkable space kept around cover
EDGE = 1.2           # how far cover keeps in from a platform's edge
LAMP_HEIGHT = 4.6    # top of a street light's pole, over its foot
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


def place(rect, sx, sy):
    """`rect` (x0, x1, y0, y1), as on blue's north side, put on the side `sx` (+1 blue, -1 red)
    and `sy` (+1 north, -1 south)."""
    x0, x1, y0, y1 = rect
    x0, x1 = sorted((sx * x0, sx * x1))
    y0, y1 = sorted((sy * y0, sy * y1))
    return (x0, x1, y0, y1)


class Map:
    """What is built so far: the collections and materials it goes in, where cover may go (the
    platforms, each at its height) and the footprints it must stay clear of."""

    def __init__(self):
        self.walk = collection("Platforms")
        self.under = collection("Underside")
        self.struct = collection("Structures")
        self.cover = collection("Cover")
        self.lights = collection("Lights")
        self.deck = material("Deck", (0.34, 0.35, 0.38))
        self.bridge = material("Bridge", (0.28, 0.29, 0.32))
        self.rock = material("Rock", (0.16, 0.15, 0.17))
        self.stairs = material("Stairs", (0.50, 0.48, 0.42))
        self.low = material("CoverLow", (0.60, 0.45, 0.25))
        self.high = material("CoverHigh", (0.40, 0.42, 0.45))
        self.pole = material("LampPole", (0.12, 0.12, 0.14))
        self.head = material("LampHead", (0.20, 0.20, 0.22))
        self.glass = material("LightGlass", (1.0, 0.0, 1.0))  # the game's marker for fixture glass
        # the game's own team colours: blue's droids are cyan
        self.team = {"red": material("TeamRed", (0.80, 0.07, 0.05)),
                     "blue": material("TeamCyan", (0.00, 0.75, 0.85))}
        self.platforms = []
        self.keep_clear = []
        self.n_lights = 0

    def platform(self, name, rect, top, keel, cover=True):
        """A slab with its top at `top` over `rect`, and, if `keel`, a rough inverted pyramid of
        rock under it, its point `keel` meters down, so it reads as a thing that floats."""
        x0, x1, y0, y1 = rect
        add_box(name, x0, x1, y0, y1, top - SLAB, top, self.walk, self.deck)
        if cover:
            self.platforms.append((rect, top))
        if keel:
            w, d = x1 - x0, y1 - y0
            bpy.ops.mesh.primitive_cone_add(vertices=4, radius1=0.0, radius2=1.0, depth=keel,
                                            location=((x0 + x1) / 2, (y0 + y1) / 2, top - SLAB - keel / 2))
            o = bpy.context.active_object
            o.name = f"{name}_Keel"
            o.rotation_euler.z = math.pi / 4
            bpy.ops.object.transform_apply(location=False, rotation=True, scale=False)
            # the cone's square top is sqrt(2) across: stretch it to the slab, a little inside it
            o.scale = (0.92 * w / math.sqrt(2), 0.92 * d / math.sqrt(2), 1.0)
            bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
            o.data.materials.append(self.rock)
            move_to(o, self.under)

    def bridge_deck(self, name, rect, top):
        """A bridge: a thinner slab, with nothing under it."""
        add_box(name, *rect, top - DECK, top, self.walk, self.bridge)

    def flight(self, name, x, y, direction, bottom, top, half_width=LINK, keep=True):
        """Solid stairs from `bottom` up to `top`, whose top step meets the upper floor at (x, y)
        and that descend toward `direction` ('+x', '-x', '+y' or '-y'). Returns how far they
        run."""
        n = round((top - bottom) / STEP_RISE)
        sign = 1 if direction[0] == "+" else -1
        along_x = direction[1] == "x"
        w = half_width
        for i in range(n):
            z = top - i * STEP_RISE                         # step i is i steps down from the top
            a, b = i * STEP_RUN * sign, (i + 1) * STEP_RUN * sign
            if along_x:
                add_box(f"{name}_{i}", x + a, x + b, y - w, y + w, bottom - DECK, z, self.struct, self.stairs)
            else:
                add_box(f"{name}_{i}", x - w, x + w, y + a, y + b, bottom - DECK, z, self.struct, self.stairs)
        run = n * STEP_RUN * sign
        footprint = (*sorted((x, x + run)), y - w, y + w) if along_x else (x - w, x + w, *sorted((y, y + run)))
        if keep:
            self.keep_clear.append(footprint)
        return n * STEP_RUN

    def wall(self, name, rect, bottom, mat=None):
        """A low wall to duck behind."""
        add_box(name, *rect, bottom, bottom + RAIL, self.struct, mat or self.low, "low")

    def street_light(self, x, y, toward, z=0.0):
        """A street light: a pole standing at (x, y) on a floor at `z`, its arm reaching out
        `toward` ('+x', '-x', '+y' or '-y') over the walkway, and a head at the end of it, glazed
        underneath."""
        name = f"StreetLight_{self.n_lights}"
        self.n_lights += 1
        dx, dy = {"+x": (1, 0), "-x": (-1, 0), "+y": (0, 1), "-y": (0, -1)}[toward]
        p, t, top = 0.1, 0.06, z + LAMP_HEIGHT
        add_box(f"{name}_Foot", x - 0.3, x + 0.3, y - 0.3, y + 0.3, z, z + 0.12, self.struct, self.pole)
        add_box(f"{name}_Pole", x - p, x + p, y - p, y + p, z + 0.12, top, self.struct, self.pole)
        # the arm, from the pole out to the head
        ax, ay = x + dx * ARM, y + dy * ARM
        if dx:
            add_box(f"{name}_Arm", x, ax, y - t, y + t, top - 0.12, top, self.struct, self.pole)
        else:
            add_box(f"{name}_Arm", x - t, x + t, y, ay, top - 0.12, top, self.struct, self.pole)
        # the head: a housing longer along the arm, and its glass on the underside
        hl, hw = 0.45, 0.25                                # half its length and width
        hx, hy = (hl, hw) if dx else (hw, hl)
        cx, cy = ax + dx * 0.2, ay + dy * 0.2
        z1, z0 = top + 0.05, top - 0.2
        add_box(f"{name}_Head", cx - hx, cx + hx, cy - hy, cy + hy, z0, z1, self.struct, self.head)
        g = 0.05
        add_box(f"{name}_Glass", cx - hx + g, cx + hx - g, cy - hy + g, cy + hy - g, z0 - 0.04, z0,
                self.lights, self.glass)
        self.keep_clear.append((x - 0.5, x + 0.5, y - 0.5, y + 0.5))


def marker(name, x, y, z, facing_x):
    """An empty where the game puts something, its +x turned to point along `facing_x` (+1 or -1)."""
    empty = bpy.data.objects.new(name, None)
    empty.location = (x, y, z)
    empty.rotation_euler.z = 0.0 if facing_x > 0 else math.pi
    bpy.context.scene.collection.objects.link(empty)


FLIP = {"+x": "-x", "-x": "+x", "+y": "-y", "-y": "+y"}


def turned(direction, sx, sy):
    """`direction`, as on blue's north side, on the side `sx`, `sy` (see `place`)."""
    if direction[1] == "x" and sx < 0 or direction[1] == "y" and sy < 0:
        return FLIP[direction]
    return direction


def build_centre(m):
    """The centre platform round its raised hub, and the stairs up from it to the high road."""
    c = CENTRE
    m.platform("Centre", (-c, c, -c, c), 0.0, 14.0)
    add_box("Hub", -HUB, HUB, -HUB, HUB, -SLAB, PERCH, m.struct, m.deck)
    m.keep_clear.append((-HUB, HUB, -HUB, HUB))
    for i, (x, d) in enumerate(((-HUB, "-x"), (HUB, "+x"))):
        m.flight(f"HubStairs_{i}", x, 0, d, 0.0, PERCH, half_width=1.5)
    # low walls along the hub's two open sides, broken in the middle to step through
    for sy in (1, -1):
        for sx in (1, -1):
            m.wall(f"HubWall_{sy}{sx}", (sx * 0.9, sx * HUB, sy * (HUB - 0.2), sy * HUB), PERCH)
    for sx, sy in ((1, 1), (-1, -1), (1, -1), (-1, 1)):
        m.street_light(sx * (c - 0.6), sy * (c - 0.6), "-x" if sx > 0 else "+x")

    for side, sy in (("N", 1), ("S", -1)):
        # up from the centre's edge to the high road: a deck out from the edge, then the flight
        run = (HIGH - 0.0) / STEP_RISE * STEP_RUN
        foot = LANE[0] - run
        m.bridge_deck(f"CentreLink{side}", place((-LINK, LINK, c, foot), 1, sy), 0.0)
        m.flight(f"CentreStairs{side}", 0, sy * LANE[0], turned("-y", 1, sy), 0.0, HIGH)
        # and on up from the high road to the sky island
        run = (SKY - HIGH) / STEP_RISE * STEP_RUN
        top = ISLAND[2]
        m.bridge_deck(f"IslandLink{side}", place((-LINK, LINK, LANE[1], top - run), 1, sy), HIGH)
        m.flight(f"IslandStairs{side}", 0, sy * top, turned("-y", 1, sy), HIGH, SKY)
        island = place(ISLAND, 1, sy)
        m.platform(f"SkyIsland{side}", island, SKY, 9.0)
        # walls along its far edge and part way down its sides, broken to look out over them
        x0, x1, y0, y1 = ISLAND
        for n, rect in enumerate([(x0, -1.5, y1 - 0.4, y1), (1.5, x1, y1 - 0.4, y1),
                                  (x0, x0 + 0.4, y0 + 4.0, y1 - 0.4), (x1 - 0.4, x1, y0 + 4.0, y1 - 0.4)]):
            m.wall(f"IslandWall{side}_{n}", place(rect, 1, sy), SKY)
        m.street_light(x0 + 0.6, sy * (y0 + 0.6), turned("+y", 1, sy), SKY)
        m.street_light(x1 - 0.6, sy * (y0 + 0.6), turned("+y", 1, sy), SKY)


def build_half(m, team, sx):
    """One side's half: its base, the middle between it and the centre, and its flanks."""
    mat = m.team[team]
    facing = -sx                                       # toward the middle
    base = place(BASE, sx, 1)
    m.platform(f"{team}Base", base, 0.0, 18.0)
    bx0, bx1, by0, by1 = BASE
    # a band of the side's colour round the base's rim, just under its edge
    r = 0.06
    for i, rect in enumerate([(bx0 - r, bx1 + r, by1, by1 + r), (bx0 - r, bx1 + r, by0 - r, by0),
                              (bx0 - r, bx0, by0, by1), (bx1, bx1 + r, by0, by1)]):
        add_box(f"{team}Rim_{i}", *place(rect, sx, 1), -0.45, -0.05, m.struct, mat)

    # a watchtower in each back corner, up a flight of stairs from the base toward the middle
    for side, sy in (("N", 1), ("S", -1)):
        tower = (bx1 - 4.0, bx1, by1 - 4.0, by1)
        add_box(f"{team}Tower{side}", *place(tower, sx, sy), -SLAB, TOWER, m.struct, m.deck)
        m.keep_clear.append(place(tower, sx, sy))
        m.flight(f"{team}TowerStairs{side}", sx * (bx1 - 4.0), sy * (by1 - 2.0), turned("-x", sx, sy),
                 0.0, TOWER, half_width=1.5)
        # walls round its two outer edges, over the void
        for n, rect in enumerate([(bx1 - 0.4, bx1, by1 - 4.0, by1), (bx1 - 4.0, bx1, by1 - 0.4, by1)]):
            m.wall(f"{team}TowerWall{side}_{n}", place(rect, sx, sy), TOWER, mat)

    # the middle: the catwalk straight across at 0, over the sunken plaza, which stairs lead down
    # into from the base's front edge and up out of to the centre's
    front = bx0
    run = (0.0 - LOW) / STEP_RISE * STEP_RUN
    plaza = (CENTRE + run, front - run, -7.0, 7.0)
    m.bridge_deck(f"{team}Catwalk", place((CENTRE, front, -CATWALK, CATWALK), sx, 1), 0.0)
    m.platform(f"{team}Plaza", place(plaza, sx, 1), LOW, 8.0)
    # the catwalk runs over it: nothing tall goes under it
    m.keep_clear.append(place((plaza[0], plaza[1], -CATWALK - 0.5, CATWALK + 0.5), sx, 1))
    for sy in (1, -1):
        m.flight(f"{team}PlazaDown{'N' if sy > 0 else 'S'}", sx * front, sy * 5.0, turned("-x", sx, 1), LOW, 0.0)
        m.flight(f"{team}PlazaUp{'N' if sy > 0 else 'S'}", sx * CENTRE, sy * 5.0, turned("+x", sx, 1), LOW, 0.0)
        m.street_light(sx * (plaza[0] + plaza[1]) / 2, sy * (plaza[3] - 0.6), turned("-y", 1, sy), LOW)

    # the flanks
    px = (PAD[0] + PAD[1]) / 2
    for side, sy in (("N", 1), ("S", -1)):
        m.platform(f"{team}Pad{side}", place(PAD, sx, sy), HIGH, 10.0)
        # the high road, from the pad to the middle, where the other side's half meets it
        lane = (0, PAD[0], *LANE)
        m.platform(f"{team}Road{side}", place(lane, sx, sy), HIGH, 0, cover=False)
        for n, (x, outer) in enumerate(((3.5, True), (10.5, False))):
            y0 = LANE[1] - 0.4 if outer else LANE[0]
            m.wall(f"{team}RoadWall{side}_{n}", place((x - 1.25, x + 1.25, y0, y0 + 0.4), sx, sy), HIGH)
        m.street_light(sx * 7.0, sy * (LANE[1] - 0.3), turned("-y", sx, sy), HIGH)
        # from the base round to the pad: out of the base's side at 0, along to the road's line,
        # then up a flight onto a deck that meets the pad
        run = HIGH / STEP_RISE * STEP_RUN
        turn = BASE[0] + LINK_AT
        stair_top = turn - 4.0 - run
        m.bridge_deck(f"{team}Link{side}_0", place((turn, turn + 4.0, BASE[3], LANE[1]), sx, sy), 0.0)
        m.bridge_deck(f"{team}Link{side}_1", place((stair_top + run, turn, *LANE), sx, sy), 0.0)
        m.flight(f"{team}LinkStairs{side}", sx * stair_top, sy * (LANE[0] + LANE[1]) / 2, turned("+x", sx, sy),
                 0.0, HIGH, half_width=(LANE[1] - LANE[0]) / 2)
        m.bridge_deck(f"{team}Link{side}_2", place((PAD[1], stair_top, *LANE), sx, sy), HIGH)
        m.street_light(sx * (turn + 3.5), sy * (BASE[3] + 4.0), turned("-x", sx, sy))
        # a perch on the pad, looking out over the road
        perch = (px - 2.0, px + 2.0, PAD[3] - 3.5, PAD[3])
        add_box(f"{team}Perch{side}", *place(perch, sx, sy), HIGH - SLAB, HIGH + PERCH, m.struct, m.deck)
        m.keep_clear.append(place(perch, sx, sy))
        m.flight(f"{team}PerchStairs{side}", sx * px, sy * (PAD[3] - 3.5), turned("-y", sx, sy), HIGH, HIGH + PERCH)
        m.wall(f"{team}PerchWall{side}", place((perch[0], perch[1], PAD[3] - 0.2, PAD[3]), sx, sy),
               HIGH + PERCH)
        m.street_light(sx * (PAD[1] - 0.6), sy * (PAD[3] - 0.6), turned("-y", sx, sy), HIGH)
        # stepping stones from the pad's inner corner down to the centre's, a meter at a time
        for n, (at, top) in enumerate(((12.5, HIGH - 1.0), (10.6, HIGH - 2.0))):
            stone = (at - 0.75, at + 0.75, at - 0.75, at + 0.75)
            m.platform(f"{team}Stone{side}_{n}", place(stone, sx, sy), top, 2.5, cover=False)

    # the base's own lights: its front corners, and the middle of its back edge
    for sy in (1, -1):
        m.street_light(sx * (bx0 + 0.6), sy * (by1 - 0.6), turned("-y", 1, sy))
    m.street_light(sx * (bx1 - 0.6), sx * 4.0, turned("-x", sx, 1))

    # the flag on a pad of the side's colour, toward the back of the base
    flag_x = sx * 40.0
    bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=2.5, depth=0.02, location=(flag_x, 0, 0.01))
    flag_pad = bpy.context.active_object
    flag_pad.name = f"{team}FlagPad"
    flag_pad.data.materials.append(mat)
    move_to(flag_pad, m.struct)
    m.keep_clear.append((flag_x - 4.5, flag_x + 4.5, -4.5, 4.5))

    # what the game puts here itself: the flag in its firewall, the computer that opens it (by
    # the back edge, its screen toward the middle), and each droid's post: one at the head of the
    # stairs down to the plaza, one on each pad
    marker(f"flag_{team}", flag_x, 0.0, 0.0, facing)
    marker(f"computer_{team}", sx * 45.5, -7.0, 0.0, facing)
    m.keep_clear.append(place((43.0, 48.0, -9.5, -4.5), sx, 1))
    for n, (x, y, z) in enumerate(((33.0, -5.0, 0.0), (px, 17.0, HIGH), (px, -17.0, HIGH)), start=1):
        marker(f"post_{team}_{n}", sx * x, y, z, facing)
        m.keep_clear.append((sx * x - 1.5, sx * x + 1.5, y - 1.5, y + 1.5))


def scatter_cover(m, rng):
    """Random cover on the blue half, mirrored through the centre onto the red half: only on the
    platforms, in from their edges, at each one's height."""
    placed = []

    def floor_at(x, y, r):
        for (x0, x1, y0, y1), top in m.platforms:
            if x0 + r + EDGE <= x <= x1 - r - EDGE and y0 + r + EDGE <= y <= y1 - r - EDGE:
                return top
        return None

    def clear(x, y, r):
        for x0, x1, y0, y1 in m.keep_clear:
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
            x, y = rng.uniform(0.5, BASE[1]), rng.uniform(-ISLAND[3], ISLAND[3])
            top = floor_at(x, y, r)
            if top is None or floor_at(-x, -y, r) != top or not (clear(x, y, r) and clear(-x, -y, r)):
                continue
            placed += [(x, y, r), (-x, -y, r)]
            kind, mat = ("Low", m.low) if low else ("High", m.high)
            add_block(f"{kind}Cover_{i}_B", x, y, w, d, ht, rot, m.cover, mat, kind.lower(), z=top)
            add_block(f"{kind}Cover_{i}_R", -x, -y, w, d, ht, rot, m.cover, mat, kind.lower(), z=top)
            n_cover += 2
            break
    return n_cover


def main():
    seed, glb, out = get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Map()
    build_centre(m)
    for team, sx in (("red", -1), ("blue", 1)):
        build_half(m, team, sx)
    n_cover = scatter_cover(m, rng)

    # light + camera, to look it over in Blender
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 30), rotation=(math.radians(45), 0, math.radians(30)))
    bpy.context.active_object.data.energy = 3
    bpy.ops.object.camera_add(location=(0, -90, 60), rotation=(math.radians(55), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.0, 0.0, 0.0, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "ctf_void.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} cover={n_cover} street lights={m.n_lights} -> {out}")
    if glb:
        export_glb(os.path.abspath(glb), [m.walk, m.under, m.struct, m.cover])


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
