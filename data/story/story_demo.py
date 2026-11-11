"""
The demo town: a small city to try the city's droids in - its crowd, its gang, its pairs, what
they think of the player and what they pass on - without waiting on Urbs. It loads in seconds.

Builds data/story/story_demo.glb, and story_demo.roads.json beside it (and a .blend to look it
over, wherever --out puts it):

    blender -b --python data/story/story_demo.py -- --seed 5 --glb data/story/story_demo.glb

Played with MAZE_MODEL=data/story/story_demo.glb. Its markers `place_station` and `place_summa`
make it a story's place, so the game peoples it as it does Urbs (data/dialogue/urbs.json): the
crowd turning up about the player, security in pairs, citizens, and the gang's groups keeping
to turf of their own.

Layout (top view): BLOCKS x BLOCKS blocks, BLOCK meters square, a kerb's height over streets
STREET wide between them and round them, inside a wall: the streets at the ground, the blocks
and all on them a kerb up. The middle block is an open plaza; the
others each hold one of the town's buildings (see data/arena/br_town.py), every one open to go
into. Crossings at every junction, striped, where the droids step onto the road; everywhere
else they keep to the pavements. The player starts at the south-west corner (`place_station`);
the north-east corner is `place_summa`. It is day.

For its townsfolk's days (see the game's routines): `home_<n>` before each house and apartment
block, `work_<n>` before each office and warehouse, `place_plaza` in the middle; and along the
plaza's south edge, facing the street, a vending machine of each kind, `vending_<kind>_1`.
"""

import bpy
import json
import math
import os
import random
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "arena"))
import br_town  # noqa: E402  - the town's buildings, boxes, markers and export
from br_town import add_box, marker, building, square, export_glb, Town, KINDS, material  # noqa: E402

BLOCKS = 3
BLOCK = 28.0          # a block's side, meters
STREET = 16.0         # a street's width, kerb to kerb, as Urbs's: two lanes each way for its traffic
KERB = 0.12           # how far the streets are under the blocks: less than a step
HALF = (BLOCKS * BLOCK + (BLOCKS + 1) * STREET) / 2   # half the town's side, inside its wall
TOWN_WALL = 4.0
STRIPE = (0.5, 1.0)   # a crossing's stripes: each one's width, and from one to the next
CROSSING = 2.0        # how wide a crossing is, along the street: 8.5 to 10.5 m out, as the traffic stops behind


class lifted:
    """Raises everything made inside it by `by` meters: the town's buildings and its square
    stand on the ground, and here the blocks are a kerb over it."""

    def __init__(self, by):
        self.by = by

    def __enter__(self):
        self.before = set(bpy.data.objects)

    def __exit__(self, *_):
        for o in set(bpy.data.objects) - self.before:
            o.location.z += self.by


def get_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    seed = int(argv[argv.index("--seed") + 1]) if "--seed" in argv else 5
    glb = argv[argv.index("--glb") + 1] if "--glb" in argv else None
    out = argv[argv.index("--out") + 1] if "--out" in argv else None
    return seed, glb, out


def street_lines():
    """The middle of every street, along x and along y alike: between the blocks, and round them."""
    pitch = BLOCK + STREET
    return [-HALF + STREET / 2 + k * pitch for k in range(BLOCKS + 1)]


def crossings():
    """Every crossing: across each street, either side of each junction - (x0, x1, y0, y1)."""
    found = []
    lines = street_lines()
    for jx in lines:
        for jy in lines:
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                ox, oy = jx + dx * (STREET / 2 + CROSSING / 2), jy + dy * (STREET / 2 + CROSSING / 2)
                if abs(ox) > HALF - 1 or abs(oy) > HALF - 1:
                    continue
                if dx:
                    found.append((ox - CROSSING / 2, ox + CROSSING / 2, jy - STREET / 2, jy + STREET / 2))
                else:
                    found.append((jx - STREET / 2, jx + STREET / 2, oy - CROSSING / 2, oy + CROSSING / 2))
    return found


def road_plan():
    """Where the roads are, for the game's droids to keep off but to cross: every street, kerb to
    kerb, and the crossings over them - each [x0, z0, x1, z1] in the game's terms (Blender's x,
    and -y); road wherever the floor is under half the kerb."""
    game = lambda x0, x1, y0, y1: [round(x0, 3), round(-y1, 3), round(x1, 3), round(-y0, 3)]
    roads = []
    for c in street_lines():
        roads.append(game(c - STREET / 2, c + STREET / 2, -HALF, HALF))
        roads.append(game(-HALF, HALF, c - STREET / 2, c + STREET / 2))
    return {"level": KERB / 2, "roads": roads, "crossings": [game(*c) for c in crossings()]}


def main():
    seed, glb, out = get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    town = Town()
    stripes = material("Stripes", (0.85, 0.85, 0.8))

    # The streets, a kerb under the blocks; the blocks' pavements, and yards of grass on them.
    add_box("Streets", -HALF, HALF, -HALF, HALF, -1.0, 0.0, town.ground, town.road)
    pitch = BLOCK + STREET
    first = -HALF + STREET + BLOCK / 2
    middle = BLOCKS // 2
    t = 1.0
    for n, (x0, x1, y0, y1) in enumerate([(-HALF - t, HALF + t, HALF, HALF + t), (-HALF - t, HALF + t, -HALF - t, -HALF),
                                           (-HALF - t, -HALF, -HALF, HALF), (HALF, HALF + t, -HALF, HALF)]):
        add_box(f"TownWall_{n}", x0, x1, y0, y1, 0.0, TOWN_WALL, town.struct, town.town_wall)

    kinds, weights = zip(*[(k, v[3]) for k, v in KINDS.items()])
    homes = works = 0
    for bi in range(BLOCKS):
        for bj in range(BLOCKS):
            cx, cy = first + bi * pitch, first + bj * pitch
            name = f"Block{bi}{bj}"
            add_box(f"{name}_Pavement", cx - BLOCK / 2, cx + BLOCK / 2, cy - BLOCK / 2, cy + BLOCK / 2, -1.0, KERB,
                    town.ground, town.pavement)
            if bi == middle and bj == middle:
                with lifted(KERB):
                    square(town, rng, cx, cy, name)
                # Where the town comes to pass the time, and along its south edge, backs to it,
                # facing the street, its vending machines: one of each.
                marker("place_plaza", cx, cy, KERB, -math.pi / 2)
                for k, kind in enumerate(("repair", "stamina", "shield")):
                    marker(f"vending_{kind}_1", cx + (k - 1) * 1.3, cy - BLOCK / 2 + 0.9, KERB, -math.pi / 2)
                continue
            # The yard: grass inside a ring of pavement, a hair over it.
            add_box(f"{name}_Yard", cx - BLOCK / 2 + 2.5, cx + BLOCK / 2 - 2.5, cy - BLOCK / 2 + 2.5, cy + BLOCK / 2 - 2.5,
                    KERB, KERB + 0.004, town.ground, town.grass)
            kind = rng.choices(kinds, weights)[0]
            _, wr, dr, _ = KINDS[kind]
            w, d = rng.uniform(*wr), rng.uniform(*dr)
            ox = rng.uniform(-(BLOCK - w) / 2 + 3, (BLOCK - w) / 2 - 3)
            oy = rng.uniform(-(BLOCK - d) / 2 + 3, (BLOCK - d) / 2 - 3)
            with lifted(KERB):
                building(town, rng, kind, cx + ox, cy + oy, w, d, 0, name)
            # Somewhere to live, or to work: before its front, which faces -y. Marked after the
            # building, drawing nothing from rng, so the town is as it was.
            if kind in ("house", "apartment"):
                homes += 1
                marker(f"home_{homes}", cx + ox, cy + oy - d / 2 - 1.5, KERB, -math.pi / 2)
            else:
                works += 1
                marker(f"work_{works}", cx + ox, cy + oy - d / 2 - 1.5, KERB, -math.pi / 2)

    # The crossings, striped across the road.
    for n, (x0, x1, y0, y1) in enumerate(crossings()):
        along_x = (x1 - x0) > (y1 - y0)
        length = (x1 - x0) if along_x else (y1 - y0)
        k = 0
        while (k + 1) * STRIPE[1] <= length:
            a = k * STRIPE[1] + (STRIPE[1] - STRIPE[0]) / 2
            if along_x:
                add_box(f"Crossing{n}_{k}", x0 + a, x0 + a + STRIPE[0], y0, y1, 0.0, 0.004, town.ground, stripes)
            else:
                add_box(f"Crossing{n}_{k}", x0, x1, y0 + a, y0 + a + STRIPE[0], 0.0, 0.004, town.ground, stripes)
            k += 1

    # Where a story starts and ends: on the pavement at the south-west corner, facing in, and at
    # the north-east.
    corner = first - BLOCK / 2 + 1.5
    marker("place_station", corner, corner, KERB, math.pi / 4)
    marker("place_summa", -corner, -corner, KERB, -3 * math.pi / 4)

    bpy.ops.object.light_add(type="SUN", location=(0, 0, 60), rotation=(math.radians(50), 0, math.radians(30)))
    bpy.context.active_object.data.energy = 3
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.55, 0.65, 0.8, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(os.path.expanduser(out)) if out else None
    if out:
        bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} {HALF * 2:.0f} m across, buildings={town.n_buildings} cover={town.n_cover}")
    if glb:
        glb = os.path.abspath(glb)
        export_glb(glb, [town.ground, town.struct, town.cover])
        roads_path = os.path.splitext(glb)[0] + ".roads.json"
        with open(roads_path, "w") as f:
            json.dump(road_plan(), f)
        print(f"[map] roads and crossings -> {roads_path}")


if __name__ == "__main__":
    main()
