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
use std::ops::ControlFlow;

use crate::defaults;

use super::chip8;
use super::interfaces;
use super::BackendError;
use super::BackendErrorKind;
use super::Instruction;

pub const DISPLAY_BUFFER_ASPECT_RATIO: f32 = (DISPLAY_BUFFER_WIDTH / DISPLAY_BUFFER_HEIGHT) as f32;
pub const DISPLAY_BUFFER_HEIGHT: usize = 64;
pub const DISPLAY_BUFFER_WIDTH: usize = 128;
pub const FONT_SIZE: usize = chip8::FONT_SIZE + HIRES_FONT_SIZE;
pub const PERSISTENT_STORAGE_SIZE: usize = 8;

const HIRES_CHARACTER_COUNT: usize = 10; // 0-9
const HIRES_CHARACTER_SIZE: usize = 10;
const HIRES_FONT_SIZE: usize = HIRES_CHARACTER_SIZE * HIRES_CHARACTER_COUNT;

pub struct Backend {
    pub(super) display_buffer:
        interfaces::DisplayBuffer<DISPLAY_BUFFER_WIDTH, DISPLAY_BUFFER_HEIGHT>,
    inner: chip8::Backend,
    pub(super) program_exited: bool,
}

impl Backend {
    pub(super) fn execute(
        &mut self,
        index: usize,
        instruction: Instruction,
        keyboard_state: &mut interfaces::KeypadState,
        persistent_storage: &mut [u8],
    ) -> Result<ControlFlow<()>, BackendError> {
        match instruction.operator_code() {
            0x0 if instruction.operand_nnn() == 0x0E0 => self.display_buffer.clear(),

            0x0 if instruction.operand_nnn() == 0x0FD => {
                self.program_exited = true;
                return Ok(ControlFlow::Break(()));
            }

            0x0 if instruction.operand_nnn() == 0x0FE => self.display_buffer.half_resolution = true,
            0x0 if instruction.operand_nnn() == 0x0FF => {
                self.display_buffer.half_resolution = false
            }

            0xD => {
                let n = if !self.display_buffer.half_resolution && instruction.operand_n() == 0 {
                    32
                } else {
                    instruction.operand_n() as usize
                };

                if self.inner.registers.address + n >= self.inner.memory.len() {
                    return Err(BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: BackendErrorKind::MemoryOverflow,
                    });
                }

                let colliding_rows = self.display_buffer.draw(
                    (
                        self.inner.registers.general[instruction.operand_x()] as usize,
                        self.inner.registers.general[instruction.operand_y()] as usize,
                    ),
                    &self.inner.memory
                        [self.inner.registers.address..self.inner.registers.address + n],
                );

                self.inner.registers.general[15] = if self.display_buffer.half_resolution {
                    (colliding_rows > 0) as u8
                } else {
                    colliding_rows as u8
                }
            }

            0xF if instruction.operand_nn() == 0x29 => 'block: {
                let character_code = self.inner.registers.general[instruction.operand_x()] as usize;

                if character_code < super::KEY_COUNT {
                    self.inner.registers.address = character_code * chip8::CHARACTER_SIZE;
                    break 'block;
                }

                if character_code & 0x10 == 0 || character_code & 0xF >= HIRES_CHARACTER_COUNT {
                    return Err(BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: BackendErrorKind::UnrecognizedSprite,
                    });
                }

                self.inner.registers.address =
                    chip8::FONT_SIZE + (character_code & 0xF) * HIRES_CHARACTER_SIZE
            }

            0xF if instruction.operand_nn() == 0x30 => {
                let character_code = self.inner.registers.general[instruction.operand_x()] as usize;

                if character_code >= HIRES_CHARACTER_COUNT {
                    return Err(BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: BackendErrorKind::UnrecognizedSprite,
                    });
                }

                self.inner.registers.address =
                    chip8::FONT_SIZE + character_code * HIRES_CHARACTER_SIZE
            }

            0xF if instruction.operand_nn() == 0x75 => {
                let registers = &self.inner.registers.general
                    [..=instruction.operand_x().min(PERSISTENT_STORAGE_SIZE - 1)];
                persistent_storage.copy_from_slice(registers);
            }
            0xF if instruction.operand_nn() == 0x85 => {
                let registers = &mut self.inner.registers.general
                    [..=instruction.operand_x().min(PERSISTENT_STORAGE_SIZE - 1)];
                registers.copy_from_slice(&persistent_storage[..PERSISTENT_STORAGE_SIZE]);
            }

            _ => return self.inner.execute(index, instruction, keyboard_state),
        }

        Ok(ControlFlow::Continue(()))
    }

    pub fn load(&mut self, font: Option<&[u8]>, program: &[u8]) -> Result<(), super::BackendError> {
        let font = font.unwrap_or(&defaults::BACKEND_FONT);

        self.inner.load(Some(&font[..chip8::FONT_SIZE]), program)?;

        self.inner.memory[chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE].copy_from_slice(
            font.get(chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE)
                .unwrap_or(
                    &defaults::BACKEND_FONT[chip8::FONT_SIZE..chip8::FONT_SIZE + HIRES_FONT_SIZE],
                ),
        );

        Ok(())
    }

    pub fn new(options: super::Options, display_options: interfaces::DisplayOptions) -> Self {
        let mut display_buffer = interfaces::DisplayBuffer::new(display_options);
        display_buffer.half_resolution = true;

        Self {
            display_buffer,
            inner: chip8::Backend::new(options, None),
            program_exited: false,
        }
    }

    pub fn options_mut(&mut self) -> &mut super::Options {
        &mut self.inner.options
    }

    pub fn reset(&mut self) {
        self.program_exited = false;
        self.inner.reset();
    }

    pub fn tick(
        &mut self,
        n: u8,
        keyboard_state: &mut interfaces::KeypadState,
        persistent_storage: &mut [u8],
    ) -> Result<(), BackendError> {
        if !self.inner.loaded {
            return Err(BackendError {
                instruction: None,
                kind: BackendErrorKind::ProgramNotLoaded,
            });
        }

        self.inner.timers.delay = self.inner.timers.delay.saturating_sub(1);
        self.inner.timers.sound = self.inner.timers.sound.saturating_sub(1);

        for _ in 0..n {
            if self.inner.index + 1 >= self.inner.memory.len() {
                return Err(BackendError {
                    instruction: Some((self.inner.index, None)),
                    kind: BackendErrorKind::MemoryOverflow,
                });
            }

            let instruction = Instruction::new([
                self.inner.memory[self.inner.index],
                self.inner.memory[self.inner.index + 1],
            ]);

            let last_index = self.inner.index;
            self.inner.index += mem::size_of::<Instruction>();

            let control_flow =
                self.execute(last_index, instruction, keyboard_state, persistent_storage)?;

            if control_flow.is_break() {
                break;
            }
        }

        Ok(())
    }

    pub fn timers(&self) -> &super::Timers {
        &self.inner.timers
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new(
            super::Options {
                copy_and_shift: false,
                increment_address: false,
                quirky_jump: true,
                reset_flag: false,
            },
            interfaces::DisplayOptions { clip_sprites: true },
        )
    }
}
