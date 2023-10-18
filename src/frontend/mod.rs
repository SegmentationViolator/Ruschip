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

use std::num;

use crate::backend::{self, interfaces};
use crate::defaults;

mod error;
mod sound;

pub use error::FrontendError;
pub use sound::Sound;

const INSTRUCTIONS_PER_TICK: u16 = 12;

#[derive(Clone, Copy)]
pub struct Colors {
    pub active: egui::Color32,
    pub inactive: egui::Color32,
}

pub struct Frontend {
    pub backend: backend::Backend,
    pub colors: Colors,
    context: egui::Context,
    display_buffer: interfaces::DisplayBuffer,
    display_texture: egui::TextureHandle,
    keyboard: interfaces::KeyboardState,
    pub options: Options,
    sound: Sound,
    stream: rodio::OutputStreamHandle,
}

#[derive(Default)]
pub struct Options {
    pub wrap_sprites: bool,
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

    #[inline]
    pub fn new(ctx: &egui::Context, options: Options, stream: rodio::OutputStreamHandle) -> Self {
        Self {
            colors: defaults::COLORS,
            context: ctx.clone(),
            backend: backend::Backend::new(),
            display_buffer: backend::interfaces::DisplayBuffer::new(interfaces::DisplayOptions {
                wrap_sprites: options.wrap_sprites,
            }),
            display_texture: ctx.load_texture(
                "Display Texture",
                egui::ColorImage::new(
                    [
                        backend::DISPLAY_BUFFER_WIDTH,
                        backend::DISPLAY_BUFFER_HEIGHT,
                    ],
                    defaults::COLORS.inactive,
                ),
                egui::TextureOptions::default(),
            ),
            keyboard: interfaces::KeyboardState::new(),
            options,
            sound: Sound::new().unwrap(),
            stream,
        }
    }

    pub fn reset(&mut self) {
        self.backend.reset();
        self.display_buffer.clear();
        self.keyboard.release();
    }

    pub fn tick(&mut self) -> Result<(), FrontendError> {
        let n = num::NonZeroU16::new(INSTRUCTIONS_PER_TICK).unwrap();

        let sink = match rodio::Sink::try_new(&self.stream) {
            Ok(sink) => sink,
            Err(error) => return Err(FrontendError::Audio(error)),
        };

        if self.backend.timers.sound > 0 {
            self.sound.play(&sink)
        }

        match self
            .backend
            .tick(n, (&mut self.display_buffer, &mut self.keyboard))
        {
            Ok(_) => (),
            Err(error) => {
                return Err(FrontendError::Backend(error));
            }
        }

        if self.display_buffer.dirty {
            self.display_buffer.dirty = false;

            self.update_texture();
        }

        Ok(())
    }

    pub fn update_keyboard_state(&mut self, state: Vec<usize>) {
        for key in state {
            self.keyboard.set(key, true);
        }
    }

    pub fn update_texture(&mut self) {
        let mut pixels: Vec<egui::Color32> =
            Vec::with_capacity(backend::DISPLAY_BUFFER_WIDTH * backend::DISPLAY_BUFFER_HEIGHT);

        for pixel in self.display_buffer.buffer.iter().flatten() {
            pixels.push(self.colors.get(*pixel));
        }

        self.display_texture.set(
            egui::ColorImage {
                size: [
                    backend::DISPLAY_BUFFER_WIDTH,
                    backend::DISPLAY_BUFFER_HEIGHT,
                ],
                pixels,
            },
            egui::TextureOptions::NEAREST,
        );

        self.context.request_repaint();
    }
}
