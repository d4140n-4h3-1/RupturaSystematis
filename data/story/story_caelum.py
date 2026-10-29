"""
Caelum: the spaceport south of Urbs, across the void - a model of its own, apart from Urbs, the
two joined by a road. It is laid out as Urbs is, fourteen blocks by fourteen, each 48 m square
between streets 16 m wide, on a platform 912 m across. Its middle street, north and south,
carries on Urbs' middle avenue: a bridge 40 m long goes from Urbs' south edge to Caelum's north
edge, under an arch of light.

It is built where it stands in Urbs' frame, its middle 952 m south of Urbs', so that the two models
line up as they are, without moving either.

Its buildings are its own, the forms of a spaceport and the campus that grew up round it, in white
and cyan light:

- The campus, in the north by the bridge (place_campus): research labs long and low under their
  solar panels, domes for the observatories and the test halls, and its headquarters, the Spire
  (place_spire), turning as it rises, each floor a little further round than the one under it.
- The terminal (place_terminal) by the bridge, glass under a roof that rises and falls like a wave.
- The control tower (place_control_tower) in the middle, its cab flaring out 64 m up over the
  field, and mission operations round it, stepped back in terraces.
- The antenna array in the west (place_array): dishes turned to the sky, lattice masts among them.
- The hangars in the east (place_hangars): vaulted, their doors lit, spaceplanes parked before
  them; spheres of cryogenic fuel.
- The landing fields through the middle: circles ringed in light, spaceplanes parked round them.
- The launch pads along the south (place_launch_1 to place_launch_6): each a step up from the
  apron, a rocket standing on it beside its service tower, its engines aglow.
- Urbs' railway, come on over the bridge down the middle street to a station by the terminal
  (place_station_caelum), where the south train ends its line.

Every coloured line of light near the ground glows and is written to story_caelum.glow.json for the
game to light with, and where the roads and their crossings are to story_caelum.roads.json, as
Urbs' are.

Builds data/story/story_caelum.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/story/story_caelum.py -- --seed 21 --out ~/Documents/blender/story/story_caelum.blend --glb data/story/story_caelum.glb

Stairs are of 0.25 m steps, 0.35 m deep; nothing on the ground is higher than a step without
stairs. Empties named place_* mark its landmarks: place_caelum_bridge, place_campus, place_spire,
place_terminal, place_control_tower, place_array, place_hangars and place_launch_1 to 6.

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
from br_nexus import WORLD, polygon, octagon, RISE  # noqa: E402
from story_urbs import (rect, middle, shrink, block_middle, Z, PITCH, STREET, BLOCKS,  # noqa: E402
                        SIDEWALK, KERB, HALF, DEPTH)
from story_ferrum import cyl, dome, sphere, beam, V, bounds  # noqa: E402

# Where Caelum stands in Urbs' frame: the bridge from Urbs' south edge is this long, and Caelum's
# middle this far south of Urbs'.
LINK = 40.0
OY = -(HALF + LINK + HALF)                   # -952 m
ENGINE = (0.35, 0.62, 1.0)


class Caelum(urbs.Urbs):
    """Urbs' builder, its geometry moved south to where Caelum stands, with the spaceport's
    materials."""

    def __init__(self, paint):
        super().__init__(paint)
        f = self.facades
        self.city, self.offset = "c", Vector((0, OY, 0))
        f["hull"] = material("HullWhite", (0.62, 0.64, 0.68), roughness=0.35, metallic=0.2)
        f["ceramic"] = material("HeatTile", (0.025, 0.027, 0.03), roughness=0.8, metallic=0.1)
        f["solar"] = material("SolarPanel", (0.02, 0.04, 0.12), roughness=0.15, metallic=0.8)
        f["apron"] = material("Apron", (0.2, 0.2, 0.21), roughness=0.9, metallic=0.0)
        self.engine = glow("EngineBlue", ENGINE, 6.0)
        self.shade_of[id(self.engine)] = ENGINE

    def mesh(self, verts, faces, col, mat, cover="none"):
        return super().mesh([Vector((v[0], v[1] + OY, v[2])) for v in verts], faces, col, mat, cover)

    def shade(self):
        return self.paint.choice(("cyan", "cyan", "cyan", "green"))


def marker(name, x, y, z, facing):
    br_town.marker(name, x, y + OY, z, facing)


# Shapes.

def frustum_along(m, base, axis, r0, r1, length, mat, n=16, col=None, cap=True):
    """A frustum from `base` along `axis`, `r0` across there narrowing or widening to `r1`; open
    at its wide end unless `cap`."""
    a = axis.normalized()
    u = a.cross(Z) if abs(a.dot(Z)) < 0.95 else a.cross(Vector((1, 0, 0)))
    u.normalize()
    w = a.cross(u)
    top = base + a * length
    ring0 = [base + (u * math.cos(2 * math.pi * k / n) + w * math.sin(2 * math.pi * k / n)) * r0 for k in range(n)]
    ring1 = [top + (u * math.cos(2 * math.pi * k / n) + w * math.sin(2 * math.pi * k / n)) * r1 for k in range(n)]
    faces = [(k, (k + 1) % n, n + (k + 1) % n, n + k) for k in range(n)]
    if cap:
        faces += [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))]
    m.mesh(ring0 + ring1, faces, col or m.struct, mat)
    return ring1


def lattice(m, x, y, side, z0, z1, mat, shade, every=6.0, top_light=True):
    """A square lattice tower `side` across from z0 to z1: posts at its corners, girders round it
    every so high, braced across; lit up one corner, a red light on its top."""
    h = side / 2
    corners = [V(x - h, y - h), V(x + h, y - h), V(x + h, y + h), V(x - h, y + h)]
    for c in corners:
        beam(m, V(c.x, c.y, z0), V(c.x, c.y, z1), 0.35, 0.35, mat)
    z = z0 + every
    k = 0
    while z < z1 + 0.01:
        for i in range(4):
            a, b = corners[i], corners[(i + 1) % 4]
            beam(m, V(a.x, a.y, z), V(b.x, b.y, z), 0.25, 0.25, mat)
            lo, hi = (a, b) if (i + k) % 2 else (b, a)
            beam(m, V(lo.x, lo.y, z - every), V(hi.x, hi.y, z), 0.15, 0.15, m.rail)
        z += every
        k += 1
    c = corners[0]
    m.line(V(c.x - 0.2, c.y - 0.2, z0), V(c.x - 0.2, c.y - 0.2, z1), m.lines[shade], width=0.08, light=False)
    if top_light:
        m.line(V(x, y, z1), V(x, y, z1 + 1.5), m.lines["red"], width=0.5)


def vault(m, poly, wall, rise, mat, end_mat, shade):
    """A vaulted hall on `poly`: walls `wall` high, and over them a barrel vault along its longer
    side rising `rise` more, its ends closed, lit along its crown and its eaves."""
    x0, x1, y0, y1 = bounds(poly)
    along_x = (x1 - x0) >= (y1 - y0)
    m.prism(poly, 0.0, wall, mat)
    n = 10
    if along_x:
        half, mid = (y1 - y0) / 2, (y0 + y1) / 2
        pts = [(mid - half * math.cos(math.pi * k / n), wall + rise * math.sin(math.pi * k / n)) for k in range(n + 1)]
        for (a, za), (b, zb) in zip(pts, pts[1:]):
            m.mesh([V(x0, a, za), V(x1, a, za), V(x1, b, zb), V(x0, b, zb)], [(0, 1, 2, 3)], m.struct, mat)
        for x in (x0, x1):
            m.mesh([V(x, a, z) for a, z in pts], [tuple(range(n + 1))], m.struct, end_mat)
        m.line(V(x0 + 0.5, mid, wall + rise + 0.06), V(x1 - 0.5, mid, wall + rise + 0.06), m.lines[shade], width=0.12,
               light=False)
    else:
        half, mid = (x1 - x0) / 2, (x0 + x1) / 2
        pts = [(mid - half * math.cos(math.pi * k / n), wall + rise * math.sin(math.pi * k / n)) for k in range(n + 1)]
        for (a, za), (b, zb) in zip(pts, pts[1:]):
            m.mesh([V(a, y0, za), V(a, y1, za), V(b, y1, zb), V(b, y0, zb)], [(0, 1, 2, 3)], m.struct, mat)
        for y in (y0, y1):
            m.mesh([V(a, y, z) for a, z in pts], [tuple(range(n + 1))], m.struct, end_mat)
        m.line(V(mid, y0 + 0.5, wall + rise + 0.06), V(mid, y1 - 0.5, wall + rise + 0.06), m.lines[shade], width=0.12,
               light=False)
    # Lit ribs over the vault, every so far along it.
    length = (x1 - x0) if along_x else (y1 - y0)
    for k in range(1, int(length / 9.0)):
        t = (x0 if along_x else y0) + length * k / int(length / 9.0)
        for (a, za), (b, zb) in zip(pts, pts[1:]):
            if along_x:
                m.line(V(t, a, za + 0.06), V(t, b, zb + 0.06), m.lines[shade], width=0.08, light=False)
            else:
                m.line(V(a, t, za + 0.06), V(b, t, zb + 0.06), m.lines[shade], width=0.08, light=False)
    m.ring(poly, wall - 0.05, m.lines[shade])
    return along_x, pts


def spaceplane(m, x, y, heading, size=1.0):
    """A spaceplane on its gear: a lifting body, delta wings, twin fins, its engines aglow."""
    c, s = math.cos(heading), math.sin(heading)
    at = lambda u, v, z: V(x + u * c - v * s, y + u * s + v * c, z)
    L, W, gear = 16.0 * size, 2.4 * size, 1.6 * size
    hull, tile = m.facades["hull"], m.facades["ceramic"]
    beam(m, at(-L / 2, 0, gear + W / 2), at(L * 0.3, 0, gear + W / 2), W, W, hull, col=m.cover)
    # The nose, tapering to a point.
    nose = [at(L * 0.3, -W / 2, gear), at(L * 0.3, W / 2, gear), at(L * 0.3, W / 2, gear + W), at(L * 0.3, -W / 2, gear + W),
            at(L / 2, 0, gear + W * 0.35)]
    m.mesh(nose, [(0, 1, 4), (1, 2, 4), (2, 3, 4), (3, 0, 4), (3, 2, 1, 0)], m.cover, hull, cover="high")
    # The wings, flat, swept back, black underneath.
    for side in (-1, 1):
        wing = [at(L * 0.15, side * W / 2, gear + 0.4), at(-L / 2, side * W / 2, gear + 0.4),
                at(-L / 2, side * W * 2.6, gear + 0.4)]
        top = [p + V(0, 0, 0.25 * size) for p in wing]
        m.mesh(wing + top, [(0, 1, 2), (5, 4, 3), (0, 3, 4, 1), (1, 4, 5, 2), (2, 5, 3, 0)], m.cover, tile, cover="high")
        fin = [at(-L / 2, side * W * 0.45, gear + W), at(-L / 2 + 3.0 * size, side * W * 0.45, gear + W),
               at(-L / 2, side * W * 0.7, gear + W + 2.6 * size)]
        m.mesh(fin, [(0, 1, 2)], m.cover, hull)
        m.line(at(L * 0.12, side * (W / 2 + 0.03), gear + W * 0.6), at(-L / 2, side * (W / 2 + 0.03), gear + W * 0.6),
               m.lines["cyan"], width=0.06, light=False)
    for v in (-0.6 * size, 0.6 * size):
        m.line(at(-L / 2 - 0.05, v, gear + W / 2 - 0.4), at(-L / 2 - 0.05, v, gear + W / 2 + 0.4), m.engine, width=0.8,
               across=V(c, s, 0), light=False)
    for u, v in ((L * 0.3, 0), (-L * 0.3, -W), (-L * 0.3, W)):
        beam(m, at(u, v, 0.0), at(u, v, gear), 0.25, 0.25, m.rail, col=m.cover)


# Buildings. Each stands on `poly`, a rectangle within its lot.

def lab(m, rng, poly, district, street):
    """A research lab: long and low in white, its ground floor glass set back under the floors
    over it, a ribbon of windows along each floor, solar panels in rows over its roof."""
    x0, x1, y0, y1 = bounds(poly)
    floors, fh = rng.randint(2, 4), 4.0
    h = floors * fh
    mat = rng.choice((m.facades["hull"], m.facades["white"], m.facades["hull"], m.facades["glass"]))
    shade = m.shade()
    under = shrink(poly, 1.5)
    m.prism(under, 0.0, fh, m.facades["glass"])
    m.facade_windows(under, 0.3, fh - 0.3, fh, "cool", rng, lit=0.9, bands=True)
    m.prism(poly, fh, h, mat)
    m.facade_windows(poly, fh + 0.3, h - 0.3, fh, "office", rng, lit=0.8, bands=True)
    m.ring(poly, fh + 0.05, m.lines[shade])
    m.ring(poly, h - 0.05, m.lines[shade])
    for p in under:
        m.box(WORLD, p.x - 0.3, p.x + 0.3, p.y - 0.3, p.y + 0.3, 0.0, fh, m.struct, m.facades["hull"])
    along_x = (x1 - x0) >= (y1 - y0)
    k = 0
    if along_x:
        y = y0 + 2.0
        while y + 1.8 < y1 - 1.0:
            m.box(WORLD, x0 + 1.5, x1 - 1.5, y, y + 0.4, h, h + 0.6, m.struct, m.rail)
            m.box(WORLD, x0 + 1.5, x1 - 1.5, y - 0.6, y + 1.2, h + 0.6, h + 0.72, m.struct, m.facades["solar"])
            y += 3.0
            k += 1
    else:
        x = x0 + 2.0
        while x + 1.8 < x1 - 1.0:
            m.box(WORLD, x, x + 0.4, y0 + 1.5, y1 - 1.5, h, h + 0.6, m.struct, m.rail)
            m.box(WORLD, x - 0.6, x + 1.2, y0 + 1.5, y1 - 1.5, h + 0.6, h + 0.72, m.struct, m.facades["solar"])
            x += 3.0
            k += 1


def research_dome(m, rng, poly, district, street):
    """A dome on a drum, banded in light, the slit of an observatory over its crown; a lab beside
    it if there is room."""
    x0, x1, y0, y1 = bounds(poly)
    side = min(x1 - x0, y1 - y0)
    if side < 16:
        return lab(m, rng, poly, district, street)
    r = min(side / 2 - 1.5, 18.0)
    cx, cy = (x0 + r + 1.5, (y0 + y1) / 2) if (x1 - x0) > (y1 - y0) + 10 else ((x0 + x1) / 2, (y0 + y1) / 2)
    shade = m.shade()
    drum = rng.uniform(4.0, 7.0)
    cyl(m, cx, cy, r, 0.0, drum, m.facades["hull"], n=24)
    m.ring(polygon(r, 24, 0, cx, cy), drum * 0.5, m.windows["office"], out=0.05, width=0.8, light=False)
    m.ring(polygon(r, 24, 0, cx, cy), drum - 0.1, m.lines[shade], out=0.08)
    dome(m, cx, cy, r, drum, m.facades["hull"], rise=rng.uniform(0.55, 0.85), n=24, steps=5)
    for t in (0.35, 0.7):
        z = drum + r * 0.7 * math.sin(t * math.pi / 2) * 0.95
        m.ring(polygon(r * math.cos(t * math.pi / 2) + 0.05, 24, 0, cx, cy), z, m.lines[shade], out=0.0, width=0.1,
               light=False)
    if rng.random() < 0.5:
        m.box(WORLD, cx - 1.2, cx + 1.2, cy - r * 0.8, cy + r * 0.8, drum + r * 0.35, drum + r * 0.72, m.struct,
              m.facades["ceramic"])
    if cx + r + 12 < x1:
        lab(m, rng, rect(cx + r + 3, x1, y0 + 3, y1 - 3), district, street)


def hangar(m, rng, poly, district, street):
    """A hangar: a vault on low walls, its great door at one end lit round its edge and in bands
    across it, a spaceplane parked on the apron before it."""
    x0, x1, y0, y1 = bounds(poly)
    along_x = (x1 - x0) >= (y1 - y0)
    apron = 18.0 if ((x1 - x0) if along_x else (y1 - y0)) > 40 else 0.0
    shade = m.shade()
    hull = rect(x0 + apron, x1, y0, y1) if along_x else rect(x0, x1, y0 + apron, y1)
    hx0, hx1, hy0, hy1 = bounds(hull)
    width = (hy1 - hy0) if along_x else (hx1 - hx0)
    wall, rise = rng.uniform(5.0, 8.0), width * rng.uniform(0.3, 0.45)
    vault(m, hull, wall, rise, m.facades["hull"], m.facades["slate"], shade)
    # The door, on the end facing the apron.
    w = width * 0.7
    h = wall + rise * 0.7
    if along_x:
        x, mid = hx0 - 0.06, (hy0 + hy1) / 2
        for a, b in ((V(x, mid - w / 2, 0.1), V(x, mid + w / 2, 0.1)), (V(x, mid - w / 2, h), V(x, mid + w / 2, h)),
                     (V(x, mid - w / 2, 0.1), V(x, mid - w / 2, h)), (V(x, mid + w / 2, 0.1), V(x, mid + w / 2, h))):
            m.line(a, b, m.lines[shade], width=0.25)
        for k in range(1, int(h / 2.5)):
            m.line(V(x, mid - w / 2 + 0.4, k * 2.5), V(x, mid + w / 2 - 0.4, k * 2.5), m.windows["office"], width=0.3,
                   height=0.04, across=V(1, 0, 0), light=False)
        if apron:
            spaceplane(m, x0 + apron / 2, mid, 0.0, size=0.8)
    else:
        y, mid = hy0 - 0.06, (hx0 + hx1) / 2
        for a, b in ((V(mid - w / 2, y, 0.1), V(mid + w / 2, y, 0.1)), (V(mid - w / 2, y, h), V(mid + w / 2, y, h)),
                     (V(mid - w / 2, y, 0.1), V(mid - w / 2, y, h)), (V(mid + w / 2, y, 0.1), V(mid + w / 2, y, h))):
            m.line(a, b, m.lines[shade], width=0.25)
        for k in range(1, int(h / 2.5)):
            m.line(V(mid - w / 2 + 0.4, y, k * 2.5), V(mid + w / 2 - 0.4, y, k * 2.5), m.windows["office"], width=0.3,
                   height=0.04, across=V(0, 1, 0), light=False)
        if apron:
            spaceplane(m, mid, y0 + apron / 2, math.pi / 2, size=0.8)


def dishes(m, rng, poly, district, street):
    """Dish antennas on their pedestals, each turned up to its own part of the sky, rimmed in
    light; and a lattice mast among them."""
    x0, x1, y0, y1 = bounds(poly)
    R = rng.uniform(5.0, 8.0)
    step = 2 * R + 4.0
    nx, ny = max(1, int((x1 - x0) // step)), max(1, int((y1 - y0) // step))
    shade = m.shade()
    for i in range(nx):
        for j in range(ny):
            cx = x0 + (x1 - x0) * (i + 0.5) / nx
            cy = y0 + (y1 - y0) * (j + 0.5) / ny
            if (i + j) % 3 == 2 and nx * ny > 2:
                lattice(m, cx, cy, 2.4, 0.0, rng.uniform(30.0, 48.0), m.facades["hull"], shade)
                continue
            ped = R * 0.9
            cyl(m, cx, cy, 1.4, 0.0, ped, m.facades["hull"], r1=0.9, n=10)
            m.box(WORLD, cx - 1.6, cx + 1.6, cy - 0.6, cy + 0.6, ped, ped + 1.0, m.struct, m.facades["slate"])
            tilt, turn = rng.uniform(0.35, 0.9), rng.uniform(0, 2 * math.pi)
            axis = V(math.sin(tilt) * math.cos(turn), math.sin(tilt) * math.sin(turn), math.cos(tilt))
            hub = V(cx, cy, ped + 1.2)
            # The bowl: from its hub out to its rim, deepening as a parabola does.
            r_prev, d_prev = 0.6, 0.0
            for k in range(1, 4):
                r = R * k / 3
                d = R * 0.32 * (k / 3) ** 2
                rim = frustum_along(m, hub + axis * d_prev, axis, r_prev, r, d - d_prev, m.facades["white"], n=20,
                                    cap=False)
                r_prev, d_prev = r, d
            frustum_along(m, hub - axis * 0.4, axis, 0.65, 0.65, 0.4, m.facades["slate"], n=10)
            for a, b in zip(rim, rim[1:] + rim[:1]):
                m.line(a, b, m.lines[shade], width=0.1, light=False)
            # The feed at the focus, on three struts from the rim.
            focus = hub + axis * (R * 0.75)
            for k in range(0, 20, 7):
                beam(m, rim[k], focus, 0.12, 0.12, m.rail)
            frustum_along(m, focus, axis, 0.4, 0.25, 0.9, m.facades["ceramic"], n=8)


def operations(m, rng, poly, district, street):
    """Mission operations: stepped back in terraces, each edged in light, a band of its screens'
    glow round every floor, masts on its roof."""
    shade = m.shade()
    p, z = poly, 0.0
    for t in range(rng.randint(2, 4)):
        top = z + rng.uniform(6.0, 10.0)
        m.prism(p, z, top, rng.choice((m.facades["hull"], m.facades["glass"], m.facades["white"])))
        m.facade_windows(p, z + 0.4, top - 0.4, 3.5, "office", rng, lit=0.85, bands=True)
        m.ring(p, top - 0.05, m.lines[shade])
        z = top
        inner = shrink(p, 3.0)
        if min((q - middle(inner)).length for q in inner) < 5.0:
            break
        p = inner
    c = middle(p)
    m.mast(c.x, c.y, z, rng.uniform(6.0, 14.0), "red")
    m.fins(poly, poly, 0.0, z * 0.4, m.lines[shade])


def cryo(m, rng, poly, district, street):
    """Cryogenic fuel: white spheres on legs, frost-blue rings round them, the pipes between."""
    x0, x1, y0, y1 = bounds(poly)
    r = rng.uniform(4.0, 6.0)
    step = 2 * r + 4.0
    nx, ny = max(1, int((x1 - x0) // step)), max(1, int((y1 - y0) // step))
    tops = []
    for i in range(nx):
        for j in range(ny):
            cx, cy = x0 + (x1 - x0) * (i + 0.5) / nx, y0 + (y1 - y0) * (j + 0.5) / ny
            cz = r + 2.0
            for k in range(6):
                t = 2 * math.pi * k / 6
                foot = V(cx + r * 0.75 * math.cos(t), cy + r * 0.75 * math.sin(t))
                beam(m, foot, foot + V(0, 0, cz), 0.35, 0.35, m.rail)
            sphere(m, cx, cy, cz, r, m.facades["hull"])
            m.ring(polygon(r + 0.02, 14, 0, cx, cy), cz, m.lines["cyan"], out=0.05, width=0.15, light=False)
            m.ring(polygon(r * 0.5, 14, 0, cx, cy), cz + r * 0.86, m.engine, out=0.05, width=0.1, light=False)
            tops.append(V(cx, cy, 1.2))
    for a, b in zip(tops, tops[1:]):
        beam(m, a, b, 0.5, 0.5, m.facades["white"])


# The special blocks.

def rocket(m, rng, x, y, z0, height, shade):
    """A rocket standing on its hold-downs: its first stage, interstage, upper stage and fairing,
    banded black and ringed in light, three engines under it aglow."""
    hull, tile = m.facades["hull"], m.facades["ceramic"]
    r = 3.0
    z1 = z0 + height * 0.55
    z2 = z1 + height * 0.06
    z3 = z0 + height * 0.85
    cyl(m, x, y, r, z0, z1, hull)
    for z in (z0 + height * 0.12, z0 + height * 0.42):
        cyl(m, x, y, r + 0.02, z, z + 1.6, tile)
    cyl(m, x, y, r, z1, z2, tile)
    cyl(m, x, y, r, z2, z3, hull)
    cyl(m, x, y, r, z3, z3 + height * 0.04, hull, r1=r * 1.15)
    cyl(m, x, y, r * 1.15, z3 + height * 0.04, z0 + height, hull, r1=0.3)
    for z in (z1, z2, z3):
        m.ring(polygon(r, 16, 0, x, y), z, m.lines[shade], out=0.06, width=0.18)
    m.line(V(x, y, z0 + height), V(x, y, z0 + height + 1.5), m.lines["red"], width=0.3)
    # Lit up its sides, so that it stands out against the night.
    for k in range(4):
        t = math.pi / 4 + k * math.pi / 2
        u, v = math.cos(t), math.sin(t)
        m.line(V(x + (r + 0.04) * u, y + (r + 0.04) * v, z0), V(x + (r + 0.04) * u, y + (r + 0.04) * v, z3),
               m.lines[shade], width=0.12, light=False)
        m.line(V(x + r * 1.17 * u, y + r * 1.17 * v, z3 + height * 0.04), V(x + 0.35 * u, y + 0.35 * v, z0 + height),
               m.lines[shade], width=0.1, light=False)
    for k in range(3):
        t = 2 * math.pi * k / 3
        ex, ey = x + 1.4 * math.cos(t), y + 1.4 * math.sin(t)
        cyl(m, ex, ey, 1.1, z0 - 2.4, z0, tile, r1=0.5, n=10)
        m.ring(polygon(1.05, 10, 0, ex, ey), z0 - 2.3, m.engine, out=0.0, width=0.25, light=False)
    for k in range(4):
        t = math.pi / 4 + k * math.pi / 2
        foot = V(x + (r + 1.0) * math.cos(t), y + (r + 1.0) * math.sin(t))
        beam(m, V(foot.x, foot.y, KERB + RISE), V(x + r * 0.9 * math.cos(t), y + r * 0.9 * math.sin(t), z0 + 1.0),
             0.6, 0.6, m.rail)


def launch_pad(m, rng, cx, cy, inner, n):
    """A launch pad: an octagon a step up from the apron, edged in light, its flame trench; the
    rocket on it, and its service tower beside it, arms reaching across; lightning masts."""
    x0, x1, y0, y1 = inner
    pad = octagon(19.0, cx, cy)
    m.solid(pad, KERB, KERB + RISE, m.ground, m.facades["apron"])
    m.ring(pad, KERB + RISE + 0.01, m.lines["yellow"], out=0.0, width=0.15, light=False)
    top = KERB + RISE
    m.box(WORLD, cx - 2.5, cx + 2.5, cy - 18.0, cy - 3.0, top, top + 0.02, m.ground, m.facades["ceramic"])
    for s in (-1, 1):
        m.line(V(cx + s * 2.55, cy - 18.0, top + 0.02), V(cx + s * 2.55, cy - 3.0, top + 0.02), m.engine, width=0.1,
               height=0.02, light=False)
    shade = m.shade()
    height = rng.uniform(55.0, 90.0)
    rocket(m, rng, cx, cy, top + 4.0, height, shade)
    tx = cx + 9.0
    lattice(m, tx, cy, 5.0, top, top + height + 8.0, m.facades["slate"], shade)
    for z in (top + height * 0.5, top + height * 0.8, top + height * 0.95):
        beam(m, V(tx - 2.5, cy, z), V(cx + 3.2, cy, z), 1.2, 1.0, m.facades["slate"])
        m.line(V(tx - 2.5, cy - 0.62, z), V(cx + 3.4, cy - 0.62, z), m.lines[shade], width=0.06, light=False)
    for x, y in ((x0 + 2.0, y0 + 2.0), (x1 - 2.0, y1 - 2.0)):
        m.mast(x, y, KERB, rng.uniform(height * 0.8, height * 1.2), "red")
    marker(f"place_launch_{n}", cx - 21.0, cy, KERB, 0.0)


def control_tower(m, rng, cx, cy, inner):
    """The control tower: its shaft, lit up its sides and ringed every 10 m, the cab flaring out
    at 64 m, glazed all round, a mast over it; the low base it rises from."""
    hull = m.facades["hull"]
    m.box(WORLD, cx - 14, cx + 14, cy - 14, cy + 14, 0.0, 8.0, m.struct, m.facades["glass"])
    m.facade_windows(rect(cx - 14, cx + 14, cy - 14, cy + 14), 0.4, 7.6, 4.0, "cool", rng, lit=0.9, bands=True)
    m.ring(rect(cx - 14, cx + 14, cy - 14, cy + 14), 7.95, m.lines["cyan"])
    cyl(m, cx, cy, 4.5, 8.0, 62.0, hull, r1=3.6, n=16)
    for z in range(18, 62, 10):
        k = 1 - 0.2 * (z - 8) / 54
        m.ring(polygon(4.5 * k, 16, 0, cx, cy), z, m.lines["cyan"], out=0.05, width=0.15)
    for t in range(4):
        a = t * math.pi / 2
        m.line(V(cx + 4.55 * math.cos(a), cy + 4.55 * math.sin(a), 8.0), V(cx + 3.65 * math.cos(a), cy + 3.65 * math.sin(a), 62.0),
               m.lines["cyan"], width=0.1)
    cyl(m, cx, cy, 3.6, 62.0, 66.0, m.facades["ceramic"], r1=9.0, n=16)
    cyl(m, cx, cy, 9.0, 66.0, 70.5, m.facades["glass"], r1=9.6, n=16)
    m.ring(polygon(9.3, 16, 0, cx, cy), 68.2, m.windows["office"], out=0.05, width=3.0, light=False)
    cyl(m, cx, cy, 9.6, 70.5, 73.0, hull, r1=5.0, n=16)
    m.ring(polygon(9.6, 16, 0, cx, cy), 70.6, m.lines["cyan"], out=0.08, width=0.2)
    m.mast(cx, cy, 73.0, 14.0, "red")
    marker("place_control_tower", cx, cy - 16.0, KERB, -math.pi / 2)


def terminal(m, rng, cx, cy, inner):
    """The terminal: a long hall of glass under a roof that rises and falls like a wave,
    overhanging it all round, its edge lit; the departures' glow along its front."""
    x0, x1, y0, y1 = inner
    hx0, hx1, hy0, hy1 = x0 + 2, x1 - 2, cy - 11, cy + 11
    m.prism(rect(hx0, hx1, hy0, hy1), 0.0, 9.0, m.facades["glass"])
    m.facade_windows(rect(hx0, hx1, hy0, hy1), 0.5, 8.5, 4.2, "cool", rng, lit=0.95, bands=True)
    n = 12
    edge = []
    for k in range(n + 1):
        x = hx0 - 3 + (hx1 - hx0 + 6) * k / n
        edge.append((x, 11.5 + 2.2 * math.sin(k / n * 2 * math.pi)))
    for (xa, za), (xb, zb) in zip(edge, edge[1:]):
        m.mesh([V(xa, hy0 - 4, za), V(xb, hy0 - 4, zb), V(xb, hy1 + 4, zb), V(xa, hy1 + 4, za),
                V(xa, hy0 - 4, za + 0.6), V(xb, hy0 - 4, zb + 0.6), V(xb, hy1 + 4, zb + 0.6), V(xa, hy1 + 4, za + 0.6)],
               [(0, 1, 2, 3), (7, 6, 5, 4), (0, 4, 5, 1), (2, 6, 7, 3)], m.struct, m.facades["hull"])
        for y in (hy0 - 4.05, hy1 + 4.05):
            m.line(V(xa, y, za + 0.3), V(xb, y, zb + 0.3), m.lines["cyan"], width=0.15)
    for xe in (hx0 - 3, hx1 + 3):
        z = 11.5 + 2.2 * math.sin(0 if xe < cx else 2 * math.pi)
        m.mesh([V(xe, hy0 - 4, z), V(xe, hy1 + 4, z), V(xe, hy1 + 4, z + 0.6), V(xe, hy0 - 4, z + 0.6)],
               [(0, 1, 2, 3)], m.struct, m.facades["hull"])
    for x in [hx0 + (hx1 - hx0) * k / 6 for k in range(7)]:
        for y in (hy0 - 3, hy1 + 3):
            beam(m, V(x, y, 0.0), V(x, y, 11.0), 0.5, 0.5, m.facades["white"])
    m.line(V(hx0 + 2, hy0 - 0.06, 7.0), V(hx1 - 2, hy0 - 0.06, 7.0), m.lines["green"], width=0.6, height=0.05,
           across=Z, light=False)
    marker("place_terminal", cx, hy0 - 6.0, KERB, -math.pi / 2)


def spire(m, rng, cx, cy, inner):
    """The Spire: the campus' headquarters, a square tower turning as it rises, each floor 3
    degrees further round than the one under it, lit up its corners; a ring of trees round its
    plaza."""
    floors, fh, side = 32, 4.0, 24.0
    shade = "cyan"
    prev = None
    for f in range(floors):
        turn = math.radians(3.0 * f) + math.pi / 4
        poly = polygon(side / math.sqrt(2) * (1 - 0.006 * f), 4, turn, cx, cy)
        m.prism(poly, f * fh, (f + 1) * fh, m.facades["glass"] if f % 8 else m.facades["hull"])
        if f % 8:
            m.facade_windows(poly, f * fh, (f + 1) * fh, fh, "office", rng, lit=0.8, bands=True)
        else:
            m.ring(poly, (f + 1) * fh - 0.05, m.lines[shade])
        if prev is not None:
            for a, b in zip(prev, poly):
                m.line(V(a.x, a.y, f * fh), V(b.x, b.y, (f + 1) * fh), m.lines[shade], width=0.1,
                       light=f * fh < urbs.LIGHT_BELOW)
        prev = poly
    top = floors * fh
    m.ring(prev, top + 0.05, m.lines[shade], width=0.3)
    m.mast(cx, cy, top, 24.0, "red")
    x0, x1, y0, y1 = inner
    for k in range(12):
        t = 2 * math.pi * k / 12
        m.tree(cx + 19.0 * math.cos(t), cy + 19.0 * math.sin(t), rng, size=0.9)
    m.ring(octagon(16.0, cx, cy), KERB + 0.01, m.lines["green"], out=0.0, width=0.1, light=False)
    marker("place_spire", cx, cy - 20.0, KERB, -math.pi / 2)


def landing_field(m, rng, cx, cy, inner):
    """A landing field: apron across the block, a landing circle ringed in light and lettered with
    bars, beacons round it; spaceplanes parked at its sides."""
    x0, x1, y0, y1 = inner
    m.box(WORLD, x0, x1, y0, y1, KERB, KERB + 0.01, m.ground, m.facades["apron"])
    z = KERB + 0.012
    for r, shade in ((15.0, "cyan"), (11.0, "green")):
        m.ring(polygon(r, 24, 0, cx, cy), z, m.lines[shade], out=0.0, width=0.25, light=r > 12)
    for dx in (-3.0, 3.0):
        m.line(V(cx + dx, cy - 5.0, z), V(cx + dx, cy + 5.0, z), m.white, width=0.5, height=0.01, light=False)
    m.line(V(cx - 3.0, cy, z), V(cx + 3.0, cy, z), m.white, width=0.5, height=0.01, light=False)
    for k in range(8):
        t = 2 * math.pi * k / 8
        bx, by = cx + 17.5 * math.cos(t), cy + 17.5 * math.sin(t)
        m.box(WORLD, bx - 0.3, bx + 0.3, by - 0.3, by + 0.3, KERB, KERB + 0.5, m.struct, m.rail)
        m.line(V(bx, by, KERB + 0.5), V(bx, by, KERB + 0.8), m.engine, width=0.35)
    for x, y, heading in ((x0 + 9.0, y0 + 9.0, math.pi / 4), (x1 - 9.0, y1 - 9.0, -3 * math.pi / 4)):
        if rng.random() < 0.8:
            spaceplane(m, x, y, heading, size=0.7)


PADS = [(1, 1), (3, 1), (5, 1), (8, 1), (10, 1), (12, 1)]
SPECIAL = {
    (7, 6): control_tower,
    (7, 12): terminal,
    (6, 10): spire,
}


# The districts' lots, and what goes up on them.

def district_of(i, j):
    if j >= 10:
        return "campus"
    if j <= 2:
        return "pads"
    if i <= 2:
        return "array"
    if i >= 11:
        return "hangars"
    return "port"


OPTIONS = {
    "campus": [lab] * 3 + [research_dome] * 2 + [operations],
    "array": [dishes] * 4 + [lab],
    "hangars": [hangar] * 3 + [cryo],
    "pads": [hangar, cryo, cryo, hangar],
    "port": [operations, hangar, hangar, lab, cryo],
}


OFFICE_CHANCE = 0.25     # the chance a lot for one of these is an office tower to go up instead
MOST_OFFICES = 10       # and the most of them

def build_lot(m, rng, district, lot, block):
    x0, x1, y0, y1 = lot
    street = urbs.faces_on_street(lot, block)
    rng.shuffle(street)
    inset = rng.uniform(0.8, 1.8)
    poly = rect(x0 + inset, x1 - inset, y0 + inset, y1 - inset)
    small = min(x1 - x0, y1 - y0)
    build = rng.choice(OPTIONS[district])
    if small < 14 and build in (hangar, dishes, research_dome, cryo):
        build = lab if district in ("campus", "array") else operations
    if (build in (lab, operations) and street and small >= urbs.OFFICE_SIZE[0]
            and m.n_lifts < MOST_OFFICES and m.lifts.random() < OFFICE_CHANCE):
        # An office tower to go up in its place (see urbs.office_tower): the one it stands for built
        # out of sight, for its draws of chance, so that the rest of the city is as it was.
        m.unseen(lambda: build(m, rng, poly, district, street))
        urbs.office_tower(m, m.lifts, poly, street, m.facades["hull"], urbs.unpainted_shade(m))
        m.n_buildings += 1
        return
    build(m, rng, poly, district, street)
    m.n_buildings += 1


def block(m, rng, i, j):
    """A block: its pavement, its lamps, and what stands on it."""
    cx, cy = block_middle(i, j)
    h = PITCH / 2 - STREET / 2
    x0, x1, y0, y1 = cx - h, cx + h, cy - h, cy + h
    district = district_of(i, j)
    m.solid(rect(x0, x1, y0, y1), 0.0, KERB, m.ground, m.pavement if district == "campus" else m.facades["apron"])
    m.ring(rect(x0, x1, y0, y1), KERB - 0.02, m.lines["cyan"], out=0.03, width=0.06, light=False)
    for sx, sy in ((-1, -1), (1, 1)) if (i + j) % 2 else ((1, -1), (-1, 1)):
        m.pylon(WORLD, cx + sx * (h - 1.5), cy + sy * (h - 1.5), KERB, height=7.0, shade=m.shade())
    if district == "campus":
        for k in range(1, 4):
            t = -h + k * (2 * h) / 4
            for x, y in ((cx + t, y0 + 1.5), (cx + t, y1 - 1.5)):
                if rng.random() < 0.6:
                    m.tree(x, y, rng, size=0.8)
    inner = (x0 + SIDEWALK, x1 - SIDEWALK, y0 + SIDEWALK, y1 - SIDEWALK)
    special = SPECIAL.get((i, j))
    if special:
        special(m, rng, cx, cy, inner)
        return
    if (i, j) in PADS:
        launch_pad(m, rng, cx, cy, inner, PADS.index((i, j)) + 1)
        return
    if district == "port" and (i + 2 * j) % 4 == 0:
        landing_field(m, rng, cx, cy, inner)
        return
    for lot in urbs.lots(rng, "midtown" if district == "campus" else "docks", *inner):
        build_lot(m, rng, district, lot, inner)


# The ground: the platform, the edge, and the bridge from Urbs.

def ground(m):
    slab = rect(-HALF, HALF, -HALF, HALF)
    m.solid(slab, -DEPTH, -0.002, m.ground, m.road)
    m.ring(slab, -0.3, m.lines["cyan"], out=0.0, light=False)
    m.ring(slab, -DEPTH + 0.15, m.lines["cyan"], out=0.0, light=False)
    # The edge: a parapet all round, but for the bridge from Urbs in the north.
    corners = rect(-HALF, HALF, -HALF, HALF)
    for i in range(4):
        a, b = corners[i], corners[(i + 1) % 4]
        inward = (middle(corners) - (a + b) / 2).normalized() * 0.3
        gaps = []
        if i == 2:
            g = STREET / 2 / (b - a).length
            gaps.append((0.5 - g, 0.5 + g))
        m.parapet(a + inward, b + inward, 0.0, m.lines["cyan"], gaps)


def link(m):
    """The road over the void from Urbs' south edge to Caelum's middle street, and an arch of light
    over it at Caelum's edge."""
    y0, y1 = HALF - 0.5, HALF + LINK
    w = STREET / 2
    m.box(WORLD, -w, w, y0, y1, -1.2, -0.002, m.ground, m.road)
    for s in (-1, 1):
        m.line(V(s * w, y0, -0.6), V(s * w, y1, -0.6), m.lines["cyan"], light=False)
        m.parapet(V(s * (w - 0.2), y0 + 0.5, 0), V(s * (w - 0.2), y1, 0), 0.0, m.lines["cyan"])
    y = y0 + 2.0
    while y < y1 - 2.0:
        m.line(V(0.0, y, 0.004), V(0.0, y + 3.0, 0.004), m.marking, width=0.15, height=0.008, light=False)
        y += 6.0
    for y in (y1 - 4.0, (y0 + y1) / 2):
        for s in (-1, 1):
            m.pylon(WORLD, s * (w - 1.0), y, 0.0, height=7.0, shade="cyan")
    # The arch: a half ring of white segments across the road at the edge, lit along its inside.
    ay, r, n = HALF + 3.0, w + 3.0, 14
    pts = [V(-r * math.cos(math.pi * k / n), ay, r * 1.3 * math.sin(math.pi * k / n)) for k in range(n + 1)]
    for a, b in zip(pts, pts[1:]):
        beam(m, a, b, 1.4, 1.4, m.facades["hull"])
        m.line(a + V(0, -0.72, 0), b + V(0, -0.72, 0), m.lines["cyan"], width=0.2)
        m.line(a + V(0, 0.72, 0), b + V(0, 0.72, 0), m.lines["cyan"], width=0.2)
    marker("place_caelum_bridge", 0.0, HALF - 6.0, 0.0, -math.pi / 2)


def road_plan():
    """Caelum's roads and crossings, as Urbs' are, but for its canal, which Caelum has none of:
    its streets are where Urbs' are, moved south."""
    lines_at = [k * PITCH for k in range(-BLOCKS // 2 + 1, BLOCKS // 2)]
    ring = HALF - STREET / 2
    every = lines_at + [-ring, ring]
    game = lambda x0, x1, y0, y1: [round(x0, 3), round(-(y1 + OY), 3), round(x1, 3), round(-(y0 + OY), 3)]
    roads, crossings = [], []
    for c in every:
        roads.append(game(c - STREET / 2, c + STREET / 2, -HALF, HALF))
        roads.append(game(-HALF, HALF, c - STREET / 2, c + STREET / 2))
    for jx in every:
        for jy in every:
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                ox, oy = jx + dx * (STREET / 2 + 1.5), jy + dy * (STREET / 2 + 1.5)
                if dx:
                    crossings.append(game(ox - 1.5, ox + 1.5, oy - STREET / 2, oy + STREET / 2))
                else:
                    crossings.append(game(ox - STREET / 2, ox + STREET / 2, oy - 1.5, oy + 1.5))
    return {"level": urbs.road_plan()["level"], "roads": roads, "crossings": crossings}


def places():
    for name, (i, j) in (("place_campus", (4, 11)), ("place_array", (1, 6)), ("place_hangars", (12, 6))):
        cx, cy = block_middle(i, j)
        marker(name, cx - PITCH / 2 + 4.0, cy, KERB, 0.0)


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Caelum(random.Random(seed + 1))
    ground(m)
    link(m)
    streets(m, rng)
    for i in range(BLOCKS):
        for j in range(BLOCKS):
            block(m, rng, i, j)
    urbs.railway(m, 328.0, HALF + 20.0, stations=((332.0, 372.0, -1, "place_station_caelum"),), canal=None, bridges=False,
                 ends=(328.0,))
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

    out = os.path.abspath(os.path.expanduser(out)) if out else os.path.join(HERE, "story_caelum.blend")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} Caelum, {HALF * 2:.0f} m across at y={OY:.0f}, buildings={m.n_buildings} lifts={m.n_lifts} "
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


def streets(m, rng):
    """Urbs' street markings and crossings, but for its canal: Caelum has none."""
    saved = urbs.CANAL
    urbs.CANAL = (1e6, 10.0, 1.0)
    try:
        urbs.streets(m, rng)
    finally:
        urbs.CANAL = saved


if __name__ == "__main__":
    main()
