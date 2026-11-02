"""
Lumen: a hover bike race track at night through a city traced in light, built by race_circuit.py
(`--track lumen`) with its road, walls, tunnel, pads and markers, and by this module round them.

The road, about 4.1 km round and 20 m wide, from the start line:

- the Boulevard, east, 740 m between towers, under the skyway that crosses it, and under arches of
  light;
- the Helix: straight on into a climb of a turn and a half round the Spire, left all the way, 27 m
  up, each turn 18 m over the one under it, hung from the Spire on lit spokes;
- off the top, west, onto the skyway, 27 m up, and round to the south over the Boulevard;
- down off it through S-bends into the Arcology - a megastructure over the road, the road through
  it in a tunnel - and out round the last corner onto the Boulevard.

Round it, the city: nothing in it a house or an office, as on the battle royale's Nexus
(br_nexus.py), but every building a form of its own - twisting towers of turned slabs, tapering
spires ringed with halos, stacks of boxes cantilevered off a core, terraced towers, twin towers
with skybridges, columns collared with floating rings, ribbed domes - dark, every edge and floor
lit, on a black floor ruled with lines of light. It is night: the game lights it as it lights the
Grid and Nexus, without shadows, and the lines of light along the walls light the bikes going by
(race_lumen.glow.json, beside the model).

    blender -b --python data/arena/race_circuit.py -- --track lumen --out ~/Documents/blender/map/race_lumen.blend --glb data/arena/race_lumen.glb
"""

import json
import math
import os
import random

from mathutils import Vector

# The Helix: round the Spire's middle, this far from it to the road's middle, from the bottom
# (straight on from the Boulevard, east) a turn and a half anticlockwise - left - to the top
# (heading west), climbing all the while.
# The layout is drawn at SCALE across and RISE up: the plan the numbers below are in, made bigger.
SCALE = 2.4
RISE = 1.8
HELIX = (170.0 * SCALE, 50.0 * SCALE, 50.0 * SCALE)
HELIX_RISE = 15.0 * RISE
HELIX_STEPS = 18                 # waypoints round it, 30 degrees apart
SPIRE = (72.0, 10.0, 300.0)      # the Spire's radius at its foot and its top, and its height


def helix():
    cx, cy, r = HELIX
    out = []
    for k in range(HELIX_STEPS + 1):
        a = math.radians(-90.0 + 30.0 * k)
        out.append((cx + r * math.cos(a), cy + r * math.sin(a), HELIX_RISE * k / HELIX_STEPS))
    return out


# The road's middle line round the way it is raced (see race_circuit.py's WAYPOINTS), as planned.
PLAN = [
    (-200.0, 0.0, 0.0),     # the start line
    (-120.0, 0.0, 0.0),
    (-40.0, 0.0, 0.0),      # under the skyway
    (40.0, 0.0, 0.0),
    (110.0, 0.0, 0.0),
] + [None] * (HELIX_STEPS + 1) + [   # 5 to 23: the Helix
    (100.0, 100.0, 15.0),   # 24: the skyway, west
    (30.0, 100.0, 15.0),
    (-25.0, 75.0, 15.0),    # round to the south
    (-40.0, 20.0, 15.0),    # over the Boulevard
    (-40.0, -40.0, 14.5),
    (-20.0, -100.0, 12.0),  # 29: the S-bends, down off it
    (-60.0, -150.0, 9.0),
    (-110.0, -175.0, 6.0),  # 31: into the Arcology
    (-170.0, -185.0, 4.0),
    (-230.0, -175.0, 2.0),
    (-280.0, -120.0, 0.5),  # 34: the last corner
    (-280.0, -50.0, 0.0),
    (-250.0, -10.0, 0.0),
]
WAYPOINTS = [(x * SCALE, y * SCALE, z * RISE) for x, y, z in PLAN[:5]] + helix() + \
    [(x * SCALE, y * SCALE, z * RISE) for x, y, z in PLAN[5 + HELIX_STEPS + 1:]]

LAYOUT = {
    "WAYPOINTS": WAYPOINTS,
    "TUNNEL": (31.0, 32.75),
    "ROAD": 20.0,
    "TUNNEL_HEIGHT": 8.0,
    "PADS": [
        (1.5, 0.3),         # down the Boulevard
        (3.2, 0.7),         # and again, near its end
        (24.4, 0.65),       # off the top of the Helix, onto the skyway
        (28.3, 0.35),       # on the skyway, heading down
        (29.6, 0.6),        # through the S-bends
        (35.2, 0.4),        # out of the last corner
    ],
    "NIGHT": True,
}

CITY_MARGIN = 420.0          # how far out from the track the city goes
GROUND_MARGIN = 600.0        # and the floor under it
GRID = 20.0                  # the floor's lines of light, this far apart
FLOOR = 4.0                  # a storey
BUILDINGS = 600
ARCH_EVERY = 60              # sections between arches over the road, at the most
ARCH_HEIGHT = 9.5

CYAN = (0.0, 0.85, 1.0)
MAGENTA = (1.0, 0.1, 0.75)
PALETTE = {
    "cyan": CYAN,
    "orange": (1.0, 0.45, 0.05),
    "pink": (1.0, 0.08, 0.55),
    "yellow": (1.0, 0.82, 0.05),
    "green": (0.15, 1.0, 0.3),
    "violet": (0.5, 0.15, 1.0),
}
WHITE = (0.75, 0.9, 1.0)
SEGMENT = 3.0                # the longest piece of a line of light the game lights with


def night_materials(rc, m):
    """The track's own materials made over for night: dark and glossy, the kerbs' white stripes
    lit, the start line's white squares glowing."""
    m.asphalt = rc.material("LumenAsphalt", (0.02, 0.022, 0.03), roughness=0.25, metallic=0.3)
    m.deck = rc.material("LumenDeck", (0.03, 0.035, 0.05), roughness=0.4, metallic=0.4)
    m.barrier = rc.material("LumenBarrier", (0.04, 0.045, 0.06), roughness=0.3, metallic=0.6)
    m.kerb_red = rc.material("LumenKerbDark", (0.03, 0.03, 0.04), roughness=0.5)
    m.kerb_white = rc.glow("LumenKerbLit", WHITE, 1.5)
    m.stone = rc.material("LumenTunnel", (0.035, 0.04, 0.055), roughness=0.4, metallic=0.5)
    m.check_white = rc.glow("LumenCheck", (0.9, 0.95, 1.0), 1.0)


def game(v):
    """A point or way in the game's terms: y up (Blender's x, z, -y)."""
    return [round(v.x, 3), round(v.z, 3), round(-v.y, 3)]


def turn(v, angle):
    c, s = math.cos(angle), math.sin(angle)
    return Vector((v.x * c - v.y * s, v.x * s + v.y * c, v.z))


def polygon(r, n, rot=0.0, cx=0.0, cy=0.0):
    return [Vector((cx + r * math.cos(rot + 2 * math.pi * i / n), cy + r * math.sin(rot + 2 * math.pi * i / n), 0.0))
            for i in range(n)]


class City:
    def __init__(self, rc, m, secs):
        self.rc, self.m, self.secs = rc, m, secs
        self.shells = rc.Builder("CityShells")
        self.lights = rc.Builder("CityLights")
        self.ground = rc.Builder("CityGround")
        self.dark = [rc.material(f"LumenShell{i}", rgb, roughness=0.3, metallic=0.55) for i, rgb in enumerate(
            [(0.03, 0.035, 0.05), (0.045, 0.05, 0.068), (0.022, 0.026, 0.04)])]
        self.floor = rc.material("LumenFloor", (0.012, 0.014, 0.022), roughness=0.15, metallic=0.6)
        self.glows = {name: rc.glow(f"LumenGlow{name.title()}", rgb, 3.0) for name, rgb in PALETTE.items()}
        self.white = rc.glow("LumenGlowWhite", WHITE, 2.5)
        self.windows = rc.glow("LumenWindows", (0.55, 0.75, 1.0), 0.9)
        self.grid = rc.glow("LumenGrid", CYAN, 1.0)
        self.colour_of = {id(mat): PALETTE[name] for name, mat in self.glows.items()}
        self.colour_of[id(m.cyan)] = CYAN
        self.colour_of[id(m.magenta)] = MAGENTA
        self.pieces = []
        self.taken = []          # (x, y, r): kept clear of buildings
        self.rng = random.Random(11)
        self.count = {}

    def shade(self):
        return self.glows[self.rng.choice(("cyan", "cyan", "cyan", "orange", "pink", "yellow", "green", "violet"))]

    # Shapes.

    def solid(self, base, z0, z1, mat, top=None, b=None):
        """A solid on the polygon `base` from z0 up to z1 - its top `top`, if it narrows."""
        b = b or self.shells
        top = top or base
        n = len(base)
        lo = [Vector((p.x, p.y, z0)) for p in base]
        hi = [Vector((p.x, p.y, z1)) for p in top]
        b.face(lo, mat)
        b.face(hi, mat)
        for i in range(n):
            j = (i + 1) % n
            b.face([lo[i], lo[j], hi[j], hi[i]], mat)

    def line(self, p, q, mat, width=0.14, height=None, across=None, light=False, b=None):
        """A bar of light from p to q, `width` across (along `across`, if given) and `height` thick;
        lit with in the game if `light`."""
        b = b or self.lights
        height = width if height is None else height
        d = q - p
        if d.length < 1e-4:
            return
        dn = d.normalized()
        a = across.copy() if across is not None else dn.cross(Vector((0, 0, 1)))
        if a.length < 1e-6:
            a = Vector((1, 0, 0))
        a = (a - dn * a.dot(dn)).normalized()
        up = dn.cross(a)
        b.oriented_box((p + q) / 2, dn, a, up, (d.length / 2, width / 2, height / 2), mat)
        if light and id(mat) in self.colour_of:
            self.light_piece(p - a * (width / 2), d, a * width, self.colour_of[id(mat)])

    def light_piece(self, corner, along, across, rgb):
        pieces = max(1, math.ceil(along.length / SEGMENT))
        for i in range(pieces):
            self.pieces.append({"corner": game(corner + along * (i / pieces)), "edges": [game(along / pieces), game(across)],
                                "colour": list(rgb)})

    def ring(self, corners, z, mat, out=0.08, width=0.14):
        n = len(corners)
        mid = sum(corners, Vector()) / n
        for i in range(n):
            a, b = corners[i], corners[(i + 1) % n]
            pa = a + (a - mid).normalized() * out
            pb = b + (b - mid).normalized() * out
            self.line(Vector((pa.x, pa.y, z)), Vector((pb.x, pb.y, z)), mat, width)

    def fins(self, base, top, z0, z1, mat, out=0.08, every=1, width=0.14):
        mid0, mid1 = sum(base, Vector()) / len(base), sum(top, Vector()) / len(top)
        for i in range(0, len(base), every):
            a = base[i] + (base[i] - mid0).normalized() * out
            b = top[i] + (top[i] - mid1).normalized() * out
            self.line(Vector((a.x, a.y, z0)), Vector((b.x, b.y, z1)), mat, width)

    def outline(self, corners, z0, z1, mat, width=0.12):
        """Every edge of the prism on `corners` from z0 to z1, lit."""
        self.ring(corners, z0, mat, width=width)
        self.ring(corners, z1, mat, width=width)
        self.fins(corners, corners, z0, z1, mat, width=width)

    def floors(self, corners, z0, z1, every=FLOOR):
        """A dim band of windows round the prism at every storey."""
        z = z0 + every
        while z < z1 - 0.5:
            self.ring(corners, z, self.windows, out=0.04, width=0.22)
            z += every

    # The buildings.

    def twist(self, x, y, r, h):
        """A tower of square slabs a storey each, each turned a little further than the one under
        it, narrowing as it goes up; a band of windows round every storey, a ring of light every
        few, and a mast on top."""
        rng = self.rng
        half, taper = r * 0.68, rng.uniform(0.45, 0.8)
        twist, rot = math.radians(rng.uniform(35, 100)) * rng.choice((-1, 1)), rng.uniform(0, math.pi)
        n = max(4, int(h / FLOOR))
        mat, shade = rng.choice(self.dark), self.shade()
        every = rng.choice((4, 5, 6))
        for k in range(n):
            s = half * (1 - (1 - taper) * k / n)
            a = rot + twist * k / n
            sq = [turn(Vector((sx * s, sy * s, 0)), a) + Vector((x, y, 0)) for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
            z0, z1 = k * FLOOR, (k + 1) * FLOOR
            self.solid(sq, z0, z1 - 0.35, mat)
            # A recessed slab between, a little narrower: the storeys read as plates.
            inner = [Vector((x, y, 0)) + (p - Vector((x, y, 0))) * 0.9 for p in sq]
            self.solid(inner, z1 - 0.35, z1, self.dark[2])
            self.ring(sq, z1 - 0.35, shade if k % every == every - 1 else self.windows, out=0.03, width=0.1)
        top = n * FLOOR
        self.line(Vector((x, y, top)), Vector((x, y, top + h * 0.15)), self.white, width=0.3)

    def spire(self, x, y, r0, r1, h, shade=None, halo_every=14.0):
        """A tapering octagonal spire lit up every edge, haloed with rings of light standing off
        it, and a needle on top."""
        shade = shade or self.shade()
        base, top = polygon(r0, 8, math.pi / 8, x, y), polygon(r1, 8, math.pi / 8, x, y)
        self.solid(base, 0.0, h, self.dark[2], top=top)
        self.fins(base, top, 0.0, h, shade)
        z, k = halo_every, 0
        while z < h - 4:
            rz = r0 + (r1 - r0) * z / h
            self.ring(polygon(rz + 2.5, 8, math.pi / 8 * (1 + k % 2), x, y), z, self.glows[list(PALETTE)[k % len(PALETTE)]],
                      out=0.0, width=0.25)
            self.ring(polygon(rz, 8, math.pi / 8, x, y), z, self.windows, out=0.04, width=0.3)
            z += halo_every
            k += 1
        self.line(Vector((x, y, h)), Vector((x, y, h + h * 0.18)), self.white, width=0.35)

    def stack(self, x, y, r, h):
        """A square core with boxes cantilevered off it at every height, turned this way and that,
        each one's edges lit."""
        rng = self.rng
        core = r * 0.32
        rot = rng.uniform(0, math.pi / 2)
        f = lambda px, py: turn(Vector((px, py, 0)), rot) + Vector((x, y, 0))
        sq = [f(-core, -core), f(core, -core), f(core, core), f(-core, core)]
        mat = rng.choice(self.dark)
        self.solid(sq, 0.0, h, mat)
        self.fins(sq, sq, 0.0, h, self.windows, width=0.2)
        z = rng.uniform(5, 10)
        while z < h - 6:
            w, d, t = rng.uniform(core * 1.4, r * 1.1), rng.uniform(core * 0.9, core * 1.6), rng.uniform(4, 8)
            side = rng.uniform(0, 2 * math.pi)
            off = Vector((math.cos(side), math.sin(side), 0)) * rng.uniform(core * 0.6, r - w / 2 - 0.5)
            g = lambda px, py: turn(Vector((px, py, 0)), side) + Vector((x, y, 0)) + off
            box = [g(-w / 2, -d / 2), g(w / 2, -d / 2), g(w / 2, d / 2), g(-w / 2, d / 2)]
            if max((p.xy - Vector((x, y))).length for p in box) > r:
                z += 2
                continue
            self.solid(box, z, z + t, rng.choice(self.dark))
            shade = self.shade()
            self.outline(box, z, z + t, shade)
            # Its end glazed.
            self.ring(box, z + t / 2, self.windows, out=0.04, width=0.5)
            z += t + rng.uniform(0.5, 6)
        self.ring(sq, h, self.white)

    def terraced(self, x, y, r, h):
        """A tower stepping in as it rises, each terrace's edge lit, glowing strips up its faces."""
        rng = self.rng
        tiers = rng.randint(3, 5)
        rot = rng.uniform(0, math.pi / 2)
        shade = self.shade()
        z = 0.0
        for t in range(tiers):
            s = r * 0.7 * (1 - 0.6 * t / tiers)
            tall = h / tiers * (1.3 if t == 0 else 1.0) * rng.uniform(0.8, 1.1)
            sq = [turn(Vector((sx * s, sy * s, 0)), rot) + Vector((x, y, 0)) for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
            # Corners cut off: an octagon from the square.
            cut = []
            for i in range(4):
                a, b = sq[i], sq[(i + 1) % 4]
                cut += [a.lerp(b, 0.18), a.lerp(b, 0.82)]
            self.solid(cut, z, z + tall, self.dark[t % 3])
            self.ring(cut, z + tall, shade, width=0.2)
            self.floors(cut, z, z + tall)
            # Strips of light up the middle of every other face.
            for i in range(0, 8, 2):
                a, b = cut[i], cut[(i + 1) % 8]
                mid = (a + b) / 2
                out = (mid - Vector((x, y, 0))).normalized() * 0.06
                self.line(Vector((mid.x, mid.y, z)) + out, Vector((mid.x, mid.y, z + tall)) + out, shade, width=0.3)
            z += tall
        self.line(Vector((x, y, z)), Vector((x, y, z + 12)), self.white, width=0.25)

    def twin(self, x, y, r, h):
        """Two slender octagonal towers, one taller, joined by skybridges."""
        rng = self.rng
        a = rng.uniform(0, math.pi)
        way = Vector((math.cos(a), math.sin(a), 0))
        tr = r * 0.32
        apart = r - tr
        heights = (h, h * rng.uniform(0.7, 0.9))
        shade = self.shade()
        for sgn, th in zip((-1, 1), heights):
            c = Vector((x, y, 0)) + way * apart * sgn
            base, top = polygon(tr, 8, a + math.pi / 8, c.x, c.y), polygon(tr * 0.75, 8, a + math.pi / 8, c.x, c.y)
            self.solid(base, 0.0, th, self.dark[0], top=top)
            self.fins(base, top, 0.0, th, shade, every=2)
            self.floors(base, 0.0, th * 0.5)
            self.ring(top, th, self.white)
            self.line(Vector((c.x, c.y, th)), Vector((c.x, c.y, th + 10)), self.white, width=0.2)
        across = Vector((-way.y, way.x, 0))
        for z in sorted(rng.sample(range(int(h * 0.25), int(min(heights) * 0.9), 4), k=min(3, max(1, int(h / 40))))):
            p = Vector((x, y, 0)) - way * (apart - tr * 0.8)
            q = Vector((x, y, 0)) + way * (apart - tr * 0.8)
            box = [p - across * 2.0, q - across * 2.0, q + across * 2.0, p + across * 2.0]
            self.solid(box, z, z + 3.5, self.dark[1])
            self.outline(box, z, z + 3.5, shade, width=0.1)

    def collared(self, x, y, r, h):
        """A round column with rings of light floating round it, and a pod near its top."""
        rng = self.rng
        cr = r * 0.3
        shade = self.shade()
        col = polygon(cr, 16, 0.0, x, y)
        self.solid(col, 0.0, h, self.dark[1])
        self.fins(col, col, 0.0, h, self.windows, every=4, width=0.2)
        for z in range(int(rng.uniform(10, 18)), int(h * 0.75), int(rng.uniform(12, 20))):
            ring = polygon(cr + rng.uniform(3, r - cr - 0.5), 16, rng.uniform(0, 1), x, y)
            self.ring(ring, z, shade, out=0.0, width=0.5)
            # Held off it by four arms.
            for i in range(0, 16, 4):
                self.line(Vector((col[i].x, col[i].y, z)), Vector((ring[i].x, ring[i].y, z)), self.dark[2], width=0.3,
                          b=self.shells)
        pz = h * rng.uniform(0.78, 0.88)
        pod = polygon(r * 0.85, 16, 0.0, x, y)
        self.solid(pod, pz, pz + 5.0, self.dark[0], top=polygon(r * 0.6, 16, 0.0, x, y))
        self.solid(polygon(r * 0.6, 16, 0.0, x, y), pz - 3.0, pz, self.dark[2], top=pod)
        self.ring(pod, pz, shade, out=0.05, width=0.3)
        self.ring(polygon(r * 0.6, 16, 0.0, x, y), pz + 5.0, self.white, out=0.05)

    def dome(self, x, y, r):
        """A dome ribbed with light."""
        shade = self.shade()
        bands = 6
        rings = []
        for b in range(bands + 1):
            a = b * (math.pi / 2) / bands
            rings.append((polygon(max(r * math.cos(a), 0.5), 16, 0.0, x, y), r * 0.8 * math.sin(a)))
        for b in range(bands):
            (lo, z0), (hi, z1) = rings[b], rings[b + 1]
            self.solid(lo, z0, z1, self.dark[b % 3], top=hi)
        for i in range(0, 16, 2):
            for b in range(bands):
                (lo, z0), (hi, z1) = rings[b], rings[b + 1]
                c = Vector((x, y, 0))
                p, q = lo[i] + (lo[i] - c).normalized() * 0.06, hi[i] + (hi[i] - c).normalized() * 0.06
                self.line(Vector((p.x, p.y, z0)), Vector((q.x, q.y, z1)), shade)
        self.ring(rings[0][0], 0.1, shade, width=0.3)
        self.ring(rings[3][0], rings[3][1], self.white)

    # Laying the city out.

    def clear_of_road(self, x, y, r, margin):
        return self.rc.far_from_track(self.secs, x, y, r + margin)

    def clear_of_buildings(self, x, y, r):
        return all(math.hypot(x - tx, y - ty) > r + tr + 5.0 for tx, ty, tr in self.taken)

    def city(self, bounds):
        (x0, y0), (x1, y1) = bounds
        rng = self.rng
        kinds = ["twist"] * 3 + ["spire"] * 2 + ["stack"] * 2 + ["terraced"] * 2 + ["twin"] * 2 + ["collared"] * 2 + ["dome"]
        placed, tries = 0, 0
        while placed < BUILDINGS and tries < 25000:
            tries += 1
            x = rng.uniform(x0 - CITY_MARGIN, x1 + CITY_MARGIN)
            y = rng.uniform(y0 - CITY_MARGIN, y1 + CITY_MARGIN)
            r = rng.uniform(10.0, 26.0)
            if not self.clear_of_buildings(x, y, r) or not self.clear_of_road(x, y, r, 4.0):
                continue
            # How near the road: the nearer, the taller - a canyon along it.
            near = min((Vector((x, y)) - s.centre.xy).length for s in self.secs[::4]) - self.rc.ROAD / 2
            if near > CITY_MARGIN:
                continue
            h = rng.uniform(30, 70) + 120 * max(0.0, 1 - near / 160) * rng.uniform(0.4, 1.0)
            kind = rng.choice(kinds)
            if kind == "dome":
                self.dome(x, y, r)
            elif kind == "spire":
                self.spire(x, y, r * 0.75, r * 0.12, h * 1.2)
            else:
                getattr(self, kind)(x, y, r, h)
            self.count[kind] = self.count.get(kind, 0) + 1
            self.taken.append((x, y, r))
            placed += 1

    def floor_grid(self, bounds):
        """The black floor under it all, ruled with lines of light."""
        (x0, y0), (x1, y1) = bounds
        x0, y0, x1, y1 = x0 - GROUND_MARGIN, y0 - GROUND_MARGIN, x1 + GROUND_MARGIN, y1 + GROUND_MARGIN
        self.ground.face([Vector((x0, y0, -0.05)), Vector((x1, y0, -0.05)), Vector((x1, y1, -0.05)), Vector((x0, y1, -0.05))],
                         self.floor)
        w = 0.08
        gx = math.ceil(x0 / GRID) * GRID
        while gx < x1:
            self.ground.face([Vector((gx - w, y0, -0.04)), Vector((gx + w, y0, -0.04)), Vector((gx + w, y1, -0.04)),
                              Vector((gx - w, y1, -0.04))], self.grid)
            gx += GRID
        gy = math.ceil(y0 / GRID) * GRID
        while gy < y1:
            self.ground.face([Vector((x0, gy - w, -0.04)), Vector((x1, gy - w, -0.04)), Vector((x1, gy + w, -0.04)),
                              Vector((x0, gy + w, -0.04))], self.grid)
            gy += GRID

    # By the road.

    def the_spire(self):
        """The Spire in the middle of the Helix, and the spokes it holds the Helix up by."""
        cx, cy, _ = HELIX
        r0, r1, h = SPIRE
        self.spire(cx, cy, r0, r1, h, shade=self.glows["cyan"], halo_every=12.0)
        self.taken.append((cx, cy, r0 + 2))
        apothem = lambda z: (r0 + (r1 - r0) * z / h) * math.cos(math.pi / 8)
        # The spokes, under the inside edge of the road wherever it is up off the ground.
        for i, s in enumerate(self.secs):
            if not (5.0 <= s.fraction <= 23.0) or i % 6 or s.left.z < 3.0:
                continue
            c = Vector((cx, cy, 0))
            way = (s.left.xy - c.xy).normalized()
            z = s.left.z - self.rc.DECK - 0.5
            p = c + Vector((way.x, way.y, 0)) * (apothem(z) - 0.3) + Vector((0, 0, z))
            q = Vector((s.left.x, s.left.y, z)) - s.right * self.rc.BARRIER[1]
            self.line(p, q, self.dark[2], width=0.9, height=1.0, b=self.shells)
            self.line(p - Vector((0, 0, 0.52)), q - Vector((0, 0, 0.52)), self.glows["cyan"], width=0.2, height=0.06)

    def arcology(self):
        """The Arcology: bays of building over the road where it runs through the tunnel, standing
        either side of it, lit storey by storey, towers stepping up off its roof."""
        rc = self.rc
        inside = [i for i, s in enumerate(self.secs) if rc.in_tunnel(s)]
        if not inside:
            return
        roof = max(self.secs[i].centre.z for i in inside) + rc.TUNNEL_HEIGHT + 1.2
        top = roof + 22.0
        wing = 16.0
        shade = self.glows["violet"]
        up = Vector((0, 0, 1))
        for k in range(0, len(inside), 4):
            s = self.secs[inside[k]]
            length = rc.STEP * 4 / 2 + 0.05
            # The span over the road.
            span = rc.ROAD / 2 + rc.BARRIER[1] + wing
            mid = Vector((s.centre.x, s.centre.y, (roof + top) / 2))
            self.shells.oriented_box(mid, s.flat, s.right, up, (length, span, (top - roof) / 2), self.dark[k // 4 % 2])
            # Its wings down to the ground either side.
            for side in (-1, 1):
                off = s.right * side * (rc.ROAD / 2 + rc.BARRIER[1] + 1.5 + wing / 2)
                c = Vector((s.centre.x, s.centre.y, roof / 2)) + Vector((off.x, off.y, 0))
                self.shells.oriented_box(c, s.flat, s.right, up, (length, wing / 2, roof / 2 + 0.05), self.dark[0])
            # Storeys lit along both faces.
            for z in [roof + 3 + 4 * j for j in range(5)] + [3.0 + 4 * j for j in range(int(roof // 4))]:
                for side in (-1, 1):
                    edge = Vector((s.centre.x, s.centre.y, z)) + s.right * side * (span + 0.06)
                    self.line(edge - s.flat * length, edge + s.flat * length, self.windows if z > 3.5 else shade, width=0.25,
                              across=Vector((0, 0, 1)))
            # The span's underside over the portals, edged in light.
            for i in (0, len(inside) - 1):
                if k == i or k == (i // 4) * 4:
                    portal = Vector((s.centre.x, s.centre.y, roof - 0.1))
                    self.line(portal - s.right * span, portal + s.right * span, shade, width=0.4)
            self.taken.append((s.centre.x, s.centre.y, span))
        # Towers off its roof.
        mid = self.secs[inside[len(inside) // 2]]
        for t, along in enumerate((-30.0, 0.0, 28.0)):
            c = mid.centre + mid.flat * along
            r = 8.0 - t
            sq = [Vector((c.x, c.y, 0)) + mid.flat * sx * r + mid.right * sy * r for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
            h = top + 18 + 14 * (t % 2)
            self.solid(sq, top, h, self.dark[2])
            self.outline(sq, top, h, shade)
            self.floors(sq, top, h)

    def arches(self):
        """Arches of light over the road, where nothing goes over it nearby: legs at its edges, a
        beam over it, lit on the inside."""
        rc = self.rc
        n = len(self.secs)
        last = -ARCH_EVERY
        colours = ("cyan", "pink", "yellow", "violet", "green", "orange")
        made = 0
        for i in range(0, n, 6):
            s = self.secs[i]
            if i - last < ARCH_EVERY or rc.in_tunnel(s) or i < 12:
                continue
            crowded = any(
                (o.centre.xy - s.centre.xy).length < rc.ROAD * 1.2 and -2.0 < o.centre.z - s.centre.z < ARCH_HEIGHT + 6
                for j, o in enumerate(self.secs) if abs(j - i) > 15 and abs(j - i) < n - 15)
            if crowded:
                continue
            last = i
            shade = self.glows[colours[made % len(colours)]]
            made += 1
            up = Vector((0, 0, 1))
            reach = rc.ROAD / 2 + rc.BARRIER[1] + 0.6
            zt = s.centre.z + ARCH_HEIGHT
            for side in (-1, 1):
                foot = (s.left if side < 0 else s.right_edge) + s.right * side * (rc.BARRIER[1] + 0.6)
                c = Vector((foot.x, foot.y, (foot.z - rc.DECK + zt + 1.0) / 2))
                self.shells.oriented_box(c, s.flat, s.right, up, (0.6, 0.6, (zt + 1.0 - foot.z + rc.DECK) / 2), self.dark[1])
                inner = Vector((foot.x, foot.y, 0)) - s.right * side * 0.63
                self.line(Vector((inner.x, inner.y, foot.z + 0.2)), Vector((inner.x, inner.y, zt)), shade, width=0.3,
                          across=s.flat, light=True)
            beam = Vector((s.centre.x, s.centre.y, zt + 0.5))
            self.shells.oriented_box(beam, s.flat, s.right, up, (0.6, reach + 0.6, 0.55), self.dark[1])
            self.line(beam - s.right * reach - up * 0.58, beam + s.right * reach - up * 0.58, shade, width=0.3, height=0.06,
                      across=s.flat)
            # A second, thinner frame just behind, so that it reads as a gate.
            for off in (-1.6, 1.6):
                b2 = beam + s.flat * off + up * 1.4
                self.line(b2 - s.right * reach, b2 + s.right * reach, shade, width=0.18)

    def road_lights(self):
        """The lines of light along the tops of the walls, as the game lights with them; and a
        dashed line of light down the middle of the road."""
        rc = self.rc
        n = len(self.secs)
        h, w = rc.BARRIER
        lines = rc.Builder("RoadLines")
        for i in range(n):
            a, b = self.secs[i], self.secs[(i + 1) % n]
            for side, rgb in ((-1, CYAN), (1, MAGENTA)):
                ea = a.left if side < 0 else a.right_edge
                eb = b.left if side < 0 else b.right_edge
                corner = ea + Vector((0, 0, h + 0.01))
                self.light_piece(corner, eb - ea, a.right * (side * w * 0.35), rgb)
            if i % 4 < 2:
                t = 0.12 / rc.ROAD
                lines.quad_strip(a.across(0.5 - t, 0.012), a.across(0.5 + t, 0.012), b.across(0.5 - t, 0.012),
                                 b.across(0.5 + t, 0.012), self.white)
        return lines.finish()


def build(rc, secs, m, bounds, glb):
    """Lumen's city round the track `secs`, and its lines of light for the game beside `glb`."""
    city = City(rc, m, secs)
    city.the_spire()
    city.arcology()
    city.arches()
    road = city.road_lights()
    city.city(bounds)
    city.floor_grid(bounds)
    objs = [road, city.shells.finish(), city.lights.finish(), city.ground.finish()]
    print(f"[lumen] buildings {sum(city.count.values())}: {city.count}; {len(city.pieces)} pieces of light")
    if glb:
        path = os.path.splitext(os.path.abspath(glb))[0] + ".glow.json"
        with open(path, "w") as f:
            json.dump({"intensity": 3.0, "pieces": city.pieces}, f, separators=(",", ":"))
        print(f"[lumen] lines of light -> {path}")
    return objs
