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

use eframe::egui;

use crate::backend::{self, interfaces::{display_buffer, keypad_state}};
use crate::defaults;

mod audio;
mod error;

pub use error::FrontendError;

#[derive(Clone, Copy)]
pub struct Colors {
    pub active: egui::Color32,
    pub inactive: egui::Color32,
}

pub struct Frontend {
    audio: audio::RodioAudio,
    pub backend: backend::Backend,
    pub colors: Colors,
    pub display_buffer: display_buffer::DisplayBuffer,
    display_texture: egui::TextureHandle,
    pub keypad_state: keypad_state::KeypadState,
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
    ) -> Result<Self, FrontendError> {
        let audio = audio::RodioAudio::new()
            .map_err(FrontendError::Audio)?;

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
        })
    }

    pub fn reset(&mut self) {
        self.backend.reset();
        self.display_buffer.clear();
        self.audio.set_enabled(false);
    }

    pub fn suspend(&mut self) {
        self.audio.set_enabled(false);
    }

    pub fn tick(
        &mut self,
    ) -> Result<(), FrontendError> {
        self.audio.set_enabled(self.backend.sound() > 0);

        match self.backend.tick(
            &mut self.display_buffer,
            &mut self.keypad_state,
        ) {
            Ok(_) => (),
            Err(error) => {
                return Err(FrontendError::Backend(error));
            }
        }

        if self.display_buffer.is_dirty() {
            self.update_texture()?;
        }

        Ok(())
    }

    pub fn update_texture(&mut self) -> Result<(), FrontendError> {
        let pixels: Vec<egui::Color32> = self
            .display_buffer
            .flattened()
            .map(|pixel| self.colors.get(pixel))
            .collect();

        self.display_texture.set(
            egui::ColorImage::new(
                self.display_buffer.size(),
                pixels,
            ),
            egui::TextureOptions::NEAREST,
        );

        Ok(())
    }
}
