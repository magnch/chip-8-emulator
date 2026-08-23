use rodio::{
    MixerDeviceSink, Player,
    source::{Source, SquareWave},
};

pub struct AudioPlayer {
    _stream: MixerDeviceSink,
    player: Player,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new(Self::DEFAULT_BUZZER_FREQ, Self::DEFAULT_VOLUME)
    }
}

impl AudioPlayer {
    const DEFAULT_BUZZER_FREQ: f32 = 440.0;
    const DEFAULT_VOLUME: f32 = 0.05;

    pub fn new(buzzer_freq: f32, volume: f32) -> Self {
        let stream =
            rodio::DeviceSinkBuilder::open_default_sink()
                .expect("open default audio stream");
        let player = Player::connect_new(stream.mixer());
        let source = SquareWave::new(buzzer_freq).amplify(volume);
        player.append(source);
        player.pause();

        Self {
            _stream: stream,
            player,
        }
    }

    pub fn set_playing(&mut self, should_play: bool) {
        if should_play {
            self.player.play();
        } else {
            self.player.pause();
        }
    }
}
