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
use std::ops;

use crate::defaults;

use super::interfaces::{display_buffer, keypad_state};
use super::BackendError;
use super::BackendErrorKind;
use super::Instruction;

pub const DISPLAY_BUFFER_HEIGHT: usize = 32;
pub const DISPLAY_BUFFER_WIDTH: usize = 64;
pub const FONT_SIZE: usize = CHARACTER_SIZE * keypad_state::KEY_COUNT;
pub const TICK_RATE: usize = 15;

pub(super) const CHARACTER_SIZE: usize = 5;
const MEMORY_PADDING: usize = 512;
const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;

pub struct Backend {
    pub(super) index: usize,
    pub(super) loaded: bool,
    pub(super) memory: [u8; MEMORY_SIZE],
    pub options: super::BackendOptions,
    pub(super) registers: Registers,
    pub(super) stack: Vec<u16>,
    pub(super) delay: super::Timer,
    pub(super) sound: super::Timer,
}

pub(super) struct Registers {
    pub address: usize,
    pub general: [u8; REGISTER_COUNT],
}

impl Backend {
    pub(super) fn execute(
        &mut self,
        index: usize,
        instruction: Instruction,
        display_buffer: &mut display_buffer::DisplayBuffer,
        keypad_state: &mut keypad_state::KeypadState,
    ) -> Result<ops::ControlFlow<()>, BackendError> {
        match instruction.operator_code() {
            0x0 => match instruction.operand_nnn() {
                0x0E0 => {
                    display_buffer.clear();
                }

                0x0EE => {
                    match self.stack.pop() {
                        None => {
                            return Err(BackendError {
                                instruction: Some((index, Some(instruction))),
                                kind: BackendErrorKind::StackUnderflow,
                            })
                        }
                        Some(address) => self.index = address as usize,
                    };
                }

                // Not implementing 0NNN, needs a 1802 or M6800 VM.
                _ => {}
            },

            opcode @ (0x1 | 0x2) => {
                if opcode == 2 {
                    if self.stack.len() == STACK_SIZE {
                        return Err(BackendError {
                            instruction: Some((index, Some(instruction))),
                            kind: BackendErrorKind::StackOverflow,
                        });
                    }

                    self.stack.push(self.index as u16);
                }

                self.index = instruction.operand_nnn();
            }

            opcode @ (0x3 | 0x4 | 0x5 | 0x9) => {
                match opcode {
                    0x3 if self.registers.general[instruction.operand_x()]
                        == instruction.operand_nn() => {}

                    0x4 if self.registers.general[instruction.operand_x()]
                        != instruction.operand_nn() => {}

                    0x5 if self.registers.general[instruction.operand_x()]
                        == self.registers.general[instruction.operand_y()] => {}

                    0x9 if self.registers.general[instruction.operand_x()]
                        != self.registers.general[instruction.operand_y()] => {}

                    _ => return Ok(ops::ControlFlow::Continue(())),
                }

                self.index += mem::size_of::<Instruction>();
            }

            0x6 => self.registers.general[instruction.operand_x()] = instruction.operand_nn(),

            0x7 => {
                self.registers.general[instruction.operand_x()] = self.registers.general
                    [instruction.operand_x()]
                .wrapping_add(instruction.operand_nn())
            }

            0x8 => match instruction.operand_n() {
                0x0 => {
                    self.registers.general[instruction.operand_x()] =
                        self.registers.general[instruction.operand_y()]
                }

                0x1 => {
                    self.registers.general[instruction.operand_x()] |=
                        self.registers.general[instruction.operand_y()];

                    if self.options.reset_flag {
                        self.registers.general[15] = 0;
                    }
                }

                0x2 => {
                    self.registers.general[instruction.operand_x()] &=
                        self.registers.general[instruction.operand_y()];

                    if self.options.reset_flag {
                        self.registers.general[15] = 0;
                    }
                }

                0x3 => {
                    self.registers.general[instruction.operand_x()] ^=
                        self.registers.general[instruction.operand_y()];

                    if self.options.reset_flag {
                        self.registers.general[15] = 0;
                    }
                }

                0x4 => {
                    let (result, flag) = self.registers.general[instruction.operand_x()]
                        .overflowing_add(self.registers.general[instruction.operand_y()]);

                    self.registers.general[instruction.operand_x()] = result;
                    self.registers.general[15] = flag as u8;
                }

                code @ (0x5 | 0x7) => {
                    let flag;
                    let result;

                    match code {
                        0x5 => {
                            result = self.registers.general[instruction.operand_x()]
                                .wrapping_sub(self.registers.general[instruction.operand_y()]);
                            flag = self.registers.general[instruction.operand_x()]
                                >= self.registers.general[instruction.operand_y()];
                        }

                        0x7 => {
                            result = self.registers.general[instruction.operand_y()]
                                .wrapping_sub(self.registers.general[instruction.operand_x()]);
                            flag = self.registers.general[instruction.operand_y()]
                                >= self.registers.general[instruction.operand_x()];
                        }

                        _ => unreachable!(),
                    }

                    self.registers.general[instruction.operand_x()] = result;
                    self.registers.general[15] = flag as u8;
                }

                code @ (0x6 | 0xE) => {
                    let flag;
                    let result;

                    if self.options.copy_and_shift {
                        self.registers.general[instruction.operand_x()] =
                            self.registers.general[instruction.operand_y()]
                    }

                    match code {
                        0x6 => {
                            result = self.registers.general[instruction.operand_x()] >> 1;
                            flag = self.registers.general[instruction.operand_x()] & 1;
                        }
                        0xE => {
                            result = self.registers.general[instruction.operand_x()] << 1;
                            flag = self.registers.general[instruction.operand_x()]
                                >> (u8::BITS - 1) as u8;
                        }
                        _ => unreachable!(),
                    }

                    self.registers.general[instruction.operand_x()] = result;
                    self.registers.general[15] = flag;
                }

                _ => {
                    return Err(BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: BackendErrorKind::UnrecognizedInstruction,
                    })
                }
            },

            0xA => self.registers.address = instruction.operand_nnn(),

            0xB => {
                self.index = self.registers.general
                    [[0, instruction.operand_x()][self.options.quirky_jump as usize]]
                    as usize
                    + instruction.operand_nnn()
            }

            0xC => {
                self.registers.general[instruction.operand_x()] =
                    rand::random::<u8>() & instruction.operand_nn();
            }

            0xD => {
                if self.registers.address + instruction.operand_n() as usize >= self.memory.len() {
                    return Err(BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: BackendErrorKind::MemoryOverflow,
                    });
                }

                let colliding_rows = display_buffer.draw(
                    (
                        self.registers.general[instruction.operand_x()] as usize,
                        self.registers.general[instruction.operand_y()] as usize,
                    ),
                    &self.memory[self.registers.address
                        ..self.registers.address + instruction.operand_n() as usize],
                );

                self.registers.general[15] = (colliding_rows > 0) as u8;

                return Ok(ops::ControlFlow::Break(()));
            }

            0xE => match instruction.operand_nn() {
                0x9E => {
                    let key = self.registers.general[instruction.operand_x()] as usize;
                    if key >= keypad_state::KEY_COUNT {
                        return Err(BackendError {
                            instruction: Some((index, Some(instruction))),
                            kind: BackendErrorKind::UnrecognizedKey,
                        });
                    }

                    if keypad_state.pressed(key) {
                        self.index += mem::size_of::<Instruction>();
                    }
                }

                0xA1 => {
                    let key = self.registers.general[instruction.operand_x()] as usize;
                    if key >= keypad_state::KEY_COUNT {
                        return Err(BackendError {
                            instruction: Some((index, Some(instruction))),
                            kind: BackendErrorKind::UnrecognizedKey,
                        });
                    }

                    if !keypad_state.pressed(key) {
                        self.index += mem::size_of::<Instruction>();
                    }
                }

                _ => {
                    return Err(BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: BackendErrorKind::UnrecognizedInstruction,
                    })
                }
            },

            0xF => match instruction.operand_nn() {
                0x07 => self.registers.general[instruction.operand_x()] = self.delay.get(),

                0x0A => {
                    match keypad_state.pressed_key() {
                        Some(key) => {
                            self.registers.general[instruction.operand_x()] = key as u8;
                        }
                        None => {
                            self.index = index;
                        }
                    }

                    return Ok(ops::ControlFlow::Break(()));
                }

                0x15 => self.delay.set(self.registers.general[instruction.operand_x()]),
                0x18 => self.sound.set(self.registers.general[instruction.operand_x()]),

                0x1E => {
                    self.registers.address = (self.registers.address
                        + self.registers.general[instruction.operand_x()] as usize)
                        & 0xFFF
                }

                0x29 => {
                    let character_code = self.registers.general[instruction.operand_x()] as usize;

                    if character_code >= keypad_state::KEY_COUNT {
                        return Err(BackendError {
                            instruction: Some((index, Some(instruction))),
                            kind: BackendErrorKind::UnrecognizedSprite,
                        });
                    }

                    self.registers.address = character_code * CHARACTER_SIZE;
                }

                0x33 => {
                    if self.registers.address + 2 >= self.memory.len() {
                        return Err(BackendError {
                            instruction: Some((index, Some(instruction))),
                            kind: BackendErrorKind::MemoryOverflow,
                        });
                    }

                    let number = self.registers.general[instruction.operand_x()];

                    self.memory[self.registers.address] = (number / 10) / 10;
                    self.memory[self.registers.address + 1] = (number / 10) % 10;
                    self.memory[self.registers.address + 2] = number % 10;
                }

                0x55 => {
                    let x = instruction.operand_x();

                    if self.registers.address + x >= self.memory.len() {
                        return Err(BackendError {
                            instruction: Some((index, Some(instruction))),
                            kind: BackendErrorKind::MemoryOverflow,
                        });
                    }

                    for i in 0..x + 1 {
                        self.memory[self.registers.address + i] = self.registers.general[i];
                    }

                    if self.options.increment_address {
                        self.registers.address += x + 1;
                    }
                }

                0x65 => {
                    let x = instruction.operand_x();

                    if self.registers.address + x >= self.memory.len() {
                        return Err(BackendError {
                            instruction: Some((self.index, Some(instruction))),
                            kind: BackendErrorKind::MemoryOverflow,
                        });
                    }

                    for i in 0..x + 1 {
                        self.registers.general[i] = self.memory[self.registers.address + i];
                    }

                    if self.options.increment_address {
                        self.registers.address += x + 1;
                    }
                }

                _ => {
                    return Err(BackendError {
                        instruction: Some((index, Some(instruction))),
                        kind: BackendErrorKind::UnrecognizedInstruction,
                    })
                }
            },

            _ => {
                return Err(BackendError {
                    instruction: Some((index, Some(instruction))),
                    kind: BackendErrorKind::UnrecognizedInstruction,
                })
            }
        }

        Ok(ops::ControlFlow::Continue(()))
    }

    pub fn load(&mut self, font: Option<&[u8]>, program: &[u8]) -> Result<(), BackendError> {
        if program.len() > MEMORY_SIZE - MEMORY_PADDING {
            return Err(BackendError {
                instruction: None,
                kind: BackendErrorKind::ProgramInvalid,
            });
        }

        self.memory[..FONT_SIZE]
            .copy_from_slice(&font.unwrap_or(&defaults::BACKEND_FONT)[..FONT_SIZE]);

        self.memory[MEMORY_PADDING..MEMORY_PADDING + program.len()].copy_from_slice(program);
        self.loaded = true;

        Ok(())
    }

    pub fn new(
        options: super::BackendOptions,
    ) -> Self {
        Self {
            index: MEMORY_PADDING,
            loaded: false,
            memory: [0; MEMORY_SIZE],
            options,
            registers: Registers {
                address: 0,
                general: [0; REGISTER_COUNT],
            },
            stack: Vec::with_capacity(STACK_SIZE),
            delay: super::Timer::new(),
            sound: super::Timer::new(),
        }
    }

    pub fn reset(&mut self) {
        self.index = MEMORY_PADDING;

        self.registers.address = 0;
        self.registers.general.fill(0);

        self.stack.clear();

        self.delay.set(0);
        self.sound.set(0);
    }

    pub fn tick(
        &mut self,
        display_buffer: &mut display_buffer::DisplayBuffer,
        keyboard_state: &mut keypad_state::KeypadState,
    ) -> Result<(), BackendError> {
        if !self.loaded {
            return Err(BackendError {
                instruction: None,
                kind: BackendErrorKind::ProgramNotLoaded,
            });
        }

        for _ in 0..TICK_RATE {
            if self.index + 1 >= self.memory.len() {
                return Err(BackendError {
                    instruction: Some((self.index, None)),
                    kind: BackendErrorKind::MemoryOverflow,
                });
            }

            let instruction =
                Instruction::new([self.memory[self.index], self.memory[self.index + 1]]);

            let last_index = self.index;
            self.index += mem::size_of::<Instruction>();

            let control_flow = self.execute(last_index, instruction, display_buffer, keyboard_state)?;

            if control_flow.is_break() {
                break;
            }
        }

        Ok(())
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new(
            super::BackendOptions {
                copy_and_shift: true,
                increment_address: true,
                quirky_jump: false,
                reset_flag: true,
            }
        )
    }
}
