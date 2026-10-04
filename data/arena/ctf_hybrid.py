"""
Hybrid capture-the-flag arena: the walled bases and lanes of "Lanes", the two floors of
"Balconies", and a maze in the middle, as the game's own levels are.

Run headless:  blender --background --python generate_hybrid_ctf_map.py -- --seed 7 [--out hybrid_ctf_map.blend] [--glb path.glb]
Or open in Blender > Scripting tab > Run Script.

Layout (top view, x runs along the length, red base at -x and blue at +x; mirrored through the
middle, so neither side gets a better layout):

    +--------------------------------------------------------------------------------+
    |BBBB catwalk (upper) ===== stairs        overlook        stairs ===== catwalk BBBB|
    |BBBB back corridor            side lane                          back corridor BBBB|
    |BB+---door---+   +--lane wall--door--+--maze--+--door--lane wall--+   +---door---+BB|
    |BB| balcony  |   |      yard       maze       maze       yard     |   | balcony  |BB|
    |BB|   well F door              entry  plaza+hub  entry           door F well   |BB|
    |BB| balcony  |   |      yard       maze       maze       yard     |   | balcony  |BB|
    |BB+---door---+   +--lane wall--door--+--maze--+--door--lane wall--+   +---door---+BB|
    |BBBB back corridor            side lane                          back corridor BBBB|
    |BBBB catwalk (upper) ===== stairs        overlook        stairs ===== catwalk BBBB|
    +--------------------------------------------------------------------------------+

- The bases: a walled room at each end in its side's colours, with a front door onto the yard
  and a door in each side wall onto a back corridor. Inside, the flag stands on the ground in a
  well, under a balcony round three sides of it; an opening at the back of the balcony leads out
  through the side wall onto a catwalk along the outer wall, which comes down by stairs into the
  side lane - the way round the back on the upper floor.
- The middle lane is a maze of corridors, 4 m cells carved from the seed, mirrored through the
  middle, with loops knocked through so it is never one way only, an open plaza in the middle
  round a raised hub, and doorways out to the yards and to both side lanes.
- The side lanes run between the lane walls and the outer walls, each with an overlook in the
  middle - a raised floor with stairs up from both ends - and cover to dodge between.

Made for Ruptura Systematis (MazeGame/maze), whose capture the flag reads the map's empties: the
game puts each side's flag in a firewall at `flag_red` / `flag_blue`, the computer that opens it
at `computer_red` / `computer_blue`, and its droids at `post_red_1` and on. Its survey of where
they can walk keeps one floor to each spot, the ground first, so every raised floor is solid down
to the ground, and reached by stairs of 0.25 m steps. As the game needs, there is a ceiling over
everything and the light fixtures' glass is pure magenta.
"""

import bpy
import math
import random
import sys
import os

HX, HY = 48.0, 28.0  # half the hall's length (x) and width (y), inside the outer walls
CEILING = 7.0        # the game's lamps reach 7.5 m, so ceiling light still reaches the ground
UPPER = 3.5          # the upper floor: balconies and catwalks
LINTEL = 6.0         # top of an opening through a wall on the upper floor
WALL = 0.6           # inner walls' thickness
STEP_RISE = 0.25
STEP_RUN = 0.35
STAIR_WIDTH = 2.5
RAIL = 1.0           # railings, and low cover, are crouched behind
HIGH = 2.2           # tall cover is stood behind

# The bases
BASE_FRONT = 34.0    # |x| of a base's front wall
BASE_SIDE = 15.0     # |y| of its side walls
BACK = 3.0           # depth of the balcony along the end wall
SIDE_BALCONY = 5.0   # depth of the balconies along the side walls
BALCONY_END = 39.0   # |x| where the side balconies end, toward the front
FLAG_X = 37.5        # |x| of the flag

# The back corridors and catwalks
CATWALK = 4.0        # width of the catwalk along the outer wall
CATWALK_END = 22.0   # |x| where it comes down to the side lane

# The middle
LANE = 10.0          # |y| of the lane walls
MAZE_X = 20.0        # |x| of the maze's ends
CELL = 4.0           # the maze's cells
EXTRA_LOOPS = 6      # walls knocked through on top of the maze, half of them mirrored
HUB = 4.0            # side of the raised hub in the plaza
HUB_HEIGHT = 1.75
OVERLOOK = 8.0       # length of each side lane's overlook
OVERLOOK_HEIGHT = 2.0

NUM_COVER = 16       # cover blocks per half (mirrored onto the other half)
MIN_GAP = 2.0        # walkable space kept round cover
LIGHT_CLEAR = 0.8    # how far a light strip keeps from any wall


def get_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    seed = int(argv[argv.index("--seed") + 1]) if "--seed" in argv else random.randrange(1_000_000)
    glb = argv[argv.index("--glb") + 1] if "--glb" in argv else None
    out = argv[argv.index("--out") + 1] if "--out" in argv else None
    return seed, glb, out


def material(name, rgb):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    m.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*rgb, 1)
    m.diffuse_color = (*rgb, 1)
    return m


def collection(name):
    c = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(c)
    return c


def move_to(obj, col):
    for c in obj.users_collection:
        c.objects.unlink(obj)
    col.objects.link(obj)


# Every full-height wall's footprint (x0, x1, y0, y1), to keep the lights clear of them.
WALLS = []


def add_block(name, x, y, w, d, h, rot, col, mat, cover, z=0.0):
    """A box whose bottom sits at height z."""
    bpy.ops.mesh.primitive_cube_add(size=1, location=(x, y, z + h / 2))
    o = bpy.context.active_object
    o.name = name
    o.scale = (w, d, h)
    o.rotation_euler.z = rot
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    o.data.materials.append(mat)
    o["cover"] = cover
    move_to(o, col)
    return o


def add_box(name, x0, x1, y0, y1, z0, z1, col, mat, cover="none"):
    """An axis-aligned box given by its extents, in either order."""
    x0, x1 = sorted((x0, x1))
    y0, y1 = sorted((y0, y1))
    if z1 >= CEILING - 0.01 and z0 <= 0.01:
        WALLS.append((x0, x1, y0, y1))
    return add_block(name, (x0 + x1) / 2, (y0 + y1) / 2, x1 - x0, y1 - y0, z1 - z0, 0, col, mat, cover, z=z0)


def add_hexagon(name, x, y, radius, col, mat):
    bpy.ops.mesh.primitive_cylinder_add(vertices=6, radius=radius, depth=CEILING, location=(x, y, CEILING / 2),
                                        rotation=(0, 0, math.pi / 6))
    o = bpy.context.active_object
    o.name = name
    o.data.materials.append(mat)
    o["cover"] = "high"
    move_to(o, col)
    WALLS.append((x - radius, x + radius, y - radius, y + radius))


def add_stairs(name, x, y, direction, rise, col, mat):
    """Solid stairs whose top step meets height `rise` at (x, y) and that descend toward
    `direction` ('+x', '-x', '+y' or '-y'). Returns the footprint (x0, x1, y0, y1)."""
    n = round(rise / STEP_RISE)
    sign = 1 if direction[0] == "+" else -1
    along_x = direction[1] == "x"
    w = STAIR_WIDTH / 2
    for i in range(n):
        top = rise - i * STEP_RISE                      # step i is i steps down from the top
        a, b = i * STEP_RUN * sign, (i + 1) * STEP_RUN * sign
        if along_x:
            add_box(f"{name}_{i}", x + a, x + b, y - w, y + w, 0, top, col, mat)
        else:
            add_box(f"{name}_{i}", x - w, x + w, y + a, y + b, 0, top, col, mat)
    run = n * STEP_RUN * sign
    if along_x:
        return (*sorted((x, x + run)), y - w, y + w)
    return (x - w, x + w, *sorted((y, y + run)))


def add_rail(name, a0, a1, at, along_x, gaps, z, col, mat):
    """A railing on a raised floor at height z from a0 to a1 along x (at y = `at`) or along y
    (at x = `at`), broken at each (gap_start, gap_end)."""
    a0, a1 = sorted((a0, a1))
    edges = [a0] + [g for gap in sorted(tuple(sorted(gap)) for gap in gaps) for g in gap] + [a1]
    for i in range(0, len(edges), 2):
        if edges[i + 1] - edges[i] > 0.1:
            span = (edges[i], edges[i + 1])
            x0, x1, y0, y1 = (*span, at - 0.1, at + 0.1) if along_x else (at - 0.1, at + 0.1, *span)
            add_box(f"{name}_{i // 2}", x0, x1, y0, y1, z, z + RAIL, col, mat, "low")


def add_wall(name, x0, x1, y0, y1, doors, along_x, col, mat):
    """A full-height wall over the footprint, with a doorway at each (start, end) along it."""
    a0, a1 = (x0, x1) if along_x else (y0, y1)
    a0, a1 = sorted((a0, a1))
    edges = [a0] + [g for door in sorted(tuple(sorted(d)) for d in doors) for g in door] + [a1]
    for i in range(0, len(edges), 2):
        if edges[i + 1] - edges[i] > 0.05:
            if along_x:
                add_box(f"{name}_{i // 2}", edges[i], edges[i + 1], y0, y1, 0, CEILING, col, mat, "high")
            else:
                add_box(f"{name}_{i // 2}", x0, x1, edges[i], edges[i + 1], 0, CEILING, col, mat, "high")


def marker(name, x, y, facing_x):
    """An empty where the game puts something, its +x turned to point along `facing_x` (+1 or -1)."""
    empty = bpy.data.objects.new(name, None)
    empty.location = (x, y, 0.0)
    empty.rotation_euler.z = 0.0 if facing_x > 0 else math.pi
    bpy.context.scene.collection.objects.link(empty)


def carve_maze(rng, cols, rows, open_cells):
    """The maze's inner walls left standing, as ('v', i, j) - between cells (i, j) and (i + 1, j) -
    or ('h', i, j) - between (i, j) and (i, j + 1). Carved Kruskal's way, but in mirrored pairs -
    a wall taken out takes its mirror image through the middle with it - so that the maze looks
    the same from either end; the cells of `open_cells` are one room. Then a few more knocked
    through, also in pairs, for loops."""
    parent = {(i, j): (i, j) for i in range(cols) for j in range(rows)}

    def find(c):
        while parent[c] != c:
            parent[c] = parent[parent[c]]
            c = parent[c]
        return c

    def union(a, b):
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[ra] = rb
            return True
        return False

    def cells_of(w):
        kind, i, j = w
        return ((i, j), (i + 1, j)) if kind == "v" else ((i, j), (i, j + 1))

    def mirror(w):
        kind, i, j = w
        return (kind, cols - 2 - i, rows - 1 - j) if kind == "v" else (kind, cols - 1 - i, rows - 2 - j)

    walls = {("v", i, j) for i in range(cols - 1) for j in range(rows)} | \
            {("h", i, j) for i in range(cols) for j in range(rows - 1)}
    for w in list(walls):
        a, b = cells_of(w)
        if a in open_cells and b in open_cells:
            walls.discard(w)
            union(a, b)
    order = sorted(walls)
    rng.shuffle(order)
    for w in order:
        if w not in walls:
            continue
        m = mirror(w)
        joined = union(*cells_of(w))
        joined |= union(*cells_of(m))
        if joined:
            walls.discard(w)
            walls.discard(m)
    for _ in range(EXTRA_LOOPS // 2):
        w = rng.choice(sorted(walls))
        walls.discard(w)
        walls.discard(mirror(w))
    return walls


def main():
    seed, glb, out = get_args()
    rng = random.Random(seed)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    WALLS.clear()

    col_hall = collection("Hall")
    col_struct = collection("Structures")
    col_cover = collection("Cover")
    col_lights = collection("Lights")

    mat_ground = material("Ground", (0.30, 0.32, 0.28))
    mat_wall = material("Wall", (0.25, 0.25, 0.28))
    mat_maze = material("MazeWall", (0.32, 0.30, 0.34))
    mat_ceiling = material("Ceiling", (0.20, 0.20, 0.22))
    mat_floor = material("UpperFloor", (0.45, 0.45, 0.48))
    mat_stairs = material("Stairs", (0.50, 0.48, 0.42))
    mat_rail = material("Railing", (0.35, 0.35, 0.38))
    mat_low = material("CoverLow", (0.60, 0.45, 0.25))
    mat_high = material("CoverHigh", (0.40, 0.42, 0.45))
    mat_pillar = material("Pillar", (0.55, 0.55, 0.58))
    mat_glass = material("LightGlass", (1.0, 0.0, 1.0))  # the game's marker for fixture glass
    # the game's own team colours: blue's droids are cyan
    mat_team = {"red": material("TeamRed", (0.80, 0.07, 0.05)),
                "blue": material("TeamCyan", (0.00, 0.75, 0.85))}

    # The hall: floor top at exactly 0 (the game keeps a safety floor of its own just below).
    T = 1.0
    add_box("Ground", -HX - T, HX + T, -HY - T, HY + T, -0.2, 0, col_hall, mat_ground)
    for i, (x0, x1, y0, y1) in enumerate([(-HX - T, HX + T, HY, HY + T), (-HX - T, HX + T, -HY - T, -HY),
                                          (HX, HX + T, -HY, HY), (-HX - T, -HX, -HY, HY)]):
        add_box(f"Wall_{i}", x0, x1, y0, y1, 0, CEILING, col_hall, mat_wall, "high")
    add_box("Ceiling", -HX - T, HX + T, -HY - T, HY + T, CEILING, CEILING + 0.3, col_hall, mat_ceiling)

    # footprints on the ground that random cover must stay clear of: (x0, x1, y0, y1)
    keep_clear = []
    w = STAIR_WIDTH / 2
    door_back = (BALCONY_END - 4.0, BALCONY_END - 1.0)     # |x| span of each base's side door
    upper_door = (HX - BACK - 0.0, HX)                    # |x| span of the opening onto the catwalk

    for team, sx in (("red", -1), ("blue", 1)):
        mat = mat_team[team]
        facing = -sx                                        # toward the middle of the hall
        X = lambda a: sx * a                               # |x| on this side

        # --- the base: a walled room from the end wall to its front wall
        add_wall(f"{team}FrontWall", X(BASE_FRONT) - WALL / 2, X(BASE_FRONT) + WALL / 2, -BASE_SIDE, BASE_SIDE,
                 [(-2.0, 2.0)], False, col_struct, mat_wall)
        for side, sy in (("N", 1), ("S", -1)):
            y = sy * BASE_SIDE
            # its side wall: a door on the ground toward the front, and an opening on the upper
            # floor at the back, over the balcony, onto the bridge to the catwalk
            add_wall(f"{team}SideWall{side}", X(BASE_FRONT), X(HX), y - WALL / 2, y + WALL / 2,
                     [tuple(map(X, door_back)), tuple(map(X, upper_door))], True, col_struct, mat_wall)
            add_box(f"{team}SideLintel{side}", *map(X, upper_door), y - WALL / 2, y + WALL / 2, LINTEL, CEILING,
                    col_struct, mat_wall)
            # the doors' jambs, in the side's colour
            for a in door_back:
                add_box(f"{team}SideJamb{side}_{a}", X(a) - 0.1, X(a) + 0.1, y - WALL / 2 - 0.02, y + WALL / 2 + 0.02,
                        0, CEILING, col_struct, mat)
            # a band of the side's colour along it, inside and out
            for off in (-1, 1):
                yy = y + off * (WALL / 2 + 0.02)
                add_box(f"{team}SideBand{side}{off}", X(BASE_FRONT), X(door_back[0]), yy - 0.02, yy + 0.02, 2.4, 2.8,
                        col_struct, mat)
        for y in (-2.0, 2.0):  # the front door's jambs
            add_box(f"{team}FrontJamb{y}", X(BASE_FRONT) - WALL / 2 - 0.02, X(BASE_FRONT) + WALL / 2 + 0.02,
                    y - 0.1, y + 0.1, 0, CEILING, col_struct, mat)
        for off in (-1, 1):
            xx = X(BASE_FRONT) + off * (WALL / 2 + 0.02)
            for y0, y1 in ((-BASE_SIDE, -2.0), (2.0, BASE_SIDE)):
                add_box(f"{team}FrontBand{off}{y0}", xx - 0.02, xx + 0.02, y0, y1, 2.4, 2.8, col_struct, mat)
        keep_clear.append((*sorted((X(BASE_FRONT - 2.0), X(HX))), -BASE_SIDE - 1, BASE_SIDE + 1))

        # --- inside: the flag in a well, a balcony round three sides of it
        add_box(f"{team}BackBalcony", X(HX - BACK), X(HX), -BASE_SIDE, BASE_SIDE, 0, UPPER, col_struct, mat_floor)
        add_rail(f"{team}BackRail", -BASE_SIDE + SIDE_BALCONY, BASE_SIDE - SIDE_BALCONY, X(HX - BACK) - sx * 0.1,
                 False, [(-1.5, 1.5)], UPPER, col_struct, mat)
        add_box(f"{team}Band", X(HX - BACK), X(HX - BACK) - sx * 0.02, -BASE_SIDE + SIDE_BALCONY,
                BASE_SIDE - SIDE_BALCONY, 2.4, 2.8, col_struct, mat)
        for side, sy in (("N", 1), ("S", -1)):
            inner = sy * (BASE_SIDE - SIDE_BALCONY)
            add_box(f"{team}Balcony{side}", X(BALCONY_END), X(HX - BACK), inner, sy * BASE_SIDE, 0, UPPER,
                    col_struct, mat_floor)
            # stairs from the well up onto it, and its railings
            stair_x = X((BALCONY_END + HX - BACK) / 2)
            add_stairs(f"{team}WellStairs{side}", stair_x, inner, "-y" if sy > 0 else "+y", UPPER, col_struct,
                       mat_stairs)
            add_rail(f"{team}BalconyRail{side}", X(BALCONY_END), X(HX - BACK), inner - sy * 0.1, True,
                     [(stair_x - w, stair_x + w)], UPPER, col_struct, mat)
            add_rail(f"{team}BalconyEnd{side}", inner, sy * BASE_SIDE, X(BALCONY_END) - sx * 0.1, False, [],
                     UPPER, col_struct, mat)
            # out through the opening at the back, across the back corridor, onto the catwalk
            add_box(f"{team}Bridge{side}", *map(X, upper_door), sy * BASE_SIDE, sy * (HY - CATWALK), 0, UPPER,
                    col_struct, mat_floor)
            add_box(f"{team}Catwalk{side}", X(CATWALK_END), X(HX), sy * (HY - CATWALK), sy * HY, 0, UPPER,
                    col_struct, mat_floor)
            add_rail(f"{team}BridgeRail{side}", sy * (BASE_SIDE + WALL / 2), sy * (HY - CATWALK),
                     X(HX - BACK) - sx * 0.1, False, [], UPPER, col_struct, mat_rail)
            add_rail(f"{team}CatwalkRail{side}", X(CATWALK_END), X(HX - BACK), sy * (HY - CATWALK) - sy * 0.1,
                     True, [], UPPER, col_struct, mat_rail)
            # and down into the side lane at its front end
            keep_clear.append(add_stairs(f"{team}CatwalkStairs{side}", X(CATWALK_END), sy * (HY - CATWALK / 2),
                                         "+x" if sx < 0 else "-x", UPPER, col_struct, mat_stairs))
            keep_clear.append((*sorted((X(CATWALK_END), X(HX))), *sorted((sy * (HY - CATWALK - 1), sy * HY))))
            keep_clear.append((*sorted((X(HX - BACK - 1), X(HX))), *sorted((sy * BASE_SIDE, sy * HY))))
            # the side door's way in, kept clear
            keep_clear.append((*sorted(map(X, door_back)), *sorted((sy * (BASE_SIDE - 3), sy * (BASE_SIDE + 3)))))

        # the pad the flag stands on
        bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=2.0, depth=0.02, location=(X(FLAG_X), 0, 0.01))
        pad = bpy.context.active_object
        pad.name = f"{team}FlagPad"
        pad.data.materials.append(mat)
        move_to(pad, col_struct)

        # what the game puts here itself: the flag in its firewall, the computer that opens it
        # (against the back balcony, its screen out from it), and each droid's post: inside its
        # front door, in its yard, and at the maze's mouth
        marker(f"flag_{team}", X(FLAG_X), 0.0, facing)
        marker(f"computer_{team}", X(HX - BACK - 0.8), -6.0, facing)
        for n, (x, y) in enumerate(((BASE_FRONT + 1.5, -3.0), (BASE_FRONT - 5.0, 0.0), (MAZE_X + 1.5, -6.0)), start=1):
            marker(f"post_{team}_{n}", X(x), y, facing)
            keep_clear.append((X(x) - 1.0, X(x) + 1.0, y - 1.0, y + 1.0))

        # --- the lane walls from the maze back toward the base, each with a doorway into the
        # side lane, and pillars at the yard's corners
        for side, sy in (("N", 1), ("S", -1)):
            add_wall(f"{team}LaneWall{side}", X(MAZE_X), X(BASE_FRONT - 4.0), sy * LANE - WALL / 2, sy * LANE + WALL / 2,
                     [tuple(map(X, (26.0, 29.0)))], True, col_struct, mat_wall)
            add_hexagon(f"{team}Pillar{side}", X(BASE_FRONT - 4.0), sy * (LANE + 2.5), 1.0, col_cover, mat_pillar)
            keep_clear.append((X(BASE_FRONT - 4.0) - 2.0, X(BASE_FRONT - 4.0) + 2.0, sy * (LANE + 2.5) - 2.0,
                               sy * (LANE + 2.5) + 2.0))
            keep_clear.append((*sorted(map(X, (26.0, 29.0))), sy * LANE - 2.0, sy * LANE + 2.0))

    # --- the maze, between the lane walls; mirrored through the middle
    cols, rows = int(2 * MAZE_X / CELL), int(2 * LANE / CELL)
    x_at = lambda i: -MAZE_X + i * CELL
    y_at = lambda j: -LANE + j * CELL
    plaza = {(i, j) for i in range(cols // 2 - 2, cols // 2 + 2) for j in range(1, rows - 1)}
    inner = carve_maze(rng, cols, rows, plaza)
    for kind, i, j in sorted(inner):
        if kind == "v":
            x = x_at(i + 1)
            add_box(f"Maze_v{i}_{j}", x - WALL / 2, x + WALL / 2, y_at(j) - WALL / 2, y_at(j + 1) + WALL / 2, 0, CEILING,
                    col_struct, mat_maze, "high")
        else:
            y = y_at(j + 1)
            add_box(f"Maze_h{i}_{j}", x_at(i) - WALL / 2, x_at(i + 1) + WALL / 2, y - WALL / 2, y + WALL / 2, 0, CEILING,
                    col_struct, mat_maze, "high")
    # its outer walls: the lane walls along its sides, with doorways out to the side lanes, and
    # its ends, with doorways out to the yards - each mirrored through the middle
    lane_doors = sorted(rng.sample(range(1, cols // 2 - 1), 2))
    end_doors = sorted(rng.sample(range(rows), 2))
    for sy in (1, -1):
        cells = lane_doors if sy > 0 else [cols - 1 - c for c in lane_doors]
        doors = [(x_at(c) + 0.6, x_at(c + 1) - 0.6) for c in cells]
        add_wall(f"MazeSide{'N' if sy > 0 else 'S'}", -MAZE_X, MAZE_X, sy * LANE - WALL / 2, sy * LANE + WALL / 2,
                 doors, True, col_struct, mat_maze)
    for sx in (1, -1):
        cells = end_doors if sx > 0 else [rows - 1 - r for r in end_doors]
        doors = [(y_at(r) + 0.6, y_at(r + 1) - 0.6) for r in cells]
        add_wall(f"MazeEnd{'E' if sx > 0 else 'W'}", sx * MAZE_X - WALL / 2, sx * MAZE_X + WALL / 2, -LANE, LANE,
                 doors, False, col_struct, mat_maze)
    keep_clear.append((-MAZE_X - 1, MAZE_X + 1, -LANE - 1, LANE + 1))
    # the hub in the middle of the plaza, with stairs up from both ends
    h = HUB / 2
    add_box("Hub", -h, h, -h, h, 0, HUB_HEIGHT, col_struct, mat_floor)
    for i, (x, d) in enumerate(((-h, "-x"), (h, "+x"))):
        add_stairs(f"HubStairs_{i}", x, 0, d, HUB_HEIGHT, col_struct, mat_stairs)
    for i, (x0, x1, y0, y1) in enumerate([(-h, h, h - 0.2, h), (-h, h, -h, -h + 0.2)]):
        add_box(f"HubRail_{i}", x0, x1, y0, y1, HUB_HEIGHT, HUB_HEIGHT + RAIL, col_struct, mat_low, "low")

    # --- the side lanes' overlooks: a raised floor against the outer wall, stairs up both ends
    for side, sy in (("N", 1), ("S", -1)):
        o = OVERLOOK / 2
        y0, y1 = sy * (HY - 5.0), sy * HY
        add_box(f"Overlook{side}", -o, o, y0, y1, 0, OVERLOOK_HEIGHT, col_struct, mat_floor)
        for i, (x, d) in enumerate(((-o, "-x"), (o, "+x"))):
            keep_clear.append(add_stairs(f"OverlookStairs{side}_{i}", x, sy * (HY - 2.5), d, OVERLOOK_HEIGHT,
                                         col_struct, mat_stairs))
        add_rail(f"OverlookRail{side}", -o, o, y0 - sy * 0.1, True, [], OVERLOOK_HEIGHT, col_struct, mat_rail)
        keep_clear.append((-o - 3.5, o + 3.5, *sorted((sy * (HY - 6.0), sy * HY))))

    # --- random cover outside the maze on the blue half, mirrored through the middle onto red's
    placed = []

    def clear(x, y, r):
        if abs(y) > HY - r - 0.5 or abs(x) > HX - r - 0.5:
            return False
        for x0, x1, y0, y1 in keep_clear:
            dx, dy = max(x0 - x, 0, x - x1), max(y0 - y, 0, y - y1)
            if math.hypot(dx, dy) < r + MIN_GAP:
                return False
        for x0, x1, y0, y1 in WALLS:
            dx, dy = max(x0 - x, 0, x - x1), max(y0 - y, 0, y - y1)
            if math.hypot(dx, dy) < r + MIN_GAP:
                return False
        return all(math.hypot(x - px, y - py) >= r + pr + MIN_GAP for px, py, pr in placed)

    n_cover = 0
    for i in range(NUM_COVER):
        low = rng.random() < 0.6
        cw = rng.uniform(1.5, 3.5)
        cd = rng.uniform(0.6, 1.2) if low else rng.uniform(0.8, 1.6)
        ht = rng.uniform(0.9, 1.2) if low else rng.uniform(HIGH, HIGH + 0.6)
        r = math.hypot(cw, cd) / 2
        rot = rng.choice([0, math.pi / 2, rng.uniform(0, math.pi)])
        for _ in range(400):
            x, y = rng.uniform(MAZE_X - 6.0, BASE_FRONT + 2.0), rng.uniform(-HY + r, HY - r)
            if clear(x, y, r) and clear(-x, -y, r):
                placed += [(x, y, r), (-x, -y, r)]
                kind, mat = ("Low", mat_low) if low else ("High", mat_high)
                add_block(f"{kind}Cover_{i}_B", x, y, cw, cd, ht, rot, col_cover, mat, kind.lower())
                add_block(f"{kind}Cover_{i}_R", -x, -y, cw, cd, ht, rot, col_cover, mat, kind.lower())
                n_cover += 2
                break

    # --- ceiling light strips: a grid over the open ground, one in the middle of every maze
    # cell, none on a wall
    def lit(x, y, length):
        for x0, x1, y0, y1 in WALLS:
            if x0 - length / 2 - LIGHT_CLEAR < x < x1 + length / 2 + LIGHT_CLEAR and \
                    y0 - 0.25 - LIGHT_CLEAR < y < y1 + 0.25 + LIGHT_CLEAR:
                return False
        return abs(x) <= HX - length / 2 - LIGHT_CLEAR and abs(y) <= HY - 0.25 - LIGHT_CLEAR

    # (x, y, length): short ones in the maze's cells, long ones over the open ground
    spots = [(x_at(i) + CELL / 2, y_at(j) + CELL / 2, 2.0) for i in range(cols) for j in range(rows)
             if (i, j) not in plaza or (i + j) % 2 == 0]
    spots += [(x, y, 3.6) for x in (-43.0, -36.5, -30.0, -24.0, 24.0, 30.0, 36.5, 43.0) for y in (-21.0, -6.0, 6.0, 21.0)]
    spots += [(x, y, 3.6) for x in (-14.0, -6.0, 6.0, 14.0) for y in (-21.0, -14.0, 14.0, 21.0)]
    n_lights = 0
    for k, (x, y, length) in enumerate(spots):
        if lit(x, y, length):
            add_block(f"LightStrip_{k}", x, y, length, 0.5, 0.1, 0, col_lights, mat_glass, "none", z=CEILING - 0.1)
            n_lights += 1

    # light + camera, for looking at it in Blender
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 30), rotation=(math.radians(45), 0, math.radians(30)))
    bpy.context.active_object.data.energy = 3
    bpy.ops.object.camera_add(location=(0, -HY * 3.2, HX * 1.6), rotation=(math.radians(40), 0, 0))
    bpy.context.scene.camera = bpy.context.active_object
    world = bpy.data.worlds.new("World")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.55, 0.65, 0.8, 1)
    bpy.context.scene.world = world
    bpy.context.scene["map_seed"] = seed

    out = os.path.abspath(out) if out else os.path.join(os.path.dirname(os.path.abspath(__file__)), "hybrid_ctf_map.blend")
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print(f"[map] seed={seed} maze walls={len(inner)} cover={n_cover} lights={n_lights} -> {out}")
    if glb:
        export_glb(os.path.abspath(glb), [col_hall, col_struct, col_cover])


def join(col):
    """Joins a collection's meshes into one object (keeping their materials)."""
    objs = [o for o in col.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    objs[0].name = col.name
    # Its transform applied, so that its bounds are its own: a joined object keeps the first
    # one's, and a turned block's turn would make them too big for the game's survey.
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def export_glb(path, cols):
    """Exports for the game with the hall, structures and cover each joined into one mesh, and
    the markers as empties. Runs after the .blend is saved, so the saved file keeps every object
    separate."""
    for col in cols:
        join(col)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_extras=True,
                              export_cameras=False, export_lights=False)
    print(f"[map] exported -> {path}")


if __name__ == "__main__":
    main()
