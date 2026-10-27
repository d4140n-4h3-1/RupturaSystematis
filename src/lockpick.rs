//! Picking a firewall's lock: a phase lock, rings of light round a core, each turning on its own.
//!
//! Each ring has a port in it - a gap in its light - and the lock has a slot at the top. Pick a
//! ring with W and S, and turn it round with A and D - held, it stops drifting - and once its
//! port is in the slot, where the ring lights up, latch it with Space: it stops there for good. Latch one out of line and the trace
//! jumps on. When every ring is latched, the firewall drops; if the trace runs out first, the
//! lock throws the pick out and has to be started again. Tab gives up.
//!
//! The inner rings turn slowest; each further out turns faster, the other way round from the one
//! inside it, so the outer rings are the hard ones.

use fyrox::{
    core::{algebra::Vector2, color::Color, pool::Handle},
    graph::SceneGraph,
    gui::{
        brush::Brush,
        text::{TextBuilder, TextMessage},
        screen::ScreenBuilder,
        vector_image::{Primitive, VectorImage, VectorImageBuilder},
        widget::{WidgetBuilder, WidgetMessage},
        HorizontalAlignment, Thickness, UiNode, UserInterface, VerticalAlignment,
    },
};
use hydroxus_ai::grid::Rng;
use std::f32::consts::{PI, TAU};

/// How near the slot a port has to be to latch, in radians either side.
const LATCH_WITHIN: f32 = 0.26;
/// How fast the player turns a ring, in radians a second.
const NUDGE: f32 = 1.8;
/// How fast the innermost ring drifts, in radians a second, and how much faster each further out.
const DRIFT: f32 = 0.25;
const DRIFT_STEP: f32 = 0.15;
/// How much of the trace a latch out of line spends, in seconds.
const MISLATCH: f32 = 3.0;

/// A lock to pick: how many rings it has, and how long the trace gives, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lock {
    pub rings: usize,
    pub trace: f32,
}

/// A cell's firewall: three rings.
pub const CELL_LOCK: Lock = Lock { rings: 3, trace: 45.0 };
/// The cell block's gate: four.
pub const GATE_LOCK: Lock = Lock { rings: 4, trace: 50.0 };

/// One ring of the lock.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ring {
    /// Where its port is, in radians round from the slot at the top, clockwise.
    port: f32,
    /// How fast it drifts, in radians a second, clockwise positive.
    drift: f32,
    latched: bool,
}

/// How picking a lock is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Picking {
    Going,
    Opened,
    Thrown,
}

/// A phase lock being picked.
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseLock {
    rings: Vec<Ring>,
    /// The ring picked, counted from the inside.
    pub picked: usize,
    /// How long the trace has left, in seconds, and how long it gave.
    pub trace: f32,
    pub trace_full: f32,
}

/// `angle` brought into -π to π.
fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

impl PhaseLock {
    /// A fresh lock of `lock`'s kind, its ports scattered by `rng`, none of them near the slot.
    pub fn new(lock: Lock, rng: &mut Rng) -> Self {
        let rings = (0..lock.rings)
            .map(|k| {
                let way = if k % 2 == 0 { 1.0 } else { -1.0 };
                let port = PI / 3.0 + (rng.below(1000) as f32 / 1000.0) * (TAU - 2.0 * PI / 3.0);
                Ring { port, drift: way * (DRIFT + DRIFT_STEP * k as f32), latched: false }
            })
            .collect();
        Self { rings, picked: 0, trace: lock.trace, trace_full: lock.trace }
    }

    /// Picks the ring further out (`step` 1) or further in (-1), latched or not.
    pub fn pick(&mut self, step: i32) {
        let last = self.rings.len() as i32 - 1;
        self.picked = (self.picked as i32 + step).clamp(0, last) as usize;
    }

    /// Latches the picked ring where it is: in line, it stops there; out of line, the trace jumps
    /// on and it keeps turning.
    pub fn latch(&mut self) {
        let Some(ring) = self.rings.get_mut(self.picked) else {
            return;
        };
        if ring.latched {
            return;
        }
        if wrap(ring.port).abs() <= LATCH_WITHIN {
            ring.latched = true;
            ring.port = 0.0;
            // On to the next one not yet latched, outward first.
            if let Some(next) = (0..self.rings.len()).map(|k| (self.picked + k) % self.rings.len()).find(|&k| !self.rings[k].latched) {
                self.picked = next;
            }
        } else {
            self.trace -= MISLATCH;
        }
    }

    /// Moves it on `dt` seconds, the picked ring turned `turn` - -1 anticlockwise, 1 clockwise,
    /// 0 not at all, when it drifts as the rest do: how it is going.
    pub fn update(&mut self, turn: f32, dt: f32) -> Picking {
        if self.rings.iter().all(|ring| ring.latched) {
            return Picking::Opened;
        }
        self.trace -= dt;
        if self.trace <= 0.0 {
            return Picking::Thrown;
        }
        for (k, ring) in self.rings.iter_mut().enumerate() {
            if ring.latched {
                continue;
            }
            // Held, it goes only as it is turned.
            let speed = if k == self.picked && turn != 0.0 { turn * NUDGE } else { ring.drift };
            ring.port = wrap(ring.port + speed * dt);
        }
        Picking::Going
    }

    /// Whether the picked ring's port is in the slot, to be latched.
    pub fn in_line(&self) -> bool {
        self.rings.get(self.picked).is_some_and(|ring| !ring.latched && wrap(ring.port).abs() <= LATCH_WITHIN)
    }

    /// How many rings it has.
    pub fn len(&self) -> usize {
        self.rings.len()
    }
}

/// The lock on screen: its rings in layers of one colour each, as a vector image draws in one -
/// the rings dim, the picked one and the slot bright, the latched ones green - and the trace bar
/// and what the keys do under it.
#[derive(Debug, Default, PartialEq)]
pub struct LockScreen {
    /// The screen it is all on, shown or hidden; its layers, the backing first; and the help.
    screen: Handle<UiNode>,
    layers: [Handle<VectorImage>; 5],
    help: Handle<UiNode>,
}

/// How big the lock is on screen: the innermost ring's radius and how far apart they are, in
/// pixels, and how thick each is.
const INNER: f32 = 70.0;
const APART: f32 = 42.0;
const THICK: f32 = 9.0;
/// How wide the port is, in radians.
const PORT: f32 = 0.42;
const SIZE: f32 = 520.0;
const DIM: Color = Color::opaque(150, 60, 30);
const BRIGHT: Color = Color::opaque(255, 120, 40);
const LATCHED: Color = Color::opaque(90, 240, 140);
const SLOT: Color = Color::opaque(255, 230, 120);
const BACKING: Color = Color::from_rgba(0, 0, 0, 190);

impl LockScreen {
    /// The lock's screen, in `ui`, hidden: in the middle of the window, the help along the top
    /// under the status line.
    pub fn build(ui: &mut UserInterface) -> Self {
        let ctx = &mut ui.build_ctx();
        let layer = |ctx: &mut fyrox::gui::BuildContext, colour: Color| {
            VectorImageBuilder::new(
                WidgetBuilder::new()
                    .with_width(SIZE)
                    .with_height(SIZE)
                    .with_horizontal_alignment(HorizontalAlignment::Center)
                    .with_vertical_alignment(VerticalAlignment::Center)
                    .with_foreground(Brush::Solid(colour).into()),
            )
            .build(ctx)
        };
        let layers = [layer(ctx, BACKING), layer(ctx, DIM), layer(ctx, BRIGHT), layer(ctx, LATCHED), layer(ctx, SLOT)];
        let help = TextBuilder::new(
            WidgetBuilder::new()
                .with_horizontal_alignment(HorizontalAlignment::Center)
                .with_vertical_alignment(VerticalAlignment::Top)
                .with_margin(Thickness { left: 0.0, top: 60.0, right: 0.0, bottom: 0.0 })
                .with_foreground(Brush::Solid(SLOT).into()),
        )
        .with_font_size(18.0.into())
        .build(ctx)
        .to_base();
        let mut screen = WidgetBuilder::new().with_visibility(false).with_child(help);
        for layer in layers {
            screen = screen.with_child(layer.to_base::<UiNode>());
        }
        let screen = ScreenBuilder::new(screen).build(ctx).to_base();
        Self { screen, layers, help }
    }

    /// Shows it, or hides it.
    pub fn show(&self, ui: &UserInterface, shown: bool) {
        ui.send(self.screen, WidgetMessage::Visibility(shown));
    }

    /// Draws `lock` as it is now.
    pub fn draw(&self, ui: &mut UserInterface, lock: &PhaseLock) {
        let middle = Vector2::new(SIZE / 2.0, SIZE / 2.0);
        // Round from the top, clockwise, as the lock reckons it; the screen's y runs down.
        let at = |angle: f32, radius: f32| middle + Vector2::new(angle.sin(), -angle.cos()) * radius;
        let arc = |out: &mut Vec<Primitive>, from: f32, to: f32, radius: f32, thick: f32| {
            let steps = ((to - from).abs() / 0.08).ceil().max(1.0) as usize;
            for k in 0..steps {
                let (a, b) = (from + (to - from) * k as f32 / steps as f32, from + (to - from) * (k + 1) as f32 / steps as f32);
                out.push(Primitive::Line { begin: at(a, radius), end: at(b, radius), thickness: thick });
            }
        };
        let mut layers: [Vec<Primitive>; 4] = Default::default();
        let outer = INNER + APART * (lock.len() as f32 - 1.0) + THICK * 2.5;
        let backing = vec![Primitive::Circle { center: middle, radius: outer + 30.0, segments: 64 }];
        // The core.
        layers[1].push(Primitive::Circle { center: middle, radius: INNER * 0.45, segments: 32 });
        for (k, ring) in lock.rings.iter().enumerate() {
            let radius = INNER + APART * k as f32;
            // Lit up in the slot's colour while it is in line, to be latched.
            let layer = match (ring.latched, k == lock.picked) {
                (true, _) => 2,
                (false, true) if lock.in_line() => 3,
                (false, true) => 1,
                (false, false) => 0,
            };
            let thick = if k == lock.picked { THICK * 1.4 } else { THICK };
            // The ring, but for its port; ticks across it, to see it turn.
            let port = ring.port;
            arc(&mut layers[layer], port + PORT / 2.0, port + TAU - PORT / 2.0, radius, thick);
            for tick in 1..6 {
                let angle = port + TAU * tick as f32 / 6.0;
                layers[layer].push(Primitive::Line {
                    begin: at(angle, radius - thick),
                    end: at(angle, radius + thick),
                    thickness: 3.0,
                });
            }
        }
        // The slot, at the top, outside the rings, and the lines down to the core either side.
        for side in [-1.0, 1.0] {
            let angle = side * PORT / 2.0;
            layers[3].push(Primitive::Line { begin: at(angle, INNER * 0.55), end: at(angle, outer), thickness: 2.0 });
        }
        layers[3].push(Primitive::Triangle {
            points: [at(0.0, outer + 4.0), at(-0.08, outer + 22.0), at(0.08, outer + 22.0)],
        });
        // The trace, under the rings: how much it has left.
        let left = (lock.trace / lock.trace_full).clamp(0.0, 1.0);
        let (x0, y) = (SIZE * 0.2, middle.y + outer + 18.0);
        layers[0].push(Primitive::Line {
            begin: Vector2::new(x0, y),
            end: Vector2::new(SIZE - x0, y),
            thickness: 10.0,
        });
        layers[if left < 0.3 { 1 } else { 3 }].push(Primitive::Line {
            begin: Vector2::new(x0, y),
            end: Vector2::new(x0 + (SIZE - 2.0 * x0) * left, y),
            thickness: 10.0,
        });
        for (layer, primitives) in self.layers.iter().zip(std::iter::once(backing).chain(layers)) {
            if let Ok(image) = ui.try_get_mut(*layer) {
                image.primitives.set_value_and_mark_modified(primitives);
                // Drawn afresh: the interface keeps what a widget drew until it is told otherwise.
                image.widget.invalidate_visual();
            }
        }
        ui.send(
            self.help,
            TextMessage::Text(format!(
                "Ring {} of {}    W/S ring    A/D turn    Space latch    Tab give up",
                lock.picked + 1,
                lock.len()
            )),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_port_turned_into_the_slot_latches_and_the_lock_opens_once_all_are() {
        let mut rng = Rng::new(4);
        let mut lock = PhaseLock::new(CELL_LOCK, &mut rng);
        assert!(lock.rings.iter().all(|ring| wrap(ring.port).abs() > LATCH_WITHIN), "none starts in the slot");
        let dt = 1.0 / 60.0;
        let mut frames = 0;
        // Turn each picked ring toward the slot, against its drift, and latch it once there.
        while lock.update(0.0, 0.0) == Picking::Going && frames < 60 * 30 {
            let port = wrap(lock.rings[lock.picked].port);
            let turn = if port > 0.0 { -1.0 } else { 1.0 };
            assert_eq!(lock.update(turn, dt), Picking::Going);
            if wrap(lock.rings[lock.picked].port).abs() <= LATCH_WITHIN * 0.5 {
                lock.latch();
            }
            frames += 1;
        }
        assert_eq!(lock.update(0.0, dt), Picking::Opened, "{frames} frames");
        assert!(lock.trace > 0.0);
    }

    #[test]
    fn latching_out_of_line_spends_the_trace_and_it_running_out_throws_the_pick() {
        let mut rng = Rng::new(9);
        let mut lock = PhaseLock::new(CELL_LOCK, &mut rng);
        let before = lock.trace;
        lock.latch();
        assert!(!lock.rings[0].latched);
        assert_eq!(lock.trace, before - MISLATCH);
        assert_eq!(lock.update(0.0, lock.trace + 0.1), Picking::Thrown);
    }

    #[test]
    fn a_ring_held_goes_only_as_it_is_turned_and_lights_up_in_line() {
        let mut lock = PhaseLock::new(CELL_LOCK, &mut Rng::new(2));
        let before = lock.rings[0].port;
        lock.update(1.0, 0.1);
        assert!((wrap(lock.rings[0].port - before) - NUDGE * 0.1).abs() < 1.0e-4, "turned, not drifting too");
        lock.rings[0].port = LATCH_WITHIN * 0.9;
        assert!(lock.in_line());
        lock.latch();
        assert!(lock.rings[0].latched && !lock.in_line(), "on to the next ring");
    }

    #[test]
    fn the_outer_rings_turn_faster_each_the_other_way() {
        let lock = PhaseLock::new(GATE_LOCK, &mut Rng::new(1));
        for pair in lock.rings.windows(2) {
            assert!(pair[1].drift.abs() > pair[0].drift.abs());
            assert!(pair[1].drift.signum() != pair[0].drift.signum());
        }
    }
}
