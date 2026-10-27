//! Jail, in a city with precincts (see [`crate::precinct`]): when security takes the player's
//! last hit, or the player puts their hands up with H while security is after them, they are
//! arrested rather than deleted. Their pistol goes into the precinct's evidence locker, everyone
//! after them gives it up, and they are put in a cell, behind its firewall, for a sentence as long
//! as they are wanted for (see [`sentence`]).
//!
//! There are six ways out:
//! - **Serve the sentence**: it runs down while they are in their cell, and then the firewall and
//!   the gate drop, and they are free to go.
//! - **Pick the lock**: E at the firewall, from inside, to pick it (see [`crate::lockpick`]);
//!   then the gate's, a harder lock - unless they have a key card.
//! - **Bribe a guard**: E at a guard standing by, and again to pay what it asks (see [`bribe`]):
//!   it drops the firewall and the gate, and they walk out as if released.
//! - **Steal a key card**: E at a guard with its back to them, near enough to reach through the
//!   firewall. It opens their cell's firewall and the gate - but a guard that sees them try adds to
//!   their sentence.
//! - **The tunnel**: in a cell with a loose panel in its back wall, E to pry it loose, a little at
//!   a time, then a stair down to a tunnel out.
//! - **The drain**: in any cell, E at the grate over it to pry it up, a little at a time, then E
//!   to drop down into the escape tunnels (see [`crate::escape`]).
//!
//! Out of the cell but for being released, they have escaped: a guard that sees them in the
//! precinct goes after them, and calls the others; caught, they are back in a cell for longer.
//! Once well clear of the precinct, unseen, or down in the escape tunnels, they are away. Seen at anything - picking, prying,
//! reaching for a card - adds to the sentence, and stops it.

/// How long a sentence is at least, in seconds, how much each count they are wanted for adds, and
/// how long it can be at most.
const SENTENCE: f32 = 60.0;
const PER_COUNT: f32 = 30.0;
const LONGEST: f32 = 300.0;
/// How much being seen at something adds, in seconds, and being caught again after escaping.
pub const CAUGHT_AT_IT: f32 = 30.0;
pub const CAUGHT_AGAIN: f32 = 60.0;
/// A bribe: how many credits at least, and how many more for each second of the sentence left.
const BRIBE: u64 = 40;
const BRIBE_PER_SECOND: u64 = 1;
/// How long a guard's offer stands, in seconds, for the player to pay it with E.
pub const OFFER_STANDS: f32 = 5.0;
/// How near a guard has to be to talk to through the firewall, and to reach for its card, in
/// meters along the ground; how near the firewall or the panel to work at it.
pub const GUARD_REACH: f32 = 2.6;
pub const CARD_REACH: f32 = 1.5;
pub const AT_LOCK: f32 = 1.3;
/// How long the panel, or a drain's grate, takes to pry loose, in seconds of prying.
pub const PRYING: f32 = 10.0;
/// How long after being put in a cell, in seconds, a player who has dropped through its floor is
/// put back in it: the physics now and then lets a body just set down there through.
pub const PUT_BACK: f32 = 2.0;
/// How far from the precinct, in meters, unseen, the player is away.
pub const AWAY: f32 = 40.0;

/// How the player is held, while they are.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Custody {
    /// The precinct and the cell.
    pub precinct: usize,
    pub cell: usize,
    /// How long the sentence has left, in seconds.
    pub sentence: f32,
    /// Whether they are out of the cell without being released.
    pub escaped: bool,
    /// Whether they have a guard's key card.
    pub card: bool,
    /// A guard's offer to look away, and how long it stands.
    pub offer: Option<(usize, u64, f32)>,
    /// Whether they are prying at the panel.
    pub prying: bool,
    /// Whether they have been seen in the cell yet: they are only taken to have left it once
    /// they have been put in it.
    pub settled: bool,
    /// How long since they were put in it, in seconds.
    pub held: f32,
}

impl Custody {
    pub fn new(precinct: usize, cell: usize, wanted: u32) -> Self {
        Self { precinct, cell, sentence: sentence(wanted), escaped: false, card: false, offer: None, prying: false, settled: false, held: 0.0 }
    }
}

/// How long a sentence is, in seconds, for being wanted for `counts`.
pub fn sentence(counts: u32) -> f32 {
    (SENTENCE + PER_COUNT * counts as f32).min(LONGEST)
}

/// What a guard asks, in credits, to look away with `left` seconds of the sentence left.
pub fn bribe(left: f32) -> u64 {
    BRIBE + BRIBE_PER_SECOND * left.max(0.0).ceil() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_more_they_are_wanted_for_the_longer_and_dearer_to_get_out_of() {
        assert_eq!(sentence(0), SENTENCE);
        assert!(sentence(3) > sentence(1));
        assert_eq!(sentence(1000), LONGEST, "but not for ever");
        assert!(bribe(sentence(3)) > bribe(sentence(0)));
        assert_eq!(bribe(0.0), BRIBE, "even at the last");
    }
}
