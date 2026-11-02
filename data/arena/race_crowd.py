"""
Crowds for the race tracks (race_circuit.py, race_lumen.py): spectators on the grandstands, built
into the track - simple figures in the droids' style, a head with a lit visor, a body, legs and
arms, some with their arms down, some with both up, some waving one, in the droids' five colours
(glowing them, at night). Thousands of them: each a few boxes, rather than a droid with its
skeleton and its fifty meshes, which would cost as much as a city's crowd each.

Each stand is cut along its length into sections CHUNK long, every section's spectators one mesh,
`Crowd_<stand>_<section>`, its origin in its middle: the game (src/race.rs, Crowd) bobs each
section up and down, idly, and jumping as the bikes come by.
"""

import math
import random

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


class Crowd:
    def __init__(self, rc, night, seed=5):
        self.rc = rc
        if night:
            self.bodies = [rc.glow(f"Crowd{name}", rgb, NIGHT_GLOW) for name, rgb in COLOURS.items()]
        else:
            self.bodies = [rc.material(f"Crowd{name}", rgb, roughness=0.5, metallic=0.1) for name, rgb in COLOURS.items()]
        self.joints = rc.material("CrowdJoints", (0.05, 0.05, 0.06), roughness=0.5)
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
