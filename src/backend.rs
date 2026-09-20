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
pub mod superchip;

pub use error::{BackendError, BackendErrorKind};
pub use instruction::Instruction;

pub use superchip::FONT_SIZE as MAX_FONT_SIZE;

pub const REGISTER_COUNT: usize = 16;
pub const TICK_RATE: f64 = 700.;
const TIMER_RATE: f64 = 60.;

pub enum Backend {
    Chip8(chip8::Backend),
    SuperChip(superchip::Backend),
}

pub struct BackendOptions {
    pub copy_and_shift: bool,
    pub increment_address: bool,
    pub quirky_jump: bool,
    pub reset_flag: bool,
}

pub enum ProgramState {
    Running,
    Halted,
    Exited,
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
    pub fn create_display_buffer(&self) -> interfaces::display_buffer::DisplayBuffer {
        match self {
            Self::Chip8(..) => interfaces::display_buffer::DisplayBuffer::new(
                [chip8::DISPLAY_BUFFER_WIDTH, chip8::DISPLAY_BUFFER_HEIGHT],
                interfaces::display_buffer::DisplayOptions {
                    clip_sprites: false,
                    half_pixel_scrolling: false,
                },
            ),
            Self::SuperChip(..) => interfaces::display_buffer::DisplayBuffer::new(
                [
                    superchip::DISPLAY_BUFFER_WIDTH,
                    superchip::DISPLAY_BUFFER_HEIGHT,
                ],
                interfaces::display_buffer::DisplayOptions {
                    clip_sprites: true,
                    half_pixel_scrolling: false,
                },
            ),
        }
    }

    pub fn load(&mut self, program: &[u8], font: Option<&[u8]>) -> Result<(), BackendError> {
        match font {
            Some(font) => match self {
                Self::Chip8(backend) => backend.load_with_font(program, font),
                Self::SuperChip(backend) => backend.load_with_font(program, font),
            },
            None => match self {
                Self::Chip8(backend) => backend.load(program),
                Self::SuperChip(backend) => backend.load(program),
            },
        }
    }

    pub fn options_mut(&mut self) -> &mut BackendOptions {
        match self {
            Self::Chip8(backend) => &mut backend.options,
            Self::SuperChip(backend) => backend.options_mut(),
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Chip8(backend) => backend.reset(),
            Self::SuperChip(backend) => backend.reset(),
        }
    }

    pub fn tick(
        &mut self,
        display_buffer: &mut interfaces::display_buffer::DisplayBuffer,
        keypad_state: &mut interfaces::keypad_state::KeypadState,
        persistent_storage: &mut [u8; REGISTER_COUNT],
    ) -> Result<ProgramState, BackendError> {
        match self {
            Self::Chip8(backend) => backend.tick(display_buffer, keypad_state),
            Self::SuperChip(backend) => {
                backend.tick(display_buffer, keypad_state, persistent_storage)
            }
        }
    }

    pub fn sound(&self) -> u8 {
        match self {
            Self::Chip8(backend) => backend.sound.get(),
            Self::SuperChip(backend) => backend.sound(),
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
        (self.value).saturating_sub((self.instant.elapsed().as_secs_f64() * TIMER_RATE) as u8)
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
