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

use std::iter;

use bitvec::view::BitViewSized;
use eframe::egui;

use crate::defaults;

pub(super) struct DisplayBuffer<const W: usize, const H: usize> {
    buffer: Vec<Vec<bool>>,
    dirty: bool,
    pub(super) half_resolution: bool,
    pub options: DisplayOptions,
}

pub struct DisplayOptions {
    pub clip_sprites: bool,
    pub half_pixel_scrolling: bool,
}

pub struct KeypadState {
    state: [KeyState; super::KEY_COUNT],
    last_state: [KeyState; super::KEY_COUNT],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyState {
    Held,
    Released,
}

impl<const W: usize, const H: usize> DisplayBuffer<W, H> {
    pub fn get_flattened<'a>(
        &'a mut self,
    ) -> iter::Copied<iter::Flatten<std::slice::Iter<'a, Vec<bool>>>> {
        self.dirty = false;
        self.buffer.iter().flatten().copied()
    }

    pub fn clear(&mut self) {
        for row in self.buffer.iter_mut() {
            row.fill(false);
        }

        self.dirty = true;
    }

    pub fn draw(&mut self, coordinates: (usize, usize), sprite: &[u8]) -> usize {
        if sprite.len() == 32 && !self.half_resolution {
            let mut sprite_16x16 = Vec::with_capacity(16);
            for i in 0..16 {
                sprite_16x16.push(u16::from_be_bytes([sprite[2 * i], sprite[2 * i + 1]]))
            }

            return self.draw_16x16(coordinates, &sprite_16x16);
        }

        let scaling_factor = if self.half_resolution { 2 } else { 1 };

        let coordinates = (
            coordinates.0 * scaling_factor % W,
            coordinates.1 * scaling_factor % H,
        );
        let mut colliding_rows = 0;

        for (y, byte) in sprite.iter().enumerate() {
            let cy = coordinates.1 + y * scaling_factor;

            if self.options.clip_sprites && cy == H {
                colliding_rows += sprite.len() - y;
                break;
            }

            let cy = cy % H;
            let mut collided = false;

            for (x, bit) in byte
                .into_bitarray::<bitvec::order::Msb0>()
                .iter()
                .enumerate()
            {
                let cx = coordinates.0 + x * scaling_factor;

                if self.options.clip_sprites && cx == W {
                    break;
                }

                let cx = cx % W;

                if *bit {
                    if !self.half_resolution {
                        self.buffer[cy][cx] ^= true;
                        collided |= !(self.buffer[cy][cx]);
                        continue;
                    }

                    for i in cy..=cy + 1 {
                        for j in cx..=cx + 1 {
                            self.buffer[i][j] ^= true;
                            collided |= !(self.buffer[i][j])
                        }
                    }
                };
            }

            colliding_rows += collided as usize;
        }
        self.dirty = true;

        colliding_rows
    }

    pub fn draw_16x16(&mut self, coordinates: (usize, usize), sprite: &[u16]) -> usize {
        let coordinates = (coordinates.0 % W, coordinates.1 % H);
        let mut colliding_rows = 0;

        for (y, row) in sprite.iter().enumerate() {
            let cy = coordinates.1 + y;

            if self.options.clip_sprites && cy == H {
                colliding_rows += sprite.len() - y;
                break;
            }

            let cy = cy % H;
            let mut collided = false;

            for (x, bit) in row
                .into_bitarray::<bitvec::order::Msb0>()
                .iter()
                .enumerate()
            {
                let cx = coordinates.0 + x;

                if self.options.clip_sprites && cx == W {
                    break;
                }

                let cx = cx % W;

                if *bit {
                    self.buffer[cy][cx] ^= true;
                    collided |= !self.buffer[cy][cx];
                };
            }

            colliding_rows += collided as usize;
        }
        self.dirty = true;

        colliding_rows
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn new(options: DisplayOptions) -> Self {
        Self {
            buffer: vec![vec![false; W]; H],
            dirty: false,
            half_resolution: false,
            options,
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let n = if self.half_resolution && !self.options.half_pixel_scrolling {
            2 * n
        } else {
            n
        };

        self.dirty = true;

        for i in (0..H - n).rev() {
            let dest = &mut self.buffer[i + n] as *mut Vec<bool>;
            let src = &mut self.buffer[i];

            unsafe {
                (*dest).copy_from_slice(src);
            }

            if i < n {
                src.fill(false);
            }
        }
    }

    pub fn scroll_left(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let n = if self.half_resolution && !self.options.half_pixel_scrolling {
            2 * n
        } else {
            n
        };

        self.dirty = true;

        for i in 0..H {
            for j in 0..W - n {
                self.buffer[i][j] = self.buffer[i][j + n];

                if j + n > W - n {
                    self.buffer[i][j + n] = false;
                }
            }
        }
    }

    pub fn scroll_right(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let n = if self.half_resolution && !self.options.half_pixel_scrolling {
            2 * n
        } else {
            n
        };

        self.dirty = true;

        for i in 0..H {
            for j in (0..W - n).rev() {
                self.buffer[i][j + n] = self.buffer[i][j];

                if j < n {
                    self.buffer[i][j] = false;
                }
            }
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let n = if self.half_resolution && !self.options.half_pixel_scrolling {
            2 * n
        } else {
            n
        };

        self.dirty = true;

        for i in 0..H - n {
            let dest = &mut self.buffer[i] as *mut Vec<bool>;
            let src = &mut self.buffer[i + n];

            unsafe {
                (*dest).copy_from_slice(src);
            }

            if i < n {
                src.fill(false);
            }
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

    #[inline]
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
