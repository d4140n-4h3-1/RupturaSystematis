"""
Team battle in the forest: two teams of five at the two ends of a clearing of abandoned buildings
in a foggy forest, the Foggy Forest's (see br_forest.py), smaller, for `examples/team_battle_forest`.

Along x, from the middle out to each team's end, the same either way round (turned half round
about the middle, so neither side has the better ground):

- The middle: a hangar, its roof fallen in - strips of it gone, a tree grown up through each
  hole - and a wreck inside: the ground both sides make for.
- A barracks for each side, one or two floors, doors and windows, stairs up inside.
- A ruin for each side: the walls of a building long gone, broken down and gapped, no roof,
  crates fallen inside.
- A watchtower near each end, a floor up, stairs up to it.
- Wrecks, rusting containers and sandbags between them, fallen trunks to crouch behind, and the
  forest all through: trunks solid, leaves and bushes not.
- A crumbling wall round it all, and a little forest past it, out of reach, that the fog swallows.

The trees, undergrowth, boulders and forest floor are the Wilderness's (plant_life's, CC0), copied
into data/forest. The trees and undergrowth are not in the map: each is an empty named
`Plant_<model>_<n>` (`_far` after it past the wall), turned and scaled as it grows, where the game plants a copy of
data/forest/trees|plants/<model>.glb - leaves and all, which hide whoever is in them but stop
nothing. Each tree's trunk is in it, as a plain eight-sided post the size of the model's trunk, in
`TreeTrunks_*` meshes: solid, for bolts, sight and the droids' ground as any trunk, and hidden by
the game behind the model's own bark. The boulders are cover, part of the map; the forest floor
is the loam's.

Builds data/arena/tb_forest.glb (and a .blend to look it over, wherever --out puts it):

    blender -b --python data/arena/tb_forest.py -- --seed 3 --out ~/Documents/blender/map/tb_forest.blend --glb data/arena/tb_forest.glb

Made for Ruptura Systematis (MazeGame/maze). The game puts each team down at its own end of x.
"""

import bpy
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import br_town  # noqa: E402
from br_town import add_box, add_block, building, collection  # noqa: E402
import br_forest  # noqa: E402
from br_forest import Base, Tiled, prism, log, broken_wall, hangar, watchtower, export_tiled, WALL_T, \
    TREE_GAP  # noqa: E402

BR_FOREST_TREE = br_forest.tree  # the hangar's, before it is given the Wilderness's

HX, HY = 56.0, 32.0   # from the middle to the wall round it, along x and y
FOREST = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "forest")

# The trees, as the Wilderness grows them (its forest.rs SPECIES): the model, how common it is
# against the others, and its trunk's radius near the ground at full size, in meters. Alder and
# willow, which grow only by water, are left out: there is none here.
TREES = [("norway-spruce", 3.0, 0.28), ("pine", 2.5, 0.28), ("silver-fir", 1.5, 0.30),
         ("european-larch", 1.5, 0.27), ("silver-birch", 1.5, 0.18), ("rowan", 0.8, 0.20),
         ("english-oak", 0.7, 0.45)]
TREE_SCALE = (0.95, 1.35)  # the models are 7-13 m; the forest's trees were 8-19
TRUNK_POST = 7.0           # how high a trunk is solid, in meters: over a watchtower's platform
# How far past the wall a tree's shadow still reaches into the arena: under the team battle's sun,
# 60 degrees up, a 12 m tree's runs 7 m, over the 3.5 m wall about 5 m. Trees further out cast none.
SHADOW_REACH = 5.0
TREE_APART = 6.0           # the least distance between two trees' trunks, spread out
TREE_TRIES = 30            # places clear for each pair of trees tried, the one furthest from the rest taken
# The undergrowth in place of the bushes: the model and how common it is, the big ones scarcer.
UNDERGROWTH = [("common-hawthorn", 1.0), ("common-broom", 1.0), ("bracken", 3.0), ("hard-fern", 2.0),
               ("bilberry", 2.0)]
LOAM_TILE = 1.0            # meters of ground to one of the loam's tiles, as the Wilderness has it
BEYOND = 10.0         # forest past the wall, out of reach
EDGE_WALL = 3.5
SPAWN_ROOM = 7.0      # kept clear of trees at each end, for the teams to be put down


def ruin(base, rng, cx, cy, w, d, name):
    """The walls of a building long gone: broken down, ragged, a doorway in each and a gap or two
    more, no roof - and crates fallen inside."""
    height = 2.7
    mat = rng.choice(base.walls)
    for along_x, at, a0, a1 in ((True, cy - d / 2, cx - w / 2, cx + w / 2), (True, cy + d / 2, cx - w / 2, cx + w / 2),
                                (False, cx - w / 2, cy - d / 2, cy + d / 2), (False, cx + w / 2, cy - d / 2, cy + d / 2)):
        length = a1 - a0
        gaps = [(a0 + length * rng.uniform(0.3, 0.7), rng.uniform(1.4, 2.4))]
        if rng.random() < 0.5:
            gaps.append((a0 + length * rng.choice((0.15, 0.85)), rng.uniform(1.0, 1.8)))
        broken_wall(base, f"{name}_{'x' if along_x else 'y'}{at:.0f}", along_x, at, a0 - WALL_T / 2, a1 + WALL_T / 2,
                    height, gaps, mat)
    for i in range(rng.randint(2, 3)):
        x, y = cx + rng.uniform(-w / 3, w / 3), cy + rng.uniform(-d / 3, d / 3)
        add_block(f"{name}_Crate{i}", x, y, 1.4, 1.4, 1.2, rng.uniform(0, math.pi), base.cover, base.low, "low")
    base.keep_clear.append((cx - w / 2 - 1.0, cx + w / 2 + 1.0, cy - d / 2 - 1.0, cy + d / 2 + 1.0))
    base.n_buildings += 1


def weighted(rng, kinds):
    """One of `kinds`, (name, weight, ...) each, as likely as its weight."""
    pick = rng.uniform(0, sum(k[1] for k in kinds))
    for kind in kinds:
        pick -= kind[1]
        if pick <= 0:
            return kind
    return kinds[-1]


def plant(base, name, x, y, yaw, scale, far=False):
    """An empty where the game plants a copy of the model `name`, turned and scaled; `far`, out of
    reach past the wall, where the game has it cast no shadow."""
    base.n_plants = getattr(base, "n_plants", 0) + 1
    empty = bpy.data.objects.new(f"Plant_{name}_{base.n_plants}{'_far' if far else ''}", None)
    empty.location = (x, y, 0.0)
    empty.rotation_euler.z = yaw
    empty.scale = (scale, scale, scale)
    bpy.context.scene.collection.objects.link(empty)


def forest_tree(base, rng, x, y, solid=True, far=False):
    """One of the Wilderness's trees at (x, y), and - if it can be reached - its trunk, solid;
    `far`, so far out past the wall that its shadow falls short of it, and the game has it cast
    none."""
    name, _, trunk = weighted(rng, TREES)
    scale = rng.uniform(*TREE_SCALE)
    plant(base, name, x, y, rng.uniform(0, 2 * math.pi), scale, far=far)
    base.n_trees += 1
    if solid:
        r = trunk * scale
        base.posts.at = (x, y)
        prism(base.posts, x, y, -0.1, TRUNK_POST, r, r * 0.8, 8)
        base.solid.append((x, y, r))


def hangar_tree(base, rng, x, y, solid=True):
    """In place of br_forest's tree, for the hangar to grow up through its roof: one of the
    Wilderness's. br_forest's still grows - its trunk thrown away, its leaves never built - so
    that it takes from `rng` as it did, and the rest of the map comes out as it was; the
    Wilderness's is chosen by where it stands."""
    trunks, n_trees = base.trunks, base.n_trees
    base.trunks = Tiled("Discarded", [base.bark])
    BR_FOREST_TREE(base, rng, x, y, solid)
    base.trunks, base.n_trees = trunks, n_trees
    forest_tree(base, random.Random(f"{x:.3f},{y:.3f}"), x, y, solid)


def undergrowth(base, rng, x, y):
    """A bush, bracken or ferns at (x, y): leaves, which stop nothing."""
    name, _ = weighted(rng, UNDERGROWTH)
    plant(base, name, x, y, rng.uniform(0, 2 * math.pi), rng.uniform(0.8, 1.25))
    base.n_bushes += 1


def imported(path):
    """The mesh object of the glTF at `path`, out of every collection: to copy, not to export."""
    bpy.ops.object.select_all(action="DESELECT")
    bpy.ops.import_scene.gltf(filepath=path)
    objects = list(bpy.context.selected_objects)
    mesh = next(o for o in objects if o.type == "MESH")
    for o in objects:
        for c in list(o.users_collection):
            c.objects.unlink(o)
    mesh.parent = None
    return mesh


def boulders(base, rng, pairs):
    """The Wilderness's granite boulders, in pairs each the other turned half round about the
    middle: low cover to crouch behind, among the trees."""
    rock = imported(os.path.join(FOREST, "rocks", "granite-boulder.glb"))
    placed = 0
    for i in range(pairs):
        for _ in range(80):
            scale = rng.uniform(1.2, 1.7)
            r = 0.55 * scale
            x, y = rng.uniform(-HX + SPAWN_ROOM + 3, HX - SPAWN_ROOM - 3), rng.uniform(-HY + 3, HY - 3)
            if abs(x) < r + 1.0 or not base.clear(x, y, r) or not base.clear(-x, -y, r):
                continue
            yaw = rng.uniform(0, 2 * math.pi)
            for k, (px, py) in enumerate(((x, y), (-x, -y))):
                o = rock.copy()
                o.data = rock.data.copy()
                o.name = f"Cover_boulder_{i}_{k}"
                o.location = (px, py, -0.05 * scale)
                o.rotation_euler = (0.0, 0.0, yaw + math.pi * k)
                o.scale = (scale, scale * rng.uniform(0.85, 1.15), scale)
                o["cover"] = "low"
                base.cover.objects.link(o)
                base.solid.append((px, py, r))
            placed += 2
            break
    base.n_boulders = placed


def loam(obj):
    """Gives `obj` the Wilderness's forest loam, a tile to every LOAM_TILE meters across it."""
    sample = imported(os.path.join(FOREST, "ground", "forest-loam.glb"))
    material = sample.data.materials[0]
    obj.data.materials.clear()
    obj.data.materials.append(material)
    uv = obj.data.uv_layers.new(name="UVMap")
    for loop in obj.data.loops:
        v = obj.matrix_world @ obj.data.vertices[loop.vertex_index].co
        uv.data[loop.index].uv = (v.x / LOAM_TILE, v.y / LOAM_TILE)


def cover_pair(base, rng, kinds):
    """Cover put down in pairs, each the other turned half round about the middle: wrecks,
    containers - now and then two high - sandbags and crates."""
    placed = 0
    for i, kind in enumerate(kinds):
        w, d, h, mat, cover = {
            "container": (6.0, 2.4, 2.6, base.high, "high"),
            "wreck": (4.4, 2.0, 1.6, base.rust, "low"),
            "sandbags": (3.0, 0.8, br_town.RAIL, base.low, "low"),
            "crate": (1.4, 1.4, 1.2, base.low, "low"),
        }[kind]
        r = math.hypot(w, d) / 2
        for _ in range(80):
            x, y = rng.uniform(-HX + SPAWN_ROOM + 3, HX - SPAWN_ROOM - 3), rng.uniform(-HY + 3, HY - 3)
            if abs(x) < r + 1.0 or not base.clear(x, y, r) or not base.clear(-x, -y, r):
                continue
            rot = rng.choice((0.0, math.pi / 2, rng.uniform(0, math.pi)))
            stacked = kind == "container" and rng.random() < 0.3
            for k, (px, py) in enumerate(((x, y), (-x, -y))):
                add_block(f"Cover_{kind}_{i}_{k}", px, py, w, d, h, rot, base.cover, mat, cover)
                if stacked:
                    add_block(f"Cover_{kind}_{i}_{k}_Up", px, py, w, d, h, rot + 0.1, base.cover, mat, cover, z=h)
                base.solid.append((px, py, r))
            placed += 2
            break
    base.n_cover = placed


def main():
    seed, glb, out = br_town.get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    base = Base()
    base.n_buildings = 0
    base.trunk_posts = collection("TreeTrunks")
    base.posts = Tiled("TreeTrunks", [base.bark])

    # The forest floor, out past the wall: the Wilderness's loam.
    loam(add_box("ForestFloor", -HX - BEYOND, HX + BEYOND, -HY - BEYOND, HY + BEYOND, -1.0, 0.0, base.ground, base.grass))

    # The wall round it, crumbling.
    for k, (along_x, at, half) in enumerate(((True, HY, HX), (True, -HY, HX), (False, HX, HY), (False, -HX, HY))):
        broken_wall(base, f"EdgeWall{k}", along_x, at, -half - WALL_T, half + WALL_T, EDGE_WALL, [], base.moss)
    for along_x, at, half in ((True, HY, HX), (True, -HY, HX), (False, HX, HY), (False, -HX, HY)):
        if along_x:
            base.keep_clear.append((-half, half, at - 0.6, at + 0.6))
        else:
            base.keep_clear.append((at - 0.6, at + 0.6, -half, half))

    # Each team's end, kept clear for it to be put down.
    for sx in (-1, 1):
        base.keep_clear.append((min(sx * HX, sx * (HX - SPAWN_ROOM)), max(sx * HX, sx * (HX - SPAWN_ROOM)), -HY * 0.6, HY * 0.6))

    # The buildings: the hangar in the middle, and each side's own, turned half round about it.
    # Its trees, through the holes in its roof, the Wilderness's too.
    br_forest.tree = hangar_tree
    try:
        hangar(base, rng, 0.0, 0.0, 20.0, 14.0, "Hangar")
    finally:
        br_forest.tree = BR_FOREST_TREE
    floors = 1 + (rng.random() < 0.6)
    width = rng.uniform(13.0, 16.0)
    for k, (x, y, quarter) in enumerate(((-26.0, 17.0, 0), (26.0, -17.0, 2))):
        made = 0
        while made != floors:
            for o in [o for o in base.struct.objects if o.name.startswith(f"Barracks{k}_")]:
                bpy.data.objects.remove(o)
            base.keep_clear = [c for c in base.keep_clear if c != (x - width / 2 - 1.0, x + width / 2 + 1.0, y - 4.25 - 1.0, y + 4.25 + 1.0)]
            made = building(base, rng, "house", x, y, width, 8.5, quarter, f"Barracks{k}")
    ruin_w, ruin_d = rng.uniform(9.0, 12.0), rng.uniform(7.0, 9.0)
    ruin_rng = rng.random()
    for k, (x, y) in enumerate(((-24.0, -15.0), (24.0, 15.0))):
        ruin(base, random.Random(ruin_rng), x, y, ruin_w, ruin_d, f"Ruin{k}")
    watchtower(base, -(HX - 14.0), -HY + 9.0, 1, "TowerWest")
    watchtower(base, HX - 14.0, HY - 9.0, -1, "TowerEast")

    cover_pair(base, rng, ["wreck", "container", "container", "sandbags", "sandbags", "crate", "crate", "wreck",
                           "sandbags", "container", "crate", "sandbags"])
    boulders(base, rng, 10)

    # The forest: all through it, and on past the wall, where none can go, for the fog. Spread
    # out: each pair of trees goes, of TREE_TRIES places clear for it, at the one furthest from
    # every tree already planted, and none nearer another than TREE_APART. Put down wherever
    # there was room, TREE_GAP from the next, they piled up in the few gaps the cover left -
    # four clumps of crowns grown into each other, and the rest of the ground bare.
    trees = []
    for _ in range(80):
        best, tried = None, 0
        for _ in range(TREE_TRIES * 40):
            if tried == TREE_TRIES:
                break
            x, y = rng.uniform(-HX + 2, HX - 2), rng.uniform(-HY + 2, HY - 2)
            if not (base.clear(x, y, 0.5, TREE_GAP) and base.clear(-x, -y, 0.5, TREE_GAP)):
                continue
            tried += 1
            apart = min((math.hypot(x - tx, y - ty) for tx, ty in trees), default=math.inf)
            apart = min(apart, math.hypot(2 * x, 2 * y))
            if best is None or apart > best[0]:
                best = (apart, x, y)
        if best is None or best[0] < TREE_APART:
            break
        _, x, y = best
        forest_tree(base, rng, x, y)
        forest_tree(base, rng, -x, -y)
        trees += [(x, y), (-x, -y)]
    for _ in range(110):
        along = rng.random()
        side = rng.randrange(4)
        past = rng.uniform(1.5, BEYOND - 0.5)
        if side < 2:
            x, y = -HX - BEYOND + along * 2 * (HX + BEYOND), (HY + past) * (1 if side == 0 else -1)
        else:
            x, y = (HX + past) * (1 if side == 2 else -1), -HY - BEYOND + along * 2 * (HY + BEYOND)
        forest_tree(base, rng, x, y, solid=False, far=past > SHADOW_REACH)
    for _ in range(12):
        for _ in range(60):
            x, y = rng.uniform(-HX + SPAWN_ROOM + 2, HX - SPAWN_ROOM - 2), rng.uniform(-HY + 3, HY - 3)
            if base.clear(x, y, 3.0, 0.5) and base.clear(-x, -y, 3.0, 0.5):
                state = rng.getstate()
                log(base, rng, x, y)
                rng.setstate(state)
                log(base, rng, -x, -y)
                break
    for _ in range(140):
        x, y = rng.uniform(-HX + 2, HX - 2), rng.uniform(-HY + 2, HY - 2)
        if base.clear(x, y, 0.8, 0.6):
            undergrowth(base, rng, x, y)
    # Low growth over the rest of the floor, closer together: bracken, ferns and bilberry, below
    # the knee - it hides nothing, and stops nothing.
    low = [kind for kind in UNDERGROWTH if kind[0] in ("bracken", "hard-fern", "bilberry")]
    for _ in range(500):
        x, y = rng.uniform(-HX + 1, HX - 1), rng.uniform(-HY + 1, HY - 1)
        if base.clear(x, y, 0.3, 0.1):
            name, _ = weighted(rng, low)
            plant(base, name, x, y, rng.uniform(0, 2 * math.pi), rng.uniform(0.7, 1.1))
            base.n_bushes += 1
    base.trunks.build(base.struct)
    base.posts.build(base.trunk_posts)

    # A grey sun and a camera, to look it over in Blender.
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 60), rotation=(math.radians(40), 0, math.radians(30)))
    bpy.context.active_object.data.energy = 2
    bpy.ops.object.camera_add(location=(0, -HY * 2.6, HY * 1.8), rotation=(math.radians(55), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.55, 0.58, 0.58, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "tb_forest.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {HX * 2:.0f} x {HY * 2:.0f} m, buildings={base.n_buildings} trees={base.n_trees} "
          f"logs={base.n_logs} undergrowth={base.n_bushes} boulders={base.n_boulders} cover={base.n_cover} -> {out}")
    if glb:
        # Every mesh with a UV map of the boulders' name: joined tile by tile, a boulder's own
        # map is kept only if what it is joined with has one by that name too - crates and
        # sandbags have none - or its textures are left with none to follow.
        cols = [base.ground, base.struct, base.cover, base.trunk_posts]
        for o in [o for col in cols for o in col.objects if o.type == "MESH"]:
            if "UVMap" not in o.data.uv_layers:
                o.data.uv_layers.new(name="UVMap")
        export_tiled(os.path.abspath(glb), cols)


if __name__ == "__main__":
    main()
