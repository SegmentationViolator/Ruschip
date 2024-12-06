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

pub mod chip8;
mod error;
mod instruction;
pub mod interfaces;
pub mod super_chip;

pub use error::{BackendError, BackendErrorKind};
pub use instruction::Instruction;

pub const KEY_COUNT: usize = 16; // 0-F
pub const MAX_FONT_SIZE: usize = super_chip::FONT_SIZE;
pub const MIN_FONT_SIZE: usize = chip8::FONT_SIZE;

pub enum Backend {
    Chip8(chip8::Backend),
    SuperChip(super_chip::Backend),
}

pub struct Options {
    pub copy_and_shift: bool,
    pub increment_address: bool,
    pub quirky_jump: bool,
    pub reset_flag: bool,
}

pub struct Timers {
    delay: u8,
    pub sound: u8,
}

impl Backend {
    pub fn display_buffer(&mut self) -> Result<Vec<bool>, BackendError> {
        match self {
            Self::Chip8(backend) => backend
                .display_buffer
                .as_mut()
                .and_then(|buffer| Some(buffer.get()))
                .ok_or(BackendError {
                    kind: BackendErrorKind::DisplayNotConnected,
                    instruction: None,
                }),
            Self::SuperChip(backend) => Ok(backend.display_buffer.get()),
        }
    }

    pub fn display_buffer_aspect_ratio(&self) -> f32 {
        match self {
            Self::Chip8(..) => chip8::DISPLAY_BUFFER_ASPECT_RATIO,
            Self::SuperChip(..) => super_chip::DISPLAY_BUFFER_ASPECT_RATIO,
        }
    }

    pub fn display_buffer_size(&self) -> [usize; 2] {
        match self {
            Self::Chip8(..) => [chip8::DISPLAY_BUFFER_WIDTH, chip8::DISPLAY_BUFFER_HEIGHT],
            Self::SuperChip(..) => [
                super_chip::DISPLAY_BUFFER_WIDTH,
                super_chip::DISPLAY_BUFFER_HEIGHT,
            ],
        }
    }

    pub fn display_options_mut(&mut self) -> &mut interfaces::DisplayOptions {
        match self {
            Self::Chip8(backend) => {
                &mut backend
                    .display_buffer
                    .as_mut()
                    .expect("Display must be connected")
                    .options
            }
            Self::SuperChip(backend) => &mut backend.display_buffer.options,
        }
    }

    pub fn is_display_buffer_dirty(&mut self) -> bool {
        match self {
            Self::Chip8(backend) => backend
                .display_buffer
                .as_mut()
                .expect("Display must be connected")
                .is_dirty(),
            Self::SuperChip(backend) => backend.display_buffer.is_dirty(),
        }
    }

    pub fn load(&mut self, font: Option<&[u8]>, program: &[u8]) -> Result<(), BackendError> {
        match self {
            Self::Chip8(backend) => backend.load(font, program),
            Self::SuperChip(backend) => backend.load(font, program),
        }
    }

    pub fn options_mut(&mut self) -> &mut Options {
        match self {
            Self::Chip8(backend) => &mut backend.options,
            Self::SuperChip(backend) => backend.options_mut(),
        }
    }

    pub fn program_exited(&self) -> bool {
        match self {
            Self::Chip8(..) => false,
            Self::SuperChip(backend) => backend.program_exited,
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Chip8(backend) => {
                backend.reset();
                backend
                    .display_buffer
                    .as_mut()
                    .expect("Display must be connected")
                    .clear();
            }
            Self::SuperChip(backend) => {
                backend.reset();
                backend.display_buffer.clear();
            }
        }
    }

    pub fn tick(
        &mut self,
        n: u8,
        keyboard_state: &mut interfaces::KeypadState,
    ) -> Result<(), BackendError> {
        match self {
            Self::Chip8(backend) => backend.tick(n, keyboard_state),
            Self::SuperChip(backend) => backend.tick(n, keyboard_state),
        }
    }

    pub fn timers(&self) -> &Timers {
        match self {
            Self::Chip8(backend) => &backend.timers,
            Self::SuperChip(backend) => backend.timers(),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::Chip8(Default::default())
    }
}
