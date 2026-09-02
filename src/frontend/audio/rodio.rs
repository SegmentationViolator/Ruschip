//    Ruschip - a multi-variant CHIP-8 emulator
//    Copyright (C) 2023 Segmentation Violator <segmentationviolator@proton.me>

//    This program is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.

//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.

//    You should have received a copy of the GNU General Public License
//    along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![cfg(not(target_arch = "wasm32"))]

use std::num;
use std::sync::{self, atomic};

struct Beep {
    sample_rate: num::NonZero<u32>,
    phase: f32,
    enabled: sync::Arc<atomic::AtomicBool>,
}

pub struct RodioAudio {
    _stream: rodio::MixerDeviceSink,
    _player: rodio::Player,
    enabled: sync::Arc<atomic::AtomicBool>,
    cached_enabled: bool,
}

impl Iterator for Beep {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let enabled = self.enabled.load(atomic::Ordering::Relaxed);

        let sample = if enabled {
            if self.phase < 0.5 {
                super::BEEP_AMPLITUDE
            } else {
                -super::BEEP_AMPLITUDE
            }
        } else {
            0.0
        };

        self.phase += super::BEEP_FREQUENCY / self.sample_rate.get() as f32;
        self.phase %= 1.0;

        Some(sample)
    }
}

impl rodio::Source for Beep {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> num::NonZero<u16> {
        num::NonZero::new(1).unwrap()
    }

    fn sample_rate(&self) -> num::NonZero<u32> {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl RodioAudio {
    pub fn new() -> Result<Self, rodio::DeviceSinkError> {
        let stream = rodio::DeviceSinkBuilder::open_default_sink()?;
        let player = rodio::Player::connect_new(stream.mixer());
        let enabled = sync::Arc::new(atomic::AtomicBool::new(false));

        let source = Beep {
            sample_rate: stream.config().sample_rate(),
            phase: 0.0,
            enabled: sync::Arc::clone(&enabled),
        };

        player.append(source);

        Ok(Self {
            _stream: stream,
            _player: player,
            enabled,
            cached_enabled: false,
        })
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if self.cached_enabled == enabled {
            return;
        }

        self.enabled.store(enabled, atomic::Ordering::Relaxed);
        self.cached_enabled = enabled;
    }
}
