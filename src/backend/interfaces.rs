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

use crate::defaults;

pub struct DisplayBuffer {
    pub buffer: [bitvec::BitArr!(for super::DISPLAY_BUFFER_WIDTH, in u64, bitvec::order::Msb0);
        super::DISPLAY_BUFFER_HEIGHT],
    pub dirty: bool,
    pub options: DisplayOptions,
}

pub struct DisplayOptions {
    pub clip_sprites: bool,
}

pub struct KeypadState {
    state: [KeyState; super::KEY_COUNT],
    last_state: [KeyState; super::KEY_COUNT],
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self { clip_sprites: true }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyState {
    Held,
    Released,
}

impl DisplayBuffer {
    pub fn clear(&mut self) {
        for row in self.buffer.iter_mut() {
            row.fill(false);
        }

        self.dirty = true;
    }

    pub fn draw(&mut self, coordinates: (usize, usize), sprite: &[u8]) -> bool {
        let coordinates = (
            coordinates.0 % super::DISPLAY_BUFFER_WIDTH,
            coordinates.1 % super::DISPLAY_BUFFER_HEIGHT,
        );

        let mut collided = false;

        for (y, byte) in sprite.iter().enumerate() {
            let cy = (coordinates.1 + y) % super::DISPLAY_BUFFER_HEIGHT;

            for (x, bit) in byte
                .into_bitarray::<bitvec::order::Msb0>()
                .iter()
                .enumerate()
            {
                let cx = (coordinates.0 + x) % super::DISPLAY_BUFFER_WIDTH;

                if *bit {
                    let mut pixel = self.buffer[cy].get_mut(cx).unwrap();

                    if pixel.replace(!*pixel) {
                        collided = true;
                    }
                };

                if self.options.clip_sprites && cx == super::DISPLAY_BUFFER_WIDTH - 1 {
                    break;
                }
            }

            if self.options.clip_sprites && cy == super::DISPLAY_BUFFER_HEIGHT - 1 {
                break;
            }
        }

        self.dirty = true;

        collided
    }

    pub fn new(options: DisplayOptions) -> Self {
        Self {
            buffer: [bitvec::array::BitArray::ZERO; super::DISPLAY_BUFFER_HEIGHT],
            dirty: false,
            options,
        }
    }
}

impl KeypadState {
    pub fn new() -> Self {
        Self {
            state: [KeyState::Released; super::KEY_COUNT],
            last_state: [KeyState::Released; super::KEY_COUNT],
        }
    }

    pub fn pressed(&self, key: usize) -> bool {
        self.state[key] == KeyState::Held
    }

    pub fn pressed_key(&self) -> Option<usize> {
        (0..super::KEY_COUNT)
            .find(|&i| self.last_state[i] == KeyState::Held && self.state[i] == KeyState::Released)
    }

    pub fn update(&mut self, input: &egui::InputState) {
        self.last_state.copy_from_slice(&self.state);

        for i in 0..super::KEY_COUNT {
            if input.key_down(defaults::KEY_MAP[i]) {
                self.state[i] = KeyState::Held;
                continue;
            }

            self.state[i] = KeyState::Released;
        }
    }
}
