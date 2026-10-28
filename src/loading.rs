//! The loading screen, from the moment a game is picked until it can be played: the name of the
//! map and what it is, a bar filling up and what is being done - the models loading, one by one,
//! then the level put together, then surveyed - and a tip that changes every few seconds.
//!
//! Most of the bar is the models loading, which happens alongside the game, so the screen keeps
//! drawing. Putting the level together and surveying it each hold up a frame or more, and in a
//! city many seconds; they wait for [`LoadingScreen::drawn`], so that the screen saying which it
//! is on is what shows while the game is busy.

use fyrox::{
    core::{color::Color, pool::Handle},
    gui::{
        border::{Border, BorderBuilder},
        brush::Brush,
        screen::{Screen, ScreenBuilder},
        stack_panel::StackPanelBuilder,
        text::{Text, TextBuilder, TextMessage},
        widget::{WidgetBuilder, WidgetMessage},
        HorizontalAlignment, Thickness, UserInterface, VerticalAlignment,
    },
};

/// How big the bar is inside its frame, in pixels.
const BAR: (f32, f32) = (520.0, 8.0);
/// The bar's colour, the menu's picked blue, and the colour of the quieter lines.
const BAR_BLUE: Color = Color::opaque(110, 225, 255);
const DIM: Color = Color::opaque(150, 155, 170);
/// How quickly the bar catches up with how far loading has got, per second.
const CATCH_UP: f32 = 5.0;
/// How long each tip stays, in seconds, and how often the dots after the step go round.
const TIP_TIME: f32 = 6.0;
const DOTS_TIME: f32 = 0.4;
/// How many frames a step is drawn before what it holds up goes ahead, and how long it waits for
/// them at most, in seconds: a hidden window may draw none.
const FRAMES_FIRST: u32 = 2;
const WAIT_AT_MOST: f32 = 2.0;

/// What there is to know while waiting.
const TIPS: [&str; 10] = [
    "Caps Lock goes between walking and jogging; a tap of Shift runs, and holding it sprints.",
    "Running and sprinting cost stamina. Run out and you are left walking until you get your breath back.",
    "Tab takes cover against a wall. A and D slide along it, and at its edge you lean round the corner.",
    "Hold Q to look behind you.",
    "C crouches and Z crawls; press either again to stand.",
    "E talks to a droid close by, or uses the computer in front of you.",
    "1 raises the shield, once you have one. It takes the hits until it breaks.",
    "CAUTION and EVASION count down. Stay out of sight until they run out.",
    "In Urbs, H gives you up to security. There is more than one way out of a cell.",
    "F switches the flashlight on and off; V goes between third and first person.",
];

/// What the game is doing, to show on the way through.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    /// Loading the level's models: how many have, out of how many.
    Models { loaded: usize, total: usize },
    /// Putting the level together from them.
    Building,
    /// Surveying it, for where it can be walked, and putting everyone in it.
    Settling,
}

impl Step {
    /// How far along the bar it puts loading, from 0 to 1. The models are most of the wait.
    pub fn progress(self) -> f32 {
        match self {
            Step::Models { loaded, total } => 0.75 * loaded as f32 / total.max(1) as f32,
            Step::Building => 0.8,
            Step::Settling => 0.92,
        }
    }

    fn describe(self) -> String {
        match self {
            Step::Models { loaded, total } => format!("Loading models  {loaded}/{total}"),
            Step::Building => "Building the level".to_string(),
            Step::Settling => "Surveying the ground".to_string(),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct LoadingScreen {
    screen: Handle<Screen>,
    name: Handle<Text>,
    about: Handle<Text>,
    step: Handle<Text>,
    tip: Handle<Text>,
    fill: Handle<Border>,
    /// Whether it is showing, and how full the bar is drawn, from 0 to 1.
    open: bool,
    shown: f32,
    /// How long it has been showing, in seconds, which picks the tip and the dots.
    time: f32,
    /// What was last written, so that nothing is sent that has not changed.
    written: (String, usize),
    /// The step showing, and how many frames it has been drawn in and for how long.
    step_shown: Option<Step>,
    frames: u32,
    waited: f32,
}

impl LoadingScreen {
    /// Builds the screen, hidden, over everything built in `ui` before it, on a backdrop nothing
    /// shows through.
    pub fn build(ui: &mut UserInterface) -> Self {
        let ctx = &mut ui.build_ctx();
        let text = |ctx: &mut _, size: f32, colour: Color, margin: Thickness| {
            TextBuilder::new(
                WidgetBuilder::new()
                    .with_margin(margin)
                    .with_foreground(Brush::Solid(colour).into()),
            )
            .with_font_size(size.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .build(ctx)
        };
        let name = text(ctx, 44.0, Color::WHITE, Thickness::bottom(8.0));
        let about = text(ctx, 18.0, DIM, Thickness::bottom(48.0));
        let step = text(ctx, 18.0, Color::WHITE, Thickness::bottom(8.0));
        let fill = BorderBuilder::new(
            WidgetBuilder::new()
                .with_horizontal_alignment(HorizontalAlignment::Left)
                .with_margin(Thickness::uniform(3.0))
                .with_width(0.0)
                .with_height(BAR.1)
                .with_background(Brush::Solid(BAR_BLUE).into()),
        )
        .with_stroke_thickness(Thickness::zero().into())
        .build(ctx);
        let frame = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(BAR.0 + 6.0)
                .with_height(BAR.1 + 6.0)
                .with_foreground(Brush::Solid(Color::opaque(90, 100, 120)).into())
                .with_child(fill),
        )
        .with_stroke_thickness(Thickness::uniform(1.0).into())
        .build(ctx);
        let middle = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_horizontal_alignment(HorizontalAlignment::Center)
                .with_vertical_alignment(VerticalAlignment::Center)
                .with_child(name)
                .with_child(about)
                .with_child(step)
                .with_child(frame),
        )
        .build(ctx);
        // Along the bottom, away from the bar, where it can be read or not.
        let tip = TextBuilder::new(
            WidgetBuilder::new()
                .with_horizontal_alignment(HorizontalAlignment::Center)
                .with_vertical_alignment(VerticalAlignment::Bottom)
                // Tall enough for the tails of the letters, which the text alone leaves off.
                .with_height(30.0)
                .with_margin(Thickness::uniform(32.0))
                .with_foreground(Brush::Solid(DIM).into()),
        )
        .with_font_size(18.0.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .build(ctx);
        let backdrop = BorderBuilder::new(
            WidgetBuilder::new()
                .with_background(Brush::Solid(Color::opaque(8, 10, 14)).into())
                .with_child(middle)
                .with_child(tip),
        )
        .with_stroke_thickness(Thickness::uniform(0.0).into())
        .build(ctx);
        let screen = ScreenBuilder::new(WidgetBuilder::new().with_visibility(false).with_child(backdrop))
            .build(ctx);
        Self { screen, name, about, step, tip, fill, ..Default::default() }
    }

    /// Shows what `step` loading is on, `dt` seconds after the last time, for the map `name`,
    /// which is `about` what it says; or with no step hides the screen.
    pub fn update(&mut self, ui: &UserInterface, dt: f32, step: Option<(Step, &str, &str)>) {
        let Some((step, name, about)) = step else {
            if self.open {
                ui.send(self.screen, WidgetMessage::Visibility(false));
                self.open = false;
                self.step_shown = None;
            }
            return;
        };
        if !self.open {
            ui.send(self.screen, WidgetMessage::Visibility(true));
            ui.send(self.name, TextMessage::Text(name.to_string()));
            ui.send(self.about, TextMessage::Text(about.to_string()));
            self.open = true;
            self.shown = 0.0;
            self.time = 0.0;
            self.written = Default::default();
        }
        self.time += dt;
        if self.step_shown != Some(step) {
            self.step_shown = Some(step);
            self.frames = 0;
            self.waited = 0.0;
        }
        self.waited += dt;
        // Eased towards how far it has got, and never back: a new maze in the same game finds
        // its models already loaded.
        let target = step.progress();
        self.shown = self.shown.max(self.shown + (target - self.shown) * (CATCH_UP * dt).min(1.0));
        ui.send(self.fill, WidgetMessage::Width(BAR.0 * self.shown));
        let dots = ".".repeat(1 + (self.time / DOTS_TIME) as usize % 3);
        let line = format!("{}{dots}", step.describe());
        let tip = (self.time / TIP_TIME) as usize % TIPS.len();
        if line != self.written.0 {
            ui.send(self.step, TextMessage::Text(line.clone()));
        }
        if tip != self.written.1 || self.written.0.is_empty() {
            ui.send(self.tip, TextMessage::Text(TIPS[tip].to_string()));
        }
        self.written = (line, tip);
    }

    /// A frame is about to be drawn.
    pub fn rendering(&mut self) {
        if self.open {
            self.frames += 1;
        }
    }

    /// Whether the step last given to [`LoadingScreen::update`] is on screen, so that what it
    /// says the game is doing can hold up the game.
    pub fn drawn(&self) -> bool {
        self.step_shown.is_some() && (self.frames >= FRAMES_FIRST || self.waited >= WAIT_AT_MOST)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_only_goes_forward_through_the_steps() {
        let steps = [
            Step::Models { loaded: 0, total: 5 },
            Step::Models { loaded: 3, total: 5 },
            Step::Models { loaded: 5, total: 5 },
            Step::Building,
            Step::Settling,
        ];
        for pair in steps.windows(2) {
            assert!(pair[0].progress() < pair[1].progress(), "{pair:?}");
        }
        assert!(Step::Settling.progress() < 1.0);
        assert_eq!(Step::Models { loaded: 0, total: 0 }.progress(), 0.0);
    }

    #[test]
    fn a_step_is_drawn_after_two_frames_or_two_seconds() {
        let mut ui = UserInterface::new(fyrox::core::algebra::Vector2::new(800.0, 600.0));
        let mut screen = LoadingScreen::build(&mut ui);
        assert!(!screen.drawn());
        screen.update(&ui, 0.01, Some((Step::Building, "Maze", "")));
        screen.rendering();
        assert!(!screen.drawn());
        screen.rendering();
        assert!(screen.drawn());
        // A new step starts again.
        screen.update(&ui, 0.01, Some((Step::Settling, "Maze", "")));
        assert!(!screen.drawn());
        screen.update(&ui, 2.0, Some((Step::Settling, "Maze", "")));
        assert!(screen.drawn());
        screen.update(&ui, 0.01, None);
        assert!(!screen.drawn());
    }
}
