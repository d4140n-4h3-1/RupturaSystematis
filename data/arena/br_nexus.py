"""
Battle royale in Nexus: a futuristic city on an octagonal platform floating in the void, every
edge of it traced in light, as on the Grid (see br_grid.py) - but nothing in it is a house or an
office. Round the middle, from the inside out:

- The Core: an octagonal spire tapering 48 m into the dark, lit up every edge, with rings of light
  floating round it, on a plinth in a plaza with a ring of light on its floor.
- The skyway: an octagonal ring of walkway 7 m up on lit pillars, parapets along both sides, with
  stairs up to it from the four sides facing the four ways - and lamps under it and on it.
- Four ziggurats on the diagonals, turned to face the Core: three stepped tiers, with stairs up the
  outside from the ground to each terrace and to the top, where a lamp stands. A bridge from the
  skyway lands on each one's second terrace.
- Four domes on the four ways, ribbed with light: solid, to hide behind.
- Eight pods round the rim between them: discs on a single column, stairs up to each, a parapet
  round each, and a lamp under each shining down.
- And off the platform's four sides, bridges with no rails out over the void to pads, each with a
  lit obelisk on it.

The platform's floor is ruled with lines of light running out from the plaza, and rings of them.
Every coloured line of light glows, and - beside the model, in br_nexus.glow.json - is what the game
lights the surroundings with near whoever is about, as on the Grid.

Builds data/arena/br_nexus.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/arena/br_nexus.py -- --seed 3 --out ~/Documents/blender/map/br_nexus.blend --glb data/arena/br_nexus.glb

Everything that climbs climbs by stairs of 0.25 m steps, 0.35 m deep: the game's walk grid and
the droids' feet go up a step at a time, and nothing higher.

Made for Ruptura Systematis (MazeGame/maze). The game puts the players down at the empties
`spawn_1` to `spawn_16`, round the platform's outer ring.
"""

import bmesh
import bpy
import json
import math
import os
import random
import sys

from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import br_town  # noqa: E402
from br_grid import material, glow, PALETTE, WHITE, SEGMENT  # noqa: E402

APOTHEM = 95.0       # from the middle to each of the platform's sides
DEPTH = 2.5          # how thick the platform is
BARRIER = 1.0        # the low wall round its edge, and the parapets
GAP = 8.0            # the gap in the barrier where each bridge leaves
BRIDGE = (18.0, 4.0)
PAD = 22.0
CORE = (9.0, 3.0, 48.0)          # the spire's radius at its foot and its top, and its height
PLINTH = (12.0, 1.0)             # its plinth's radius and height: cover, not a step
SKYWAY = (30.0, 5.0, 7.0)        # the skyway's middle (apothem), width, and the top of its deck
DECK = 0.4                       # how thick a deck is
ZIGGURAT = (56.0, (12.0, 8.0, 4.0), 3.5)   # how far out, each tier's half side, each tier's height
DOME = (68.0, 10.0)              # how far out, and how big
POD = (80.0, 6.5, 4.5)           # how far out, the disc's radius, the top of the disc
RISE, RUN = 0.25, 0.35           # a step
FLIGHT = 1.6                     # a flight's width; the skyway's are wider
TRIM = 0.12
SPAWNS = 16
COVER_REACH = 88.0               # cover goes no further out than this


def turn(v, angle):
    c, s = math.cos(angle), math.sin(angle)
    return Vector((v.x * c - v.y * s, v.x * s + v.y * c, v.z))


class Frame:
    """A place turned by `angle` about the vertical: local coordinates to the world's."""

    def __init__(self, x, y, angle):
        self.origin, self.angle = Vector((x, y, 0.0)), angle

    def at(self, x, y, z=0.0):
        return self.origin + turn(Vector((x, y, z)), self.angle)

    def way(self, x, y, z=0.0):
        return turn(Vector((x, y, z)), self.angle)


WORLD = Frame(0.0, 0.0, 0.0)


def polygon(r, n, rot, cx=0.0, cy=0.0):
    """The corners of a regular polygon `r` from its middle to each corner."""
    return [Vector((cx + r * math.cos(rot + 2 * math.pi * i / n), cy + r * math.sin(rot + 2 * math.pi * i / n), 0.0))
            for i in range(n)]


def octagon(apothem, cx=0.0, cy=0.0, rot=0.0):
    """An octagon with sides facing the four ways and the diagonals, `apothem` from its middle to
    each - or turned `rot` from that."""
    return polygon(apothem / math.cos(math.pi / 8), 8, math.pi / 8 + rot, cx, cy)


class Nexus:
    def __init__(self, paint):
        self.paint = paint
        bc = br_town.collection
        self.ground, self.struct, self.cover = bc("Ground"), bc("Structures"), bc("Cover")
        self.trim, self.lights = bc("Trim"), bc("Lights")
        self.floor = material("Floor", (0.015, 0.018, 0.028), roughness=0.15, metallic=0.6)
        self.dark = [material(f"Shell{i}", rgb) for i, rgb in enumerate(
            [(0.03, 0.035, 0.05), (0.045, 0.05, 0.065), (0.02, 0.025, 0.04)])]
        self.deck = material("Deck", (0.05, 0.055, 0.07))
        self.steps = material("Steps", (0.06, 0.065, 0.08))
        self.rail = material("Rail", (0.02, 0.02, 0.03))
        self.crate = material("Crate", (0.05, 0.05, 0.065))
        self.glass = br_town.material("LightGlass", (1.0, 0.0, 1.0))   # the game's lamp glass
        self.lines = {name: glow(f"Trim{name.title()}", rgb, 3.0) for name, rgb in PALETTE.items()}
        self.circuit = glow("Circuit", PALETTE["cyan"], 1.0)
        self.white = glow("TrimWhite", WHITE, 2.0)
        self.shade_of = {id(m): PALETTE[name] for name, m in self.lines.items()}
        self.pieces = []          # the lines of light, for the game to light with
        self.n = 0
        self.n_lights = 0
        self.n_cover = 0
        self.clear = []           # (x, y, r): kept clear of cover

    def shade(self):
        return self.paint.choice(("cyan", "cyan", "orange", "pink", "yellow", "green", "violet", "red"))

    # Making things.

    def mesh(self, verts, faces, col, mat, cover="none"):
        self.n += 1
        mesh = bpy.data.meshes.new(f"M{self.n}")
        mesh.from_pydata([tuple(v) for v in verts], [], faces)
        bm = bmesh.new()
        bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        bm.to_mesh(mesh)
        bm.free()
        mesh.materials.append(mat)
        mesh.update()
        o = bpy.data.objects.new(f"O{self.n}", mesh)
        o["cover"] = cover
        col.objects.link(o)
        return o

    def solid(self, base, z0, z1, col, mat, top=None, cover="none"):
        """A solid standing on the polygon `base` from z0 to z1 - its top the polygon `top`, if it
        narrows."""
        top = top or base
        n = len(base)
        verts = [Vector((p.x, p.y, z0)) for p in base] + [Vector((p.x, p.y, z1)) for p in top]
        faces = [tuple(range(n)), tuple(range(n, 2 * n))] + [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
        return self.mesh(verts, faces, col, mat, cover)

    def box(self, f, x0, x1, y0, y1, z0, z1, col, mat, cover="none"):
        """A box in frame `f`, given by its extents there."""
        x0, x1 = sorted((x0, x1))
        y0, y1 = sorted((y0, y1))
        return self.solid([f.at(x0, y0), f.at(x1, y0), f.at(x1, y1), f.at(x0, y1)], z0, z1, col, mat, cover=cover)

    def glass_pane(self, f, x0, x1, y0, y1, z0, z1):
        self.box(f, x0, x1, y0, y1, z0, z1, self.lights, self.glass)
        self.n_lights += 1

    def line(self, p, q, mat, width=TRIM, height=None, across=None, light=True):
        """A line of light from `p` to `q`, `width` across - along `across`, if given - and `height`
        thick. A coloured one is lit with, unless not `light`."""
        height = width if height is None else height
        d = (q - p)
        if d.length < 1e-4:
            return
        dn = d.normalized()
        a = across.copy() if across is not None else dn.cross(Vector((0, 0, 1)))
        if a.length < 1e-6:
            a = Vector((1, 0, 0))
        a = (a - dn * a.dot(dn)).normalized()
        b = dn.cross(a)
        ha, hb = a * (width / 2), b * (height / 2)
        ends = [p, q]
        verts = [e + sa * ha + sb * hb for e in ends for sa, sb in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
        self.mesh(verts, [(0, 1, 2, 3), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)],
                  self.trim, mat)
        if light and id(mat) in self.shade_of:
            pieces = max(1, math.ceil(d.length / SEGMENT))
            game = lambda v: [round(v.x, 3), round(v.z, 3), round(-v.y, 3)]
            for i in range(pieces):
                start = p + d * (i / pieces) - ha
                self.pieces.append({"corner": game(start), "edges": [game(d / pieces), game(a * width)],
                                    "colour": list(self.shade_of[id(mat)])})

    def ring(self, corners, z, mat, out=0.06, width=TRIM, light=True):
        """Lines of light round the polygon `corners`, at height z, standing `out` outside it."""
        n = len(corners)
        mid = sum(corners, Vector()) / n
        for i in range(n):
            a, b = corners[i], corners[(i + 1) % n]
            pa = a + (a - mid).normalized() * out
            pb = b + (b - mid).normalized() * out
            self.line(Vector((pa.x, pa.y, z)), Vector((pb.x, pb.y, z)), mat, width, light=light)

    def fins(self, base, top, z0, z1, mat, out=0.06, every=1):
        """Lines of light up the corners of a solid from `base` at z0 to `top` at z1."""
        mid0 = sum(base, Vector()) / len(base)
        mid1 = sum(top, Vector()) / len(top)
        for i in range(0, len(base), every):
            a = base[i] + (base[i] - mid0).normalized() * out
            b = top[i] + (top[i] - mid1).normalized() * out
            self.line(Vector((a.x, a.y, z0)), Vector((b.x, b.y, z1)), mat)

    def flight(self, f, x, y0, y1, bottom, top, way, mat=None):
        """Stairs in frame `f` from `bottom` up to `top`, across y0..y1, rising along x from `x` in
        the direction `way` (+1 or -1), each step solid to the ground; lit along both sides.
        Returns where along x they end."""
        n = round((top - bottom) / RISE)
        for i in range(n):
            a, b = x + way * i * RUN, x + way * (i + 1) * RUN
            self.box(f, a, b, y0, y1, 0.0, bottom + (i + 1) * RISE, self.struct, self.steps)
        end = x + way * n * RUN
        colour = self.lines[mat] if mat else self.white
        for y in (y0, y1):
            self.line(f.at(x, y, bottom + RISE), f.at(end - way * RUN, y, top), colour, width=0.08)
        return end

    def pylon(self, f, x, y, z=0.0, height=6.0, shade="cyan"):
        """A light pylon: a dark mast lit up two sides, a crossbar, and under it the lamp."""
        p, top = 0.12, z + height
        self.box(f, x - 0.35, x + 0.35, y - 0.35, y + 0.35, z, z + 0.15, self.struct, self.rail)
        self.box(f, x - p, x + p, y - p, y + p, z + 0.15, top, self.struct, self.rail)
        for s in (-1, 1):
            self.line(f.at(x + s * (p + 0.015), y, z + 0.15), f.at(x + s * (p + 0.015), y, top), self.lines[shade],
                      width=0.05, across=f.way(0, 1))
        self.box(f, x - 0.9, x + 0.9, y - 0.25, y + 0.25, top, top + 0.18, self.struct, self.rail)
        self.line(f.at(x - 0.9, y - 0.26, top + 0.09), f.at(x + 0.9, y - 0.26, top + 0.09), self.white, light=False)
        self.line(f.at(x - 0.9, y + 0.26, top + 0.09), f.at(x + 0.9, y + 0.26, top + 0.09), self.white, light=False)
        self.glass_pane(f, x - 0.8, x + 0.8, y - 0.2, y + 0.2, top - 0.04, top)
        w = f.at(x, y)
        self.clear.append((w.x, w.y, 0.8))

    def parapet(self, a, b, z, mat, gaps=()):
        """A parapet BARRIER high along a to b standing on z, lit along its top, but for `gaps`
        - (from, to) as shares of the way along."""
        cuts = [(0.0, 1.0)]
        for g0, g1 in sorted(gaps):
            last = cuts.pop()
            cuts += [(last[0], g0), (g1, last[1])]
        d = b - a
        out = Vector((-d.y, d.x, 0)).normalized() * 0.12
        for s0, s1 in cuts:
            if s1 - s0 < 1e-3:
                continue
            p, q = a + d * s0, a + d * s1
            self.solid([p - out, q - out, q + out, p + out], z, z + BARRIER, self.struct, self.rail)
            self.line(Vector((p.x, p.y, z + BARRIER + 0.03)), Vector((q.x, q.y, z + BARRIER + 0.03)), mat,
                      width=0.26, height=0.06)


def platform(m):
    """The octagonal platform, its edges lit, the barrier round it, and the lines on its floor."""
    edge = octagon(APOTHEM)
    m.solid(edge, -DEPTH, -0.002, m.ground, m.floor)
    m.ring(edge, -0.3, m.lines["cyan"], out=0.0, light=False)
    m.ring(edge, -DEPTH + 0.15, m.lines["cyan"], out=0.0, light=False)
    # The barrier, but for the middle of each side facing the four ways.
    for i in range(8):
        a, b = edge[i], edge[(i + 1) % 8]
        mid = (a + b) / 2
        inward = -mid.normalized() * 0.3
        facing = math.degrees(math.atan2(mid.y, mid.x)) % 90
        gap = GAP / (b - a).length / 2
        gaps = [(0.5 - gap, 0.5 + gap)] if abs(facing) < 1 or abs(facing - 90) < 1 else []
        m.parapet(a + inward, b + inward, 0.0, m.lines["cyan"], gaps)
    # Lines of light running out from the plaza, and rings of them.
    for k in range(16):
        a = 2 * math.pi * k / 16 + math.pi / 16
        d = Vector((math.cos(a), math.sin(a), 0))
        m.line(d * 14 + Vector((0, 0, 0.006)), d * (APOTHEM - 1.0) + Vector((0, 0, 0.006)), m.circuit,
               width=0.08, height=0.012, light=False)
    for r in (20.0, 46.0, 76.0, 90.0):
        m.ring(octagon(r), 0.006, m.circuit, out=0.0, width=0.08, light=False)


def core(m):
    """The Core: a tapering octagonal spire on its plinth, edges lit, rings floating round it."""
    r0, r1, h = CORE
    plinth = octagon(PLINTH[0])
    m.solid(plinth, 0.0, PLINTH[1], m.struct, m.dark[0], cover="high")
    m.ring(plinth, PLINTH[1] - 0.06, m.lines["yellow"])
    base, top = octagon(r0), octagon(r1)
    m.solid(base, PLINTH[1], h, m.struct, m.dark[2], top=top)
    shades = list(PALETTE)
    m.fins(base, top, PLINTH[1], h, m.lines["cyan"])
    for k, z in enumerate((10.0, 18.0, 26.0, 34.0, 42.0)):
        r = r0 + (r1 - r0) * (z - PLINTH[1]) / (h - PLINTH[1])
        m.ring(octagon(r + 2.5, rot=math.pi / 8 * (k % 2)), z, m.lines[shades[k + 1]], out=0.0, width=0.2)
        m.ring(octagon(r), z, m.lines[shades[k + 1]], light=False)
    m.ring(octagon(r1), h, m.white, light=False)
    # The plaza's ring on the floor.
    m.ring(octagon(15.0), 0.006, m.lines["yellow"], out=0.0, width=0.18)
    m.clear.append((0.0, 0.0, PLINTH[0] + 2.0))


def skyway(m):
    """The ring of walkway 7 m up on its pillars, stairs up from the four ways, lamps under and
    on it."""
    middle, width, top = SKYWAY
    inner, outer = octagon(middle - width / 2), octagon(middle + width / 2)
    centre = octagon(middle)
    for i in range(8):
        j = (i + 1) % 8
        m.solid([inner[i], inner[j], outer[j], outer[i]], top - DECK, top, m.struct, m.deck)
        # The deck's edges lit underneath and along both sides.
        for a, b in ((inner[i], inner[j]), (outer[i], outer[j])):
            m.line(Vector((a.x, a.y, top - DECK - 0.05)), Vector((b.x, b.y, top - DECK - 0.05)), m.lines["cyan"])
        # Parapets along both edges; on the outside a gap where the stairs or a bridge meet it.
        m.parapet(inner[i] + (centre[i] - inner[i]).normalized() * 0.15,
                  inner[j] + (centre[j] - inner[j]).normalized() * 0.15, top, m.lines["cyan"])
        gap = 1.8 / (outer[j] - outer[i]).length
        m.parapet(outer[i] - (outer[i] - centre[i]).normalized() * 0.15,
                  outer[j] - (outer[j] - centre[j]).normalized() * 0.15, top, m.lines["cyan"],
                  [(0.5 - gap, 0.5 + gap)])
        # A pillar under each corner, lit up its outer face.
        c = centre[i]
        out = c.normalized()
        f = Frame(c.x, c.y, math.atan2(out.y, out.x) - math.pi / 2)
        m.box(f, -0.6, 0.6, -0.6, 0.6, 0.0, top - DECK, m.struct, m.dark[1])
        m.line(f.at(0, 0.66, 0.0), f.at(0, 0.66, top - DECK), m.lines["cyan"], width=0.1, across=f.way(1, 0))
        m.clear.append((c.x, c.y, 1.5))
        # A lamp under the middle of each side, and a short pylon on the deck at each corner.
        mid = (centre[i] + centre[j]) / 2
        g = Frame(mid.x, mid.y, math.atan2(mid.y, mid.x) - math.pi / 2)
        m.glass_pane(g, -0.8, 0.8, -0.3, 0.3, top - DECK - 0.04, top - DECK)
        m.pylon(f, 0.0, 0.0, top, height=3.0, shade="cyan")
    # Stairs up from the ground on the four ways, rising inward to the deck's outer edge.
    for k in range(4):
        angle = k * math.pi / 2
        f = Frame(0.0, 0.0, angle)   # local +x is outward along this way
        edge = middle + width / 2
        n = round(top / RISE)
        m.flight(f, edge + n * RUN, -1.3, 1.3, 0.0, top, -1)
        w = f.at(edge + n * RUN / 2, 0)
        m.clear.append((w.x, w.y, n * RUN / 2 + 2.0))


def ziggurat(m, k):
    """A ziggurat on a diagonal, facing the Core: three tiers, stairs up the outside of each, a
    lamp on top, and a bridge to it from the skyway."""
    out, halves, tall = ZIGGURAT
    angle = math.pi / 4 + k * math.pi / 2
    f = Frame(out * math.cos(angle), out * math.sin(angle), angle - math.pi / 2)   # +y outward
    shade = ("orange", "pink", "green", "violet")[k]
    mat = m.dark[k % 3]
    for t, h in enumerate(halves):
        z0, z1 = t * tall, (t + 1) * tall
        base = [f.at(-h, -h), f.at(h, -h), f.at(h, h), f.at(-h, h)]
        m.solid(base, z0, z1, m.struct, mat)
        m.ring(base, z1 - 0.06, m.lines[shade])
        m.fins(base, base, z0, z1, m.lines[shade])
    # The stairs: from the ground up the outer face, then up each terrace's outer side.
    run = round(tall / RISE) * RUN
    m.flight(f, -7.0, halves[0], halves[0] + FLIGHT, 0.0, tall, +1, shade)
    m.flight(f, 1.0 + run, halves[1], halves[1] + FLIGHT, tall, 2 * tall, -1, shade)
    m.flight(f, -run / 2, halves[2], halves[2] + FLIGHT, 2 * tall, 3 * tall, +1, shade)
    m.pylon(f, 0.0, 0.0, 3 * tall, height=4.0, shade=shade)
    # The bridge from the skyway's outer edge, at its height, to the second terrace.
    middle, width, top = SKYWAY
    near = -(out - (middle + width / 2)) - 0.2
    m.box(f, -1.6, 1.6, near, -halves[1], top - DECK, top, m.struct, m.deck)
    for x in (-1.6, 1.6):
        m.line(f.at(x, near, top + 0.006), f.at(x, -halves[1], top + 0.006), m.lines[shade],
               height=0.012, across=f.way(1, 0))
    m.clear.append((f.origin.x, f.origin.y, halves[0] * 1.42 + FLIGHT + 2.5))


def dome(m, k):
    """A dome on one of the four ways, ribbed with light."""
    out, r = DOME
    angle = k * math.pi / 2
    cx, cy = out * math.cos(angle), out * math.sin(angle)
    shade = m.shade()
    bands = 6
    rings = []
    for b in range(bands + 1):
        a = b * (math.pi / 2) / bands
        rings.append((polygon(max(r * math.cos(a), 0.4), 16, 0.0, cx, cy), r * math.sin(a)))
    for b in range(bands):
        (lo, z0), (hi, z1) = rings[b], rings[b + 1]
        m.solid(lo, z0, z1, m.struct, m.dark[k % 3], top=hi, cover="high" if b == 0 else "none")
    for i in range(0, 16, 2):
        for b in range(bands):
            (lo, z0), (hi, z1) = rings[b], rings[b + 1]
            c = Vector((cx, cy, 0))
            p = lo[i] + (lo[i] - c).normalized() * 0.06
            q = hi[i] + (hi[i] - c).normalized() * 0.06
            m.line(Vector((p.x, p.y, z0)), Vector((q.x, q.y, z1)), m.lines[shade])
    m.ring(rings[0][0], 0.06, m.lines[shade])
    m.ring(rings[2][0], rings[2][1], m.lines[shade])
    m.clear.append((cx, cy, r + 2.0))


def pod(m, k):
    """A pod: a disc on a column, stairs up to it from outward, a parapet round it, a lamp under."""
    out, r, top = POD
    angle = math.pi / 8 + k * math.pi / 4
    f = Frame(out * math.cos(angle), out * math.sin(angle), angle - math.pi / 2)   # +y outward
    shade = m.shade()
    column = [f.at(p.x, p.y) for p in octagon(1.4)]
    m.solid(column, 0.0, top - DECK, m.struct, m.dark[1])
    m.fins(column, column, 0.0, top - DECK, m.lines[shade], every=2)
    disc = [f.at(p.x, p.y) for p in octagon(r * math.cos(math.pi / 8))]
    m.solid(disc, top - DECK, top, m.struct, m.deck)
    m.ring(disc, top - DECK - 0.05, m.lines[shade])
    apothem = r * math.cos(math.pi / 8)
    for i in range(8):
        a, b = disc[i], disc[(i + 1) % 8]
        mid = (a + b) / 2 - f.origin
        inward = -mid.normalized() * 0.15
        outward = f.way(0, 1)
        gaps = [(0.5 - 1.0 / (b - a).length, 0.5 + 1.0 / (b - a).length)] if mid.normalized().dot(outward) > 0.99 else []
        m.parapet(a + inward, b + inward, top, m.lines[shade], gaps)
    n = round(top / RISE)
    m.flight(Frame(f.origin.x, f.origin.y, f.angle + math.pi / 2), apothem + n * RUN, -0.8, 0.8, 0.0, top, -1, shade)
    m.glass_pane(f, -0.6, 0.6, -0.6, 0.6, top - DECK - 0.04, top - DECK)
    m.clear.append((f.origin.x, f.origin.y, r + 1.5))
    w = f.at(0, apothem + n * RUN / 2)
    m.clear.append((w.x, w.y, n * RUN / 2 + 1.5))


def bridges(m):
    """Off each of the four sides, a bridge with no rail out over the void to a pad with a lit
    obelisk and two pylons on it."""
    length, width = BRIDGE
    for k in range(4):
        f = Frame(0.0, 0.0, k * math.pi / 2 - math.pi / 2)   # +y outward along this way
        shade = ("orange", "pink", "green", "violet")[(k + 2) % 4]
        far = APOTHEM + length
        m.box(f, -width / 2, width / 2, APOTHEM - 0.5, far, -0.5, -0.002, m.ground, m.floor)
        for x in (-width / 2 + 0.07, width / 2 - 0.07):
            m.line(f.at(x, APOTHEM, 0.006), f.at(x, far, 0.006), m.lines[shade], height=0.012, across=f.way(1, 0))
        py = far + PAD / 2
        pad = [f.at(-PAD / 2, far), f.at(PAD / 2, far), f.at(PAD / 2, far + PAD), f.at(-PAD / 2, far + PAD)]
        m.solid(pad, -1.5, -0.002, m.ground, m.floor)
        m.ring(pad, 0.006, m.lines[shade], out=-0.1)
        m.ring(pad, -0.3, m.lines[shade], out=0.0, light=False)
        g = Frame(f.at(0, py + 4.0).x, f.at(0, py + 4.0).y, f.angle)
        base, top = [g.at(p.x, p.y) for p in octagon(2.5)], [g.at(p.x, p.y) for p in octagon(0.8)]
        m.solid(base, 0.0, 18.0, m.struct, m.dark[0], top=top)
        m.fins(base, top, 0.0, 18.0, m.lines[shade])
        m.ring(base, 1.0, m.lines[shade])
        for x in (-PAD / 2 + 1.2, PAD / 2 - 1.2):
            m.pylon(f, x, far + 1.2, shade=shade)
        # A few crates on it.
        for x, y in ((-5.0, py - 2.0), (5.0, py + 1.0)):
            m.box(f, x - 0.7, x + 0.7, y - 0.7, y + 0.7, 0.0, 1.2, m.cover, m.crate, cover="low")
            m.ring([f.at(x - 0.7, y - 0.7), f.at(x + 0.7, y - 0.7), f.at(x + 0.7, y + 0.7), f.at(x - 0.7, y + 0.7)],
                   1.17, m.lines[shade], out=0.03, width=0.06)


def lamps(m):
    """Pylons on the ground: between the skyway's stairs and the ziggurats, and out on the
    diagonals near the rim."""
    for k in range(8):
        a = math.pi / 8 + k * math.pi / 4
        m.pylon(WORLD, 44.0 * math.cos(a), 44.0 * math.sin(a), shade=m.shade())
    for k in range(4):
        a = math.pi / 4 + k * math.pi / 2
        m.pylon(WORLD, 86.0 * math.cos(a), 86.0 * math.sin(a), shade=m.shade())


def scatter_cover(m, rng, tries):
    """Crates, low walls and parked light cycles about the platform, clear of everything else and
    each other, each with a glowing edge."""
    placed = []
    inside = lambda x, y: all(x * math.cos(a) + y * math.sin(a) < COVER_REACH
                              for a in (k * math.pi / 4 for k in range(8)))
    for i in range(tries):
        kind = rng.choice(("crate", "low", "cycle"))
        w, d, h = {"crate": (1.4, 1.4, 1.2), "low": (3.0, 0.6, 1.0), "cycle": (4.2, 1.9, 1.5)}[kind]
        r = math.hypot(w, d) / 2
        for _ in range(60):
            x, y = rng.uniform(-COVER_REACH, COVER_REACH), rng.uniform(-COVER_REACH, COVER_REACH)
            if not inside(x, y):
                continue
            if any(math.hypot(x - cx, y - cy) < r + cr + 1.8 for cx, cy, cr in m.clear + placed):
                continue
            placed.append((x, y, r))
            f = Frame(x, y, rng.uniform(0, math.pi))
            m.box(f, -w / 2, w / 2, -d / 2, d / 2, 0.0, h, m.cover, m.crate, cover="low")
            corners = [f.at(-w / 2, -d / 2), f.at(w / 2, -d / 2), f.at(w / 2, d / 2), f.at(-w / 2, d / 2)]
            m.ring(corners, h * 0.45 if kind == "cycle" else h - 0.03, m.lines[m.shade()], out=0.03, width=0.06)
            m.n_cover += 1
            break


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Nexus(random.Random(seed + 1))
    platform(m)
    core(m)
    skyway(m)
    for k in range(4):
        ziggurat(m, k)
        dome(m, k)
    for k in range(8):
        pod(m, k)
    bridges(m)
    lamps(m)
    # The spawns: round the outer ring, between the pods, each facing in.
    for i in range(SPAWNS):
        a = math.pi / 16 + 2 * math.pi * i / SPAWNS
        x, y = 86.0 * math.cos(a), 86.0 * math.sin(a)
        br_town.marker(f"spawn_{i + 1}", x, y, 0.0, math.atan2(-y, -x))
        m.clear.append((x, y, 2.5))
    scatter_cover(m, rng, 110)

    bpy.ops.object.camera_add(location=(0, -APOTHEM * 2.3, APOTHEM * 1.3), rotation=(math.radians(58), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.0, 0.0, 0.004, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "br_nexus.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {APOTHEM * 2:.0f} m across, lights={m.n_lights} cover={m.n_cover} "
          f"pieces of light={len(m.pieces)} -> {out}")
    if glb:
        glow_path = os.path.splitext(os.path.abspath(glb))[0] + ".glow.json"
        with open(glow_path, "w") as f:
            json.dump({"intensity": 3.0, "pieces": m.pieces}, f, separators=(",", ":"))
        print(f"[map] {len(m.pieces)} pieces of light -> {glow_path}")
        br_town.export_glb(os.path.abspath(glb), [m.ground, m.struct, m.cover, m.trim])


if __name__ == "__main__":
    main()
