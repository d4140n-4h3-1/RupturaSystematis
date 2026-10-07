//! Capture the flag: two sides, red and blue, each with a base round its flag at one end of the
//! map (see [`crate::firewall`]). The player is blue - [`PLAYERS`] - and takes red's flag.
//!
//! Each side has [`DROIDS`] droids and a drone of its own. Blue's are the player's allies:
//! they pay the player no heed, and go after red's. Red's guard their end, watching for the
//! player all the while, and go after the player or blue's, whichever they see. Of each side's
//! droids, each keeps to a post of its own in its side's half, as the map marks them; each
//! side's drone patrols round its own flag. A droid comes after one of the other side it sees
//! and shoots it with its pistol; a drone fires at them. Each side's shots harm only the other
//! side: red's harm the player too.

use fyrox::core::algebra::Vector3;

/// A map it can be played on: what the main menu calls it, its model, and a line about it.
/// `void` is for one in open space, with nothing under it: no floor catches whoever goes over an
/// edge, and the sky is [`VOID_SKY`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Map {
    pub name: &'static str,
    pub path: &'static str,
    pub about: &'static str,
    pub void: bool,
    /// Whether it is out in the open, with the sky over it rather than a ceiling - as a map in
    /// the void is.
    pub open_sky: bool,
    /// Whether it is night there: the void's black sky over it and no sun, lit only by its lamps.
    pub night: bool,
    /// Whether voices echo there, off the buildings of a city in the open (see
    /// [`crate::formants::synth::CITY_ECHO`]).
    pub echoes: bool,
}

/// The Grid: a city traced in light on a platform in the void.
pub const GRID: Map = Map {
    name: "Grid",
    path: "data/arena/br_grid.glb",
    about: "A black city traced in light, on a platform in the void, with towers out over it on pads.",
    void: true,
    open_sky: true,
    night: true,
    echoes: true,
};

/// Nexus: a futuristic city traced in light, round a soaring spire, in the void.
pub const NEXUS: Map = Map {
    name: "Nexus",
    path: "data/arena/br_nexus.glb",
    about: "A city of light round a soaring spire: a skyway ring, ziggurats, domes and hovering pods.",
    void: true,
    open_sky: true,
    night: true,
    echoes: true,
};

/// The maps battle royale is played on, picked from in the main menu.
pub const ROYALE_MAPS: [Map; 2] = [GRID, NEXUS];

/// Aurum: a bank district, its vault behind three firewall doors.
pub const AURUM: Map = Map {
    name: "Aurum",
    path: "data/arena/heist_aurum.glb",
    about: "A bank district. Through the compound's gate, the bank's door and the vault's, and out.",
    void: true,
    open_sky: true,
    night: true,
    echoes: true,
};

/// The districts heists are played in, picked from in the main menu (see [`crate::heist`]).
pub const HEIST_MAPS: [Map; 1] = [AURUM];

/// The sky round a map in the void, each face of it.
pub const VOID_SKY: &str = "data/skybox.png";

/// The map whose model is at `path`, if one is.
pub fn map_at(path: &str) -> Option<&'static Map> {
    MAPS.iter().chain(&ROYALE_MAPS).chain(&HEIST_MAPS).find(|map| map.path == path)
}

/// The maps it can be played on, picked from in the main menu.
pub const MAPS: [Map; 4] = [
    Map {
        name: "Lanes",
        path: "data/arena/ctf_map.glb",
        about: "Walled bases joined by three lanes, all on one floor.",
        void: false,
        open_sky: false,
        night: false,
        echoes: false,
    },
    Map {
        name: "Balconies",
        path: "data/arena/ctf_balconies.glb",
        about: "Two floors: each flag in a well under a balcony, catwalks along the walls, a raised hub.",
        void: false,
        open_sky: false,
        night: false,
        echoes: false,
    },
    Map {
        name: "Hybrid",
        path: "data/arena/ctf_hybrid.glb",
        about: "The walled bases and lanes of Lanes, the two floors of Balconies, and a maze in the middle.",
        void: false,
        open_sky: false,
        night: false,
        echoes: false,
    },
    Map {
        name: "Void",
        path: "data/arena/ctf_void.glb",
        about: "Platforms in the void, joined by bridges, lit by street lights. Mind the edges.",
        void: true,
        open_sky: true,
        night: false,
        echoes: false,
    },
];

/// The player's side; the other is the enemy's.
pub const PLAYERS: Side = Side::Blue;

/// How many droids each side has.
pub const DROIDS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Red,
    Blue,
    /// A side of one, in battle royale (see [`crate::royale`]): everyone else is the enemy.
    Lone(u8),
}

impl Side {
    pub const BOTH: [Side; 2] = [Side::Red, Side::Blue];

    pub const fn name(self) -> &'static str {
        match self {
            Side::Red => "red",
            Side::Blue => "blue",
            Side::Lone(_) => "a lone droid",
        }
    }

    /// The other side, of red and blue; a side of one has none.
    pub const fn other(self) -> Side {
        match self {
            Side::Red => Side::Blue,
            Side::Blue => Side::Red,
            Side::Lone(n) => Side::Lone(n),
        }
    }

    /// Whether `them` are the enemy: of another side.
    pub fn against(self, them: Side) -> bool {
        self != them
    }

    /// Whether it is the player's side.
    pub fn is_players(self) -> bool {
        self == PLAYERS
    }
}

/// Where a side's droids and drone go: the flags, and each side's posts, red's and blue's, as
/// the map has them.
#[derive(Debug, Clone, PartialEq)]
pub struct Bases {
    pub red: Vector3<f32>,
    pub blue: Vector3<f32>,
    pub posts: [Vec<Vector3<f32>>; 2],
}

impl Bases {
    /// Where `side`'s flag is: red's, for a side of one, which has none.
    pub fn flag(&self, side: Side) -> Vector3<f32> {
        match side {
            Side::Red | Side::Lone(_) => self.red,
            Side::Blue => self.blue,
        }
    }

    /// Where the `n`th droid of `side` keeps to: its post, or its own flag with no post for it.
    pub fn post(&self, side: Side, n: usize) -> Vector3<f32> {
        let posts = &self.posts[usize::from(side == Side::Blue)];
        posts.get(n).copied().unwrap_or_else(|| self.flag(side))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_map_is_there_to_play() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for map in MAPS {
            assert!(dir.join(map.path).is_file(), "{} has no model at {}", map.name, map.path);
        }
        for map in ROYALE_MAPS.iter().chain(&HEIST_MAPS) {
            assert!(dir.join(map.path).is_file(), "{} has no model at {}", map.name, map.path);
        }
        assert!(dir.join(VOID_SKY).is_file(), "no sky for the void at {VOID_SKY}");
    }

    #[test]
    fn each_droid_keeps_to_its_own_post_or_else_its_flag() {
        let post = |x: f32| Vector3::new(x, 0.0, 0.0);
        let bases = Bases {
            red: post(-37.0),
            blue: post(37.0),
            posts: [vec![post(-34.0), post(-27.0), post(-17.0)], vec![post(34.0)]],
        };
        let red: Vec<_> = (0..DROIDS).map(|n| bases.post(Side::Red, n)).collect();
        assert_eq!(red, [post(-34.0), post(-27.0), post(-17.0)]);
        let blue: Vec<_> = (0..DROIDS).map(|n| bases.post(Side::Blue, n)).collect();
        assert_eq!(blue, [post(34.0), post(37.0), post(37.0)], "the flag, past the posts there are");
    }
}
