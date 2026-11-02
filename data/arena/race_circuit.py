"""
Hover bike race tracks, in open country under the sky: a road walled on both sides the whole way
round, the walls' tops lit, cyan on the left and magenta on the right, so that which way is round
reads at a glance. Each is a layout of TRACKS, built the same way.

The Circuit (`circuit`) is about 1.1 km round, a figure of eight, its road 16 m wide. Going round,
from the start line:

- the start/finish straight, east, 210 m under a gantry and past the grandstand, and under the
  viaduct that crosses it;
- a wide hairpin at the east end, banked, swinging round to the north and back west;
- a chicane, left-right-left, climbing all the while;
- the viaduct, 10 m up on pillars, curving south over the start straight;
- down off it into a run of S-bends, the third through a tunnel, and round a last banked corner
  onto the straight again.

The Grand Circuit (`grand`) is the same plan, wider and longer: about 2.1 km round, its road 24 m
wide. A 540 m straight, east, under the viaduct; a long banked sweeper round to the north; a
chicane of four, climbing west along the north side; round onto the viaduct, 14 m up, and south
over the straight; down through S-bends and a 120 m tunnel; and a hairpin in the south-west back
onto the straight.

The road banks into every corner by as much as it bends (see `banking`): up to BANK_MAX, the
outside edge up and the inside left where the middle line is, so that no edge dips into the
ground. Kerbs, red and white, run along both edges. Where the road is up off the ground
it is a deck on pillars, none of them standing on the road below.

Builds a track's .glb for the game (and a .blend to look it over, wherever --out puts it), the
Circuit unless --track says otherwise:

    blender -b --python data/arena/race_circuit.py -- --out ~/Documents/blender/map/race_circuit.blend --glb data/arena/race_circuit.glb
    blender -b --python data/arena/race_circuit.py -- --track grand --out ~/Documents/blender/map/race_grand.blend --glb data/arena/race_grand.glb

Made for Ruptura Systematis (MazeGame/maze), whose race mode (src/race.rs) reads its empties:
`race_start` on the start line, its +x the way round; `checkpoint_1` to `checkpoint_N`, every
CHECKPOINT_EVERY meters round the middle of the road, in order, the last just before the line;
and `race_grid_1` to `race_grid_8`, the grid behind the line, two by two.
"""

import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector

# The road's middle line, round the way it is raced: (x, y, height). A closed loop, smoothed
# through these (see `centre_line`). The viaduct crosses the straight at about (-42, 0).
WAYPOINTS = [
    (-140.0, 0.0, 0.0),     # the start line
    (-60.0, 0.0, 0.0),
    (20.0, 0.0, 0.0),
    (70.0, 0.0, 0.0),       # the end of the straight
    (112.0, 14.0, 0.5),     # the hairpin
    (128.0, 60.0, 1.5),
    (102.0, 104.0, 3.0),
    (40.0, 116.0, 5.0),     # the chicane
    (10.0, 100.0, 6.0),
    (-16.0, 116.0, 7.5),
    (-46.0, 104.0, 9.0),
    (-56.0, 60.0, 10.0),    # onto the viaduct
    (-42.0, 0.0, 10.0),     # over the straight
    (-36.0, -55.0, 9.0),
    (-56.0, -100.0, 7.0),   # the S-bends, down off it
    (-100.0, -118.0, 5.0),
    (-140.0, -100.0, 3.0),  # the tunnel
    (-172.0, -124.0, 1.5),
    (-206.0, -100.0, 0.5),
    (-216.0, -46.0, 0.0),   # the last corner
    (-190.0, -6.0, 0.0),
]
TUNNEL = (15.6, 17.4)       # from where to where, in waypoints, the road goes through the tunnel

ROAD = 16.0                 # the road's width
STEP = 2.0                  # how far apart the road's sections are along it
DECK = 1.0                  # how thick the road is where it is up off the ground
BARRIER = (1.1, 0.5)        # each wall's height over the road's edge, and thickness
KERB = (0.9, 4.0)           # a kerb's width, and each stripe's length
BANK_MAX = 18.0             # the most the road banks, in degrees
BANK_SPEED = 32.0           # the speed it is banked for, in m/s
PILLAR = (1.6, 24.0)        # a pillar's side, and how far apart they are along the road
TUNNEL_HEIGHT = 7.0         # the tunnel's ceiling over the road
CHECKPOINT_EVERY = 25.0
GROUND_MARGIN = 140.0       # how far the ground goes out past the track
MONOLITHS = 140
LIGHT_TOWERS = 14
STAND_LENGTH = 58.0         # the grandstand's length along the straight

# The layouts: what each changes from the Circuit's measures above.
TRACKS = {
    "circuit": {},
    "grand": {
        "WAYPOINTS": [
            (-300.0, 0.0, 0.0),     # the start line
            (-150.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),        # under the viaduct
            (150.0, 0.0, 0.0),
            (240.0, 0.0, 0.0),      # the end of the straight
            (300.0, 30.0, 0.5),     # the sweeper
            (322.0, 100.0, 1.5),
            (292.0, 170.0, 3.0),
            (230.0, 202.0, 4.0),
            (170.0, 190.0, 5.0),    # the chicane
            (130.0, 216.0, 6.0),
            (90.0, 190.0, 7.0),
            (50.0, 216.0, 8.0),
            (0.0, 200.0, 10.0),
            (-40.0, 160.0, 12.0),   # round onto the viaduct
            (-20.0, 90.0, 14.0),
            (0.0, 0.0, 14.0),       # over the straight
            (10.0, -80.0, 13.0),
            (-20.0, -150.0, 11.0),  # the S-bends, down off it
            (-80.0, -180.0, 9.0),
            (-140.0, -160.0, 7.0),  # the tunnel
            (-200.0, -190.0, 5.0),
            (-260.0, -170.0, 3.0),
            (-320.0, -200.0, 1.5),
            (-382.0, -160.0, 0.5),  # the hairpin
            (-392.0, -80.0, 0.0),
            (-360.0, -20.0, 0.0),
        ],
        "TUNNEL": (20.3, 22.3),
        "ROAD": 24.0,
        "TUNNEL_HEIGHT": 8.0,
        "MONOLITHS": 320,
        "LIGHT_TOWERS": 28,
        "STAND_LENGTH": 90.0,
    },
}

ASPHALT = (0.14, 0.145, 0.16)
CYAN = (0.0, 0.85, 1.0)
MAGENTA = (1.0, 0.1, 0.75)
WHITE = (1.0, 0.97, 0.92)


def get_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    glb = argv[argv.index("--glb") + 1] if "--glb" in argv else None
    out = argv[argv.index("--out") + 1] if "--out" in argv else None
    track = argv[argv.index("--track") + 1] if "--track" in argv else "circuit"
    return glb, out, track


def material(name, rgb, roughness=0.6, metallic=0.1):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*rgb, 1)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    m.diffuse_color = (*rgb, 1)
    return m


def glow(name, rgb, strength):
    """Glowing `rgb`, its own colour `rgb` too: the engine scales what a surface gives off by it."""
    m = material(name, rgb, roughness=0.5, metallic=0.0)
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Emission Color"].default_value = (*rgb, 1)
    bsdf.inputs["Emission Strength"].default_value = strength
    return m


class Mats:
    def __init__(self):
        self.asphalt = material("RaceAsphalt", ASPHALT, roughness=0.85)
        self.deck = material("RaceConcrete", (0.32, 0.33, 0.35), roughness=0.9)
        self.barrier = material("RaceBarrier", (0.75, 0.77, 0.8), roughness=0.4, metallic=0.3)
        self.kerb_red = material("RaceKerbRed", (0.75, 0.05, 0.04), roughness=0.6)
        self.kerb_white = material("RaceKerbWhite", (0.85, 0.85, 0.85), roughness=0.6)
        self.ground = material("RaceGround", (0.26, 0.34, 0.2), roughness=1.0)
        self.dark = material("RaceGraphite", (0.035, 0.04, 0.05), roughness=0.4, metallic=0.6)
        self.stone = material("RaceStone", (0.22, 0.21, 0.24), roughness=0.9)
        self.seats = material("RaceSeats", (0.05, 0.22, 0.3), roughness=0.8)
        self.check_black = material("RaceCheckBlack", (0.02, 0.02, 0.02), roughness=0.7)
        self.check_white = material("RaceCheckWhite", (0.9, 0.9, 0.9), roughness=0.7)
        self.cyan = glow("RaceGlowCyan", CYAN, 6.0)
        self.magenta = glow("RaceGlowMagenta", MAGENTA, 6.0)
        self.white = glow("RaceGlowWhite", WHITE, 4.0)


class Builder:
    """Faces gathered into one mesh object, each with its material."""

    def __init__(self, name):
        self.name = name
        self.bm = bmesh.new()
        self.mats = []

    def mat(self, m):
        if m not in self.mats:
            self.mats.append(m)
        return self.mats.index(m)

    def face(self, pts, m):
        f = self.bm.faces.new([self.bm.verts.new(p) for p in pts])
        f.material_index = self.mat(m)
        return f

    def quad_strip(self, a0, a1, b0, b1, m):
        """The quad from a0-a1 (this section) to b0-b1 (the next), facing whichever way round
        it comes: normals are put right in `finish`."""
        self.face([a0, a1, b1, b0], m)

    def box(self, x0, x1, y0, y1, z0, z1, m, top=None):
        x0, x1 = sorted((x0, x1))
        y0, y1 = sorted((y0, y1))
        z0, z1 = sorted((z0, z1))
        v = [Vector((x, y, z)) for z in (z0, z1) for y in (y0, y1) for x in (x0, x1)]
        for idx, fm in (((0, 1, 3, 2), m), ((4, 6, 7, 5), top or m), ((0, 4, 5, 1), m),
                        ((2, 3, 7, 6), m), ((0, 2, 6, 4), m), ((1, 5, 7, 3), m)):
            self.face([v[i] for i in idx], fm)

    def oriented_box(self, centre, along, across, up, half, m):
        """A box round `centre`, `half` (along, across, up) each way along those axes."""
        c = Vector(centre)
        corners = [c + along * (sa * half[0]) + across * (sb * half[1]) + up * (sc * half[2])
                   for sc in (-1, 1) for sb in (-1, 1) for sa in (-1, 1)]
        for idx in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)):
            self.face([corners[i] for i in idx], m)

    def finish(self):
        bmesh.ops.remove_doubles(self.bm, verts=self.bm.verts, dist=1e-4)
        bmesh.ops.recalc_face_normals(self.bm, faces=self.bm.faces)
        mesh = bpy.data.meshes.new(self.name)
        self.bm.to_mesh(mesh)
        self.bm.free()
        for m in self.mats:
            mesh.materials.append(m)
        for poly in mesh.polygons:
            poly.use_smooth = False
        obj = bpy.data.objects.new(self.name, mesh)
        bpy.context.scene.collection.objects.link(obj)
        return obj


def catmull_rom(p0, p1, p2, p3, t):
    """Centripetal Catmull-Rom between p1 and p2: no loops or overshoots where points bunch up."""
    def tj(ti, a, b):
        return ti + max((b - a).length, 1e-4) ** 0.5
    t0 = 0.0
    t1 = tj(t0, p0, p1)
    t2 = tj(t1, p1, p2)
    t3 = tj(t2, p2, p3)
    t = t1 + (t2 - t1) * t
    a1 = p0 * ((t1 - t) / (t1 - t0)) + p1 * ((t - t0) / (t1 - t0))
    a2 = p1 * ((t2 - t) / (t2 - t1)) + p2 * ((t - t1) / (t2 - t1))
    a3 = p2 * ((t3 - t) / (t3 - t2)) + p3 * ((t - t2) / (t3 - t2))
    b1 = a1 * ((t2 - t) / (t2 - t0)) + a2 * ((t - t0) / (t2 - t0))
    b2 = a2 * ((t3 - t) / (t3 - t1)) + a3 * ((t - t1) / (t3 - t1))
    return b1 * ((t2 - t) / (t2 - t1)) + b2 * ((t - t1) / (t2 - t1))


def centre_line():
    """The road's middle, every STEP meters round, from the start line: (point, which waypoint
    along it is, as a fraction)."""
    pts = [Vector(w) for w in WAYPOINTS]
    n = len(pts)
    dense = []
    for i in range(n):
        p0, p1, p2, p3 = pts[(i - 1) % n], pts[i], pts[(i + 1) % n], pts[(i + 2) % n]
        for k in range(200):
            dense.append((catmull_rom(p0, p1, p2, p3, k / 200), i + k / 200))
    # Even steps along it.
    out = [dense[0]]
    run = 0.0
    for (a, fa), (b, fb) in zip(dense, dense[1:] + dense[:1]):
        seg = (b - a).length
        while run + seg >= STEP:
            t = (STEP - run) / seg
            a = a.lerp(b, t)
            fa = fa + (fb - fa if fb >= fa else fb + n - fa) * t
            seg = (b - a).length
            run = 0.0
            out.append((a.copy(), fa % n))
        run += seg
    if (out[-1][0] - out[0][0]).length < STEP * 0.5:
        out.pop()
    return out


def banking(points):
    """How far the road banks at each point, in radians, left turns positive (the right edge up):
    by how sharply it turns, for BANK_SPEED, smoothed along it."""
    n = len(points)
    raw = []
    for i in range(n):
        a, b, c = points[i - 2], points[i], points[(i + 2) % n]
        h0 = math.atan2(b.y - a.y, b.x - a.x)
        h1 = math.atan2(c.y - b.y, c.x - b.x)
        turn = (h1 - h0 + math.pi) % (2 * math.pi) - math.pi
        curvature = turn / max((c - a).xy.length / 2, 1e-3)
        raw.append(math.atan(BANK_SPEED ** 2 * curvature / 9.81) * 0.5)
    k = 8
    bank = []
    for i in range(n):
        avg = sum(raw[(i + j) % n] for j in range(-k, k + 1)) / (2 * k + 1)
        bank.append(max(-math.radians(BANK_MAX), min(math.radians(BANK_MAX), avg)))
    return bank


class Section:
    """The road across at one point along it."""

    def __init__(self, centre, ahead, bank, fraction):
        self.centre = centre
        flat = Vector((ahead.x, ahead.y, 0)).normalized()
        self.ahead = ahead.normalized()
        self.flat = flat
        self.right = Vector((flat.y, -flat.x, 0))
        self.bank = bank
        self.fraction = fraction
        # The road's edges, the right one up by the bank for a left turn: turned about the
        # inside edge, which stays at the middle line's height.
        rise = math.sin(bank) * ROAD / 2
        run = math.cos(bank) * ROAD / 2
        lift = Vector((0, 0, abs(rise)))
        self.left = centre - self.right * run - Vector((0, 0, rise)) + lift
        self.right_edge = centre + self.right * run + Vector((0, 0, rise)) + lift
        self.centre = (self.left + self.right_edge) / 2
        self.low = min(self.left.z, self.right_edge.z)
        # Which way is up off the road, banked.
        self.up = (Vector((0, 0, 1)) * math.cos(bank) - self.right * math.sin(bank)).normalized()

    def across(self, t, lift=0.0):
        """A point across the road from its left edge (0) to its right (1), `lift` up off it."""
        return self.left.lerp(self.right_edge, t) + self.up * lift


def sections():
    line = centre_line()
    points = [p for p, _ in line]
    bank = banking(points)
    n = len(points)
    out = []
    for i, (p, frac) in enumerate(line):
        ahead = points[(i + 1) % n] - points[i - 1]
        out.append(Section(p, ahead, bank[i], frac))
    return out


def in_tunnel(s):
    return TUNNEL[0] <= s.fraction <= TUNNEL[1]


def build_road(secs, m):
    road = Builder("Road")
    kerbs = Builder("Kerbs")
    n = len(secs)
    kerb_t = KERB[0] / ROAD
    run = 0.0
    for i in range(n):
        a, b = secs[i], secs[(i + 1) % n]
        # The top, between the kerbs; the kerbs, striped red and white; the underside and the
        # sides, a deck's depth down.
        road.quad_strip(a.across(kerb_t), a.across(1 - kerb_t), b.across(kerb_t), b.across(1 - kerb_t), m.asphalt)
        stripe = m.kerb_red if int(run / KERB[1]) % 2 == 0 else m.kerb_white
        run += (b.centre - a.centre).length
        for t0, t1 in ((0.0, kerb_t), (1 - kerb_t, 1.0)):
            kerbs.quad_strip(a.across(t0, 0.01), a.across(t1, 0.01), b.across(t0, 0.01), b.across(t1, 0.01), stripe)
        down = Vector((0, 0, -DECK))
        under = m.deck
        road.quad_strip(a.left + down, a.right_edge + down, b.left + down, b.right_edge + down, under)
        road.quad_strip(a.left, a.left + down, b.left, b.left + down, under)
        road.quad_strip(a.right_edge, a.right_edge + down, b.right_edge, b.right_edge + down, under)
        # Under the kerbs, so that nothing is seen through the seam between them and the top.
        road.quad_strip(a.across(0), a.across(kerb_t), b.across(0), b.across(kerb_t), m.asphalt)
        road.quad_strip(a.across(1 - kerb_t), a.across(1), b.across(1 - kerb_t), b.across(1), m.asphalt)
    return [road.finish(), kerbs.finish()]


def build_barriers(secs, m):
    walls = Builder("Barriers")
    lights = Builder("BarrierLights")
    n = len(secs)
    h, w = BARRIER
    for i in range(n):
        a, b = secs[i], secs[(i + 1) % n]
        for side, light in ((-1, m.cyan), (1, m.magenta)):
            def edge(s):
                return s.left if side < 0 else s.right_edge
            out_a, out_b = a.right * (side * w), b.right * (side * w)
            up = Vector((0, 0, h))
            # Inner face, top, outer face; and down the outside to the deck's underside.
            ea, eb = edge(a), edge(b)
            walls.quad_strip(ea, ea + up, eb, eb + up, m.barrier)
            walls.quad_strip(ea + up, ea + up + out_a, eb + up, eb + up + out_b, m.barrier)
            walls.quad_strip(ea + out_a + up, ea + out_a - Vector((0, 0, DECK)),
                             eb + out_b + up, eb + out_b - Vector((0, 0, DECK)), m.barrier)
            # A line of light along the top, inside edge.
            li = Vector((0, 0, 0.01))
            lights.quad_strip(ea + up + li, ea + up + li + out_a * 0.35, eb + up + li, eb + up + li + out_b * 0.35, light)
    return [walls.finish(), lights.finish()]


def build_pillars(secs, m):
    """Under the road wherever it is up off the ground, every PILLAR[1] meters - but none on the
    road below where the viaduct crosses it."""
    pillars = Builder("Pillars")
    n = len(secs)
    every = max(1, int(PILLAR[1] / STEP))
    for i in range(0, n, every):
        s = secs[i]
        bottom = s.low - DECK
        if bottom < 1.0:
            continue
        clear = True
        for j, o in enumerate(secs):
            if abs(j - i) < 20 or abs(j - i) > n - 20:
                continue
            if (o.centre.xy - s.centre.xy).length < ROAD / 2 + BARRIER[1] + PILLAR[0] + 1.0 and o.centre.z < bottom:
                clear = False
                break
        if not clear:
            continue
        half = PILLAR[0] / 2
        c = s.centre
        pillars.box(c.x - half, c.x + half, c.y - half, c.y + half, -0.05, bottom, m.deck)
        # A cap across the deck's width.
        pillars.oriented_box(Vector((c.x, c.y, bottom - 0.4)), s.flat, s.right, Vector((0, 0, 1)),
                             (half, ROAD / 2, 0.4), m.deck)
    return pillars.finish()


def build_tunnel(secs, m):
    tunnel = Builder("Tunnel")
    lamps = Builder("TunnelLamps")
    inside = [i for i, s in enumerate(secs) if in_tunnel(s)]
    if not inside:
        return []
    w = BARRIER[1]
    top = TUNNEL_HEIGHT
    for i in inside[:-1]:
        a, b = secs[i], secs[i + 1]
        la, lb = a.left - a.right * w, b.left - b.right * w
        ra, rb = a.right_edge + a.right * w, b.right_edge + b.right * w
        up = Vector((0, 0, top))
        # The walls up from the barriers' tops to the roof, the roof, and its top.
        tunnel.quad_strip(a.left + Vector((0, 0, BARRIER[0])), a.left + up, b.left + Vector((0, 0, BARRIER[0])), b.left + up, m.stone)
        tunnel.quad_strip(a.right_edge + Vector((0, 0, BARRIER[0])), a.right_edge + up,
                          b.right_edge + Vector((0, 0, BARRIER[0])), b.right_edge + up, m.stone)
        tunnel.quad_strip(la + up, ra + up, lb + up, rb + up, m.stone)
        tunnel.quad_strip(la + up + Vector((0, 0, 1.0)), ra + up + Vector((0, 0, 1.0)),
                          lb + up + Vector((0, 0, 1.0)), rb + up + Vector((0, 0, 1.0)), m.stone)
        tunnel.quad_strip(la + Vector((0, 0, BARRIER[0])), la + up + Vector((0, 0, 1.0)),
                          lb + Vector((0, 0, BARRIER[0])), lb + up + Vector((0, 0, 1.0)), m.stone)
        tunnel.quad_strip(ra + Vector((0, 0, BARRIER[0])), ra + up + Vector((0, 0, 1.0)),
                          rb + Vector((0, 0, BARRIER[0])), rb + up + Vector((0, 0, 1.0)), m.stone)
        # A strip of light down the middle of the ceiling: a thin bar a section long, closed all
        # round so that it faces down whichever way its faces come out.
        mid = (a.centre + b.centre) / 2 + up - Vector((0, 0, 0.08))
        ahead = b.centre - a.centre
        lamps.oriented_box(mid, ahead.normalized(), a.right, Vector((0, 0, 1)), (ahead.length / 2, 0.6, 0.06), m.white)
    # Portals: a frame round each mouth.
    for i in (inside[0], inside[-1]):
        s = secs[i]
        c = s.centre + Vector((0, 0, top + 0.5))
        tunnel.oriented_box(c, s.flat, s.right, Vector((0, 0, 1)), (0.8, ROAD / 2 + 1.5, 1.3), m.dark)
        for side in (-1, 1):
            p = s.centre + s.right * side * (ROAD / 2 + 1.0) + Vector((0, 0, top / 2))
            tunnel.oriented_box(p, s.flat, s.right, Vector((0, 0, 1)), (0.8, 0.9, top / 2 + 0.8), m.dark)
    return [tunnel.finish(), lamps.finish()]


def build_start(secs, m):
    """The checkered line across the road at the start, and the gantry over it."""
    start = Builder("StartLine")
    s = secs[0]
    rows, cols = 2, 16
    depth = 1.0
    for r in range(rows):
        for c in range(cols):
            t0, t1 = c / cols, (c + 1) / cols
            f0, f1 = (r - rows / 2) * depth, (r - rows / 2 + 1) * depth
            pts = [s.across(t0, 0.02) + s.flat * f0, s.across(t1, 0.02) + s.flat * f0,
                   s.across(t1, 0.02) + s.flat * f1, s.across(t0, 0.02) + s.flat * f1]
            start.face(pts, m.check_black if (r + c) % 2 else m.check_white)
    up = Vector((0, 0, 1))
    for side in (-1, 1):
        p = s.centre + s.right * side * (ROAD / 2 + 1.6) + up * 4.0
        start.oriented_box(p, s.flat, s.right, up, (0.5, 0.5, 4.0), m.dark)
    beam = s.centre + up * 8.4
    start.oriented_box(beam, s.flat, s.right, up, (0.6, ROAD / 2 + 2.1, 0.6), m.dark)
    start.oriented_box(beam - s.flat * 0.62, s.flat, s.right, up, (0.02, ROAD / 2, 0.25), m.white)
    return start.finish()


def far_from_track(secs, x, y, clearance):
    return all((Vector((x, y)) - s.centre.xy).length > ROAD / 2 + clearance for s in secs[::3])


def build_scenery(secs, m, bounds):
    rng = random.Random(7)
    scenery = Builder("Scenery")
    lamps = Builder("SceneryLamps")
    # The grandstand: down the straight's left side, outside the wall, seats rising away from it.
    s0 = secs[0]
    stand_y = s0.centre.y + ROAD / 2 + 6.0
    # Just past the start line, along the straight (east), clear of the viaduct.
    x0_, x1_ = s0.centre.x + 8.0, s0.centre.x + 8.0 + STAND_LENGTH
    for row in range(8):
        y0 = stand_y + row * 1.6
        scenery.box(x0_, x1_, y0, y0 + 1.6, 0.0, 0.8 + row * 0.9, m.seats if row % 2 else m.deck)
    scenery.box(x0_ - 2, x1_ + 2, stand_y + 12.8, stand_y + 14.0, 0.0, 12.0, m.dark)
    scenery.box(x0_ - 2, x1_ + 2, stand_y - 1.0, stand_y + 14.0, 12.0, 12.6, m.dark)
    lamps.box(x0_ - 1, x1_ + 1, stand_y - 0.9, stand_y - 0.6, 11.6, 11.9, m.white)
    # Light towers round the track.
    (x0, y0), (x1, y1) = bounds
    placed = 0
    tries = 0
    while placed < LIGHT_TOWERS and tries < 4000:
        tries += 1
        x, y = rng.uniform(x0, x1), rng.uniform(y0, y1)
        if not far_from_track(secs, x, y, 6.0) or far_from_track(secs, x, y, 18.0):
            continue
        scenery.box(x - 0.4, x + 0.4, y - 0.4, y + 0.4, 0.0, 22.0, m.dark)
        scenery.box(x - 1.6, x + 1.6, y - 0.6, y + 0.6, 22.0, 23.2, m.dark)
        lamps.box(x - 1.5, x + 1.5, y - 0.65, y - 0.6, 22.1, 23.1, m.white)
        placed += 1
    # Monoliths, near and far, to see the speed by.
    placed = 0
    tries = 0
    while placed < MONOLITHS and tries < 20000:
        tries += 1
        x = rng.uniform(x0 - GROUND_MARGIN * 0.7, x1 + GROUND_MARGIN * 0.7)
        y = rng.uniform(y0 - GROUND_MARGIN * 0.7, y1 + GROUND_MARGIN * 0.7)
        if not far_from_track(secs, x, y, 10.0):
            continue
        w, d, h = rng.uniform(2, 7), rng.uniform(2, 7), rng.uniform(4, 22)
        a = rng.uniform(0, math.pi)
        along = Vector((math.cos(a), math.sin(a), 0))
        across = Vector((-math.sin(a), math.cos(a), 0))
        scenery.oriented_box(Vector((x, y, h / 2 - 0.05)), along, across, Vector((0, 0, 1)), (w / 2, d / 2, h / 2), m.stone)
        if rng.random() < 0.3:
            light = m.cyan if rng.random() < 0.5 else m.magenta
            lamps.oriented_box(Vector((x, y, h + 0.05)), along, across, Vector((0, 0, 1)), (w / 2 * 0.8, d / 2 * 0.8, 0.06), light)
        placed += 1
    return [scenery.finish(), lamps.finish()]


def build_ground(m, bounds):
    (x0, y0), (x1, y1) = bounds
    g = Builder("Ground")
    mg = GROUND_MARGIN
    g.face([Vector((x0 - mg, y0 - mg, -0.05)), Vector((x1 + mg, y0 - mg, -0.05)),
            Vector((x1 + mg, y1 + mg, -0.05)), Vector((x0 - mg, y1 + mg, -0.05))], m.ground)
    return g.finish()


def marker(name, at, facing):
    """An empty where the game puts something, its +x turned `facing` radians round from +x."""
    empty = bpy.data.objects.new(name, None)
    empty.location = at
    empty.rotation_euler.z = facing
    bpy.context.scene.collection.objects.link(empty)
    return empty


def place_markers(secs):
    s0 = secs[0]
    facing = math.atan2(s0.flat.y, s0.flat.x)
    marker("race_start", s0.centre, facing)
    every = int(CHECKPOINT_EVERY / STEP)
    count = 0
    for i in range(every, len(secs) - every // 2, every):
        count += 1
        marker(f"checkpoint_{count}", secs[i].centre, math.atan2(secs[i].flat.y, secs[i].flat.x))
    # The grid: two by two behind the line, 8 m between rows, staggered.
    for k in range(8):
        row, col = k // 2, k % 2
        back = 10.0 + row * 8.0 + col * 4.0
        across = (-1 if col == 0 else 1) * ROAD / 4
        n = len(secs)
        s = secs[(-int(back / STEP)) % n]
        marker(f"race_grid_{k + 1}", s.centre + s.right * across, math.atan2(s.flat.y, s.flat.x))
    return count


def main():
    glb, out, track = get_args()
    # The layout's own measures in place of the Circuit's, for everything below to build with.
    globals().update(TRACKS[track])
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Mats()
    secs = sections()
    length = len(secs) * STEP
    xs = [s.centre.x for s in secs]
    ys = [s.centre.y for s in secs]
    bounds = ((min(xs), min(ys)), (max(xs), max(ys)))
    objs = []
    objs += build_road(secs, m)
    objs += build_barriers(secs, m)
    objs.append(build_pillars(secs, m))
    objs += build_tunnel(secs, m)
    objs.append(build_start(secs, m))
    objs += build_scenery(secs, m, bounds)
    objs.append(build_ground(m, bounds))
    checkpoints = place_markers(secs)
    highest = max(s.centre.z for s in secs)
    banked = max(abs(s.bank) for s in secs)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in objs)
    print(f"[race] {track}: {length:.0f} m round, {len(secs)} sections, {checkpoints} checkpoints, "
          f"up to {highest:.1f} m high, banked up to {math.degrees(banked):.1f} deg, {tris} tris")
    if out:
        os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
        bpy.ops.wm.save_as_mainfile(filepath=os.path.abspath(out))
    if glb:
        bpy.ops.export_scene.gltf(filepath=os.path.abspath(glb), export_format="GLB", export_extras=True,
                                  export_cameras=False, export_lights=False)
        print(f"[race] exported -> {glb}")


if __name__ == "__main__":
    main()
