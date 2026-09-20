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

use std::mem;

use crate::defaults;

use super::chip8;
use super::interfaces::{display_buffer, keypad_state};

pub const DISPLAY_BUFFER_ASPECT_RATIO: f32 = (DISPLAY_BUFFER_WIDTH / DISPLAY_BUFFER_HEIGHT) as f32;
pub const DISPLAY_BUFFER_HEIGHT: usize = 64;
pub const DISPLAY_BUFFER_WIDTH: usize = 128;
pub const FONT_SIZE: usize = chip8::FONT_SIZE + HIRES_FONT_SIZE;

const HIRES_CHARACTER_COUNT: usize = 10; // 0-9
const HIRES_CHARACTER_SIZE: usize = 10;
const HIRES_FONT_SIZE: usize = HIRES_CHARACTER_SIZE * HIRES_CHARACTER_COUNT;

pub struct Backend {
    inner: chip8::Backend,
}

impl Backend {
    pub(super) fn execute(
        &mut self,
        index: usize,
        instruction: super::Instruction,
        display_buffer: &mut display_buffer::DisplayBuffer,
        keypad_state: &mut keypad_state::KeypadState,
        persistent_storage: &mut [u8],
    ) -> Result<super::ProgramState, super::BackendError> {
        match instruction.operator_code() {
            0x0 if instruction.operand_xy() == 0x0B => {
                display_buffer.scroll_up(instruction.operand_n() as usize)
            }
            0x0 if instruction.operand_xy() == 0x0C => {
                display_buffer.scroll_down(instruction.operand_n() as usize)
            }

            0x0 if instruction.operand_nnn() == 0x0E0 => display_buffer.clear(),

            0x0 if instruction.operand_nnn() == 0x0FB => display_buffer.scroll_right(4),
            0x0 if instruction.operand_nnn() == 0x0FC => display_buffer.scroll_left(4),

            0x0 if instruction.operand_nnn() == 0x0FD => {
                return Ok(super::ProgramState::Exited);
            }

            0x0 if instruction.operand_nnn() == 0x0FE => {
                display_buffer.halve_resolution = true;
                display_buffer.clear();
            }
            0x0 if instruction.operand_nnn() == 0x0FF => {
                display_buffer.halve_resolution = false;
                display_buffer.clear();
            }

            0xD => {
                let n = if instruction.operand_n() == 0 {
                    32
                } else {
                    instruction.operand_n() as usize
                };

                if self.inner.registers.address as usize + n > self.inner.memory.len() {
                    return Err(super::BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: super::BackendErrorKind::MemoryOverflow,
                    });
                }

                let colliding_rows = display_buffer.draw(
                    (
                        self.inner.registers.general[instruction.operand_x() as usize] as usize,
                        self.inner.registers.general[instruction.operand_y() as usize] as usize,
                    ),
                    &self.inner.memory[self.inner.registers.address as usize
                        ..self.inner.registers.address as usize + n],
                );

                self.inner.registers.general[15] = if display_buffer.halve_resolution {
                    (colliding_rows > 0) as u8
                } else {
                    colliding_rows as u8
                }
            }

            0xF if instruction.operand_nn() == 0x29 => 'block: {
                let character_code =
                    self.inner.registers.general[instruction.operand_x() as usize] as usize;

                if character_code < keypad_state::KEY_COUNT {
                    self.inner.registers.address = (character_code * chip8::CHARACTER_SIZE) as u16;
                    break 'block;
                }

                if character_code & 0x10 == 0 || character_code & 0xF >= HIRES_CHARACTER_COUNT {
                    return Err(super::BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: super::BackendErrorKind::UnrecognizedSprite,
                    });
                }

                self.inner.registers.address =
                    (chip8::FONT_SIZE + (character_code & 0xF) * HIRES_CHARACTER_SIZE) as u16
            }

            0xF if instruction.operand_nn() == 0x30 => {
                let character_code =
                    self.inner.registers.general[instruction.operand_x() as usize] as usize;

                if character_code >= HIRES_CHARACTER_COUNT {
                    return Err(super::BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: super::BackendErrorKind::UnrecognizedSprite,
                    });
                }

                self.inner.registers.address =
                    (chip8::FONT_SIZE + (character_code & 0xF) * HIRES_CHARACTER_SIZE) as u16
            }

            0xF if instruction.operand_nn() == 0x75 => {
                let registers = &self.inner.registers.general[..=instruction.operand_x() as usize];
                persistent_storage.copy_from_slice(registers);
            }

            0xF if instruction.operand_nn() == 0x85 => {
                let registers =
                    &mut self.inner.registers.general[..=instruction.operand_x() as usize];
                registers.copy_from_slice(&persistent_storage);
            }

            _ => {
                return self
                    .inner
                    .execute(index, instruction, display_buffer, keypad_state);
            }
        }

        Ok(super::ProgramState::Running)
    }

    pub fn load(&mut self, program: &[u8]) -> Result<(), super::BackendError> {
        let font = &defaults::BACKEND_FONT;

        self.inner.load(program)?;
        self.inner.memory[chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE].copy_from_slice(
            font.get(chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE)
                .unwrap_or(
                    &defaults::BACKEND_FONT[chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE],
                ),
        );

        Ok(())
    }

    pub fn load_with_font(
        &mut self,
        font: &[u8],
        program: &[u8],
    ) -> Result<(), super::BackendError> {
        self.inner
            .load_with_font(&font[..chip8::FONT_SIZE], program)?;
        self.inner.memory[chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE].copy_from_slice(
            font.get(chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE)
                .unwrap_or(
                    &defaults::BACKEND_FONT[chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE],
                ),
        );

        Ok(())
    }

    pub fn new(options: super::BackendOptions) -> Self {
        Self {
            inner: chip8::Backend::new(options),
        }
    }

    #[inline]
    pub fn options_mut(&mut self) -> &mut super::BackendOptions {
        &mut self.inner.options
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    #[inline]
    pub fn sound(&self) -> u8 {
        self.inner.sound.get()
    }

    pub fn tick(
        &mut self,
        display_buffer: &mut display_buffer::DisplayBuffer,
        keypad_state: &mut keypad_state::KeypadState,
        persistent_storage: &mut [u8],
    ) -> Result<super::ProgramState, super::BackendError> {
        if !self.inner.loaded {
            return Err(super::BackendError {
                instruction: None,
                kind: super::BackendErrorKind::ProgramNotLoaded,
            });
        }

        if self.inner.index + 1 >= self.inner.memory.len() {
            return Err(super::BackendError {
                instruction: Some((self.inner.index, None)),
                kind: super::BackendErrorKind::MemoryOverflow,
            });
        }

        let instruction = super::Instruction::new([
            self.inner.memory[self.inner.index],
            self.inner.memory[self.inner.index + 1],
        ]);

        let last_index = self.inner.index;
        self.inner.index += mem::size_of::<super::Instruction>();

        self.execute(
            last_index,
            instruction,
            display_buffer,
            keypad_state,
            persistent_storage,
        )
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new(super::BackendOptions {
            copy_and_shift: false,
            increment_address: false,
            quirky_jump: true,
            reset_flag: false,
        })
    }
}
