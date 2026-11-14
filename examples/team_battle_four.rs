//! Team battle with four teams: red, cyan, green and yellow droids, every one against every other,
//! in the combat arena, `data/arena/combat_map.glb`.
//!
//! The same battle as `team_battle` (see there for how the droids fight, and the keys), five a
//! side, each team starting from a side of the arena - red's across from cyan's, green's from
//! yellow's - in its own droids' colours: red the hostile droid's, cyan the full deform droid's,
//! green the explorer's and yellow the yellow droid's. Each team fights as a team against the
//! other three, keeping track of all of them; heading out with no enemy in sight, it makes for
//! one of the others' sides. The first team to the kill limit wins.
//!
//! `BATTLE_OLD=<team>` (`red`, `cyan`, `green` or `yellow`, or `all`) has that team fight as they
//! did before fighting as a team, as in `team_battle`.
//!
//! Run with `cargo run --example team_battle_four`.

#[path = "team_battle.rs"]
mod team_battle;

fn main() {
    team_battle::run(team_battle::Arena { title: "Team battle - four teams", teams: 4, ..team_battle::COMBAT });
}
