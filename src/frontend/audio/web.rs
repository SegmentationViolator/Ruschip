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

#![cfg(target_arch = "wasm32")]

use std::sync;

use wasm_bindgen::prelude::*;

pub struct WebAudio {
    _context: web_sys::AudioContext,
    enabled: bool,
    gain: web_sys::GainNode,
    once_lock: sync::OnceLock<()>,
    oscillator: web_sys::OscillatorNode,
}

impl WebAudio {
    pub fn new() -> Result<Self, JsValue> {
        let context = web_sys::AudioContext::new()?;

        let oscillator = context.create_oscillator()?;
        let gain = context.create_gain()?;

        oscillator.set_type(web_sys::OscillatorType::Square);
        oscillator.frequency().set_value(super::BEEP_FREQUENCY);
        gain.gain().set_value(0.0);

        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&context.destination())?;

        Ok(Self {
            _context: context,
            enabled: false,
            gain,
            once_lock: sync::OnceLock::new(),
            oscillator,
        })
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.once_lock.get_or_init(|| {
            if let Err(e) = self.oscillator.start() {
                web_sys::console::error_1(&e);
            }
        });

        if self.enabled == enabled {
            return;
        }

        self.enabled = enabled;
        self.gain.gain().set_value(if enabled { super::BEEP_AMPLITUDE } else { 0.0 });
    }
}
