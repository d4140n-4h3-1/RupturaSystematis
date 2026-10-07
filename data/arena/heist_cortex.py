"""
Heist in Cortex: a data quarter of the city in the void, every edge traced in light, green and
cyan for the data and orange for security. Its data hall stands walled in the middle of an
octagonal district - a long server building, rows of glowing racks inside it - and at the hall's
far end, behind a wall of its own, the core chamber: a glowing octagonal data core with six
terminals round it, the heist's vault. Three firewall doors (see data/heist/door.py) stand in the
way, each opened by hacking a computer on its near side:

1. The gate on the compound's west side, from a computer on its wall, out in the street.
2. The data hall's door, from a computer on its front, in the yard.
3. The core chamber's door, from a computer on its wall, among the racks.

Guard towers stand at the compound's corners, stairs up to a platform with a parapet round it,
where a guard sees a long way; more guards keep posts in the streets, the yard and the hall, and
drones patrol. Server blocks - square towers banded with light - and cooling units stand about the
streets. The player comes in at the south-west, and gets out at either of two pads, north-west
and south-east, over bridges with no rail.

Builds data/arena/heist_cortex.glb (and a .blend, wherever --out puts it):

    blender -b --python data/arena/heist_cortex.py -- --seed 9 --out ~/Documents/blender/map/heist_cortex.blend --glb data/arena/heist_cortex.glb

Markers as Aurum's (see heist_aurum.py): heist_start, door_N and computer_door_N, loot (the
core), terminal_N, extract_N, guard_N, drone_N and wave_N. Stairs are of 0.25 m steps, and every
coloured line of light glows and is written to heist_cortex.glow.json for the game to light with.
"""

import json
import math
import os
import random
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "heist"))
import br_town  # noqa: E402
from br_nexus import Nexus, Frame, WORLD, octagon, polygon, RISE, RUN  # noqa: E402
from heist_aurum import wall  # noqa: E402

APOTHEM = 75.0              # the district: an octagon, this far from its middle to each side
DEPTH = 2.5
GAP = 7.0
BRIDGE = (18.0, 4.0)
PAD = 18.0
YARD = (0.0, 6.0, 30.0, 22.0)        # the compound: its middle (x, y) and half its width and depth
YARD_WALL = (4.5, 0.6)
HALL = (2.0, 7.5, 21.0, 8.0)         # the data hall: middle (x, y), half width, half depth
HALL_HEIGHT = 8.0
CORE_WIDTH = 13.0                    # how wide the core chamber is, at the hall's east end
WALL = 0.5
TOWER = (4.0, 5.0)                   # a guard tower's half side, and its platform's height
# The guard towers' middles: at the compound's corners, clear of the hall and of the walls, their
# stairs included.
TOWERS = ((-24.0, -9.0), (24.0, -9.0), (-24.0, 22.0), (24.0, 22.0))


def server_block(m, x, y, w, d, height, shade):
    """A server block: a square tower, banded with light every few meters up."""
    f = Frame(x, y, 0.0)
    base = [f.at(-w, -d), f.at(w, -d), f.at(w, d), f.at(-w, d)]
    m.solid(base, 0.0, height, m.struct, m.dark[m.n % 3], cover="high")
    m.fins(base, base, 0.0, height, m.lines[shade])
    for z in range(4, int(height), 4):
        m.ring(base, float(z), m.lines[shade], light=False)
    m.ring(base, height - 0.06, m.lines[shade])
    m.clear.append((x, y, math.hypot(w, d) + 2.0))


def cooler(m, x, y, shade):
    """A cooling unit: a squat cylinder, ringed with light, a fan's grille of lines on top."""
    base = polygon(2.2, 16, 0.0, x, y)
    m.solid(base, 0.0, 2.4, m.cover, m.crate, cover="high")
    m.ring(base, 0.1, m.lines[shade])
    m.ring(base, 2.34, m.lines[shade])
    for k in range(3):
        a = k * math.pi / 3
        d = Vector((math.cos(a), math.sin(a), 0)) * 1.9
        m.line(Vector((x, y, 2.42)) - d, Vector((x, y, 2.42)) + d, m.lines[shade], width=0.06, height=0.04, light=False)
    m.clear.append((x, y, 4.0))


def guard_tower(m, x, y, shade):
    """A guard tower: a platform on four legs with a parapet, stairs up its side."""
    h, top = TOWER
    f = Frame(x, y, 0.0)
    for sx in (-1, 1):
        for sy in (-1, 1):
            m.box(f, sx * h - 0.3, sx * h + 0.3, sy * h - 0.3, sy * h + 0.3, 0.0, top - 0.4, m.struct, m.dark[1])
    deck = [f.at(-h, -h), f.at(h, -h), f.at(h, h), f.at(-h, h)]
    m.solid(deck, top - 0.4, top, m.struct, m.deck)
    m.ring(deck, top - 0.46, m.lines[shade])
    # A parapet round it, but for where the stairs come up, on its south side.
    for i in range(4):
        a, b = deck[i], deck[(i + 1) % 4]
        inward = (Vector((x, y, 0)) - (a + b) / 2).normalized() * 0.15
        gaps = [(0.62, 0.92)] if i == 0 else []
        m.parapet(a + inward, b + inward, top, m.lines[shade], gaps)
    # The stairs, up along its south side from the west, landing on the deck's south-east.
    run = round(top / RISE) * RUN
    m.flight(f, h * 0.95 - run, -h - 1.7, -h - 0.1, 0.0, top, +1, shade)
    m.clear.append((x, y, h * 1.5 + 2.5))


def compound(m):
    """The walled yard round the data hall, its gate on the west, guard towers at its corners."""
    cx, cy, hw, hd = YARD
    height, thick = YARD_WALL
    c = [Vector((cx - hw, cy - hd, 0)), Vector((cx + hw, cy - hd, 0)), Vector((cx + hw, cy + hd, 0)),
         Vector((cx - hw, cy + hd, 0))]
    for a, b in ((c[0], c[1]), (c[1], c[2]), (c[2], c[3])):
        wall(m, a, b, height, thick, m.dark[0], "orange")
    # The west side, from north to south, its gate in the middle.
    wall(m, c[3], c[0], height, thick, m.dark[0], "orange", openings=[hd])
    br_town.marker("door_1", cx - hw, cy, 0.0, 0.0)
    br_town.marker("computer_door_1", cx - hw - thick / 2 - 0.45, cy - 5.0, 0.0, math.pi)
    for (x, y) in TOWERS:
        guard_tower(m, x, y, "orange")
    # Cable trenches across the yard: lines of light on the floor, green.
    for k in range(-2, 3):
        m.line(Vector((cx - hw + 1, cy + k * 3.0, 0.006)), Vector((HALL[0] - HALL[2] - 1, cy + k * 3.0, 0.006)),
               m.lines["green"], width=0.1, height=0.012, light=False)


def data_hall(m):
    """The data hall: racks in rows, its door on its south side, the core chamber at its east end
    behind a wall and a door of its own, the core in the middle of that."""
    hx, hy, hw, hd = HALL
    x0, x1, y0, y1 = hx - hw, hx + hw, hy - hd, hy + hd
    c = [Vector((x0, y0, 0)), Vector((x1, y0, 0)), Vector((x1, y1, 0)), Vector((x0, y1, 0))]
    shade = "cyan"
    # Its door on the south side, west of the middle, clear of the core chamber.
    door_x = hw * 0.6
    wall(m, c[0], c[1], HALL_HEIGHT, WALL, m.dark[1], shade, openings=[door_x])
    for a, b in ((c[1], c[2]), (c[2], c[3]), (c[3], c[0])):
        wall(m, a, b, HALL_HEIGHT, WALL, m.dark[1], shade)
    roof = [Vector((x0 - WALL / 2, y0 - WALL / 2, 0)), Vector((x1 + WALL / 2, y0 - WALL / 2, 0)),
            Vector((x1 + WALL / 2, y1 + WALL / 2, 0)), Vector((x0 - WALL / 2, y1 + WALL / 2, 0))]
    m.solid(roof, HALL_HEIGHT, HALL_HEIGHT + 0.4, m.struct, m.deck)
    m.ring(roof, HALL_HEIGHT + 0.34, m.lines[shade])
    m.fins(roof, roof, 0.0, HALL_HEIGHT + 0.4, m.lines[shade])
    br_town.marker("door_2", x0 + door_x, y0, 0.0, math.pi / 2)
    br_town.marker("computer_door_2", x0 + door_x - 5.0, y0 - WALL / 2 - 0.45, 0.0, -math.pi / 2)
    # The core chamber: a wall across the east end, its door in the middle.
    wx = x1 - CORE_WIDTH
    wall(m, Vector((wx, y0, 0)), Vector((wx, y1, 0)), HALL_HEIGHT, 0.8, m.dark[2], "green", openings=[hd])
    br_town.marker("door_3", wx, hy, 0.0, 0.0)
    br_town.marker("computer_door_3", wx - 0.4 - 0.45, hy + 4.5, 0.0, math.pi)
    # The racks, in rows across the hall west of the chamber, gaps between them to creep along.
    for k in range(5):
        x = x0 + 4.0 + k * 4.4
        if x > wx - 3.0:
            break
        for (ya, yb) in ((y0 + 2.5, hy - 2.0), (hy + 2.0, y1 - 2.5)):
            f = Frame(x, (ya + yb) / 2, 0.0)
            half = (yb - ya) / 2
            m.box(f, -0.6, 0.6, -half, half, 0.0, 2.6, m.cover, m.crate, cover="high")
            for z in (0.7, 1.4, 2.1):
                for s in (-1, 1):
                    m.line(f.at(s * 0.63, -half, z), f.at(s * 0.63, half, z), m.lines["green"], width=0.04, height=0.04)
    # The core: an octagonal column of light in the chamber's middle, terminals on its walls.
    kx = (wx + x1) / 2
    core = octagon(1.6, kx, hy)
    m.solid(core, 0.0, HALL_HEIGHT - 1.0, m.struct, m.dark[0], top=octagon(1.0, kx, hy))
    m.fins(core, octagon(1.0, kx, hy), 0.0, HALL_HEIGHT - 1.0, m.lines["green"])
    for z in (1.0, 3.0, 5.0):
        m.ring(octagon(2.6, kx, hy), z, m.lines["cyan"], out=0.0, width=0.1)
    br_town.marker("loot", kx, hy, 0.0, 0.0)
    for n, (x, y, yaw) in enumerate((
            (kx - 3.5, y1 - WALL / 2 - 0.45, -math.pi / 2), (kx + 3.5, y1 - WALL / 2 - 0.45, -math.pi / 2),
            (kx - 3.5, y0 + WALL / 2 + 0.45, math.pi / 2), (kx + 3.5, y0 + WALL / 2 + 0.45, math.pi / 2),
            (x1 - WALL / 2 - 0.45, hy - 3.5, math.pi), (x1 - WALL / 2 - 0.45, hy + 3.5, math.pi))):
        br_town.marker(f"terminal_{n + 1}", x, y, 0.0, yaw)
    # Lights: down the hall, and round the core.
    for k in range(3):
        m.glass_pane(Frame(x0 + 5.0 + k * 7.0, hy, 0.0), -0.8, 0.8, -0.4, 0.4, HALL_HEIGHT - 0.06, HALL_HEIGHT - 0.02)
    for (x, y) in ((kx - 3.5, hy - 4.0), (kx + 3.5, hy - 4.0), (kx - 3.5, hy + 4.0), (kx + 3.5, hy + 4.0)):
        m.glass_pane(Frame(x, y, 0.0), -0.6, 0.6, -0.4, 0.4, HALL_HEIGHT - 0.06, HALL_HEIGHT - 0.02)


def district(m):
    """The octagonal platform, its floor's lines, its barrier, and the bridges out to the pads -
    north-west and south-east."""
    edge = octagon(APOTHEM)
    m.solid(edge, -DEPTH, -0.002, m.ground, m.floor)
    m.ring(edge, -0.3, m.lines["green"], out=0.0, light=False)
    # Lines of light along the streets, every 12 m, both ways, clipped to the octagon.
    reach = APOTHEM - 1.0
    for k in range(-6, 7):
        c = k * 12.0
        span = min(reach, reach * math.sqrt(2) - abs(c))
        if span > 0:
            m.line(Vector((c, -span, 0.006)), Vector((c, span, 0.006)), m.circuit, width=0.06, height=0.012, light=False)
            m.line(Vector((-span, c, 0.006)), Vector((span, c, 0.006)), m.circuit, width=0.06, height=0.012, light=False)
    # The barrier, a gap in the north-west and south-east sides.
    out_sides = []
    for i in range(8):
        a, b = edge[i], edge[(i + 1) % 8]
        mid = (a + b) / 2
        angle = math.degrees(math.atan2(mid.y, mid.x)) % 360
        open_side = abs(angle - 135) < 1 or abs(angle - 315) < 1
        gap = GAP / 2 / (b - a).length
        m.parapet(a - mid.normalized() * 0.3, b - mid.normalized() * 0.3, 0.0, m.lines["green"],
                  [(0.5 - gap, 0.5 + gap)] if open_side else [])
        if open_side:
            out_sides.append(math.radians(angle))
    for n, angle in enumerate(out_sides):
        f = Frame(0.0, 0.0, angle - math.pi / 2)    # local +y outward
        length, width = BRIDGE
        far = APOTHEM + length
        m.box(f, -width / 2, width / 2, APOTHEM - 0.5, far, -0.5, -0.002, m.ground, m.floor)
        for x in (-width / 2 + 0.07, width / 2 - 0.07):
            m.line(f.at(x, APOTHEM, 0.006), f.at(x, far, 0.006), m.lines["green"], height=0.012, across=f.way(1, 0))
        pad = [f.at(-PAD / 2, far), f.at(PAD / 2, far), f.at(PAD / 2, far + PAD), f.at(-PAD / 2, far + PAD)]
        m.solid(pad, -1.5, -0.002, m.ground, m.floor)
        m.ring(pad, 0.006, m.lines["green"], out=-0.1)
        middle = f.at(0, far + PAD / 2)
        m.ring(octagon(3.0, middle.x, middle.y), 0.006, m.lines["green"], out=0.0, width=0.18)
        br_town.marker(f"extract_{n + 1}", middle.x, middle.y, 0.0, 0.0)
        for x in (-PAD / 2 + 1.2, PAD / 2 - 1.2):
            m.pylon(f, x, far + PAD - 1.2, shade="green")


def city(m, rng):
    """Server blocks and cooling units about the streets outside the compound, and pylons."""
    cx, cy, hw, hd = YARD
    start = (-50.0, -42.0)
    inside = lambda x, y: all(x * math.cos(a) + y * math.sin(a) < APOTHEM - 8
                              for a in (k * math.pi / 4 for k in range(8)))
    for _ in range(30):
        for _ in range(40):
            x, y = rng.uniform(-APOTHEM, APOTHEM), rng.uniform(-APOTHEM, APOTHEM)
            w, d = rng.uniform(2.5, 5.0), rng.uniform(2.5, 5.0)
            r = math.hypot(w, d)
            near_yard = abs(x - cx) < hw + r + 6 and abs(y - cy) < hd + r + 6
            if not inside(x, y) or near_yard or math.hypot(x - start[0], y - start[1]) < r + 8:
                continue
            if any(math.hypot(x - a, y - b) < r + q + 3.5 for a, b, q in m.clear):
                continue
            if rng.random() < 0.7:
                server_block(m, x, y, w, d, rng.uniform(10.0, 32.0), rng.choice(("green", "cyan", "cyan", "violet")))
            else:
                cooler(m, x, y, rng.choice(("cyan", "green")))
            break
    for k in range(-3, 4):
        for j in range(-3, 4):
            x, y = k * 18.0 + 4.0, j * 18.0 - 4.0
            near_yard = abs(x - cx) < hw + 3 and abs(y - cy) < hd + 3
            if not inside(x, y) or near_yard or any(math.hypot(x - a, y - b) < q + 1.5 for a, b, q in m.clear):
                continue
            m.pylon(WORLD, x, y, shade=rng.choice(("green", "cyan")))
    # Lights in the yard.
    for (x, y) in ((cx - hw + 6, cy + 6), (cx, cy - hd + 3), (cx, cy + hd - 3)):
        m.pylon(WORLD, x, y, shade="orange")


def cover(m, rng, tries):
    """Crates and low walls about the streets and the yard, clear of everything else."""
    hx, hy, hw, hd = HALL
    placed = []
    inside = lambda x, y: all(x * math.cos(a) + y * math.sin(a) < APOTHEM - 4
                              for a in (k * math.pi / 4 for k in range(8)))
    for _ in range(tries):
        w, d = rng.choice(((1.4, 1.4), (3.0, 0.6), (2.2, 1.2)))
        r = math.hypot(w, d) / 2
        for _ in range(60):
            x, y = rng.uniform(-APOTHEM, APOTHEM), rng.uniform(-APOTHEM, APOTHEM)
            in_hall = abs(x - hx) < hw + 2 and abs(y - hy) < hd + 2
            on_wall = any(abs(abs(x - YARD[0]) - YARD[2]) < 2.5 and abs(y - YARD[1]) < YARD[3] + 2 for _ in [0]) or \
                any(abs(abs(y - YARD[1]) - YARD[3]) < 2.5 and abs(x - YARD[0]) < YARD[2] + 2 for _ in [0])
            if not inside(x, y) or in_hall or on_wall:
                continue
            if any(math.hypot(x - a, y - b) < r + q + 1.8 for a, b, q in m.clear + placed):
                continue
            placed.append((x, y, r))
            f = Frame(x, y, rng.choice((0.0, math.pi / 2)))
            m.box(f, -w / 2, w / 2, -d / 2, d / 2, 0.0, 1.2, m.cover, m.crate, cover="low")
            m.ring([f.at(-w / 2, -d / 2), f.at(w / 2, -d / 2), f.at(w / 2, d / 2), f.at(-w / 2, d / 2)], 1.17,
                   m.lines[rng.choice(("green", "cyan", "orange"))], out=0.03, width=0.06)
            break


def markers(m):
    """Where the player comes in, the guards' posts, the drones' patrols and where waves come in."""
    cx, cy, hw, hd = YARD
    hx, hy, hhw, hhd = HALL
    br_town.marker("heist_start", -50.0, -42.0, 0.0, math.radians(40))
    h, top = TOWER
    # Up the guard towers, at their corners nearest the outside.
    posts = [(x + math.copysign(h - 1, x), y + math.copysign(h - 1, y - cy)) for x, y in TOWERS]
    posts += [
        # The streets west of the gate, and round the compound.
        (cx - hw - 8, cy + 6), (cx - hw - 8, cy - 10), (cx, cy - hd - 8), (cx, cy + hd + 8),
        # The yard, and the hall.
        (cx - hw + 10, cy - 8), (cx - 6, cy - hd + 5), (hx - hhw + 10, hy), (hx + 4, hy - 5),
    ]
    # The first four up on the towers' platforms, the rest on the ground.
    for n, (x, y) in enumerate(posts):
        br_town.marker(f"guard_{n + 1}", x, y, top if n < len(TOWERS) else 0.0, 0.0)
    br_town.marker("drone_1", cx - hw + 6, cy - 6, 0.0, 0.0)
    br_town.marker("drone_2", 30.0, -45.0, 0.0, 0.0)
    for n, (x, y) in enumerate(((cx - hw - 12, cy), (cx, cy - hd - 14), (cx + hw + 12, cy), (cx, cy + hd + 14))):
        br_town.marker(f"wave_{n + 1}", x, y, 0.0, 0.0)


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Nexus(random.Random(seed + 1))
    district(m)
    compound(m)
    data_hall(m)
    markers(m)
    city(m, rng)
    cover(m, rng, 60)

    bpy.ops.object.camera_add(location=(-APOTHEM * 1.5, -APOTHEM * 1.8, APOTHEM * 1.2), rotation=(math.radians(56), 0, math.radians(-38)))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.0, 0.0, 0.004, 1)
    bpy.context.scene.world = world
    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "heist_cortex.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {APOTHEM * 2:.0f} m across, lights={m.n_lights} pieces of light={len(m.pieces)} -> {out}")
    if glb:
        glow_path = os.path.splitext(os.path.abspath(glb))[0] + ".glow.json"
        with open(glow_path, "w") as f:
            json.dump({"intensity": 3.0, "pieces": m.pieces}, f, separators=(",", ":"))
        br_town.export_glb(os.path.abspath(glb), [m.ground, m.struct, m.cover, m.trim])


if __name__ == "__main__":
    main()
