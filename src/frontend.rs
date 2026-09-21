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

use bitvec::view::BitViewSized;
use eframe::egui;
use web_time as time;

use crate::backend::{
    self,
    interfaces::{display_buffer, keypad_state},
};
use crate::defaults;

mod audio;
mod error;

pub use error::FrontendError;

const PAUSE_ICON: &[u8] = &[0x00, 0x50, 0x50, 0x00];

const PAUSE_ICON_WIDTH: usize = 5;

const PAUSE_ICON_TARGET_RESOLUTION: [usize; 2] = [
    backend::chip8::DISPLAY_BUFFER_WIDTH,
    backend::chip8::DISPLAY_BUFFER_HEIGHT,
];

#[derive(Clone, Copy)]
pub struct Colors {
    pub active: egui::Color32,
    pub inactive: egui::Color32,
}

pub struct Frontend {
    #[cfg(target_arch = "wasm32")]
    audio: audio::web::WebAudio,
    #[cfg(not(target_arch = "wasm32"))]
    audio: audio::rodio::RodioAudio,
    pub backend: backend::Backend,
    pub colors: Colors,
    pub display_buffer: display_buffer::DisplayBuffer,
    display_texture: egui::TextureHandle,
    pub keypad_state: keypad_state::KeypadState,
    last_tick: Option<time::Instant>,
    pub persistent_storage: [u8; backend::REGISTER_COUNT],
    suspended: bool,
}

impl Colors {
    fn get(&self, pixel: bool) -> egui::Color32 {
        match pixel {
            true => self.active,
            false => self.inactive,
        }
    }
}

impl Frontend {
    #[inline]
    pub fn display_texture(&self) -> egui::TextureId {
        self.display_texture.id()
    }

    pub fn new(
        ctx: &egui::Context,
        backend: backend::Backend,
        mut display_buffer: display_buffer::DisplayBuffer,
        persistent_storage: [u8; backend::REGISTER_COUNT],
    ) -> Result<Self, FrontendError> {
        #[cfg(target_arch = "wasm32")]
        let audio = audio::web::WebAudio::new().map_err(FrontendError::Audio)?;
        #[cfg(not(target_arch = "wasm32"))]
        let audio = audio::rodio::RodioAudio::new().map_err(FrontendError::Audio)?;

        let pixels: Vec<egui::Color32> = display_buffer
            .flattened()
            .map(|pixel| defaults::COLORS.get(pixel))
            .collect();

        Ok(Self {
            audio,
            backend,
            colors: defaults::COLORS,
            display_texture: ctx.load_texture(
                "Display Texture",
                egui::ColorImage::new(display_buffer.size(), pixels),
                egui::TextureOptions::default(),
            ),
            display_buffer,
            keypad_state: keypad_state::KeypadState::new(),
            last_tick: None,
            persistent_storage,
            suspended: false,
        })
    }

    pub fn reset(&mut self) {
        self.backend.reset(&mut self.display_buffer);
        self.suspend();
        self.update_texture();
    }

    pub fn suspend(&mut self) {
        self.audio.set_enabled(false);
        self.last_tick = None;
        self.suspended = true;
        self.update_texture();
    }

    pub fn tick(&mut self) -> Result<backend::ProgramState, FrontendError> {
        self.audio.set_enabled(self.backend.sound() > 0);
        let pending_ticks = (self
            .last_tick
            .map(|instant| instant.elapsed().as_secs_f64())
            .unwrap_or(1.)
            * backend::TICK_RATE) as u128;

        let mut program_state = Ok(backend::ProgramState::Running);

        for _ in 0..pending_ticks {
            match self.backend.tick(
                &mut self.display_buffer,
                &mut self.keypad_state,
                &mut self.persistent_storage,
            ) {
                Ok(backend::ProgramState::Running) => (),

                Ok(state) => {
                    program_state = Ok(state);
                    break;
                }

                Err(error) => {
                    program_state = Err(FrontendError::Backend(error));
                    break;
                }
            }
        }

        if pending_ticks > 0 {
            self.last_tick = Some(time::Instant::now());
        }

        if self.display_buffer.is_dirty() || self.suspended {
            self.suspended = false;
            self.update_texture();
        }

        program_state
    }

    pub fn update_texture(&mut self) {
        let mut pixels: Vec<egui::Color32> = self
            .display_buffer
            .flattened()
            .map(|pixel| self.colors.get(pixel))
            .collect();

        let size = self.display_buffer.size();

        if self.suspended {
            let scale = (size[0] / PAUSE_ICON_TARGET_RESOLUTION[0])
                .min(size[1] / PAUSE_ICON_TARGET_RESOLUTION[1])
                .max(1);

            for (y, bits) in PAUSE_ICON.iter().enumerate() {
                for (x, bit) in bits
                    .into_bitarray::<bitvec::order::Msb0>()
                    .iter()
                    .by_vals()
                    .take(PAUSE_ICON_WIDTH)
                    .enumerate()
                {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            pixels[(y * scale + dy) * size[0] + x * scale + dx] =
                                self.colors.get(bit);
                        }
                    }
                }
            }
        }

        self.display_texture.set(
            egui::ColorImage::new(size, pixels),
            egui::TextureOptions::NEAREST,
        );
    }
}
