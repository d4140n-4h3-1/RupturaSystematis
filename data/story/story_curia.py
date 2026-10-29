"""
Curia: the capital north of Urbs, across the void - a model of its own, apart from Urbs, the two
joined by a road. It is laid out as Urbs is, fourteen blocks by fourteen, each 48 m square between
streets 16 m wide, on a platform 912 m across. Its middle street, south and north, carries on Urbs'
middle avenue: a bridge 40 m long goes from Urbs' north edge to Curia's south edge, through the Gate
of Nations.

It is built where it stands in Urbs' frame, its middle 952 m north of Urbs', so that the two models
line up as they are, without moving either.

Its buildings are its own, the forms of a seat of government, in white stone and gold light, the
middle avenue its axis:

- The Gate of Nations (place_gate) where the bridge comes in: a triumphal arch over the avenue.
- The Mall (place_mall), up the avenue from the gate: lawns either side of it, reflecting pools
  down them, obelisks edged in gold, rows of trees and gold lamps.
- The Parliament (place_parliament) across the avenue in the middle, the streets under it closed:
  a rotunda under a dome ribbed in gold behind a ring of columns, a portico before it, a wing either
  side, all on a podium climbed by stairs, a plaza with fountains before it.
- The Gardens of the Republic behind it, and at the head of the avenue the Tower of the Republic
  (place_tower), white and octagonal, stepping in as it rises, crowned in gold.
- The Supreme Court (place_court) west of the Parliament, a temple on a stepped base, and the State
  Archive (place_archive) east of it, a rotunda behind its colonnade.
- The Plazas of Nations (place_nations_west, place_nations_east): circles of flags round an obelisk.
- Urbs' railway, come on over the bridge and through the Gate up the avenue to a station by the
  first plazas (place_station_curia), where the north train ends its line.
- The ministries all round them, colonnaded along their streets, flags on their roofs; the embassies
  in the west (place_embassies), pavilions in gardens; the residences in the east
  (place_residences), terraced, their gardens on their roofs.

Every coloured line of light near the ground glows and is written to story_curia.glow.json for the
game to light with, and where the roads and their crossings are to story_curia.roads.json, as
Urbs' are - but for the streets closed under the Parliament and the Tower.

Builds data/story/story_curia.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/story/story_curia.py -- --seed 31 --out ~/Documents/blender/story/story_curia.blend --glb data/story/story_curia.glb

Stairs are of 0.25 m steps, 0.35 m deep; nothing on the ground is higher than a step without
stairs. Empties named place_* mark its landmarks: place_curia_bridge, place_gate, place_mall,
place_parliament, place_tower, place_court, place_archive, place_nations_west, place_nations_east,
place_embassies and place_residences.

Made for Ruptura Systematis (MazeGame/maze).
"""

import json
import math
import os
import random
import sys

import bpy
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "arena"))
import br_town  # noqa: E402
import story_urbs as urbs  # noqa: E402
from br_grid import material, glow  # noqa: E402
from br_nexus import WORLD, Frame, polygon, octagon, RISE, RUN  # noqa: E402
from story_urbs import (rect, middle, shrink, block_middle, Z, PITCH, STREET, BLOCKS,  # noqa: E402
                        SIDEWALK, KERB, HALF, DEPTH)
from story_ferrum import cyl, dome, sphere, beam, V, bounds  # noqa: E402

# Where Curia stands in Urbs' frame: the bridge from Urbs' north edge is this long, and Curia's
# middle this far north of Urbs'.
LINK = 40.0
OY = HALF + LINK + HALF                      # 952 m
GOLD = (1.0, 0.6, 0.12)
PALE = (0.85, 0.9, 1.0)

# Where the landmarks stand, by block (i west to east, j south to north). The Parliament and the
# Tower each take blocks either side of the avenue, and the streets between them.
PARLIAMENT = (6, 7, 7, 8)                    # blocks i0..i1, j0..j1
TOWER = (6, 7, 12, 12)
MALL = [2, 3, 4, 5, 9, 10, 11]               # the rows either side of the avenue that are lawns
FORUM = [0, 1, 6, 13]                        # and those that are plazas
SPECIAL_AT = {(3, 8): "court", (10, 8): "archive", (3, 4): "nations_west", (10, 4): "nations_east"}


class Curia(urbs.Urbs):
    """Urbs' builder, its geometry moved north to where Curia stands, with the capital's
    materials."""

    def __init__(self, paint):
        super().__init__(paint)
        f = self.facades
        self.city, self.offset = "r", Vector((0, OY, 0))
        f["marble"] = material("Marble", (0.74, 0.72, 0.68), roughness=0.4, metallic=0.05)
        f["travertine"] = material("Travertine", (0.58, 0.53, 0.45), roughness=0.6, metallic=0.0)
        f["gilt"] = material("Gilt", (0.8, 0.58, 0.2), roughness=0.25, metallic=0.95)
        f["leads"] = material("Leads", (0.07, 0.08, 0.09), roughness=0.5, metallic=0.6)
        self.paving = material("Paving", (0.3, 0.29, 0.27), roughness=0.75, metallic=0.0)
        self.lines["gold"] = glow("TrimGold", GOLD, 3.0)
        self.lines["pale"] = glow("TrimPale", PALE, 3.0)
        for name, rgb in (("gold", GOLD), ("pale", PALE)):
            self.shade_of[id(self.lines[name])] = rgb
        self.gold = self.lines["gold"]
        self.flags = [material(f"Flag{k}", rgb, roughness=0.8, metallic=0.0) for k, rgb in enumerate(
            ((0.5, 0.04, 0.05), (0.04, 0.1, 0.42), (0.05, 0.3, 0.1), (0.7, 0.55, 0.1), (0.75, 0.75, 0.75),
             (0.3, 0.05, 0.35)))]

    def mesh(self, verts, faces, col, mat, cover="none"):
        return super().mesh([Vector((v[0], v[1] + OY, v[2])) for v in verts], faces, col, mat, cover)

    def shade(self):
        return self.paint.choice(("gold", "gold", "gold", "pale"))


def marker(name, x, y, z, facing):
    br_town.marker(name, x, y + OY, z, facing)


def area(i0, i1, j0, j1):
    """The ground blocks i0..i1, j0..j1 cover, kerb to kerb, with the streets between them."""
    h = PITCH / 2 - STREET / 2
    (ax, ay), (bx, by) = block_middle(i0, j0), block_middle(i1, j1)
    return ax - h, bx + h, ay - h, by + h


def within(i, j, blocks):
    i0, i1, j0, j1 = blocks
    return i0 <= i <= i1 and j0 <= j <= j1


# The orders: columns, the beams over them, pediments.

def column(m, x, y, z0, h, r, shade):
    """A column: a plinth, a shaft of marble, a capital; a gold line up its face."""
    m.box(WORLD, x - r * 1.3, x + r * 1.3, y - r * 1.3, y + r * 1.3, z0, z0 + 0.4, m.struct, m.facades["marble"])
    cyl(m, x, y, r, z0 + 0.4, z0 + h - 0.6, m.facades["marble"], r1=r * 0.88, n=8)
    m.box(WORLD, x - r * 1.35, x + r * 1.35, y - r * 1.35, y + r * 1.35, z0 + h - 0.6, z0 + h, m.struct,
          m.facades["gilt"])
    return x, y


def colonnade(m, a, b, z0, h, r, every, shade, out, lines=True):
    """Columns from `a` to `b` every `every` m or so, a beam over them; `out` the way they face, along
    which the beam's gold edges and, with `lines`, the columns' are seen."""
    n = max(2, round((b - a).length / every) + 1)
    for k in range(n):
        p = a.lerp(b, k / (n - 1))
        column(m, p.x, p.y, z0, h, r, shade)
        if lines:
            m.line(V(p.x + out.x * (r + 0.03), p.y + out.y * (r + 0.03), z0 + 0.4),
                   V(p.x + out.x * (r * 0.9 + 0.03), p.y + out.y * (r * 0.9 + 0.03), z0 + h - 0.6), m.lines[shade],
                   width=0.08, across=out, light=False)
    d = (b - a).normalized()
    ea, eb = a - d * (r * 1.6), b + d * (r * 1.6)
    side = V(-d.y, d.x, 0) * (r * 1.6)
    m.solid([ea - side, eb - side, eb + side, ea + side], z0 + h, z0 + h + 1.2, m.struct, m.facades["marble"])
    for z in (z0 + h + 0.05, z0 + h + 1.15):
        m.line(ea + out * (r * 1.6 + 0.04) + V(0, 0, z), eb + out * (r * 1.6 + 0.04) + V(0, 0, z), m.lines[shade],
               width=0.12, light=False)
    return z0 + h + 1.2


def pediment(m, a, b, z, rise, depth, shade, out):
    """A low gable from `a` to `b` at `z`, `depth` back from its face toward -`out`, edged in gold."""
    c = (a + b) / 2
    top = V(c.x, c.y, z + rise)
    back = -out * depth
    pa, pb = V(a.x, a.y, z), V(b.x, b.y, z)
    verts = [pa, pb, top, pa + back, pb + back, top + back]
    m.mesh(verts, [(0, 1, 2), (5, 4, 3), (0, 3, 4, 1), (1, 4, 5, 2), (2, 5, 3, 0)], m.struct, m.facades["marble"])
    o = out * 0.05
    for p, q in ((pa, top), (top, pb), (pa, pb)):
        m.line(p + o, q + o, m.lines[shade], width=0.15, light=False)


def flag(m, x, y, z, h, rng):
    """A flag on a mast, a gold light at the top."""
    m.box(WORLD, x - 0.08, x + 0.08, y - 0.08, y + 0.08, z, z + h, m.struct, m.rail)
    m.box(WORLD, x + 0.08, x + 2.6, y - 0.03, y + 0.03, z + h - 1.9, z + h - 0.3, m.struct, rng.choice(m.flags))
    m.line(V(x, y, z + h), V(x, y, z + h + 0.3), m.gold, width=0.2, light=False)


def obelisk(m, x, y, z, h, w, shade):
    """An obelisk on a stepped base, its pyramidion gilt, gold up its edges."""
    m.box(WORLD, x - w * 1.6, x + w * 1.6, y - w * 1.6, y + w * 1.6, z, z + 0.25, m.struct, m.facades["marble"])
    m.box(WORLD, x - w * 1.2, x + w * 1.2, y - w * 1.2, y + w * 1.2, z + 0.25, z + 1.4, m.struct, m.facades["marble"])
    base = polygon(w / math.sqrt(0.5) / 2 * 1.0, 4, math.pi / 4, x, y)
    top = polygon(w * 0.65 / math.sqrt(0.5) / 2, 4, math.pi / 4, x, y)
    m.prism(base, z + 1.4, z + h, m.facades["marble"], top=top)
    tip = polygon(0.02, 4, math.pi / 4, x, y)
    m.prism(top, z + h, z + h + w * 0.8, m.facades["gilt"], top=tip)
    for a, b in zip(base, top):
        m.line(V(a.x, a.y, z + 1.4), V(b.x, b.y, z + h), m.lines[shade], width=0.08, light=False)


def basin(m, cx, cy, r, shade):
    """A round fountain: a rim a step high, water in it, a jet of light in the middle."""
    ring = polygon(r, 24, 0, cx, cy)
    m.solid(ring, KERB, KERB + 0.25, m.struct, m.facades["marble"])
    m.solid(polygon(r - 0.6, 24, 0, cx, cy), KERB + 0.25, KERB + 0.27, m.ground, m.water)
    m.ring(ring, KERB + 0.25, m.lines[shade], out=0.03, width=0.08)
    cyl(m, cx, cy, 0.8, KERB + 0.25, KERB + 1.6, m.facades["marble"], r1=0.5, n=10)
    m.line(V(cx, cy, KERB + 1.6), V(cx, cy, KERB + 3.2), m.lines["pale"], width=0.25)


def gold_lamp(m, x, y):
    m.pylon(WORLD, x, y, KERB, height=6.0, shade="gold")


def ribs(m, cx, cy, r, z, rise, shade, n=16, steps=6):
    """Gold ribs up a dome as `dome` draws it, from its foot to its crown."""
    for k in range(n):
        t = 2 * math.pi * k / n
        pts = []
        for s in range(steps + 1):
            a = s / steps * math.pi / 2
            rr = max(r * math.cos(a), r * 0.08) + 0.06
            pts.append(V(cx + rr * math.cos(t), cy + rr * math.sin(t), z + r * rise * math.sin(a) + 0.02))
        for p, q in zip(pts, pts[1:]):
            m.line(p, q, m.lines[shade], width=0.15, light=False)


# The ministries, the embassies and the residences, each on `poly`, a rectangle within its lot.

def ministry(m, rng, poly, district, street):
    """A ministry: a block of stone a colonnade along each face on a street, its windows in warm
    bands, a stepped attic over it ringed in gold, flags on its roof."""
    x0, x1, y0, y1 = bounds(poly)
    shade = m.shade()
    mat = rng.choice((m.facades["marble"], m.facades["travertine"], m.facades["marble"]))
    h = rng.choice((14.0, 18.0, 22.0, 26.0))
    deep = 2.4 if min(x1 - x0, y1 - y0) > 16 else 0.0
    sides = {0: (V(x0, y0), V(x1, y0), V(0, -1)), 1: (V(x1, y0), V(x1, y1), V(1, 0)),
             2: (V(x1, y1), V(x0, y1), V(0, 1)), 3: (V(x0, y1), V(x0, y0), V(-1, 0))}
    body = rect(x0 + (deep if 3 in street else 0), x1 - (deep if 1 in street else 0),
                y0 + (deep if 0 in street else 0), y1 - (deep if 2 in street else 0))
    m.prism(body, 0.0, h, mat)
    m.facade_windows(body, 0.6, h - 0.8, 4.0, "warm", rng, lit=0.75, bands=True, faces=street or None)
    m.ring(body, h - 0.05, m.lines[shade])
    if deep:
        for s in street:
            a, b, out = sides[s]
            d = (b - a).normalized()
            a2, b2 = a + d * 1.0 + out * -0.9, b - d * 1.0 + out * -0.9
            colonnade(m, a2, b2, 0.0, h * 0.62, 0.45, 4.0, shade, out, lines=False)
    attic = shrink(body, 3.0)
    top = h + rng.choice((4.0, 6.0))
    m.prism(attic, h, top, mat)
    m.ring(attic, top - 0.05, m.lines[shade])
    m.fins(body, body, 0.0, h, m.lines[shade], every=1)
    c = middle(attic)
    for k in range(rng.randint(1, 3)):
        flag(m, c.x - 3.0 + 3.0 * k, c.y, top, 6.0, rng)


def pavilion(m, rng, poly, district, street):
    """An embassy: a pavilion of two floors under a dome or a gilt roof, in its garden, its flag
    before it."""
    x0, x1, y0, y1 = bounds(poly)
    shade = m.shade()
    inner = shrink(poly, 3.5)
    ix0, ix1, iy0, iy1 = bounds(inner)
    h = rng.choice((8.0, 10.0, 12.0))
    m.prism(inner, 0.0, h, rng.choice((m.facades["marble"], m.facades["travertine"], m.facades["white"])))
    m.facade_windows(inner, 0.6, h - 0.6, 4.0, "warm", rng, lit=0.8, bands=True)
    m.ring(inner, h - 0.05, m.lines[shade])
    m.fins(inner, inner, 0.0, h, m.lines[shade], every=1)
    c = middle(inner)
    side = min(ix1 - ix0, iy1 - iy0)
    if rng.random() < 0.55 and side > 8:
        r = side * 0.32
        cyl(m, c.x, c.y, r, h, h + 2.0, m.facades["marble"], n=16)
        dome(m, c.x, c.y, r, h + 2.0, m.facades["leads"], rise=0.8, n=16, steps=4)
        ribs(m, c.x, c.y, r, h + 2.0, 0.8, shade, n=8, steps=4)
    else:
        m.prism(inner, h, h + 2.5, m.facades["gilt"], top=shrink(inner, min(ix1 - ix0, iy1 - iy0) * 0.35))
    for x, y in ((x0 + 1.2, y0 + 1.2), (x1 - 1.2, y0 + 1.2), (x0 + 1.2, y1 - 1.2), (x1 - 1.2, y1 - 1.2)):
        if rng.random() < 0.7:
            m.tree(x, y, rng, size=0.7)
    s = street[0] if street else 0
    fx, fy = {0: (c.x, y0 + 1.0), 1: (x1 - 1.0, c.y), 2: (c.x, y1 - 1.0), 3: (x0 + 1.0, c.y)}[s]
    flag(m, fx, fy, KERB, 8.0, rng)


def residence(m, rng, poly, district, street):
    """Residences: terraced back as they rise, a line of balconies' light round each floor, gardens
    on the terraces."""
    shade = m.shade()
    p, z = poly, 0.0
    for t in range(rng.randint(2, 4)):
        top = z + rng.choice((9.0, 12.0, 15.0))
        m.prism(p, z, top, rng.choice((m.facades["travertine"], m.facades["marble"], m.facades["white"])))
        m.facade_windows(p, z + 0.5, top - 0.5, 3.0, "warm", rng, lit=0.85, bands=True)
        m.ring(p, top - 0.05, m.lines[shade])
        inner = shrink(p, 3.0)
        if min((q - middle(inner)).length for q in inner) < 5.0:
            break
        for a in p:
            b = a + (middle(p) - a).normalized() * 1.6
            if rng.random() < 0.6:
                m.tree(b.x, b.y, rng, z=top, size=0.6)
        z, p = top, inner
    m.fins(poly, poly, 0.0, 9.0, m.lines[shade], every=1)


# The landmarks.

def gate(m):
    """The Gate of Nations: a triumphal arch over the avenue at the edge, its vault lit in gold, a
    gilt globe on it."""
    y0, y1 = -HALF + 2.0, -HALF + 9.0
    w, pier, spring, attic, top = 9.5, 6.0, 15.0, 22.0, 29.0
    for s in (-1, 1):
        xa, xb = sorted((s * w, s * (w + pier)))
        m.box(WORLD, xa, xb, y0, y1, 0.0, attic, m.struct, m.facades["marble"])
        for x in (xa, xb):
            for y in (y0 - 0.05, y1 + 0.05):
                m.line(V(x, y, 0.0), V(x, y, attic), m.gold, width=0.2, light=True)
    m.box(WORLD, -(w + pier), w + pier, y0, y1, attic, top, m.struct, m.facades["marble"])
    m.box(WORLD, -(w + pier) - 0.4, w + pier + 0.4, y0 - 0.4, y1 + 0.4, top, top + 0.8, m.struct,
          m.facades["gilt"])
    for y in (y0 - 0.06, y1 + 0.06):
        for z in (attic + 0.1, top - 0.1, (attic + top) / 2):
            m.line(V(-(w + pier), y, z), V(w + pier, y, z), m.gold, width=0.18 if z != (attic + top) / 2 else 0.5,
                   light=False)
    # The vault: from the springing on either pier up to its crown, filled to the attic.
    n = 12
    soffit = [V(-w * math.cos(math.pi * k / n), 0.0, spring + (attic - 1.5 - spring) * math.sin(math.pi * k / n))
              for k in range(n + 1)]
    for a, b in zip(soffit, soffit[1:]):
        verts = [V(a.x, y0, a.z), V(b.x, y0, b.z), V(b.x, y0, attic), V(a.x, y0, attic),
                 V(a.x, y1, a.z), V(b.x, y1, b.z), V(b.x, y1, attic), V(a.x, y1, attic)]
        m.mesh(verts, [(0, 1, 2, 3), (7, 6, 5, 4), (4, 5, 1, 0)], m.struct, m.facades["marble"])
        for y in (y0 - 0.06, y1 + 0.06):
            m.line(V(a.x, y, a.z), V(b.x, y, b.z), m.gold, width=0.25, light=False)
    sphere(m, 0.0, (y0 + y1) / 2, top + 4.2, 3.2, m.facades["gilt"], n=16, bands=8)
    m.ring(polygon(3.3, 16, 0, 0.0, (y0 + y1) / 2), top + 4.2, m.gold, out=0.0, width=0.2, light=False)
    marker("place_gate", 0.0, y1 + 6.0, KERB, math.pi / 2)


def mall(m, rng, cx, cy, inner, j):
    """A lawn by the avenue: a reflecting pool down it, an obelisk at its end, trees in rows, gold
    lamps along the avenue."""
    x0, x1, y0, y1 = inner
    m.solid(rect(x0, x1, y0, y1), KERB, KERB + 0.01, m.ground, m.grass)
    toward = -1 if cx > 0 else 1                     # toward the avenue
    px = cx + toward * 4.0
    pool = rect(px - 6.0, px + 6.0, y0 + 6.0, y1 - 6.0)
    m.solid(pool, KERB, KERB + 0.25, m.struct, m.facades["marble"])
    m.solid(shrink(pool, 0.6), KERB + 0.25, KERB + 0.26, m.ground, m.water)
    m.ring(pool, KERB + 0.25, m.gold, out=0.03, width=0.08)
    for y in (y0 + 3.0, y1 - 3.0):
        for x in (px - 9.5, px + 9.5):
            m.tree(x, y, rng, size=0.9)
    for k in range(5):
        y = y0 + 6.0 + (y1 - y0 - 12.0) * k / 4
        m.tree(cx - toward * 15.0, y, rng, size=0.9)
    if j % 2:
        obelisk(m, px, (y0 + y1) / 2, KERB, 22.0, 2.0, "gold")
    edge = x1 - 0.6 if toward > 0 else x0 + 0.6
    for k in range(4):
        gold_lamp(m, edge, y0 + 4.0 + (y1 - y0 - 8.0) * k / 3)


def forum(m, rng, cx, cy, inner):
    """A plaza by the avenue: paved, a fountain in it, gold lamps round it."""
    x0, x1, y0, y1 = inner
    m.solid(rect(x0, x1, y0, y1), KERB, KERB + 0.01, m.ground, m.paving)
    basin(m, cx, cy, 8.0, "gold")
    for k in range(8):
        t = 2 * math.pi * k / 8 + math.pi / 8
        gold_lamp(m, cx + 15.0 * math.cos(t), cy + 15.0 * math.sin(t))
    for x, y in ((x0 + 3, y0 + 3), (x1 - 3, y0 + 3), (x0 + 3, y1 - 3), (x1 - 3, y1 - 3)):
        m.tree(x, y, rng, size=0.9)


def stairs_north(m, x0, x1, y, bottom, top, way):
    """Stairs from `bottom` up to `top` across x0..x1, rising from `y` along y the way `way`."""
    f = Frame(0.0, 0.0, math.pi / 2)                 # local x along the world's y, local y along -x
    return m.flight(f, y, -x1, -x0, bottom, top, way, mat="gold")


def parliament(m, rng):
    """The Parliament, across the avenue: a plaza with fountains, a podium climbed from it, on the
    podium a rotunda behind its ring of columns under a dome ribbed in gold, a portico before it,
    a wing either side."""
    x0, x1, y0, y1 = area(*PARLIAMENT)
    m.solid(rect(x0, x1, y0, y1), 0.0, KERB, m.ground, m.paving)
    m.ring(rect(x0, x1, y0, y1), KERB - 0.02, m.gold, out=0.03, width=0.06, light=False)
    pod = KERB + 4 * RISE                             # four steps up from the plaza
    px0, px1, py0, py1 = x0 + 6.0, x1 - 6.0, y0 + 30.0, y1 - 8.0
    m.solid(rect(px0, px1, py0, py1), KERB, pod, m.struct, m.facades["travertine"])
    m.ring(rect(px0, px1, py0, py1), pod - 0.03, m.gold, out=0.03, width=0.08)
    n = 4
    stairs_north(m, -30.0, 30.0, py0 - n * RUN, KERB, KERB + n * RISE, 1)
    stairs_north(m, -12.0, 12.0, py1 + n * RUN, KERB, KERB + n * RISE, -1)
    z = KERB + n * RISE
    for fx in (-32.0, 32.0):
        basin(m, fx, y0 + 14.0, 7.0, "gold")
    for x in (-44.0, -20.0, 20.0, 44.0):
        gold_lamp(m, x, y0 + 4.0)
    # The rotunda, its ring of columns, its dome.
    cx, cy = 0.0, (py0 + py1) / 2 + 6.0
    r, drum = 19.0, 22.0
    cyl(m, cx, cy, r, z, z + drum, m.facades["marble"], n=32)
    m.ring(polygon(r, 32, 0, cx, cy), z + drum * 0.55, m.windows["warm"], out=0.05, width=1.4, light=False)
    for k in range(28):
        t = 2 * math.pi * k / 28
        column(m, cx + (r + 4.0) * math.cos(t), cy + (r + 4.0) * math.sin(t), z, drum - 4.0, 0.7, "gold")
        m.line(V(cx + (r + 4.75) * math.cos(t), cy + (r + 4.75) * math.sin(t), z + 0.4),
               V(cx + (r + 4.7) * math.cos(t), cy + (r + 4.7) * math.sin(t), z + drum - 4.6), m.gold, width=0.1,
               light=False)
    m.prism(polygon(r + 5.2, 32, 0, cx, cy), z + drum - 4.0, z + drum - 2.6, m.facades["marble"],
            top=polygon(r + 5.2, 32, 0, cx, cy))
    m.ring(polygon(r + 5.2, 32, 0, cx, cy), z + drum - 2.65, m.gold, out=0.04, width=0.2, light=False)
    m.ring(polygon(r + 0.1, 32, 0, cx, cy), z + drum - 0.1, m.gold, out=0.04, width=0.3, light=False)
    dome(m, cx, cy, r, z + drum, m.facades["marble"], rise=0.8, n=32, steps=8)
    ribs(m, cx, cy, r, z + drum, 0.8, "gold", n=16, steps=8)
    crown = z + drum + r * 0.8
    cyl(m, cx, cy, 2.8, crown - 0.4, crown + 5.0, m.facades["marble"], n=12)
    for k in range(6):
        t = 2 * math.pi * k / 6
        m.line(V(cx + 2.9 * math.cos(t), cy + 2.9 * math.sin(t), crown), V(cx + 2.9 * math.cos(t),
               cy + 2.9 * math.sin(t), crown + 5.0), m.gold, width=0.15, light=False)
    cyl(m, cx, cy, 3.2, crown + 5.0, crown + 6.0, m.facades["gilt"], r1=0.3, n=12)
    cyl(m, cx, cy, 0.3, crown + 6.0, crown + 14.0, m.facades["gilt"], r1=0.05, n=6)
    # A halo of gold over the dome's foot.
    m.ring(polygon(r + 2.5, 48, 0, cx, cy), z + drum + 4.0, m.gold, out=0.0, width=0.35, light=False)
    # The portico before it, facing the plaza.
    front = cy - r - 12.0
    m.box(WORLD, -14.0, 14.0, front, cy - r + 2.0, z, z + drum - 4.0, m.struct, m.facades["marble"])
    top = colonnade(m, V(-13.0, front - 1.5), V(13.0, front - 1.5), z, drum - 4.0, 0.8, 3.7, "gold", V(0, -1))
    pediment(m, V(-14.4, front - 2.5), V(14.4, front - 2.5), top, 6.0, 12.0, "gold", V(0, -1))
    m.line(V(-11.0, front - 0.06, z + 5.0), V(11.0, front - 0.06, z + 5.0), m.windows["warm"], width=3.0, height=0.05,
           across=Z, light=False)
    # The wings, east and west, colonnaded toward the plaza.
    for s in (-1, 1):
        wx0, wx1 = sorted((s * 26.0, s * (px1 - 2.0)))
        wy0, wy1 = py0 + 6.0, py1 - 10.0
        m.prism(rect(wx0, wx1, wy0, wy1), z, z + 16.0, m.facades["marble"])
        m.facade_windows(rect(wx0, wx1, wy0, wy1), z + 0.6, z + 15.4, 4.0, "warm", rng, lit=0.85, bands=True)
        m.ring(rect(wx0, wx1, wy0, wy1), z + 15.95, m.gold)
        m.prism(rect(wx0 + 2.0, wx1 - 2.0, wy0 + 2.0, wy1 - 2.0), z + 16.0, z + 19.0, m.facades["leads"])
        m.ring(rect(wx0 + 2.0, wx1 - 2.0, wy0 + 2.0, wy1 - 2.0), z + 18.95, m.gold, light=False)
        m.fins(rect(wx0, wx1, wy0, wy1), rect(wx0, wx1, wy0, wy1), z, z + 16.0, m.gold)
        t = colonnade(m, V(wx0 + 1.5, wy0 - 2.0), V(wx1 - 1.5, wy0 - 2.0), z, 12.0, 0.55, 3.6, "gold", V(0, -1))
        mid = (wx0 + wx1) / 2
        pediment(m, V(mid - 7.0, wy0 - 2.8), V(mid + 7.0, wy0 - 2.8), t, 3.5, 2.5, "gold", V(0, -1))
        flag(m, mid, (wy0 + wy1) / 2, z + 19.0, 9.0, rng)
    marker("place_parliament", 0.0, y0 + 24.0, KERB, math.pi / 2)


def tower(m, rng):
    """The Tower of the Republic at the head of the avenue: octagonal, white, stepping in as it rises,
    gold up its corners; a lantern of light at its top and a gilt needle over it; a garden round its
    foot."""
    x0, x1, y0, y1 = area(*TOWER)
    m.solid(rect(x0, x1, y0, y1), 0.0, KERB, m.ground, m.paving)
    m.ring(rect(x0, x1, y0, y1), KERB - 0.02, m.gold, out=0.03, width=0.06, light=False)
    cx, cy = 0.0, (y0 + y1) / 2
    z = KERB
    for r, h in ((17.0, 64.0), (13.5, 52.0), (10.5, 40.0), (8.0, 24.0)):
        base = polygon(r, 8, math.pi / 8, cx, cy)
        top = polygon(r * 0.96, 8, math.pi / 8, cx, cy)
        m.prism(base, z, z + h, m.facades["marble"], top=top)
        m.facade_windows(base, z + 1.0, z + h - 1.0, 4.0, "office", rng, lit=0.7, bands=True)
        for a, b in zip(base, top):
            m.line(V(a.x, a.y, z), V(b.x, b.y, z + h), m.gold, width=0.25)
        m.ring(top, z + h - 0.05, m.gold, width=0.35)
        m.solid(polygon(r * 0.99, 8, math.pi / 8, cx, cy), z + h, z + h + 1.0, m.struct, m.facades["gilt"])
        z += h + 1.0
    cyl(m, cx, cy, 6.0, z, z + 10.0, m.facades["glass"], n=16)
    m.ring(polygon(6.05, 16, 0, cx, cy), z + 5.0, m.lines["pale"], out=0.0, width=2.0, light=False)
    cyl(m, cx, cy, 7.0, z + 10.0, z + 11.0, m.facades["gilt"], n=16)
    cyl(m, cx, cy, 6.0, z + 11.0, z + 40.0, m.facades["gilt"], r1=0.2, n=16)
    m.mast(cx, cy, z + 40.0, 6.0, "red")
    # The garden round its foot, and its doors toward the avenue.
    for x in (-50.0, -42.0, -34.0, -26.0, 26.0, 34.0, 42.0, 50.0):
        for y in (cy - 16.0, cy + 16.0):
            m.tree(x, y, rng, size=0.9)
    m.ring(octagon(22.0, cx, cy), KERB + 0.01, m.gold, out=0.0, width=0.12, light=False)
    m.line(V(-5.0, cy - 16.4, KERB + 4.0), V(5.0, cy - 16.4, KERB + 4.0), m.windows["warm"], width=8.0, height=0.05,
           across=Z, light=False)
    for x in (-40.0, -28.0, 28.0, 40.0):
        gold_lamp(m, x, y0 + 4.0)
    marker("place_tower", 0.0, y0 - 6.0, 0.0, math.pi / 2)


def court(m, rng, cx, cy, inner):
    """The Supreme Court: a temple on a base of three steps the height of a man, stairs up its front,
    columns all round it, a pediment at either end."""
    x0, x1, y0, y1 = inner
    m.solid(rect(x0, x1, y0, y1), KERB, KERB + 0.01, m.ground, m.paving)
    bx0, bx1, by0, by1 = x0 + 4.0, x1 - 4.0, y0 + 9.0, y1 - 3.0
    base = 6 * RISE
    m.solid(rect(bx0, bx1, by0, by1), KERB, KERB + base, m.struct, m.facades["marble"])
    m.ring(rect(bx0, bx1, by0, by1), KERB + base - 0.03, m.gold, out=0.03, width=0.08)
    n = round(base / RISE)
    stairs_north(m, bx0 + 4.0, bx1 - 4.0, by0 - n * RUN, KERB, KERB + n * RISE, 1)
    z = KERB + n * RISE
    cell = rect(bx0 + 5.0, bx1 - 5.0, by0 + 5.0, by1 - 5.0)
    m.prism(cell, z, z + 14.0, m.facades["marble"])
    m.line(V(cx - 3.0, by0 + 4.95, z + 0.1), V(cx - 3.0, by0 + 4.95, z + 7.0), m.windows["warm"], width=0.3,
           light=False)
    tops = []
    for a, b, out in ((V(bx0 + 1.5, by0 + 1.5), V(bx1 - 1.5, by0 + 1.5), V(0, -1)),
                      (V(bx0 + 1.5, by1 - 1.5), V(bx1 - 1.5, by1 - 1.5), V(0, 1)),
                      (V(bx0 + 1.5, by0 + 5.0), V(bx0 + 1.5, by1 - 5.0), V(-1, 0)),
                      (V(bx1 - 1.5, by0 + 5.0), V(bx1 - 1.5, by1 - 5.0), V(1, 0))):
        tops.append(colonnade(m, a, b, z, 13.0, 0.6, 3.6, "gold", out))
    t = tops[0]
    m.prism(rect(bx0 + 0.5, bx1 - 0.5, by0 + 0.5, by1 - 0.5), t, t + 0.8, m.facades["marble"])
    roof = rect(bx0 + 0.5, bx1 - 0.5, by0 + 0.5, by1 - 0.5)
    # The roof: a ridge from end to end, pediments under its ends.
    rz, rise = t + 0.8, 5.0
    xa, xb = bx0 + 0.5, bx1 - 0.5
    ya, yb = by0 + 0.5, by1 - 0.5
    verts = [V(xa, ya, rz), V(xb, ya, rz), V(cx, ya, rz + rise), V(xa, yb, rz), V(xb, yb, rz), V(cx, yb, rz + rise)]
    m.mesh(verts, [(0, 1, 2), (5, 4, 3), (0, 3, 4, 1), (1, 4, 5, 2), (2, 5, 3, 0)], m.struct, m.facades["leads"])
    for y, o in ((ya - 0.05, -1), (yb + 0.05, 1)):
        for p, q in ((V(xa, y, rz), V(cx, y, rz + rise)), (V(cx, y, rz + rise), V(xb, y, rz)), (V(xa, y, rz), V(xb, y, rz))):
            m.line(p, q, m.gold, width=0.2, light=False)
    m.line(V(cx, ya, rz + rise + 0.05), V(cx, yb, rz + rise + 0.05), m.gold, width=0.2, light=False)
    for x in (x0 + 2.0, x1 - 2.0):
        gold_lamp(m, x, y0 + 3.0)
    marker("place_court", cx, y0 + 3.0, KERB, math.pi / 2)


def archive(m, rng, cx, cy, inner):
    """The State Archive: a rotunda of stone behind a ring of columns, a shallow dome ribbed in gold,
    its windows a band of warm light."""
    x0, x1, y0, y1 = inner
    m.solid(rect(x0, x1, y0, y1), KERB, KERB + 0.01, m.ground, m.paving)
    r, h = 13.0, 14.0
    cyl(m, cx, cy, r, KERB, KERB + h, m.facades["travertine"], n=24)
    m.ring(polygon(r, 24, 0, cx, cy), KERB + h * 0.5, m.windows["warm"], out=0.05, width=1.0, light=False)
    for k in range(20):
        t = 2 * math.pi * k / 20
        column(m, cx + (r + 3.5) * math.cos(t), cy + (r + 3.5) * math.sin(t), KERB, h - 2.0, 0.55, "gold")
        m.line(V(cx + (r + 4.1) * math.cos(t), cy + (r + 4.1) * math.sin(t), KERB + 0.4),
               V(cx + (r + 4.05) * math.cos(t), cy + (r + 4.05) * math.sin(t), KERB + h - 2.6), m.gold, width=0.1)
    m.prism(polygon(r + 4.5, 24, 0, cx, cy), KERB + h - 2.0, KERB + h - 1.0, m.facades["marble"])
    m.ring(polygon(r + 4.5, 24, 0, cx, cy), KERB + h - 1.05, m.gold, out=0.04, width=0.15, light=False)
    dome(m, cx, cy, r, KERB + h, m.facades["leads"], rise=0.55, n=24, steps=6)
    ribs(m, cx, cy, r, KERB + h, 0.55, "gold", n=12, steps=6)
    for x, y in ((x0 + 3, y0 + 3), (x1 - 3, y0 + 3), (x0 + 3, y1 - 3), (x1 - 3, y1 - 3)):
        m.tree(x, y, rng, size=0.9)
    marker("place_archive", cx, y0 + 3.0, KERB, math.pi / 2)


def nations(m, rng, cx, cy, inner, name):
    """A Plaza of Nations: a ring of flags round a gold-edged obelisk, gold lamps round them."""
    x0, x1, y0, y1 = inner
    m.solid(rect(x0, x1, y0, y1), KERB, KERB + 0.01, m.ground, m.paving)
    m.ring(polygon(17.0, 32, 0, cx, cy), KERB + 0.012, m.gold, out=0.0, width=0.15, light=False)
    for k in range(24):
        t = 2 * math.pi * k / 24
        flag(m, cx + 15.0 * math.cos(t), cy + 15.0 * math.sin(t), KERB, 9.0, rng)
    obelisk(m, cx, cy, KERB, 26.0, 2.2, "gold")
    for x, y in ((x0 + 2.0, y0 + 2.0), (x1 - 2.0, y0 + 2.0), (x0 + 2.0, y1 - 2.0), (x1 - 2.0, y1 - 2.0)):
        gold_lamp(m, x, y)
    marker(name, cx, y0 + 3.0, KERB, math.pi / 2)


# The districts' lots, and what goes up on them.

def district_of(i, j):
    if i <= 2:
        return "embassies"
    if i >= 11:
        return "residences"
    return "ministries"


OPTIONS = {
    "ministries": [ministry],
    "embassies": [pavilion, pavilion, pavilion, ministry],
    "residences": [residence, residence, residence, ministry],
}

OFFICE_CHANCE = 0.25     # the chance a lot for a ministry is an office tower to go up instead
MOST_OFFICES = 8         # and the most of them


def build_lot(m, rng, district, lot, block):
    x0, x1, y0, y1 = lot
    street = urbs.faces_on_street(lot, block)
    rng.shuffle(street)
    inset = rng.uniform(0.8, 1.8)
    poly = rect(x0 + inset, x1 - inset, y0 + inset, y1 - inset)
    small = min(x1 - x0, y1 - y0)
    build = rng.choice(OPTIONS[district])
    if small < 12 and build is pavilion:
        build = residence if district == "residences" else ministry
    if (build is ministry and street and small >= urbs.OFFICE_SIZE[0]
            and m.n_lifts < MOST_OFFICES and m.lifts.random() < OFFICE_CHANCE):
        # An office tower to go up in its place (see urbs.office_tower): the one it stands for built
        # out of sight, for its draws of chance, so that the rest of the city is as it was.
        m.unseen(lambda: build(m, rng, poly, district, street))
        urbs.office_tower(m, m.lifts, poly, street, m.facades["marble"], urbs.unpainted_shade(m))
        m.n_buildings += 1
        return
    build(m, rng, poly, district, street)
    m.n_buildings += 1


def block(m, rng, i, j):
    """A block: its pavement, its lamps, and what stands on it."""
    if within(i, j, PARLIAMENT) or within(i, j, TOWER):
        return
    cx, cy = block_middle(i, j)
    h = PITCH / 2 - STREET / 2
    x0, x1, y0, y1 = cx - h, cx + h, cy - h, cy + h
    district = district_of(i, j)
    m.solid(rect(x0, x1, y0, y1), 0.0, KERB, m.ground, m.paving)
    m.ring(rect(x0, x1, y0, y1), KERB - 0.02, m.gold, out=0.03, width=0.06, light=False)
    for sx, sy in ((-1, -1), (1, 1)) if (i + j) % 2 else ((1, -1), (-1, 1)):
        m.pylon(WORLD, cx + sx * (h - 1.5), cy + sy * (h - 1.5), KERB, height=7.0, shade="gold")
    inner = (x0 + SIDEWALK, x1 - SIDEWALK, y0 + SIDEWALK, y1 - SIDEWALK)
    if i in (6, 7) and j in MALL:
        mall(m, rng, cx, cy, inner, j)
        return
    if i in (6, 7) and j in FORUM:
        forum(m, rng, cx, cy, inner)
        return
    special = SPECIAL_AT.get((i, j))
    if special == "court":
        return court(m, rng, cx, cy, inner)
    if special == "archive":
        return archive(m, rng, cx, cy, inner)
    if special in ("nations_west", "nations_east"):
        return nations(m, rng, cx, cy, inner, "place_" + special)
    if district == "embassies":
        for k in range(1, 4):
            t = -h + k * (2 * h) / 4
            for x, y in ((cx + t, y0 + 1.5), (cx + t, y1 - 1.5)):
                if rng.random() < 0.6:
                    m.tree(x, y, rng, size=0.8)
    kind = {"ministries": "downtown", "embassies": "midtown", "residences": "midtown"}[district]
    for lot in urbs.lots(rng, kind, *inner):
        build_lot(m, rng, district, lot, inner)


# The ground: the platform, the edge, and the bridge from Urbs.

def ground(m):
    slab = rect(-HALF, HALF, -HALF, HALF)
    m.solid(slab, -DEPTH, -0.002, m.ground, m.road)
    m.ring(slab, -0.3, m.gold, out=0.0, light=False)
    m.ring(slab, -DEPTH + 0.15, m.gold, out=0.0, light=False)
    # The edge: a parapet all round, but for the bridge from Urbs in the south.
    corners = rect(-HALF, HALF, -HALF, HALF)
    for i in range(4):
        a, b = corners[i], corners[(i + 1) % 4]
        inward = (middle(corners) - (a + b) / 2).normalized() * 0.3
        gaps = []
        if i == 0:
            g = STREET / 2 / (b - a).length
            gaps.append((0.5 - g, 0.5 + g))
        m.parapet(a + inward, b + inward, 0.0, m.gold, gaps)


def link(m):
    """The road over the void from Urbs' north edge to Curia's middle street."""
    y0, y1 = -HALF - LINK, -HALF + 0.5
    w = STREET / 2
    m.box(WORLD, -w, w, y0, y1, -1.2, -0.002, m.ground, m.road)
    for s in (-1, 1):
        m.line(V(s * w, y0, -0.6), V(s * w, y1, -0.6), m.gold, light=False)
        m.parapet(V(s * (w - 0.2), y0, 0), V(s * (w - 0.2), y1 - 0.5, 0), 0.0, m.gold)
    y = y0 + 2.0
    while y < y1 - 2.0:
        m.line(V(0.0, y, 0.004), V(0.0, y + 3.0, 0.004), m.marking, width=0.15, height=0.008, light=False)
        y += 6.0
    for y in (y0 + 4.0, (y0 + y1) / 2):
        for s in (-1, 1):
            m.pylon(WORLD, s * (w - 1.0), y, 0.0, height=7.0, shade="gold")
    marker("place_curia_bridge", 0.0, -HALF + 6.0, 0.0, math.pi / 2)


def cut(strip, holes):
    """The rectangle `strip` (x0, x1, y0, y1), long one way, with whatever of it is in `holes` taken
    out along its length."""
    x0, x1, y0, y1 = strip
    along_y = (y1 - y0) > (x1 - x0)
    parts = [(y0, y1) if along_y else (x0, x1)]
    for hx0, hx1, hy0, hy1 in holes:
        if along_y and (x1 <= hx0 or x0 >= hx1) or not along_y and (y1 <= hy0 or y0 >= hy1):
            continue
        a, b = (hy0, hy1) if along_y else (hx0, hx1)
        parts = [q for lo, hi in parts for q in ((lo, min(hi, a)), (max(lo, b), hi)) if q[1] - q[0] > 0.5]
    return [(x0, x1, lo, hi) if along_y else (lo, hi, y0, y1) for lo, hi in parts]


def road_plan():
    """Curia's roads and crossings, as Urbs' are, but for the canal, which Curia has none of, and
    the streets closed under the Parliament and the Tower."""
    lines_at = [k * PITCH for k in range(-BLOCKS // 2 + 1, BLOCKS // 2)]
    ring = HALF - STREET / 2
    every = lines_at + [-ring, ring]
    holes = [area(*PARLIAMENT), area(*TOWER)]
    game = lambda x0, x1, y0, y1: [round(x0, 3), round(-(y1 + OY), 3), round(x1, 3), round(-(y0 + OY), 3)]
    inside = lambda x0, x1, y0, y1: any(x0 < hx1 and x1 > hx0 and y0 < hy1 and y1 > hy0 for hx0, hx1, hy0, hy1 in holes)
    roads, crossings = [], []
    for c in every:
        for strip in ((c - STREET / 2, c + STREET / 2, -HALF, HALF), (-HALF, HALF, c - STREET / 2, c + STREET / 2)):
            roads += [game(*part) for part in cut(strip, holes)]
    for jx in every:
        for jy in every:
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                ox, oy = jx + dx * (STREET / 2 + 1.5), jy + dy * (STREET / 2 + 1.5)
                if dx:
                    r = (ox - 1.5, ox + 1.5, oy - STREET / 2, oy + STREET / 2)
                else:
                    r = (ox - STREET / 2, ox + STREET / 2, oy - 1.5, oy + 1.5)
                if not inside(*r):
                    crossings.append(game(*r))
    return {"level": urbs.road_plan()["level"], "roads": roads, "crossings": crossings}


def places():
    for name, (i, j) in (("place_mall", (6, 3)), ("place_embassies", (1, 6)), ("place_residences", (12, 6))):
        cx, cy = block_middle(i, j)
        marker(name, cx - PITCH / 2 + 4.0, cy, KERB, 0.0)


def streets(m, rng):
    """Urbs' street markings and crossings, but for its canal: Curia has none."""
    saved = urbs.CANAL
    urbs.CANAL = (1e6, 10.0, 1.0)
    try:
        urbs.streets(m, rng)
    finally:
        urbs.CANAL = saved


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Curia(random.Random(seed + 1))
    ground(m)
    link(m)
    streets(m, rng)
    gate(m)
    for i in range(BLOCKS):
        for j in range(BLOCKS):
            block(m, rng, i, j)
    parliament(m, rng)
    tower(m, rng)
    urbs.railway(m, -(HALF + 20.0), -328.0, stations=((-372.0, -332.0, 1, "place_station_curia"),), canal=None,
                 ends=(-328.0,))
    places()
    m.build_buckets()
    for piece in m.pieces:
        piece["corner"][2] = round(piece["corner"][2] - OY, 3)

    bpy.ops.object.camera_add(location=(0.0, OY - HALF * 2.0, HALF * 1.2), rotation=(math.radians(58), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.0, 0.002, 0.006, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else os.path.join(HERE, "story_curia.blend")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} Curia, {HALF * 2:.0f} m across at y={OY:.0f}, buildings={m.n_buildings} lifts={m.n_lifts} "
          f"lights={m.n_lights} pieces of light={len(m.pieces)} -> {out}")
    if glb:
        base = os.path.splitext(os.path.abspath(glb))[0]
        with open(base + ".glow.json", "w") as f:
            json.dump({"intensity": 3.0, "pieces": m.pieces}, f, separators=(",", ":"))
        with open(base + ".roads.json", "w") as f:
            json.dump(road_plan(), f, separators=(",", ":"))
        bpy.ops.export_scene.gltf(filepath=os.path.abspath(glb), export_format="GLB", export_extras=True,
                                  export_cameras=False, export_lights=False)
        print(f"[map] exported -> {glb}")


if __name__ == "__main__":
    main()
