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

use rodio::source;
use rodio::Source;

use crate::backend::{self, interfaces};
use crate::defaults;

mod error;

pub use backend::interfaces::DisplayOptions as Options;
pub use error::FrontendError;

const INSTRUCTIONS_PER_TICK: u8 = 28;
const BUZZ_FREQUENCY: f32 = 220.0;
const BUZZ_AMPLITUDE: f32 = 10.0;

#[repr(transparent)]
pub struct Beep {
    sine: source::SineWave,
}

#[derive(Clone, Copy)]
pub struct Colors {
    pub active: egui::Color32,
    pub inactive: egui::Color32,
}

pub struct Frontend {
    pub backend: backend::Backend,
    pub colors: Colors,
    pub display_buffer: interfaces::DisplayBuffer,
    display_texture: egui::TextureHandle,
    keypad_state: interfaces::KeypadState,
    sink: rodio::Sink,
    _stream: rodio::OutputStreamHandle,
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
        backend: backend::Backend,
        ctx: &egui::Context,
        options: Options,
        stream: rodio::OutputStreamHandle,
    ) -> Self {
        let sink = rodio::Sink::try_new(&stream)
            .map_err(FrontendError::Audio)
            .unwrap();
        sink.pause();
        sink.append(
            source::SineWave::new(BUZZ_FREQUENCY)
                .stoppable()
                .amplify(BUZZ_AMPLITUDE),
        );

        Self {
            colors: defaults::COLORS,
            backend,
            display_buffer: backend::interfaces::DisplayBuffer::new(options),
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
            keypad_state: interfaces::KeypadState::new(),
            sink,
            _stream: stream,
        }
    }

    pub fn reset(&mut self) {
        self.backend.reset();
        self.display_buffer.clear();
        self.sink.pause();
    }

    pub fn suspend(&self) {
        self.sink.pause()
    }

    pub fn tick(&mut self, ctx: &egui::Context) -> Result<(), FrontendError> {
        match self.backend.timers.sound {
            0 => self.sink.pause(),
            _ => self.sink.play(),
        }

        ctx.input(|input| {
            self.keypad_state.update(input);
        });

        match self.backend.tick(
            INSTRUCTIONS_PER_TICK,
            (&mut self.display_buffer, &mut self.keypad_state),
        ) {
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

    pub fn update_texture(&mut self) {
        let mut pixels: Vec<egui::Color32> =
            Vec::with_capacity(backend::DISPLAY_BUFFER_WIDTH * backend::DISPLAY_BUFFER_HEIGHT);

        for row in self.display_buffer.buffer.iter() {
            for pixel in row.iter() {
                pixels.push(self.colors.get(*pixel));
            }
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
    }
}
