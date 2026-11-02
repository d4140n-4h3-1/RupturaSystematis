"""
Crowds for the race tracks (race_circuit.py, race_lumen.py): spectators on the grandstands, built
into the track - simple figures in the droids' style, a head with a lit visor, a body, legs and
arms, some with their arms down, some with both up, some waving one, in the droids' five colours
(glowing them, at night). Thousands of them: each a few boxes, rather than a droid with its
skeleton and its fifty meshes, which would cost as much as a city's crowd each.

Each stand is cut along its length into sections CHUNK long, every section's spectators one mesh,
`Crowd_<stand>_<section>`, its origin in its middle: the game (src/race.rs, Crowd) bobs each
section up and down, idly, and jumping as the bikes come by.

Over each stand, a giant monitor (`monitor`): a 16:9 screen in a lit frame up on two masts,
facing the road and turned a little towards the bikes coming, for the crowd across the road and
the riders to see. The screen is a mesh of its own, `Monitor_<n>`, its picture's corners at its
own; the game shows the race on it, live, from a camera of its own (src/race.rs, Broadcast).
"""

import math
import random

import bmesh
import bpy
from mathutils import Vector

CHUNK = 6.0             # how long a section of a stand is, that moves as one
SPACING = 0.95          # how far apart spectators stand along a row
EMPTY = 0.12            # how many places along a row are empty
HEIGHT = 1.2            # how tall a spectator is
COLOURS = {
    "Cyan": (0.0, 0.85, 1.0),
    "Red": (1.0, 0.1, 0.05),
    "Orange": (1.0, 0.45, 0.05),
    "Green": (0.15, 1.0, 0.3),
    "Blue": (0.15, 0.3, 1.0),
}
NIGHT_GLOW = 0.25       # how brightly they glow their colours at night, as the droids do
MONITOR_TURN = 20.0     # how far a monitor is turned from square on to the road, towards the bikes coming, in degrees
MONITOR_TILT = 8.0      # and how far it leans down


class Crowd:
    def __init__(self, rc, night, seed=5):
        self.rc = rc
        if night:
            self.bodies = [rc.glow(f"Crowd{name}", rgb, NIGHT_GLOW) for name, rgb in COLOURS.items()]
        else:
            self.bodies = [rc.material(f"Crowd{name}", rgb, roughness=0.5, metallic=0.1) for name, rgb in COLOURS.items()]
        self.joints = rc.material("CrowdJoints", (0.05, 0.05, 0.06), roughness=0.5)
        self.frame = rc.material("MonitorFrame", (0.03, 0.035, 0.045), roughness=0.4, metallic=0.5)
        self.trim = rc.glow("MonitorTrim", (0.0, 0.85, 1.0), 3.0)
        self.screen = rc.glow("MonitorScreen", (1.0, 1.0, 1.0), 1.0)
        self.monitors = 0
        self.visor = rc.glow("CrowdVisor", (0.9, 0.95, 1.0), 3.0)
        self.rng = random.Random(seed)
        self.stands = 0
        self.figures = 0
        self.objs = []

    def figure(self, b, foot, face, side):
        """A spectator standing at `foot`, facing `face`, `side` to its right-hand side."""
        rng = self.rng
        up = Vector((0, 0, 1))
        sc = HEIGHT / 1.2 * rng.uniform(0.92, 1.08)
        body = rng.choice(self.bodies)

        def box(centre, half, m, along=side, across=face):
            b.oriented_box(foot + centre * sc, along, across, along.cross(across) if along is not side else up,
                           tuple(h * sc for h in half), m)

        for s in (-1, 1):
            box(side * (0.08 * s) + up * 0.25, (0.06, 0.07, 0.25), self.joints)
        box(up * 0.72, (0.19, 0.11, 0.22), body)
        box(up * 1.07, (0.13, 0.12, 0.12), body)
        box(up * 1.09 + face * 0.125, (0.1, 0.012, 0.03), self.visor)
        # Arms: down by its sides, both up, or one up waving.
        pose = rng.choices(("down", "up", "wave"), weights=(3, 4, 3))[0]
        for s in (-1, 1):
            raised = pose == "up" or (pose == "wave" and s == 1)
            shoulder = side * (0.24 * s) + up * 0.9
            if raised:
                way = (side * (0.35 * s) + up).normalized()
                box(shoulder + way * 0.22, (0.22, 0.05, 0.05), body, along=way, across=face)
            else:
                box(side * (0.25 * s) + up * 0.7, (0.05, 0.05, 0.2), body)
        self.figures += 1

    def monitor(self, centre, facing, along, width):
        """A giant monitor `width` across, 16:9, its screen's middle at `centre`: facing the way
        `facing` is, flat, turned towards `-along` (the way the bikes come from) by MONITOR_TURN and
        tilted down by MONITOR_TILT; on two masts down to the ground."""
        rc = self.rc
        up = Vector((0, 0, 1))
        turn, tilt = math.radians(MONITOR_TURN), math.radians(MONITOR_TILT)
        n = (facing * math.cos(turn) - along * math.sin(turn)).normalized()
        n = (n * math.cos(tilt) - up * math.sin(tilt)).normalized()
        f = -n                                  # the way someone looking at it looks
        right = f.cross(up).normalized()
        upward = right.cross(f).normalized()
        height = width * 9 / 16
        name = f"Monitor_{self.monitors}"
        self.monitors += 1
        # The screen: its picture's left at the left, its top at the top, as one looking at it sees.
        bm = bmesh.new()
        uv = bm.loops.layers.uv.new("UVMap")
        corners = [(-1, -1, 0.0, 0.0), (1, -1, 1.0, 0.0), (1, 1, 1.0, 1.0), (-1, 1, 0.0, 1.0)]
        face = bm.faces.new([bm.verts.new(n * 0.05 + right * (sx * width / 2) + upward * (sy * height / 2)) for sx, sy, _, _ in corners])
        for loop, (_, _, u, v) in zip(face.loops, corners):
            loop[uv].uv = (u, v)
        face.normal_update()
        if face.normal.dot(n) < 0:
            face.normal_flip()
        mesh = bpy.data.meshes.new(name)
        bm.to_mesh(mesh)
        bm.free()
        mesh.materials.append(self.screen)
        obj = bpy.data.objects.new(name, mesh)
        obj.location = centre
        bpy.context.scene.collection.objects.link(obj)
        self.objs.append(obj)
        # Its frame, lit round the screen, and the masts it stands on.
        b = rc.Builder(f"MonitorFrame_{self.monitors - 1}")
        b.oriented_box(centre - n * 0.4, right, n, upward, (width / 2 + 0.8, 0.4, height / 2 + 0.8), self.frame)
        for a, c in (((-1, -1), (1, -1)), ((1, -1), (1, 1)), ((1, 1), (-1, 1)), ((-1, 1), (-1, -1))):
            p = centre + n * 0.06 + right * (a[0] * (width / 2 + 0.3)) + upward * (a[1] * (height / 2 + 0.3))
            q = centre + n * 0.06 + right * (c[0] * (width / 2 + 0.3)) + upward * (c[1] * (height / 2 + 0.3))
            d = q - p
            b.oriented_box((p + q) / 2, d.normalized(), n, d.normalized().cross(n), (d.length / 2 + 0.15, 0.06, 0.15), self.trim)
        bottom = centre.z - height / 2 - 0.8
        for side in (-1, 1):
            foot = centre - n * 0.9 + right * (side * width / 3)
            b.oriented_box(Vector((foot.x, foot.y, bottom / 2)), right, Vector((n.x, n.y, 0)).normalized(), up,
                           (0.5, 0.5, bottom / 2 + 0.3), self.frame)
        self.objs.append(b.finish())

    def stand(self, middle, along, out, length, rows):
        """Spectators on a stand: `middle` the middle of its front at the ground, `along` along it,
        `out` away from the road, and each row (front, back, top) - how far out its front and its
        back are, and how high its top is. They face the road."""
        rc = self.rc
        stand = self.stands
        self.stands += 1
        up = Vector((0, 0, 1))
        face = -out
        side = face.cross(up).normalized()     # its right hand, facing the road
        chunks = max(1, math.ceil(length / CHUNK))
        deep = rows[-1][1]
        for c in range(chunks):
            a0 = -length / 2 + c * length / chunks
            a1 = a0 + length / chunks
            centre = middle + along * ((a0 + a1) / 2) + out * (deep / 2) + up * (rows[-1][2] / 2)
            b = rc.Builder(f"Crowd_{stand}_{c}")
            for front, back, top in rows:
                a = a0 + SPACING / 2
                while a < a1 - 0.2:
                    if self.rng.random() > EMPTY:
                        jitter = self.rng.uniform(-0.12, 0.12)
                        foot = middle + along * (a + jitter) + out * (front + (back - front) * 0.45) + up * top
                        self.figure(b, foot - centre, face, side)
                    a += SPACING
            if not b.bm.faces:
                b.bm.free()
                continue
            obj = b.finish()
            obj.location = centre
            self.objs.append(obj)
        return self.stands - 1
