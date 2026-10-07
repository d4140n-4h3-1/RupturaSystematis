"""
Heist in Helix: a gene lab's campus in the city in the void, every edge traced in light, pink and
violet for the lab and orange for security. A round wall rings the campus in the middle of a round
district; in it stands the lab, two storeys - an atrium to the roof by its door, and at its back a
gallery six metres up, reached by a flight of stairs along the atrium's west wall - and on the
gallery's east end, walled off, the gene vault: a double helix of light turning over a plinth,
six terminals on the walls round it, the heist's vault. Three firewall doors (see
data/heist/door.py) stand in the way, each opened by hacking a computer on its near side:

1. The campus gate, on the south, from a computer on the wall by it, out in the street.
2. The lab's door, from a computer on its front, in the yard.
3. The vault's door, up on the gallery, from a computer on the vault's wall there.

Sentry pods - discs on a column, stairs up to each - stand either side of the gate, a guard on
each; greenhouse domes stand in the yard; specimen tanks glow in the atrium and about the streets,
among helix towers - square towers twisting a quarter turn as they rise. The player comes in at
the south, and gets out at either of two pads, east and west, over bridges with no rail.

Builds data/arena/heist_helix.glb (and a .blend, wherever --out puts it):

    blender -b --python data/arena/heist_helix.py -- --seed 4 --out ~/Documents/blender/map/heist_helix.blend --glb data/arena/heist_helix.glb

Markers as Aurum's (see heist_aurum.py): heist_start, door_N and computer_door_N, loot (the
helix), terminal_N, extract_N, guard_N, drone_N and wave_N - those up on the gallery or a pod
marked at its height. Stairs are of 0.25 m steps, and every coloured line of light glows and is
written to heist_helix.glow.json for the game to light with.
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
from br_nexus import Nexus, Frame, WORLD, octagon, polygon, RISE, RUN, DECK  # noqa: E402
from heist_aurum import wall  # noqa: E402

APOTHEM = 75.0               # the district: a sixteen-sided platform, this far from its middle to each side
DEPTH = 2.5
GAP = 7.0
BRIDGE = (18.0, 4.0)
PAD = 18.0
CAMPUS = (0.0, 8.0, 30.0)            # the campus wall: its middle (x, y), and how far to each side
CAMPUS_WALL = (4.5, 0.6)
LAB = (0.0, 12.0, 16.0, 10.0)        # the lab: middle (x, y), half width, half depth
LAB_HEIGHT = 11.0
WALL = 0.5
GALLERY = (11.0, 6.0)                # where the gallery's south edge is (y), and its floor's height
VAULT_X = 3.0                        # the vault's west wall: the vault is the gallery east of it
STAIRS = (-15.2, -13.6)              # the gallery stairs, across x, along the lab's west wall
POD = (3.5, 4.5)                     # a sentry pod's disc, how far to each side, and its top
PODS = ((-14.0, -33.0), (14.0, -33.0))
START = (0.0, -62.0)


def ring16(apothem, cx=0.0, cy=0.0):
    """A sixteen-sided ring with a side facing each of the four ways, `apothem` to each side."""
    return polygon(apothem / math.cos(math.pi / 16), 16, math.pi / 16, cx, cy)


def helix_tower(m, x, y, r, height, shade, twist):
    """A helix tower: a square tower narrowing a little and twisting `twist` radians as it rises,
    its corners lit all the way up, a ring of light on top."""
    segments = max(3, round(height / 3.0))
    rot = m.paint.uniform(0, math.pi / 2)
    rings = [(polygon(r * (1 - 0.25 * k / segments), 4, rot + twist * k / segments, x, y), height * k / segments)
             for k in range(segments + 1)]
    for k in range(segments):
        (lo, z0), (hi, z1) = rings[k], rings[k + 1]
        m.solid(lo, z0, z1, m.struct, m.dark[m.n % 3], top=hi, cover="high" if k == 0 else "none")
        mid = Vector((x, y, 0))
        for i in range(4):
            p = lo[i] + (lo[i] - mid).normalized() * 0.06
            q = hi[i] + (hi[i] - mid).normalized() * 0.06
            m.line(Vector((p.x, p.y, z0)), Vector((q.x, q.y, z1)), m.lines[shade], light=i % 2 == 0)
    m.ring(rings[-1][0], height - 0.06, m.lines[shade], light=False)
    m.clear.append((x, y, r + 2.0))


def tank(m, x, y, shade, r=1.3, height=3.4):
    """A specimen tank: a column of glowing fluid on a dark base, capped, ringed with light."""
    outer = polygon(r + 0.2, 16, 0.0, x, y)
    m.solid(outer, 0.0, 0.5, m.cover, m.crate, cover="high")
    m.solid(polygon(r, 16, 0.0, x, y), 0.5, height - 0.4, m.cover, m.lines[shade], cover="high")
    m.solid(outer, height - 0.4, height, m.cover, m.crate)
    m.ring(outer, 0.5, m.white, out=0.02, width=0.06, light=False)
    m.ring(outer, height - 0.4, m.white, out=0.02, width=0.06, light=False)
    # Two lines down its sides, to light what is round it with.
    for s in (-1, 1):
        m.line(Vector((x + s * (r + 0.05), y, 0.6)), Vector((x + s * (r + 0.05), y, height - 0.5)), m.lines[shade],
               width=0.06, across=Vector((0, 1, 0)))
    m.clear.append((x, y, r + 1.5))


def greenhouse(m, x, y, r, shade):
    """A greenhouse dome, ribbed with light, a ring of it round its foot: solid, to hide behind."""
    bands = 5
    rings = []
    for b in range(bands + 1):
        a = b * (math.pi / 2) / bands
        rings.append((polygon(max(r * math.cos(a), 0.4), 16, 0.0, x, y), r * math.sin(a)))
    for b in range(bands):
        (lo, z0), (hi, z1) = rings[b], rings[b + 1]
        m.solid(lo, z0, z1, m.struct, m.dark[b % 3], top=hi, cover="high" if b == 0 else "none")
    c = Vector((x, y, 0))
    for i in range(0, 16, 2):
        for b in range(bands):
            (lo, z0), (hi, z1) = rings[b], rings[b + 1]
            p = lo[i] + (lo[i] - c).normalized() * 0.06
            q = hi[i] + (hi[i] - c).normalized() * 0.06
            m.line(Vector((p.x, p.y, z0)), Vector((q.x, q.y, z1)), m.lines[shade], light=False)
    m.ring(rings[0][0], 0.06, m.lines[shade])
    m.ring(rings[2][0], rings[2][1], m.lines[shade], light=False)
    m.clear.append((x, y, r + 2.0))


def sentry_pod(m, x, y, shade):
    """A sentry pod: a disc on a column, a parapet round it, stairs up from the south, a lamp
    under it."""
    r, top = POD
    f = Frame(x, y, 0.0)
    column = octagon(1.2, x, y)
    m.solid(column, 0.0, top - DECK, m.struct, m.dark[1])
    m.fins(column, column, 0.0, top - DECK, m.lines[shade], every=2)
    disc = octagon(r, x, y)
    m.solid(disc, top - DECK, top, m.struct, m.deck)
    m.ring(disc, top - DECK - 0.05, m.lines[shade])
    for i in range(8):
        a, b = disc[i], disc[(i + 1) % 8]
        mid = (a + b) / 2 - f.origin
        inward = -mid.normalized() * 0.15
        south = mid.normalized().dot(Vector((0, -1, 0))) > 0.99
        half = 1.0 / (b - a).length
        m.parapet(a + inward, b + inward, top, m.lines[shade], [(0.5 - half, 0.5 + half)] if south else [])
    # Up from the south: frame turned so its +x runs north.
    n = round(top / RISE)
    g = Frame(x, y, math.pi / 2)
    m.flight(g, -r - n * RUN, -0.8, 0.8, 0.0, top, +1, shade)
    m.glass_pane(f, -0.6, 0.6, -0.6, 0.6, top - DECK - 0.04, top - DECK)
    m.clear.append((x, y, r + 1.5))
    m.clear.append((x, y - r - n * RUN / 2, n * RUN / 2 + 1.5))


def campus(m):
    """The round wall about the campus, its gate on the south, sentry pods either side outside it,
    greenhouses and planters in the yard."""
    cx, cy, apothem = CAMPUS
    height, thick = CAMPUS_WALL
    c = ring16(apothem, cx, cy)
    for i in range(16):
        a, b = c[i], c[(i + 1) % 16]
        mid = (a + b) / 2 - Vector((cx, cy, 0))
        south = mid.normalized().dot(Vector((0, -1, 0))) > 0.99
        wall(m, a, b, height, thick, m.dark[0], "orange", openings=[(b - a).length / 2] if south else [])
    gate_y = cy - apothem
    br_town.marker("door_1", cx, gate_y, 0.0, math.pi / 2)
    br_town.marker("computer_door_1", cx + 4.5, gate_y - thick / 2 - 0.45, 0.0, -math.pi / 2)
    m.clear.append((cx, gate_y - 3.0, 4.0))
    for (x, y) in PODS:
        sentry_pod(m, x, y, "orange")
    for s in (-1, 1):
        greenhouse(m, cx + s * 14.0, cy - 18.0, 4.5, "green")
    # Planters along the walk from the gate to the lab: low, lit green.
    lx, ly, lw, ld = LAB
    for s in (-1, 1):
        for k in range(3):
            y = gate_y + 6.0 + k * 5.0
            f = Frame(cx + s * 4.0, y, 0.0)
            m.box(f, -0.7, 0.7, -1.6, 1.6, 0.0, 0.8, m.cover, m.crate, cover="low")
            m.ring([f.at(-0.7, -1.6), f.at(0.7, -1.6), f.at(0.7, 1.6), f.at(-0.7, 1.6)], 0.78, m.lines["green"],
                   out=0.03, width=0.06, light=k == 1)
    # A ring of light on the yard's floor, just inside the wall.
    m.ring(ring16(apothem - 2.0, cx, cy), 0.006, m.lines["violet"], out=0.0, width=0.1, light=False)


def lab(m):
    """The lab: an atrium by its door, open to the roof; a gallery over the back, its stairs up the
    west wall; and on the gallery's east end, walled off, the vault and the helix in it."""
    lx, ly, lw, ld = LAB
    x0, x1, y0, y1 = lx - lw, lx + lw, ly - ld, ly + ld
    gy, gz = GALLERY
    c = [Vector((x0, y0, 0)), Vector((x1, y0, 0)), Vector((x1, y1, 0)), Vector((x0, y1, 0))]
    shade = "pink"
    wall(m, c[0], c[1], LAB_HEIGHT, WALL, m.dark[1], shade, openings=[lw])
    for a, b in ((c[1], c[2]), (c[2], c[3]), (c[3], c[0])):
        wall(m, a, b, LAB_HEIGHT, WALL, m.dark[1], shade)
    roof = [Vector((x0 - WALL / 2, y0 - WALL / 2, 0)), Vector((x1 + WALL / 2, y0 - WALL / 2, 0)),
            Vector((x1 + WALL / 2, y1 + WALL / 2, 0)), Vector((x0 - WALL / 2, y1 + WALL / 2, 0))]
    m.solid(roof, LAB_HEIGHT, LAB_HEIGHT + 0.4, m.struct, m.deck)
    m.ring(roof, LAB_HEIGHT + 0.34, m.lines[shade])
    m.ring(roof, gz, m.lines["violet"], light=False)
    m.fins(roof, roof, 0.0, LAB_HEIGHT + 0.4, m.lines[shade])
    # Strips of light up its front, either side of the door.
    for x in (-12.0, -8.0, -4.0, 4.0, 8.0, 12.0):
        m.line(Vector((x, y0 - WALL / 2 - 0.04, 0.4)), Vector((x, y0 - WALL / 2 - 0.04, LAB_HEIGHT - 0.6)),
               m.lines["violet"], width=0.08, across=Vector((1, 0, 0)), light=False)
    br_town.marker("door_2", lx, y0, 0.0, math.pi / 2)
    br_town.marker("computer_door_2", lx - 5.0, y0 - WALL / 2 - 0.45, 0.0, -math.pi / 2)

    # The gallery: a deck over the back of the lab, on columns, a parapet along its edge but for
    # where the stairs come up.
    inner = (x0 + WALL / 2, x1 - WALL / 2, y1 - WALL / 2)
    m.box(WORLD, inner[0], inner[1], gy, inner[2], gz - DECK, gz, m.struct, m.deck)
    m.line(Vector((inner[0], gy - 0.05, gz - DECK - 0.05)), Vector((inner[1], gy - 0.05, gz - DECK - 0.05)),
           m.lines["violet"])
    for x in (-9.0, -3.0, 3.0, 9.0):
        m.box(WORLD, x - 0.3, x + 0.3, gy + 0.1, gy + 0.7, 0.0, gz - DECK, m.struct, m.dark[1])
        m.line(Vector((x, gy + 0.06, 0.0)), Vector((x, gy + 0.06, gz - DECK)), m.lines["violet"], width=0.08,
               across=Vector((1, 0, 0)), light=False)
    m.parapet(Vector((STAIRS[1] + 0.1, gy + 0.15, 0)), Vector((VAULT_X, gy + 0.15, 0)), gz, m.lines["violet"])
    # The stairs up the west wall from the front, rising north: the frame turned so its +x is the
    # world's +y and its +y the world's -x.
    run = round(gz / RISE) * RUN
    m.flight(Frame(0.0, 0.0, math.pi / 2), gy - run, -STAIRS[1], -STAIRS[0], 0.0, gz, +1, "violet")

    # The vault: walls on the gallery, its door on the west.
    vy0 = gy + 0.25
    wall(m, Vector((VAULT_X, vy0, 0)), Vector((VAULT_X, y1, 0)), LAB_HEIGHT - gz, 0.8, m.dark[2], "violet",
         openings=[(y1 - vy0) / 2], z0=gz)
    wall(m, Vector((VAULT_X, vy0, 0)), Vector((x1, vy0, 0)), LAB_HEIGHT - gz, WALL, m.dark[2], "violet", z0=gz)
    hy = (vy0 + y1) / 2
    br_town.marker("door_3", VAULT_X, hy, gz, 0.0)
    br_town.marker("computer_door_3", VAULT_X - 0.4 - 0.45, hy - 3.5, gz, math.pi)

    # The helix: two strands of light turning about each other over a plinth, rungs between them.
    kx = (VAULT_X + inner[1]) / 2
    plinth = octagon(1.8, kx, hy)
    m.solid(plinth, gz, gz + 0.5, m.struct, m.dark[0])
    m.ring(plinth, gz + 0.47, m.lines["cyan"], out=0.03, width=0.06)
    m.solid(octagon(0.15, kx, hy), gz + 0.5, LAB_HEIGHT, m.struct, m.rail)
    bottom, top, turns, steps, r = gz + 0.8, LAB_HEIGHT - 0.6, 2.0, 32, 1.1
    strand = lambda k, s: Vector((kx + r * math.cos(s + 2 * math.pi * turns * k / steps),
                                  hy + r * math.sin(s + 2 * math.pi * turns * k / steps),
                                  bottom + (top - bottom) * k / steps))
    for k in range(steps):
        m.line(strand(k, 0.0), strand(k + 1, 0.0), m.lines["pink"], width=0.12)
        m.line(strand(k, math.pi), strand(k + 1, math.pi), m.lines["cyan"], width=0.12)
        if k % 2 == 0:
            m.line(strand(k, 0.0), strand(k, math.pi), m.white, width=0.05, light=False)
    br_town.marker("loot", kx, hy, gz, 0.0)
    for n, (x, y, yaw) in enumerate((
            (kx - 3.5, y1 - WALL / 2 - 0.45, -math.pi / 2), (kx + 3.5, y1 - WALL / 2 - 0.45, -math.pi / 2),
            (kx - 3.5, vy0 + WALL / 2 + 0.45, math.pi / 2), (kx + 3.5, vy0 + WALL / 2 + 0.45, math.pi / 2),
            (inner[1] - 0.45, hy - 3.0, math.pi), (inner[1] - 0.45, hy + 3.0, math.pi))):
        br_town.marker(f"terminal_{n + 1}", x, y, gz, yaw)

    # In the atrium, tanks either side of the way in; under the gallery, benches.
    for x in (-8.0, 8.0):
        tank(m, x, 6.0, "violet")
    for x in (-9.5, -3.0, 6.0, 12.0):
        for y in (14.5, 19.0):
            f = Frame(x, y, 0.0)
            m.box(f, -1.6, 1.6, -0.5, 0.5, 0.0, 1.0, m.cover, m.crate, cover="low")
            m.line(f.at(-1.6, -0.53, 0.97), f.at(1.6, -0.53, 0.97), m.lines["green"], width=0.04, light=False)
            m.line(f.at(-1.6, 0.53, 0.97), f.at(1.6, 0.53, 0.97), m.lines["green"], width=0.04, light=False)
    # Lights: in the roof over the atrium, the gallery and the vault, and under the gallery.
    for x in (-10.0, 0.0, 10.0):
        m.glass_pane(Frame(x, 6.5, 0.0), -0.8, 0.8, -0.4, 0.4, LAB_HEIGHT - 0.06, LAB_HEIGHT - 0.02)
    for x in (-10.0, -3.0):
        m.glass_pane(Frame(x, 16.5, 0.0), -0.8, 0.8, -0.4, 0.4, LAB_HEIGHT - 0.06, LAB_HEIGHT - 0.02)
    for (x, y) in ((kx - 3.5, hy - 3.0), (kx + 3.5, hy - 3.0), (kx - 3.5, hy + 3.0), (kx + 3.5, hy + 3.0)):
        m.glass_pane(Frame(x, y, 0.0), -0.6, 0.6, -0.4, 0.4, LAB_HEIGHT - 0.06, LAB_HEIGHT - 0.02)
    for x in (-9.0, 0.0, 9.0):
        m.glass_pane(Frame(x, 16.5, 0.0), -0.8, 0.8, -0.4, 0.4, gz - DECK - 0.04, gz - DECK)


def district(m):
    """The round platform, its floor's lines, its barrier, and the bridges out to the pads - east
    and west."""
    edge = ring16(APOTHEM)
    m.solid(edge, -DEPTH, -0.002, m.ground, m.floor)
    m.ring(edge, -0.3, m.lines["violet"], out=0.0, light=False)
    # Lines of light along the streets: out from the campus, and rings about it.
    cx, cy, apothem = CAMPUS
    for k in range(16):
        a = k * math.pi / 8 + math.pi / 16
        d = Vector((math.cos(a), math.sin(a), 0))
        p = Vector((cx, cy, 0.006)) + d * (apothem + 2.0)
        q = d * (APOTHEM - 3.0)
        m.line(p, Vector((q.x, q.y, 0.006)), m.circuit, width=0.06, height=0.012, light=False)
    for r in (45.0, 62.0):
        m.ring(ring16(r), 0.006, m.circuit, out=0.0, width=0.06, light=False)
    out_sides = []
    for i in range(16):
        a, b = edge[i], edge[(i + 1) % 16]
        mid = (a + b) / 2
        angle = math.degrees(math.atan2(mid.y, mid.x)) % 360
        open_side = abs(angle - 0) < 1 or abs(angle - 360) < 1 or abs(angle - 180) < 1
        gap = GAP / 2 / (b - a).length
        m.parapet(a - mid.normalized() * 0.3, b - mid.normalized() * 0.3, 0.0, m.lines["violet"],
                  [(0.5 - gap, 0.5 + gap)] if open_side else [])
        if open_side:
            out_sides.append(math.atan2(mid.y, mid.x))
    for n, angle in enumerate(out_sides):
        f = Frame(0.0, 0.0, angle - math.pi / 2)    # local +y outward
        length, width = BRIDGE
        far = APOTHEM + length
        m.box(f, -width / 2, width / 2, APOTHEM - 0.5, far, -0.5, -0.002, m.ground, m.floor)
        for x in (-width / 2 + 0.07, width / 2 - 0.07):
            m.line(f.at(x, APOTHEM, 0.006), f.at(x, far, 0.006), m.lines["violet"], height=0.012, across=f.way(1, 0))
        pad = [f.at(-PAD / 2, far), f.at(PAD / 2, far), f.at(PAD / 2, far + PAD), f.at(-PAD / 2, far + PAD)]
        m.solid(pad, -1.5, -0.002, m.ground, m.floor)
        m.ring(pad, 0.006, m.lines["violet"], out=-0.1)
        middle = f.at(0, far + PAD / 2)
        m.ring(octagon(3.0, middle.x, middle.y), 0.006, m.lines["green"], out=0.0, width=0.18)
        br_town.marker(f"extract_{n + 1}", middle.x, middle.y, 0.0, 0.0)
        for x in (-PAD / 2 + 1.2, PAD / 2 - 1.2):
            m.pylon(f, x, far + PAD - 1.2, shade="green")


def inside(x, y, margin):
    return all(x * math.cos(a) + y * math.sin(a) < APOTHEM - margin for a in (k * math.pi / 8 for k in range(16)))


def city(m, rng):
    """Helix towers, specimen tanks and greenhouses about the streets outside the campus, and
    pylons."""
    cx, cy, apothem = CAMPUS
    for _ in range(30):
        for _ in range(40):
            x, y = rng.uniform(-APOTHEM, APOTHEM), rng.uniform(-APOTHEM, APOTHEM)
            kind = rng.random()
            r = rng.uniform(3.0, 5.5) if kind < 0.6 else (1.5 if kind < 0.8 else rng.uniform(4.0, 6.0))
            if not inside(x, y, 8) or math.hypot(x - cx, y - cy) < apothem + r + 6:
                continue
            if math.hypot(x - START[0], y - START[1]) < r + 8 or abs(y) < r + 4 and abs(x) > 60:
                continue
            if any(math.hypot(x - a, y - b) < r + q + 3.5 for a, b, q in m.clear):
                continue
            if kind < 0.6:
                helix_tower(m, x, y, r, rng.uniform(12.0, 34.0), rng.choice(("pink", "violet", "violet", "cyan")),
                            rng.choice((-1, 1)) * rng.uniform(math.pi / 4, math.pi / 2))
            elif kind < 0.8:
                tank(m, x, y, rng.choice(("pink", "violet", "green")))
            else:
                greenhouse(m, x, y, r, rng.choice(("green", "violet")))
            break
    for k in range(-3, 4):
        for j in range(-3, 4):
            x, y = k * 18.0 + 4.0, j * 18.0 - 4.0
            if not inside(x, y, 6) or math.hypot(x - cx, y - cy) < apothem + 3:
                continue
            if any(math.hypot(x - a, y - b) < q + 1.5 for a, b, q in m.clear):
                continue
            m.pylon(WORLD, x, y, shade=rng.choice(("pink", "violet")))
    # Lights in the yard.
    for (x, y) in ((cx - 20.0, cy + 4.0), (cx + 20.0, cy + 4.0), (cx - 9.0, cy - 22.0), (cx + 9.0, cy - 22.0),
                   (cx, cy + 25.0)):
        m.pylon(WORLD, x, y, shade="orange")


def cover(m, rng, tries):
    """Crates and low walls about the streets and the yard, clear of everything else."""
    lx, ly, lw, ld = LAB
    cx, cy, apothem = CAMPUS
    placed = []
    for _ in range(tries):
        w, d = rng.choice(((1.4, 1.4), (3.0, 0.6), (2.2, 1.2)))
        r = math.hypot(w, d) / 2
        for _ in range(60):
            x, y = rng.uniform(-APOTHEM, APOTHEM), rng.uniform(-APOTHEM, APOTHEM)
            in_lab = abs(x - lx) < lw + 2 and abs(y - ly) < ld + 2
            on_wall = abs(math.hypot(x - cx, y - cy) - apothem) < r + 2.0
            on_walk = abs(x - cx) < 6 and cy - apothem - 6 < y < ly - ld
            if not inside(x, y, 4) or in_lab or on_wall or on_walk:
                continue
            if any(math.hypot(x - a, y - b) < r + q + 1.8 for a, b, q in m.clear + placed):
                continue
            placed.append((x, y, r))
            f = Frame(x, y, rng.choice((0.0, math.pi / 2)))
            m.box(f, -w / 2, w / 2, -d / 2, d / 2, 0.0, 1.2, m.cover, m.crate, cover="low")
            m.ring([f.at(-w / 2, -d / 2), f.at(w / 2, -d / 2), f.at(w / 2, d / 2), f.at(-w / 2, d / 2)], 1.17,
                   m.lines[rng.choice(("pink", "violet", "orange"))], out=0.03, width=0.06)
            break


def markers(m):
    """Where the player comes in, the guards' posts, the drones' patrols and where waves come in."""
    cx, cy, apothem = CAMPUS
    gy, gz = GALLERY
    r, top = POD
    br_town.marker("heist_start", START[0], START[1], 0.0, math.pi / 2)
    posts = [
        # Up on the sentry pods, at their far sides.
        (PODS[0][0], PODS[0][1] + r - 1.2, top), (PODS[1][0], PODS[1][1] + r - 1.2, top),
        # Up on the gallery: by the stairs' head, and by the vault's door.
        (-11.0, 18.0, gz), (-3.0, 14.5, gz),
        # The streets round the campus.
        (-30.0, -28.0, 0.0), (30.0, -28.0, 0.0), (-42.0, 10.0, 0.0), (42.0, 10.0, 0.0), (0.0, cy + apothem + 8, 0.0),
        # The yard, and the lab's ground floor.
        (-7.0, -14.0, 0.0), (8.0, -4.0, 0.0), (-21.0, 20.0, 0.0), (4.0, 8.5, 0.0), (-6.0, 16.5, 0.0),
    ]
    for n, (x, y, z) in enumerate(posts):
        br_town.marker(f"guard_{n + 1}", x, y, z, 0.0)
        m.clear.append((x, y, 2.0))
    br_town.marker("drone_1", -20.0, 0.0, 0.0, 0.0)
    br_town.marker("drone_2", -40.0, -35.0, 0.0, 0.0)
    br_town.marker("drone_3", 38.0, 40.0, 0.0, 0.0)
    m.clear += [(-40.0, -35.0, 3.0), (38.0, 40.0, 3.0)]
    for n, (x, y) in enumerate(((cx, cy - apothem - 14), (cx - apothem - 12, cy), (cx + apothem + 12, cy),
                                (cx, cy + apothem + 14))):
        br_town.marker(f"wave_{n + 1}", x, y, 0.0, 0.0)
        m.clear.append((x, y, 4.0))


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Nexus(random.Random(seed + 1))
    district(m)
    campus(m)
    lab(m)
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
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "heist_helix.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {APOTHEM * 2:.0f} m across, lights={m.n_lights} pieces of light={len(m.pieces)} -> {out}")
    if glb:
        glow_path = os.path.splitext(os.path.abspath(glb))[0] + ".glow.json"
        with open(glow_path, "w") as f:
            json.dump({"intensity": 3.0, "pieces": m.pieces}, f, separators=(",", ":"))
        br_town.export_glb(os.path.abspath(glb), [m.ground, m.struct, m.cover, m.trim])


if __name__ == "__main__":
    main()
