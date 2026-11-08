//! Team battle in the forest: red droids against cyan droids among abandoned buildings in a
//! foggy forest, `data/arena/tb_forest.glb` (built by `data/arena/tb_forest.py`) - a hangar with
//! its roof fallen in at the middle, and for each side a barracks, a roofless ruin and a
//! watchtower, with wrecks, containers, sandbags, fallen trunks and the trees for cover between.
//!
//! The same battle as `team_battle` (see there for how the droids fight, and the keys), in the
//! Foggy Forest's fog and grey daylight: the droids see no further than 30 m in it, and the
//! leaves hide them but stop nothing.
//!
//! Run with `cargo run --example team_battle_forest`.

#[path = "team_battle.rs"]
mod team_battle;

fn main() {
    team_battle::run(team_battle::FOREST);
}
