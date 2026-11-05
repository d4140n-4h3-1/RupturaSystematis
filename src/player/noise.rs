//! What the player makes heard, for the droids hunting them to hear (see
//! [`crate::inhabitants`]): footfalls jogging, running and sprinting, landing from a fall, and the gun
//! - its shot where it is fired, heard furthest of all, and its bolt where it hits, which can draw a
//! droid away from the player. A drone's shots are heard too (see [`SHOT_NOISE`]). Walking, crouched
//! or crawling, the player makes no sound.
//!
//! Each noise is where it is made, how far off it carries, in meters along the corridors, and
//! whether it is sudden - a shot, a bolt hitting something - which no one mistakes for anything
//! else, or the sort a sneaking player makes, footfalls and landings, which only make those who
//! hear them more aware of them (see [`crate::inhabitants::Inhabitants::hear`]).

use super::{posture::Gait, posture::Posture, Player};
use fyrox::core::algebra::Vector3;

/// How far off footfalls carry, jogging, running and sprinting, in meters.
const JOG_NOISE: f32 = 5.0;
const RUN_NOISE: f32 = 7.0;
const SPRINT_NOISE: f32 = 10.0;
/// How far off landing from a fall carries, in meters, and how fast the body has to be falling,
/// in meters per second, for it to make any.
const LANDING_NOISE: f32 = 6.0;
const LOUD_LANDING: f32 = 3.0;
/// How far off a gunshot carries - the pistol's, the rifle's, a drone's - and a bolt hitting
/// something, in meters. A shot is the loudest thing in the maze: those searching for the player,
/// or wary of them still, hear it from well down the corridors.
pub const SHOT_NOISE: f32 = 30.0;
pub(super) const IMPACT_NOISE: f32 = 8.0;
/// How often footfalls are heard, in seconds: about a step's time.
const FOOTFALL: f32 = 0.3;
/// How fast the body has to be going, in meters per second, for its feet to be heard.
const HEARD_MOVING: f32 = 0.5;

#[derive(Debug, Default, Clone, PartialEq)]
pub(super) struct Noises {
    /// What has been made heard since the game last asked: where, how far off it carries, and
    /// whether it is sudden.
    made: Vec<(Vector3<f32>, f32, bool)>,
    /// How long until the next footfall is heard, in seconds.
    next_footfall: f32,
}

/// How far off footfalls carry at `gait` in `posture`: nowhere, but for jogging, running and
/// sprinting on their feet.
fn footfall(posture: Posture, gait: Gait) -> f32 {
    match (posture, gait) {
        (Posture::Standing, Gait::Sprinting) => SPRINT_NOISE,
        (Posture::Standing, Gait::Running) => RUN_NOISE,
        (Posture::Standing, Gait::Jogging) => JOG_NOISE,
        _ => 0.0,
    }
}

impl Player {
    /// Makes `loudness` heard at `at`, `sudden` or not.
    pub(super) fn make_noise(&mut self, at: Vector3<f32>, loudness: f32, sudden: bool) {
        if loudness > 0.0 {
            self.noises.made.push((at, loudness, sudden));
        }
    }

    /// The feet at `feet`, going `speed` along the ground at `gait`, for another `dt`: heard a
    /// step at a time, if they are loud enough to be.
    pub(super) fn footfalls(&mut self, feet: Vector3<f32>, speed: f32, gait: Gait, dt: f32) {
        self.noises.next_footfall -= dt;
        if !self.grounded || speed < HEARD_MOVING {
            return;
        }
        if self.noises.next_footfall <= 0.0 {
            self.noises.next_footfall = FOOTFALL;
            self.make_noise(feet, footfall(self.posture, gait), false);
        }
    }

    /// Landing at `feet`, falling as fast as the body was: heard if it was a hard landing.
    pub(super) fn thud(&mut self, feet: Vector3<f32>) {
        if self.fall_speed > LOUD_LANDING {
            self.make_noise(feet, LANDING_NOISE, false);
        }
    }

    /// What the player has made heard since this was last asked: where, how far off it carries,
    /// in meters, and whether it is sudden.
    pub fn noises(&mut self) -> Vec<(Vector3<f32>, f32, bool)> {
        std::mem::take(&mut self.noises.made)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_jogging_running_and_sprinting_on_their_feet_are_heard() {
        assert_eq!(footfall(Posture::Standing, Gait::Walking), 0.0);
        let (sprint, run, jog) = (
            footfall(Posture::Standing, Gait::Sprinting),
            footfall(Posture::Standing, Gait::Running),
            footfall(Posture::Standing, Gait::Jogging),
        );
        assert!(sprint > run && run > jog && jog > 0.0);
        assert_eq!(footfall(Posture::Crouching, Gait::Sprinting), 0.0);
        assert_eq!(footfall(Posture::Crawling, Gait::Jogging), 0.0);
    }

    #[test]
    fn a_shot_is_heard_from_further_down_the_corridors_than_anything_else() {
        use hydroxus_ai::{
            navigation::{grid::WalkGrid, navmesh::NavMesh},
            perception::hearing::Heard,
        };
        let loudest = [SPRINT_NOISE, LANDING_NOISE, IMPACT_NOISE].into_iter().fold(0.0, f32::max);
        assert!(SHOT_NOISE > 2.0 * loudest);
        // A corridor 40 m long and 4 m wide; the one listening down the middle of it, 25 m from
        // where the noise is made.
        let mut grid = WalkGrid::new(80, 8, 0.5);
        for x in 0..80 {
            for z in 0..8 {
                grid.set(x, z, true);
            }
        }
        let ground = &NavMesh::build(&grid, Vector3::zeros(), |_, _| 0);
        let (at, listener) = (Vector3::new(0.25, 0.0, 2.25), Vector3::new(25.25, 0.0, 2.25));
        assert!(Heard::at(ground, at, SHOT_NOISE).unwrap().by(ground, listener), "a shot");
        assert!(!Heard::at(ground, at, SPRINT_NOISE).unwrap().by(ground, listener), "a sprint");
    }

    #[test]
    fn a_step_is_heard_once_per_footfall() {
        let mut player = Player {
            grounded: true,
            ..Default::default()
        };
        let feet = Vector3::zeros();
        for _ in 0..30 {
            player.footfalls(feet, 3.0, Gait::Sprinting, 0.01);
        }
        assert_eq!(player.noises().len(), 1, "a third of a second, one step");
        player.footfalls(feet, 1.0, Gait::Walking, 1.0);
        assert!(player.noises().is_empty(), "walking is quiet");
    }
}
