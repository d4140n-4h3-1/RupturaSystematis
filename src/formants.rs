//! Sounds made from formants, as a voice is (see [`hydroxus_voice`]): read from the game's own
//! files, and made into something to play. Each sound is described in a file of its own kind (see
//! `data/sounds/pistol_formants.json`, whose `about` says what everything in it means), and made
//! into samples when the game starts; [`synth`] does the making. [`speech`] makes a voice speak,
//! as the maze's droids do, and [`chirps`] a drone.

pub use hydroxus_voice::{chirps, speech, synth, Curve, Sound, Sounds};

use fyrox::scene::sound::{DataSource, SoundBuffer, SoundBufferResource};
use serde::de::DeserializeOwned;
use synth::Echo;

/// Where each droid's voice comes from.
pub const VOICES: &str = "data/sounds/voice_formants.json";
/// Where the drones' voice comes from.
pub const DRONE_VOICE: &str = "data/sounds/drone_voice.json";

/// The echo of a city of tall buildings in the void: a quarter of a second off the far walls,
/// back three or four times before it fades.
pub const CITY_ECHO: Echo = Echo {
    delay: 0.26,
    feedback: 0.42,
    damping: 0.45,
    repeats: 5,
};

/// Sounds, voices or chirps read from a file of the game's.
pub trait Load: Sized {
    /// What is in the file at `path`.
    fn load(path: &str) -> Result<Self, String>;
}

impl<T: DeserializeOwned> Load for T {
    fn load(path: &str) -> Result<Self, String> {
        let text = crate::platform::read_to_string(path)?;
        serde_json::from_str(&text).map_err(|error| format!("{path}: {error}"))
    }
}

/// `sound`, made at `sample_rate` into something to play.
pub fn buffer(sound: &Sound, sample_rate: u32) -> Option<SoundBufferResource> {
    playable(synth::make(sound, sample_rate), sample_rate)
}

/// `samples`, one channel at `sample_rate` a second, as something to play.
pub fn playable(samples: Vec<f32>, sample_rate: u32) -> Option<SoundBufferResource> {
    let data = DataSource::Raw {
        sample_rate: sample_rate as usize,
        channel_count: 1,
        samples,
    };
    SoundBuffer::raw_generic(data)
        .ok()
        .map(SoundBufferResource::new_embedded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::Mood;
    use chirps::Chirps;
    use speech::Voices;

    #[test]
    fn the_pistols_sounds_load() {
        let sounds = Sounds::load("data/sounds/pistol_formants.json").unwrap();
        for name in ["shot", "hum"] {
            let sound = &sounds.sounds[name];
            assert!(sound.length > 0.0 && !sound.formants.is_empty(), "{name}");
        }
        assert!(sounds.sounds["hum"].looped && !sounds.sounds["shot"].looped);
    }

    #[test]
    fn the_rifles_shot_is_made_and_is_shorter_and_deeper_than_the_pistols() {
        let rifle = Sounds::load("data/sounds/rifle_formants.json").unwrap();
        let pistol = Sounds::load("data/sounds/pistol_formants.json").unwrap();
        let (shot, pistol_shot) = (&rifle.sounds["shot"], &pistol.sounds["shot"]);
        let samples = synth::make(shot, rifle.sample_rate);
        let peak = samples.iter().fold(0.0_f32, |p, s| p.max(s.abs()));
        assert!(samples.iter().all(|s| s.is_finite()));
        assert!((peak - shot.volume).abs() < 1e-4, "{peak}");
        assert!(shot.length < pistol_shot.length, "short enough for ten a second over each other");
        assert!(shot.pitch.at(0.0) < pistol_shot.pitch.at(0.0), "deeper");
        assert!(rifle.sounds["hum"].looped && !shot.looped);
    }

    #[test]
    fn the_pistols_sounds_are_made_and_the_hum_goes_round_seamlessly() {
        let sounds = Sounds::load("data/sounds/pistol_formants.json").unwrap();
        for (name, sound) in &sounds.sounds {
            let samples = synth::make(sound, sounds.sample_rate);
            let peak = samples.iter().fold(0.0_f32, |p, s| p.max(s.abs()));
            assert!(samples.iter().all(|s| s.is_finite()), "{name}");
            assert!((peak - sound.volume).abs() < 1e-4, "{name}: {peak}");
        }
        let hum = synth::make(&sounds.sounds["hum"], sounds.sample_rate);
        let (last, first) = (hum[hum.len() - 1], hum[0]);
        let typical = hum
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f32::max);
        assert!(
            (first - last).abs() <= typical,
            "the seam jumps {} ",
            (first - last).abs()
        );
    }

    fn voices() -> Voices {
        Voices::load(VOICES).unwrap()
    }

    #[test]
    fn every_letter_has_a_sound_and_every_droid_a_voice() {
        let voices = voices();
        for phone in speech::phones("abcdefghijklmnopqrstuvwxyz") {
            assert!(voices.phones.contains_key(phone), "{phone}");
        }
        let script = crate::dialogue::Script::load(crate::dialogue::SCRIPT).unwrap();
        for character in &script.characters {
            assert!(
                voices.voices.contains_key(&character.name),
                "{}",
                character.name
            );
        }
    }

    #[test]
    fn a_mood_raises_or_lowers_the_voice() {
        let voices = voices();
        assert_eq!(voices.mood_pitch(Mood::Normal.name()), 1.0);
        assert!(voices.mood_pitch(Mood::Agitated.name()) > 1.0, "agitated, higher");
        assert!(voices.mood_pitch(Mood::Hostile.name()) < 1.0, "hostile, lower");
        let voice = voices.voice("default").unwrap();
        let first = |pitch: f32| voices.speak("Sta.", voice, pitch).pitch.0[0][1];
        let agitated = voices.mood_pitch(Mood::Agitated.name());
        assert!((first(agitated) - first(1.0) * agitated).abs() < 1.0e-3);
    }

    #[test]
    fn a_question_rises_and_a_statement_falls() {
        let voices = voices();
        let voice = voices.voice("default").unwrap();
        let pitch_at_end = |text: &str| {
            let sound = voices.speak(text, voice, 1.0);
            let last = sound.pitch.0.iter().rev().find(|p| p[1] > 0.0).unwrap()[1];
            (sound.pitch.0[0][1], last)
        };
        let (start, end) = pitch_at_end("Quis esas tu?");
        assert!(end > start, "{start} to {end}");
        let (start, end) = pitch_at_end("Hostis detectum.");
        assert!(end < start, "{start} to {end}");
    }

    #[test]
    fn a_line_is_spoken_in_about_the_time_it_should_take() {
        let voices = voices();
        for (name, voice) in &voices.voices {
            let sound = voices.speak(
                "Sta. Eso defendator codex quattuor septem. Quis esas?",
                voice,
                1.0,
            );
            assert!(
                sound.length > 1.5 && sound.length < 8.0,
                "{name}: {}",
                sound.length
            );
            let samples = synth::make(&sound, voices.sample_rate);
            assert!(samples.iter().all(|s| s.is_finite()), "{name}");
            let times = sound.loudness.0.windows(2).all(|w| w[0][0] <= w[1][0]);
            assert!(times, "{name}: the curves go forward in time");
        }
    }

    #[test]
    fn the_drones_voice_loads_and_is_heard() {
        let chirps = Chirps::load(DRONE_VOICE).expect("the drones' voice");
        let samples = synth::make(&chirps.say("Intrusor detectum. Sta.", 1.0), chirps.sample_rate);
        assert!(samples.iter().any(|s| s.abs() > 0.1), "silent");
    }
}
