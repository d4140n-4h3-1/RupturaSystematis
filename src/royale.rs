//! Battle royale: the player and up to [`MOST`] less one droids on one of [`crate::ctf::ROYALE_MAPS`],
//! every one against every other, each droid on a side of its own ([`Side::Lone`]).
//!
//! Each has [`LIVES`] lives. Shot down with a life left, it comes back [`BACK_IN`] seconds later
//! at a start inside the ring, as a new droid while the old one lies where it fell; down the
//! third time, it is out for the rest of the match. The last one in wins - the player, or not:
//! out, the player is told how they placed.
//!
//! A ring closes in on the map in stages ([`STAGES`]), each round a middle picked inside the
//! one before, so that it always ends somewhere new. Anyone outside it is hurt every
//! [`RING_HURTS_EVERY`] seconds, as by a bolt. It shows as a fence of glowing posts, tall enough to
//! see over the roofs.

use crate::ctf::Side;
use fyrox::{
    core::{
        algebra::{Matrix4, Vector3},
        color::Color,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{Material, MaterialResource},
    scene::{
        base::BaseBuilder,
        graph::Graph,
        mesh::{
            surface::{SurfaceBuilder, SurfaceData, SurfaceResource},
            MeshBuilder,
        },
        node::Node,
    },
};

/// How many play at most, the player included; and how many lives each has.
pub const MOST: usize = 16;
pub const LIVES: u32 = 3;
/// How long after going down someone with a life left comes back, in seconds.
pub const BACK_IN: f32 = 5.0;
/// The ring's stages: how long it waits, how long it then takes to close, in seconds, and how far
/// it reaches once it has, in meters.
pub const STAGES: [(f32, f32, f32); 5] = [
    (60.0, 45.0, 70.0),
    (40.0, 40.0, 42.0),
    (30.0, 30.0, 24.0),
    (25.0, 25.0, 12.0),
    (20.0, 20.0, 4.0),
];
/// How often anyone outside the ring is hurt, in seconds.
pub const RING_HURTS_EVERY: f32 = 2.0;
/// How long before the ring closes everyone is told so, in seconds.
const WARNING: f32 = 15.0;
/// The ring's posts: how many, how tall and how thick, in meters, and how they glow.
const POSTS: usize = 96;
const POST_HEIGHT: f32 = 24.0;
const POST_WIDTH: f32 = 0.2;
const POST_COLOUR: Color = Color::opaque(255, 60, 40);
const POST_GLOW: f32 = 3.0;

/// The ring everyone has to keep inside.
#[derive(Debug, Clone, PartialEq)]
pub struct Ring {
    /// Where it was and how far it reached as this stage began, and where it is closing to.
    from: (Vector3<f32>, f32),
    to: (Vector3<f32>, f32),
    /// Which stage it is in, and how long since that began, in seconds.
    stage: usize,
    time: f32,
}

/// What happened to the ring this frame, to tell everyone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RingNews {
    /// It closes in so many seconds.
    ClosesIn(f32),
    /// It is closing.
    Closing,
}

impl Ring {
    /// A ring round all of a map `reach` from its middle, `middle`, whose first stage closes to
    /// somewhere `pick` says: a share of the way, from 0 to 1, each time it is asked.
    pub fn new(middle: Vector3<f32>, reach: f32, pick: &mut impl FnMut() -> f32) -> Self {
        let from = (middle, reach);
        Self { from, to: next(from, STAGES[0].2, pick), stage: 0, time: 0.0 }
    }

    /// Where it is now: its middle, and how far it reaches.
    pub fn now(&self) -> (Vector3<f32>, f32) {
        let Some(&(wait, close, _)) = STAGES.get(self.stage) else {
            return self.to;
        };
        let t = ((self.time - wait) / close).clamp(0.0, 1.0);
        (self.from.0.lerp(&self.to.0, t), self.from.1 + (self.to.1 - self.from.1) * t)
    }

    /// Moves it on by `dt`, picking where each stage closes to by `pick`. What to tell everyone.
    pub fn update(&mut self, dt: f32, pick: &mut impl FnMut() -> f32) -> Option<RingNews> {
        let Some(&(wait, close, _)) = STAGES.get(self.stage) else {
            return None;
        };
        let before = self.time;
        self.time += dt;
        let crossed = |at: f32| before < at && self.time >= at;
        if crossed(wait - WARNING) {
            return Some(RingNews::ClosesIn(WARNING));
        }
        if crossed(wait) {
            return Some(RingNews::Closing);
        }
        if self.time >= wait + close {
            self.from = self.to;
            self.stage += 1;
            self.time = 0.0;
            if let Some(&(_, _, reach)) = STAGES.get(self.stage) {
                self.to = next(self.from, reach, pick);
            }
        }
        None
    }

    /// Whether `at` is inside it.
    pub fn holds(&self, at: Vector3<f32>) -> bool {
        let (middle, reach) = self.now();
        flat_distance(at - middle) <= reach
    }
}

/// Where the ring closes to from `from`, its middle and reach, to reach `reach`: a middle picked
/// so that all of the new ring is inside the old.
fn next(from: (Vector3<f32>, f32), reach: f32, pick: &mut impl FnMut() -> f32) -> (Vector3<f32>, f32) {
    let angle = std::f32::consts::TAU * pick();
    let off = (from.1 - reach).max(0.0) * pick().sqrt();
    (from.0 + Vector3::new(angle.cos() * off, 0.0, angle.sin() * off), reach)
}

fn flat_distance(v: Vector3<f32>) -> f32 {
    (v.x * v.x + v.z * v.z).sqrt()
}

/// Who is in a match: the player, as 0, and each droid's side, as its number.
pub type Who = u8;

/// A match of battle royale.
#[derive(Debug, Clone, PartialEq)]
pub struct Royale {
    pub ring: Ring,
    /// Where each start is, and which way it faces.
    pub starts: Vec<(Vector3<f32>, f32)>,
    /// How many lives each has left, the player first.
    lives: Vec<u32>,
    /// Whose droid is waiting to come back, and how long until it does, in seconds.
    pub coming_back: Vec<(Who, f32)>,
    /// How long until anyone outside the ring is hurt again, in seconds.
    hurt_in: f32,
    /// Who went out, in turn.
    pub out: Vec<Who>,
    /// Where the droids start the match, and which way they face.
    pub first_starts: Vec<(Vector3<f32>, f32)>,
    /// How long it has gone on, in seconds.
    pub time: f32,
}

impl Royale {
    /// A match among `players`, the player and the droids, starting at `starts`, on a map
    /// `reach` round from `middle`.
    pub fn new(
        players: usize,
        starts: Vec<(Vector3<f32>, f32)>,
        middle: Vector3<f32>,
        reach: f32,
        pick: &mut impl FnMut() -> f32,
    ) -> Self {
        Self {
            ring: Ring::new(middle, reach, pick),
            starts,
            lives: vec![LIVES; players],
            coming_back: Vec::new(),
            hurt_in: RING_HURTS_EVERY,
            out: Vec::new(),
            first_starts: Vec::new(),
            time: 0.0,
        }
    }

    /// The side the droid of `who` is on.
    pub fn side(who: Who) -> Side {
        Side::Lone(who)
    }

    /// How many are in the match, the player included.
    pub fn players(&self) -> usize {
        self.lives.len()
    }

    /// How many lives `who` has left.
    pub fn lives(&self, who: Who) -> u32 {
        self.lives.get(who as usize).copied().unwrap_or(0)
    }

    /// How many are still in, with a life left.
    pub fn still_in(&self) -> usize {
        self.lives.iter().filter(|&&lives| lives > 0).count()
    }

    /// `who` went down: one life less. Whether they have one left to come back with - and so,
    /// if they are a droid, are on their way back.
    pub fn went_down(&mut self, who: Who) -> bool {
        let Some(lives) = self.lives.get_mut(who as usize).filter(|lives| **lives > 0) else {
            return false;
        };
        *lives -= 1;
        if *lives == 0 {
            self.out.push(who);
            return false;
        }
        if who != 0 {
            self.coming_back.push((who, BACK_IN));
        }
        true
    }

    /// Whose droids come back now, after `dt` more of waiting.
    pub fn back_now(&mut self, dt: f32) -> Vec<Who> {
        let mut back = Vec::new();
        self.coming_back.retain_mut(|(who, left)| {
            *left -= dt;
            if *left <= 0.0 {
                back.push(*who);
                false
            } else {
                true
            }
        });
        back
    }

    /// Whether it is time to hurt anyone outside the ring, after `dt` more.
    pub fn hurts_now(&mut self, dt: f32) -> bool {
        self.hurt_in -= dt;
        if self.hurt_in <= 0.0 {
            self.hurt_in += RING_HURTS_EVERY;
            true
        } else {
            false
        }
    }

    /// Where someone coming back starts: the start inside the ring furthest from everyone in
    /// `others`, or failing any inside it, the one nearest the ring's middle.
    pub fn start_for(&self, others: &[Vector3<f32>]) -> Option<(Vector3<f32>, f32)> {
        let (middle, _) = self.ring.now();
        let alone = |at: Vector3<f32>| others.iter().map(|&o| flat_distance(o - at)).fold(f32::INFINITY, f32::min);
        let inside = self.starts.iter().copied().filter(|&(at, _)| self.ring.holds(at));
        inside
            .max_by(|a, b| alone(a.0).total_cmp(&alone(b.0)))
            .or_else(|| {
                self.starts
                    .iter()
                    .copied()
                    .min_by(|a, b| flat_distance(a.0 - middle).total_cmp(&flat_distance(b.0 - middle)))
            })
            .map(|(at, facing)| {
                // Facing the ring's middle.
                let to = middle - at;
                (at, if flat_distance(to) > 1.0 { to.x.atan2(to.z) } else { facing })
            })
    }
}

/// The ring's posts in the scene.
#[derive(Debug, Default, PartialEq)]
pub struct RingPosts {
    posts: Vec<Handle<Node>>,
}

impl RingPosts {
    /// Puts the posts into `graph`, hidden until they are placed.
    pub fn build(graph: &mut Graph) -> Self {
        let mut material = Material::standard();
        material.set_property("diffuseColor", POST_COLOUR);
        material.set_property("emissionStrength", Vector3::repeat(POST_GLOW));
        let material = MaterialResource::new_embedded(material);
        let surface = SurfaceResource::new_embedded(SurfaceData::make_cube(Matrix4::new_nonuniform_scaling(
            &Vector3::new(POST_WIDTH, POST_HEIGHT, POST_WIDTH),
        )));
        let posts = (0..POSTS)
            .map(|_| {
                MeshBuilder::new(BaseBuilder::new().with_visibility(false).with_cast_shadows(false))
                    .with_surfaces(vec![SurfaceBuilder::new(surface.clone()).with_material(material.clone()).build()])
                    .build(graph)
                    .to_base()
            })
            .collect();
        Self { posts }
    }

    /// Stands the posts round `ring`, its middle and reach, on the ground.
    pub fn place(&self, graph: &mut Graph, (middle, reach): (Vector3<f32>, f32)) {
        for (n, &post) in self.posts.iter().enumerate() {
            let angle = std::f32::consts::TAU * n as f32 / self.posts.len() as f32;
            let at = Vector3::new(middle.x + reach * angle.cos(), POST_HEIGHT / 2.0, middle.z + reach * angle.sin());
            if let Ok(node) = graph.try_get_mut(post) {
                node.set_visibility(true);
                node.local_transform_mut().set_position(at);
            }
        }
    }

    /// Hides them, out of battle royale.
    pub fn hide(&self, graph: &mut Graph) {
        for &post in &self.posts {
            if let Ok(node) = graph.try_get_mut(post) {
                node.set_visibility(false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picks() -> impl FnMut() -> f32 {
        let mut n = 0.0f32;
        move || {
            n = (n + 0.37).fract();
            n
        }
    }

    #[test]
    fn the_ring_waits_then_closes_each_stage_inside_the_last() {
        let mut pick = picks();
        let mut ring = Ring::new(Vector3::zeros(), 130.0, &mut pick);
        assert_eq!(ring.now(), (Vector3::zeros(), 130.0));
        let mut news = Vec::new();
        let dt = 0.1;
        let mut last = ring.now();
        for _ in 0..((STAGES.iter().map(|s| s.0 + s.1).sum::<f32>() + 5.0) / dt) as usize {
            if let Some(said) = ring.update(dt, &mut pick) {
                news.push(said);
            }
            let now = ring.now();
            // It never grows, nor jumps.
            assert!(now.1 <= last.1 + 1e-3);
            assert!((now.0 - last.0).norm() < 1.0);
            last = now;
        }
        assert_eq!(last.1, STAGES[STAGES.len() - 1].2, "closed all the way");
        assert_eq!(news.iter().filter(|n| matches!(n, RingNews::Closing)).count(), STAGES.len());
        assert_eq!(news.iter().filter(|n| matches!(n, RingNews::ClosesIn(_))).count(), STAGES.len());
    }

    #[test]
    fn three_lives_then_out() {
        let mut pick = picks();
        let mut royale = Royale::new(3, Vec::new(), Vector3::zeros(), 100.0, &mut pick);
        assert_eq!(royale.still_in(), 3);
        assert!(royale.went_down(2), "back again");
        assert!(royale.went_down(2), "back again");
        assert!(!royale.went_down(2), "out");
        assert!(!royale.went_down(2), "and stays out");
        assert_eq!(royale.out, vec![2]);
        assert_eq!(royale.still_in(), 2);
        // Back after a while, the two times it had a life left.
        assert!(royale.back_now(1.0).is_empty());
        assert_eq!(royale.back_now(BACK_IN), vec![2, 2]);
    }

    #[test]
    fn coming_back_starts_inside_the_ring_away_from_everyone() {
        let mut pick = picks();
        let starts = vec![
            (Vector3::new(10.0, 0.0, 0.0), 0.0),
            (Vector3::new(-10.0, 0.0, 0.0), 0.0),
            (Vector3::new(500.0, 0.0, 0.0), 0.0),
        ];
        let royale = Royale::new(2, starts, Vector3::zeros(), 100.0, &mut pick);
        let (at, _) = royale.start_for(&[Vector3::new(9.0, 0.0, 0.0)]).unwrap();
        assert_eq!(at, Vector3::new(-10.0, 0.0, 0.0));
    }
}
