//! Team battle at scale: four teams of fifty - red, cyan, green and yellow, every one against every
//! other - in Nexus, the battle royale's city of light on a platform in the void,
//! `data/arena/br_nexus.glb`: to see how the AI and the droids hold up with 200 fighting at once.
//!
//! The same battle as `team_battle` (see there for how the droids fight, and the keys): each team
//! from its own side of the platform, the skyway ring 7 m up and the ziggurats for high ground, and
//! the first to three kills a droid it has - 150 - wins. `BATTLE_BENCH=<seconds>` times every part
//! of every frame (see `team_battle`).
//!
//! Run with `cargo run --release --example team_battle_nexus`.

#[path = "team_battle.rs"]
mod team_battle;

fn main() {
    team_battle::run(team_battle::NEXUS);
}
