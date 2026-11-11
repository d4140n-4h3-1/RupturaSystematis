//! Power-ups the player carries: bought from a merchant - the Mutator, the city's yellow droid
//! (see [`crate::dialogue::Buy`]) - and used with the number keys. 1 raises the shield, a spare
//! cell for it bought going with the shield's own (see [`crate::shield::MOST_SPARES`]); 2 is a
//! repair kit, a heart's worth of health back at once (see [`crate::health::HEART`]); 3 is a
//! stamina cell, the player's breath frozen a while (see [`crate::player::breath::FREEZE`]).
//! Found lying about the maze, hearts and stamina cells are used where they are picked up, as
//! ever; bought, they are carried, [`MOST`] of each at most.

use serde::Deserialize;

/// How many of each the player can carry.
pub const MOST: u32 = 3;

/// Something the player can buy and use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerUp {
    /// A spare cell for the shield: key 1.
    Shield,
    /// A repair kit: a heart's worth of health back. Key 2.
    Repair,
    /// A stamina cell: breath frozen a while. Key 3.
    Stamina,
}

impl PowerUp {
    /// What it is called, as the player is told.
    pub fn name(self) -> &'static str {
        match self {
            PowerUp::Shield => "shield cell",
            PowerUp::Repair => "repair kit",
            PowerUp::Stamina => "stamina cell",
        }
    }

    /// The number key that uses it.
    pub fn key(self) -> u32 {
        match self {
            PowerUp::Shield => 1,
            PowerUp::Repair => 2,
            PowerUp::Stamina => 3,
        }
    }
}

/// What the player carries of those kept here: repair kits and stamina cells. Shield cells are
/// the shield's own spares.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Carried {
    pub repairs: u32,
    pub stamina: u32,
}

impl Carried {
    fn count(&mut self, power: PowerUp) -> Option<&mut u32> {
        match power {
            PowerUp::Repair => Some(&mut self.repairs),
            PowerUp::Stamina => Some(&mut self.stamina),
            PowerUp::Shield => None,
        }
    }

    /// Whether there is room for another `power`; none here for shield cells.
    pub fn room(&self, power: PowerUp) -> bool {
        match power {
            PowerUp::Repair => self.repairs < MOST,
            PowerUp::Stamina => self.stamina < MOST,
            PowerUp::Shield => false,
        }
    }

    /// One more `power`, if there is room. Whether there was.
    pub fn add(&mut self, power: PowerUp) -> bool {
        match self.count(power) {
            Some(n) if *n < MOST => {
                *n += 1;
                true
            }
            _ => false,
        }
    }

    /// One `power` taken out to use, if there is one. Whether there was.
    pub fn take(&mut self, power: PowerUp) -> bool {
        match self.count(power) {
            Some(n) if *n > 0 => {
                *n -= 1;
                true
            }
            _ => false,
        }
    }

    /// How it is shown: each carried, with its key - nothing, carrying none.
    pub fn shown(&self) -> String {
        let mut parts = Vec::new();
        if self.repairs > 0 {
            parts.push(format!("[2] REPAIR ×{}", self.repairs));
        }
        if self.stamina > 0 {
            parts.push(format!("[3] STAMINA ×{}", self.stamina));
        }
        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_carries_up_to_the_most_of_each_and_uses_them_one_at_a_time() {
        let mut carried = Carried::default();
        assert_eq!(carried.shown(), "");
        for _ in 0..MOST {
            assert!(carried.add(PowerUp::Repair));
        }
        assert!(!carried.room(PowerUp::Repair) && !carried.add(PowerUp::Repair));
        assert!(carried.room(PowerUp::Stamina) && carried.add(PowerUp::Stamina));
        assert_eq!(carried.shown(), format!("[2] REPAIR ×{MOST}\n[3] STAMINA ×1"));
        assert!(carried.take(PowerUp::Stamina) && !carried.take(PowerUp::Stamina));
        assert!(carried.take(PowerUp::Repair) && carried.repairs == MOST - 1);
        // Shield cells are the shield's: none carried here.
        assert!(!carried.room(PowerUp::Shield) && !carried.add(PowerUp::Shield) && !carried.take(PowerUp::Shield));
        assert_eq!(PowerUp::Repair.key(), 2);
    }
}
