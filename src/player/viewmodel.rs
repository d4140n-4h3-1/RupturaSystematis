//! The pistol in first person - and the rifle, held as it is - as in Call of Duty: a copy of the
//! droid's own pistol, carried
//! by the camera low and to the right, pointing a little in towards the middle of the view.
//! Holding the right mouse button brings it up into the middle to aim, with the screen on top of
//! it - crosshair and all - square in the middle of the view, as a sight.
//!
//! It comes up from below as the droid draws its pistol and goes back down as it holsters it,
//! at the same moments the droid's own pistol shows and goes. Each shot kicks it back and up,
//! and it settles again. Right up against a wall it is tucked down out of the way, rather than
//! pushed into it.
//!
//! Its parts do what the droid's own do: the ball at the muzzle tumbles, and the screen flashes
//! as the trigger is pulled. A shot leaves its muzzle, not the droid's, for wherever the middle
//! of the view is looking: see [`Player::aim_from_view`].

use super::{avatar::Avatar, Player};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
    },
    graph::SceneGraph,
    scene::{base::BaseBuilder, graph::Graph, node::Node, pivot::PivotBuilder},
};

/// How a gun is held in first person.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Hold {
    /// How big it is held, as a scale on the model's own gun.
    size: f32,
    /// Where it is held from the camera, in the camera's own terms - +z ahead, +y up, and -x to
    /// the right - at the hip.
    hip: Vector3<f32>,
    /// Raised to aim, the middle of its sight is on the middle of the view, this far ahead, in
    /// meters: the sight sits `screen_above` its grip's middle line and `screen_behind` its
    /// middle, in the model's own terms.
    screen_above: f32,
    screen_behind: f32,
    sight_distance: f32,
}

/// The pistol: 24 cm long, its screen on top (`SIGHT` in build_cyber_pistol.py).
const PISTOL: Hold = Hold {
    size: 0.1,
    hip: Vector3::new(-0.13, -0.17, 0.3),
    screen_above: 1.05,
    screen_behind: 0.95,
    sight_distance: 0.28,
};
/// The rifle: 74 cm long, held lower at the hip, and raised to look through its reflex sight
/// (`SIGHT` in build_cyber_rifle.py), its stock below the view, into the shoulder.
const RIFLE: Hold = Hold {
    size: 0.1,
    hip: Vector3::new(-0.12, -0.2, 0.22),
    screen_above: 1.29,
    screen_behind: 0.55,
    sight_distance: 0.3,
};
/// How far off the pistol at the hip points to cross the middle of the view, in meters.
const CONVERGE: f32 = 8.0;
/// How much lower and nearer it is put away, in meters, and how far its muzzle is dipped, in
/// radians.
const PUT_AWAY: Vector3<f32> = Vector3::new(0.0, -0.25, -0.1);
const PUT_AWAY_DIP: f32 = 50.0 * std::f32::consts::PI / 180.0;
/// How long it takes to come up or go down, in seconds.
const RAISE_TIME: f32 = 0.2;
/// How quickly it comes up to aim, or goes back to the hip, like
/// [`EYE_EASING`](super::posture::EYE_EASING).
const AIM_EASING: f32 = 14.0;
/// How far a shot kicks it back, in meters, and its muzzle up, in radians, and how quickly it
/// settles again, like a rate.
const KICK_BACK: f32 = 0.04;
const KICK_UP: f32 = 8.0 * std::f32::consts::PI / 180.0;
const KICK_SETTLE: f32 = 12.0;
/// How near a wall ahead has to be, in meters, for the pistol to start tucking down out of the
/// way, and how much nearer it has to be for it to be all the way down.
const TUCK_FROM: f32 = 0.5;
const TUCK_OVER: f32 = 0.25;
/// How quickly it tucks down and comes back up, like a rate.
const TUCK_EASING: f32 = 10.0;
/// How far out from its middle line it is, for the graphics effects, in meters.
const RADIUS: f32 = 0.1;
/// How quickly its barrel comes round onto whatever the middle of the view is on, like a rate;
/// and how far off that is taken to be with nothing there, in meters.
const ONTO_EASING: f32 = 15.0;
pub(super) const SIGHTED: f32 = 100.0;

/// The pistol held in first person, or the rifle.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Viewmodel {
    /// Whether it is the rifle, and how it is held.
    rifle: bool,
    hold: Hold,
    /// What the camera carries, turned and moved to hold the pistol.
    grip: Handle<Node>,
    /// The copy of the pistol in it, and its muzzle.
    pistol: Handle<Node>,
    muzzle: Handle<Node>,
    /// Every part of the droid's own pistol below the pistol itself, with the part of the copy
    /// that does what it does.
    parts: Vec<(Handle<Node>, Handle<Node>)>,
    /// How far up it is, from 0 put away to 1 held; how far in to aim, from 0 at the hip to 1
    /// raised; how much of a shot's kick is left, from 1 to 0; and how far it is tucked down out
    /// of a wall's way, from 0 to 1.
    up: f32,
    aim: f32,
    kick: f32,
    tuck: f32,
    /// Where its muzzle is in the grip, and the turn, about its sight, that brings the barrel
    /// round onto whatever the middle of the view is on, as far as it has come.
    muzzle_at: Vector3<f32>,
    onto: UnitQuaternion<f32>,
    /// Where its middle was, across the world, the last time the graphics effects were told.
    last_seen: Option<Vector3<f32>>,
}

/// The pistol's own turn in the grip: its barrel, along its +x, turned to the camera's ahead.
fn quarter_turn() -> UnitQuaternion<f32> {
    UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -std::f32::consts::FRAC_PI_2)
}

/// Where the middle of its screen is in the grip, held as `hold` has it, `at` and turned by
/// `turn`.
fn sight(hold: &Hold, at: Vector3<f32>, turn: UnitQuaternion<f32>) -> Vector3<f32> {
    at + turn * quarter_turn() * Vector3::new(-hold.screen_behind, hold.screen_above, 0.0).scale(hold.size)
}

/// The turn, about its sight, that brings the barrel of the pistol held `aim` of the way in, its
/// muzzle `muzzle_at` in the grip, round onto what the middle of the view is on, `sighted` ahead.
fn onto(hold: &Hold, aim: f32, muzzle_at: Vector3<f32>, sighted: f32) -> UnitQuaternion<f32> {
    let (at, turn) = held(hold, aim);
    let (sight, target) = (sight(hold, at, turn), Vector3::z() * sighted);
    // Turning it about the sight moves the muzzle a little too: a few goes settle it to within
    // a small part of a degree.
    let mut onto = UnitQuaternion::identity();
    for _ in 0..8 {
        let muzzle = sight + onto * (at + turn * muzzle_at - sight);
        let barrel = onto * turn * Vector3::z();
        onto = UnitQuaternion::rotation_between(&barrel, &(target - muzzle)).unwrap_or_default()
            * onto;
    }
    onto
}

/// Where the gun is held, as `hold` has it, and how it is turned, `aim` of the way in from the
/// hip to aiming.
fn held(hold: &Hold, aim: f32) -> (Vector3<f32>, UnitQuaternion<f32>) {
    let hip = hold.hip;
    let sights = Vector3::new(
        0.0,
        -hold.screen_above * hold.size,
        hold.sight_distance + hold.screen_behind * hold.size,
    );
    // At the hip it points in and up at the middle of the view, some way off.
    let hip_turn = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), (-hip.x).atan2(CONVERGE))
        * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), hip.y.atan2(CONVERGE));
    let at = hip + (sights - hip) * aim;
    let turn = hip_turn.slerp(&UnitQuaternion::identity(), aim);
    (at, turn)
}

impl Viewmodel {
    /// A copy of `avatar`'s pistol - or its rifle, if `rifle` - carried by `camera`, out of
    /// sight until it is drawn. None if the droid has no such gun.
    pub(super) fn new(graph: &mut Graph, avatar: &Avatar, camera: Handle<Node>, rifle: bool) -> Option<Self> {
        let hold = if rifle { RIFLE } else { PISTOL };
        let (original, original_muzzle) = avatar.gun_nodes(rifle)?;
        let (pistol, map) = graph.copy_node_inplace(original, &mut |_, _| true);
        let mut muzzle = original_muzzle;
        if !map.try_map(&mut muzzle) {
            return None;
        }
        let parts = graph
            .traverse_handle_iter(original)
            .skip(1)
            .filter_map(|part| {
                let mut copy = part;
                map.try_map(&mut copy).then_some((part, copy))
            })
            .collect();
        let grip = PivotBuilder::new(BaseBuilder::new().with_visibility(false))
            .build(graph)
            .to_base();
        graph.link_nodes(grip, camera);
        graph.link_nodes(pistol, grip);
        // Its barrel runs along its +x; turned a quarter turn round, that is the camera's ahead.
        let node = &mut graph[pistol];
        node.set_visibility(true);
        node.local_transform_mut()
            .set_position(Vector3::zeros())
            .set_rotation(quarter_turn())
            .set_scale(Vector3::repeat(hold.size));
        let muzzle_at = quarter_turn() * graph[muzzle].local_transform().position().scale(hold.size);
        Some(Self {
            rifle,
            hold,
            grip,
            pistol,
            muzzle,
            parts,
            up: 0.0,
            aim: 0.0,
            kick: 0.0,
            tuck: 0.0,
            muzzle_at,
            onto: UnitQuaternion::identity(),
            last_seen: None,
        })
    }

    /// Whether it can be seen.
    fn shown(&self, graph: &Graph) -> bool {
        graph[self.grip].visibility()
    }

    /// Holds it for another `dt`: `seen` while the view is from the droid's own eyes, up while
    /// the droid's pistol is `out`, `raised` to aim or at the hip, its barrel coming round onto
    /// what the middle of the view is on, `sighted` meters ahead, and tucked out of the way of a
    /// wall if that is near.
    fn hold(
        &mut self,
        graph: &mut Graph,
        seen: bool,
        out: bool,
        raised: bool,
        sighted: f32,
        dt: f32,
    ) {
        let room = sighted.min(TUCK_FROM);
        let step = dt / RAISE_TIME;
        self.up = (self.up + if out { step } else { -step }).clamp(0.0, 1.0);
        let ease = |rate: f32| 1.0 - (-rate * dt).exp();
        let aim = if raised && out { 1.0 } else { 0.0 };
        self.aim += (aim - self.aim) * ease(AIM_EASING);
        self.kick *= (-KICK_SETTLE * dt).exp();
        let tuck = ((TUCK_FROM - room) / TUCK_OVER).clamp(0.0, 1.0);
        self.tuck += (tuck - self.tuck) * ease(TUCK_EASING);

        let shown = seen && self.up > 0.0;
        if graph[self.grip].visibility() != shown {
            graph[self.grip].set_visibility(shown);
        }
        if !shown {
            self.last_seen = None;
            return;
        }
        // Its barrel onto what the middle of the view is on, turned about its sight, which stays
        // where it is: from the muzzle to there.
        self.onto = self
            .onto
            .slerp(&onto(&self.hold, self.aim, self.muzzle_at, sighted), ease(ONTO_EASING));
        self.place(graph);
        // Every part of it as the droid's own: the ball at the muzzle tumbling, and the screen
        // lit.
        for &(part, copy) in &self.parts {
            let (rotation, visible) = {
                let part = &graph[part];
                (**part.local_transform().rotation(), part.visibility())
            };
            let copy = &mut graph[copy];
            copy.local_transform_mut().set_rotation(rotation);
            if copy.visibility() != visible {
                copy.set_visibility(visible);
            }
        }
    }

    /// Kicks it back and up, for a shot that has just left it.
    fn kick(&mut self, graph: &mut Graph) {
        self.kick = 1.0;
        if self.shown(graph) {
            self.place(graph);
        }
    }

    /// Puts it where it is held, as far up, in, tucked and kicked as it is.
    fn place(&self, graph: &mut Graph) {
        let (at, turn) = held(&self.hold, self.aim);
        let sight = sight(&self.hold, at, turn);
        let (at, turn) = (sight + self.onto * (at - sight), self.onto * turn);
        // Put away, or tucked down out of a wall's way: lower, nearer, and dipped. Smoothed at
        // both ends, so it eases up and down.
        let away = 1.0 - self.up.min(1.0 - self.tuck);
        let away = away * away * (3.0 - 2.0 * away);
        let dip = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PUT_AWAY_DIP * away);
        let kick = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -KICK_UP * self.kick);
        let transform = graph[self.grip].local_transform_mut();
        transform
            .set_position(at + PUT_AWAY * away + Vector3::new(0.0, 0.0, -KICK_BACK * self.kick))
            .set_rotation(turn * dip * kick);
    }
}

impl Player {
    /// Makes the pistol and the rifle to hold in first person, from the droid's.
    pub(super) fn make_viewmodel(&mut self, graph: &mut Graph) {
        let camera = self.camera.transmute();
        self.viewmodels = match self.avatar.as_ref() {
            Some(avatar) => [false, true]
                .into_iter()
                .filter_map(|rifle| Viewmodel::new(graph, avatar, camera, rifle))
                .collect(),
            None => Vec::new(),
        };
    }

    /// The gun held in first person that can be seen, if one can.
    fn shown_viewmodel(&self, graph: &Graph) -> Option<&Viewmodel> {
        self.viewmodels.iter().find(|v| v.shown(graph))
    }

    /// The copy held in first person that can be seen, and its muzzle, if one can.
    pub(super) fn viewmodel_nodes(&self, graph: &Graph) -> Option<(Handle<Node>, Handle<Node>)> {
        self.shown_viewmodel(graph).map(|v| (v.pistol, v.muzzle))
    }

    /// Holds the pistol and the rifle in first person for another `dt`.
    pub(super) fn hold_pistol(&mut self, graph: &mut Graph, dt: f32) {
        let Some(avatar) = self.avatar.as_ref() else {
            return;
        };
        let camera = &graph[self.camera];
        let (eye, ahead) = (camera.global_position(), camera.look_vector());
        let sighted = self.distance_to_hit(graph, eye, ahead.normalize(), SIGHTED);
        let (seen, raised) = (self.in_own_eyes, self.keys.strafe);
        for viewmodel in &mut self.viewmodels {
            let out = avatar.gun_out(graph, viewmodel.rifle);
            viewmodel.hold(graph, seen, out, raised, sighted, dt);
        }
    }

    /// Kicks the gun held in first person, for a shot that has just left it.
    pub(super) fn kick_pistol(&mut self, graph: &mut Graph) {
        for viewmodel in &mut self.viewmodels {
            if viewmodel.shown(graph) {
                viewmodel.kick(graph);
            }
        }
    }

    /// Where a shot leaves, and which way it goes, one meter long, while a gun is held in first
    /// person: from its muzzle, for whatever the middle of the view is looking at. None while
    /// none is.
    pub(super) fn aim_from_view(&self, graph: &Graph) -> Option<(Vector3<f32>, Vector3<f32>)> {
        let viewmodel = self.shown_viewmodel(graph)?;
        let camera = &graph[self.camera];
        let (eye, ahead) = (camera.global_position(), camera.look_vector().normalize());
        let target = eye + ahead * self.distance_to_hit(graph, eye, ahead, SIGHTED);
        let muzzle = graph[viewmodel.muzzle].global_position();
        let way = (target - muzzle).try_normalize(1.0e-4).unwrap_or(ahead);
        Some((muzzle, way))
    }

    /// The gun held in first person, while it can be seen, for the graphics effects: a capsule
    /// round it, and how far it has moved since the last time this was asked.
    pub(super) fn viewmodel_moving(&mut self, graph: &Graph) -> Option<fyrox_gfx::MovingThing> {
        let viewmodel = self.viewmodels.iter_mut().find(|v| v.shown(graph))?;
        let (back, front) = (
            graph[viewmodel.pistol].global_position(),
            graph[viewmodel.muzzle].global_position(),
        );
        let middle = (back + front) * 0.5;
        let moved = viewmodel
            .last_seen
            .map_or(Vector3::zeros(), |last| middle - last);
        viewmodel.last_seen = Some(middle);
        Some(fyrox_gfx::MovingThing {
            bottom: back,
            top: front,
            radius: RADIUS,
            moved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_the_hip_it_is_low_and_right_and_points_in_at_the_middle() {
        let (at, turn) = held(&PISTOL, 0.0);
        assert!(
            at.x < 0.0 && at.y < 0.0,
            "low, and to the right (-x): {at:?}"
        );
        let ahead = turn * Vector3::z();
        let crosses = at + ahead * (CONVERGE / ahead.z);
        assert!(
            crosses.xy().norm() < 0.02,
            "crosses the middle some way off: {crosses:?}"
        );
    }

    #[test]
    fn raised_its_screen_is_in_the_middle_of_the_view() {
        let (at, turn) = held(&PISTOL, 1.0);
        // The screen's middle, in the pistol's own terms, turned the way the pistol is held in
        // the grip: its +x ahead.
        let quarter = -std::f32::consts::FRAC_PI_2;
        let to_screen = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), quarter)
            * Vector3::new(-PISTOL.screen_behind, PISTOL.screen_above, 0.0)
            * PISTOL.size;
        let screen = at + turn * to_screen;
        assert!(screen.xy().norm() < 1.0e-4, "{screen:?}");
        assert!((screen.z - PISTOL.sight_distance).abs() < 1.0e-4);
    }

    #[test]
    fn its_barrel_comes_round_onto_what_the_middle_of_the_view_is_on() {
        // The model's muzzle: ahead of the grip and above it.
        let muzzle_at = quarter_turn() * Vector3::new(0.96, 0.64, 0.0).scale(PISTOL.size);
        for aim in [0.0, 0.5, 1.0] {
            for sighted in [1.0, 3.0, SIGHTED] {
                let (at, turn) = held(&PISTOL, aim);
                let sight = sight(&PISTOL, at, turn);
                let onto = onto(&PISTOL, aim, muzzle_at, sighted);
                let (at, turn) = (sight + onto * (at - sight), onto * turn);
                let muzzle = at + turn * muzzle_at;
                let to = (Vector3::z() * sighted - muzzle).normalize();
                assert!(
                    (turn * Vector3::z() - to).norm() < 1.0e-3,
                    "aim {aim}, {sighted} m"
                );
                assert!(
                    (self::sight(&PISTOL, at, turn) - sight).norm() < 1.0e-5,
                    "the sight stays put"
                );
            }
        }
    }
}
