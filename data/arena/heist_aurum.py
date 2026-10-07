"""
Heist in Aurum: a bank district of a city in the void, every edge traced in light (as Nexus is -
see br_nexus.py - and from its helpers). The bank stands walled in the middle of it, its vault
behind three firewall doors (see data/heist/door.py), each opened by hacking a computer on the
near side of it:

1. The gate in the compound's wall, from a computer on the wall by it, out in the street.
2. The bank's own door, from a computer on its front, in the courtyard.
3. The vault's door, from a computer on the vault's wall, in the lobby.

The player comes in at the district's corner farthest from the bank. Once the vault is open,
its terminals are there to hack for credits while security sends waves of guards in from the
district's edges and the gate; the player gets out whenever they like, at either of two pads off
its edges, over bridges with no rail. Guards keep posts in the streets,
the courtyard and the lobby; drones patrol the courtyard and the streets. Towers, pods and domes
stand about the streets to hide behind; pylons light them; the bank's lobby has lights in its
ceiling.

Builds data/arena/heist_aurum.glb (and a .blend, wherever --out puts it):

    blender -b --python data/arena/heist_aurum.py -- --seed 5 --out ~/Documents/blender/map/heist_aurum.blend --glb data/arena/heist_aurum.glb

Markers, the empties the game reads (see src/heist.rs):

- `heist_start`: where the player comes in, facing in.
- `door_1` .. `door_3`: each firewall door, on the floor in the middle of its doorway, its +X the
  way through it, in the order they open; `computer_door_1` ..: each one's computer, just off a
  wall, facing out from it.
- `loot`: the middle of the vault; `terminal_1` .. `terminal_6`: its terminals, each just off a
  wall, facing out from it, to hack for credits once the vault is open.
- `wave_1` ..: where security's waves of guards come in from, once it is.
- `extract_1`, `extract_2`: where to get out with it.
- `guard_1` ..: the guards' posts; `drone_1` ..: the middles of the drones' patrols.

As everywhere, what climbs climbs by stairs of 0.25 m steps; and every coloured line of light
glows, and is written beside the model, in heist_aurum.glow.json, for the game to light with.
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
from br_nexus import Nexus, Frame, WORLD, octagon, polygon  # noqa: E402
from door import OPENING, FRAME_HEIGHT  # noqa: E402

HALF = 70.0                  # half the district's side
DEPTH = 2.5
BARRIER = 1.0
BRIDGE = (18.0, 4.0)
PAD = 18.0
GAP = 7.0
COMPOUND = (10.0, 12.0, 25.0)    # the compound's middle (x, y) and half its side
COMPOUND_WALL = (4.5, 0.6)       # its wall's height and thickness
BANK = (10.0, 18.0, 14.0, 8.0)   # the bank's middle (x, y) and half its width and depth
BANK_HEIGHT = 7.0
VAULT_DEPTH = 6.5                # how deep the vault is, from the bank's back wall
WALL = 0.5
SPAWNS_CLEAR = 3.0


def wall(m, a, b, height, thickness, mat, shade, openings=(), z0=0.0):
    """A wall from `a` to `b` on the floor, `height` tall and `thickness` thick, its top lit in
    `shade`, with door openings - (how far along, in meters, to the opening's middle) - each a
    firewall door's OPENING wide and FRAME_HEIGHT tall."""
    d = b - a
    length = d.length
    f = Frame(a.x, a.y, math.atan2(d.y, d.x))
    t = thickness / 2
    edges = [0.0]
    for at in sorted(openings):
        edges += [at - OPENING / 2, at + OPENING / 2]
    edges.append(length)
    for k in range(0, len(edges), 2):
        if edges[k + 1] - edges[k] > 1e-3:
            m.box(f, edges[k], edges[k + 1], -t, t, z0, z0 + height, m.struct, mat)
    for at in openings:
        m.box(f, at - OPENING / 2, at + OPENING / 2, -t, t, z0 + FRAME_HEIGHT, z0 + height, m.struct, mat)
    for s in (-1, 1):
        m.line(f.at(0, s * (t + 0.03), z0 + height - 0.06), f.at(length, s * (t + 0.03), z0 + height - 0.06),
               m.lines[shade], width=0.06, height=0.12, across=f.way(1, 0))


def tower(m, x, y, r0, r1, height, shade, sides=8):
    """A tower: a tapering prism, its corners lit, a ring of light every so often up it."""
    base, top = polygon(r0, sides, math.pi / sides, x, y), polygon(r1, sides, math.pi / sides, x, y)
    m.solid(base, 0.0, height, m.struct, m.dark[m.n % 3], top=top, cover="high")
    m.fins(base, top, 0.0, height, m.lines[shade])
    for z in range(6, int(height), 6):
        r = r0 + (r1 - r0) * z / height
        m.ring(polygon(r, sides, math.pi / sides, x, y), float(z), m.lines[shade], light=False)
    m.clear.append((x, y, r0 + 2.0))


def compound(m):
    """The bank's compound: a walled square with a gate, a courtyard, and the bank in it."""
    cx, cy, h = COMPOUND
    height, thick = COMPOUND_WALL
    corners = [Vector((cx - h, cy - h, 0)), Vector((cx + h, cy - h, 0)), Vector((cx + h, cy + h, 0)),
               Vector((cx - h, cy + h, 0))]
    # The gate in the middle of its south side.
    wall(m, corners[0], corners[1], height, thick, m.dark[0], "orange", openings=[h])
    for a, b in ((corners[1], corners[2]), (corners[2], corners[3]), (corners[3], corners[0])):
        wall(m, a, b, height, thick, m.dark[0], "orange")
    gate_y = cy - h
    br_town.marker("door_1", cx, gate_y, 0.0, math.pi / 2)
    # Its computer on the wall's outer face, beside the gate, facing the street.
    br_town.marker("computer_door_1", cx - 5.0, gate_y - thick / 2 - 0.45, 0.0, -math.pi / 2)
    # The courtyard: planters and low walls to creep between, lit by pylons.
    for (x, y, w, d) in ((cx - 14, cy - 14, 4.0, 1.2), (cx + 14, cy - 14, 4.0, 1.2), (cx - 18, cy + 4, 1.2, 6.0),
                         (cx + 18, cy + 4, 1.2, 6.0), (cx - 8, cy - 18, 3.0, 0.8), (cx + 8, cy - 18, 3.0, 0.8)):
        f = Frame(x, y, 0.0)
        m.box(f, -w / 2, w / 2, -d / 2, d / 2, 0.0, 1.1, m.cover, m.crate, cover="low")
        m.ring([f.at(-w / 2, -d / 2), f.at(w / 2, -d / 2), f.at(w / 2, d / 2), f.at(-w / 2, d / 2)], 1.07,
               m.lines["yellow"], out=0.03, width=0.06)
    for (x, y) in ((cx - 21, cy - 21), (cx + 21, cy - 21), (cx - 21, cy + 21), (cx + 21, cy + 21)):
        m.pylon(WORLD, x, y, shade="orange")
    m.ring(octagon(4.0, cx, cy - 12.0), 0.006, m.lines["orange"], out=0.0, width=0.12)


def bank(m):
    """The bank: a hall with its door on the courtyard, its lobby, and the vault at the back
    behind a wall of its own, both doors firewall doors."""
    bx, by, hw, hd = BANK
    x0, x1, y0, y1 = bx - hw, bx + hw, by - hd, by + hd
    c = [Vector((x0, y0, 0)), Vector((x1, y0, 0)), Vector((x1, y1, 0)), Vector((x0, y1, 0))]
    shade = "cyan"
    wall(m, c[0], c[1], BANK_HEIGHT, WALL, m.dark[1], shade, openings=[hw])
    for a, b in ((c[1], c[2]), (c[2], c[3]), (c[3], c[0])):
        wall(m, a, b, BANK_HEIGHT, WALL, m.dark[1], shade)
    # The roof, lit round its edge, a light in the lobby's ceiling and the vault's.
    roof = [Vector((x0 - WALL / 2, y0 - WALL / 2, 0)), Vector((x1 + WALL / 2, y0 - WALL / 2, 0)),
            Vector((x1 + WALL / 2, y1 + WALL / 2, 0)), Vector((x0 - WALL / 2, y1 + WALL / 2, 0))]
    m.solid(roof, BANK_HEIGHT, BANK_HEIGHT + 0.4, m.struct, m.deck)
    m.ring(roof, BANK_HEIGHT + 0.34, m.lines[shade])
    m.fins(roof, roof, 0.0, BANK_HEIGHT + 0.4, m.lines[shade])
    # Lights in the lobby's ceiling, and three along the vault's.
    vault_y = by + hd - VAULT_DEPTH / 2
    for (x, y) in ((bx - 8, by - 5), (bx + 8, by - 5), (bx - 8, by + 1), (bx + 8, by + 1),
                   (bx - 7, vault_y), (bx, vault_y), (bx + 7, vault_y)):
        m.glass_pane(Frame(x, y, 0.0), -0.8, 0.8, -0.4, 0.4, BANK_HEIGHT - 0.06, BANK_HEIGHT - 0.02)
    br_town.marker("door_2", bx, y0, 0.0, math.pi / 2)
    br_town.marker("computer_door_2", bx + 5.0, y0 - WALL / 2 - 0.45, 0.0, -math.pi / 2)
    # The vault: a wall across the back of the hall, its door in the middle.
    vy = y1 - VAULT_DEPTH
    wall(m, Vector((x0, vy, 0)), Vector((x1, vy, 0)), BANK_HEIGHT, 0.8, m.dark[2], "yellow", openings=[hw])
    br_town.marker("door_3", bx, vy, 0.0, math.pi / 2)
    br_town.marker("computer_door_3", bx - 6.0, vy - 0.4 - 0.45, 0.0, -math.pi / 2)
    br_town.marker("loot", bx, y1 - VAULT_DEPTH / 2, 0.0, 0.0)
    # The vault's terminals: four on its back wall, two inside its front, clear of the door.
    back = y1 - WALL / 2 - 0.45
    for n, x in enumerate((bx - 8, bx - 4, bx + 4, bx + 8)):
        br_town.marker(f"terminal_{n + 1}", x, back, 0.0, -math.pi / 2)
    front = vy + 0.4 + 0.45
    for n, x in enumerate((bx - 9, bx + 9)):
        br_town.marker(f"terminal_{n + 5}", x, front, 0.0, math.pi / 2)
    # In the vault, racks of it lit gold along either side.
    for s in (-1, 1):
        f = Frame(bx + s * (hw - 1.2), y1 - VAULT_DEPTH / 2, 0.0)
        m.box(f, -0.5, 0.5, -2.2, 2.2, 0.0, 2.2, m.cover, m.crate, cover="high")
        for z in (0.6, 1.3, 2.0):
            m.line(f.at(-s * 0.53, -2.2, z), f.at(-s * 0.53, 2.2, z), m.lines["yellow"], width=0.05, height=0.05)
    # The lobby: a counter across it, and pillars, to hide behind.
    f = Frame(bx, by - 1.0, 0.0)
    m.box(f, -6.0, -1.8, -0.5, 0.5, 0.0, 1.1, m.cover, m.crate, cover="low")
    m.box(f, 1.8, 6.0, -0.5, 0.5, 0.0, 1.1, m.cover, m.crate, cover="low")
    m.line(f.at(-6.0, -0.53, 1.07), f.at(-1.8, -0.53, 1.07), m.lines[shade], width=0.05, height=0.05)
    m.line(f.at(1.8, -0.53, 1.07), f.at(6.0, -0.53, 1.07), m.lines[shade], width=0.05, height=0.05)
    for x in (bx - 10, bx + 10):
        for y in (by - 5, by + 1):
            p = Frame(x, y, 0.0)
            m.box(p, -0.5, 0.5, -0.5, 0.5, 0.0, BANK_HEIGHT, m.struct, m.dark[0])
            m.line(p.at(0.53, 0, 0), p.at(0.53, 0, BANK_HEIGHT), m.lines[shade], width=0.06, across=p.way(0, 1))


def streets(m):
    """The district's floor, edge, barrier and the bridges out to the extraction pads."""
    edge = [Vector((-HALF, -HALF, 0)), Vector((HALF, -HALF, 0)), Vector((HALF, HALF, 0)), Vector((-HALF, HALF, 0))]
    m.solid(edge, -DEPTH, -0.002, m.ground, m.floor)
    m.ring(edge, -0.3, m.lines["cyan"], out=0.0, light=False)
    # Lines of light along the streets, every 10 m.
    for k in range(-6, 7):
        c = k * 10.0
        m.line(Vector((c, -HALF, 0.006)), Vector((c, HALF, 0.006)), m.circuit, width=0.06, height=0.012, light=False)
        m.line(Vector((-HALF, c, 0.006)), Vector((HALF, c, 0.006)), m.circuit, width=0.06, height=0.012, light=False)
    # The barrier, with a gap on the east and north sides for the bridges out.
    for i in range(4):
        a, b = edge[i], edge[(i + 1) % 4]
        inward = -((a + b) / 2).normalized() * 0.3
        gaps = [(0.5 - GAP / 2 / (b - a).length, 0.5 + GAP / 2 / (b - a).length)] if i in (1, 2) else []
        m.parapet(a + inward, b + inward, 0.0, m.lines["cyan"], gaps)
    for n, (dx, dy) in enumerate(((1, 0), (0, 1))):
        f = Frame(0.0, 0.0, math.atan2(dy, dx) - math.pi / 2)   # local +y outward
        length, width = BRIDGE
        far = HALF + length
        m.box(f, -width / 2, width / 2, HALF - 0.5, far, -0.5, -0.002, m.ground, m.floor)
        for x in (-width / 2 + 0.07, width / 2 - 0.07):
            m.line(f.at(x, HALF, 0.006), f.at(x, far, 0.006), m.lines["green"], height=0.012, across=f.way(1, 0))
        pad = [f.at(-PAD / 2, far), f.at(PAD / 2, far), f.at(PAD / 2, far + PAD), f.at(-PAD / 2, far + PAD)]
        m.solid(pad, -1.5, -0.002, m.ground, m.floor)
        m.ring(pad, 0.006, m.lines["green"], out=-0.1)
        middle = f.at(0, far + PAD / 2)
        m.ring(octagon(3.0, middle.x, middle.y), 0.006, m.lines["green"], out=0.0, width=0.18)
        br_town.marker(f"extract_{n + 1}", middle.x, middle.y, 0.0, 0.0)
        for x in (-PAD / 2 + 1.2, PAD / 2 - 1.2):
            m.pylon(f, x, far + PAD - 1.2, shade="green")


def city(m, rng):
    """Towers, pods and domes about the streets outside the compound, and pylons lighting them."""
    cx, cy, h = COMPOUND
    keep = [(cx, cy, h * 1.45 + 4.0), (-58.0, -58.0, 8.0)]
    for _ in range(26):
        for _ in range(40):
            x, y = rng.uniform(-HALF + 8, HALF - 8), rng.uniform(-HALF + 8, HALF - 8)
            r = rng.uniform(3.0, 6.0)
            if any(math.hypot(x - kx, y - ky) < r + kr + 4.0 for kx, ky, kr in keep + m.clear):
                continue
            kind = rng.random()
            if kind < 0.6:
                tower(m, x, y, r, r * rng.uniform(0.35, 0.8), rng.uniform(14.0, 40.0), m.shade())
            else:
                base = polygon(r, 16, 0.0, x, y)
                for b in range(5):
                    a0, a1 = b * math.pi / 10, (b + 1) * math.pi / 10
                    m.solid(polygon(max(r * math.cos(a0), 0.4), 16, 0.0, x, y), r * math.sin(a0), r * math.sin(a1),
                            m.struct, m.dark[1], top=polygon(max(r * math.cos(a1), 0.4), 16, 0.0, x, y),
                            cover="high" if b == 0 else "none")
                m.ring(base, 0.06, m.lines[m.shade()])
                m.clear.append((x, y, r + 2.0))
            break
    for k in range(-3, 4):
        for j in range(-3, 4):
            x, y = k * 20.0 + 5.0, j * 20.0 + 5.0
            if math.hypot(x - cx, y - cy) < h * 1.45 or any(math.hypot(x - a, y - b) < r + 1.5 for a, b, r in m.clear):
                continue
            m.pylon(WORLD, x, y, shade=m.shade())


def cover(m, rng, tries):
    """Crates and low walls about the streets, clear of everything else."""
    cx, cy, h = COMPOUND
    placed = []
    for i in range(tries):
        w, d = rng.choice(((1.4, 1.4), (3.0, 0.6), (4.2, 1.9)))
        r = math.hypot(w, d) / 2
        for _ in range(60):
            x, y = rng.uniform(-HALF + 3, HALF - 3), rng.uniform(-HALF + 3, HALF - 3)
            inside = abs(x - cx) < h + 2 and abs(y - cy) < h + 2
            if inside or any(math.hypot(x - a, y - b) < r + q + 1.8 for a, b, q in m.clear + placed):
                continue
            placed.append((x, y, r))
            f = Frame(x, y, rng.choice((0.0, math.pi / 2)))
            m.box(f, -w / 2, w / 2, -d / 2, d / 2, 0.0, 1.2, m.cover, m.crate, cover="low")
            m.ring([f.at(-w / 2, -d / 2), f.at(w / 2, -d / 2), f.at(w / 2, d / 2), f.at(-w / 2, d / 2)], 1.17,
                   m.lines[m.shade()], out=0.03, width=0.06)
            break


def markers(m):
    """Where the player comes in, and the guards' posts and the drones' patrols."""
    cx, cy, h = COMPOUND
    bx, by, hw, hd = BANK
    br_town.marker("heist_start", -58.0, -58.0, 0.0, math.pi / 4)
    posts = [
        # The streets round the compound.
        (cx - h - 6, cy - h - 6), (cx + h + 6, cy - h - 6), (cx + h + 6, cy + h + 6), (cx - h - 6, cy + h + 6),
        (cx, cy - h - 8),
        # The courtyard.
        (cx - 16, cy - 18), (cx + 16, cy - 18), (cx - 18, cy + 18), (cx + 18, cy + 18),
        # The lobby, and the vault's door.
        (bx - 6, by - 4), (bx + 6, by + 3),
    ]
    for n, (x, y) in enumerate(posts):
        br_town.marker(f"guard_{n + 1}", x, y, 0.0, 0.0)
    br_town.marker("drone_1", cx, cy - 15.0, 0.0, 0.0)
    br_town.marker("drone_2", -30.0, -30.0, 0.0, 0.0)
    # Where security's waves come in from, once the vault is open: outside the gate, and from
    # three sides of the district.
    for n, (x, y) in enumerate(((cx, cy - h - 12), (-50.0, 12.0), (62.0, 30.0), (cx, 58.0))):
        br_town.marker(f"wave_{n + 1}", x, y, 0.0, 0.0)


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Nexus(random.Random(seed + 1))
    m.struct = m.struct   # buildings and walls
    streets(m)
    compound(m)
    bank(m)
    markers(m)
    city(m, rng)
    cover(m, rng, 70)

    bpy.ops.object.camera_add(location=(-HALF * 1.6, -HALF * 1.9, HALF * 1.3), rotation=(math.radians(55), 0, math.radians(-38)))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.0, 0.0, 0.004, 1)
    bpy.context.scene.world = world
    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "heist_aurum.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {HALF * 2:.0f} m across, lights={m.n_lights} pieces of light={len(m.pieces)} -> {out}")
    if glb:
        glow_path = os.path.splitext(os.path.abspath(glb))[0] + ".glow.json"
        with open(glow_path, "w") as f:
            json.dump({"intensity": 3.0, "pieces": m.pieces}, f, separators=(",", ":"))
        br_town.export_glb(os.path.abspath(glb), [m.ground, m.struct, m.cover, m.trim])


if __name__ == "__main__":
    main()
