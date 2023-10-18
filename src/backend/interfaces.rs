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

use super::KEY_COUNT;

pub struct DisplayBuffer {
    pub buffer: [bitvec::BitArr!(for super::DISPLAY_BUFFER_WIDTH, in u64, bitvec::order::Msb0);
        super::DISPLAY_BUFFER_HEIGHT],
    pub dirty: bool,
    pub options: DisplayOptions,
}

pub struct DisplayOptions {
    pub wrap_sprites: bool,
}

pub struct KeyboardState {
    state: Vec<bool>,
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

                if !self.options.wrap_sprites && cx == super::DISPLAY_BUFFER_WIDTH - 1 {
                    break;
                }
            }

            if !self.options.wrap_sprites && cy == super::DISPLAY_BUFFER_HEIGHT - 1 {
                break;
            }
        }

        self.dirty = true;

        collided
    }

    #[inline]
    pub fn new(options: DisplayOptions) -> Self {
        Self {
            buffer: [bitvec::array::BitArray::ZERO; super::DISPLAY_BUFFER_HEIGHT],
            dirty: false,
            options,
        }
    }
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            state: vec![false; KEY_COUNT],
        }
    }

    #[inline]
    pub fn pressed(&self, key: usize) -> bool {
        self.state[key]
    }

    #[inline]
    pub fn pressed_key(&self) -> Option<usize> {
        let k = self.state.iter().position(|pressed| *pressed);
        k
    }

    #[inline]
    pub fn release(&mut self) {
        self.state.fill(false)
    }

    #[inline]
    pub fn set(&mut self, key: usize, pressed: bool) {
        self.state[key] = pressed
    }
}
