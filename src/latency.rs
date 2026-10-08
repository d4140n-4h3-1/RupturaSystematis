//! How late sound is heard after the game plays it: the engine's own buffer, the sound server's,
//! and, for Bluetooth earphones, the earphones' own, which can come to a quarter of a second. Sound
//! cannot be played early, so what is shown along with a sound waits for it instead: see the
//! subtitles in [`crate::game`].
//!
//! On Linux the sound server is asked, every few seconds so that earphones put on mid-game count,
//! with `pactl`, which PulseAudio and PipeWire both have. `MAZE_AUDIO_LATENCY_MS=<n>` sets it
//! outright instead, for earphones the guess below is wrong about.

// Elsewhere only the setting above is read: the parsing is for Linux, and its tests.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::platform;

/// How late the engine's own buffer plays a sound: two blocks of 512 samples at 44.1 kHz, or of
/// 2052 in a browser.
#[cfg(not(target_arch = "wasm32"))]
const ENGINE: f32 = 2.0 * 512.0 / 44100.0;
#[cfg(target_arch = "wasm32")]
const ENGINE: f32 = 2.0 * 2052.0 / 44100.0;

/// How often the sound server is asked again.
#[cfg(target_os = "linux")]
const ASK_EVERY: std::time::Duration = std::time::Duration::from_secs(5);

/// How late sound is heard, in seconds, kept up to date away from the game.
#[derive(Debug, Default, Clone)]
pub struct OutputLatency(Arc<AtomicU32>);

impl PartialEq for OutputLatency {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl OutputLatency {
    /// Starts keeping track of it.
    pub fn watch() -> Self {
        let latency = Self::default();
        latency.set(ENGINE);
        if let Some(ms) = platform::var("MAZE_AUDIO_LATENCY_MS").and_then(|ms| ms.parse::<f32>().ok()) {
            latency.set(ms / 1000.0);
            return latency;
        }
        #[cfg(target_os = "linux")]
        {
            let shared = latency.clone();
            let _ = std::thread::Builder::new()
                .name("output latency".into())
                .spawn(move || {
                    let mut said = None;
                    // Once the game has gone, nothing else holds the other end.
                    while Arc::strong_count(&shared.0) > 1 {
                        if let Some(output) = ask_pactl() {
                            shared.set(ENGINE + output.latency());
                            // Said once for each output, as how late it is wavers a little.
                            let which = (output.name.clone(), output.bluetooth.clone());
                            if said.as_ref() != Some(&which) {
                                fyrox::core::log::Log::info(format!(
                                    "Sound output: {}, heard {:.0} ms late",
                                    output.name,
                                    shared.seconds() * 1000.0
                                ));
                                said = Some(which);
                            }
                        }
                        std::thread::sleep(ASK_EVERY);
                    }
                });
        }
        latency
    }

    /// How late sound is heard now, in seconds.
    pub fn seconds(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    fn set(&self, seconds: f32) {
        self.0.store(seconds.to_bits(), Ordering::Relaxed);
    }
}

/// What the sound server says of where sound goes.
#[derive(Debug, Clone, PartialEq)]
struct Output {
    name: String,
    /// How late the server itself plays it, in seconds. While nothing plays it says none.
    server: f32,
    /// The Bluetooth codec, if it is Bluetooth.
    bluetooth: Option<String>,
    /// How late it is set to be heard on top, by hand, in seconds.
    offset: f32,
}

impl Output {
    /// How late sound is heard on it, the engine aside.
    fn latency(&self) -> f32 {
        // The server cannot know how long Bluetooth earphones hold sound before playing it. An
        // offset set by hand is the player's own measure of that, and is already in its figure.
        let earphones = match &self.bluetooth {
            Some(_) if self.offset > 0.0 => 0.0,
            Some(codec) => earphones(codec),
            None => 0.0,
        };
        self.server + earphones
    }
}

/// How long Bluetooth earphones typically hold sound on the codec called `codec`, in seconds:
/// low-latency aptX and FastStream little, other aptX less than SBC, AAC and LDAC.
fn earphones(codec: &str) -> f32 {
    let codec = codec.to_lowercase();
    if codec.contains("ll") || codec.contains("faststream") {
        0.02
    } else if codec.contains("aptx") {
        0.07
    } else {
        0.15
    }
}

#[cfg(target_os = "linux")]
fn ask_pactl() -> Option<Output> {
    let run = |args: &[&str]| {
        let output = std::process::Command::new("pactl").args(args).output().ok()?;
        output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned())
    };
    let default = run(&["get-default-sink"])?;
    let sinks = run(&["list", "sinks"])?;
    read_sinks(&sinks, default.trim())
}

/// The sink called `name` in what `pactl list sinks` printed.
fn read_sinks(list: &str, name: &str) -> Option<Output> {
    let sink = list
        .split("Sink #")
        .find(|sink| sink.lines().any(|line| line.trim() == format!("Name: {name}")))?;
    let field = |key: &str| {
        sink.lines()
            .find_map(|line| line.trim().strip_prefix(key).map(|rest| rest.trim().to_string()))
    };
    // "Latency: 53576 usec, configured 45317 usec"
    let server = field("Latency:")
        .and_then(|latency| latency.split_whitespace().next()?.parse::<f32>().ok())
        .map_or(0.0, |usec| usec / 1.0e6);
    let bluetooth = (field("device.api =").as_deref() == Some("\"bluez\""))
        .then(|| field("bluetooth.codec =").unwrap_or_default().trim_matches('"').to_string());
    // The active port's line under "Ports:" ends "latency offset: 0 usec, available)".
    let offset = field("Active Port:")
        .and_then(|port| {
            let line = sink.lines().find(|line| line.trim().starts_with(&format!("{port}:")))?;
            let (_, after) = line.split_once("latency offset:")?;
            after.split_whitespace().next()?.parse::<f32>().ok()
        })
        .map_or(0.0, |usec| usec / 1.0e6);
    Some(Output {
        name: field("Description:").unwrap_or_else(|| name.to_string()),
        server,
        bluetooth,
        offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINKS: &str = "Sink #0
	State: SUSPENDED
	Name: alsa_output.pci-0000_0f_00.4.analog-stereo
	Description: Built-in Audio Analog Stereo
	Latency: 0 usec, configured 0 usec
	Properties:
		device.api = \"alsa\"
	Ports:
		analog-output-headphones: Headphones (type: Headphones, priority: 9900, latency offset: 0 usec, available)
	Active Port: analog-output-headphones

Sink #2
	State: RUNNING
	Name: bluez_sink.15_FF_2F_F4_C1_09.a2dp_sink
	Description: Earbuds
	Latency: 53576 usec, configured 45317 usec
	Properties:
		bluetooth.codec = \"sbc\"
		device.api = \"bluez\"
	Ports:
		headset-output: Headset (type: Headset, priority: 0, latency offset: 0 usec, available)
	Active Port: headset-output
";

    #[test]
    fn wired_output_is_as_late_as_the_server_says() {
        let output = read_sinks(SINKS, "alsa_output.pci-0000_0f_00.4.analog-stereo").unwrap();
        assert_eq!(output.name, "Built-in Audio Analog Stereo");
        assert_eq!(output.bluetooth, None);
        assert_eq!(output.latency(), 0.0);
    }

    #[test]
    fn bluetooth_earphones_add_their_own_hold_for_the_codec() {
        let output = read_sinks(SINKS, "bluez_sink.15_FF_2F_F4_C1_09.a2dp_sink").unwrap();
        assert_eq!(output.bluetooth.as_deref(), Some("sbc"));
        assert!((output.latency() - (0.053576 + 0.15)).abs() < 1.0e-6);
    }

    #[test]
    fn an_offset_set_by_hand_replaces_the_guess() {
        let sinks = SINKS.replace("priority: 0, latency offset: 0 usec", "priority: 0, latency offset: 180000 usec");
        let output = read_sinks(&sinks, "bluez_sink.15_FF_2F_F4_C1_09.a2dp_sink").unwrap();
        assert_eq!(output.offset, 0.18);
        assert!((output.latency() - 0.053576).abs() < 1.0e-6);
    }

    #[test]
    fn low_latency_codecs_hold_less() {
        assert!(earphones("aptx_ll") < earphones("aptx"));
        assert!(earphones("aptx") < earphones("sbc"));
        assert!(earphones("faststream") < earphones("aac"));
    }

    #[test]
    fn an_unknown_sink_is_not_found() {
        assert_eq!(read_sinks(SINKS, "nothing"), None);
    }
}
