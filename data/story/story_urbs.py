"""
Urbs: a whole city in the void, for a story mode - twenty times the ground of Nexus, every street
lined with buildings that differ as a real city's do, in the traced-light style of the other
cities (see data/arena/br_nexus.py, whose way of building this borrows).

A grid of fourteen by fourteen blocks, each 48 m square, between streets 16 m wide, on a platform 912 m
across - and a canal across its south. The blocks change from the middle out:

- Downtown, the four blocks round the middle: towers 70 to 150 m - stepped back as they rise,
  round, twisting, tapering, on podiums, twin towers joined by a skybridge, and wedges with a
  sloping face. Summa, the tallest at 185 m, takes a block of its own, over a plaza on its podium.
- Midtown, the ring round it: mid-rise towers, apartment blocks with balconies, courtyard blocks
  round a garden, buildings up on stilts, and shops along the street.
- The outer ring: low apartments, terraced housing climbed by stairs, rows of townhouses and
  shops.
- The suburbs, out to the edge: streets of townhouses and terraces, a few shops, yards with
  trees between them.
- South of the canal, the docks: warehouses, works with cooling towers and sawtooth roofs.

And places of their own: the central plaza with its monument, six parks, the market, the civic
hall under a dome behind its colonnade, the arena, and the elevated railway down the middle
avenue, with three stations on it - out in the north, in the middle and out in the south by the
canal - each climbed by stairs. Bridges cross the canal at every street, steps go
down to its water, and gates lead out over the void east and west, to pads with an obelisk each; in the south the middle
avenue goes on over a bridge to Caelum, and in the north to Curia.

Precincts, where security takes those it arrests: three blocks have one on a corner (precinct_<n>),
up on a podium climbed by a flight from the street - a hall with the desk (precinct_<n>_desk) and
the evidence locker (precinct_<n>_locker), a gate (precinct_<n>_gate) through to the
cell block - a firewall - a corridor where the guard keeps (precinct_<n>_guard), and the cells
along the back (cell_<n>_<c>, each behind a firewall, cell_wall_<n>_<c>, which drops). The first cell's back
wall has a loose panel (tunnel_panel_<n>, tunnel_<n> before it) onto a stair down into a tunnel
under the podium, out at the side (tunnel_<n>_exit). And each cell has a drain in its floor, under a
grate (cell_grate_<n>_<c>, grate_<n>_<c> on it), down into the escape tunnels.

The escape tunnels, out of sight under it all (escape_tunnels): a den at their heart, where a gang keeps
(den_<k>) and a fence will wipe a record clean (fence), four ways out to a ring, and off the ring
twelve dead ends, each with a hatch up (hatch_<name>_below) - into each precinct's tunnel, each
bank's strongroom, and three manholes in the street (hatch_<name>, each); and a side room off each
way out with a cache of credits in it (cache_<k>, its box cache_box_<k>). The game takes the player
between a hatch and its other end as through a door.

Stores and banks to rob: some of the shops are stores to walk into, with shelves, a counter and a
computer at the back keeping the takings (store_<n>); and six blocks have a bank on a corner, a hall
behind a colonnade with a gold credit sign, a tellers' counter, four computers (bank_<n>_vault,
which opens the vault, and bank_<n>_2 to _4) and at the back the vault: a strongroom of gold behind
a door that swings open (vault_door_<n>, with vault_<n> in the middle of the strongroom). They have
a stream of chance of their own.

Office towers to go up: some of the towers are open to walk into (see `office_tower`), a lobby
off the street and open floors over it, their windows open to the city over sills too high to jump,
and a lift up the back to every floor and the roof (lift_<n>, its car; lift_<n>_<k> where it stops
at floor k; lift_<n>_gate_<k> the gate before the shaft at each floor, open while the car stands
there). They have a stream of chance of their own, and each is built where a tower would have
stood, which is built out of sight, so the rest of the city is as it was.

Windows are lit or dark at random, warm in homes and cool in offices. Streets have dashed lines,
crossings at every junction, lamps, trees and parked light cycles. Every building with corners has
them lit, in its own colour (a bank's in gold). Every coloured line of light
near the ground glows and is written to story_urbs.glow.json for the game to light with; those
higher up only glow. Where the roads and their crossings are is written to story_urbs.roads.json,
for the game to keep its droids on the pavements but to cross.

Builds data/story/story_urbs.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/story/story_urbs.py -- --seed 7 --out ~/Documents/blender/story/story_urbs.blend --glb data/story/story_urbs.glb

The geometry is gathered into one mesh per 64 m tile and kind (ground, buildings, cover, lines of
light), rather than an object for each piece, which a city this size would have tens of
thousands of; each lamp's glass stays a piece of its own, as the game finds lamps by it.

Stairs are of 0.25 m steps, 0.35 m deep. Empties named place_* mark the city's landmarks for a
story to use: place_summa, place_plaza, place_station, place_station_middle, place_station_south, place_park_west, place_park_east,
place_park_north, place_park_far_west, place_park_far_east, place_park_far_north, place_market, place_hall, place_arena, place_docks,
place_gate_east and place_gate_west.

Made for Ruptura Systematis (MazeGame/maze).
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "arena"))
import br_town  # noqa: E402
from br_grid import material, glow, PALETTE  # noqa: E402
from br_nexus import Nexus, Frame, WORLD, octagon, polygon, RISE, RUN, DECK  # noqa: E402

PITCH = 64.0            # from one street's middle to the next
STREET = 16.0           # a street's width, kerb to kerb
BLOCKS = 14             # blocks each way
SIDEWALK = 3.0          # round the edge of each block
KERB = 0.12             # how high a block's pavement stands: less than a step
HALF = BLOCKS / 2 * PITCH + STREET / 2      # from the middle to the platform's edge: 456 m
DEPTH = 3.0
CANAL = (-320.0, 10.0, 1.0)   # the canal: its middle (y), its width, and how far down its water is
BRIDGE_WIDTH = 12.0
RAIL = (9.5, 1.2, 1.4)        # the railway's beam: the top of it, how wide, how deep
STATION = (332.0, 372.0, 8.0)  # the north station: from and to (y), and its platforms' height
# The stations along the railway, each between two cross streets: from and to (y), which end its
# stairs go down from (1 its north end, -1 its south), and its marker.
TRAIN_HALF = 2.85   # how far out from the middle the train's cars reach, with room to spare
STATIONS = (
    (STATION[0], STATION[1], 1, "place_station"),
    (12.0, 52.0, 1, "place_station_middle"),
    (-372.0, -332.0, -1, "place_station_south"),
)
GATE = (22.0, 6.0, 24.0)      # the gates' bridges: how long and wide, and their pads' size
TILE = 128.0                  # the meshes are gathered by tiles of this size: big, as the game pays by the mesh
FAR_TILE = 256.0              # and again, simplified, by blocks of this size, for the game to show far off
# Far off, what is too small to make out goes: pieces of a collection smaller than this across,
# in meters, and these collections and materials altogether.
FAR_LEAST = {"Structures": 2.0, "Trim": 6.0, "Ground": 0.0}
FAR_DROPS = ("Marking",)
# The lit windows, which are the city at night from near and far alike, gathered by block apart
# from the rest, for the game to show whatever the distance.
WINDOWS = "Window"
LIGHT_BELOW = 10.0            # lines of light lower than this light their surroundings in the game
TRIM = 0.12
WALL = 0.3                    # a store's or a bank's walls, inside
STORE_SIZE = (7.0, 9.0)       # the least a shop needs, along the street and in, to be a store to walk into
STORE_CHANCE = 0.07           # the chance a shop that big is one
STORE_HEIGHT = 4.0            # a store's ceiling
STORE_DOOR = (2.4, 2.9)       # its doorway, wide and high
BANKS = ((6, 4), (9, 6), (4, 9), (7, 10), (11, 8), (2, 4))   # the blocks with a bank on a corner
BANK_LOT = 12.5               # the least a lot's narrower side can be, for a bank
BANK_SIZE = (22.0, 18.0)      # its hall, along the street and in, at most
BANK_FORECOURT = 3.5          # how far back from the street, behind its colonnade
BANK_HEIGHT = 6.5             # its hall's ceiling
BANK_DOOR = (3.2, 4.2)
BANK_DEEP = 14.0              # how deep a bank would be, to have room for its vault behind its hall
VAULT = 3.6                   # the vault, from its wall to the back
VAULT_DOOR = (1.8, 2.6)       # its doorway, wide and high
PRECINCTS = ((4, 6), (9, 8), (7, 4))   # the blocks with a security precinct on a corner
PRECINCT_SIZE = (20.0, 13.6)  # its building, along the street at most, and in
PRECINCT_PODIUM = 3.0         # how high it stands, over the tunnel under it
PRECINCT_HIGH = 3.4           # its rooms, floor to ceiling
CELL = 3.0                    # a cell, along the block
UNDER = -120.0                # the escape tunnels' floor, hung in the void well under the city
MANHOLES = ((3, 3), (10, 10), (11, 2))   # the blocks with a manhole down to them, on their pavement
PRECINCT_LANDING = 2.0        # the landing before its door, out from it, up at the podium's height
OFFICE_CHANCE = 0.5           # the chance a tower in midtown or downtown is one to go up
OFFICE_SIZE = (14.0, 26.0)    # the least a lot needs to be across for one, and the most it is across
OFFICE_FLOORS = (6, 12)       # how many floors it has, the lobby one of them; its roof is a stop too
OFFICE_FLOOR = 4.0            # from one floor to the next
OFFICE_SLAB = 0.3             # a floor's thickness
OFFICE_SILL = 1.15            # its windows' sills, over the floor: higher than a jump
OFFICE_HEAD = 3.2             # and their heads
OFFICE_PIER = 0.8             # the piers between its windows
OFFICE_BAY = 4.6              # a window and a pier, about
OFFICE_DOOR = (2.4, 2.9)      # its doorway off the street
SHAFT = 2.8                   # the lift's shaft, inside, either way
SHAFT_DOOR = (1.6, 2.6)       # the doorway into it at each floor
CAR = 2.7                     # the car, either way, and how high its walls are
CAR_HIGH = 2.5
ROOF_HOUSE = 3.2              # how high the shaft stands over the roof, for the lift's last stop


def rect(x0, x1, y0, y1):
    return [Vector((x0, y0, 0)), Vector((x1, y0, 0)), Vector((x1, y1, 0)), Vector((x0, y1, 0))]


def middle(poly):
    return sum(poly, Vector()) / len(poly)


def shrink(poly, by):
    """`poly` drawn in towards its middle by `by` at each corner."""
    c = middle(poly)
    return [p + (c - p).normalized() * by for p in poly]


def turned(poly, angle):
    c = middle(poly)
    out = []
    for p in poly:
        d = p - c
        cos, sin = math.cos(angle), math.sin(angle)
        out.append(Vector((c.x + d.x * cos - d.y * sin, c.y + d.x * sin + d.y * cos, 0)))
    return out


def scaled(poly, k):
    c = middle(poly)
    return [c + (p - c) * k for p in poly]


Z = Vector((0, 0, 1))


def far_off(verts, col, mat):
    """Whether a piece of `col` in `mat` with its corners at `verts` is still seen far off."""
    if col not in FAR_LEAST or mat.startswith(FAR_DROPS):
        return False
    lo = Vector((min(v[0] for v in verts), min(v[1] for v in verts), min(v[2] for v in verts)))
    hi = Vector((max(v[0] for v in verts), max(v[1] for v in verts), max(v[2] for v in verts)))
    return (hi - lo).length >= FAR_LEAST[col]


class Urbs(Nexus):
    """The city's materials, and its geometry gathered by tile as it is made."""

    def __init__(self, paint):
        super().__init__(paint)
        self.road = material("Road", (0.02, 0.022, 0.03), roughness=0.3, metallic=0.4)
        self.pavement = material("Pavement", (0.11, 0.11, 0.12), roughness=0.8, metallic=0.0)
        self.grass = material("Grass", (0.03, 0.08, 0.035), roughness=0.9, metallic=0.0)
        self.water = material("Water", (0.01, 0.04, 0.06), roughness=0.05, metallic=0.8)
        self.bark = material("Bark", (0.07, 0.05, 0.035), roughness=0.9, metallic=0.0)
        self.leaves = material("Leaves", (0.04, 0.12, 0.06), roughness=0.7, metallic=0.0)
        self.facades = {
            "glass": material("GlassFacade", (0.04, 0.06, 0.09), roughness=0.1, metallic=0.8),
            "teal": material("TealGlass", (0.03, 0.09, 0.1), roughness=0.1, metallic=0.7),
            "bronze": material("Bronze", (0.18, 0.11, 0.06), roughness=0.3, metallic=0.9),
            "slate": material("Slate", (0.09, 0.1, 0.12), roughness=0.5, metallic=0.3),
            "white": material("WhitePanel", (0.5, 0.52, 0.55), roughness=0.4, metallic=0.1),
            "concrete": material("Concrete", (0.32, 0.31, 0.29), roughness=0.85, metallic=0.0),
            "stone": material("Stone", (0.28, 0.22, 0.17), roughness=0.7, metallic=0.0),
            "brick": material("Brick", (0.25, 0.09, 0.06), roughness=0.9, metallic=0.0),
            "sand": material("Sand", (0.38, 0.32, 0.22), roughness=0.8, metallic=0.0),
            "rust": material("Rust", (0.2, 0.08, 0.03), roughness=0.8, metallic=0.5),
        }
        self.windows = {
            "warm": glow("WindowWarm", (1.0, 0.72, 0.38), 1.4),
            "cool": glow("WindowCool", (0.75, 0.88, 1.0), 1.4),
            "office": glow("WindowOffice", (0.88, 0.95, 1.0), 1.1),
        }
        self.marking = glow("Marking", (0.8, 0.82, 0.85), 0.6)
        self.buckets = {}      # (collection, tile) -> [verts, faces, material per face]
        self.far = {}          # (collection, block) -> the same, of what is seen far off
        self.n_buildings = 0
        # The stores and banks have a stream of chance of their own, so the city is the same with them.
        self.trade = random.Random(0)
        self.n_stores = self.n_banks = 0
        self.bank_here = False
        self.precinct_here = False
        self.n_precincts = 0
        # The office towers to go up have a stream of chance of their own, and so do their lifts'
        # names, each city's of its own, as the cities are played together.
        self.lifts = random.Random(1)
        self.n_lifts = 0
        self.city = "u"
        # Where this city's own coordinates are in the world (Ferrum's and Caelum's are moved off).
        self.offset = Vector((0, 0, 0))
        # While a piece of its own is being made, its geometry, rather than gathered by tile.
        self.apart = None

    # The geometry, gathered by tile.

    def mesh(self, verts, faces, col, mat, cover="none"):
        if self.apart is not None:
            self.apart.append((verts, faces, mat))
            return None
        if col is self.lights:
            return super().mesh(verts, faces, col, mat, cover)
        c = sum((Vector(v) for v in verts), Vector()) / len(verts)
        tile = (math.floor(c.x / TILE), math.floor(c.y / TILE))
        block = (math.floor(c.x / FAR_TILE), math.floor(c.y / FAR_TILE))
        if mat.name.startswith(WINDOWS):
            buckets = [self.buckets.setdefault(("Windows", block), ([], [], []))]
        else:
            buckets = [self.buckets.setdefault((col.name, tile), ([], [], []))]
            if far_off(verts, col.name, mat.name):
                buckets.append(self.far.setdefault((col.name, block), ([], [], [])))
        for bucket in buckets:
            base = len(bucket[0])
            bucket[0].extend(tuple(v) for v in verts)
            bucket[1].extend(tuple(base + i for i in f) for f in faces)
            bucket[2].extend([mat] * len(faces))
        return None

    def build_buckets(self):
        """A mesh for each tile of each collection, its pieces' faces turned outward, and for each
        block of the lit windows; and one for each block of each collection, of what is seen far
        off, simplified - named Far<collection>_<x>_<y> for the game to show in place of the
        tiles in the block once it is far enough away."""
        cols = {c.name: c for c in (self.ground, self.struct, self.cover, self.trim)}
        cols["Windows"] = self.trim
        far = {(f"Far{name}", block): bucket for (name, block), bucket in self.far.items()}
        for (name, (tx, ty)), (verts, faces, mats) in sorted(self.buckets.items()) + sorted(far.items()):
            mesh = bpy.data.meshes.new(f"{name}_{tx}_{ty}")
            mesh.from_pydata(verts, [], faces)
            slots = {}
            for mat in mats:
                if id(mat) not in slots:
                    slots[id(mat)] = len(slots)
                    mesh.materials.append(mat)
            for poly, mat in zip(mesh.polygons, mats):
                poly.material_index = slots[id(mat)]
            bm = bmesh.new()
            bm.from_mesh(mesh)
            if name.startswith("Far"):
                # Decimated: flat faces next to each other made one, so long as they are the same
                # material.
                bmesh.ops.dissolve_limit(bm, angle_limit=math.radians(1.0), verts=bm.verts[:], edges=bm.edges[:],
                                         delimit={'MATERIAL'})
            bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
            bm.to_mesh(mesh)
            bm.free()
            mesh.update()
            o = bpy.data.objects.new(f"{name}_{tx}_{ty}", mesh)
            cols[name.removeprefix("Far")].objects.link(o)
        print(f"[map] {len(self.buckets)} meshes, {len(self.far)} far off")

    def line(self, p, q, mat, width=TRIM, height=None, across=None, light=True):
        # Only lines near the ground light their surroundings: those up a tower would light
        # nobody, and there would be tens of thousands.
        light = light and min(p.z, q.z) < LIGHT_BELOW
        before = len(self.pieces)
        super().line(p, q, mat, width, height, across, light)
        # Of a line up a tower, only the pieces near the ground. (The game's y is up.)
        self.pieces[before:] = [piece for piece in self.pieces[before:] if piece["corner"][1] < LIGHT_BELOW]

    def piece(self, name, origin, build):
        """Whatever `build` makes, as an object of its own named `name`, its origin at `origin`:
        for the game to move, or to take out of the level's shape."""
        self.apart = []
        build()
        parts, self.apart = self.apart, None
        o = origin + self.offset
        verts, faces, mats = [], [], []
        for vs, fs, mat in parts:
            base = len(verts)
            verts.extend(tuple(Vector(v) - o) for v in vs)
            faces.extend(tuple(base + i for i in f) for f in fs)
            mats.extend([mat] * len(fs))
        mesh = bpy.data.meshes.new(name)
        mesh.from_pydata(verts, [], faces)
        slots = {}
        for mat in mats:
            if id(mat) not in slots:
                slots[id(mat)] = len(slots)
                mesh.materials.append(mat)
        for poly, mat in zip(mesh.polygons, mats):
            poly.material_index = slots[id(mat)]
        bm = bmesh.new()
        bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        bm.to_mesh(mesh)
        bm.free()
        mesh.update()
        obj = bpy.data.objects.new(name, mesh)
        obj.location = o
        self.struct.objects.link(obj)
        return obj

    def unseen(self, build):
        """Runs `build` for its draws of chance only: what it makes is thrown away."""
        pieces, lights = len(self.pieces), self.n_lights
        self.apart = []
        build()
        self.apart = None
        del self.pieces[pieces:]
        self.n_lights = lights

    def mark(self, name, at, facing=0.0):
        """An empty for the game, at `at` in this city's own coordinates."""
        br_town.marker(name, at.x + self.offset.x, at.y + self.offset.y, at.z, facing)

    def parapet(self, a, b, z, mat, gaps=()):
        # A gap all but at an end goes to the end, leaving no stub of parapet.
        length = (b - a).length
        ends = 0.6 / length
        gaps = [(0.0 if g0 < ends else g0, 1.0 if g1 > 1.0 - ends else g1) for g0, g1 in gaps]
        super().parapet(a, b, z, mat, gaps)

    # Pieces of buildings.

    def prism(self, poly, z0, z1, mat, top=None, col=None):
        self.solid(poly, z0, z1, col or self.struct, mat, top=top)

    def facade_windows(self, poly, z0, z1, floor, kind, rng, lit=0.5, bands=False, faces=None):
        """Windows on the faces of `poly` from z0 to z1, a floor every `floor` m: a strip along
        each floor if `bands`, otherwise windows of their own, each lit or not at random."""
        c = middle(poly)
        n = len(poly)
        floors = int((z1 - z0) / floor)
        warm, cool = self.windows["warm"], self.windows[kind]
        for i in range(n):
            if faces is not None and i not in faces:
                continue
            a, b = poly[i], poly[(i + 1) % n]
            e = b - a
            length = e.length
            if length < 2.0:
                continue
            u = e / length
            out = Vector((u.y, -u.x, 0))
            if out.dot((a + b) / 2 - c) < 0:
                out = -out
            out *= 0.05
            h = floor * 0.45
            for f in range(floors):
                z = z0 + f * floor + floor * 0.55
                if bands:
                    # Strips broken where a light is off.
                    t = 0.6
                    while t < length - 0.6:
                        run = min(rng.uniform(3.0, 9.0), length - 0.6 - t)
                        if rng.random() < lit:
                            p = a + u * t + out
                            self.line(Vector((p.x, p.y, z)), Vector((p.x, p.y, z)) + u * run, cool,
                                      width=h, height=0.06, across=Z, light=False)
                        t += run + 0.4
                else:
                    count = int((length - 1.0) / 2.6)
                    if count < 1:
                        continue
                    gap = (length - count * 1.4) / (count + 1)
                    for w in range(count):
                        if rng.random() >= lit:
                            continue
                        p = a + u * (gap + w * (1.4 + gap)) + out
                        mat = warm if kind == "warm" or rng.random() < 0.3 else cool
                        self.line(Vector((p.x, p.y, z)), Vector((p.x, p.y, z)) + u * 1.4, mat,
                                  width=h * 1.3, height=0.06, across=Z, light=False)

    def mast(self, x, y, z, height, shade):
        """An antenna: a mast, a light on its tip."""
        self.box(WORLD, x - 0.25, x + 0.25, y - 0.25, y + 0.25, z, z + height, self.struct, self.rail)
        self.line(Vector((x, y, z + height)), Vector((x, y, z + height + 1.2)), self.lines[shade], width=0.3)

    def roof_clutter(self, poly, z, rng):
        """Plant on a roof: boxes, a tank, a mast perhaps."""
        c = middle(poly)
        span = min((p - c).length for p in poly) * 0.55
        for _ in range(rng.randint(1, 4)):
            x, y = c.x + rng.uniform(-span, span), c.y + rng.uniform(-span, span)
            w, d, h = rng.uniform(1.5, 4), rng.uniform(1.5, 4), rng.uniform(1.2, 3)
            self.box(WORLD, x - w / 2, x + w / 2, y - d / 2, y + d / 2, z, z + h, self.struct, self.facades["slate"])
        if rng.random() < 0.4:
            self.prism(polygon(1.4, 10, 0, c.x + span * 0.5, c.y - span * 0.5), z, z + 3.5, self.facades["rust"])

    def helipad(self, poly, z, shade):
        c = middle(poly)
        r = min((p - c).length for p in poly) * 0.6
        self.ring(polygon(r, 16, 0, c.x, c.y), z + 0.03, self.lines[shade], out=0.0, width=0.2)
        for dx in (-r * 0.3, r * 0.3):
            self.line(Vector((c.x + dx, c.y - r * 0.4, z + 0.03)), Vector((c.x + dx, c.y + r * 0.4, z + 0.03)),
                      self.white, width=0.3, height=0.04, light=False)
        self.line(Vector((c.x - r * 0.3, c.y, z + 0.03)), Vector((c.x + r * 0.3, c.y, z + 0.03)),
                  self.white, width=0.3, height=0.04, light=False)

    def tree(self, x, y, rng, z=KERB, size=1.0):
        """A tree: a trunk, and a canopy of two tiers, high enough to walk under."""
        self.prism(polygon(0.22 * size, 6, 0, x, y), z, z + 3.2 * size, self.bark, col=self.cover)
        r = rng.uniform(1.4, 2.0) * size
        lo, hi = z + 2.6 * size, z + 3.8 * size
        self.prism(polygon(r, 8, rng.uniform(0, 1), x, y), lo, hi, self.leaves, top=polygon(r * 0.8, 8, 0.4, x, y))
        self.prism(polygon(r * 0.75, 8, 0.2, x, y), hi, hi + 1.4 * size, self.leaves, top=polygon(r * 0.25, 8, 0.6, x, y))
        if rng.random() < 0.35:
            self.ring(polygon(r * 0.95, 8, 0, x, y), lo + 0.1, self.lines["green"], out=0.0, width=0.06, light=False)


# Buildings. Each stands on `poly`, a rectangle within its lot, and is in `district`.

DISTRICT_FACADES = {
    "downtown": ("glass", "glass", "teal", "bronze", "slate", "white"),
    "midtown": ("concrete", "white", "stone", "teal", "glass", "sand", "bronze"),
    "outer": ("brick", "sand", "concrete", "stone", "white", "slate"),
    "suburbs": ("brick", "brick", "sand", "white", "stone"),
    "docks": ("slate", "concrete", "rust", "brick"),
}


def facade(m, rng, district):
    return m.facades[rng.choice(DISTRICT_FACADES[district])]


def tall(rng, district):
    return {"downtown": rng.uniform(70, 150), "midtown": rng.uniform(28, 70),
            "outer": rng.uniform(12, 26), "suburbs": rng.uniform(7, 13), "docks": rng.uniform(8, 16)}[district]


def setback_tower(m, rng, poly, district, height=None):
    """An office tower stepped back as it rises, lit up its corners, a ring at each step."""
    height = height or tall(rng, district)
    mat, shade = facade(m, rng, district), m.shade()
    tiers = rng.randint(2, 4)
    z = 0.0
    p = poly
    for t in range(tiers):
        top = z + height * (0.55 if t == 0 else 0.45 / (tiers - 1))
        m.prism(p, z, top, mat)
        m.facade_windows(p, z + 0.5, top - 0.5, 4.0, "office", rng, lit=0.75, bands=True)
        m.fins(p, p, z, top, m.lines[shade])
        m.ring(p, top - 0.05, m.lines[shade])
        z = top
        inner = shrink(p, rng.uniform(2.5, 5.0))
        if min((q - middle(inner)).length for q in inner) < 4.0:
            break
        p = inner
    if rng.random() < 0.5:
        m.helipad(p, z, shade)
    else:
        c = middle(p)
        m.mast(c.x, c.y, z, rng.uniform(8, 20), shade)
    m.roof_clutter(p, z, rng)


def round_tower(m, rng, poly, district, height=None):
    """A round tower, perhaps narrowing as it rises, its floors in bands of light."""
    c = middle(poly)
    r = min(abs(poly[1].x - poly[0].x), abs(poly[2].y - poly[1].y)) / 2
    height = height or tall(rng, district)
    taper = rng.choice((1.0, 1.0, 0.8, 0.65))
    sides = 20
    mat, shade = facade(m, rng, district), m.shade()
    storeys = 6
    for k in range(storeys):
        z0, z1 = height * k / storeys, height * (k + 1) / storeys
        r0 = r * (1 - (1 - taper) * k / storeys)
        r1 = r * (1 - (1 - taper) * (k + 1) / storeys)
        lo, hi = polygon(r0, sides, 0, c.x, c.y), polygon(r1, sides, 0, c.x, c.y)
        m.prism(lo, z0, z1, mat, top=hi)
        m.facade_windows(hi, z0 + 0.5, z1 - 0.5, 3.8, "office", rng, lit=0.7, bands=True)
    top = polygon(r * taper, sides, 0, c.x, c.y)
    m.ring(top, height - 0.05, m.lines[shade])
    m.ring(polygon(r, sides, 0, c.x, c.y), 4.0, m.lines[shade])
    # A crown: a narrower drum and a ring of light floating over it.
    crown = polygon(r * taper * 0.6, sides, 0, c.x, c.y)
    m.prism(crown, height, height + 5.0, m.facades["slate"])
    m.ring(polygon(r * taper * 0.8, sides, 0, c.x, c.y), height + 7.0, m.lines[shade], out=0.0, width=0.25)


def twist_tower(m, rng, poly, district, height=None):
    """A square tower turning a quarter as it rises, a slab a floor."""
    c = middle(poly)
    side = min(abs(poly[1].x - poly[0].x), abs(poly[2].y - poly[1].y)) * 0.72
    height = height or tall(rng, district)
    floor = 4.0
    n = int(height / floor)
    turn_all = rng.choice((-1, 1)) * math.pi / 2
    mat, shade = facade(m, rng, district), m.shade()
    base = rect(c.x - side / 2, c.x + side / 2, c.y - side / 2, c.y + side / 2)
    for k in range(n):
        p = turned(base, turn_all * k / n)
        m.prism(p, k * floor, (k + 1) * floor - 0.6, mat)
        m.prism(shrink(p, 0.6), (k + 1) * floor - 0.6, (k + 1) * floor, m.facades["slate"])
        m.facade_windows(p, k * floor, (k + 1) * floor - 0.6, floor, "office", rng, lit=0.7, bands=True)
        m.fins(p, p, k * floor, (k + 1) * floor - 0.6, m.lines[shade])
        if k % 3 == 0:
            m.ring(p, (k + 1) * floor - 0.62, m.lines[shade], width=0.08)
    top = turned(base, turn_all)
    m.ring(top, n * floor, m.lines[shade])
    m.roof_clutter(top, n * floor, rng)


def tapered_tower(m, rng, poly, district, height=None):
    """A tower narrowing to a point of light: six-sided, or three."""
    c = middle(poly)
    r = min(abs(poly[1].x - poly[0].x), abs(poly[2].y - poly[1].y)) / 2
    sides = rng.choice((3, 6, 6, 8))
    if sides == 3:
        r *= 1.05
    height = height or tall(rng, district)
    rot = rng.uniform(0, math.pi)
    base, top = polygon(r, sides, rot, c.x, c.y), polygon(r * 0.3, sides, rot, c.x, c.y)
    mat, shade = facade(m, rng, district), m.shade()
    m.prism(base, 0.0, height, mat, top=top)
    m.fins(base, top, 0.0, height, m.lines[shade])
    for k in range(8):
        za, zb = height * k / 8, height * (k + 1) / 8
        p = [q + (t - q) * (zb / height) for q, t in zip(base, top)]
        m.facade_windows(p, za, zb, 4.0, "office", rng, lit=0.6, bands=True)
    for k in range(1, 8):
        z = height * k / 8
        p = [b + (t - b) * (z / height) for b, t in zip(base, top)]
        m.ring(p, z, m.lines[shade] if k % 2 else m.windows["office"], light=k % 2 == 1)
    m.mast(c.x, c.y, height, rng.uniform(6, 14), shade)


def wedge_tower(m, rng, poly, district, height=None):
    """A building with one face sloping back from the street the whole way up."""
    height = height or tall(rng, district) * 0.8
    side = rng.randrange(4)
    top = [Vector(p) for p in poly]
    # Pull the top edge on one side back.
    a, b = side, (side + 1) % 4
    inward = (middle(poly) - (poly[a] + poly[b]) / 2) * rng.uniform(0.9, 1.4)
    top[a] += inward
    top[b] += inward
    mat, shade = facade(m, rng, district), m.shade()
    m.prism(poly, 0.0, height, mat, top=top)
    m.facade_windows(poly, 0.5, height * 0.95, 4.0, "office", rng, lit=0.6, bands=True,
                     faces=[(side + 2) % 4])
    # Lines up the slope, and round the top.
    for i in (a, b):
        m.line(Vector((poly[i].x, poly[i].y, 0.0)), Vector((top[i].x, top[i].y, height)), m.lines[shade])
    for i in ((side + 2) % 4, (side + 3) % 4):
        m.line(Vector((poly[i].x, poly[i].y, 0.0)), Vector((poly[i].x, poly[i].y, height)), m.lines[shade])
    m.ring(top, height, m.lines[shade])
    for k in range(1, int(height / 8)):
        z = k * 8.0
        f = z / height
        p = [q + (t - q) * f for q, t in zip(poly, top)]
        m.line(Vector((p[a].x, p[a].y, z)), Vector((p[b].x, p[b].y, z)), m.windows["office"], width=0.5,
               light=False)


def podium_tower(m, rng, poly, district):
    """A podium of shops filling the lot, and a tower on it."""
    h = rng.uniform(8.0, 14.0)
    mat, shade = facade(m, rng, district), m.shade()
    m.prism(poly, 0.0, h, mat)
    m.facade_windows(poly, 0.0, 4.0, 4.0, "warm", rng, lit=0.9, bands=True)
    m.facade_windows(poly, 4.0, h, 4.0, "office", rng, lit=0.5, bands=True)
    m.ring(poly, h - 0.05, m.lines[shade])
    m.ring(poly, 4.2, m.lines[shade], width=0.08)
    m.fins(poly, poly, 0.0, h, m.lines[shade])
    c = middle(poly)
    w, d = abs(poly[1].x - poly[0].x), abs(poly[2].y - poly[1].y)
    k = rng.uniform(0.5, 0.7)
    ox, oy = rng.uniform(-1, 1) * w * (1 - k) / 2, rng.uniform(-1, 1) * d * (1 - k) / 2
    tw, td = w * k / 2, d * k / 2
    upper = rect(c.x + ox - tw, c.x + ox + tw, c.y + oy - td, c.y + oy + td)
    height = tall(rng, district)
    kind = rng.choice((setback_tower, round_tower, twist_tower, tapered_tower))
    # The tower's own builder stands on the ground; it is built up from the podium's top.
    lift(m, h, lambda: kind(m, rng, upper, district, height=height))


def lift(m, dz, build):
    """Builds with `build`, all of it raised `dz`."""
    solid, line = m.solid, m.line

    def raised_solid(base, z0, z1, col, mat, top=None, cover="none"):
        return solid(base, z0 + dz, z1 + dz, col, mat, top=top, cover=cover)

    def raised_line(p, q, mat, width=TRIM, height=None, across=None, light=True):
        return line(p + Vector((0, 0, dz)), q + Vector((0, 0, dz)), mat, width, height, across, light)

    m.solid, m.line = raised_solid, raised_line
    try:
        build()
    finally:
        m.solid, m.line = solid, line


def twin_towers(m, rng, poly, district):
    """Two slim towers, joined by a skybridge two thirds of the way up."""
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    along_x = (x1 - x0) >= (y1 - y0)
    height = tall(rng, district)
    mat, shade = facade(m, rng, district), m.shade()
    if along_x:
        w = (x1 - x0) * 0.36
        a = rect(x0, x0 + w, y0 + 3, y1 - 3)
        b = rect(x1 - w, x1, y0 + 3, y1 - 3)
    else:
        w = (y1 - y0) * 0.36
        a = rect(x0 + 3, x1 - 3, y0, y0 + w)
        b = rect(x0 + 3, x1 - 3, y1 - w, y1)
    for p, h in ((a, height), (b, height * rng.uniform(0.85, 1.0))):
        m.prism(p, 0.0, h, mat)
        m.facade_windows(p, 0.5, h - 0.5, 4.0, "office", rng, lit=0.75, bands=True)
        m.fins(p, p, 0.0, h, m.lines[shade])
        m.ring(p, h, m.lines[shade])
        c = middle(p)
        m.mast(c.x, c.y, h, 10.0, shade)
    z = height * 0.66
    ca, cb = middle(a), middle(b)
    if along_x:
        m.box(WORLD, ca.x, cb.x, ca.y - 2.5, ca.y + 2.5, z, z + 4.5, m.struct, m.facades["glass"])
        for y in (ca.y - 2.55, ca.y + 2.55):
            m.line(Vector((ca.x, y, z)), Vector((cb.x, y, z)), m.lines[shade])
            m.line(Vector((ca.x, y, z + 2.2)), Vector((cb.x, y, z + 2.2)), m.windows["cool"], width=1.6,
                   height=0.06, across=Z, light=False)
    else:
        m.box(WORLD, ca.x - 2.5, ca.x + 2.5, ca.y, cb.y, z, z + 4.5, m.struct, m.facades["glass"])
        for x in (ca.x - 2.55, ca.x + 2.55):
            m.line(Vector((x, ca.y, z)), Vector((x, cb.y, z)), m.lines[shade])
            m.line(Vector((x, ca.y, z + 2.2)), Vector((x, cb.y, z + 2.2)), m.windows["cool"], width=1.6,
                   height=0.06, across=Z, light=False)


def apartments(m, rng, poly, district, street_faces):
    """An apartment block, its windows lit warm here and there, balconies on its street faces."""
    floor = 3.2
    floors = max(3, int(tall(rng, district) / floor))
    height = floors * floor
    mat, shade = facade(m, rng, district), m.shade()
    m.prism(poly, 0.0, height, mat)
    m.facade_windows(poly, 0.0, height, floor, "warm", rng, lit=0.45)
    m.ring(poly, height - 0.05, m.lines[shade], width=0.08)
    m.fins(poly, poly, 0.0, height, m.lines[shade])
    # A lit entrance on one street face.
    n = len(poly)
    c = middle(poly)
    for i in street_faces:
        a, b = poly[i], poly[(i + 1) % n]
        e = (b - a)
        u = e.normalized()
        out = Vector((u.y, -u.x, 0))
        if out.dot((a + b) / 2 - c) < 0:
            out = -out
        # Balconies: slabs out from the face, from the second floor up.
        if rng.random() < 0.7:
            step = rng.choice((5.2, 7.8))
            t = 2.0
            while t + 3.0 < e.length - 1.0:
                p = a + u * t
                for f in range(1, floors):
                    z = f * floor
                    q0, q1 = p, p + u * 3.0
                    m.solid([q0, q1, q1 + out * 1.3, q0 + out * 1.3], z - 0.15, z, m.struct, m.facades["slate"])
                    rail_mat = m.lines[shade] if f % 2 == 0 else m.rail
                    m.line(q0 + out * 1.3 + Vector((0, 0, z + 0.9)), q1 + out * 1.3 + Vector((0, 0, z + 0.9)),
                           rail_mat, width=0.05, light=False)
                t += step
        mid = (a + b) / 2
        m.line(mid - u * 1.5 + out * 0.08 + Vector((0, 0, 1.3)), mid + u * 1.5 + out * 0.08 + Vector((0, 0, 1.3)),
               m.windows["warm"], width=2.4, height=0.06, across=Z, light=False)
        m.line(mid - u * 2.0 + out * 0.15 + Vector((0, 0, 2.9)), mid + u * 2.0 + out * 0.15 + Vector((0, 0, 2.9)),
               m.lines[shade])
        break
    m.roof_clutter(poly, height, rng)


def courtyard(m, rng, poly, district, street_faces):
    """A block round a garden court, open to the street on one side."""
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    wing = 8.0
    floor = 3.2
    height = max(3, int(tall(rng, district) / floor)) * floor
    mat, shade = facade(m, rng, district), m.shade()
    open_side = street_faces[0] if street_faces else 0
    wings = {
        0: rect(x0, x1, y0, y0 + wing),          # south
        1: rect(x1 - wing, x1, y0, y1),          # east
        2: rect(x0, x1, y1 - wing, y1),          # north
        3: rect(x0, x0 + wing, y0, y1),          # west
    }
    for side, p in wings.items():
        if side == open_side:
            continue
        m.prism(p, 0.0, height, mat)
        m.facade_windows(p, 0.0, height, floor, "warm", rng, lit=0.45)
        m.ring(p, height - 0.05, m.lines[shade], width=0.08)
    outline_corners(m, [p for side, p in wings.items() if side != open_side], 0.0, height, m.lines[shade])
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    m.tree(cx, cy, rng, size=1.3)
    for dx, dy in ((-4, -3), (4, 3)):
        m.box(WORLD, cx + dx - 1.0, cx + dx + 1.0, cy + dy - 0.3, cy + dy + 0.3, 0.0, 0.5, m.cover, m.crate)
    m.ring(polygon(3.0, 12, 0, cx, cy), KERB + 0.01, m.lines["green"], out=0.0, width=0.08)


def stilts(m, rng, poly, district):
    """A building up on columns over an open, lit ground floor."""
    lift_h = rng.uniform(6.0, 8.0)
    height = lift_h + tall(rng, district) * 0.7
    mat, shade = facade(m, rng, district), m.shade()
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    for x in (x0 + 1.5, (x0 + x1) / 2, x1 - 1.5):
        for y in (y0 + 1.5, y1 - 1.5):
            m.prism(polygon(0.6, 8, 0, x, y), 0.0, lift_h, m.facades["concrete"])
            m.line(Vector((x + 0.62, y, 0.0)), Vector((x + 0.62, y, lift_h)), m.lines[shade], width=0.08)
    m.prism(poly, lift_h, height, mat)
    m.fins(poly, poly, lift_h, height, m.lines[shade])
    m.facade_windows(poly, lift_h + 0.4, height, 3.6, rng.choice(("warm", "office")), rng, lit=0.55,
                     bands=rng.random() < 0.5)
    m.ring(poly, lift_h - 0.05, m.lines[shade])
    m.ring(poly, height - 0.05, m.lines[shade], width=0.08)
    for x, y in (((x0 + x1) / 2 - 4, (y0 + y1) / 2), ((x0 + x1) / 2 + 4, (y0 + y1) / 2)):
        m.glass_pane(WORLD, x - 0.6, x + 0.6, y - 0.6, y + 0.6, lift_h - 0.04, lift_h)


def shops(m, rng, poly, district, street_faces):
    """A row of shops along the street, each its own width, height and colour, with an awning
    and a sign; behind them, the rest of the lot."""
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    side = street_faces[0] if street_faces else 0
    depth = rng.uniform(9.0, 14.0)
    # The row runs along the street face; local u along it, v inward.
    if side == 0:
        start, u, v, length = Vector((x0, y0, 0)), Vector((1, 0, 0)), Vector((0, 1, 0)), x1 - x0
    elif side == 2:
        start, u, v, length = Vector((x1, y1, 0)), Vector((-1, 0, 0)), Vector((0, -1, 0)), x1 - x0
    elif side == 1:
        start, u, v, length = Vector((x1, y0, 0)), Vector((0, 1, 0)), Vector((-1, 0, 0)), y1 - y0
    else:
        start, u, v, length = Vector((x0, y1, 0)), Vector((0, -1, 0)), Vector((1, 0, 0)), y1 - y0
    t = 0.0
    while t < length - 3.0:
        w = min(rng.uniform(5.0, 11.0), length - t)
        h = rng.uniform(4.0, 9.5)
        a, b = start + u * t, start + u * (t + w)
        p = [a, b, b + v * depth, a + v * depth]
        mat, shade = facade(m, rng, district), m.shade()
        if w >= STORE_SIZE[0] and depth >= STORE_SIZE[1] and m.trade.random() < STORE_CHANCE:
            # A store to walk into, with a computer at the back to hack.
            store(m, a, u, v, w, depth, h, mat, shade)
        else:
            m.prism(p, 0.0, h, mat)
            # The shop window, lit warm.
            m.line(a + u * 0.6 - v * 0.05 + Vector((0, 0, 1.6)), b - u * 0.6 - v * 0.05 + Vector((0, 0, 1.6)),
                   m.windows["warm"], width=2.2, height=0.06, across=Z, light=False)
        m.fins(p, p, 0.0, max(h, STORE_HEIGHT), m.lines[shade])
        # A sign over the awning.
        q0, q1 = a + u * 0.2, b - u * 0.2
        m.solid([q0, q1, q1 - v * 1.6, q0 - v * 1.6], 3.1, 3.25, m.struct, m.rail)
        m.line(q0 - v * 1.6 + Vector((0, 0, 3.17)), q1 - v * 1.6 + Vector((0, 0, 3.17)), m.lines[shade],
               width=0.1)
        if h > 5.0:
            m.line(a + u * w * 0.2 - v * 0.08 + Vector((0, 0, 4.1)), b - u * w * 0.2 - v * 0.08 + Vector((0, 0, 4.1)),
                   m.lines[m.shade()], width=0.6, height=0.08, across=Z)
        if h > 7.5:
            m.facade_windows(p, 5.0, h, 3.0, "warm", rng, lit=0.5, faces=[0])
        t += w
    # Behind the row, a lower building or a yard with a tree.
    rest = [start + v * (depth + 2.0), start + u * length + v * (depth + 2.0)]
    far = {0: y1, 2: y0, 1: x0, 3: x1}[side]
    reach = abs(far - (rest[0].y if side in (0, 2) else rest[0].x))
    if reach > 6.0:
        p = [rest[0], rest[1], rest[1] + v * reach, rest[0] + v * reach]
        if rng.random() < 0.5:
            apartments(m, rng, p, district, [])
        else:
            c = middle(p)
            m.tree(c.x, c.y, rng)


def outline_corners(m, rects, z0, z1, mat, out=0.06):
    """Lines of light up the corners of what the rectangles `rects` make together - each corner
    of their outline, going in or out - not where one meets another along a wall."""
    def inside(x, y):
        return any(min(p.x for p in r) < x < max(p.x for p in r) and min(p.y for p in r) < y < max(p.y for p in r)
                   for r in rects)
    seen = set()
    for r in rects:
        for c in r:
            key = (round(c.x, 3), round(c.y, 3))
            if key in seen:
                continue
            seen.add(key)
            empty = [Vector((dx, dy, 0)) for dx in (-1, 1) for dy in (-1, 1) if not inside(c.x + dx * 0.05, c.y + dy * 0.05)]
            # A corner has one quarter round it empty (going out) or three (going in); a wall, two.
            if len(empty) not in (1, 3):
                continue
            way = sum(empty, Vector()).normalized() * out
            m.line(Vector((c.x + way.x, c.y + way.y, z0)), Vector((c.x + way.x, c.y + way.y, z1)), mat)


def quad(at, u, v, u0, u1, v0, v1):
    """The rectangle from u0 to u1 along `u` and v0 to v1 along `v`, from `at`."""
    return [at + u * u0 + v * v0, at + u * u1 + v * v0, at + u * u1 + v * v1, at + u * u0 + v * v1]


def slab(base, z0, z1):
    """The vertices and faces of a solid on the polygon `base` from z0 to z1."""
    n = len(base)
    verts = [Vector((p.x, p.y, z0)) for p in base] + [Vector((p.x, p.y, z1)) for p in base]
    faces = [tuple(range(n)), tuple(range(n, 2 * n))] + [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    return verts, faces


def bar(p, q, width, out):
    """The vertices and faces of a square bar from `p` to `q`, `width` across, flat to `out`."""
    d = (q - p).normalized()
    a = d.cross(out).normalized() * (width / 2)
    b = out.normalized() * (width / 2)
    verts = [e + sa * a + sb * b for e in (p, q) for sa, sb in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
    return verts, [(0, 1, 2, 3), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]


def facing(direction):
    """The turn round from +x that faces along `direction`, for a marker."""
    return math.atan2(direction.y, direction.x)


def room(m, at, u, v, w, depth, h, mat, door):
    """A room to walk into, `w` along the street (`u`) and `depth` in (`v`) from `at`, `h` high to
    its roof: walls WALL thick, and a doorway in the middle of the front `door` (wide, high)."""
    t = WALL
    dw, dh = door
    d0, d1 = (w - dw) / 2, (w + dw) / 2
    for u0, u1 in ((0.0, d0), (d1, w)):
        m.prism(quad(at, u, v, u0, u1, 0.0, t), 0.0, h, mat)
    m.prism(quad(at, u, v, d0, d1, 0.0, t), dh, h, mat)
    m.prism(quad(at, u, v, 0.0, w, depth - t, depth), 0.0, h, mat)
    for u0, u1 in ((0.0, t), (w - t, w)):
        m.prism(quad(at, u, v, u0, u1, t, depth - t), 0.0, h, mat)
    m.prism(quad(at, u, v, 0.0, w, 0.0, depth), h - 0.3, h, mat)


def store(m, a, u, v, w, depth, h, mat, shade):
    """A store, open to walk into: shelves down its sides, a counter, lights along its ceiling,
    and against the back wall the computer that keeps its takings - a store_* marker."""
    inside = STORE_HEIGHT
    room(m, a, u, v, w, depth, inside, mat, STORE_DOOR)
    if h > inside + 0.5:
        m.prism(quad(a, u, v, 0.0, w, 0.0, depth), inside, h, mat)
    up = Vector((0, 0, 1))
    d0, d1 = (w - STORE_DOOR[0]) / 2, (w + STORE_DOOR[0]) / 2
    # The shop window, lit warm, either side of the door.
    for u0, u1 in ((0.6, d0 - 0.3), (d1 + 0.3, w - 0.6)):
        if u1 - u0 > 0.5:
            m.line(a + u * u0 - v * 0.05 + up * 1.6, a + u * u1 - v * 0.05 + up * 1.6,
                   m.windows["warm"], width=2.2, height=0.06, across=Z, light=False)
    # Its doorway outlined in its colour, and its ceiling lit with it.
    lines = m.lines[shade]
    for k in (d0, d1):
        m.line(a + u * k - v * 0.06 + up * KERB, a + u * k - v * 0.06 + up * STORE_DOOR[1], lines, width=0.08)
    m.line(a + u * d0 - v * 0.06 + up * STORE_DOOR[1], a + u * d1 - v * 0.06 + up * STORE_DOOR[1], lines, width=0.08)
    for k in (w * 0.35, w * 0.65):
        m.line(a + u * k + v * 1.2 + up * (inside - 0.36), a + u * k + v * (depth - 1.2) + up * (inside - 0.36),
               lines, width=0.12)
    # Shelves down both sides, short of the back, each shelf's edge lit.
    t = WALL
    for u0, u1 in ((t, t + 0.6), (w - t - 0.6, w - t)):
        m.prism(quad(a, u, v, u0, u1, 1.4, depth - 2.6), 0.0, 1.8, m.crate, col=m.cover)
        for z in (0.7, 1.25, 1.8):
            edge = u1 if u0 < w / 2 else u0
            m.line(a + u * edge + v * 1.4 + up * z, a + u * edge + v * (depth - 2.6) + up * z, m.white,
                   width=0.03, light=False)
    # The counter, to one side of the back, and the computer behind it in the middle.
    m.prism(quad(a, u, v, t + 0.9, w / 2 - 1.3, depth - 2.8, depth - 2.2), 0.0, 1.05, m.rail, col=m.cover)
    m.line(a + u * (t + 0.9) + v * (depth - 2.83) + up * 1.05, a + u * (w / 2 - 1.3) + v * (depth - 2.83) + up * 1.05,
           lines, width=0.05)
    spot = a + u * (w / 2) + v * (depth - WALL - 0.45)
    br_town.marker(f"store_{m.n_stores + 1}", spot.x, spot.y, KERB, facing(-v))
    m.n_stores += 1


# Office towers to go up.

# The segments of a seven-segment figure lit for each sign, as (from, to) corners of a figure one
# wide and two high: a the top, then clockwise, g the middle.
SEGMENT_ENDS = {"a": ((0, 2), (1, 2)), "b": ((1, 2), (1, 1)), "c": ((1, 1), (1, 0)), "d": ((0, 0), (1, 0)),
                "e": ((0, 0), (0, 1)), "f": ((0, 1), (0, 2)), "g": ((0, 1), (1, 1))}
SEGMENTS = {"0": "abcdef", "1": "bc", "2": "abged", "3": "abgcd", "4": "fgbc", "5": "afgcd", "6": "afgedc",
            "7": "abc", "8": "abcdefg", "9": "abcdfg", "L": "fed", "R": "eg"}


def figures(m, text, at, along, size, mat):
    """`text` in seven-segment figures `size` m high, from `at` along `along`, upright, its face out
    the way `along` turned a quarter clockwise looks from above - lit lines on a wall."""
    up = Vector((0, 0, 1))
    w = size / 2
    for i, sign in enumerate(text):
        corner = at + along * (i * w * 1.6)
        for seg in SEGMENTS[sign]:
            (x0, y0), (x1, y1) = SEGMENT_ENDS[seg]
            p = corner + along * (x0 * w) + up * (y0 * w)
            q = corner + along * (x1 * w) + up * (y1 * w)
            m.line(p, q, mat, width=0.05, light=False)


def unpainted_shade(m):
    """A colour as the city would pick one, without taking a draw from its stream of colours."""
    state = m.paint.getstate()
    shade = m.shade()
    m.paint.setstate(state)
    return shade


def office_tower(m, rng, poly, street, mat, shade):
    """An office tower to go up: a lobby off the street, through a doorway in the middle of its
    face on `street`, and open floors over it, their windows open to the city over sills too high to
    jump; and up the back, a lift to every floor and the roof - its car (lift_<n>, its origin the
    middle of its deck as it stands at the lobby), where it stops (lift_<n>_<k>, at floor k), and a
    gate before its shaft at each floor (lift_<n>_gate_<k>, standing open while the car is there),
    which the game moves and opens. Desks on the floors, a counter in the lobby, the floor's number
    by the lift, and lights along every ceiling."""
    x0, x1 = min(p.x for p in poly), max(p.x for p in poly)
    y0, y1 = min(p.y for p in poly), max(p.y for p in poly)
    side = street[0]
    # Local u along the street face, v in from it, as the shops have them.
    if side == 0:
        start, u, v, length, depth = Vector((x0, y0, 0)), Vector((1, 0, 0)), Vector((0, 1, 0)), x1 - x0, y1 - y0
    elif side == 2:
        start, u, v, length, depth = Vector((x1, y1, 0)), Vector((-1, 0, 0)), Vector((0, -1, 0)), x1 - x0, y1 - y0
    elif side == 1:
        start, u, v, length, depth = Vector((x1, y0, 0)), Vector((0, 1, 0)), Vector((-1, 0, 0)), y1 - y0, x1 - x0
    else:
        start, u, v, length, depth = Vector((x0, y1, 0)), Vector((0, -1, 0)), Vector((1, 0, 0)), y1 - y0, x1 - x0
    w, d = min(length, OFFICE_SIZE[1]), min(depth, OFFICE_SIZE[1])
    a = start + u * ((length - w) / 2)
    up = Vector((0, 0, 1))
    t = WALL
    floors = rng.randint(*OFFICE_FLOORS)
    z = lambda k: KERB + k * OFFICE_FLOOR       # floor k's top; the roof is floor `floors`
    roof = z(floors)
    lines = m.lines[shade]
    m.n_lifts += 1
    name = f"lift_{m.city}{m.n_lifts}"
    # The shaft: up the back, in the middle; its inside from c - SHAFT/2 to c + SHAFT/2 along u,
    # and from s0 to the back wall along v; its own walls round it, the front one with a doorway
    # at every floor.
    c = w / 2
    h = SHAFT / 2
    s1 = d - t
    s0 = s1 - SHAFT
    # The outside walls: a sill and a head along each face at every floor, piers between the
    # windows, and the doorway off the street in the middle of the front.
    faces = (  # (along from, along to, as a quad: u0, u1, v0, v1 for a span s0..s1)
        ("front", w, lambda p, q: quad(a, u, v, p, q, 0.0, t)),
        ("back", w, lambda p, q: quad(a, u, v, p, q, d - t, d)),
        ("left", d, lambda p, q: quad(a, u, v, 0.0, t, p, q)),
        ("right", d, lambda p, q: quad(a, u, v, w - t, w, p, q)),
    )
    for face, span, at in faces:
        bays = max(1, round(span / OFFICE_BAY))
        if face == "front" and bays % 2 == 0:
            bays += 1            # a window in the middle, for the door
        bay = span / bays
        # The piers the whole way up, and between one floor's windows and the next's, one band of
        # wall: from a window's head to the sill over it.
        for i in range(bays + 1):
            p0, p1 = max(0.0, i * bay - OFFICE_PIER / 2), min(span, i * bay + OFFICE_PIER / 2)
            m.prism(at(p0, p1), 0.0, roof, mat)
        if face == "front":
            g0, g1 = (span - OFFICE_DOOR[0]) / 2, (span + OFFICE_DOOR[0]) / 2
            m.prism(at(0.0, g0), 0.0, z(0) + OFFICE_SILL, mat)
            m.prism(at(g1, span), 0.0, z(0) + OFFICE_SILL, mat)
        else:
            m.prism(at(0.0, span), 0.0, z(0) + OFFICE_SILL, mat)
        for k in range(floors):
            m.prism(at(0.0, span), z(k) + OFFICE_HEAD, min(z(k + 1) + OFFICE_SILL, roof), mat)
        if face == "back":
            # Behind the shaft, solid.
            m.prism(at(c - h - t, c + h + t), 0.0, roof, mat)
    # The floors, and the roof: all but the shaft.
    for k in range(1, floors + 1):
        for q in (quad(a, u, v, 0.0, w, 0.0, s0), quad(a, u, v, 0.0, c - h, s0, d), quad(a, u, v, c + h, w, s0, d),
                  quad(a, u, v, c - h, c + h, s1, d)):
            m.prism(q, z(k) - OFFICE_SLAB, z(k), mat)
    # A parapet round the roof, chest high.
    for q in (quad(a, u, v, 0.0, w, 0.0, 0.25), quad(a, u, v, 0.0, w, d - 0.25, d),
              quad(a, u, v, 0.0, 0.25, 0.25, d - 0.25), quad(a, u, v, w - 0.25, w, 0.25, d - 0.25)):
        m.prism(q, roof, roof + 1.2, mat)
    outline = quad(a, u, v, 0.0, w, 0.0, d)
    m.ring(outline, roof + 1.2, lines)
    m.fins(outline, outline, 0.0, roof + 1.2, lines)
    for k in range(1, floors + 1):
        m.ring(outline, z(k) - OFFICE_SLAB / 2, lines, width=0.08)
    # The shaft's walls, up to its house on the roof, and its roof.
    top = roof + ROOF_HOUSE
    for q in (quad(a, u, v, c - h - t, c - h, s0 - t, d), quad(a, u, v, c + h, c + h + t, s0 - t, d)):
        m.prism(q, 0.0, top, m.facades["slate"])
    m.prism(quad(a, u, v, c - h, c + h, d - t, d), roof, top, mat)
    m.prism(quad(a, u, v, c - h - t, c + h + t, s0 - t, d), top, top + 0.3, mat)
    m.ring(quad(a, u, v, c - h - t, c + h + t, s0 - t, d), top + 0.3, lines)
    dw, dh = SHAFT_DOOR
    for k in range(floors + 1):
        low = 0.0 if k == 0 else z(k)
        high = z(k + 1) - OFFICE_SLAB if k < floors else top
        for q in (quad(a, u, v, c - h - t, c - dw / 2, s0 - t, s0), quad(a, u, v, c + dw / 2, c + h + t, s0 - t, s0)):
            m.prism(q, low, z(k) + dh, m.facades["slate"])
        m.prism(quad(a, u, v, c - dw / 2, c + dw / 2, s0 - t, s0), z(k) + dh, max(high, z(k) + dh + 0.3),
                m.facades["slate"])
        # Its doorway lit round, and the floor's number beside it.
        front = s0 - t - 0.02
        for e in (c - dw / 2 - 0.05, c + dw / 2 + 0.05):
            m.line(a + u * e + v * front + up * z(k), a + u * e + v * front + up * (z(k) + dh + 0.05), lines,
                   width=0.06, light=False)
        m.line(a + u * (c - dw / 2 - 0.05) + v * front + up * (z(k) + dh + 0.05),
               a + u * (c + dw / 2 + 0.05) + v * front + up * (z(k) + dh + 0.05), lines, width=0.06, light=False)
        sign = "L" if k == 0 else "R" if k == floors else str(k)
        figures(m, sign, a + u * (c - h + 0.05) + v * front + up * (z(k) + 1.6), u, 0.36, m.white)
    # Inside each floor: lights along the ceiling, a line round the floor, and desks.
    for k in range(floors):
        ceiling = z(k + 1) - OFFICE_SLAB - 0.04
        for share in (0.25, 0.5, 0.75):
            m.line(a + u * (w * share) + v * 1.0 + up * ceiling, a + u * (w * share) + v * (s0 - t - 1.0) + up * ceiling,
                   m.white, width=0.25, height=0.04, across=u)
        m.ring(quad(a, u, v, t + 0.03, w - t - 0.03, t + 0.03, d - t - 0.03), z(k) + 0.04, lines, out=0.0,
               width=0.05, light=False)
        if k == 0:
            # The lobby's counter, to one side.
            m.prism(quad(a, u, v, 1.6, 4.6, 4.0, 4.8), z(0), z(0) + 1.1, m.rail, col=m.cover)
            m.line(a + u * 1.6 + v * 3.98 + up * (z(0) + 1.1), a + u * 4.6 + v * 3.98 + up * (z(0) + 1.1), lines,
                   width=0.05)
            continue
        for row in range(int((s0 - t - 4.0) / 3.6)):
            dv = 2.5 + row * 3.6
            for col in range(int((w - 2 * t - 3.0) / 2.4)):
                if rng.random() < 0.6:
                    continue
                du = 1.8 + col * 2.4
                m.prism(quad(a, u, v, du, du + 1.6, dv, dv + 0.8), z(k), z(k) + 0.75, m.rail, col=m.cover)
                m.line(a + u * du + v * (dv - 0.01) + up * (z(k) + 0.75), a + u * (du + 1.6) + v * (dv - 0.01)
                       + up * (z(k) + 0.75), m.white, width=0.03, light=False)
    # The doorway off the street, lit round, and its name over it.
    g0, g1 = (w - OFFICE_DOOR[0]) / 2, (w + OFFICE_DOOR[0]) / 2
    for e in (g0, g1):
        m.line(a + u * e - v * 0.06 + up * KERB, a + u * e - v * 0.06 + up * OFFICE_DOOR[1], lines, width=0.08)
    m.line(a + u * g0 - v * 0.06 + up * OFFICE_DOOR[1], a + u * g1 - v * 0.06 + up * OFFICE_DOOR[1], lines, width=0.08)
    m.line(a + u * (g0 - 1.0) - v * 0.08 + up * (OFFICE_HEAD + 0.3), a + u * (g1 + 1.0) - v * 0.08 + up * (OFFICE_HEAD + 0.3),
           lines, width=0.5, height=0.08, across=Z)

    # The lift: its car, at the lobby, a piece of its own; where it stops; and its gates.
    deck = a + u * c + v * (s0 + h)
    deck.z = z(0)
    glass = m.white

    def car():
        lo, hi = -CAR / 2, CAR / 2
        m.prism(quad(deck, u, v, lo, hi, lo, hi), z(0) - 0.2, z(0), m.rail)
        for q in (quad(deck, u, v, lo, hi, hi - 0.08, hi), quad(deck, u, v, lo, lo + 0.08, lo, hi - 0.08),
                  quad(deck, u, v, hi - 0.08, hi, lo, hi - 0.08)):
            m.prism(q, z(0), z(0) + CAR_HIGH, m.facades["slate"])
        m.prism(quad(deck, u, v, lo, hi, lo, hi), z(0) + CAR_HIGH, z(0) + CAR_HIGH + 0.1, m.rail)
        # Its light, its rail and the edge of its deck lit.
        m.prism(quad(deck, u, v, lo + 0.4, hi - 0.4, lo + 0.4, hi - 0.4), z(0) + CAR_HIGH - 0.03, z(0) + CAR_HIGH, glass)
        for q in (quad(deck, u, v, lo + 0.08, hi - 0.08, hi - 0.12, hi - 0.08), quad(deck, u, v, lo + 0.08, lo + 0.12, lo, hi - 0.12),
                  quad(deck, u, v, hi - 0.12, hi - 0.08, lo, hi - 0.12)):
            m.prism(q, z(0) + 0.95, z(0) + 1.0, lines)
        m.prism(quad(deck, u, v, lo, hi, lo, lo + 0.04), z(0) - 0.02, z(0) + 0.01, lines)

    m.piece(name, deck, car)
    for k in range(1, floors + 1):
        m.mark(f"{name}_{k}", deck + up * (z(k) - z(0)))
    for k in range(floors + 1):
        middle_ = a + u * c + v * (s0 - t / 2) + up * z(k)

        def gate(k=k, middle_=middle_):
            mid = s0 - t / 2
            for i in range(9):
                e = -dw / 2 + 0.05 + i * (dw - 0.1) / 8
                m.prism(quad(a, u, v, c + e - 0.02, c + e + 0.02, mid - 0.02, mid + 0.02), z(k), z(k) + dh, lines)
            for hgt in (0.1, 1.2, dh - 0.06):
                m.prism(quad(a, u, v, c - dw / 2, c + dw / 2, mid - 0.025, mid + 0.025), z(k) + hgt - 0.03, z(k) + hgt + 0.03,
                        lines)

        m.piece(f"{name}_gate_{k}", middle_, gate)


def bank(m, poly, street_faces):
    """A bank: a hall behind a colonnade, its credit sign in gold over the door, a teller's counter
    across it, and at the back the vault: a strongroom of gold behind a door that swings open -
    vault_door_<n>, a piece of its own for the game to move, and vault_<n> marking the middle of
    the strongroom. Four computers: bank_<n>_vault beside the vault's door, which opens it, another
    on its other side, and one on each side wall of the lobby, before the counter. Its own stream
    of chance, so the rest of the city is as it was."""
    rng = m.trade
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    # Facing the street that leaves it deep enough for its vault, the longest of those.
    def along(f):
        return x1 - x0 if f in (0, 2) else y1 - y0
    def into(f):
        return y1 - y0 if f in (0, 2) else x1 - x0
    deep = [f for f in street_faces if into(f) - BANK_FORECOURT >= BANK_DEEP]
    side = max(deep or street_faces, key=along if deep else into)
    start, u, v, length, reach = {
        0: (Vector((x0, y0, 0)), Vector((1, 0, 0)), Vector((0, 1, 0)), x1 - x0, y1 - y0),
        2: (Vector((x1, y1, 0)), Vector((-1, 0, 0)), Vector((0, -1, 0)), x1 - x0, y1 - y0),
        1: (Vector((x1, y0, 0)), Vector((0, 1, 0)), Vector((-1, 0, 0)), y1 - y0, x1 - x0),
        3: (Vector((x0, y1, 0)), Vector((0, -1, 0)), Vector((1, 0, 0)), y1 - y0, x1 - x0),
    }[side]
    w, depth = min(length - 1.0, BANK_SIZE[0]), min(reach - BANK_FORECOURT, BANK_SIZE[1])
    a = start + u * ((length - w) / 2) + v * BANK_FORECOURT
    hall = BANK_HEIGHT
    mat = m.facades[rng.choice(("stone", "white", "sand"))]
    gold = m.lines["yellow"]
    up = Vector((0, 0, 1))
    room(m, a, u, v, w, depth, hall, mat, BANK_DOOR)
    top = hall + rng.uniform(4.0, 9.0)
    m.prism(quad(a, u, v, -0.4, w + 0.4, -0.4, depth + 0.4), hall, hall + 0.6, mat)
    m.prism(quad(a, u, v, 0.0, w, 0.0, depth), hall + 0.6, top, mat)
    m.fins(quad(a, u, v, 0.0, w, 0.0, depth), quad(a, u, v, 0.0, w, 0.0, depth), 0.0, top, gold)
    m.facade_windows(quad(a, u, v, 0.0, w, 0.0, depth), hall + 1.5, top, 3.0, "office", rng, lit=0.4)
    m.ring(quad(a, u, v, -0.4, w + 0.4, -0.4, depth + 0.4), hall + 0.62, gold, width=0.1)
    m.ring(quad(a, u, v, 0.0, w, 0.0, depth), top + 0.02, gold, width=0.1, light=False)
    # The colonnade, out in the forecourt, with a gold line up each column.
    columns = max(4, int(w // 3.2))
    for k in range(columns):
        cu = 0.8 + (w - 1.6) * k / (columns - 1)
        if abs(cu - w / 2) < BANK_DOOR[0] / 2 + 0.6:
            continue
        c = a + u * cu - v * 2.2
        m.solid(polygon(0.4, 12, 0, c.x, c.y), KERB, hall, m.struct, mat)
        m.line(c - v * 0.42 + up * KERB, c - v * 0.42 + up * hall, gold, width=0.06, light=False)
    m.prism(quad(a, u, v, 0.0, w, -3.0, 0.0), hall - 0.5, hall, mat)
    # The credit sign over the door: a ring of gold, a bar down through it.
    sign = a + u * (w / 2) - v * 3.05 + up * (hall - 1.6)
    ring = [sign + u * (0.9 * math.cos(t)) + up * (0.9 * math.sin(t)) for t in (k * math.tau / 16 for k in range(17))]
    for p, q in zip(ring, ring[1:]):
        m.line(p, q, gold, width=0.12, across=-v, light=False)
    m.line(sign - up * 1.2, sign + up * 1.2, gold, width=0.12, across=-v, light=False)
    # The doorway in gold, and the hall lit gold.
    d0, d1 = (w - BANK_DOOR[0]) / 2, (w + BANK_DOOR[0]) / 2
    for k in (d0, d1):
        m.line(a + u * k - v * 0.06 + up * KERB, a + u * k - v * 0.06 + up * BANK_DOOR[1], gold, width=0.1)
    for k in (w * 0.3, w * 0.7):
        m.line(a + u * k + v * 1.5 + up * (hall - 0.4), a + u * k + v * (depth - 1.5) + up * (hall - 0.4),
               gold, width=0.14)
    # The vault across the back of the hall: a wall with a doorway, and its door in it.
    front = depth - VAULT - WALL                 # the hall side of the vault's wall
    half, high = VAULT_DOOR[0] / 2, VAULT_DOOR[1]
    for u0, u1 in ((WALL, w / 2 - half), (w / 2 + half, w - WALL)):
        m.prism(quad(a, u, v, u0, u1, front, front + WALL), 0.0, hall, mat)
    m.prism(quad(a, u, v, w / 2 - half, w / 2 + half, front, front + WALL), high, hall, mat)
    m.n_banks += 1
    door = quad(a, u, v, w / 2 - half + 0.02, w / 2 + half - 0.02, front + 0.03, front + WALL - 0.03)
    Nexus.mesh(m, *slab(door, KERB + 0.02, high - 0.02), m.struct, m.facades["bronze"]).name = f"vault_door_{m.n_banks}"
    # Its wheel and bolts in gold, on the door, to swing open with it.
    hub = a + u * (w / 2) + v * (front - 0.02) + up * (KERB + high / 2)
    spokes = []
    for k in range(4):
        t = k * math.pi / 4
        d = u * math.cos(t) + up * math.sin(t)
        spokes.append(bar(hub - d * 0.55, hub + d * 0.55, 0.07, -v))
    for k in range(-1, 2):
        spokes.append(bar(hub + u * (half - 0.2) + up * (k * 0.7) - u * 0.25, hub + u * (half - 0.2) + up * (k * 0.7), 0.1, -v))
    verts, faces = [], []
    for vs, fs in spokes:
        faces += [tuple(len(verts) + i for i in f) for f in fs]
        verts += vs
    Nexus.mesh(m, verts, faces, m.struct, gold).name = f"vault_door_{m.n_banks}_wheel"
    # The doorway framed in gold.
    for k in (w / 2 - half - 0.1, w / 2 + half + 0.1):
        m.line(a + u * k + v * (front - 0.04) + up * KERB, a + u * k + v * (front - 0.04) + up * (high + 0.1), gold, width=0.14)
    m.line(a + u * (w / 2 - half - 0.1) + v * (front - 0.04) + up * (high + 0.1),
           a + u * (w / 2 + half + 0.1) + v * (front - 0.04) + up * (high + 0.1), gold, width=0.14)
    # In the strongroom: gold stacked along its walls, lit gold.
    for u0, u1, v0, v1 in ((WALL + 0.2, w / 2 - half - 0.6, depth - WALL - 1.0, depth - WALL - 0.2),
                           (w / 2 + half + 0.6, w - WALL - 0.2, depth - WALL - 1.0, depth - WALL - 0.2)):
        if u1 - u0 > 0.8:
            for layer in range(3):
                m.prism(quad(a, u, v, u0, u1, v0, v1), KERB + layer * 0.3, KERB + layer * 0.3 + 0.26,
                        m.lines["yellow"] if layer % 2 == 0 else m.facades["bronze"], col=m.cover)
    m.line(a + u * (w / 2 - 2.0) + v * (front + WALL + VAULT / 2) + up * 2.9,
           a + u * (w / 2 + 2.0) + v * (front + WALL + VAULT / 2) + up * 2.9, gold, width=0.14)
    middle_ = a + u * (w / 2) + v * (front + WALL + VAULT / 2)
    br_town.marker(f"vault_{m.n_banks}", middle_.x, middle_.y, KERB, facing(-v))
    hatch(m, middle_ - u * 2.8, KERB, f"bank_{m.n_banks}")
    # The tellers' counter across the hall, but for a way through in the middle.
    across_at = front * 0.45
    for u0, u1 in ((WALL, w / 2 - 1.5), (w / 2 + 1.5, w - WALL)):
        m.prism(quad(a, u, v, u0, u1, across_at, across_at + 0.7), 0.0, 1.15, m.rail, col=m.cover)
        m.line(a + u * u0 + v * (across_at - 0.03) + up * 1.15, a + u * u1 + v * (across_at - 0.03) + up * 1.15,
               gold, width=0.05)
    # Its computers: either side of the vault's door, the left one opening it, and one on each
    # side wall of the lobby.
    off = min(3.0, w / 2 - 1.2)
    spots = [("vault", w / 2 - off, front - 0.45, -v), ("2", w / 2 + off, front - 0.45, -v),
             ("3", WALL + 0.45, across_at * 0.55, u), ("4", w - WALL - 0.45, across_at * 0.55, -u)]
    for name, su, sv, face in spots:
        spot = a + u * su + v * sv
        br_town.marker(f"bank_{m.n_banks}_{name}", spot.x, spot.y, KERB, facing(face))



def precinct_frame(poly, street_faces):
    """Where a precinct on `poly` stands: the corner of its front at the street, which way along the
    street and which way in, how far it has along the street and in - facing the street that
    leaves it deep enough, the longest of those - or None if none does."""
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    frames = {
        0: (Vector((x0, y0, 0)), Vector((1, 0, 0)), Vector((0, 1, 0)), x1 - x0, y1 - y0),
        2: (Vector((x1, y1, 0)), Vector((-1, 0, 0)), Vector((0, -1, 0)), x1 - x0, y1 - y0),
        1: (Vector((x1, y0, 0)), Vector((0, 1, 0)), Vector((-1, 0, 0)), y1 - y0, x1 - x0),
        3: (Vector((x0, y1, 0)), Vector((0, -1, 0)), Vector((1, 0, 0)), y1 - y0, x1 - x0),
    }
    fore = PRECINCT_LANDING + 0.4
    fits = [f for f in street_faces if frames[f][4] - fore >= PRECINCT_SIZE[1] and frames[f][3] >= 15.0]
    if not fits:
        return None
    return frames[max(fits, key=lambda f: frames[f][3])], fore


def precinct_fits(lot, street_faces):
    x0, x1, y0, y1 = lot
    return precinct_frame(rect(x0 + 1.5, x1 - 1.5, y0 + 1.5, y1 - 1.5), street_faces) is not None


def precinct(m, poly, street_faces):
    """A security precinct: up on a podium, climbed by a flight from the street, a hall with the
    desk across it and the evidence locker behind; a firewall through to the cell block, a corridor
    along it, and the cells along the back, each behind a firewall; and from the first cell, a
    loose panel onto a stair down into a tunnel under the podium, out at its side. Dark and hard,
    lit in red and cyan. Its own stream of chance, so the rest of the city is as it was."""
    rng = m.law
    (start, u, v, length, reach), fore = precinct_frame(poly, street_faces)
    w, depth = min(length - 1.0, PRECINCT_SIZE[0]), PRECINCT_SIZE[1]
    a = start + u * ((length - w) / 2) + v * fore
    m.n_precincts += 1
    n = m.n_precincts
    P, H, t = PRECINCT_PODIUM, PRECINCT_HIGH, WALL
    up = Vector((0, 0, 1))
    mat, trim = m.facades["slate"], m.facades["concrete"]
    red, cyan = m.lines["red"], m.lines["cyan"]
    # Where things are, from the front: the hall, the gate's wall, the corridor, the bars, the
    # cells, their back wall, the stair and tunnel behind it, and the back wall.
    hall, gate0, gate1, bars0, bars1 = 5.6, 5.6, 5.9, 8.3, 8.4
    back0, back1, tunnel1 = 11.6, 11.9, 13.3
    cells = int((w - 2 * t) // CELL)
    landing = 1.2
    steps = round((P - KERB) / RISE)
    foot = t + landing + steps * RUN          # where the stair down ends, along the back
    floor_ = P + H                            # the roof's underside
    # The podium: solid under everything but the stair and tunnel along the back.
    m.prism(quad(a, u, v, 0.0, w, 0.0, back1), 0.0, P, trim)
    m.prism(quad(a, u, v, 0.0, t + landing, back1, tunnel1), 0.0, P, trim)
    for k in range(steps - 1):
        top = P - (k + 1) * RISE
        m.prism(quad(a, u, v, t + landing + k * RUN, t + landing + (k + 1) * RUN, back1, tunnel1), 0.0, top, m.steps)
    m.line(a + u * (t + landing) + v * (back1 + 0.05) + up * (P - 0.1), a + u * foot + v * (back1 + 0.05) + up * 0.3,
           red, width=0.05)
    # The tunnel's roof, from the stair's foot on, and all above it to the roof.
    m.prism(quad(a, u, v, foot, w, back1, tunnel1), 2.6, floor_, trim)
    m.line(a + u * foot + v * (back1 + tunnel1) / 2 + up * 2.55, a + u * (w - 0.2) + v * (back1 + tunnel1) / 2 + up * 2.55,
           red, width=0.06, across=-up)
    # The walls: front with its door, the sides - the far one with the tunnel's way out - and back.
    door_w, door_h = 2.4, 2.9
    d0, d1 = (w - door_w) / 2, (w + door_w) / 2
    for u0, u1 in ((0.0, d0), (d1, w)):
        m.prism(quad(a, u, v, u0, u1, 0.0, t), P, floor_, mat)
    m.prism(quad(a, u, v, d0, d1, 0.0, t), P + door_h, floor_, mat)
    m.prism(quad(a, u, v, 0.0, t, t, depth), 0.0, floor_, mat)
    m.prism(quad(a, u, v, w - t, w, t, back1), P, floor_, mat)
    m.prism(quad(a, u, v, w - t, w, tunnel1, depth), 0.0, floor_, mat)
    m.prism(quad(a, u, v, w - t, w, back1, tunnel1), 2.4, floor_, mat)
    m.prism(quad(a, u, v, t, w - t, tunnel1, depth), 0.0, floor_, mat)
    # The roof, and offices over it: a dark block stepped back, its windows cool.
    top = floor_ + 0.3
    m.prism(quad(a, u, v, -0.3, w + 0.3, -0.3, depth + 0.3), floor_, top, mat)
    over = top + rng.uniform(6.0, 12.0)
    body = quad(a, u, v, 1.5, w - 1.5, 2.0, depth - 1.0)
    m.prism(body, top, over, mat)
    m.facade_windows(body, top + 0.8, over, 3.0, "cool", rng, lit=0.35)
    m.ring(quad(a, u, v, -0.3, w + 0.3, -0.3, depth + 0.3), top + 0.02, red, width=0.1, light=False)
    m.ring(body, over + 0.02, cyan, width=0.08, light=False)
    m.fins(body, body, top, over, red)
    # The landing before the door, and the flight up to it from the street along the front, lit
    # red along its edge.
    out = PRECINCT_LANDING
    m.prism(quad(a, u, v, d0 - 0.6, d1 + 0.6, -out, 0.0), 0.0, P, trim)
    m.line(a + u * (d0 - 0.6) - v * (out + 0.04) + up * (P - 0.05), a + u * (d1 + 0.6) - v * (out + 0.04) + up * (P - 0.05),
           red, width=0.06, across=-v)
    for k in range(steps - 1):
        u0 = d1 + 0.6 + k * RUN
        m.prism(quad(a, u, v, u0, u0 + RUN, -out, -0.02), 0.0, P - (k + 1) * RISE, m.steps)
    m.line(a + u * (d1 + 0.6) - v * (out + 0.04) + up * (P - RISE), a + u * (d1 + 0.6 + (steps - 1) * RUN) - v * (out + 0.04) + up * 0.3,
           red, width=0.06, across=-v)
    # The badge over the door: a shield in cyan, a bar of red across it.
    badge = a + u * (w / 2) - v * 0.06 + up * (P + door_h + 0.4)
    shield = [badge + u * x + up * y for x, y in ((-0.6, 0.5), (0.6, 0.5), (0.6, -0.1), (0.0, -0.7), (-0.6, -0.1), (-0.6, 0.5))]
    for p_, q_ in zip(shield, shield[1:]):
        m.line(p_, q_, cyan, width=0.1, across=-v, light=False)
    m.line(badge + u * -0.45 + up * 0.15, badge + u * 0.45 + up * 0.15, red, width=0.1, across=-v, light=False)
    for k in (d0, d1):
        m.line(a + u * k - v * 0.06 + up * P, a + u * k - v * 0.06 + up * (P + door_h), cyan, width=0.08)
    # The hall: lit cyan; the desk across it, but for a way round at one end; the evidence locker
    # against the far side wall behind it.
    for k in (w * 0.3, w * 0.7):
        m.line(a + u * k + v * 1.0 + up * (floor_ - 0.36), a + u * k + v * (hall - 0.8) + up * (floor_ - 0.36), cyan,
               width=0.12)
    desk = 2.4
    m.prism(quad(a, u, v, t, w - t - 2.0, desk, desk + 0.7), P, P + 1.1, m.rail, col=m.cover)
    m.line(a + u * t + v * (desk - 0.03) + up * (P + 1.1), a + u * (w - t - 2.0) + v * (desk - 0.03) + up * (P + 1.1),
           red, width=0.05)
    spot = a + u * (w / 2) + v * (desk + 1.5)
    br_town.marker(f"precinct_{n}_desk", spot.x, spot.y, P, facing(-v))
    spot = a + u * (w / 2) + v * (desk - 1.1)
    br_town.marker(f"precinct_{n}", spot.x, spot.y, P, facing(v))
    locker = quad(a, u, v, t, t + 0.6, desk + 1.1, hall - 0.2)
    m.prism(locker, P, P + 2.0, m.facades["bronze"], col=m.cover)
    m.line(a + u * (t + 0.62) + v * (desk + 1.1) + up * (P + 1.0), a + u * (t + 0.62) + v * (hall - 0.2) + up * (P + 1.0),
           red, width=0.04, light=False)
    spot = a + u * (t + 1.1) + v * ((desk + 1.1 + hall - 0.2) / 2)
    br_town.marker(f"precinct_{n}_locker", spot.x, spot.y, P, facing(-u))
    # The gate's wall across, and its gate: a firewall, which the game makes glow.
    gw, gh = 1.4, 2.5
    g0, g1 = w / 2 - gw / 2, w / 2 + gw / 2
    for u0, u1 in ((t, g0), (g1, w - t)):
        m.prism(quad(a, u, v, u0, u1, gate0, gate1), P, floor_, mat)
    m.prism(quad(a, u, v, g0, g1, gate0, gate1), P + gh, floor_, mat)
    gate = quad(a, u, v, g0 + 0.02, g1 - 0.02, gate0 + 0.1, gate1 - 0.1)
    Nexus.mesh(m, *slab(gate, P + 0.02, P + gh - 0.02), m.struct, red).name = f"precinct_{n}_gate"
    for k in (g0 - 0.08, g1 + 0.08):
        m.line(a + u * k + v * (gate0 - 0.04) + up * P, a + u * k + v * (gate0 - 0.04) + up * (P + gh), red, width=0.08)
    # The corridor, lit red, where the guard keeps.
    m.line(a + u * 1.0 + v * (gate1 + bars0) / 2 + up * (floor_ - 0.3), a + u * (w - 1.0) + v * (gate1 + bars0) / 2 + up * (floor_ - 0.3),
           red, width=0.1)
    spot = a + u * (w / 2) + v * (gate1 + bars0) / 2
    br_town.marker(f"precinct_{n}_guard", spot.x, spot.y, P, facing(u))
    # The cells: walls between them, their back wall - the first's with its loose panel - and
    # each behind a firewall.
    end = t + cells * CELL
    m.prism(quad(a, u, v, end, w - t, bars0, back0), P, floor_, mat)
    panel0, panel1 = t + 0.1, t + landing
    m.prism(quad(a, u, v, t, panel0, back0, back1), P, floor_, mat)
    m.prism(quad(a, u, v, panel1, w - t, back0, back1), P, floor_, mat)
    m.prism(quad(a, u, v, panel0, panel1, back0, back1), P + 2.2, floor_, mat)
    panel = quad(a, u, v, panel0 + 0.02, panel1 - 0.02, back0 + 0.02, back1 - 0.02)
    Nexus.mesh(m, *slab(panel, P + 0.02, P + 2.18), m.struct, trim).name = f"tunnel_panel_{n}"
    spot = a + u * ((panel0 + panel1) / 2) + v * (back0 - 0.5)
    br_town.marker(f"tunnel_{n}", spot.x, spot.y, P, facing(v))
    spot = a + u * (w + 0.8) + v * ((back1 + tunnel1) / 2)
    br_town.marker(f"tunnel_{n}_exit", spot.x, spot.y, KERB, facing(u))
    hatch(m, a + u * ((foot + w) / 2 - 0.5) + v * ((back1 + tunnel1) / 2), KERB, f"precinct_{n}")
    m.prism(quad(a, u, v, end, w - t, bars0, bars1), P, floor_, mat)
    fw = P + 2.6                              # a firewall's top, under its emitter
    for c in range(cells):
        c0, c1 = t + c * CELL, t + (c + 1) * CELL
        if c > 0:
            m.prism(quad(a, u, v, c0 - 0.1, c0 + 0.1, bars0, back0), P, floor_, mat)
        cm = (c0 + c1) / 2
        f0, f1 = c0 + (0.1 if c > 0 else 0.0), c1 - (0.1 if c < cells - 1 else 0.0)
        # The firewall across its front, between its emitters - a sill in the floor and a beam
        # over it, lit red - which the game makes glow, and drops when the cell is opened.
        m.prism(quad(a, u, v, f0, f1, bars0 - 0.05, bars1 + 0.05), P, P + 0.06, m.rail)
        m.prism(quad(a, u, v, f0, f1, bars0 - 0.1, bars1 + 0.1), fw, floor_, m.rail)
        for z in (P + 0.06, fw):
            m.line(a + u * f0 + v * (bars0 - 0.12) + up * z, a + u * f1 + v * (bars0 - 0.12) + up * z, red, width=0.04,
                   light=False)
        wall = quad(a, u, v, f0, f1, bars0 + 0.02, bars1 - 0.02)
        Nexus.mesh(m, *slab(wall, P + 0.06, fw), m.struct, red).name = f"cell_wall_{n}_{c + 1}"
        # Its bunk along the back, and a dim light over it.
        m.prism(quad(a, u, v, c0 + 0.2, c0 + 2.1, back0 - 0.8, back0), P, P + 0.45, m.rail, col=m.cover)
        m.line(a + u * (c0 + 0.4) + v * (back0 - 0.4) + up * (floor_ - 0.2), a + u * (c1 - 0.4) + v * (back0 - 0.4) + up * (floor_ - 0.2),
               m.white, width=0.05)
        spot = a + u * cm + v * ((bars1 + back0) / 2)
        br_town.marker(f"cell_{n}_{c + 1}", spot.x, spot.y, P, facing(-v))
        # A drain in its floor, past the bunk's foot: a dark pit under a grate lit cyan, which the
        # game hides once it is pried up, and drops the player through into the escape tunnels.
        gu, gv, gr = c0 + 2.55, back0 - 0.5, 0.3
        m.prism(quad(a, u, v, gu - gr, gu + gr, gv - gr, gv + gr), P, P + 0.005, m.facades["slate"])
        bars = [quad(a, u, v, gu - gr, gu + gr, gv + k - 0.025, gv + k + 0.025) for k in (-gr + 0.025, -0.1, 0.1, gr - 0.025)]
        bars += [quad(a, u, v, gu + k - 0.025, gu + k + 0.025, gv - gr, gv + gr) for k in (-gr + 0.025, gr - 0.025)]
        verts, faces = [], []
        for bar_ in bars:
            vs, fs = slab(bar_, P, P + 0.03)
            faces += [tuple(len(verts) + i for i in f) for f in fs]
            verts += vs
        Nexus.mesh(m, verts, faces, m.struct, cyan).name = f"cell_grate_{n}_{c + 1}"
        spot = a + u * gu + v * gv
        br_town.marker(f"grate_{n}_{c + 1}", spot.x, spot.y, P, facing(-v))



def hatch(m, at, z, name):
    """A hatch in the floor at `at`, `z` up - a round plate ringed in cyan, with a grip across it - and
    hatch_<name> on it, for the game to take the player down through to its other end."""
    plate = polygon(0.55, 16, 0, at.x, at.y)
    m.prism(plate, z, z + 0.03, m.rail)
    m.ring(polygon(0.55, 16, 0, at.x, at.y), z + 0.035, m.lines["cyan"], out=0.0, width=0.05, light=False)
    m.line(Vector((at.x - 0.3, at.y, z + 0.06)), Vector((at.x + 0.3, at.y, z + 0.06)), m.lines["cyan"], width=0.05, light=False)
    br_town.marker(f"hatch_{name}", at.x, at.y, z, 0.0)


def warren(m, rects, z0, mat, light, edge):
    """Rooms and passages on one floor at z0, each a rectangle (x0, x1, y0, y1, high) touching the
    ones it opens onto edge to edge: a floor and a ceiling for each, and walls round each but
    where another opens onto it - over which the taller has a lintel down to the lower's ceiling.
    Each lit along its middle, under its ceiling; and each wall outlined in `edge` along its foot
    and its top, as the city's are, to see it by."""
    t, eps = 0.4, 1e-3
    for x0, x1, y0, y1, high in rects:
        m.box(WORLD, x0, x1, y0, y1, z0 - t, z0, m.struct, mat)
        m.box(WORLD, x0, x1, y0, y1, z0 + high, z0 + high + t, m.struct, mat)
        # Each edge: the line it is on, which way out is, its extent along it.
        edges = (("y", y0, -1, x0, x1), ("y", y1, 1, x0, x1), ("x", x0, -1, y0, y1), ("x", x1, 1, y0, y1))
        for axis, at, out, a, b in edges:
            opens = []
            for ox0, ox1, oy0, oy1, ohigh in rects:
                if axis == "y" and abs((oy1 if out < 0 else oy0) - at) < eps:
                    lo, hi = max(a, ox0), min(b, ox1)
                elif axis == "x" and abs((ox1 if out < 0 else ox0) - at) < eps:
                    lo, hi = max(a, oy0), min(b, oy1)
                else:
                    continue
                if hi - lo > eps:
                    opens.append((lo, hi, ohigh))
            opens.sort()
            walls, k = [], a
            for lo, hi, ohigh in opens:
                if lo > k:
                    walls.append((k, lo, z0 - t, z0 + high + t))
                if ohigh < high:
                    walls.append((lo, hi, z0 + ohigh, z0 + high + t))
                k = max(k, hi)
            if k < b:
                walls.append((k, b, z0 - t, z0 + high + t))
            for lo, hi, za, zb in walls:
                # Its foot and its top outlined, just in from its face.
                inset = -out * 0.03
                for zl in {za + t + 0.02 if za < z0 else za - 0.02, z0 + high - 0.04}:
                    if zl < za - 0.03 or zl > z0 + high:
                        continue
                    if axis == "y":
                        m.line(Vector((lo, at + inset, zl)), Vector((hi, at + inset, zl)), edge, width=0.04, light=False)
                    else:
                        m.line(Vector((at + inset, lo, zl)), Vector((at + inset, hi, zl)), edge, width=0.04, light=False)
                # Out to the corners at the rectangle's own ends, to close them.
                lo -= t if abs(lo - a) < eps else 0.0
                hi += t if abs(hi - b) < eps else 0.0
                if axis == "y":
                    m.box(WORLD, lo, hi, *sorted((at, at + out * t)), za, zb, m.struct, mat)
                else:
                    m.box(WORLD, *sorted((at, at + out * t)), lo, hi, za, zb, m.struct, mat)
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
        if x1 - x0 >= y1 - y0:
            m.line(Vector((x0 + 0.5, cy, z0 + high - 0.08)), Vector((x1 - 0.5, cy, z0 + high - 0.08)), light, width=0.06)
        else:
            m.line(Vector((cx, y0 + 0.5, z0 + high - 0.08)), Vector((cx, y1 - 0.5, z0 + high - 0.08)), light, width=0.06)


def escape_tunnels(m):
    """The escape tunnels, hung in the void well under the city at UNDER, out of sight: the den in
    the middle, where a gang keeps and the fence deals; a way out from each of its sides to a
    ring; off the ring's outsides, twelve dead ends, each with a hatch up to its other end in the
    city; and off each way out, a side room with a cache of credits."""
    z = UNDER
    stone = m.facades["concrete"]
    rects = [(-10.0, 10.0, -10.0, 10.0, 5.0)]
    ring, wide, way, high = 45.0, 3.0, 1.5, 3.2
    r0, r1 = ring - wide / 2, ring + wide / 2
    # The ring: its north and south sides the full width, its east and west between them.
    rects += [(-r1, r1, r0, r1, high), (-r1, r1, -r1, -r0, high), (r0, r1, -r0, r0, high), (-r1, -r0, -r0, r0, high)]
    # The ways out, from the den's sides to the ring.
    rects += [(-way, way, 10.0, r0, high), (-way, way, -r0, -10.0, high), (10.0, r0, -way, way, high), (-r0, -10.0, -way, way, high)]
    # The dead ends off the ring, and which hatch is at each: three a side.
    names = {"n": ("precinct_1", "bank_1", "bank_2"), "e": ("precinct_2", "bank_3", "bank_4"),
             "s": ("precinct_3", "bank_5", "bank_6"), "w": ("street_1", "street_2", "street_3")}
    ends = []
    for side, (dx, dy) in (("n", (0, 1)), ("e", (1, 0)), ("s", (0, -1)), ("w", (-1, 0))):
        for k, along in enumerate((-25.0, 0.0, 25.0)):
            if dy:
                y0, y1 = sorted((dy * r1, dy * (r1 + 5.0)))
                rects.append((along - 2.0, along + 2.0, y0, y1, high))
                spot = Vector((along, dy * (r1 + 4.0), 0))
            else:
                x0, x1 = sorted((dx * r1, dx * (r1 + 5.0)))
                rects.append((x0, x1, along - 2.0, along + 2.0, high))
                spot = Vector((dx * (r1 + 4.0), along, 0))
            ends.append((names[side][k], spot))
    # A side room off each way out, through a doorway, with a cache in it.
    rooms = []
    for (dx, dy) in ((0, 1), (0, -1), (1, 0), (-1, 0)):
        mid = 26.0
        if dy:
            a, b = sorted((dy * (mid - 1.0), dy * (mid + 1.0)))
            rects.append((way, way + 1.0, a, b, 2.6))
            c0, c1 = sorted((dy * (mid - 3.0), dy * (mid + 3.0)))
            rects.append((way + 1.0, way + 7.0, c0, c1, high))
            rooms.append(Vector((way + 5.0, dy * mid, 0)))
        else:
            a, b = sorted((dx * (mid - 1.0), dx * (mid + 1.0)))
            rects.append((a, b, way, way + 1.0, 2.6))
            c0, c1 = sorted((dx * (mid - 3.0), dx * (mid + 3.0)))
            rects.append((c0, c1, way + 1.0, way + 7.0, high))
            rooms.append(Vector((dx * mid, way + 5.0, 0)))
    warren(m, rects, z, stone, m.lines["orange"], m.lines["cyan"])
    # Pipes along the ring's walls, lit dim green where they run.
    for y in (r0 + 0.25, -r0 - 0.25):
        m.line(Vector((-r1, y, z + 2.4)), Vector((r1, y, z + 2.4)), m.facades["rust"], width=0.18, light=False)
    # A hatch at the end of each dead end: a rung ladder up the wall to a ring in the ceiling.
    for name, spot in ends:
        m.ring(polygon(0.55, 16, 0, spot.x, spot.y), z + high - 0.04, m.lines["cyan"], out=0.0, width=0.06, light=False)
        for k in range(1, 6):
            m.line(Vector((spot.x - 0.3, spot.y, z + k * 0.5)), Vector((spot.x + 0.3, spot.y, z + k * 0.5)), m.rail,
                   width=0.05, light=False)
        br_town.marker(f"hatch_{name}_below", spot.x, spot.y, z, 0.0)
    # The caches: a crate each, banded gold.
    for k, spot in enumerate(rooms, 1):
        box = [Vector((spot.x - 0.45, spot.y - 0.35, 0)), Vector((spot.x + 0.45, spot.y - 0.35, 0)),
               Vector((spot.x + 0.45, spot.y + 0.35, 0)), Vector((spot.x - 0.45, spot.y + 0.35, 0))]
        Nexus.mesh(m, *slab(box, z, z + 0.6), m.struct, m.facades["bronze"]).name = f"cache_box_{k}"
        m.ring(box, z + 0.3, m.lines["yellow"], out=0.02, width=0.04, light=False)
        br_town.marker(f"cache_{k}", spot.x, spot.y, z, 0.0)
    # The den: tables about it, the fence's counter at its back, lit warm, and where the gang keeps.
    for tx, ty in ((-5.0, -4.0), (5.0, -4.0), (-5.0, 4.0)):
        m.box(WORLD, tx - 0.8, tx + 0.8, ty - 0.5, ty + 0.5, z, z + 0.9, m.cover, m.rail)
        m.line(Vector((tx - 0.8, ty - 0.52, z + 0.9)), Vector((tx + 0.8, ty - 0.52, z + 0.9)), m.lines["red"], width=0.04)
    m.box(WORLD, 3.0, 8.0, 6.0, 7.0, z, z + 1.1, m.cover, m.facades["bronze"])
    m.line(Vector((3.0, 5.97, z + 1.1)), Vector((8.0, 5.97, z + 1.1)), m.lines["yellow"], width=0.05)
    br_town.marker("fence", 5.5, 5.2, z, -math.pi / 2)
    for k, (dx, dy) in enumerate(((-3.0, 0.0), (3.0, -1.0), (-7.0, 7.0), (7.0, 2.0)), 1):
        br_town.marker(f"den_{k}", dx, dy, z, 0.0)
    m.ring(polygon(9.5, 4, math.pi / 4), z + 4.6, m.lines["red"], out=0.0, width=0.08)

def townhouses(m, rng, poly, district, street_faces):
    """A row of narrow houses, each its own height and colour, a lit door and windows each."""
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    along_x = (x1 - x0) >= (y1 - y0)
    n = max(2, int(((x1 - x0) if along_x else (y1 - y0)) / 6.5))
    shade = m.shade()
    for i in range(n):
        if along_x:
            a, b = x0 + (x1 - x0) * i / n, x0 + (x1 - x0) * (i + 1) / n
            p = rect(a, b - 0.3, y0, y1)
        else:
            a, b = y0 + (y1 - y0) * i / n, y0 + (y1 - y0) * (i + 1) / n
            p = rect(x0, x1, a, b - 0.3)
        h = rng.choice((6.4, 9.6, 9.6, 12.8))
        mat = facade(m, rng, district)
        m.prism(p, 0.0, h, mat)
        m.facade_windows(p, 0.0, h, 3.2, "warm", rng, lit=0.5)
        m.ring(p, h - 0.05, m.lines[shade], width=0.06, light=False)
        m.fins(p, p, 0.0, h, m.lines[shade])
        if rng.random() < 0.4:
            m.prism(shrink(p, 1.5), h, h + 2.2, m.facades["slate"])


def terraces(m, rng, poly, district, street_faces):
    """Terraced housing: tiers stepping back from the street, a garden on each terrace, climbed
    by a flight up to each from the one below, the first from a front yard."""
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    side = street_faces[0] if street_faces else 0
    # Work in a frame whose +y runs inward from the street face, and whose +x along it.
    angle = {0: 0.0, 1: math.pi / 2, 2: math.pi, 3: -math.pi / 2}[side]
    c = middle(poly)
    f = Frame(c.x, c.y, angle)
    w = (x1 - x0) if side in (0, 2) else (y1 - y0)
    d = (y1 - y0) if side in (0, 2) else (x1 - x0)
    hw, hd = w / 2, d / 2
    tier = 3.5
    n = round(tier / RISE)
    run = n * RUN
    yard = run + 0.4
    tiers = min(3, int((d - yard) / (run + 0.6)))
    if tiers < 1 or hw < 4.0:
        apartments(m, rng, poly, district, street_faces)
        return
    each = (d - yard) / tiers
    mat, shade = facade(m, rng, district), m.shade()
    front = -hd + yard
    for t in range(tiers):
        ya = front + t * each
        m.box(f, -hw, hw, ya, hd, t * tier, (t + 1) * tier, m.struct, mat)
        face = [f.at(-hw, ya), f.at(hw, ya), f.at(hw, hd), f.at(-hw, hd)]
        m.facade_windows(face, t * tier, (t + 1) * tier, tier, "warm", rng, lit=0.55, faces=[0])
        m.fins(face, face, t * tier, (t + 1) * tier, m.lines[shade])
        m.line(f.at(-hw, ya - 0.06, (t + 1) * tier - 0.05), f.at(hw, ya - 0.06, (t + 1) * tier - 0.05),
               m.lines[shade])
        # Its flight, up to its front from the terrace below (or the yard), at one end or the other.
        xa, xb = (-hw + 0.5, -hw + 2.1) if t % 2 == 0 else (hw - 2.1, hw - 0.5)
        g = Frame(f.origin.x, f.origin.y, angle + math.pi / 2)   # local +x is f's +y, local y f's -x
        m.flight(g, ya - run, -xb, -xa, t * tier, (t + 1) * tier, +1, shade)
        if t > 0:
            q = f.at(0.0, ya - each / 2)
            m.tree(q.x, q.y, rng, z=t * tier, size=0.8)


def industrial(m, rng, poly, district):
    """Works: a long shed with a sawtooth roof, a cooling tower or two, and pipes."""
    x0, y0, x1, y1 = poly[0].x, poly[0].y, poly[2].x, poly[2].y
    h = rng.uniform(7.0, 11.0)
    mat, shade = facade(m, rng, "docks"), rng.choice(("orange", "yellow", "red"))
    along_x = (x1 - x0) >= (y1 - y0)
    split = 0.65
    if along_x:
        shed = rect(x0, x0 + (x1 - x0) * split, y0, y1)
        yard = (x0 + (x1 - x0) * split + 2, x1, y0, y1)
    else:
        shed = rect(x0, x1, y0, y0 + (y1 - y0) * split)
        yard = (x0, x1, y0 + (y1 - y0) * split + 2, y1)
    m.prism(shed, 0.0, h, mat)
    m.ring(shed, h - 0.05, m.lines[shade])
    m.fins(shed, shed, 0.0, h, m.lines[shade])
    m.facade_windows(shed, 0.0, h, h, "office", rng, lit=0.4, bands=True)
    # The sawtooth roof: prisms across the shed.
    sx0, sy0, sx1, sy1 = shed[0].x, shed[0].y, shed[2].x, shed[2].y
    teeth = max(2, int(((sx1 - sx0) if along_x else (sy1 - sy0)) / 5.0))
    for k in range(teeth):
        if along_x:
            a = sx0 + (sx1 - sx0) * k / teeth
            b = sx0 + (sx1 - sx0) * (k + 1) / teeth
            verts = [Vector((a, sy0, h)), Vector((b, sy0, h)), Vector((b, sy0, h + 2.5)),
                     Vector((a, sy1, h)), Vector((b, sy1, h)), Vector((b, sy1, h + 2.5))]
            m.line(Vector((b - 0.08, sy0, h + 1.25)), Vector((b - 0.08, sy1, h + 1.25)), m.windows["office"],
                   width=2.0, height=0.06, across=Z, light=False)
        else:
            a = sy0 + (sy1 - sy0) * k / teeth
            b = sy0 + (sy1 - sy0) * (k + 1) / teeth
            verts = [Vector((sx0, a, h)), Vector((sx0, b, h)), Vector((sx0, b, h + 2.5)),
                     Vector((sx1, a, h)), Vector((sx1, b, h)), Vector((sx1, b, h + 2.5))]
            m.line(Vector((sx0, b - 0.08, h + 1.25)), Vector((sx1, b - 0.08, h + 1.25)), m.windows["office"],
                   width=2.0, height=0.06, across=Z, light=False)
        m.mesh(verts, [(0, 1, 2), (3, 5, 4), (0, 3, 4, 1), (1, 4, 5, 2), (0, 2, 5, 3)], m.struct, m.facades["slate"])
    # The yard: cooling towers, pipes between them and the shed.
    yx0, yx1, yy0, yy1 = yard
    cx, cy = (yx0 + yx1) / 2, (yy0 + yy1) / 2
    r = min(yx1 - yx0, yy1 - yy0) / 2 - 1.0
    if r > 3.0:
        tall_ = rng.uniform(14.0, 22.0)
        ring_lo = polygon(r, 16, 0, cx, cy)
        waist = polygon(r * 0.65, 16, 0, cx, cy)
        lip = polygon(r * 0.78, 16, 0, cx, cy)
        m.prism(ring_lo, 0.0, tall_ * 0.65, m.facades["concrete"], top=waist)
        m.prism(waist, tall_ * 0.65, tall_, m.facades["concrete"], top=lip)
        m.ring(lip, tall_, m.lines[shade])
        m.ring(ring_lo, 0.5, m.lines[shade])
    for k in range(3):
        z = 4.0 + k * 0.9
        if along_x:
            m.box(WORLD, sx1, yx0 + 1.0, cy - 0.3 + k, cy + 0.3 + k, z, z + 0.6, m.struct, m.facades["rust"])
        else:
            m.box(WORLD, cx - 0.3 + k, cx + 0.3 + k, sy1, yy0 + 1.0, z, z + 0.6, m.struct, m.facades["rust"])


def warehouse(m, rng, poly, district):
    """A warehouse: a big plain shed with roller doors lit along its street face."""
    h = rng.uniform(8.0, 13.0)
    mat, shade = facade(m, rng, "docks"), rng.choice(("orange", "yellow", "cyan"))
    m.prism(poly, 0.0, h, mat, top=shrink(poly, 0.8))
    m.fins(poly, shrink(poly, 0.8), 0.0, h, m.lines[shade])
    m.ring(poly, 0.4, m.lines[shade])
    for i in range(4):
        a, b = poly[i], poly[(i + 1) % 4]
        e = b - a
        u = e.normalized()
        out = Vector((u.y, -u.x, 0))
        if out.dot((a + b) / 2 - middle(poly)) < 0:
            out = -out
        for k in range(int(e.length / 9.0)):
            p = a + u * (2.0 + k * 9.0) + out * 0.06
            m.line(p + Vector((0, 0, 2.2)), p + u * 4.0 + Vector((0, 0, 2.2)), m.lines[shade] if k % 2 else m.rail,
                   width=4.4, height=0.06, across=Z, light=False)
    c = middle(poly)
    m.ring(shrink(poly, 0.8), h, m.lines[shade], light=False)
    m.mast(c.x, c.y, h, 4.0, shade)


# The districts' lots, and what goes up on them.

def district_of(i, j):
    x, y = block_middle(i, j)
    if y < CANAL[0]:
        return "docks"
    r = max(abs(x), abs(y))
    return "downtown" if r < 40 else "midtown" if r < 200 else "outer" if r < 300 else "suburbs"


def block_middle(i, j):
    return (i - (BLOCKS - 1) / 2) * PITCH, (j - (BLOCKS - 1) / 2) * PITCH


def lots(rng, district, x0, x1, y0, y1):
    """The block's buildable ground, split into lots with alleys between them."""
    alley = 2.0
    xm, ym = (x0 + x1) / 2, (y0 + y1) / 2
    choices = {
        "downtown": ("whole", "halves", "halves", "half_quarters"),
        "midtown": ("halves", "quarters", "half_quarters", "quarters", "thirds", "sixths"),
        "outer": ("quarters", "sixths", "sixths", "half_quarters", "thirds", "ninths"),
        "suburbs": ("sixths", "ninths", "ninths", "quarters"),
        "docks": ("halves", "quarters", "thirds", "whole"),
    }[district]
    how = rng.choice(choices)
    a = alley / 2
    if how == "whole":
        return [(x0, x1, y0, y1)]
    if how == "halves":
        if rng.random() < 0.5:
            return [(x0, xm - a, y0, y1), (xm + a, x1, y0, y1)]
        return [(x0, x1, y0, ym - a), (x0, x1, ym + a, y1)]
    if how == "quarters":
        return [(x0, xm - a, y0, ym - a), (xm + a, x1, y0, ym - a), (x0, xm - a, ym + a, y1), (xm + a, x1, ym + a, y1)]
    if how in ("sixths", "ninths"):
        across = 3
        down = 2 if how == "sixths" else 3
        out = []
        for i in range(across):
            for j in range(down):
                xa = x0 + (x1 - x0) * i / across + (a if i else 0)
                xb = x0 + (x1 - x0) * (i + 1) / across - (a if i < across - 1 else 0)
                ya = y0 + (y1 - y0) * j / down + (a if j else 0)
                yb = y0 + (y1 - y0) * (j + 1) / down - (a if j < down - 1 else 0)
                out.append((xa, xb, ya, yb))
        # The middle lot of nine faces no street: a yard, with a tree.
        return out
    if how == "thirds":
        t1, t2 = x0 + (x1 - x0) / 3, x0 + 2 * (x1 - x0) / 3
        return [(x0, t1 - a, y0, y1), (t1 + a, t2 - a, y0, y1), (t2 + a, x1, y0, y1)]
    # Half and two quarters.
    return [(x0, xm - a, y0, y1), (xm + a, x1, y0, ym - a), (xm + a, x1, ym + a, y1)]


def faces_on_street(lot, block):
    """Which of a lot's faces - 0 south, 1 east, 2 north, 3 west - face a street."""
    x0, x1, y0, y1 = lot
    bx0, bx1, by0, by1 = block
    out = []
    if abs(y0 - by0) < 0.01:
        out.append(0)
    if abs(x1 - bx1) < 0.01:
        out.append(1)
    if abs(y1 - by1) < 0.01:
        out.append(2)
    if abs(x0 - bx0) < 0.01:
        out.append(3)
    return out


def build_lot(m, rng, district, lot, block):
    x0, x1, y0, y1 = lot
    street = faces_on_street(lot, block)
    rng.shuffle(street)
    w, d = x1 - x0, y1 - y0
    inset = rng.uniform(0.0, 1.5)
    poly = rect(x0 + inset, x1 - inset, y0 + inset, y1 - inset)
    small = min(w, d)
    big = max(w, d) > 30 and small > 30
    options = {
        "downtown": [setback_tower] * 3 + [round_tower] * 2 + [twist_tower] * 2 + [tapered_tower, wedge_tower]
        + [podium_tower] * 3 + ([twin_towers] * 2 if big else []),
        "midtown": [setback_tower, round_tower, podium_tower, wedge_tower, stilts, stilts]
        + [apartments] * 3 + [courtyard] * (2 if small > 18 else 0) + [shops] * 2,
        "outer": [apartments] * 3 + [townhouses] * 3 + [shops] * 2 + [terraces] * (2 if small > 18 else 0)
        + [courtyard] * (1 if small > 18 else 0) + [stilts],
        "suburbs": [townhouses] * 4 + [terraces] * (2 if small > 18 else 0) + [apartments, shops],
        "docks": [industrial] * 3 + [warehouse] * 3 + [shops, townhouses],
    }[district]
    if not street and small < 16:
        c = middle(poly)
        m.tree(c.x, c.y, rng, size=1.2)
        return
    build = rng.choice(options)
    if m.precinct_here and street and precinct_fits(lot, street):
        m.precinct_here = False
        precinct(m, poly, street)
        m.n_buildings += 1
        return
    if m.bank_here and street and small >= BANK_LOT:
        m.bank_here = False
        bank(m, poly, street)
        m.n_buildings += 1
        return
    towers = (setback_tower, round_tower, twist_tower, tapered_tower, wedge_tower, podium_tower, stilts)
    if (build in towers and street and small >= OFFICE_SIZE[0] and district in ("downtown", "midtown")
            and m.lifts.random() < OFFICE_CHANCE):
        # A tower to go up in its place: the one it stands for built out of sight, for its draws of
        # chance, so that the rest of the city is as it was.
        m.unseen(lambda: build(m, rng, poly, district))
        office_tower(m, m.lifts, poly, street, facade(m, m.lifts, district), unpainted_shade(m))
        m.n_buildings += 1
        return
    if build in (apartments, courtyard, shops, townhouses, terraces):
        build(m, rng, poly, district, street)
    else:
        build(m, rng, poly, district)
    m.n_buildings += 1


# The ground: the platform, the canal, streets and blocks.

def ground(m):
    cy, cw, cd = CANAL
    south, north = cy - cw / 2, cy + cw / 2
    for y0, y1 in ((-HALF, south), (north, HALF)):
        slab = rect(-HALF, HALF, y0, y1)
        m.solid(slab, -DEPTH, -0.002, m.ground, m.road)
        m.ring(slab, -0.3, m.lines["cyan"], out=0.0, light=False)
        m.ring(slab, -DEPTH + 0.15, m.lines["cyan"], out=0.0, light=False)
    # The canal's water, its walls, and steps down to it.
    m.box(WORLD, -HALF, HALF, south, north, -DEPTH, -cd, m.ground, m.water)
    for y in (south + 0.05, north - 0.05):
        m.line(Vector((-HALF, y, -cd + 0.05)), Vector((HALF, y, -cd + 0.05)), m.lines["cyan"], width=0.08)
    xs = [-HALF + 0.3] + [k * PITCH for k in range(-BLOCKS // 2, BLOCKS // 2 + 1)] + [HALF - 0.3]
    bridges = sorted(set(round(x, 3) for x in xs[1:-1]))
    bridges = [max(-HALF + BRIDGE_WIDTH / 2 + 0.5, min(HALF - BRIDGE_WIDTH / 2 - 0.5, x)) for x in bridges]
    for x in bridges:
        b0, b1 = x - BRIDGE_WIDTH / 2, x + BRIDGE_WIDTH / 2
        m.box(WORLD, b0, b1, south - 0.3, north + 0.3, -0.6, -0.002, m.ground, m.road)
        for bx in (b0 + 0.2, b1 - 0.2):
            m.parapet(Vector((bx, south, 0)), Vector((bx, north, 0)), 0.0, m.lines["cyan"])
        m.line(Vector((b0, south, -0.62)), Vector((b1, south, -0.62)), m.lines["cyan"], light=False)
        m.line(Vector((b0, north, -0.62)), Vector((b1, north, -0.62)), m.lines["cyan"], light=False)
    # The quays' parapets, broken by the bridges and the steps down.
    spans = []
    edges = [-HALF] + [v for x in bridges for v in (x - BRIDGE_WIDTH / 2, x + BRIDGE_WIDTH / 2)] + [HALF]
    for k in range(0, len(edges), 2):
        spans.append((edges[k], edges[k + 1]))
    n_steps = round(cd / RISE)
    for s0, s1 in spans:
        if s1 - s0 < 4:
            continue
        mid_x = (s0 + s1) / 2
        for y, way in ((south, +1), (north, -1)):
            gap_lo, gap_hi = mid_x - 1.0, mid_x + 1.0
            for a, b in ((s0, gap_lo), (gap_hi, s1)):
                m.parapet(Vector((a, y - way * 0.2, 0)), Vector((b, y - way * 0.2, 0)), 0.0, m.lines["cyan"])
            # Steps down to the water, out from the quay wall, through the gap.
            for i in range(n_steps - 1):
                top = -RISE * (i + 1)
                ya = y + way * i * RUN
                m.box(WORLD, gap_lo + 0.2, gap_hi - 0.2, ya, ya + way * RUN, -cd, top, m.ground, m.steps)
    # The edge of the platform: a parapet round it, but for the gates east and west, and the middle
    # avenue, which goes on over a bridge in the south to Caelum (data/story/story_caelum.py) and in
    # the north to Curia (data/story/story_curia.py).
    corners = rect(-HALF, HALF, -HALF, HALF)
    for i in range(4):
        a, b = corners[i], corners[(i + 1) % 4]
        inward = (middle(corners) - (a + b) / 2).normalized() * 0.3
        gaps = []
        if i in (1, 3):    # east and west
            g = GATE[1] / 2 / (b - a).length
            gaps = [(0.5 - g, 0.5 + g)]
        if i in (0, 2):    # south and north
            g = STREET / 2 / (b - a).length
            gaps = [(0.5 - g, 0.5 + g)]
        # The canal crosses the east and west edges too.
        if i in (1, 3):
            lo = (south - a.y) / (b.y - a.y)
            hi = (north - a.y) / (b.y - a.y)
            gaps.append((min(lo, hi), max(lo, hi)))
        m.parapet(a + inward, b + inward, 0.0, m.lines["cyan"], gaps)


def streets(m, rng):
    """Dashed lines down the middle of every street, crossings at the junctions."""
    cy, cw, _ = CANAL
    lines_at = [k * PITCH for k in range(-BLOCKS // 2 + 1, BLOCKS // 2)]
    ring = HALF - STREET / 2
    for c in lines_at + [-ring, ring]:
        for horizontal in (True, False):
            t = -HALF + 2.0
            while t < HALF - 2.0:
                near_junction = any(abs(t - j) < STREET / 2 + 2 for j in lines_at + [-ring, ring])
                if not near_junction and not (horizontal is False and cy - cw / 2 - 1 < t < cy + cw / 2 + 1):
                    if horizontal and not (abs(c - cy) < 9):
                        m.line(Vector((t, c, 0.004)), Vector((t + 3.0, c, 0.004)), m.marking, width=0.15,
                               height=0.008, light=False)
                    elif not horizontal:
                        m.line(Vector((c, t, 0.004)), Vector((c, t + 3.0, 0.004)), m.marking, width=0.15,
                               height=0.008, light=False)
                t += 6.0
    # Crossings: stripes across each street at each side of each junction.
    for jx in lines_at + [-ring, ring]:
        for jy in lines_at + [-ring, ring]:
            if abs(jy - cy) < 9:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                ox, oy = jx + dx * (STREET / 2 + 1.5), jy + dy * (STREET / 2 + 1.5)
                if abs(ox) > HALF - 1 or abs(oy) > HALF - 1:
                    continue
                for k in range(-3, 4):
                    if dx:
                        p = Vector((ox - 1.0, oy + k * 2.0, 0.004))
                        m.line(p, p + Vector((2.0, 0, 0)), m.marking, width=0.9, height=0.008, light=False)
                    else:
                        p = Vector((ox + k * 2.0, oy - 1.0, 0.004))
                        m.line(p, p + Vector((0, 2.0, 0)), m.marking, width=0.9, height=0.008, light=False)


def road_plan():
    """Where the roads are, for the game to keep its droids off them but to cross: every street,
    kerb to kerb, but the canal's - its quays and bridges are walked along - and the crossings at
    the junctions, where droids may step onto the road. Each a rectangle, [x0, z0, x1, z1] in the
    game's terms (Blender's x, and -y); roads only where the floor is lower than `level`, under
    the kerb, so nothing built over a street counts."""
    cy, _, _ = CANAL
    game = lambda x0, x1, y0, y1: [round(x0, 3), round(-y1, 3), round(x1, 3), round(-y0, 3)]
    lines_at = [k * PITCH for k in range(-BLOCKS // 2 + 1, BLOCKS // 2)]
    ring = HALF - STREET / 2
    every = lines_at + [-ring, ring]
    roads, crossings = [], []
    for c in every:
        roads.append(game(c - STREET / 2, c + STREET / 2, -HALF, HALF))
        if abs(c - cy) >= 9:
            roads.append(game(-HALF, HALF, c - STREET / 2, c + STREET / 2))
    # The canal's bridges, where its street crosses the others.
    for c in every:
        crossings.append(game(c - STREET / 2, c + STREET / 2, cy - STREET / 2, cy + STREET / 2))
    # As streets() stripes them: across each street, either side of each junction.
    for jx in every:
        for jy in every:
            if abs(jy - cy) < 9:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                ox, oy = jx + dx * (STREET / 2 + 1.5), jy + dy * (STREET / 2 + 1.5)
                if abs(ox) > HALF - 1 or abs(oy) > HALF - 1:
                    continue
                if dx:
                    crossings.append(game(ox - 1.0, ox + 1.0, jy - STREET / 2, jy + STREET / 2))
                else:
                    crossings.append(game(jx - STREET / 2, jx + STREET / 2, oy - 1.0, oy + 1.0))
    return {"level": KERB / 2, "roads": roads, "crossings": crossings}


def block(m, rng, i, j):
    """A block: its pavement, its lamps and trees, and what stands on it."""
    cx, cy = block_middle(i, j)
    h = PITCH / 2 - STREET / 2
    x0, x1, y0, y1 = cx - h, cx + h, cy - h, cy + h
    pave = rect(x0, x1, y0, y1)
    district = district_of(i, j)
    special = SPECIAL.get((i, j))
    parks = (park, park_east, park_north, park_far_west, park_far_east, park_far_north)
    m.solid(pave, 0.0, KERB, m.ground, m.grass if special in parks else m.pavement)
    m.ring(pave, KERB - 0.02, m.lines["cyan"] if district != "docks" else m.lines["orange"], out=0.03,
           width=0.06, light=False)
    # Lamps at two corners, trees along the pavement where it is not downtown.
    for sx, sy in ((-1, -1), (1, 1)) if (i + j) % 2 else ((1, -1), (-1, 1)):
        m.pylon(WORLD, cx + sx * (h - 1.5), cy + sy * (h - 1.5), KERB, height=6.0, shade=m.shade())
    if special is None and district != "docks":
        for k in range(1, 4):
            t = -h + k * (2 * h) / 4
            for x, y in ((cx + t, y0 + 1.5), (cx + t, y1 - 1.5), (x0 + 1.5, cy + t), (x1 - 1.5, cy + t)):
                if rng.random() < 0.6:
                    m.tree(x, y, rng, size=0.9)
    inner = (x0 + SIDEWALK, x1 - SIDEWALK, y0 + SIDEWALK, y1 - SIDEWALK)
    if (i, j) in MANHOLES:
        hatch(m, Vector((x0 + 1.5, cy + 6.0, 0)), KERB, f"street_{MANHOLES.index((i, j)) + 1}")
    if special:
        special(m, rng, cx, cy, inner)
        return
    m.bank_here = (i, j) in BANKS
    m.precinct_here = (i, j) in PRECINCTS
    for lot in lots(rng, district, *inner):
        build_lot(m, rng, district, lot, inner)
    if m.bank_here:
        print(f"[map] no lot for a bank in block {(i, j)}")
    if m.precinct_here:
        print(f"[map] no lot for a precinct in block {(i, j)}")
    m.bank_here = False
    m.precinct_here = False


def parked(m, rng):
    """Light cycles parked along the kerbs, clear of the junctions."""
    cy_, cw, _ = CANAL
    lines_at = [k * PITCH for k in range(-BLOCKS // 2, BLOCKS // 2 + 1)]
    lines_at[0], lines_at[-1] = -HALF + STREET / 2, HALF - STREET / 2
    count = 0
    for c in lines_at:
        for horizontal in (True, False):
            if horizontal and abs(c - cy_) < 9:
                continue
            for _ in range(10):
                t = rng.uniform(-HALF + 10, HALF - 10)
                if any(abs(t - j) < STREET / 2 + 4 for j in lines_at):
                    continue
                if not horizontal and abs(t - cy_) < cw / 2 + 4:
                    continue
                if not horizontal and abs(c) < 1 and any(s0 - 14 < t < s1 + 14 for s0, s1, _, _ in STATIONS):
                    continue
                side = rng.choice((-1, 1)) * (STREET / 2 - 1.5)
                x, y = (t, c + side) if horizontal else (c + side, t)
                f = Frame(x, y, 0.0 if horizontal else math.pi / 2)
                m.box(f, -2.1, 2.1, -0.9, 0.9, 0.0, 1.4, m.cover, m.crate, cover="low")
                corners = [f.at(-2.1, -0.9), f.at(2.1, -0.9), f.at(2.1, 0.9), f.at(-2.1, 0.9)]
                m.ring(corners, 0.6, m.lines[m.shade()], out=0.03, width=0.06)
                count += 1
    m.n_cover += count


# Places of their own.

def plaza(m, rng, cx, cy, inner):
    """The central plaza: paving ruled with rings of light, a monument of floating rings, benches
    and trees round it."""
    x0, x1, y0, y1 = inner
    for r in (6.0, 12.0, 18.0):
        m.ring(octagon(r, cx, cy), KERB + 0.006, m.lines["yellow"] if r == 12 else m.circuit, out=0.0,
               width=0.12, light=r == 12)
    base = octagon(4.0, cx, cy)
    m.solid(base, 0.0, 1.0, m.struct, m.facades["stone"], cover="high")
    m.ring(base, 0.97, m.lines["yellow"])
    obel, top = octagon(1.2, cx, cy), octagon(0.4, cx, cy)
    m.solid(obel, 1.0, 16.0, m.struct, m.facades["bronze"], top=top)
    m.fins(obel, top, 1.0, 16.0, m.lines["yellow"])
    for k, z in enumerate((6.0, 9.5, 13.0)):
        m.ring(polygon(3.5 - k * 0.6, 24, k * 0.3, cx, cy), z, m.lines[("cyan", "pink", "yellow")[k]], out=0.0,
               width=0.2)
    for a in range(8):
        ang = a * math.pi / 4 + math.pi / 8
        bx, by = cx + 9.0 * math.cos(ang), cy + 9.0 * math.sin(ang)
        f = Frame(bx, by, ang + math.pi / 2)
        m.box(f, -1.2, 1.2, -0.3, 0.3, 0.0, 0.5, m.cover, m.crate, cover="low")
        tx, ty = cx + 15.0 * math.cos(ang), cy + 15.0 * math.sin(ang)
        m.tree(tx, ty, rng)
    for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
        m.pylon(WORLD, cx + sx * 12.0, cy + sy * 12.0, KERB, height=7.0, shade="yellow")
    br_town.marker("place_plaza", cx, cy - 20.0, KERB, math.pi / 2)


def summa(m, rng, cx, cy, inner):
    """Summa, the tallest tower: a podium over the whole block, a terrace on it climbed from the
    street, and the tower stepping back to a crown and a spire."""
    x0, x1, y0, y1 = inner
    pod_h = 6.0
    # The podium's south half is a raised terrace reached by a broad flight; the tower stands on
    # the north half.
    podium = rect(x0, x1, y0 + 10.0, y1)
    m.prism(podium, 0.0, pod_h, m.facades["stone"])
    m.fins(podium, podium, 0.0, pod_h, m.lines["yellow"])
    m.facade_windows(podium, 0.0, pod_h, 3.0, "warm", rng, lit=0.85, bands=True)
    m.ring(podium, pod_h - 0.05, m.lines["yellow"])
    # Broad stairs up the middle of its south face, from the pavement to the terrace on top,
    # and a parapet round the terrace but for them.
    g = Frame(cx, y0, math.pi / 2)   # local +x north, local y west
    n = round(pod_h / RISE)
    m.flight(g, 10.0 - n * RUN, -4.0, 4.0, 0.0, pod_h, +1, "yellow")
    front = y0 + 10.0
    span = (x1 - x0)
    m.parapet(Vector((x0 + 0.15, front + 0.15, 0)), Vector((x1 - 0.15, front + 0.15, 0)), pod_h, m.lines["yellow"],
              [((cx - 4.0 - x0) / span, (cx + 4.0 - x0) / span)])
    for x in (x0 + 0.15, x1 - 0.15):
        m.parapet(Vector((x, front + 0.15, 0)), Vector((x, cy - 2.0, 0)), pod_h, m.lines["yellow"])
    tower_poly = rect(cx - 13, cx + 13, cy - 2, y1 - 3)
    height = 185.0
    z = pod_h
    p = tower_poly
    for t, share in enumerate((0.45, 0.3, 0.25)):
        top = z + (height - pod_h) * share
        m.prism(p, z, top, m.facades["glass"])
        m.facade_windows(p, z, top, 4.0, "office", rng, lit=0.8, bands=True)
        m.fins(p, p, z, top, m.lines["cyan"])
        m.ring(p, top - 0.05, m.lines["cyan"])
        z = top
        p = shrink(p, 4.0)
    c = middle(p)
    m.prism(octagon(3.0, c.x, c.y), z, z + 12.0, m.facades["bronze"], top=octagon(1.5, c.x, c.y))
    m.line(Vector((c.x, c.y, z + 12.0)), Vector((c.x, c.y, z + 40.0)), m.lines["cyan"], width=0.5)
    for k in range(4):
        m.ring(octagon(5.0 + k, c.x, c.y), z + 2.0 + k * 3.0, m.lines[("cyan", "pink", "yellow", "white")[k]]
               if k < 3 else m.white, out=0.0, width=0.2)
    br_town.marker("place_summa", cx, y0 + 2.0, KERB, math.pi / 2)


def park(m, rng, cx, cy, inner, name="place_park_west"):
    """A park: lawns crossed by lit paths, a pond, trees, benches and lamps."""
    x0, x1, y0, y1 = inner
    for a, b in (((x0, cy), (x1, cy)), ((cx, y0), (cx, y1)), ((x0, y0), (x1, y1)), ((x0, y1), (x1, y0))):
        p, q = Vector((a[0], a[1], KERB + 0.004)), Vector((b[0], b[1], KERB + 0.004))
        m.line(p, q, m.pavement, width=2.4, height=0.01, light=False)
        m.line(p, q, m.lines["green"], width=0.06, height=0.014, light=False)
    pond = polygon(7.0, 20, 0, cx, cy)
    m.solid(pond, KERB, KERB + 0.02, m.ground, m.water)
    m.ring(pond, KERB + 0.03, m.lines["cyan"], out=0.0, width=0.1)
    placed = [(cx, cy, 8.5)]
    for _ in range(40):
        x, y = rng.uniform(x0 + 2, x1 - 2), rng.uniform(y0 + 2, y1 - 2)
        # Off the paths and the pond, and apart.
        if abs(x - cx) < 2.5 or abs(y - cy) < 2.5 or abs(abs(x - cx) - abs(y - cy)) < 2.5:
            continue
        if any(math.hypot(x - px, y - py) < pr + 3.0 for px, py, pr in placed):
            continue
        placed.append((x, y, 1.5))
        m.tree(x, y, rng, size=rng.uniform(0.9, 1.5))
    for k in range(4):
        ang = k * math.pi / 2 + math.pi / 4
        f = Frame(cx + 10 * math.cos(ang), cy + 10 * math.sin(ang), ang + math.pi / 2)
        m.box(f, -1.2, 1.2, -0.3, 0.3, 0.0, 0.5, m.cover, m.crate, cover="low")
        m.pylon(WORLD, cx + 12 * math.cos(ang + 0.3), cy + 12 * math.sin(ang + 0.3), KERB, height=4.5, shade="green")
    br_town.marker(name, cx, cy - 9.0, KERB, math.pi / 2)


def park_east(m, rng, cx, cy, inner):
    park(m, rng, cx, cy, inner, "place_park_east")


def market(m, rng, cx, cy, inner):
    """The market: rows of stalls under lit canopies, crates between them."""
    x0, x1, y0, y1 = inner
    for gx in range(4):
        for gy in range(4):
            x = x0 + 5.0 + gx * (x1 - x0 - 10.0) / 3
            y = y0 + 5.0 + gy * (y1 - y0 - 10.0) / 3
            shade = m.shade()
            m.box(WORLD, x - 2.0, x + 2.0, y - 1.0, y + 1.0, 0.0, 1.0, m.cover, m.crate, cover="low")
            for px, py in ((-2.2, -1.6), (2.2, -1.6), (-2.2, 1.6), (2.2, 1.6)):
                m.box(WORLD, x + px - 0.08, x + px + 0.08, y + py - 0.08, y + py + 0.08, 0.0, 3.0, m.struct, m.rail)
            canopy = rect(x - 2.6, x + 2.6, y - 2.0, y + 2.0)
            m.solid(canopy, 3.0, 3.15, m.struct, m.facades[rng.choice(("brick", "teal", "sand", "bronze"))])
            m.ring(canopy, 3.07, m.lines[shade], out=0.03, width=0.06)
            m.line(Vector((x - 1.8, y - 1.05, 1.0)), Vector((x + 1.8, y - 1.05, 1.0)), m.windows["warm"],
                   width=0.1, light=False)
    for sx, sy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
        m.pylon(WORLD, cx + sx * 8.0, cy + sy * 8.0, KERB, height=5.0, shade="orange")
    br_town.marker("place_market", cx, cy, KERB, 0.0)


def civic_hall(m, rng, cx, cy, inner):
    """The civic hall: on a stepped base, a colonnade round a hall, a dome on it lit by ribs."""
    x0, x1, y0, y1 = inner
    hw, hd = 16.0, 13.0
    # The base: four steps all round, so it can be climbed from any side.
    for s in range(4):
        e = 4 - s
        m.box(WORLD, cx - hw - e * RUN, cx + hw + e * RUN, cy - hd - e * RUN, cy + hd + e * RUN,
              0.0, (s + 1) * RISE, m.struct, m.facades["stone"])
    top = 4 * RISE
    m.ring(rect(cx - hw, cx + hw, cy - hd, cy + hd), top - 0.03, m.lines["yellow"], out=0.03, width=0.06)
    hall = rect(cx - hw + 4, cx + hw - 4, cy - hd + 4, cy + hd - 4)
    m.prism(hall, top, top + 12.0, m.facades["white"])
    m.fins(hall, hall, top, top + 12.0, m.lines["yellow"])
    m.facade_windows(hall, top, top + 12.0, 6.0, "warm", rng, lit=0.8)
    # Columns round it.
    for k in range(9):
        x = cx - hw + 1.0 + k * (2 * hw - 2.0) / 8
        for y in (cy - hd + 1.0, cy + hd - 1.0):
            m.prism(polygon(0.5, 10, 0, x, y), top, top + 12.0, m.facades["white"])
    for k in range(1, 7):
        y = cy - hd + 1.0 + k * (2 * hd - 2.0) / 7
        for x in (cx - hw + 1.0, cx + hw - 1.0):
            m.prism(polygon(0.5, 10, 0, x, y), top, top + 12.0, m.facades["white"])
    roof = rect(cx - hw, cx + hw, cy - hd, cy + hd)
    m.prism(roof, top + 12.0, top + 13.2, m.facades["white"])
    m.ring(roof, top + 12.6, m.lines["yellow"])
    # The dome, ribbed with light.
    z0 = top + 13.2
    r = 10.0
    bands = 6
    rings = [(polygon(max(r * math.cos(b * (math.pi / 2) / bands), 0.4), 16, 0.0, cx, cy),
              z0 + r * math.sin(b * (math.pi / 2) / bands)) for b in range(bands + 1)]
    for b in range(bands):
        (lo, za), (hi, zb) = rings[b], rings[b + 1]
        m.prism(lo, za, zb, m.facades["bronze"], top=hi)
        for i in range(0, 16, 2):
            c = Vector((cx, cy, 0))
            p = lo[i] + (lo[i] - c).normalized() * 0.06
            q = hi[i] + (hi[i] - c).normalized() * 0.06
            m.line(Vector((p.x, p.y, za)), Vector((q.x, q.y, zb)), m.lines["yellow"])
    for sx in (-1, 1):
        m.pylon(WORLD, cx + sx * 6.0, y0 + 1.0, KERB, height=6.0, shade="yellow")
    br_town.marker("place_hall", cx, cy - hd - 3.0, KERB, math.pi / 2)


def arena(m, rng, cx, cy, inner):
    """The arena: an oval of stands round a field, four ways in, rings of light round its rim."""
    ro, ri = (21.0, 18.0), (13.0, 10.0)
    sides = 40
    gates = {0, 10, 20, 30}
    shade = "violet"
    for k in range(sides):
        if k in gates:
            continue
        a0, a1 = 2 * math.pi * k / sides, 2 * math.pi * (k + 1) / sides
        pt = lambda r, a: Vector((cx + r[0] * math.cos(a), cy + r[1] * math.sin(a), 0))
        mid = ((ro[0] + ri[0]) / 2, (ro[1] + ri[1]) / 2)
        m.solid([pt(ri, a0), pt(ri, a1), pt(mid, a1), pt(mid, a0)], 0.0, 6.0, m.struct, m.facades["concrete"])
        m.solid([pt(mid, a0), pt(mid, a1), pt(ro, a1), pt(ro, a0)], 0.0, 13.0, m.struct, m.facades["slate"])
        for r, z in ((ro, 13.0), (ro, 6.5), (mid, 6.0)):
            m.line(pt(r, a0) + Vector((0, 0, z)), pt(r, a1) + Vector((0, 0, z)), m.lines[shade])
        if k % 2:
            p = pt(ro, (a0 + a1) / 2)
            m.line(p + Vector((0, 0, 1.0)), p + Vector((0, 0, 12.0)), m.windows["cool"], width=0.6, light=False)
    field = polygon(1.0, 40, 0, cx, cy)
    field = [Vector((cx + (p.x - cx) * ri[0] * 0.95, cy + (p.y - cy) * ri[1] * 0.95, 0)) for p in field]
    m.solid(field, KERB, KERB + 0.01, m.ground, m.grass)
    m.ring(field, KERB + 0.02, m.white, out=0.0, width=0.1,
           light=False)
    for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
        m.pylon(WORLD, cx + sx * 7.0, cy + sy * 5.0, KERB, height=8.0, shade=shade)
    br_town.marker("place_arena", cx, cy, KERB, 0.0)


def docks_place(m, rng, cx, cy, inner):
    """The docks' yard on the canal: stacks of containers round a crane."""
    x0, x1, y0, y1 = inner
    colours = ("rust", "teal", "brick", "sand", "slate")
    for gx in range(3):
        for gy in range(3):
            if gx == 1 and gy == 1:
                continue
            x = x0 + 7 + gx * (x1 - x0 - 14) / 2
            y = y0 + 7 + gy * (y1 - y0 - 14) / 2
            for level in range(rng.randint(1, 3)):
                z = level * 2.6
                m.box(WORLD, x - 6.0, x + 6.0, y - 1.25, y + 1.25, z, z + 2.6, m.cover,
                      m.facades[rng.choice(colours)], cover="high")
                m.ring(rect(x - 6.0, x + 6.0, y - 1.25, y + 1.25), z + 2.55, m.lines["orange"], out=0.03, width=0.05,
                       light=level == 0)
    # The crane: a tower, its jib out over the canal.
    m.box(WORLD, cx - 1.2, cx + 1.2, cy - 1.2, cy + 1.2, 0.0, 30.0, m.struct, m.facades["rust"])
    m.box(WORLD, cx - 0.8, cx + 0.8, cy - 10.0, cy + 40.0, 30.0, 31.6, m.struct, m.facades["rust"])
    m.line(Vector((cx, cy - 10.0, 31.7)), Vector((cx, cy + 40.0, 31.7)), m.lines["orange"])
    for y in (cy - 10.0, cy + 40.0):
        m.line(Vector((cx, y, 31.6)), Vector((cx, y, 34.0)), m.lines["red"], width=0.4)
    br_town.marker("place_docks", cx, cy - 15.0, KERB, math.pi / 2)


SPECIAL = {}


def railway(m):
    """The elevated railway down the middle avenue: a beam on pillars, off both ends into the
    void, and stations over the avenue along it with platforms either side, climbed by stairs."""
    top, w, d = RAIL
    far = HALF + 25.0
    plat = STATION[2]
    m.box(WORLD, -w / 2, w / 2, -far, far, top - d, top, m.struct, m.facades["slate"])
    for x in (-w / 2 - 0.02, w / 2 + 0.02):
        m.line(Vector((x, -far, top - 0.2)), Vector((x, far, top - 0.2)), m.lines["cyan"], across=Vector((0, 0, 1)))
    y = -HALF + 32.0
    while y < HALF:
        if not any(s0 - 2 < y < s1 + 2 for s0, s1, _, _ in STATIONS) and abs(y - CANAL[0]) > CANAL[1] / 2 + 1:
            m.box(WORLD, -0.7, 0.7, y - 0.7, y + 0.7, 0.0, top - d, m.struct, m.facades["concrete"])
            m.line(Vector((0.72, y, 0.0)), Vector((0.72, y, top - d)), m.lines["cyan"], width=0.08,
                   across=Vector((0, 1, 0)))
        y += PITCH
    for s0, s1, way, marker in STATIONS:
        station(m, s0, s1, plat, way, marker)


def station(m, s0, s1, plat, way, marker):
    """A station from `s0` to `s1` along the avenue: a platform either side of the beam, a canopy
    over each, and stairs down from the end `way` says - 1 the north, -1 the south - to the street
    past it; a parapet round the rest of each platform. Its `marker` at the foot of the stairs."""
    _, w, _ = RAIL
    shut, stairs = (s0, s1) if way > 0 else (s1, s0)
    n = round(plat / RISE)
    for sx in (-1, 1):
        a, b = sorted((sx * (w / 2 + 0.3), sx * (STREET / 2 - 1.0)))
        m.box(WORLD, a, b, s0, s1, plat - DECK, plat, m.struct, m.deck)
        m.line(Vector((sx * (w / 2 + 0.3), s0, plat + 0.006)), Vector((sx * (w / 2 + 0.3), s1, plat + 0.006)),
               m.lines["yellow"], width=0.25, height=0.012)
        outer_x = sx * (STREET / 2 - 1.0)
        m.parapet(Vector((outer_x - sx * 0.15, s0, 0)), Vector((outer_x - sx * 0.15, s1, 0)), plat, m.lines["cyan"])
        # Across the end away from the stairs, whole; across the end with them, but for the top
        # of the stairs - both clear of where the train's cars go through.
        ea, eb = sorted((sx * TRAIN_HALF, outer_x))
        m.parapet(Vector((ea, shut + way * 0.15, 0)), Vector((eb, shut + way * 0.15, 0)), plat, m.lines["cyan"])
        lo_x, hi_x = sorted((sx * (STREET / 2 - 1.2), sx * (STREET / 2 - 2.8)))
        m.parapet(Vector((ea, stairs - way * 0.15, 0)), Vector((eb, stairs - way * 0.15, 0)), plat, m.lines["cyan"],
                  [((lo_x - ea) / (eb - ea), (hi_x - ea) / (eb - ea))])
        for y in (s0 + 2.0, (s0 + s1) / 2, s1 - 6.0) if way > 0 else (s1 - 2.0, (s0 + s1) / 2, s0 + 6.0):
            m.box(WORLD, outer_x - sx * 0.6 - 0.15, outer_x - sx * 0.6 + 0.15, y - 0.15, y + 0.15, 0.0, plat - DECK,
                  m.struct, m.facades["concrete"])
            m.box(WORLD, outer_x - sx * 0.6 - 0.12, outer_x - sx * 0.6 + 0.12, y - 0.12, y + 0.12, plat, plat + 3.4,
                  m.struct, m.rail)
        canopy = rect(a, b, s0, s1)
        m.solid(canopy, plat + 3.4, plat + 3.6, m.struct, m.facades["teal"])
        m.ring(canopy, plat + 3.5, m.lines["cyan"], out=0.03, width=0.06)
        for y in (s0 + 8.0, s1 - 12.0) if way > 0 else (s1 - 8.0, s0 + 12.0):
            m.glass_pane(WORLD, min(a, b) + 1.0, max(a, b) - 1.0, y - 0.3, y + 0.3, plat + 3.36, plat + 3.4)
        # The stairs, from the platform's end down to the street, on away from it.
        if way > 0:
            g = Frame(0.0, 0.0, math.pi / 2)    # local +x north, local y is -x
            m.flight(g, s1 + n * RUN, -hi_x, -lo_x, 0.0, plat, -1, "yellow")
        else:
            g = Frame(0.0, 0.0, -math.pi / 2)   # local +x south, local y is +x
            m.flight(g, -s0 + n * RUN, lo_x, hi_x, 0.0, plat, -1, "yellow")
    # Pillars under the platforms.
    for sx in (-1, 1):
        for y in (s0 + 1.0, s1 - 1.0):
            x = sx * (STREET / 2 - 3.0)
            m.box(WORLD, x - 0.4, x + 0.4, y - 0.4, y + 0.4, 0.0, plat - DECK, m.struct, m.facades["concrete"])
    foot = stairs + way * (n * RUN + 2.0)
    br_town.marker(marker, 0.0, foot, 0.0, -math.pi / 2 if way > 0 else math.pi / 2)


def gates(m):
    """Out of the east and west edges, a bridge over the void to a pad with an obelisk on it."""
    length, width, pad = GATE
    for sx, name, shade in ((1, "place_gate_east", "orange"), (-1, "place_gate_west", "violet")):
        f = Frame(0.0, 0.0, -math.pi / 2 if sx > 0 else math.pi / 2)   # +y outward
        far = HALF + length
        m.box(f, -width / 2, width / 2, HALF - 0.5, far, -0.5, -0.002, m.ground, m.road)
        for x in (-width / 2 + 0.07, width / 2 - 0.07):
            m.line(f.at(x, HALF, 0.006), f.at(x, far, 0.006), m.lines[shade], height=0.012, across=f.way(1, 0))
        pads = [f.at(-pad / 2, far), f.at(pad / 2, far), f.at(pad / 2, far + pad), f.at(-pad / 2, far + pad)]
        m.solid(pads, -1.5, -0.002, m.ground, m.road)
        m.ring(pads, 0.006, m.lines[shade], out=-0.1)
        g = f.at(0, far + pad / 2 + 3.0)
        base, top = octagon(2.5, g.x, g.y), octagon(0.8, g.x, g.y)
        m.solid(base, 0.0, 22.0, m.struct, m.dark[0], top=top)
        m.fins(base, top, 0.0, 22.0, m.lines[shade])
        m.ring(base, 1.0, m.lines[shade])
        for x in (-pad / 2 + 1.2, pad / 2 - 1.2):
            m.pylon(f, x, far + 1.2, shade=shade)
        p = f.at(0, far + 3.0)
        br_town.marker(name, p.x, p.y, 0.0, math.atan2(-p.y, -p.x))


def park_north(m, rng, cx, cy, inner):
    park(m, rng, cx, cy, inner, "place_park_north")


def park_far_west(m, rng, cx, cy, inner):
    park(m, rng, cx, cy, inner, "place_park_far_west")


def park_far_east(m, rng, cx, cy, inner):
    park(m, rng, cx, cy, inner, "place_park_far_east")


def park_far_north(m, rng, cx, cy, inner):
    park(m, rng, cx, cy, inner, "place_park_far_north")


SPECIAL.update({
    (7, 6): plaza,
    (6, 7): summa,
    (5, 8): park,
    (8, 7): park_east,
    (5, 5): market,
    (8, 5): civic_hall,
    (8, 9): arena,
    (6, 1): docks_place,
    (9, 10): park_north,
    (2, 8): park_far_west,
    (12, 6): park_far_east,
    (4, 12): park_far_north,
})


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    m = Urbs(random.Random(seed + 1))
    m.trade = random.Random(seed + 2)
    m.law = random.Random(seed + 3)
    ground(m)
    streets(m, rng)
    for i in range(BLOCKS):
        for j in range(BLOCKS):
            block(m, rng, i, j)
    railway(m)
    gates(m)
    escape_tunnels(m)
    parked(m, rng)
    m.build_buckets()

    bpy.ops.object.camera_add(location=(0, -HALF * 2.0, HALF * 1.2), rotation=(math.radians(58), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.0, 0.0, 0.004, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "story_urbs.blend")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {HALF * 2:.0f} m across, buildings={m.n_buildings} lifts={m.n_lifts} lights={m.n_lights} "
          f"cover={m.n_cover} stores={m.n_stores} banks={m.n_banks} precincts={m.n_precincts} pieces of light={len(m.pieces)} -> {out}")
    if glb:
        glow_path = os.path.splitext(os.path.abspath(glb))[0] + ".glow.json"
        with open(glow_path, "w") as f:
            json.dump({"intensity": 3.0, "pieces": m.pieces}, f, separators=(",", ":"))
        print(f"[map] {len(m.pieces)} pieces of light -> {glow_path}")
        roads_path = os.path.splitext(os.path.abspath(glb))[0] + ".roads.json"
        with open(roads_path, "w") as f:
            json.dump(road_plan(), f, separators=(",", ":"))
        print(f"[map] roads and crossings -> {roads_path}")
        bpy.ops.export_scene.gltf(filepath=os.path.abspath(glb), export_format="GLB", export_extras=True,
                                  export_cameras=False, export_lights=False)
        print(f"[map] exported -> {glb}")


if __name__ == "__main__":
    main()
