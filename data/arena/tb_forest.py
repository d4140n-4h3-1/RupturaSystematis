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

The canopies and bushes go in meshes named `Foliage_*`, which the game leaves out of the level's
collider - they hide whoever is in them, but bolts and droids go through.

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
from br_town import add_box, add_block, building  # noqa: E402
import br_forest  # noqa: E402
from br_forest import Base, tree, bush, log, broken_wall, hangar, watchtower, export_tiled, WALL_T, \
    TREE_GAP  # noqa: E402

HX, HY = 56.0, 32.0   # from the middle to the wall round it, along x and y
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

    # The forest floor, out past the wall.
    add_box("ForestFloor", -HX - BEYOND, HX + BEYOND, -HY - BEYOND, HY + BEYOND, -1.0, 0.0, base.ground, base.grass)

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
    hangar(base, rng, 0.0, 0.0, 20.0, 14.0, "Hangar")
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

    # The forest: all through it, and on past the wall, where none can go, for the fog.
    planted = 0
    for _ in range(8000):
        if planted >= 80:
            break
        x, y = rng.uniform(-HX + 2, HX - 2), rng.uniform(-HY + 2, HY - 2)
        if base.clear(x, y, 0.5, TREE_GAP) and base.clear(-x, -y, 0.5, TREE_GAP):
            tree(base, rng, x, y)
            tree(base, rng, -x, -y)
            planted += 1
    for _ in range(110):
        along = rng.random()
        side = rng.randrange(4)
        past = rng.uniform(1.5, BEYOND - 0.5)
        if side < 2:
            x, y = -HX - BEYOND + along * 2 * (HX + BEYOND), (HY + past) * (1 if side == 0 else -1)
        else:
            x, y = (HX + past) * (1 if side == 2 else -1), -HY - BEYOND + along * 2 * (HY + BEYOND)
        tree(base, rng, x, y, solid=False)
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
            bush(base, rng, x, y)
    base.trunks.build(base.struct)
    base.canopy.build(base.foliage)

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
          f"logs={base.n_logs} bushes={base.n_bushes} cover={base.n_cover} -> {out}")
    if glb:
        export_tiled(os.path.abspath(glb), [base.ground, base.struct, base.cover, base.foliage])


if __name__ == "__main__":
    main()
