"""
Subura: the undercity west of Urbs, across the void - a model of its own, apart from Urbs, the two
joined by a road. It is laid out as Urbs is, fourteen blocks by fourteen, each 48 m square between
streets 16 m wide, on a platform 912 m across, Urbs' canal running on through it. A bridge 40 m long
goes from the pad off Urbs' west gate to Subura's middle street, through the Breach, a gateway of
scrap; and Urbs' west line, a railway of its own along Urbs' middle street west, comes on over it to
Subura's station.

It is built where it stands in Urbs' frame, its middle 998 m west of Urbs', so that the two models
line up as they are, without moving either.

Its buildings are its own, a slum's grown upon itself, dark and grimy and lit in neon - pink,
acid green, violet and amber:

- The Breach (place_subura_bridge) where the bridge comes in: two towers of welded plate, a girder
  across them with the city's name lit on it in bars.
- The Hive (place_hive) in the middle, over the streets between its blocks: four towers of stacked
  dwellings at its corners, skybridges between them, crowned in neon, round a court open to the
  streets, where the stalls and the braziers are.
- The Night Market (place_market): rows of stalls under tarps, lanterns strung over them.
- The Stronghold (place_stronghold): the Praedatores' compound, walled in plate, a watchtower either
  side of its gate, braziers in its yard, a bunker at its back - where the gang keeps (hideout_<k>).
- The Pit (place_pit): a cage to fight in, ringed by stepped stands.
- The Sump (place_sump), south of the canal: shacks on stilts and heaps of scrap.
- The Strip (place_strip), by the station: low buildings under great neon signs.
- The station (place_station_subura) on Urbs' west line, by the Breach, where the west train ends.
- All round them, the stacks: shanties crowded on their lots with lanes between, towers of stacked
  containers, tenement slabs hung with balconies and boxes, workshops, and some office towers,
  their lifts working; and over the streets cables strung across, hung with lanterns.

Every coloured line of light near the ground glows and is written to story_subura.glow.json for the
game to light with, and where the roads and their crossings are to story_subura.roads.json, as
Urbs' are - but for the streets closed under the Hive.

Builds data/story/story_subura.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/story/story_subura.py -- --seed 37 --out ~/Documents/blender/story/story_subura.blend --glb data/story/story_subura.glb

Stairs are of 0.25 m steps, 0.35 m deep; nothing on the ground is higher than a step without
stairs. Empties named place_* mark its landmarks: place_subura_bridge, place_hive, place_market,
place_stronghold, place_pit, place_sump, place_strip and place_scrapyard.

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
from br_nexus import WORLD, Frame, polygon, RISE, RUN  # noqa: E402
from story_urbs import (rect, middle, faces_on_street, block_middle, Z, PITCH, STREET, BLOCKS,  # noqa: E402
                        SIDEWALK, KERB, HALF, DEPTH, CANAL, BRIDGE_WIDTH, GATE)
from story_ferrum import cyl, beam, V, bounds  # noqa: E402
from story_curia import cut  # noqa: E402

# Where Subura stands in Urbs' frame: Urbs' west gate pad ends this far west of Urbs' middle, and
# the bridge from it this long.
URBS_PAD = HALF + GATE[0] + GATE[2]          # 502 m
LINK = 40.0
OX = -(URBS_PAD + LINK + HALF)               # Subura's middle: 998 m west of Urbs'
ACID = (0.45, 1.0, 0.08)
AMBER = (1.0, 0.5, 0.08)
NEON = ("pink", "pink", "acid", "violet", "amber", "cyan")

# The west line, in a frame whose y runs east along Subura's middle street: from a buffer past its
# station, on over the bridge to the middle of Urbs' west pad, where Urbs' takes it on.
LINE = Frame(0.0, 0.0, -math.pi / 2)
LINE_BUFFER = 328.0
LINE_STATION = (332.0, 372.0, -1, "place_station_subura")
LINE_END = -OX - urbs.WEST_PAD                # 508 m: the pad's middle, in Subura's terms

# Where the landmarks stand, by block (i west to east, j south to north). The Hive takes the four
# blocks in the middle, and the streets between them.
HIVE = (6, 7, 6, 7)                          # blocks i0..i1, j0..j1
SPECIAL_AT = {(9, 9): "market", (2, 10): "stronghold", (10, 4): "pit"}
STRIP = 11                                   # the blocks from here east, by the station, are the Strip


class Subura(urbs.Urbs):
    """Urbs' builder, its geometry moved west to where Subura stands, with the slum's materials."""

    def __init__(self, paint):
        super().__init__(paint)
        f = self.facades
        self.city, self.offset = "s", Vector((OX, 0, 0))
        f["grime"] = material("Grime", (0.1, 0.095, 0.088), roughness=0.9, metallic=0.0)
        f["soot"] = material("SootConcrete", (0.055, 0.052, 0.05), roughness=0.9, metallic=0.0)
        f["plate"] = material("Plate", (0.11, 0.1, 0.1), roughness=0.55, metallic=0.75)
        f["rusted"] = material("Rusted", (0.2, 0.075, 0.035), roughness=0.8, metallic=0.5)
        f["sheet"] = material("Corrugated", (0.17, 0.17, 0.18), roughness=0.5, metallic=0.7)
        f["teal"] = material("TealPaint", (0.03, 0.12, 0.11), roughness=0.7, metallic=0.2)
        f["ochre"] = material("OchrePaint", (0.3, 0.17, 0.05), roughness=0.8, metallic=0.1)
        self.tarps = [material(f"Tarp{k}", rgb, roughness=0.85, metallic=0.0) for k, rgb in enumerate(
            ((0.04, 0.08, 0.25), (0.35, 0.12, 0.03), (0.05, 0.14, 0.05), (0.25, 0.03, 0.08), (0.2, 0.18, 0.15)))]
        self.pavement = material("Pavement", (0.075, 0.07, 0.068), roughness=0.85, metallic=0.0)
        self.scrap = material("Scrap", (0.09, 0.07, 0.06), roughness=0.9, metallic=0.4)
        self.lines["acid"] = glow("TrimAcid", ACID, 3.0)
        self.lines["amber"] = glow("TrimAmber", AMBER, 3.0)
        for name, rgb in (("acid", ACID), ("amber", AMBER)):
            self.shade_of[id(self.lines[name])] = rgb
        self.fire = glow("Fire", (1.0, 0.3, 0.05), 8.0)
        # Windows lit by lamps and screens, dim and orange, not offices' white.
        self.windows["warm"] = glow("WindowWarmSubura", (1.0, 0.45, 0.16), 1.1)
        self.windows["cool"] = glow("WindowCoolSubura", (0.5, 0.95, 0.8), 0.9)
        self.shade_of[id(self.fire)] = (1.0, 0.3, 0.05)

    def mesh(self, verts, faces, col, mat, cover="none"):
        return super().mesh([Vector((v[0] + OX, v[1], v[2])) for v in verts], faces, col, mat, cover)

    def shade(self):
        return self.paint.choice(NEON)


def marker(name, x, y, z, facing):
    br_town.marker(name, x + OX, y, z, facing)


def area(i0, i1, j0, j1):
    """The ground blocks i0..i1, j0..j1 cover, kerb to kerb, with the streets between them."""
    h = PITCH / 2 - STREET / 2
    (ax, ay), (bx, by) = block_middle(i0, j0), block_middle(i1, j1)
    return ax - h, bx + h, ay - h, by + h


def within(i, j, blocks):
    i0, i1, j0, j1 = blocks
    return i0 <= i <= i1 and j0 <= j <= j1


# Pieces.

def neon(m, rng, x, y, z, along, size, shade=None):
    """A neon sign standing out from a wall at (x, y) along `along` (a unit vector out from it): a dark
    board, its face lit in bars of a colour or two."""
    out = along.normalized()
    side = out.cross(Z).normalized()
    w, h = size, size * rng.uniform(1.6, 3.2)
    shade = shade or m.shade()
    other = rng.choice(NEON)
    p = Vector((x, y, z))
    for s in (-1, 1):
        beam(m, p + side * (s * 0.04), p + side * (s * 0.04) + out * 0.6, 0.06, 0.06, m.rail)
    board = p + out * (0.6 + w / 2)
    m.box(Frame(board.x, board.y, math.atan2(out.y, out.x)), -w / 2, w / 2, -0.06, 0.06, z - h / 2, z + h / 2, m.struct,
          m.facades["plate"])
    n = max(2, int(h / 0.55))
    for k in range(n):
        if rng.random() < 0.25:
            continue
        zz = z - h / 2 + (k + 0.5) * h / n
        length = w * rng.uniform(0.35, 0.8)
        for s in (-1, 1):
            q = board + side * (s * 0.07) + Vector((0, 0, zz - z))
            m.line(q - out * length / 2, q + out * length / 2, m.lines[shade if k % 3 else other], width=0.12,
                   across=Z, light=False)


def window_dots(m, rng, poly, z0, z1, floor, lit=0.3, size=1.0):
    """Small windows here and there over the faces of `poly`, lit warm or in neon, few of them."""
    c = middle(poly)
    n = len(poly)
    for i in range(n):
        a, b = poly[i], poly[(i + 1) % n]
        e = b - a
        length = e.length
        if length < 2.0:
            continue
        u = e / length
        out = Vector((u.y, -u.x, 0))
        if out.dot((a + b) / 2 - c) < 0:
            out = -out
        z = z0 + floor * 0.5
        while z < z1 - 0.5:
            t = rng.uniform(0.5, 2.5)
            while t < length - 1.2:
                if rng.random() < lit:
                    p = a + u * t + out * 0.05
                    mat = m.windows["warm"] if rng.random() < 0.8 else m.lines[rng.choice(NEON)]
                    m.line(Vector((p.x, p.y, z)), Vector((p.x, p.y, z)) + u * size, mat, width=size * 0.8,
                           height=0.05, across=Z, light=False)
                t += rng.uniform(2.5, 6.0)
            z += floor


def roof_clutter(m, rng, poly, z):
    """On a roof: a shack or two, a water tank, an aerial, a dish."""
    x0, x1, y0, y1 = bounds(poly)
    for _ in range(rng.randint(1, 3)):
        w, d = rng.uniform(2.0, 4.0), rng.uniform(2.0, 4.0)
        if x1 - x0 < w + 1 or y1 - y0 < d + 1:
            break
        sx, sy = rng.uniform(x0 + 0.5, x1 - w - 0.5), rng.uniform(y0 + 0.5, y1 - d - 0.5)
        what = rng.random()
        if what < 0.45:
            m.box(WORLD, sx, sx + w, sy, sy + d, z, z + 2.4, m.struct, rng.choice((m.facades["sheet"], m.facades["rusted"])))
            m.box(WORLD, sx - 0.2, sx + w + 0.2, sy - 0.2, sy + d + 0.2, z + 2.4, z + 2.55, m.struct, m.facades["sheet"])
        elif what < 0.75:
            r = min(w, d) / 2
            cyl(m, sx + r, sy + r, r, z + 1.2, z + 3.4, m.facades["rusted"], n=10)
            for dx, dy in ((-0.6, -0.6), (0.6, 0.6), (-0.6, 0.6), (0.6, -0.6)):
                beam(m, V(sx + r + dx * r, sy + r + dy * r, z), V(sx + r + dx * r, sy + r + dy * r, z + 1.2), 0.15,
                     0.15, m.rail)
        else:
            h = rng.uniform(4.0, 9.0)
            beam(m, V(sx, sy, z), V(sx, sy, z + h), 0.12, 0.12, m.rail)
            m.line(V(sx, sy, z + h), V(sx, sy, z + h + 0.3), m.lines["red"], width=0.2, light=False)


# Buildings. Each stands on `poly`, a rectangle within its lot.

def shanty(m, rng, poly, district, street):
    """Shacks crowded on the lot, lanes between them: sheet and plate, a storey or two, roofs
    overhanging, a light at a door here and a strip of neon there."""
    x0, x1, y0, y1 = bounds(poly)
    lane = 1.8
    nx = max(1, int((x1 - x0 + lane) // rng.uniform(5.0, 7.5)))
    ny = max(1, int((y1 - y0 + lane) // rng.uniform(5.0, 7.5)))
    cw, cd = (x1 - x0 + lane) / nx - lane, (y1 - y0 + lane) / ny - lane
    for a in range(nx):
        for b in range(ny):
            if rng.random() < 0.12:
                continue
            hx0 = x0 + a * (cw + lane) + rng.uniform(0, 0.6)
            hy0 = y0 + b * (cd + lane) + rng.uniform(0, 0.6)
            hx1, hy1 = hx0 + cw - rng.uniform(0, 0.8), hy0 + cd - rng.uniform(0, 0.8)
            if hx1 - hx0 < 2.0 or hy1 - hy0 < 2.0:
                continue
            mat = rng.choice((m.facades["sheet"], m.facades["rusted"], m.facades["plate"], m.facades["teal"],
                              m.facades["ochre"], m.facades["grime"]))
            h = rng.uniform(2.8, 4.2)
            m.box(WORLD, hx0, hx1, hy0, hy1, KERB, h, m.struct, mat)
            m.box(WORLD, hx0 - 0.3, hx1 + 0.3, hy0 - 0.3, hy1 + 0.3, h, h + 0.12, m.struct, m.facades["sheet"])
            top = h + 0.12
            if rng.random() < 0.45:
                # A storey more, set back or out over the lane.
                ux0, uy0 = hx0 + rng.uniform(-0.4, 1.0), hy0 + rng.uniform(-0.4, 1.0)
                ux1, uy1 = hx1 - rng.uniform(-0.4, 1.0), hy1 - rng.uniform(-0.4, 1.0)
                h2 = top + rng.uniform(2.5, 3.2)
                m.box(WORLD, ux0, ux1, uy0, uy1, top, h2, m.struct, rng.choice((m.facades["sheet"], m.facades["rusted"],
                                                                                  m.facades["ochre"])))
                m.box(WORLD, ux0 - 0.3, ux1 + 0.3, uy0 - 0.3, uy1 + 0.3, h2, h2 + 0.12, m.struct, m.facades["sheet"])
                top = h2 + 0.12
            if rng.random() < 0.6:
                # A light by its door, on whichever side.
                side = rng.randrange(4)
                p = [V((hx0 + hx1) / 2, hy0 - 0.06, 2.2), V(hx1 + 0.06, (hy0 + hy1) / 2, 2.2),
                     V((hx0 + hx1) / 2, hy1 + 0.06, 2.2), V(hx0 - 0.06, (hy0 + hy1) / 2, 2.2)][side]
                along = V(1, 0) if side in (0, 2) else V(0, 1)
                mat = m.windows["warm"] if rng.random() < 0.6 else m.lines[rng.choice(NEON)]
                m.line(p - along * 0.4, p + along * 0.4, mat, width=0.25, height=0.05, across=Z, light=False)
            if rng.random() < 0.3:
                m.ring(rect(hx0, hx1, hy0, hy1), top - 0.4, m.lines[rng.choice(NEON)], out=0.05, width=0.06,
                       light=False)
            if rng.random() < 0.25:
                roof_clutter(m, rng, rect(hx0, hx1, hy0, hy1), top)
    # A sign over the way in from the street.
    if street and rng.random() < 0.5:
        face = street[0]
        x, y, out = [((x0 + x1) / 2, y0, V(0, -1)), (x1, (y0 + y1) / 2, V(1, 0)), ((x0 + x1) / 2, y1, V(0, 1)),
                     (x0, (y0 + y1) / 2, V(-1, 0))][face]
        beam(m, V(x, y, KERB), V(x, y, 5.5), 0.2, 0.2, m.rail)
        neon(m, rng, x, y, 4.6, out, 0.7)


def stack(m, rng, poly, district, street):
    """A tower of containers stacked on a core, each layer turned or shifted from the one under it,
    lit at its windows and edged here and there in neon."""
    x0, x1, y0, y1 = bounds(poly)
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    span = min(x1 - x0, y1 - y0)
    core = min(span * 0.4, 9.0) / 2
    layers = rng.randint(5, 14)
    lh = 2.9
    top = KERB + layers * lh
    m.box(WORLD, cx - core, cx + core, cy - core, cy + core, KERB, top, m.struct, m.facades["soot"])
    window_dots(m, rng, rect(cx - core, cx + core, cy - core, cy + core), KERB, top, lh, lit=0.25, size=0.8)
    reach = span / 2 - 0.3
    for k in range(1, layers):
        z = KERB + k * lh
        for _ in range(rng.randint(1, 3)):
            long_x = rng.random() < 0.5
            length, width = rng.uniform(6.0, 12.0), 2.5
            lx, ly = (length, width) if long_x else (width, length)
            px = cx + rng.uniform(-reach + lx / 2, reach - lx / 2) if reach > lx / 2 else cx
            py = cy + rng.uniform(-reach + ly / 2, reach - ly / 2) if reach > ly / 2 else cy
            box = rect(px - lx / 2, px + lx / 2, py - ly / 2, py + ly / 2)
            mat = rng.choice((m.facades["rusted"], m.facades["teal"], m.facades["ochre"], m.facades["sheet"],
                              m.tarps[1], m.tarps[0]))
            m.prism(box, z, z + lh - 0.1, mat)
            if rng.random() < 0.5:
                window_dots(m, rng, box, z, z + lh, lh, lit=0.5, size=0.9)
            if rng.random() < 0.2:
                m.ring(box, z + lh - 0.15, m.lines[rng.choice(NEON)], out=0.04, width=0.07, light=False)
    roof_clutter(m, rng, rect(cx - core, cx + core, cy - core, cy + core), top)
    if street and rng.random() < 0.6:
        face = street[0]
        x, y, out = [(cx, cy - core, V(0, -1)), (cx + core, cy, V(1, 0)), (cx, cy + core, V(0, 1)),
                     (cx - core, cy, V(-1, 0))][face]
        neon(m, rng, x, y, rng.uniform(8.0, min(20.0, top - 3.0)), out, rng.uniform(1.0, 1.8))


def slab(m, rng, poly, district, street):
    """A tenement slab gone to seed: grimy, tall, a balcony along every floor on its street side
    hung with washing and boxes, a sign up its corner and shacks on its roof."""
    x0, x1, y0, y1 = bounds(poly)
    fh = 3.0
    floors = rng.randint(8, 22)
    h = KERB + floors * fh
    body = rect(x0 + 1.5, x1 - 1.5, y0 + 1.5, y1 - 1.5)
    bx0, bx1, by0, by1 = bounds(body)
    m.prism(body, 0.0, h, rng.choice((m.facades["grime"], m.facades["soot"], m.facades["concrete"])))
    window_dots(m, rng, body, KERB, h, fh, lit=0.45, size=1.0)
    face = street[0] if street else 0
    for f in range(1, floors):
        z = KERB + f * fh
        if rng.random() < 0.2:
            continue
        if face in (0, 2):
            y = by0 if face == 0 else by1
            way = -1 if face == 0 else 1
            m.box(WORLD, bx0 + 0.5, bx1 - 0.5, y, y + way * 1.3, z - 0.15, z, m.struct, m.facades["concrete"])
            if rng.random() < 0.4:
                m.line(V(bx0 + 0.5, y + way * 1.3, z + 1.0), V(bx1 - 0.5, y + way * 1.3, z + 1.0),
                       m.lines[rng.choice(NEON)], width=0.05, light=False)
            for _ in range(rng.randint(0, 3)):
                t = rng.uniform(bx0 + 1.0, bx1 - 1.6)
                m.box(WORLD, t, t + 0.9, y + way * 0.3, y + way * 1.0, z, z + 0.7, m.struct, m.facades["plate"])
        else:
            x = bx1 if face == 1 else bx0
            way = 1 if face == 1 else -1
            m.box(WORLD, x, x + way * 1.3, by0 + 0.5, by1 - 0.5, z - 0.15, z, m.struct, m.facades["concrete"])
            if rng.random() < 0.4:
                m.line(V(x + way * 1.3, by0 + 0.5, z + 1.0), V(x + way * 1.3, by1 - 0.5, z + 1.0),
                       m.lines[rng.choice(NEON)], width=0.05, light=False)
            for _ in range(rng.randint(0, 3)):
                t = rng.uniform(by0 + 1.0, by1 - 1.6)
                m.box(WORLD, x + way * 0.3, x + way * 1.0, t, t + 0.9, z, z + 0.7, m.struct, m.facades["plate"])
    corner = [V(bx0, by0), V(bx1, by0), V(bx1, by1), V(bx0, by1)][face]
    out = [V(0, -1), V(1, 0), V(0, 1), V(-1, 0)][face]
    neon(m, rng, corner.x, corner.y, min(h - 4.0, rng.uniform(10.0, 26.0)), out, rng.uniform(1.4, 2.4))
    roof_clutter(m, rng, body, h)


def workshop(m, rng, poly, district, street):
    """A workshop: a shed of sheet, its door rolled up on a glow inside, scrap heaped by it."""
    x0, x1, y0, y1 = bounds(poly)
    h = rng.uniform(4.5, 7.0)
    shed = rect(x0 + 1.0, x1 - 1.0, y0 + 1.0, y1 - 1.0)
    m.prism(shed, 0.0, h, rng.choice((m.facades["sheet"], m.facades["rusted"])))
    m.box(WORLD, x0 + 0.6, x1 - 0.6, y0 + 0.6, y1 - 0.6, h, h + 0.15, m.struct, m.facades["sheet"])
    face = street[0] if street else rng.randrange(4)
    sx0, sx1, sy0, sy1 = bounds(shed)
    p, along = [(V((sx0 + sx1) / 2, sy0 - 0.05), V(1, 0)), (V(sx1 + 0.05, (sy0 + sy1) / 2), V(0, 1)),
                (V((sx0 + sx1) / 2, sy1 + 0.05), V(1, 0)), (V(sx0 - 0.05, (sy0 + sy1) / 2), V(0, 1))][face]
    w = min(4.0, (sx1 - sx0 if face in (0, 2) else sy1 - sy0) - 1.0)
    m.line(p - along * (w / 2) + V(0, 0, 1.4), p + along * (w / 2) + V(0, 0, 1.4), m.lines["amber"], width=2.4,
           height=0.04, across=Z, light=True)
    m.line(p - along * (w / 2) + V(0, 0, 2.9), p + along * (w / 2) + V(0, 0, 2.9), m.lines[m.shade()], width=0.1)
    if rng.random() < 0.5:
        roof_clutter(m, rng, shed, h + 0.15)


def strip_front(m, rng, poly, district, street):
    """On the Strip: two to four storeys, shopfronts lit along the street and great signs over them."""
    x0, x1, y0, y1 = bounds(poly)
    floors = rng.randint(2, 4)
    h = KERB + floors * 3.4
    body = rect(x0 + 0.8, x1 - 0.8, y0 + 0.8, y1 - 0.8)
    m.prism(body, 0.0, h, rng.choice((m.facades["grime"], m.facades["soot"], m.facades["plate"])))
    window_dots(m, rng, body, KERB + 3.4, h, 3.4, lit=0.45, size=1.2)
    bx0, bx1, by0, by1 = bounds(body)
    for face in street or [0]:
        a, b, out = [(V(bx0, by0), V(bx1, by0), V(0, -1)), (V(bx1, by0), V(bx1, by1), V(1, 0)),
                     (V(bx1, by1), V(bx0, by1), V(0, 1)), (V(bx0, by1), V(bx0, by0), V(-1, 0))][face]
        u = (b - a).normalized()
        length = (b - a).length
        shade = m.shade()
        m.line(a + out * 0.05 + V(0, 0, 1.6), b + out * 0.05 + V(0, 0, 1.6), m.lines["amber"], width=2.2, height=0.04,
               across=Z, light=False)
        m.line(a + out * 0.4 + V(0, 0, 3.2), b + out * 0.4 + V(0, 0, 3.2), m.lines[shade], width=0.12)
        # A great sign flat on the face over the shopfronts, its bars in a colour or two.
        t = 1.0
        while t < length - 3.0:
            w = min(rng.uniform(3.0, 7.0), length - t - 1.0)
            z0, z1 = 4.0, min(h - 0.5, 4.0 + rng.uniform(2.0, 6.0))
            p = a + u * t + out * 0.15
            c = p + u * (w / 2)
            m.box(Frame(c.x, c.y, math.atan2(u.y, u.x)), -w / 2, w / 2, -0.1, 0.1, z0, z1, m.struct, m.facades["plate"])
            other = rng.choice(NEON)
            zz = z0 + 0.4
            while zz < z1 - 0.3:
                frac = rng.uniform(0.4, 0.95)
                q = p + out * 0.12 + u * (w * (1 - frac) / 2)
                m.line(q + V(0, 0, zz), q + u * (w * frac) + V(0, 0, zz), m.lines[shade if rng.random() < 0.7 else other],
                       width=0.18, across=Z, light=zz < 6.0)
                zz += rng.uniform(0.5, 0.9)
            t += w + rng.uniform(1.0, 4.0)
    roof_clutter(m, rng, body, h)


# The special blocks.

def market(m, rng, cx, cy, inner):
    """The Night Market: rows of stalls under tarps, lanterns strung over the lanes between them."""
    x0, x1, y0, y1 = inner
    m.solid(rect(x0, x1, y0, y1), KERB, KERB + 0.02, m.ground, m.facades["soot"])
    lane, sw, sd = 3.0, 3.2, 2.6
    rows = int((y1 - y0 - lane) // (2 * sd + lane))
    cols = int((x1 - x0 - 2.0) // (sw + 0.4))
    for r in range(rows):
        yy = y0 + lane + r * (2 * sd + lane)
        for c in range(cols):
            xx = x0 + 1.0 + c * (sw + 0.4)
            for k, (ya, yb) in enumerate(((yy, yy + sd), (yy + sd, yy + 2 * sd))):
                if rng.random() < 0.1:
                    continue
                front = ya if k == 0 else yb           # each faces its lane
                m.box(WORLD, xx, xx + sw, ya + 0.2, yb - 0.2, KERB, KERB + 0.95, m.cover, m.crate, cover="low")
                m.line(V(xx, front, KERB + 0.95), V(xx + sw, front, KERB + 0.95), m.lines[m.shade()], width=0.05,
                       light=rng.random() < 0.3)
                for px in (xx + 0.1, xx + sw - 0.1):
                    beam(m, V(px, (ya + yb) / 2, KERB), V(px, (ya + yb) / 2, 2.9), 0.1, 0.1, m.rail)
                tarp = rng.choice(m.tarps)
                over = -0.6 if k == 0 else 0.6
                m.mesh([V(xx - 0.1, ya + (over if k == 0 else 0), 2.9 if k == 0 else 3.1),
                        V(xx + sw + 0.1, ya + (over if k == 0 else 0), 2.9 if k == 0 else 3.1),
                        V(xx + sw + 0.1, yb + (over if k == 1 else 0), 3.1 if k == 0 else 2.9),
                        V(xx - 0.1, yb + (over if k == 1 else 0), 3.1 if k == 0 else 2.9)],
                       [(0, 1, 2, 3)], m.struct, tarp)
        # Lanterns strung over the lane in front of the row.
        ly = yy - lane / 2
        if ly > y0 + 0.5:
            beam(m, V(x0, ly, 4.2), V(x1, ly, 4.2), 0.03, 0.03, m.rail)
            t = x0 + 1.5
            while t < x1 - 1.0:
                if rng.random() < 0.25:
                    m.glass_pane(WORLD, t - 0.15, t + 0.15, ly - 0.15, ly + 0.15, 3.7, 4.1)
                else:
                    m.line(V(t, ly, 3.75), V(t, ly, 4.15), m.lines[rng.choice(NEON)], width=0.25, light=False)
                t += rng.uniform(1.5, 3.0)
    for x in (x0 + 0.3, x1 - 0.3):
        beam(m, V(x, y0 + 0.3, KERB), V(x, y0 + 0.3, 6.5), 0.2, 0.2, m.rail)
    neon(m, rng, x0 + 0.3, y0 + 0.3, 5.5, V(0, -1), 1.4, "acid")
    marker("place_market", (x0 + x1) / 2, y0 + 1.5, KERB, math.pi / 2)


def brazier(m, x, y):
    """A drum with fire in it."""
    cyl(m, x, y, 0.45, KERB, KERB + 0.9, m.facades["rusted"], n=10, col=m.cover)
    cyl(m, x, y, 0.36, KERB + 0.9, KERB + 1.05, m.fire, n=10)
    m.ring(polygon(0.36, 6, 0, x, y), KERB + 1.06, m.fire, out=0.0, width=0.1)


def stronghold(m, rng, cx, cy, inner):
    """The Praedatores' compound: a wall of plate round it, a gate in the side on the street east,
    a watchtower either side of the gate, braziers in the yard, crates to sit on, and a bunker at
    its back."""
    x0, x1, y0, y1 = inner
    wall_h, t = 4.2, 0.5
    gate = 5.0
    m.solid(rect(x0, x1, y0, y1), KERB, KERB + 0.02, m.ground, m.facades["soot"])
    # The walls, of plates set unevenly, the east one broken by the gate.
    for a, b in ((V(x0, y0), V(x1, y0)), (V(x0, y1), V(x1, y1)), (V(x0, y0), V(x0, y1))):
        u = (b - a)
        n = max(1, int(u.length // 3.0))
        for k in range(n):
            p, q = a + u * (k / n), a + u * ((k + 1) / n)
            h = wall_h + rng.uniform(-0.4, 0.6)
            px0, px1 = sorted((p.x, q.x))
            py0, py1 = sorted((p.y, q.y))
            m.box(WORLD, px0 - t / 2, px1 + t / 2, py0 - t / 2, py1 + t / 2, 0.0, h, m.struct,
                  rng.choice((m.facades["plate"], m.facades["rusted"], m.facades["sheet"])))
        m.line(a + V(0, 0, wall_h - 0.3), b + V(0, 0, wall_h - 0.3), m.lines["pink"], width=0.07)
    for s in (-1, 1):
        y_a, y_b = (cy + s * gate / 2, y1) if s > 0 else (y0, cy - gate / 2)
        m.box(WORLD, x1 - t / 2, x1 + t / 2, y_a, y_b, 0.0, wall_h + 0.3, m.struct, m.facades["plate"])
        m.line(V(x1 + t / 2 + 0.02, y_a, wall_h - 0.3), V(x1 + t / 2 + 0.02, y_b, wall_h - 0.3), m.lines["pink"],
               width=0.07)
        # A watchtower beside the gate: legs, a platform at 6 m, a roof, lit red and pink.
        ty = cy + s * (gate / 2 + 2.2)
        tx = x1 - 2.0
        for dx in (-1.4, 1.4):
            for dy in (-1.4, 1.4):
                beam(m, V(tx + dx, ty + dy, 0.0), V(tx + dx * 0.8, ty + dy * 0.8, 6.0), 0.2, 0.2, m.rail)
        m.box(WORLD, tx - 1.6, tx + 1.6, ty - 1.6, ty + 1.6, 6.0, 6.25, m.struct, m.facades["plate"])
        m.box(WORLD, tx - 1.6, tx + 1.6, ty - 1.6, ty + 1.6, 6.25, 7.3, m.struct, m.facades["rusted"])
        m.box(WORLD, tx - 1.9, tx + 1.9, ty - 1.9, ty + 1.9, 9.0, 9.2, m.struct, m.facades["sheet"])
        for dx in (-1.5, 1.5):
            beam(m, V(tx + dx, ty, 7.3), V(tx + dx, ty, 9.0), 0.12, 0.12, m.rail)
        m.ring(rect(tx - 1.6, tx + 1.6, ty - 1.6, ty + 1.6), 7.25, m.lines["red"], out=0.03, width=0.08)
        m.line(V(tx - 0.3, ty, 8.95), V(tx + 0.3, ty, 8.95), m.lines["pink"], width=0.3, light=False)
    # Over the gate, a girder with the gang's mark: three bars of pink.
    beam(m, V(x1, cy - gate / 2 - 0.5, 6.0), V(x1, cy + gate / 2 + 0.5, 6.0), 0.4, 0.6, m.facades["plate"])
    for k in range(3):
        m.line(V(x1 + 0.25, cy - 1.8 + k * 1.8, 5.55), V(x1 + 0.25, cy - 1.8 + k * 1.8, 6.45), m.lines["pink"],
               width=0.35, across=V(0, 1), light=False)
    # The bunker at the back: low, thick, its door lit, a sign on its roof.
    bx0, bx1, by0, by1 = x0 + 1.5, x0 + 13.0, cy - 9.0, cy + 9.0
    m.box(WORLD, bx0, bx1, by0, by1, 0.0, 4.0, m.struct, m.facades["soot"])
    m.box(WORLD, bx0 - 0.4, bx1 + 0.4, by0 - 0.4, by1 + 0.4, 4.0, 4.4, m.struct, m.facades["plate"])
    m.line(V(bx1 + 0.03, cy - 1.2, 1.2), V(bx1 + 0.03, cy + 1.2, 1.2), m.lines["pink"], width=2.2, height=0.04,
           across=Z, light=True)
    m.ring(rect(bx0, bx1, by0, by1), 3.9, m.lines["violet"], out=0.03, width=0.06)
    neon(m, rng, bx1, by0 + 1.0, 6.0, V(1, 0), 1.2, "pink")
    # The yard: braziers, crates and drums, where the gang stands about - its posts by the fires.
    fires = [(cx - 2.0, cy - 7.0), (cx + 5.0, cy + 6.0), (cx - 6.0, cy + 9.0)]
    for fx, fy in fires:
        brazier(m, fx, fy)
    k = 0
    for fx, fy in fires:
        for ang in (0.6, 2.8):
            k += 1
            marker(f"hideout_{k}", fx + 1.8 * math.cos(ang + fx), fy + 1.8 * math.sin(ang + fx), KERB, 0.0)
    for _ in range(9):
        px, py = rng.uniform(x0 + 15.0, x1 - 4.0), rng.uniform(y0 + 2.0, y1 - 2.0)
        if any(abs(px - fx) < 3.0 and abs(py - fy) < 3.0 for fx, fy in fires) or abs(py - cy) < 4.0:
            continue
        m.box(WORLD, px - 0.6, px + 0.6, py - 0.6, py + 0.6, KERB, KERB + 1.1, m.cover, m.crate, cover="low")
        m.line(V(px - 0.6, py - 0.62, 0.8), V(px + 0.6, py - 0.62, 0.8), m.lines["pink"], width=0.05, light=False)
    marker("place_stronghold", x1 + 4.0, cy, KERB, math.pi)


def pit(m, rng, cx, cy, inner):
    """The Pit: a cage to fight in, ringed by stands stepping up from it, lit from posts at its
    corners."""
    x0, x1, y0, y1 = inner
    m.solid(rect(x0, x1, y0, y1), KERB, KERB + 0.02, m.ground, m.facades["soot"])
    r = 8.0
    ring_ = polygon(r, 8, math.pi / 8, cx, cy)
    m.solid(ring_, KERB, KERB + 0.12, m.ground, m.facades["plate"])
    # The cage: posts at the corners, mesh between them as bars, open on two sides to walk in.
    for i, p in enumerate(ring_):
        beam(m, V(p.x, p.y, KERB), V(p.x, p.y, 4.5), 0.15, 0.15, m.rail)
        q = ring_[(i + 1) % 8]
        if i in (1, 5):
            continue
        for z in (1.2, 2.4, 3.6):
            beam(m, V(p.x, p.y, z), V(q.x, q.y, z), 0.05, 0.05, m.rail)
        m.parapet(V(p.x, p.y), V(q.x, q.y), KERB, m.lines["red"])
    m.ring(ring_, 4.5, m.lines["red"], out=0.0, width=0.1)
    # The stands: tiers of steps on the north and south sides, east to west.
    for way in (-1, 1):
        base = cy + way * (r + 2.0)
        for k in range(8):
            ya, yb = base + way * k * RUN * 2, base + way * (k + 1) * RUN * 2
            m.box(WORLD, cx - 12.0, cx + 12.0, *sorted((ya, yb)), 0.0, KERB + (k + 1) * RISE, m.struct, m.steps)
        top = base + way * 8 * RUN * 2
        m.line(V(cx - 12.0, top, KERB + 8 * RISE + 0.05), V(cx + 12.0, top, KERB + 8 * RISE + 0.05), m.lines["amber"],
               width=0.08)
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = cx + sx * (r + 6.0), cy + sy * (r + 6.0)
            if not (x0 + 1 < x < x1 - 1 and y0 + 1 < y < y1 - 1):
                continue
            m.pylon(WORLD, x, y, KERB, height=9.0, shade="red")
    marker("place_pit", cx, cy - r - 1.5, KERB, math.pi / 2)


def hive(m, rng):
    """The Hive over the four middle blocks and the streets between them: a tower of stacked
    dwellings at each corner, skybridges between them, crowned in neon, round a court open to the
    streets either way through it."""
    x0, x1, y0, y1 = area(*HIVE)
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    m.solid(rect(x0, x1, y0, y1), 0.0, KERB, m.ground, m.pavement)
    m.ring(rect(x0, x1, y0, y1), KERB - 0.02, m.lines["pink"], out=0.03, width=0.06, light=False)
    half = (x1 - x0) / 2
    gap = 9.0                                   # half the ways through, either way
    heights = [118.0, 96.0, 140.0, 84.0]
    towers = []
    for k, (sx, sy) in enumerate(((-1, -1), (1, -1), (1, 1), (-1, 1))):
        tx0, tx1 = sorted((cx + sx * gap, cx + sx * (half - 3.0)))
        ty0, ty1 = sorted((cy + sy * gap, cy + sy * (half - 3.0)))
        h = heights[k]
        towers.append((tx0, tx1, ty0, ty1, h))
        # The core, stepping in as it rises, and the dwellings stacked out from it.
        steps = 4
        for s in range(steps):
            inset = s * 3.0
            za, zb = KERB + h * s / steps, KERB + h * (s + 1) / steps
            body = rect(tx0 + inset, tx1 - inset, ty0 + inset, ty1 - inset)
            m.prism(body, za, zb, m.facades["soot"])
            window_dots(m, rng, body, za, zb, 3.0, lit=0.5, size=1.1)
            m.ring(body, zb - 0.1, m.lines[NEON[(k + s) % len(NEON)]], out=0.05, width=0.15)
            # Boxes stacked out of its faces.
            bx0, bx1, by0, by1 = tx0 + inset, tx1 - inset, ty0 + inset, ty1 - inset
            for _ in range(int((zb - za) / 3.0 * 0.8)):
                z = rng.uniform(za + 6.0, zb - 3.0) if zb - za > 9.0 else za + 6.0
                if z < 6.0:
                    continue
                face = rng.randrange(4)
                w, d = rng.uniform(3.0, 7.0), rng.uniform(1.5, 3.0)
                if face == 0:
                    t = rng.uniform(bx0, max(bx0, bx1 - w))
                    box = (t, t + w, by0 - d, by0)
                elif face == 1:
                    t = rng.uniform(by0, max(by0, by1 - w))
                    box = (bx1, bx1 + d, t, t + w)
                elif face == 2:
                    t = rng.uniform(bx0, max(bx0, bx1 - w))
                    box = (t, t + w, by1, by1 + d)
                else:
                    t = rng.uniform(by0, max(by0, by1 - w))
                    box = (bx0 - d, bx0, t, t + w)
                mat = rng.choice((m.facades["rusted"], m.facades["teal"], m.facades["ochre"], m.facades["sheet"]))
                m.box(WORLD, *box, z, z + 2.8, m.struct, mat)
                if rng.random() < 0.5:
                    window_dots(m, rng, rect(*box), z, z + 2.8, 2.8, lit=0.6, size=0.9)
        # The crown: a mast and rings of neon.
        mx, my = (tx0 + tx1) / 2, (ty0 + ty1) / 2
        beam(m, V(mx, my, KERB + h), V(mx, my, KERB + h + 22.0), 0.6, 0.6, m.rail)
        for z, r in ((KERB + h + 6.0, 4.0), (KERB + h + 12.0, 3.0), (KERB + h + 18.0, 2.0)):
            m.ring(polygon(r, 12, 0, mx, my), z, m.lines[NEON[k % len(NEON)]], out=0.0, width=0.3)
        m.line(V(mx, my, KERB + h + 22.0), V(mx, my, KERB + h + 22.6), m.lines["red"], width=0.4, light=False)
        # At the foot of the tower, on the court, its doorway lit, and a sign.
        door_x = tx1 if sx < 0 else tx0
        m.line(V(door_x - sx * 0.05, (ty0 + ty1) / 2 - 1.2, 1.4), V(door_x - sx * 0.05, (ty0 + ty1) / 2 + 1.2, 1.4),
               m.lines["amber"], width=2.4, height=0.04, across=Z)
        neon(m, rng, door_x, ty0 + 4.0 if sy > 0 else ty1 - 4.0, 9.0, V(-sx, 0), 2.0)
    # Skybridges between them, high over the ways through.
    for (a, b), z in (((0, 1), 36.0), ((1, 2), 52.0), ((2, 3), 30.0), ((3, 0), 64.0), ((0, 1), 78.0)):
        ta, tb = towers[a], towers[b]
        if ta[0] == tb[0]:      # one north of the other
            x = (ta[0] + ta[1]) / 2
            ya, yb = sorted((ta[3] if ta[2] < tb[2] else ta[2], tb[2] if ta[2] < tb[2] else tb[3]))
            m.box(WORLD, x - 2.0, x + 2.0, ya - 12.0, yb + 12.0, z, z + 3.2, m.struct, m.facades["plate"])
            m.line(V(x - 2.02, ya, z + 1.6), V(x - 2.02, yb, z + 1.6), m.lines["acid"], width=0.6, height=0.04,
                   across=Z, light=False)
            m.line(V(x + 2.02, ya, z + 1.6), V(x + 2.02, yb, z + 1.6), m.lines["acid"], width=0.6, height=0.04,
                   across=Z, light=False)
        else:
            y = (ta[2] + ta[3]) / 2
            xa, xb = sorted((ta[1] if ta[0] < tb[0] else ta[0], tb[0] if ta[0] < tb[0] else tb[1]))
            m.box(WORLD, xa - 12.0, xb + 12.0, y - 2.0, y + 2.0, z, z + 3.2, m.struct, m.facades["plate"])
            m.line(V(xa, y - 2.02, z + 1.6), V(xb, y - 2.02, z + 1.6), m.lines["pink"], width=0.6, height=0.04,
                   across=Z, light=False)
            m.line(V(xa, y + 2.02, z + 1.6), V(xb, y + 2.02, z + 1.6), m.lines["pink"], width=0.6, height=0.04,
                   across=Z, light=False)
    # The court: stalls about its middle, braziers, and over it lanterns strung from tower to tower.
    for k in range(6):
        ang = k / 6 * math.tau
        px, py = cx + 5.5 * math.cos(ang), cy + 5.5 * math.sin(ang)
        if k % 2:
            brazier(m, px, py)
        else:
            m.box(WORLD, px - 1.0, px + 1.0, py - 0.6, py + 0.6, KERB, KERB + 0.95, m.cover, m.crate, cover="low")
            m.line(V(px - 1.0, py - 0.62, KERB + 0.95), V(px + 1.0, py - 0.62, KERB + 0.95), m.lines[m.shade()],
                   width=0.05)
    for (ax, ay), (bx, by) in (((cx - gap, cy - gap), (cx + gap, cy + gap)), ((cx - gap, cy + gap), (cx + gap, cy - gap))):
        beam(m, V(ax, ay, 11.0), V(bx, by, 11.0), 0.04, 0.04, m.rail)
        for t in (0.2, 0.35, 0.65, 0.8):
            p = V(ax + (bx - ax) * t, ay + (by - ay) * t, 10.6)
            m.line(p, p + V(0, 0, 0.35), m.lines[NEON[int(t * 10) % len(NEON)]], width=0.3, light=False)
    marker("place_hive", cx, y0 + 2.0, KERB, math.pi / 2)


# The districts' lots, and what goes up on them.

def district_of(i, j):
    x, y = block_middle(i, j)
    if y < CANAL[0]:
        return "sump"
    if i == 0:
        return "scrapyard"
    if i >= STRIP:
        return "strip"
    r = max(abs(x), abs(y))
    return "core" if r < 200 else "stacks"


OPTIONS = {
    "core": [slab, slab, stack, stack, shanty],
    "stacks": [shanty, shanty, shanty, stack, stack, slab, workshop],
    "strip": [strip_front, strip_front, strip_front, stack, shanty],
    "sump": [shanty, shanty, workshop],
    "scrapyard": [workshop, shanty],
}
LOTS = {"core": "midtown", "stacks": "outer", "strip": "midtown", "sump": "outer", "scrapyard": "docks"}

OFFICE_CHANCE = 0.3      # the chance a lot for a slab is an office tower to go up instead
MOST_OFFICES = 5         # and the most of them


def build_lot(m, rng, district, lot, block):
    x0, x1, y0, y1 = lot
    street = faces_on_street(lot, block)
    rng.shuffle(street)
    inset = rng.uniform(0.3, 1.0)
    poly = rect(x0 + inset, x1 - inset, y0 + inset, y1 - inset)
    small = min(x1 - x0, y1 - y0)
    build = rng.choice(OPTIONS[district])
    if small < 12 and build is slab:
        build = stack
    if small < 8 and build is stack:
        build = shanty
    if (build is slab and street and small >= urbs.OFFICE_SIZE[0]
            and m.n_lifts < MOST_OFFICES and m.lifts.random() < OFFICE_CHANCE):
        # An office tower to go up in its place (see urbs.office_tower): the one it stands for built
        # out of sight, for its draws of chance, so that the rest of the city is as it was.
        m.unseen(lambda: build(m, rng, poly, district, street))
        urbs.office_tower(m, m.lifts, poly, street, m.facades["grime"], urbs.unpainted_shade(m))
        m.n_buildings += 1
        return
    build(m, rng, poly, district, street)
    m.n_buildings += 1


def scrapyard(m, rng, cx, cy, inner):
    """Heaps of scrap on the block, wrecks among them."""
    x0, x1, y0, y1 = inner
    for _ in range(rng.randint(4, 7)):
        hx, hy = rng.uniform(x0 + 5, x1 - 5), rng.uniform(y0 + 5, y1 - 5)
        r = rng.uniform(3.0, 5.0)
        h = rng.uniform(2.5, 6.0)
        m.prism(polygon(r, 7, rng.uniform(0, 1), hx, hy), KERB, KERB + h * 0.2, m.scrap,
                top=polygon(r * 0.75, 7, rng.uniform(0, 1), hx, hy))
        m.prism(polygon(r * 0.75, 7, 0.3, hx, hy), KERB + h * 0.2, KERB + h, m.scrap,
                top=polygon(r * 0.2, 7, 0.7, hx, hy))
    for _ in range(rng.randint(3, 6)):
        wx, wy = rng.uniform(x0 + 2, x1 - 2), rng.uniform(y0 + 2, y1 - 2)
        f = Frame(wx, wy, rng.uniform(0, math.pi))
        m.box(f, -2.0, 2.0, -0.9, 0.9, KERB, KERB + 1.2, m.cover, rng.choice((m.facades["rusted"], m.scrap)), cover="low")


def block(m, rng, i, j):
    """A block: its pavement, its lamps, and what stands on it."""
    if within(i, j, HIVE):
        return
    cx, cy = block_middle(i, j)
    h = PITCH / 2 - STREET / 2
    x0, x1, y0, y1 = cx - h, cx + h, cy - h, cy + h
    district = district_of(i, j)
    m.solid(rect(x0, x1, y0, y1), 0.0, KERB, m.ground, m.pavement)
    m.ring(rect(x0, x1, y0, y1), KERB - 0.02, m.lines["pink" if district != "sump" else "amber"], out=0.03,
           width=0.06, light=False)
    for sx, sy in ((-1, -1), (1, 1)) if (i + j) % 2 else ((1, -1), (-1, 1)):
        m.pylon(WORLD, cx + sx * (h - 1.5), cy + sy * (h - 1.5), KERB, height=6.0, shade=m.shade())
    inner = (x0 + SIDEWALK, x1 - SIDEWALK, y0 + SIDEWALK, y1 - SIDEWALK)
    special = SPECIAL_AT.get((i, j))
    if special == "market":
        return market(m, rng, cx, cy, inner)
    if special == "stronghold":
        return stronghold(m, rng, cx, cy, inner)
    if special == "pit":
        return pit(m, rng, cx, cy, inner)
    if district == "scrapyard" and j % 3 == 1:
        return scrapyard(m, rng, cx, cy, inner)
    for lot in urbs.lots(rng, LOTS[district], *inner):
        build_lot(m, rng, district, lot, inner)


def cables(m, rng):
    """Over the streets, cables strung across from poles on the pavements either side, here and
    there hung with lanterns - but not over the railway, or the Hive's court."""
    lines_at = [k * PITCH for k in range(-BLOCKS // 2 + 1, BLOCKS // 2)]
    hx0, hx1, hy0, hy1 = area(*HIVE)
    n = 0
    for c in lines_at:
        for along_x in (True, False):
            for k in range(-BLOCKS // 2, BLOCKS // 2):
                if rng.random() > 0.3:
                    continue
                t = (k + 0.5) * PITCH + rng.uniform(-14.0, 14.0)
                x, y = (t, c) if along_x else (c, t)
                if along_x and abs(c) < 1 and x > LINE_BUFFER - 40:
                    continue
                if hx0 - 2 < x < hx1 + 2 and hy0 - 2 < y < hy1 + 2:
                    continue
                if abs(y - CANAL[0]) < CANAL[1] / 2 + 3:
                    continue
                z = rng.uniform(6.0, 8.0)
                across = V(0, 1) if along_x else V(1, 0)
                a = V(x, y) - across * (STREET / 2 + 1.0)
                b = V(x, y) + across * (STREET / 2 + 1.0)
                for p in (a, b):
                    beam(m, V(p.x, p.y, KERB), V(p.x, p.y, z + 0.4), 0.14, 0.14, m.rail)
                sag = V(0, 0, -0.5)
                mid = (a + b) / 2 + V(0, 0, z) + sag
                beam(m, a + V(0, 0, z), mid, 0.04, 0.04, m.rail)
                beam(m, mid, b + V(0, 0, z), 0.04, 0.04, m.rail)
                for u in (0.3, 0.5, 0.7):
                    if rng.random() < 0.6:
                        p = a + (b - a) * u + V(0, 0, z - 0.6 - 0.5 * (1 - abs(u - 0.5) * 2))
                        m.line(p, p + V(0, 0, 0.3), m.lines[rng.choice(NEON)], width=0.28, light=False)
                n += 1
    print(f"[map] {n} cables over the streets")


# The ground: the platform, the canal, the edge, and the bridge from Urbs.

def ground(m):
    cy, cw, cd = CANAL
    south, north = cy - cw / 2, cy + cw / 2
    pink = m.lines["pink"]
    for y0, y1 in ((-HALF, south), (north, HALF)):
        slab_ = rect(-HALF, HALF, y0, y1)
        m.solid(slab_, -DEPTH, -0.002, m.ground, m.road)
        m.ring(slab_, -0.3, pink, out=0.0, light=False)
        m.ring(slab_, -DEPTH + 0.15, pink, out=0.0, light=False)
    m.box(WORLD, -HALF, HALF, south, north, -DEPTH, -cd, m.ground, m.water)
    for y in (south + 0.05, north - 0.05):
        m.line(V(-HALF, y, -cd + 0.05), V(HALF, y, -cd + 0.05), m.lines["acid"], width=0.08)
    streets_at = [k * PITCH for k in range(-BLOCKS // 2 + 1, BLOCKS // 2)] + [-(HALF - STREET / 2), HALF - STREET / 2]
    for x in streets_at:
        b0, b1 = x - BRIDGE_WIDTH / 2, x + BRIDGE_WIDTH / 2
        m.box(WORLD, b0, b1, south - 0.3, north + 0.3, -0.6, -0.002, m.ground, m.road)
        for bx in (b0 + 0.2, b1 - 0.2):
            m.parapet(V(bx, south, 0), V(bx, north, 0), 0.0, pink)
    edges = sorted(v for x in streets_at for v in (x - BRIDGE_WIDTH / 2, x + BRIDGE_WIDTH / 2))
    spans = [(-HALF, edges[0])] + [(edges[k], edges[k + 1]) for k in range(1, len(edges) - 1, 2)] + [(edges[-1], HALF)]
    for s0, s1 in spans:
        if s1 - s0 > 1:
            for y, way in ((south, +1), (north, -1)):
                m.parapet(V(s0, y - way * 0.2, 0), V(s1, y - way * 0.2, 0), 0.0, pink)
    # The edge: a parapet all round, but for the bridge from Urbs in the east and the canal.
    corners = rect(-HALF, HALF, -HALF, HALF)
    for i in range(4):
        a, b = corners[i], corners[(i + 1) % 4]
        inward = (middle(corners) - (a + b) / 2).normalized() * 0.3
        gaps = []
        if i in (1, 3):
            lo, hi = (south - a.y) / (b.y - a.y), (north - a.y) / (b.y - a.y)
            gaps.append((min(lo, hi), max(lo, hi)))
        if i == 1:
            g = STREET / 2 / (b - a).length
            gaps.append((0.5 - g, 0.5 + g))
        m.parapet(a + inward, b + inward, 0.0, pink, gaps)


def link(m):
    """The road over the void from Urbs' west gate pad to Subura's middle street, and the Breach
    over it at Subura's edge."""
    x0, x1 = HALF, HALF + LINK
    w = STREET / 2
    m.box(WORLD, x0 - 0.5, x1, -w, w, -1.2, -0.002, m.ground, m.road)
    for s in (-1, 1):
        m.line(V(x0, s * w, -0.6), V(x1, s * w, -0.6), m.lines["pink"], light=False)
        m.parapet(V(x0, s * (w - 0.2), 0), V(x1, s * (w - 0.2), 0), 0.0, m.lines["pink"])
    x = x0 + 2.0
    while x < x1 - 2.0:
        m.line(V(x, 0.0, 0.004), V(x + 3.0, 0.0, 0.004), m.marking, width=0.15, height=0.008, light=False)
        x += 6.0
    for x in (x1 - 4.0, (x0 + x1) / 2):
        for s in (-1, 1):
            m.pylon(WORLD, x, s * (w - 1.0), 0.0, height=7.0, shade="pink")
    # The Breach: two towers of welded plate either side, a girder across them high over the
    # railway, the city's name on it in bars of pink and green.
    gx = x0 + 3.0
    for s in (-1, 1):
        ya, yb = sorted((s * (w + 0.4), s * (w + 3.4)))
        for k in range(6):
            za, zb = k * 3.0, (k + 1) * 3.0
            jog = 0.3 if k % 2 else 0.0
            m.box(WORLD, gx - 1.5 + jog, gx + 1.5 + jog, ya, yb, za, zb, m.struct,
                  (m.facades["plate"], m.facades["rusted"])[k % 2])
        m.fins(rect(gx - 1.5, gx + 1.5, ya, yb), rect(gx - 1.5, gx + 1.5, ya, yb), 0.0, 18.0, m.lines["pink"])
    m.box(WORLD, gx - 1.2, gx + 1.2, -w - 3.4, w + 3.4, 14.5, 17.0, m.struct, m.facades["plate"])
    for k in range(9):
        y = -6.0 + k * 1.5
        m.line(V(gx + 1.22, y, 15.0), V(gx + 1.22, y, 16.5), m.lines["acid" if k % 3 == 1 else "pink"], width=0.5,
               across=V(0, 1), light=False)
    marker("place_subura_bridge", x0 - 6.0, 0.0, 0.0, math.pi)


def road_plan():
    """Subura's roads and crossings, as Urbs' are: its streets are where Urbs' are, moved west, but
    for those under the Hive."""
    plan = urbs.road_plan()
    hx0, hx1, hy0, hy1 = area(*HIVE)
    hole = (hx0, -hy1, hx1, -hy0)            # in the game's terms, x0, z0, x1, z1
    overlaps = lambda r: r[0] < hole[2] and r[2] > hole[0] and r[1] < hole[3] and r[3] > hole[1]
    roads = []
    for x0, z0, x1, z1 in plan["roads"]:
        for a0, a1, b0, b1 in cut((x0, x1, z0, z1), [(hole[0], hole[2], hole[1], hole[3])]):
            roads.append([round(a0, 3), round(b0, 3), round(a1, 3), round(b1, 3)])
    crossings = [r for r in plan["crossings"] if not overlaps(r)]
    move = lambda r: [round(r[0] + OX, 3), r[1], round(r[2] + OX, 3), r[3]]
    return {"level": plan["level"], "roads": [move(r) for r in roads], "crossings": [move(r) for r in crossings]}


def places():
    for name, (i, j) in (("place_sump", (5, 1)), ("place_strip", (12, 9)), ("place_scrapyard", (0, 7))):
        cx, cy = block_middle(i, j)
        marker(name, cx - PITCH / 2 + 4.0, cy, KERB, 0.0)


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Subura(random.Random(seed + 1))
    ground(m)
    link(m)
    urbs.streets(m, rng)
    for i in range(BLOCKS):
        for j in range(BLOCKS):
            block(m, rng, i, j)
    hive(m, rng)
    cables(m, rng)
    urbs.railway(m, LINE_BUFFER, LINE_END, stations=(LINE_STATION,), canal=None, ends=(LINE_BUFFER,), bridges=False,
                 f=LINE, pillars_at=(HALF + LINK / 2,))
    places()
    m.build_buckets()
    for piece in m.pieces:
        piece["corner"][0] = round(piece["corner"][0] + OX, 3)

    bpy.ops.object.camera_add(location=(OX, -HALF * 2.0, HALF * 1.2), rotation=(math.radians(58), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.004, 0.0, 0.006, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else os.path.join(HERE, "story_subura.blend")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} Subura, {HALF * 2:.0f} m across at x={OX:.0f}, buildings={m.n_buildings} lifts={m.n_lifts} "
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
