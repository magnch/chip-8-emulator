//! CHIP-8 buzzer playback backed by `rodio`.
//!
//! The CHIP-8 sound timer is a simple on/off signal, so this plays a
//! continuous square wave and just toggles play/pause to match it.

use rodio::{
    MixerDeviceSink, Player,
    source::{Source, SquareWave},
};

/// A square-wave buzzer that can be started and stopped to match the CHIP-8 sound timer.
pub struct AudioPlayer {
    // Kept alive for as long as `player` needs the output device.
    _stream: MixerDeviceSink,
    player: Player,
}

impl Default for AudioPlayer {
    /// Create a player using the default buzzer frequency and volume.
    fn default() -> Self {
        Self::new(Self::DEFAULT_BUZZER_FREQ, Self::DEFAULT_VOLUME)
    }
}

impl AudioPlayer {
    const DEFAULT_BUZZER_FREQ: f32 = 440.0;
    const DEFAULT_VOLUME: f32 = 0.05;

    /// Open the default output device and prepare (but do not start) the buzzer tone.
    pub fn new(buzzer_freq: f32, volume: f32) -> Self {
        let stream =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = Player::connect_new(stream.mixer());
        let source = SquareWave::new(buzzer_freq).amplify(volume);
        player.append(source);
        player.pause();

        Self {
            _stream: stream,
            player,
        }
    }

    /// Start or stop the buzzer tone.
    pub fn set_playing(&mut self, should_play: bool) {
        if should_play {
            self.player.play();
        } else {
            self.player.pause();
        }
    }
}
