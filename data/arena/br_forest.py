"""
Battle royale in the Foggy Forest: an abandoned military base the forest has taken back, under a
grey sky in fog (the game lays the fog over it: see `ctf::FOREST`).

From the middle out:

- The compound, inside a broken concrete wall with gaps blown in it and its gate standing open:
  its apron of cracked concrete slabs, grass and trees coming up between them.
- The command post in the middle: three floors, stairs up inside to the roof - the highest
  ground there is.
- Two hangars, their roofs fallen in: strips of roof gone, and trees growing up through the
  holes; crates and a wrecked vehicle inside.
- Three barracks, a floor or two each, doors and windows, along the north side.
- A helipad, its H worn, and wrecked vehicles, rusting containers - some stacked - and sandbag
  rings scattered about.
- A watchtower at each corner of the compound: a platform on four legs a floor up, reached by a
  flight of stairs, a parapet round it.
- And all round, the forest: pines and broad-leaved trees, bushes, and fallen trunks to crouch
  behind, out to a crumbling outer wall at the map's edge - and a little more forest past it,
  out of reach, that the fog swallows.

Trunks, logs, buildings and cover are solid. The leaves are not: the canopies and bushes go in
one mesh named `Foliage`, which the game leaves out of the level's collider - they hide whoever is
in them, but bolts and droids go through. Everything that climbs climbs by stairs of 0.25 m steps.

Builds data/arena/br_forest.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/arena/br_forest.py -- --seed 5 --out ~/Documents/blender/map/br_forest.blend --glb data/arena/br_forest.glb

Made for Ruptura Systematis (MazeGame/maze). The game puts the players down at the empties
`spawn_1` to `spawn_16`, in the forest round the compound.
"""

import bmesh
import bpy
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import br_town  # noqa: E402
from br_town import add_box, add_block, marker, collection, material, building, flight, \
    FLOOR, SLAB, RAIL, PARAPET, STEP_RISE, STEP_RUN  # noqa: E402

HALF = 100.0          # from the middle to the outer wall
BEYOND = 12.0         # forest past the outer wall, out of reach
OUTER_WALL = 3.5
COMPOUND = 55.0       # from the middle to the compound's wall
COMPOUND_WALL = 2.6
WALL_T = 0.5
GATE = 10.0           # the compound's gate, in the middle of its south side
BREACHES = 7          # gaps blown in the compound's wall
TOWER = (4.5, FLOOR)  # a watchtower's platform side and height
TREE_GAP = 3.2        # least distance between two trunks
KEEP = 1.8            # walkable space kept round anything solid
SPAWNS = 16
# The model is cut into tiles this many meters square, each a mesh of its own for each of the
# ground, the buildings, the cover and the leaves: so that the renderer can leave out what is
# behind the camera or past the fog, and draw what is near before what is far.
TILE = 56.0


class Base:
    """What is built so far, in the collections br_town's helpers build into, with the base's
    materials in place of the town's."""

    def __init__(self):
        self.ground = collection("Ground")
        self.struct = collection("Buildings")
        self.cover = collection("Cover")
        self.lights = collection("Lights")
        self.foliage = collection("Foliage")
        self.grass = material("ForestFloor", (0.15, 0.20, 0.10))
        self.apron = material("Apron", (0.40, 0.40, 0.38))
        self.moss = material("MossyConcrete", (0.30, 0.34, 0.25))
        self.paint = material("WornPaint", (0.62, 0.60, 0.52))
        self.walls = [material(f"BaseWall{i}", rgb) for i, rgb in enumerate(
            [(0.42, 0.42, 0.39), (0.36, 0.38, 0.33), (0.45, 0.43, 0.38)])]
        self.floor_mat = material("BaseFloor", (0.33, 0.32, 0.30))
        self.roof = material("BaseRoof", (0.26, 0.27, 0.26))
        self.stairs = material("BaseStairs", (0.40, 0.40, 0.37))
        self.rail = material("Rust", (0.36, 0.20, 0.11))
        self.low = material("Sandbags", (0.42, 0.38, 0.27))
        self.high = material("Container", (0.30, 0.33, 0.24))
        self.rust = self.rail
        self.bark = material("Bark", (0.22, 0.16, 0.11))
        self.leaves = [material(f"Leaves{i}", rgb) for i, rgb in enumerate(
            [(0.07, 0.17, 0.07), (0.11, 0.22, 0.08), (0.15, 0.24, 0.09)])]
        self.n_lights = self.n_buildings = self.n_cover = 0
        self.keep_clear = []   # (x0, x1, y0, y1): buildings, stairs, the spawns
        self.solid = []        # (x, y, r): trunks and cover, kept apart
        self.trunks = Tiled("Trunks", [self.bark])
        self.canopy = Tiled("Canopy", self.leaves)
        self.n_trees = self.n_logs = self.n_bushes = 0

    def light(self, x, y, ceiling):
        """The base has long had no power: no lights."""

    def clear(self, x, y, r, gap=KEEP):
        for x0, x1, y0, y1 in self.keep_clear:
            dx, dy = max(x0 - x, 0, x - x1), max(y0 - y, 0, y - y1)
            if math.hypot(dx, dy) < r + gap:
                return False
        return all(math.hypot(x - sx, y - sy) >= r + sr + gap for sx, sy, sr in self.solid)


class Mesh:
    """One mesh built up from many shapes, faces given a material each: hundreds of trees as
    one object each for trunks and leaves, rather than an object - and a draw - a tree."""

    def __init__(self, name, mat, mats=None):
        self.name, self.mats = name, mats or [mat]
        self.verts, self.faces, self.face_mats = [], [], []

    def add(self, verts, faces, mat=0):
        base = len(self.verts)
        self.verts.extend(verts)
        self.faces.extend(tuple(base + i for i in f) for f in faces)
        self.face_mats.extend([mat] * len(faces))

    def build(self, col):
        mesh = bpy.data.meshes.new(self.name)
        mesh.from_pydata(self.verts, [], self.faces)
        for m in self.mats:
            mesh.materials.append(m)
        for poly, m in zip(mesh.polygons, self.face_mats):
            poly.material_index = m
        # Every face turned outward: the game draws only the side a face is turned to, and
        # these shapes are built up by hand.
        bm = bmesh.new()
        bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        bm.to_mesh(mesh)
        bm.free()
        mesh.update()
        o = bpy.data.objects.new(self.name, mesh)
        col.objects.link(o)
        return o


def tile_of(x, y):
    """Which of the map's tiles, TILE meters square, (x, y) is in."""
    edge = HALF + BEYOND
    last = int(2 * edge // TILE)
    return (min(last, max(0, int((x + edge) // TILE))), min(last, max(0, int((y + edge) // TILE))))


class Tiled:
    """A Mesh for each tile: shapes go in the one for the tile they stand in."""

    def __init__(self, name, mats):
        self.name, self.mats, self.tiles = name, mats, {}
        self.at = (0.0, 0.0)

    def __getattr__(self, attr):
        raise AttributeError(attr)

    @property
    def faces(self):
        return [f for m in self.tiles.values() for f in m.faces]

    def add(self, verts, faces, mat=0):
        tile = tile_of(*self.at)
        if tile not in self.tiles:
            self.tiles[tile] = Mesh(f"{self.name}_{tile[0]}_{tile[1]}", None, self.mats)
        self.tiles[tile].add(verts, faces, mat)

    def build(self, col):
        for m in self.tiles.values():
            m.build(col)


def ring(x, y, z, r, n, turn=0.0):
    return [(x + r * math.cos(turn + 2 * math.pi * i / n), y + r * math.sin(turn + 2 * math.pi * i / n), z)
            for i in range(n)]


def prism(m, x, y, z0, z1, r0, r1, n, mat=0, turn=0.0):
    """A tapering n-sided prism, capped at the top: a trunk."""
    verts = ring(x, y, z0, r0, n, turn) + ring(x, y, z1, r1, n, turn) + [(x, y, z1)]
    faces = [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    faces += [(n + i, n + (i + 1) % n, 2 * n) for i in range(n)]
    m.add(verts, faces, mat)


def cone(m, x, y, z0, z1, r, n, mat, turn=0.0):
    """A cone, its base closed: a tier of a pine."""
    verts = ring(x, y, z0, r, n, turn) + [(x, y, z1), (x, y, z0)]
    faces = [(i, (i + 1) % n, n) for i in range(n)] + [((i + 1) % n, i, n + 1) for i in range(n)]
    m.add(verts, faces, mat)


def blob(m, rng, x, y, z, rx, rz, mat):
    """A lumpy ball of leaves: an octahedron's faces split once, pushed out to a sphere, and each
    point moved in or out a little."""
    pts = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]
    tris = [(0, 2, 4), (2, 1, 4), (1, 3, 4), (3, 0, 4), (2, 0, 5), (1, 2, 5), (3, 1, 5), (0, 3, 5)]
    mids = {}

    def mid(a, b):
        key = (min(a, b), max(a, b))
        if key not in mids:
            p = [(pts[a][k] + pts[b][k]) / 2 for k in range(3)]
            length = math.sqrt(sum(c * c for c in p))
            pts.append(tuple(c / length for c in p))
            mids[key] = len(pts) - 1
        return mids[key]

    faces = []
    for a, b, c in tris:
        ab, bc, ca = mid(a, b), mid(b, c), mid(c, a)
        faces += [(a, ab, ca), (ab, b, bc), (ca, bc, c), (ab, bc, ca)]
    verts = []
    for px, py, pz in pts:
        k = rng.uniform(0.85, 1.15)
        verts.append((x + px * rx * k, y + py * rx * k, z + pz * rz * k))
    m.add(verts, faces, mat)


def tree(base, rng, x, y, solid=True):
    """A pine or a broad-leaved tree at (x, y): its trunk solid, its leaves not."""
    turn = rng.uniform(0, math.pi)
    base.trunks.at = base.canopy.at = (x, y)
    if rng.random() < 0.65:
        height = rng.uniform(11.0, 19.0)
        r = rng.uniform(0.22, 0.38)
        prism(base.trunks, x, y, -0.1, height * 0.75, r, r * 0.35, 7, turn=turn)
        tiers = rng.randint(3, 4)
        bottom = rng.uniform(2.6, 4.0)
        for t in range(tiers):
            f = t / tiers
            z0 = bottom + (height - bottom) * f * 0.85
            spread = (1.0 - f * 0.7) * rng.uniform(2.4, 3.4)
            cone(base.canopy, x, y, z0, z0 + spread * 1.6, spread, 8, 0, turn + t * 0.4)
    else:
        height = rng.uniform(8.0, 13.0)
        r = rng.uniform(0.28, 0.5)
        prism(base.trunks, x, y, -0.1, height * 0.7, r, r * 0.45, 7, turn=turn)
        for _ in range(rng.randint(2, 4)):
            ox, oy = rng.uniform(-1.6, 1.6), rng.uniform(-1.6, 1.6)
            size = rng.uniform(2.2, 3.4)
            blob(base.canopy, rng, x + ox, y + oy, height * rng.uniform(0.62, 0.85), size, size * 0.75,
                 rng.choice((1, 2)))
    if solid:
        base.solid.append((x, y, r))
    base.n_trees += 1


def bush(base, rng, x, y):
    size = rng.uniform(0.7, 1.4)
    base.canopy.at = (x, y)
    blob(base.canopy, rng, x, y, size * 0.55, size, size * 0.7, rng.choice((1, 2)))
    base.n_bushes += 1


def log(base, rng, x, y):
    """A fallen trunk lying on the ground: low cover, a little under waist height."""
    length, r = rng.uniform(5.0, 9.0), rng.uniform(0.32, 0.45)
    base.trunks.at = (x, y)
    a = rng.uniform(0, math.pi)
    c, s = math.cos(a), math.sin(a)
    n = 7
    verts = []
    for end in (-0.5, 0.5):
        cx, cy = x + c * length * end, y + s * length * end
        for i in range(n):
            t = 2 * math.pi * i / n
            # round the trunk's length: across it (perpendicular, flat) and up
            across, up = r * math.cos(t), r + r * math.sin(t)
            verts.append((cx - s * across, cy + c * across, up))
    faces = [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    faces += [tuple(reversed(range(n))), tuple(range(n, 2 * n))]
    base.trunks.add(verts, faces)
    base.solid.append((x, y, length / 2))
    base.n_logs += 1


def broken_wall(base, name, along_x, at, a0, a1, height, gaps, mat):
    """A wall WALL_T thick along x (or y), from a0 to a1, `height` high, broken through at each of
    `gaps` - (middle, width) - its height ragged, piece by piece."""
    edges = [a0]
    for middle, width in sorted(gaps):
        edges += [middle - width / 2, middle + width / 2]
    edges.append(a1)
    rng = random.Random(hash(name) & 0xFFFF)
    for k in range(0, len(edges), 2):
        b0, b1 = edges[k], edges[k + 1]
        n = max(1, int((b1 - b0) // 4.0))
        for i in range(n):
            c0, c1 = b0 + (b1 - b0) * i / n, b0 + (b1 - b0) * (i + 1) / n
            top = height * rng.uniform(0.7, 1.0)
            if along_x:
                add_box(f"{name}_{k}_{i}", c0, c1, at - WALL_T / 2, at + WALL_T / 2, 0.0, top, base.struct, mat)
            else:
                add_box(f"{name}_{k}_{i}", at - WALL_T / 2, at + WALL_T / 2, c0, c1, 0.0, top, base.struct, mat)


def hangar(base, rng, cx, cy, w, d, name):
    """A hangar - br_town's warehouse - its roof fallen in: strips of it gone, and trees come up
    through the holes."""
    building(base, rng, "warehouse", cx, cy, w, d, 0, name)
    top = br_town.WAREHOUSE_HEIGHT
    roof = [o for o in base.struct.objects if o.name.startswith(f"{name}_Slab1")]
    for o in roof:
        bpy.data.objects.remove(o)
    x0, x1, y0, y1 = cx - w / 2, cx + w / 2, cy - d / 2, cy + d / 2
    strips = 6
    gone = set(rng.sample(range(strips), 2))
    for i in range(strips):
        a, b = x0 + w * i / strips, x0 + w * (i + 1) / strips
        if i in gone:
            # a tree grows up through the hole, from the hangar's floor
            tree(base, rng, (a + b) / 2 + rng.uniform(-0.5, 0.5), cy + rng.uniform(-d / 4, d / 4))
            continue
        add_box(f"{name}_Roof{i}", a, b, y0, y1, top, top + SLAB, base.struct, base.roof)
    # a wreck parked inside
    add_block(f"{name}_Wreck", cx + rng.uniform(-w / 5, w / 5), cy - d / 5, 4.4, 2.2, 1.7, rng.uniform(-0.3, 0.3),
              base.cover, base.rust, "low")


def watchtower(base, x, y, toward_x, name):
    """A platform TOWER[0] square a floor up on four legs, a parapet round it but where a flight
    comes up to it from the side `toward_x` (+1 or -1) - the compound's inside."""
    side, height = TOWER
    h = side / 2
    for sx in (-1, 1):
        for sy in (-1, 1):
            add_box(f"{name}_Leg{sx}{sy}", x + sx * h - 0.2 * sx - 0.15, x + sx * h - 0.2 * sx + 0.15,
                    y + sy * h - 0.2 * sy - 0.15, y + sy * h - 0.2 * sy + 0.15, 0.0, height - SLAB, base.struct, base.rust)
    add_box(f"{name}_Deck", x - h, x + h, y - h, y + h, height - SLAB, height, base.struct, base.floor_mat)
    # The flight, along the platform's south side, rising toward it from inside the compound.
    run = round(FLOOR / STEP_RISE) * STEP_RUN
    start = x + toward_x * (h + run)
    flight(base, f"{name}_Flight", start, y - h, y - h + 1.4, 0.0, -toward_x)
    base.keep_clear.append((min(start, x) - h - 1.0, max(start, x) + h + 1.0, y - h - 1.0, y + h + 1.0))
    # The parapet, open where the flight lands.
    t = 0.12
    rz = height
    add_box(f"{name}_P0", x - h, x + h, y + h - t, y + h, rz, rz + PARAPET, base.struct, base.rust)
    add_box(f"{name}_P1", x - h, x + h, y - h, y - h + t, rz, rz + PARAPET, base.struct, base.rust)
    far = x - toward_x * h
    add_box(f"{name}_P2", far - t / 2, far + t / 2, y - h, y + h, rz, rz + PARAPET, base.struct, base.rust)
    near = x + toward_x * h
    add_box(f"{name}_P3", near - t / 2, near + t / 2, y - h + 1.5, y + h, rz, rz + PARAPET, base.struct, base.rust)
    # A roof over it, on the legs.
    add_box(f"{name}_Roof", x - h - 0.3, x + h + 0.3, y - h - 0.3, y + h + 0.3, rz + 2.6, rz + 2.8, base.struct, base.roof)
    for sx in (-1, 1):
        for sy in (-1, 1):
            add_box(f"{name}_Post{sx}{sy}", x + sx * (h - 0.1) - 0.08, x + sx * (h - 0.1) + 0.08,
                    y + sy * (h - 0.1) - 0.08, y + sy * (h - 0.1) + 0.08, rz, rz + 2.6, base.struct, base.rust)


def helipad(base, cx, cy):
    r = 9.0
    n = 8
    pts = ring(cx, cy, 0.0, r, n, math.pi / n)
    for i in range(n):
        (ax, ay, _), (bx, by, _) = pts[i], pts[(i + 1) % n]
        # a wedge of the pad, a hair over the apron
        mesh = bpy.data.meshes.new(f"Helipad_{i}")
        mesh.from_pydata([(cx, cy, 0.04), (ax, ay, 0.04), (bx, by, 0.04)], [], [(0, 1, 2)])
        mesh.materials.append(base.moss)
        o = bpy.data.objects.new(f"Helipad_{i}", mesh)
        base.ground.objects.link(o)
    # the H, worn
    for x0, x1, y0, y1 in ((-3.0, -2.0, -4.0, 4.0), (2.0, 3.0, -4.0, 4.0), (-2.0, 2.0, -0.5, 0.5)):
        add_box("Helipad_H", cx + x0, cx + x1, cy + y0, cy + y1, 0.04, 0.06, base.ground, base.paint)
    base.keep_clear.append((cx - r, cx + r, cy - r, cy + r))


def scatter_cover(base, rng):
    """Containers - some stacked two high - wrecked vehicles and sandbag rings round the apron."""
    placed = 0
    for i in range(70):
        kind = rng.choice(("container", "container", "wreck", "sandbags", "crate"))
        w, d, h, mat, cover = {
            "container": (6.0, 2.4, 2.6, base.high, "high"),
            "wreck": (4.4, 2.0, 1.6, base.rust, "low"),
            "sandbags": (3.0, 0.8, RAIL, base.low, "low"),
            "crate": (1.4, 1.4, 1.2, base.low, "low"),
        }[kind]
        r = math.hypot(w, d) / 2
        for _ in range(60):
            x, y = rng.uniform(-COMPOUND + 4, COMPOUND - 4), rng.uniform(-COMPOUND + 4, COMPOUND - 4)
            if not base.clear(x, y, r):
                continue
            rot = rng.choice((0.0, math.pi / 2, rng.uniform(0, math.pi)))
            add_block(f"Cover_{kind}_{i}", x, y, w, d, h, rot, base.cover, mat, cover)
            if kind == "container" and rng.random() < 0.3:
                add_block(f"Cover_{kind}_{i}_Up", x, y, w, d, h, rot + rng.uniform(-0.15, 0.15), base.cover, mat,
                          cover, z=h)
            base.solid.append((x, y, r))
            placed += 1
            break
    base.n_cover = placed


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    base = Base()

    # The ground: the forest floor out past the outer wall, and the compound's apron of cracked
    # slabs over it, grass between them.
    edge = HALF + BEYOND
    add_box("ForestFloor", -edge, edge, -edge, edge, -1.0, 0.0, base.ground, base.grass)
    slab = 6.0
    n = int(2 * COMPOUND // slab)
    for i in range(n):
        for j in range(n):
            if rng.random() < 0.12:
                continue                                      # broken up, grass come through
            x0, y0 = -COMPOUND + i * slab + 0.15, -COMPOUND + j * slab + 0.15
            add_box(f"Apron_{i}_{j}", x0, x0 + slab - 0.3, y0, y0 + slab - 0.3, 0.0, 0.02, base.ground,
                    base.apron if rng.random() < 0.7 else base.moss)

    # The outer wall at the map's edge, crumbling.
    for k, (along_x, at) in enumerate(((True, HALF), (True, -HALF), (False, HALF), (False, -HALF))):
        broken_wall(base, f"OuterWall{k}", along_x, at, -HALF - WALL_T, HALF + WALL_T, OUTER_WALL, [], base.moss)

    # The compound's wall: the gate open in its south side, and breaches blown in it.
    sides = [(True, -COMPOUND), (True, COMPOUND), (False, -COMPOUND), (False, COMPOUND)]
    gaps = {k: [] for k in range(4)}
    gaps[0].append((0.0, GATE))
    for _ in range(BREACHES):
        k = rng.randrange(4)
        middle = rng.uniform(-COMPOUND + 12, COMPOUND - 12)
        if all(abs(middle - m) > 10 for m, _ in gaps[k]):
            gaps[k].append((middle, rng.uniform(3.0, 6.0)))
    for k, (along_x, at) in enumerate(sides):
        broken_wall(base, f"CompoundWall{k}", along_x, at, -COMPOUND - WALL_T / 2, COMPOUND + WALL_T / 2,
                    COMPOUND_WALL, gaps[k], base.walls[k % len(base.walls)])
    for k, (along_x, at) in enumerate(sides):
        for middle, width in gaps[k]:
            # trees and cover kept out of the gaps, to go through
            if along_x:
                base.keep_clear.append((middle - width / 2, middle + width / 2, at - 2.0, at + 2.0))
            else:
                base.keep_clear.append((at - 2.0, at + 2.0, middle - width / 2, middle + width / 2))
        if along_x:
            base.keep_clear.append((-COMPOUND, COMPOUND, at - 0.6, at + 0.6))
        else:
            base.keep_clear.append((at - 0.6, at + 0.6, -COMPOUND, COMPOUND))
    base.keep_clear.append((-GATE / 2, GATE / 2, -COMPOUND - 6, -COMPOUND + 6))

    # The buildings.
    command_floors = 0
    while command_floors < 3:
        # br_town's offices are three floors; the post stands in the middle, its front south.
        for o in [o for o in base.struct.objects if o.name.startswith("Command_")]:
            bpy.data.objects.remove(o)
        command_floors = building(base, rng, "office", 0.0, 10.0, 16.0, 13.0, 0, "Command")
    hangar(base, rng, -32.0, -22.0, 22.0, 16.0, "HangarWest")
    hangar(base, rng, 32.0, -22.0, 22.0, 16.0, "HangarEast")
    for i, x in enumerate((-30.0, 0.0, 30.0)):
        building(base, rng, "house", x, 40.0, rng.uniform(14.0, 18.0), 8.5, 0, f"Barracks{i}")
    for sx in (-1, 1):
        for sy in (-1, 1):
            watchtower(base, sx * (COMPOUND - 5.0), sy * (COMPOUND - 5.0), -sx, f"Tower{'W' if sx < 0 else 'E'}{'S' if sy < 0 else 'N'}")
    helipad(base, 0.0, -32.0)

    # The spawns, in the forest round the compound, facing in.
    for i in range(SPAWNS):
        a = 2 * math.pi * (i + 0.5) / SPAWNS
        r = (COMPOUND + HALF) / 2
        x = max(-HALF + 12, min(HALF - 12, r * 1.1 * math.cos(a)))
        y = max(-HALF + 12, min(HALF - 12, r * 1.1 * math.sin(a)))
        marker(f"spawn_{i + 1}", x, y, 0.0, math.atan2(-y, -x))
        base.keep_clear.append((x - 2.5, x + 2.5, y - 2.5, y + 2.5))

    scatter_cover(base, rng)

    # The forest: thick outside the compound, thinner on its apron, and on past the outer wall,
    # where none can go, so that the fog has trees to swallow there too.
    def plant(count, inside, outside_wall=False):
        planted = 0
        for _ in range(count * 8):
            if planted >= count:
                break
            if outside_wall:
                along, out = rng.uniform(-edge, edge), rng.uniform(HALF + 1.5, edge - 0.5)
                x, y = rng.choice(((along, out), (along, -out), (out, along), (-out, along)))
                tree(base, rng, x, y, solid=False)
                planted += 1
                continue
            x, y = rng.uniform(-HALF + 2, HALF - 2), rng.uniform(-HALF + 2, HALF - 2)
            within = max(abs(x), abs(y)) < COMPOUND
            if within != inside:
                continue
            if not base.clear(x, y, 0.5, TREE_GAP):
                continue
            tree(base, rng, x, y)
            planted += 1

    plant(520, inside=False)
    plant(45, inside=True)
    plant(220, inside=False, outside_wall=True)
    for i in range(40):
        for _ in range(40):
            x, y = rng.uniform(-HALF + 3, HALF - 3), rng.uniform(-HALF + 3, HALF - 3)
            if max(abs(x), abs(y)) > COMPOUND + 3 and base.clear(x, y, 2.0, 1.0):
                log(base, rng, x, y)
                break
    for i in range(320):
        x, y = rng.uniform(-HALF + 2, HALF - 2), rng.uniform(-HALF + 2, HALF - 2)
        if base.clear(x, y, 0.8, 0.6):
            bush(base, rng, x, y)
    base.trunks.build(base.struct)
    base.canopy.build(base.foliage)

    # A grey sun and a camera, to look it over in Blender.
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 60), rotation=(math.radians(40), 0, math.radians(30)))
    bpy.context.active_object.data.energy = 2
    bpy.ops.object.camera_add(location=(0, -HALF * 1.9, HALF * 1.2), rotation=(math.radians(55), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.55, 0.58, 0.58, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "br_forest.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {HALF * 2:.0f} m across, trees={base.n_trees} logs={base.n_logs} "
          f"bushes={base.n_bushes} cover={base.n_cover} trunk tris~{len(base.trunks.faces)} "
          f"leaf faces={len(base.canopy.faces)} -> {out}")
    if glb:
        export_tiled(os.path.abspath(glb), [base.ground, base.struct, base.cover, base.foliage])


def export_tiled(path, cols):
    """Exports for the game with each collection's meshes joined tile by tile - by where the
    middle of each one's bounds is - into `<collection>_<x>_<y>`: one mesh a tile and collection,
    not one for the whole map, which would be drawn whole wherever the camera looked. The spawns
    go as empties. Runs after the .blend is saved, which keeps every object separate."""
    from mathutils import Vector
    for col in cols:
        groups = {}
        for o in [o for o in col.objects if o.type == "MESH"]:
            corners = [o.matrix_world @ Vector(c) for c in o.bound_box]
            middle = sum(corners, Vector()) / 8
            groups.setdefault(tile_of(middle.x, middle.y), []).append(o)
        for (tx, ty), objs in sorted(groups.items()):
            bpy.ops.object.select_all(action="DESELECT")
            for o in objs:
                o.select_set(True)
            bpy.context.view_layer.objects.active = objs[0]
            if len(objs) > 1:
                bpy.ops.object.join()
            objs[0].name = f"{col.name}_{tx}_{ty}"
            bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_extras=True,
                              export_cameras=False, export_lights=False)
    print(f"[map] exported -> {path}")


if __name__ == "__main__":
    main()
