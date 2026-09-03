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

use web_time as time;

pub mod chip8;
mod error;
mod instruction;
pub mod interfaces;

pub use error::{BackendError, BackendErrorKind};
pub use instruction::Instruction;

pub use chip8::FONT_SIZE as MAX_FONT_SIZE;

const TIMER_RATE: u128 = 1000 / 60;

pub enum Backend {
    Chip8(chip8::Backend),
}

pub struct BackendOptions {
    pub copy_and_shift: bool,
    pub increment_address: bool,
    pub quirky_jump: bool,
    pub reset_flag: bool,
}

pub struct Timer {
    instant: time::Instant,
    value: u8,
}

impl Default for Backend {
    fn default() -> Self {
        Self::Chip8(Default::default())
    }
}

impl Backend {
    pub fn default_display_buffer(&self) -> interfaces::display_buffer::DisplayBuffer {
        match self {
            Self::Chip8(..) => interfaces::display_buffer::DisplayBuffer::new(
                [chip8::DISPLAY_BUFFER_WIDTH, chip8::DISPLAY_BUFFER_HEIGHT],
                interfaces::display_buffer::DisplayOptions {
                    clip_sprites: false,
                    half_pixel_scrolling: false,
                },
            ),
        }
    }

    pub fn load(&mut self, program: &[u8], font: Option<&[u8]>) -> Result<(), BackendError> {
        match font {
            Some(font) => match self {
                Self::Chip8(backend) => backend.load_with_font(program, font),
            },
            None => match self {
                Self::Chip8(backend) => backend.load(program),
            },
        }
    }

    pub fn options_mut(&mut self) -> &mut BackendOptions {
        match self {
            Self::Chip8(backend) => &mut backend.options,
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Chip8(backend) => backend.reset(),
        }
    }

    pub fn tick(
        &mut self,
        display_buffer: &mut interfaces::display_buffer::DisplayBuffer,
        keypad_state: &mut interfaces::keypad_state::KeypadState,
    ) -> Result<(), BackendError> {
        match self {
            Self::Chip8(backend) => backend.tick(display_buffer, keypad_state),
        }
    }

    pub fn sound(&self) -> u8 {
        match self {
            Self::Chip8(backend) => backend.sound.get(),
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn get(&self) -> u8 {
        (self.value as u128).saturating_sub(self.instant.elapsed().as_millis() / TIMER_RATE) as u8
    }

    pub fn new() -> Self {
        Self {
            instant: time::Instant::now(),
            value: 0,
        }
    }

    pub fn set(&mut self, value: u8) {
        self.instant = time::Instant::now();
        self.value = value;
    }
}
