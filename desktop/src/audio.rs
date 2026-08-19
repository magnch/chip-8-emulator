extern crate sdl2;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

struct SquareWave {
    phase: f32,
    phase_inc: f32,
    volume: f32,
}


impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            // Generate square wave for a buzzer sound
            *sample = if self.phase <= 0.5 { self.volume } else { -self.volume };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}


pub struct AudioPlayer {
    device: AudioDevice<SquareWave>,
    is_playing: bool, // tracks current state to avoid redundant resume()/pause() calls
}

impl AudioPlayer {
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Self, String> {
        let audio_subsystem = sdl_context.audio().expect("failed to init audio subsystem");

        let desired_spec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.05,
        })?;

        Ok(AudioPlayer { device, is_playing: false })
    }

    /// Called once per frame (or per timer tick) with chip8.is_beeping().
    /// Only touches the device when the state actually changes.
    pub fn update(&mut self, should_beep: bool) {
        if should_beep && !self.is_playing {
            self.device.resume();
            self.is_playing = true;
        } else if !should_beep && self.is_playing {
            self.device.pause();
            self.is_playing = false;
        }
    }
}