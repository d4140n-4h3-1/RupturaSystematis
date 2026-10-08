//! Lines of light that light what is round them: the glowing trim of a map such as the Grid,
//! shining on the walls and the floor by it - and on whoever is near it, in its colour, so that
//! a droid by a lit wall stands out against it - with traced, soft shadows.
//!
//! A map's lines come from `<model>.glow.json` beside its model (see `data/arena/br_grid.py`),
//! already cut into pieces a few meters long. There are thousands; the renderer lights only
//! [`fyrox_gfx::MAX_AREA_LIGHTS`] at once. So each frame the pieces nearest the player are lit,
//! and those nearest each droid, the nearest droids first - each fading in as it is taken up and
//! out as it is let go, rather than popping on and off.

use fyrox::core::{algebra::Vector3, color::Color, log::Log};
use fyrox_gfx::AreaLight;
use serde::Deserialize;

/// How far a piece lights anything, in meters.
const REACH: f32 = 6.0;
/// How near the middle of a piece someone must be, in meters, for it to be lit for them.
const PICK_WITHIN: f32 = 4.5;
/// How many pieces are lit round the player, and round each droid.
const AROUND_PLAYER: usize = 10;
const AROUND_DROID: usize = 4;
/// How far from the player a droid can be for the pieces by it to be lit, in meters.
const DROIDS_WITHIN: f32 = 70.0;
/// How long a piece takes to fade in or out, in seconds.
const FADE: f32 = 0.3;
/// How bright a piece's light is, as a share of how brightly it glows.
const BRIGHTNESS: f32 = 6.0;
/// How far apart the points a piece's light is worked out from are, in meters.
const SPACING: f32 = 0.6;

#[derive(Deserialize)]
struct File {
    /// How brightly the lines glow.
    intensity: f32,
    pieces: Vec<Piece>,
}

#[derive(Deserialize)]
struct Piece {
    corner: [f32; 3],
    edges: [[f32; 3]; 2],
    colour: [f32; 3],
}

/// A piece of a line of light, ready to light with.
#[derive(Debug, Clone, PartialEq)]
struct Lit {
    light: AreaLight,
    middle: Vector3<f32>,
}

/// A map's lines of light, and which pieces of them are lit, how far faded in.
#[derive(Debug, Default, PartialEq)]
pub struct GlowLights {
    pieces: Vec<Lit>,
    /// The pieces lit, by index, wanted ones first, with how far each is faded in, 0 to 1.
    lit: Vec<(usize, f32)>,
}

impl GlowLights {
    /// The lines of light beside the model at `model_path`, if it has any.
    pub fn load(model_path: &str) -> Option<Self> {
        let stem = model_path.rsplit_once('.').map_or(model_path, |(stem, _)| stem);
        let path = format!("{stem}.glow.json");
        if !std::path::Path::new(&path).is_file() {
            return None;
        }
        let read = crate::platform::read_to_string(&path)
            .and_then(|text| serde_json::from_str::<File>(&text).map_err(|e| e.to_string()));
        match read {
            Ok(file) => {
                let lights = Self::from_pieces(file);
                Log::info(format!("Glow: {} pieces of light in {path}", lights.pieces.len()));
                Some(lights)
            }
            Err(error) => {
                Log::err(format!("Glow: could not read {path}: {error}"));
                None
            }
        }
    }

    fn from_pieces(file: File) -> Self {
        let v = |v: [f32; 3]| Vector3::new(v[0], v[1], v[2]);
        let pieces = file
            .pieces
            .into_iter()
            .map(|piece| {
                let [u, w] = piece.edges.map(v);
                let corner = v(piece.corner);
                let [r, g, b] = piece.colour.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
                let light = AreaLight::new(corner, [u, w], SPACING)
                    .with_colour(Color::opaque(r, g, b), file.intensity * BRIGHTNESS)
                    .with_reach(REACH)
                    .two_sided();
                Lit { light, middle: corner + (u + w) * 0.5 }
            })
            .collect();
        Self { pieces, lit: Vec::new() }
    }

    /// Picks the pieces to light for another `dt`: those nearest the `player`, and those nearest
    /// each of the `droids`, the nearest to the player first; no more than `most` in all, those
    /// fading out included.
    pub fn update(&mut self, dt: f32, player: Vector3<f32>, droids: &[Vector3<f32>], most: usize) {
        let mut droids: Vec<Vector3<f32>> =
            droids.iter().copied().filter(|d| (d - player).norm() < DROIDS_WITHIN).collect();
        droids.sort_by(|a, b| (a - player).norm_squared().total_cmp(&(b - player).norm_squared()));
        let mut wanted: Vec<usize> = Vec::new();
        let focus = std::iter::once((player, AROUND_PLAYER)).chain(droids.into_iter().map(|d| (d, AROUND_DROID)));
        for (at, how_many) in focus {
            if wanted.len() >= most {
                break;
            }
            let mut near: Vec<(f32, usize)> = self
                .pieces
                .iter()
                .enumerate()
                .map(|(n, piece)| ((piece.middle - at).norm_squared(), n))
                .filter(|&(d, n)| d < PICK_WITHIN * PICK_WITHIN && !wanted.contains(&n))
                .collect();
            near.sort_by(|a, b| a.0.total_cmp(&b.0));
            wanted.extend(near.into_iter().take(how_many.min(most - wanted.len())).map(|(_, n)| n));
        }
        let step = dt / FADE;
        let mut lit: Vec<(usize, f32)> = wanted
            .iter()
            .map(|&n| {
                let was = self.lit.iter().find(|&&(m, _)| m == n).map_or(0.0, |&(_, w)| w);
                (n, (was + step).min(1.0))
            })
            .collect();
        // Those let go fade out, while there is room for them.
        let going = self
            .lit
            .iter()
            .filter(|(n, _)| !wanted.contains(n))
            .map(|&(n, w)| (n, w - step))
            .filter(|&(_, w)| w > 0.0);
        lit.extend(going);
        lit.truncate(most);
        self.lit = lit;
    }

    /// The pieces lit, as bright as they are faded in.
    pub fn lights(&self) -> impl Iterator<Item = AreaLight> + '_ {
        self.lit.iter().map(|&(n, faded)| {
            let light = self.pieces[n].light;
            light.with_colour(light.colour, light.intensity * faded)
        })
    }

    /// Puts every piece out at once.
    pub fn clear(&mut self) {
        self.lit.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A row of pieces a meter long along x, one every `apart` meters from the origin.
    fn row(count: usize, apart: f32) -> GlowLights {
        let pieces = (0..count)
            .map(|i| Piece {
                corner: [i as f32 * apart, 1.0, 0.0],
                edges: [[1.0, 0.0, 0.0], [0.0, 0.1, 0.0]],
                colour: [0.0, 0.85, 1.0],
            })
            .collect();
        GlowLights::from_pieces(File { intensity: 3.0, pieces })
    }

    fn lit(glow: &GlowLights) -> Vec<usize> {
        glow.lit.iter().map(|&(n, _)| n).collect()
    }

    #[test]
    fn the_pieces_nearest_the_player_and_each_droid_are_lit() {
        let mut glow = row(40, 2.0);
        glow.update(0.1, Vector3::new(0.0, 1.0, 0.0), &[Vector3::new(60.0, 1.0, 0.0)], 64);
        let lit = lit(&glow);
        assert_eq!(&lit[..2], &[0, 1], "round the player first, nearest first");
        assert!(lit.contains(&30), "and by the droid: {lit:?}");
        assert!(!lit.contains(&15), "but nothing near neither");
    }

    #[test]
    fn no_more_are_lit_than_there_is_room_for_and_the_player_comes_first() {
        let mut glow = row(40, 0.5);
        glow.update(0.1, Vector3::zeros(), &[Vector3::new(10.0, 1.0, 0.0)], 6);
        assert_eq!(glow.lit.len(), 6);
        assert!(lit(&glow).iter().all(|&n| n < 10), "{:?}", lit(&glow));
    }

    #[test]
    fn a_piece_fades_in_when_taken_up_and_out_when_let_go() {
        let mut glow = row(1, 1.0);
        let near = Vector3::new(0.5, 1.0, 0.0);
        glow.update(FADE / 2.0, near, &[], 64);
        let brightness = |glow: &GlowLights| glow.lights().next().map_or(0.0, |l| l.intensity);
        let full = 3.0 * BRIGHTNESS;
        assert!((brightness(&glow) - full / 2.0).abs() < 1.0e-4, "half way in");
        glow.update(FADE, near, &[], 64);
        assert!((brightness(&glow) - full).abs() < 1.0e-4, "all the way in");
        let far = Vector3::new(100.0, 1.0, 0.0);
        glow.update(FADE / 4.0, far, &[], 64);
        assert!((brightness(&glow) - full * 0.75).abs() < 1.0e-4, "fading out");
        glow.update(FADE, far, &[], 64);
        assert_eq!(glow.lights().count(), 0, "out");
    }

    #[test]
    fn urbs_has_its_lines_of_light_near_the_ground() {
        let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), crate::ctf::URBS.path);
        let glow = GlowLights::load(&path).expect("lines of light");
        assert!(glow.pieces.len() > 1000);
        for piece in &glow.pieces {
            let [u, w] = piece.light.edges;
            assert!(u.norm() <= 3.01 && w.norm() <= u.norm() + 1.0e-3, "{u} {w}");
            // On the platform, its gates or its railway, and no higher than lines are lit.
            assert!(piece.middle.x.abs() < 265.0 && piece.middle.z.abs() < 265.0, "{}", piece.middle);
            assert!(piece.middle.y > -3.0 && piece.middle.y < 12.0, "{}", piece.middle);
        }
    }

    #[test]
    fn the_grid_and_nexus_have_their_lines_of_light() {
        for map in ["br_grid", "br_nexus"] {
            let path = format!("{}/data/arena/{map}.glb", env!("CARGO_MANIFEST_DIR"));
            let glow = GlowLights::load(&path).expect("lines of light");
            assert!(glow.pieces.len() > 1000, "{map}");
            // Every piece is short enough to light round it, and lies on the platform or its pads.
            for piece in &glow.pieces {
                let [u, w] = piece.light.edges;
                assert!(u.norm() <= 3.01 && w.norm() <= u.norm() + 1.0e-3, "{map}: {u} {w}");
                assert!(piece.middle.x.abs() < 145.0 && piece.middle.z.abs() < 145.0 && piece.middle.y > -3.0, "{map}");
            }
        }
    }
}
