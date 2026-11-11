//! Maze: a small maze game, played as a droid seen from behind, rendered with Vulkan.
//!
//! Every round is a new maze, put together at random from the tile models in `data/` (see
//! [`generate`] and [`tiles`]). The player starts at one end of the longest route through it and a
//! glowing exit waits at the other end; the clock runs until the player reaches it.
//!
//! Controls: WASD to move, Caps Lock to go between walking and jogging, a tap of Shift between
//! walking and running and Shift held to sprint - running and sprinting cost stamina, and running
//! out of it leaves the player walking - Space to
//! jump, mouse to look, C to crouch, Z to crawl (each toggles), hold Q to look behind, Tab to
//! take cover against a wall - A and D slide along it, and lean round its corner at the edge - F
//! for the flashlight, V for third or first person, hold the right mouse button to strafe, R to
//! draw or holster the pistol, the left mouse button to draw it and fire, 1 to raise the shield
//! (see [`shield`]), E to talk to a droid close by, H in Urbs to give yourself up to security (see
//! [`jail`]), N for a new maze, Escape to pause. Run with `cargo run` from this directory;
//! `MAZE_SIZE=<w>x<d>` sets how many junctions wide and deep the maze is, `MAZE_DEBUG=1` also prints
//! the walkable map the game made of the level, `MAZE_SEED=<n>` makes every maze identical,
//! `MAZE_WINDOWED=1` opens a window instead of filling the screen, and `MAZE_MODEL=<path>` plays a
//! fixed maze model instead, such as `data/maze_full.fbx`; with a race track, such as
//! `data/arena/race_circuit.glb`, `MAZE_RACE_AUTOPILOT=1` drives the bike round it by itself, and `MAZE_CHIPS=1` puts a credit chip of each worth in front of the player - straight into a maze, or in Story mode with Urbs as the model. In a browser these go in the page's
//! address instead, as `?MAZE_SEED=7` (see [`platform::var`]).

//!
//! The game is this library, which `src/main.rs` runs; the examples (see `examples/`) use it too,
//! such as the droids the player and the maze's inhabitants are made from ([`player::avatar`]).

/// The AI's tuning: how far and how long the droids, drones and traffic do what they do (see
/// [`hydroxus_ai::Tuning`]) - hydroxus-ai's own numbers, which are Maze's.
pub const AI: hydroxus_ai::Tuning = hydroxus_ai::Tuning::DEFAULT;

/// The fog over the foggy maps (see `ctf::Map::foggy`), for the examples that fight in them,
/// and what the droids see through it as.
pub use ctf::{sight_fog, FOG};

mod alarm;
mod chips;
mod computer;
mod credits;
mod ctf;
mod culling;
mod diagnostics;
mod distant;
mod dialogue;
pub mod dismember;
mod drone;
mod drone_shot;
mod escape;
mod health;
mod hearts;
mod heist;
mod helicopter;
mod shield;
mod firewall;
mod ferry;
mod lift;
mod locker;
mod glow;
mod fixtures;
mod formants;
pub mod game;
mod generate;
mod hud;
mod inhabitants;
mod jail;
mod latency;
mod layout;
mod leaves;
mod level;
mod loading;
mod lockpick;
mod menu;
mod notes;
mod powerups;
mod precinct;
mod race;
pub mod platform;
pub mod player;
pub mod ragdoll;
mod roads;
mod royale;
mod survey;
mod tiles;
mod traffic;
mod train;
mod vault;
mod vending;

#[cfg(test)]
mod ai_tuning {
    /// The game's AI tuning holds no number the droids could not work with (see
    /// [`hydroxus_ai::validate`]): a change to [`super::AI`] that does is caught here.
    #[test]
    fn the_ai_tuning_is_valid() {
        if let Err(invalid) = super::AI.validate() {
            panic!("{invalid}");
        }
    }
}
