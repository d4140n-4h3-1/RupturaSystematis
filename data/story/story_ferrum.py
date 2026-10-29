"""
Ferrum: the works city east of Urbs, across the void - a model of its own, apart from Urbs, the
two joined by a road. It is laid out as Urbs is, fourteen blocks by fourteen, each 48 m square
between streets 16 m wide, on a platform 912 m across; Urbs' canal carries on across its south. Its
middle street, east and west, carries on Urbs' gate road: a bridge 40 m long goes from the far edge
of Urbs' east gate pad to Ferrum's west edge.

It is built where it stands in Urbs' frame, its middle 998 m east of Urbs', so that the two models
line up as they are, without moving either.

Its buildings are its own, the forms of heavy industry, in amber, orange and red light:

- The Crucible (place_crucible): a blast furnace, its hearth, bosh and stack banded in light, a
  bustle pipe round it, uptakes into a downcomer and dust catcher, three hot-blast stoves, a skip
  incline up to its top, and molten iron running in channels across the cast floor.
- The power station (place_power): a turbine hall, the boiler house, three chimneys up to 175 m
  ringed with red lights, a coal gallery up to the bunkers.
- The rail yard across the north (place_yard): tracks, wagons and gantry cranes over them.
- The steelworks in the middle: rolling-mill halls under a glowing monitor roof, works with
  chimneys, ore heaps and silos.
- The refinery, south-east: tank farms behind bund walls, spheres on legs, distillation columns
  ringed with platforms, flare stacks burning.
- The gasworks, north-east: gasholders, their bells up inside lattice frames.
- The workers' quarter, west, by the bridge from Urbs (place_quarter): galleried tenement slabs,
  access decks along every floor, stair towers at their ends.
- The docks south of the canal (place_docks_ferrum): quay cranes reaching out over the water.

Pipe racks cross the streets between the works, high enough to drive under. Every coloured line of
light near the ground glows and is written to story_ferrum.glow.json for the game to light with,
and where the roads and their crossings are to story_ferrum.roads.json, as Urbs' are.

Builds data/story/story_ferrum.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/story/story_ferrum.py -- --seed 13 --out ~/Documents/blender/story/story_ferrum.blend --glb data/story/story_ferrum.glb

Stairs are of 0.25 m steps, 0.35 m deep; nothing on the ground is higher than a step without
stairs. Empties named place_* mark its landmarks: place_ferrum_bridge, place_quarter,
place_crucible, place_power, place_yard, place_refinery, place_gasworks and place_docks_ferrum.

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
from br_nexus import WORLD, polygon, RISE  # noqa: E402
from story_urbs import (rect, middle, shrink, faces_on_street, block_middle, Z, PITCH, STREET, BLOCKS,  # noqa: E402
                        SIDEWALK, KERB, HALF, DEPTH, CANAL, BRIDGE_WIDTH, GATE)

# Where Ferrum stands in Urbs' frame: Urbs' east gate pad ends this far east of Urbs' middle, and
# the bridge from it this long.
URBS_PAD = HALF + GATE[0] + GATE[2]          # 502 m
LINK = 40.0
OX = URBS_PAD + LINK + HALF                  # Ferrum's middle: 998 m east of Urbs'
MOLTEN = (1.0, 0.32, 0.04)
RACK = 7.5                                   # how high the pipe racks cross the streets


class Ferrum(urbs.Urbs):
    """Urbs' builder, its geometry moved east to where Ferrum stands, with the works' materials."""

    def __init__(self, paint):
        super().__init__(paint)
        f = self.facades
        self.city, self.offset = "f", Vector((OX, 0, 0))
        f["steel"] = material("Steel", (0.08, 0.085, 0.09), roughness=0.45, metallic=0.8)
        f["soot"] = material("Soot", (0.035, 0.033, 0.032), roughness=0.9, metallic=0.1)
        f["oxide"] = material("Oxide", (0.26, 0.07, 0.035), roughness=0.7, metallic=0.4)
        f["tank"] = material("TankWhite", (0.45, 0.45, 0.42), roughness=0.5, metallic=0.3)
        f["green"] = material("WorksGreen", (0.05, 0.12, 0.09), roughness=0.6, metallic=0.4)
        self.coal = material("Coal", (0.02, 0.02, 0.022), roughness=0.95, metallic=0.0)
        self.ore = material("Ore", (0.17, 0.07, 0.035), roughness=0.95, metallic=0.0)
        self.rails = material("Rails", (0.3, 0.3, 0.32), roughness=0.3, metallic=0.9)
        self.molten = glow("Molten", MOLTEN, 8.0)
        self.shade_of[id(self.molten)] = MOLTEN

    def mesh(self, verts, faces, col, mat, cover="none"):
        return super().mesh([Vector((v[0] + OX, v[1], v[2])) for v in verts], faces, col, mat, cover)

    def shade(self):
        return self.paint.choice(("orange", "orange", "yellow", "red"))


def marker(name, x, y, z, facing):
    br_town.marker(name, x + OX, y, z, facing)


# Shapes.

def cyl(m, cx, cy, r, z0, z1, mat, r1=None, n=16, col=None):
    """An upright cylinder, or a cone's frustum narrowing to `r1`."""
    m.prism(polygon(r, n, 0, cx, cy), z0, z1, mat, top=polygon(r if r1 is None else r1, n, 0, cx, cy), col=col)


def dome(m, cx, cy, r, z, mat, rise=0.6, n=16, steps=4):
    for k in range(steps):
        a0, a1 = k / steps * math.pi / 2, (k + 1) / steps * math.pi / 2
        r0, r1 = r * math.cos(a0), max(r * math.cos(a1), r * 0.08)
        cyl(m, cx, cy, r0, z + r * rise * math.sin(a0), z + r * rise * math.sin(a1), mat, r1=r1, n=n)


def sphere(m, cx, cy, cz, r, mat, n=14, bands=8):
    for k in range(bands):
        a0, a1 = -math.pi / 2 + k / bands * math.pi, -math.pi / 2 + (k + 1) / bands * math.pi
        cyl(m, cx, cy, max(r * math.cos(a0), r * 0.06), cz + r * math.sin(a0), cz + r * math.sin(a1), mat,
            r1=max(r * math.cos(a1), r * 0.06), n=n)


def beam(m, p, q, w, h, mat, col=None):
    """A solid bar from `p` to `q`, `w` across and `h` deep."""
    d = q - p
    if d.length < 1e-4:
        return
    dn = d.normalized()
    a = dn.cross(Z)
    if a.length < 1e-4:
        a = Vector((1, 0, 0))
    a.normalize()
    b = dn.cross(a)
    ha, hb = a * (w / 2), b * (h / 2)
    verts = [e + sa * ha + sb * hb for e in (p, q) for sa, sb in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
    m.mesh(verts, [(0, 1, 2, 3), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)],
           col or m.struct, mat)


def V(x, y, z=0.0):
    return Vector((x, y, z))


def gable(m, x0, x1, y0, y1, eave, ridge, mat, roof):
    """A hall from the ground to `eave`, its roof rising to `ridge` along its longer side."""
    m.prism(rect(x0, x1, y0, y1), 0.0, eave, mat)
    if (x1 - x0) >= (y1 - y0):
        ym = (y0 + y1) / 2
        verts = [V(x0, y0, eave), V(x1, y0, eave), V(x1, ym, ridge), V(x0, ym, ridge), V(x1, y1, eave), V(x0, y1, eave)]
    else:
        xm = (x0 + x1) / 2
        verts = [V(x0, y0, eave), V(x0, y1, eave), V(xm, y1, ridge), V(xm, y0, ridge), V(x1, y1, eave), V(x1, y0, eave)]
    m.mesh(verts, [(0, 1, 2, 3), (3, 2, 4, 5), (0, 3, 5), (1, 4, 2), (0, 5, 4, 1)], m.struct, roof)


def chimney(m, cx, cy, r, h, mat, every=25.0):
    """A tapering stack, ringed with red lights every so high, an orange band round its top."""
    cyl(m, cx, cy, r, 0.0, h, mat, r1=r * 0.65, n=12)
    z = every
    while z < h - 2:
        k = 1 - 0.35 * z / h
        m.ring(polygon(r * k, 12, 0, cx, cy), z, m.lines["red"], out=0.05, width=0.25)
        z += every
    m.ring(polygon(r * 0.66, 12, 0, cx, cy), h - 1.0, m.lines["orange"], out=0.05, width=0.5)


def gallery(m, p, q, mat, w=2.4, h=2.2, shade="yellow"):
    """A conveyor gallery from `p` up to `q` on trestles, lit along its side."""
    beam(m, p + V(0, 0, h / 2), q + V(0, 0, h / 2), w, h, mat)
    d = q - p
    n = max(1, int(d.length / 12))
    for k in range(1, n + 1):
        t = k / (n + 1)
        top = p + d * t
        if top.z > 3:
            beam(m, V(top.x, top.y, 0.0), top, 0.5, 0.5, m.rail)
    side = d.normalized().cross(Z).normalized() * (w / 2 + 0.03)
    m.line(p + side + V(0, 0, h * 0.5), q + side + V(0, 0, h * 0.5), m.lines[shade], width=0.08)


def rack(m, p, q, z, shade):
    """A pipe rack from `p` to `q` on posts, three pipes along it at `z`, one of them lit."""
    d = q - p
    across = d.normalized().cross(Z).normalized()
    n = max(1, round(d.length / 8))
    for k in range(n + 1):
        c = p + d * (k / n)
        for s in (-1, 1):
            beam(m, c + across * (s * 1.6) , c + across * (s * 1.6) + V(0, 0, z + 1.4), 0.35, 0.35, m.rail)
        beam(m, c + across * -1.8 + V(0, 0, z - 0.2), c + across * 1.8 + V(0, 0, z - 0.2), 0.3, 0.35, m.rail)
        beam(m, c + across * -1.8 + V(0, 0, z + 1.2), c + across * 1.8 + V(0, 0, z + 1.2), 0.3, 0.35, m.rail)
    for k, (off, size, mat) in enumerate(((-1.0, 0.7, m.facades["steel"]), (0.0, 0.9, m.facades["tank"]),
                                          (1.0, 0.55, m.facades["oxide"]))):
        a, b = p + across * off + V(0, 0, z + size / 2), q + across * off + V(0, 0, z + size / 2)
        beam(m, a, b, size, size, mat)
    m.line(p + across * 1.45 + V(0, 0, z + 0.3), q + across * 1.45 + V(0, 0, z + 0.3), m.lines[shade],
           width=0.07, light=False)


# Buildings. Each stands on `poly`, a rectangle within its lot.

def bounds(poly):
    return poly[0].x, poly[2].x, poly[0].y, poly[2].y


def steel_hall(m, rng, poly, district, street):
    """A rolling-mill hall: long, pilastered, a glowing monitor along its ridge, its doors aglow
    with what is worked inside."""
    x0, x1, y0, y1 = bounds(poly)
    eave = rng.uniform(13.0, 21.0)
    ridge = eave + rng.uniform(3.0, 5.0)
    shade = rng.choice(("orange", "yellow"))
    mat = rng.choice((m.facades["steel"], m.facades["green"], m.facades["oxide"]))
    gable(m, x0, x1, y0, y1, eave, ridge, mat, m.facades["soot"])
    along_x = (x1 - x0) >= (y1 - y0)
    # The monitor: a lit lantern along the ridge.
    if along_x:
        ym = (y0 + y1) / 2
        m.box(WORLD, x0 + 2, x1 - 2, ym - 1.6, ym + 1.6, ridge - 0.6, ridge + 1.6, m.struct, m.facades["soot"])
        for s in (-1, 1):
            m.line(V(x0 + 2.5, ym + s * 1.62, ridge + 0.5), V(x1 - 2.5, ym + s * 1.62, ridge + 0.5),
                   m.windows["warm"], width=1.4, height=0.05, across=Z, light=False)
    else:
        xm = (x0 + x1) / 2
        m.box(WORLD, xm - 1.6, xm + 1.6, y0 + 2, y1 - 2, ridge - 0.6, ridge + 1.6, m.struct, m.facades["soot"])
        for s in (-1, 1):
            m.line(V(xm + s * 1.62, y0 + 2.5, ridge + 0.5), V(xm + s * 1.62, y1 - 2.5, ridge + 0.5),
                   m.windows["warm"], width=1.4, height=0.05, across=Z, light=False)
    # Pilasters down the long sides.
    length = (x1 - x0) if along_x else (y1 - y0)
    for k in range(int(length / 6) + 1):
        t = min(k * 6.0, length)
        if along_x:
            for y, s in ((y0, -1), (y1, 1)):
                m.box(WORLD, x0 + t - 0.3, x0 + t + 0.3, y, y + s * 0.6, 0.0, eave, m.struct, m.facades["soot"])
        else:
            for x, s in ((x0, -1), (x1, 1)):
                m.box(WORLD, x, x + s * 0.6, y0 + t - 0.3, y0 + t + 0.3, 0.0, eave, m.struct, m.facades["soot"])
    m.ring(poly, eave - 0.1, m.lines[shade])
    m.fins(poly, poly, 0.0, eave, m.lines[shade])
    # The great doors at its ends, glowing molten.
    ends = [1, 3] if along_x else [0, 2]
    for face in ends:
        door_w, door_h = min(8.0, (y1 - y0 if along_x else x1 - x0) * 0.4), min(9.0, eave * 0.6)
        if face == 1:
            p, q = V(x1 + 0.05, (y0 + y1) / 2 - door_w / 2, 0.02), V(x1 + 0.05, (y0 + y1) / 2 + door_w / 2, 0.02)
        elif face == 3:
            p, q = V(x0 - 0.05, (y0 + y1) / 2 - door_w / 2, 0.02), V(x0 - 0.05, (y0 + y1) / 2 + door_w / 2, 0.02)
        elif face == 0:
            p, q = V((x0 + x1) / 2 - door_w / 2, y0 - 0.05, 0.02), V((x0 + x1) / 2 + door_w / 2, y0 - 0.05, 0.02)
        else:
            p, q = V((x0 + x1) / 2 - door_w / 2, y1 + 0.05, 0.02), V((x0 + x1) / 2 + door_w / 2, y1 + 0.05, 0.02)
        for k in range(int(door_h / 1.5)):
            z = 0.75 + k * 1.5
            m.line(p + V(0, 0, z), q + V(0, 0, z), m.molten if k % 2 == 0 else m.lines["orange"],
                   width=1.2, height=0.05, across=Z, light=k == 0)


def works(m, rng, poly, district, street):
    """Works: a shed under barrel vaults, a chimney at its corner, loading bays lit along it."""
    x0, x1, y0, y1 = bounds(poly)
    h = rng.uniform(8.0, 12.0)
    mat = rng.choice((m.facades["brick"], m.facades["oxide"], m.facades["concrete"]))
    shade = m.shade()
    m.prism(poly, 0.0, h, mat)
    along_x = (x1 - x0) >= (y1 - y0)
    span = 7.0
    n = max(1, int(((y1 - y0) if along_x else (x1 - x0)) / span))
    for k in range(n):
        if along_x:
            a, b = y0 + (y1 - y0) * k / n, y0 + (y1 - y0) * (k + 1) / n
        else:
            a, b = x0 + (x1 - x0) * k / n, x0 + (x1 - x0) * (k + 1) / n
        half, mid = (b - a) / 2, (a + b) / 2
        pts = [(mid + half * math.cos(t), h + half * 0.5 * math.sin(t)) for t in
               (math.pi * i / 6 for i in range(7))]
        for (u0, z0), (u1, z1_) in zip(pts, pts[1:]):
            if along_x:
                verts = [V(x0, u0, z0), V(x1, u0, z0), V(x1, u1, z1_), V(x0, u1, z1_)]
            else:
                verts = [V(u0, y0, z0), V(u0, y1, z0), V(u1, y1, z1_), V(u1, y0, z1_)]
            m.mesh(verts, [(0, 1, 2, 3)], m.struct, m.facades["slate"])
        if along_x:
            m.line(V(x0 + 1, mid, h + half * 0.5 + 0.05), V(x1 - 1, mid, h + half * 0.5 + 0.05), m.lines[shade],
                   width=0.1, light=False)
        else:
            m.line(V(mid, y0 + 1, h + half * 0.5 + 0.05), V(mid, y1 - 1, h + half * 0.5 + 0.05), m.lines[shade],
                   width=0.1, light=False)
    m.ring(poly, 0.5, m.lines[shade])
    m.fins(poly, poly, 0.0, h, m.lines[shade])
    m.facade_windows(poly, 2.5, h - 1.0, h, "warm", rng, lit=0.5, bands=True)
    if rng.random() < 0.6:
        cx, cy = (x0 + 2.5, y0 + 2.5) if rng.random() < 0.5 else (x1 - 2.5, y1 - 2.5)
        chimney(m, cx, cy, 1.8, rng.uniform(28.0, 48.0), m.facades["brick"], every=14.0)


def tank_farm(m, rng, poly, district, street):
    """Tanks behind a bund wall a step high, their stairs and rims lit; or spheres on legs."""
    x0, x1, y0, y1 = bounds(poly)
    for a, b in ((V(x0, y0), V(x1, y0)), (V(x1, y0), V(x1, y1)), (V(x1, y1), V(x0, y1)), (V(x0, y1), V(x0, y0))):
        beam(m, a + V(0, 0, RISE / 2), b + V(0, 0, RISE / 2), 0.4, RISE, m.facades["concrete"])
    spheres = rng.random() < 0.35
    r = rng.uniform(4.0, 7.5) if not spheres else rng.uniform(4.0, 5.5)
    step = 2 * r + 3.0
    nx, ny = max(1, int((x1 - x0 - 2) // step)), max(1, int((y1 - y0 - 2) // step))
    shade = rng.choice(("orange", "yellow", "red"))
    for i in range(nx):
        for j in range(ny):
            cx = x0 + (x1 - x0) * (i + 0.5) / nx
            cy = y0 + (y1 - y0) * (j + 0.5) / ny
            rr = min(r, (x1 - x0) / nx / 2 - 1.5, (y1 - y0) / ny / 2 - 1.5)
            if rr < 2.0:
                continue
            if spheres:
                cz = rr + 2.5
                for k in range(6):
                    t = 2 * math.pi * k / 6
                    foot = V(cx + rr * 0.8 * math.cos(t), cy + rr * 0.8 * math.sin(t))
                    beam(m, foot, foot + V(0, 0, cz), 0.4, 0.4, m.rail)
                sphere(m, cx, cy, cz, rr, m.facades["tank"])
                m.ring(polygon(rr, 14, 0, cx, cy), cz, m.lines[shade], out=0.05, light=False)
            else:
                h = rng.uniform(8.0, 15.0)
                cyl(m, cx, cy, rr, 0.0, h, m.facades["tank"])
                cyl(m, cx, cy, rr, h, h + rr * 0.15, m.facades["tank"], r1=rr * 0.15)
                m.ring(polygon(rr, 16, 0, cx, cy), h - 0.1, m.lines[shade], out=0.08)
                # The stair round its side, as a lit line climbing it.
                pts = [V(cx + (rr + 0.1) * math.cos(t), cy + (rr + 0.1) * math.sin(t), h * t / (1.5 * math.pi))
                       for t in (1.5 * math.pi * k / 10 for k in range(11))]
                for a, b in zip(pts, pts[1:]):
                    m.line(a, b, m.lines["yellow"], width=0.06, light=False)


def refinery(m, rng, poly, district, street):
    """Distillation columns ringed with platforms, a furnace and its stack, pipes, and a flare."""
    x0, x1, y0, y1 = bounds(poly)
    along_x = (x1 - x0) >= (y1 - y0)
    n = rng.randint(3, 6)
    shade = rng.choice(("orange", "yellow"))
    for k in range(n):
        t = (k + 0.5) / n
        cx = x0 + (x1 - x0) * t if along_x else x0 + (x1 - x0) * 0.35
        cy = y0 + (y1 - y0) * 0.35 if along_x else y0 + (y1 - y0) * t
        r = rng.uniform(1.2, 2.6)
        h = rng.uniform(24.0, 58.0)
        cyl(m, cx, cy, r, 0.0, h, m.facades["tank"], n=12)
        dome(m, cx, cy, r, h, m.facades["tank"], n=12, steps=3)
        z = 6.0
        while z < h - 2:
            cyl(m, cx, cy, r + 0.9, z, z + 0.15, m.rail, n=12)
            m.ring(polygon(r + 0.9, 12, 0, cx, cy), z + 1.1, m.lines[shade], out=0.0, width=0.05)
            z += rng.choice((6.0, 8.0))
    # A furnace and its stack, and the rack along the other side.
    fx, fy = (x0 + (x1 - x0) * 0.25, y0 + (y1 - y0) * 0.78) if along_x else (x0 + (x1 - x0) * 0.78, y0 + (y1 - y0) * 0.25)
    m.box(WORLD, fx - 4, fx + 4, fy - 3, fy + 3, 0.0, 10.0, m.struct, m.facades["oxide"])
    m.line(V(fx - 4.05, fy - 2, 1.2), V(fx - 4.05, fy + 2, 1.2), m.molten, width=0.5)
    chimney(m, fx, fy, 1.2, rng.uniform(30.0, 40.0), m.facades["steel"], every=12.0)
    if along_x:
        rack(m, V(x0 + 2, y0 + (y1 - y0) * 0.62), V(x1 - 2, y0 + (y1 - y0) * 0.62), 5.0, shade)
    else:
        rack(m, V(x0 + (x1 - x0) * 0.62, y0 + 2), V(x0 + (x1 - x0) * 0.62, y1 - 2), 5.0, shade)
    if rng.random() < 0.6:
        flare(m, x1 - 2.5, y1 - 2.5, rng.uniform(50.0, 75.0))


def flare(m, x, y, h):
    """A flare stack, guyed, burning at its tip."""
    cyl(m, x, y, 0.7, 0.0, h, m.facades["steel"], r1=0.5, n=8)
    for k in range(3):
        t = 2 * math.pi * k / 3
        beam(m, V(x + 0.6 * math.cos(t), y + 0.6 * math.sin(t), h * 0.7), V(x + 2.2 * math.cos(t), y + 2.2 * math.sin(t)),
             0.12, 0.12, m.rail)
    m.line(V(x, y, h), V(x, y, h + 3.5), m.molten, width=1.4)
    m.line(V(x, y, h + 3.5), V(x, y, h + 5.5), m.lines["orange"], width=0.8)
    m.ring(polygon(0.55, 8, 0, x, y), h - 0.5, m.lines["red"], out=0.05, width=0.2)


def gasholder(m, rng, poly, district, street):
    """A gasholder: its bell up inside a lattice frame of columns and girders."""
    x0, x1, y0, y1 = bounds(poly)
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    r = min(x1 - x0, y1 - y0) / 2 - 1.0
    if r < 8:
        return works(m, rng, poly, district, street)
    r = min(r, 21.0)
    h = rng.uniform(28.0, 42.0)
    n = 12
    pts = polygon(r, n, 0, cx, cy)
    for p in pts:
        beam(m, V(p.x, p.y, 0), V(p.x, p.y, h), 0.7, 0.7, m.facades["oxide"])
    for k in range(1, 5):
        z = h * k / 4
        for a, b in zip(pts, pts[1:] + pts[:1]):
            beam(m, V(a.x, a.y, z), V(b.x, b.y, z), 0.5, 0.6, m.facades["oxide"])
        if k < 4:
            for i, (a, b) in enumerate(zip(pts, pts[1:] + pts[:1])):
                lo, hi = (a, b) if i % 2 else (b, a)
                beam(m, V(lo.x, lo.y, z - h / 4), V(hi.x, hi.y, z), 0.25, 0.25, m.rail)
    bell = h * rng.uniform(0.45, 0.95)
    cyl(m, cx, cy, r - 1.3, 0.0, bell, m.facades["green"], n=24)
    dome(m, cx, cy, r - 1.3, bell, m.facades["green"], rise=0.12, n=24, steps=3)
    m.ring(polygon(r - 1.3, 24, 0, cx, cy), bell - 0.3, m.lines["orange"], out=0.05, width=0.2, light=False)
    m.ring(pts, h, m.lines["red"], out=0.3, width=0.2)
    m.ring(polygon(r - 1.3, 24, 0, cx, cy), 0.6, m.lines["orange"], out=0.05)


def silos(m, rng, poly, district, street):
    """Silos in a cluster under a headhouse, an elevator tower at its end, a gallery down to the
    ground."""
    x0, x1, y0, y1 = bounds(poly)
    r = rng.uniform(3.0, 4.0)
    step = 2 * r + 0.4
    nx, ny = max(1, min(4, int((x1 - x0) // step))), max(1, min(3, int((y1 - y0) // step)))
    h = rng.uniform(26.0, 38.0)
    mat = rng.choice((m.facades["concrete"], m.facades["tank"]))
    sx0, sy0 = (x0 + x1) / 2 - nx * step / 2, (y0 + y1) / 2 - ny * step / 2
    for i in range(nx):
        for j in range(ny):
            cyl(m, sx0 + step * (i + 0.5), sy0 + step * (j + 0.5), r, 0.0, h, mat)
    hx0, hx1, hy0, hy1 = sx0, sx0 + nx * step, sy0, sy0 + ny * step
    m.box(WORLD, hx0, hx1, hy0, hy1, h, h + 4.0, m.struct, m.facades["steel"])
    m.box(WORLD, hx1 - 4, hx1, hy0, hy0 + 4, 0.0, h + 12.0, m.struct, m.facades["steel"])
    m.ring(rect(hx0, hx1, hy0, hy1), h + 2.0, m.lines["yellow"], width=0.15)
    m.ring(rect(hx1 - 4, hx1, hy0, hy0 + 4), h + 11.5, m.lines["red"], width=0.2)
    m.facade_windows(rect(hx0, hx1, hy0, hy1), h + 0.5, h + 3.5, 3.0, "office", rng, lit=0.7, bands=True)
    end = V(min(x1 - 1.0, hx1 + 9.0), hy0 + 2.0, 0.0) if x1 - hx1 > 6 else V(hx0 + 2.0, max(y0 + 1.0, hy0 - 9.0), 0.0)
    gallery(m, end, V(hx1 - 2, hy0 + 2, h), m.facades["steel"])


def heaps(m, rng, poly, district, street):
    """Heaps of ore and coal, a stacker's gantry over them and a conveyor along it."""
    x0, x1, y0, y1 = bounds(poly)
    along_x = (x1 - x0) >= (y1 - y0)
    length = (x1 - x0) if along_x else (y1 - y0)
    n = max(1, int(length / 18))
    width = (y1 - y0) if along_x else (x1 - x0)
    for k in range(n):
        t = (k + 0.5) / n
        cx = x0 + (x1 - x0) * t if along_x else (x0 + x1) / 2
        cy = (y0 + y1) / 2 if along_x else y0 + (y1 - y0) * t
        r = min(length / n / 2 - 0.5, width / 2 - 1.0)
        if r < 3:
            continue
        mat = rng.choice((m.coal, m.ore))
        cyl(m, cx, cy, r, 0.0, r * rng.uniform(0.5, 0.75), mat, r1=r * 0.12, n=10)
    gz = width * 0.45 + 3.0
    if along_x:
        ym = (y0 + y1) / 2
        for x in (x0 + 1, x1 - 1):
            for s in (-1, 1):
                beam(m, V(x, ym + s * (width / 2 - 0.5)), V(x, ym + s * 1.0, gz), 0.5, 0.5, m.facades["oxide"])
        rack(m, V(x0 + 1, ym), V(x1 - 1, ym), gz, "yellow")
    else:
        xm = (x0 + x1) / 2
        for y in (y0 + 1, y1 - 1):
            for s in (-1, 1):
                beam(m, V(xm + s * (width / 2 - 0.5), y), V(xm + s * 1.0, y, gz), 0.5, 0.5, m.facades["oxide"])
        rack(m, V(xm, y0 + 1), V(xm, y1 - 1), gz, "yellow")


def tenement(m, rng, poly, district, street):
    """Workers' tenements: slabs along the lot, an access deck along every floor on the street
    side, lit along its railing, and a stair tower at each end."""
    x0, x1, y0, y1 = bounds(poly)
    floors = rng.randint(4, 7)
    fh = 3.2
    h = floors * fh
    mat = rng.choice((m.facades["brick"], m.facades["concrete"], m.facades["sand"]))
    shade = rng.choice(("orange", "yellow"))
    face = street[0] if street else 0
    along_x = face in (0, 2)
    depth = 9.0
    slabs = []
    if along_x:
        span = y1 - y0
        k = max(1, int((span + 6) // (depth + 2 + 8)))
        for i in range(k):
            a = y0 + i * (span / k)
            slabs.append((x0 + 4.5, x1 - 4.5, a + 2.0, a + 2.0 + depth, -1 if face == 0 else 1))
    else:
        span = x1 - x0
        k = max(1, int((span + 6) // (depth + 2 + 8)))
        for i in range(k):
            a = x0 + i * (span / k)
            slabs.append((a + 2.0, a + 2.0 + depth, y0 + 4.5, y1 - 4.5, -1 if face == 3 else 1))
    for sx0, sx1, sy0, sy1, way in slabs:
        if sx1 - sx0 < 8 or sy1 - sy0 < 8:
            continue
        body = rect(sx0, sx1, sy0, sy1)
        m.prism(body, 0.0, h, mat)
        m.facade_windows(body, 0.5, h, fh, "warm", rng, lit=0.55)
        m.ring(body, h - 0.05, m.lines[shade])
        # The decks, on the side the slab faces, and the stair towers.
        for f in range(1, floors):
            z = f * fh
            if along_x:
                y = sy0 if way < 0 else sy1
                m.box(WORLD, sx0, sx1, y, y + way * 1.6, z - 0.2, z, m.struct, m.facades["concrete"])
                m.line(V(sx0, y + way * 1.6, z + 1.0), V(sx1, y + way * 1.6, z + 1.0), m.lines[shade], width=0.06,
                       light=False)
            else:
                x = sx0 if way < 0 else sx1
                m.box(WORLD, x, x + way * 1.6, sy0, sy1, z - 0.2, z, m.struct, m.facades["concrete"])
                m.line(V(x + way * 1.6, sy0, z + 1.0), V(x + way * 1.6, sy1, z + 1.0), m.lines[shade], width=0.06,
                       light=False)
        towers = ([(sx0 - 4.0, sx0, sy0, sy1), (sx1, sx1 + 4.0, sy0, sy1)] if along_x
                  else [(sx0, sx1, sy0 - 4.0, sy0), (sx0, sx1, sy1, sy1 + 4.0)])
        for tx0, tx1, ty0, ty1 in towers:
            t = rect(tx0, tx1, ty0, ty1)
            m.prism(t, 0.0, h + 2.5, m.facades["concrete"])
            m.fins(t, t, 0.0, h + 2.5, m.lines[shade])
            m.facade_windows(t, 1.5, h + 2.0, fh, "cool", rng, lit=0.8, bands=True)


def gantry(m, x0, x1, y, h, span_axis_x, shade):
    """A gantry crane: legs at either end of its girder, a trolley on it."""
    if span_axis_x:
        for x in (x0, x1):
            for dy in (-3.0, 3.0):
                beam(m, V(x, y + dy), V(x, y + dy * 0.3, h), 0.6, 0.6, m.facades["oxide"])
        beam(m, V(x0 - 1, y, h + 0.6), V(x1 + 1, y, h + 0.6), 1.6, 1.4, m.facades["oxide"])
        tx = x0 + (x1 - x0) * 0.4
        m.box(WORLD, tx - 1.5, tx + 1.5, y - 1.2, y + 1.2, h + 1.3, h + 3.0, m.struct, m.facades["steel"])
        m.line(V(x0, y - 0.82, h + 0.6), V(x1, y - 0.82, h + 0.6), m.lines[shade], width=0.08, light=False)
        m.line(V(x0, y + 0.82, h + 0.6), V(x1, y + 0.82, h + 0.6), m.lines[shade], width=0.08, light=False)
    else:
        for yy in (x0, x1):
            for dx in (-3.0, 3.0):
                beam(m, V(y + dx, yy), V(y + dx * 0.3, yy, h), 0.6, 0.6, m.facades["oxide"])
        beam(m, V(y, x0 - 1, h + 0.6), V(y, x1 + 1, h + 0.6), 1.6, 1.4, m.facades["oxide"])


# The special blocks.

def crucible(m, rng, cx, cy, inner):
    """The blast furnace."""
    fx, fy = cx - 3.0, cy - 3.0
    rust, soot, steel = m.facades["oxide"], m.facades["soot"], m.facades["steel"]
    cyl(m, fx, fy, 7.0, 0.0, 10.0, soot, n=20)
    cyl(m, fx, fy, 7.0, 10.0, 16.0, rust, r1=8.5, n=20)
    cyl(m, fx, fy, 8.5, 16.0, 42.0, rust, r1=5.5, n=20)
    cyl(m, fx, fy, 5.5, 42.0, 46.0, soot, n=20)
    m.box(WORLD, fx - 7, fx + 7, fy - 7, fy + 7, 46.0, 47.0, m.struct, steel)
    cyl(m, fx, fy, 3.0, 47.0, 55.0, steel, r1=2.2, n=12)
    for z, r in ((12.0, 7.6), (20.0, 8.25), (28.0, 7.4), (36.0, 6.5)):
        m.ring(polygon(r, 20, 0, fx, fy), z, m.lines["orange"], out=0.1, width=0.3)
    m.ring(rect(fx - 7, fx + 7, fy - 7, fy + 7), 47.1, m.lines["red"], out=0.05, width=0.2)
    # The bustle pipe and its tuyeres.
    ring_pts = polygon(10.5, 12, 0, fx, fy)
    for a, b in zip(ring_pts, ring_pts[1:] + ring_pts[:1]):
        beam(m, V(a.x, a.y, 15.0), V(b.x, b.y, 15.0), 1.4, 1.4, rust)
        c = (a + b) / 2
        inward = (V(fx, fy) - V(c.x, c.y)).normalized()
        beam(m, V(c.x, c.y, 14.4), V(c.x, c.y) + inward * 3.2 + V(0, 0, 10.5), 0.4, 0.4, steel)
    # Uptakes into the downcomer, down to the dust catcher.
    for dx, dy in ((-3, -3), (3, -3), (3, 3), (-3, 3)):
        beam(m, V(fx + dx, fy + dy, 47.0), V(fx + dx * 0.6, fy + dy * 0.6, 62.0), 1.2, 1.2, rust)
        beam(m, V(fx + dx * 0.6, fy + dy * 0.6, 62.0), V(fx, fy, 66.0), 1.2, 1.2, rust)
    dcx, dcy = cx + 14.0, cy - 13.0
    for k in range(4):
        t = math.pi / 4 + k * math.pi / 2
        beam(m, V(dcx + 3.2 * math.cos(t), dcy + 3.2 * math.sin(t)), V(dcx + 2.6 * math.cos(t), dcy + 2.6 * math.sin(t), 9.0),
             0.6, 0.6, m.rail)
    cyl(m, dcx, dcy, 1.0, 4.0, 9.0, steel, r1=3.5)
    cyl(m, dcx, dcy, 3.5, 9.0, 24.0, rust)
    cyl(m, dcx, dcy, 3.5, 24.0, 28.0, rust, r1=1.0)
    beam(m, V(fx, fy, 66.0), V(dcx, dcy, 28.0), 1.8, 1.8, rust)
    m.ring(polygon(3.5, 16, 0, dcx, dcy), 23.5, m.lines["orange"], out=0.05, width=0.2)
    # The hot-blast stoves, and the main from them to the bustle pipe.
    for k in range(3):
        sx, sy = cx - 14.0 + k * 9.5, cy + 15.0
        cyl(m, sx, sy, 4.0, 0.0, 34.0, steel, n=16)
        dome(m, sx, sy, 4.0, 34.0, steel, rise=0.8)
        m.ring(polygon(4.0, 16, 0, sx, sy), 33.0, m.lines["orange"], out=0.05, width=0.2)
        m.ring(polygon(4.0, 16, 0, sx, sy), 1.0, m.lines["orange"], out=0.05)
        beam(m, V(sx, sy - 4.0, 15.0), V(sx, fy + 10.5, 15.0), 1.2, 1.2, rust)
    # The skip incline from the stock bunker up to the furnace's top.
    m.box(WORLD, cx + 10, cx + 19, cy + 6, cy + 13, 0.0, 6.0, m.struct, soot)
    gallery(m, V(cx + 14, cy + 9, 6.0), V(fx + 5.0, fy + 4.0, 46.0), steel, w=3.0, h=2.0, shade="orange")
    # The cast floor: a low shed on the west, and iron running in channels from the taphole.
    m.box(WORLD, cx - 21, cx - 12, cy - 18, cy + 6, 0.0, 9.0, m.struct, m.facades["brick"])
    m.facade_windows(rect(cx - 21, cx - 12, cy - 18, cy + 6), 2.0, 8.0, 6.0, "warm", rng, lit=0.9, bands=True)
    for a, b in ((V(fx - 7.2, fy, 0.03), V(cx - 12.2, fy, 0.03)), (V(fx - 9.0, fy, 0.03), V(fx - 9.0, fy - 12.0, 0.03)),
                 (V(fx - 9.0, fy - 12.0, 0.03), V(cx - 12.2, fy - 12.0, 0.03))):
        m.line(a, b, m.molten, width=0.6, height=0.02)
    m.line(V(fx - 7.05, fy - 1.0, 1.2), V(fx - 7.05, fy + 1.0, 1.2), m.molten, width=0.8)
    marker("place_crucible", cx - 10.0, cy - 6.0, KERB, 0.0)


def power_station(m, rng, cx, cy, inner):
    """The power station: turbine hall, boiler house, three chimneys, the coal gallery."""
    gable(m, cx - 20, cx + 20, cy - 20, cy - 5, 20.0, 24.0, m.facades["steel"], m.facades["soot"])
    m.facade_windows(rect(cx - 20, cx + 20, cy - 20, cy - 5), 4.0, 19.0, 15.0, "cool", rng, lit=0.8, bands=True)
    m.ring(rect(cx - 20, cx + 20, cy - 20, cy - 5), 19.9, m.lines["orange"])
    m.box(WORLD, cx - 20, cx + 8, cy - 5, cy + 10, 0.0, 42.0, m.struct, m.facades["brick"])
    m.facade_windows(rect(cx - 20, cx + 8, cy - 5, cy + 10), 4.0, 40.0, 6.0, "warm", rng, lit=0.4)
    m.fins(rect(cx - 20, cx + 8, cy - 5, cy + 10), rect(cx - 20, cx + 8, cy - 5, cy + 10), 0.0, 42.0, m.lines["orange"])
    for x, h in ((cx - 14.0, 160.0), (cx, 175.0), (cx + 14.0, 150.0)):
        chimney(m, x, cy + 16.0, 3.6, h, m.facades["concrete"])
    gallery(m, V(cx + 19, cy - 19, 0.0), V(cx + 8.0, cy + 4.0, 38.0), m.facades["steel"])
    for k in range(3):
        x = cx + 11 + k * 3.2
        m.box(WORLD, x, x + 2.2, cy + 2, cy + 6, 0.0, RISE, m.struct, m.facades["concrete"])
        m.box(WORLD, x + 0.2, x + 2.0, cy + 2.4, cy + 5.6, RISE, 3.5, m.struct, m.facades["green"], cover="low")
        m.line(V(x + 0.2, cy + 2.38, 3.0), V(x + 2.0, cy + 2.38, 3.0), m.lines["yellow"], width=0.06)
    marker("place_power", cx + 6.0, cy - 21.0, KERB, -math.pi / 2)


def rail_yard(m, rng, cx, cy, inner):
    """Tracks across the block, wagons on them, a gantry crane over them."""
    x0, x1, y0, y1 = inner
    m.box(WORLD, x0, x1, y0, y1, KERB, KERB + 0.01, m.ground, m.facades["soot"])
    tracks = [y0 + 4.0 + k * 5.0 for k in range(int((y1 - y0 - 6.0) / 5.0) + 1)]
    for ty in tracks:
        for g in (-0.72, 0.72):
            m.box(WORLD, x0 + 1, x1 - 1, ty + g - 0.04, ty + g + 0.04, KERB, KERB + 0.06, m.ground, m.rails)
        if rng.random() < 0.55:
            x = x0 + 3
            while x + 12 < x1 - 3:
                if rng.random() < 0.65:
                    if rng.random() < 0.5:
                        m.box(WORLD, x, x + 12, ty - 1.4, ty + 1.4, KERB + 0.9, KERB + 3.6, m.cover,
                              rng.choice((m.facades["oxide"], m.facades["green"], m.facades["steel"])), cover="high")
                    else:
                        cyl_body = [V(x + 0.5, ty), V(x + 11.5, ty)]
                        beam(m, cyl_body[0] + V(0, 0, KERB + 2.3), cyl_body[1] + V(0, 0, KERB + 2.3), 2.6, 2.6,
                             m.facades["tank"], col=m.cover)
                    m.box(WORLD, x + 0.5, x + 11.5, ty - 1.2, ty + 1.2, KERB + 0.06, KERB + 0.9, m.cover, m.rail,
                          cover="low")
                x += 13.0
    gantry(m, x0 + 2.0, x1 - 2.0, (y0 + y1) / 2, 14.0, True, "yellow")
    for x in (x0 + 6, x1 - 6):
        m.pylon(WORLD, x, y0 + 1.2, KERB, height=8.0, shade="yellow")


def quay_cranes(m, rng, cx, cy, inner):
    """Dock blocks along the canal: heaps and sheds behind, cranes along the quay reaching over the
    water."""
    x0, x1, y0, y1 = inner
    for k, x in enumerate((x0 + 8.0, x1 - 8.0)):
        for dx in (-3.0, 3.0):
            for dy in (-3.0, 3.0):
                beam(m, V(x + dx, y1 - 6.0 + dy), V(x + dx * 0.6, y1 - 6.0 + dy * 0.6, 22.0), 0.7, 0.7, m.facades["oxide"])
        m.box(WORLD, x - 3, x + 3, y1 - 9, y1 - 3, 22.0, 26.0, m.struct, m.facades["steel"])
        beam(m, V(x, y1 - 14.0, 26.0), V(x, y1 + 18.0, 26.0), 1.6, 1.6, m.facades["oxide"])
        m.line(V(x - 0.85, y1 - 14, 26.0), V(x - 0.85, y1 + 18, 26.0), m.lines["orange"], width=0.1, light=False)
        m.ring(rect(x - 3, x + 3, y1 - 9, y1 - 3), 25.9, m.lines["red"], width=0.2)
    heaps(m, rng, rect(x0, x1, y0, y1 - 14.0), "docks", [])


SPECIAL = {
    (6, 7): crucible,
    (7, 6): power_station,
}
YARD = [(i, 13) for i in range(3, 11)]


# The districts' lots, and what goes up on them.

def district_of(i, j):
    x, y = block_middle(i, j)
    if y < CANAL[0]:
        return "docks"
    if i <= 1:
        return "quarter"
    if (i, j) in YARD:
        return "yard"
    if i >= 9:
        return "refinery" if y < 0 else "gasworks"
    return "steelworks"


OPTIONS = {
    "quarter": [tenement] * 4 + [works],
    "steelworks": [steel_hall] * 4 + [works] * 2 + [silos, heaps],
    "refinery": [tank_farm] * 3 + [refinery] * 3,
    "gasworks": [gasholder] * 3 + [silos] * 2 + [works],
    "docks": [heaps, silos, works, steel_hall],
}


OFFICE_CHANCE = 0.25     # the chance a lot for one of these is an office tower to go up instead
MOST_OFFICES = 8       # and the most of them

def build_lot(m, rng, district, lot, block):
    x0, x1, y0, y1 = lot
    street = faces_on_street(lot, block)
    rng.shuffle(street)
    inset = rng.uniform(0.5, 1.5)
    poly = rect(x0 + inset, x1 - inset, y0 + inset, y1 - inset)
    small = min(x1 - x0, y1 - y0)
    build = rng.choice(OPTIONS[district])
    if small < 14 and build in (gasholder, tank_farm, refinery, steel_hall):
        build = works
    if (build in (tenement,) and street and small >= urbs.OFFICE_SIZE[0]
            and m.n_lifts < MOST_OFFICES and m.lifts.random() < OFFICE_CHANCE):
        # An office tower to go up in its place (see urbs.office_tower): the one it stands for built
        # out of sight, for its draws of chance, so that the rest of the city is as it was.
        m.unseen(lambda: build(m, rng, poly, district, street))
        urbs.office_tower(m, m.lifts, poly, street, m.facades["brick"], urbs.unpainted_shade(m))
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
    m.solid(rect(x0, x1, y0, y1), 0.0, KERB, m.ground, m.pavement)
    m.ring(rect(x0, x1, y0, y1), KERB - 0.02, m.lines["orange" if district != "quarter" else "yellow"], out=0.03,
           width=0.06, light=False)
    for sx, sy in ((-1, -1), (1, 1)) if (i + j) % 2 else ((1, -1), (-1, 1)):
        m.pylon(WORLD, cx + sx * (h - 1.5), cy + sy * (h - 1.5), KERB, height=7.0, shade=m.shade())
    if district == "quarter":
        for k in range(1, 4):
            t = -h + k * (2 * h) / 4
            for x, y in ((cx + t, y0 + 1.5), (cx + t, y1 - 1.5)):
                if rng.random() < 0.5:
                    m.tree(x, y, rng, size=0.8)
    inner = (x0 + SIDEWALK, x1 - SIDEWALK, y0 + SIDEWALK, y1 - SIDEWALK)
    special = SPECIAL.get((i, j))
    if special:
        special(m, rng, cx, cy, inner)
        return
    if district == "yard":
        rail_yard(m, rng, cx, cy, inner)
        return
    if district == "docks" and j == 1:
        quay_cranes(m, rng, cx, cy, inner)
        return
    choice = "midtown" if district == "quarter" else "docks"
    for lot in urbs.lots(rng, choice, *inner):
        build_lot(m, rng, district, lot, inner)


def racks(m, rng):
    """Pipe racks over the streets between the works, posts on the pavements either side."""
    n = 0
    for i in range(BLOCKS):
        for j in range(BLOCKS):
            here = district_of(i, j)
            if here not in ("steelworks", "refinery", "gasworks"):
                continue
            cx, cy = block_middle(i, j)
            h = PITCH / 2 - STREET / 2
            for di, dj in ((1, 0), (0, 1)):
                there = (i + di, j + dj)
                if there[0] >= BLOCKS or there[1] >= BLOCKS or district_of(*there) not in ("steelworks", "refinery",
                                                                                          "gasworks"):
                    continue
                if (i, j) in SPECIAL or there in SPECIAL or rng.random() > 0.3:
                    continue
                along = rng.uniform(-h + 6, h - 6)
                if di:
                    p, q = V(cx + h - 1.2, cy + along, KERB), V(cx + PITCH - h + 1.2, cy + along, KERB)
                else:
                    p, q = V(cx + along, cy + h - 1.2, KERB), V(cx + along, cy + PITCH - h + 1.2, KERB)
                rack(m, p, q, RACK, m.shade())
                n += 1
    print(f"[map] {n} pipe racks over the streets")


# The ground: the platform, the canal, the edge, and the bridge from Urbs.

def ground(m):
    cy, cw, cd = CANAL
    south, north = cy - cw / 2, cy + cw / 2
    for y0, y1 in ((-HALF, south), (north, HALF)):
        slab = rect(-HALF, HALF, y0, y1)
        m.solid(slab, -DEPTH, -0.002, m.ground, m.road)
        m.ring(slab, -0.3, m.lines["orange"], out=0.0, light=False)
        m.ring(slab, -DEPTH + 0.15, m.lines["orange"], out=0.0, light=False)
    m.box(WORLD, -HALF, HALF, south, north, -DEPTH, -cd, m.ground, m.water)
    for y in (south + 0.05, north - 0.05):
        m.line(V(-HALF, y, -cd + 0.05), V(HALF, y, -cd + 0.05), m.lines["orange"], width=0.08)
    streets_at = [k * PITCH for k in range(-BLOCKS // 2 + 1, BLOCKS // 2)] + [-(HALF - STREET / 2), HALF - STREET / 2]
    for x in streets_at:
        b0, b1 = x - BRIDGE_WIDTH / 2, x + BRIDGE_WIDTH / 2
        m.box(WORLD, b0, b1, south - 0.3, north + 0.3, -0.6, -0.002, m.ground, m.road)
        for bx in (b0 + 0.2, b1 - 0.2):
            m.parapet(V(bx, south, 0), V(bx, north, 0), 0.0, m.lines["orange"])
    edges = sorted(v for x in streets_at for v in (x - BRIDGE_WIDTH / 2, x + BRIDGE_WIDTH / 2))
    spans = [(-HALF, edges[0])] + [(edges[k], edges[k + 1]) for k in range(1, len(edges) - 1, 2)] + [(edges[-1], HALF)]
    for s0, s1 in spans:
        if s1 - s0 > 1:
            for y, way in ((south, +1), (north, -1)):
                m.parapet(V(s0, y - way * 0.2, 0), V(s1, y - way * 0.2, 0), 0.0, m.lines["orange"])
    # The edge: a parapet all round, but for the bridge from Urbs in the west and the canal.
    corners = rect(-HALF, HALF, -HALF, HALF)
    for i in range(4):
        a, b = corners[i], corners[(i + 1) % 4]
        inward = (middle(corners) - (a + b) / 2).normalized() * 0.3
        gaps = []
        if i in (1, 3):
            lo, hi = (south - a.y) / (b.y - a.y), (north - a.y) / (b.y - a.y)
            gaps.append((min(lo, hi), max(lo, hi)))
        if i == 3:
            g = STREET / 2 / (b - a).length
            gaps.append((0.5 - g, 0.5 + g))
        m.parapet(a + inward, b + inward, 0.0, m.lines["orange"], gaps)


def link(m):
    """The road over the void from Urbs' east gate pad to Ferrum's middle street, and a gateway
    over it at Ferrum's edge."""
    x0, x1 = -HALF - LINK, -HALF
    w = STREET / 2
    m.box(WORLD, x0, x1 + 0.5, -w, w, -1.2, -0.002, m.ground, m.road)
    m.line(V(x0, -w, -0.6), V(x1, -w, -0.6), m.lines["orange"], light=False)
    m.line(V(x0, w, -0.6), V(x1, w, -0.6), m.lines["orange"], light=False)
    for s in (-1, 1):
        m.parapet(V(x0, s * (w - 0.2), 0), V(x1, s * (w - 0.2), 0), 0.0, m.lines["orange"])
    x = x0 + 2.0
    while x < x1 - 2.0:
        m.line(V(x, 0.0, 0.004), V(x + 3.0, 0.0, 0.004), m.marking, width=0.15, height=0.008, light=False)
        x += 6.0
    for x in (x0 + 4.0, (x0 + x1) / 2):
        for s in (-1, 1):
            m.pylon(WORLD, x, s * (w - 1.0), 0.0, height=7.0, shade="orange")
    # The gateway: two pylons of steel and a girder across at 12 m, its name lit on it in bars.
    gx = x1 - 3.0
    for s in (-1, 1):
        m.box(WORLD, gx - 1.2, gx + 1.2, s * (w + 0.4), s * (w + 2.8), 0.0, 14.0, m.struct, m.facades["oxide"])
        m.fins(rect(gx - 1.2, gx + 1.2, *sorted((s * (w + 0.4), s * (w + 2.8)))),
               rect(gx - 1.2, gx + 1.2, *sorted((s * (w + 0.4), s * (w + 2.8)))), 0.0, 14.0, m.lines["orange"])
    m.box(WORLD, gx - 1.0, gx + 1.0, -w - 2.8, w + 2.8, 12.0, 14.0, m.struct, m.facades["steel"])
    for k in range(7):
        y = -4.5 + k * 1.5
        m.line(V(gx - 1.02, y, 12.4), V(gx - 1.02, y, 13.6), m.molten if k % 2 else m.lines["orange"], width=0.5,
               across=V(0, 1, 0), light=False)
    marker("place_ferrum_bridge", x1 + 6.0, 0.0, 0.0, 0.0)


def road_plan():
    """Ferrum's roads and crossings, as Urbs' are: its streets are where Urbs' are, moved east."""
    plan = urbs.road_plan()
    move = lambda r: [round(r[0] + OX, 3), r[1], round(r[2] + OX, 3), r[3]]
    return {"level": plan["level"], "roads": [move(r) for r in plan["roads"]],
            "crossings": [move(r) for r in plan["crossings"]]}


def places():
    for name, (i, j) in (("place_quarter", (0, 7)), ("place_yard", (6, 13)), ("place_refinery", (11, 3)),
                         ("place_gasworks", (11, 10)), ("place_docks_ferrum", (6, 1))):
        cx, cy = block_middle(i, j)
        marker(name, cx - PITCH / 2 + 4.0, cy, KERB, 0.0)


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Ferrum(random.Random(seed + 1))
    ground(m)
    link(m)
    urbs.streets(m, rng)
    for i in range(BLOCKS):
        for j in range(BLOCKS):
            block(m, rng, i, j)
    racks(m, rng)
    places()
    m.build_buckets()
    for piece in m.pieces:
        piece["corner"][0] = round(piece["corner"][0] + OX, 3)

    bpy.ops.object.camera_add(location=(OX, -HALF * 2.0, HALF * 1.2), rotation=(math.radians(58), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.004, 0.001, 0.0, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else os.path.join(HERE, "story_ferrum.blend")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} Ferrum, {HALF * 2:.0f} m across at x={OX:.0f}, buildings={m.n_buildings} lifts={m.n_lifts} "
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
