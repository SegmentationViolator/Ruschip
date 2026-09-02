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

use std::ptr;

use bitvec::view::BitViewSized;

pub struct DisplayBuffer {
    aspect_ratio: f32,
    buffer: Vec<bitvec::vec::BitVec<u64, bitvec::order::Msb0>>,
    dirty: bool,
    size: [usize; 2],
    pub halve_resolution: bool,
    pub options: DisplayOptions,
}

pub struct DisplayOptions {
    pub clip_sprites: bool,
    pub half_pixel_scrolling: bool,
}

impl DisplayBuffer {
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn clear(&mut self) {
        for row in self.buffer.iter_mut() {
            row.fill(false);
        }

        self.dirty = true;
    }

    pub fn flattened<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = bool> + 'a {
        self.dirty = false;
        self.buffer.iter().flat_map(|bit_array| bit_array.iter().by_vals())
    }

    pub fn draw(&mut self, coordinates: (usize, usize), sprite: &[u8]) -> usize {
        if sprite.len() == 32 {
            let sprite_16x16: Vec<u16> = sprite
                .chunks(2)
                .map(|pair| u16::from_be_bytes(pair.try_into().unwrap()))
                .collect();

            return self.internal_draw(coordinates, &sprite_16x16);
        }

        self.internal_draw(coordinates, sprite)
    }

    fn internal_draw<B: BitViewSized + Copy>(&mut self, coordinates: (usize, usize), sprite: &[B]) -> usize {
        let scaling_factor = if self.halve_resolution { 2 } else { 1 };

        let coordinates = (
            coordinates.0 * scaling_factor % self.size[0],
            coordinates.1 * scaling_factor % self.size[1],
        );
        let mut colliding_rows = 0;

        for (y, row) in sprite.iter().enumerate() {
            let cy = coordinates.1 + y * scaling_factor;

            if self.options.clip_sprites && cy == self.size[1] {
                colliding_rows += sprite.len() - y;
                break;
            }

            let cy = cy % self.size[1];
            let mut collided = false;

            for (x, bit) in row
                .into_bitarray::<bitvec::order::Msb0>()
                .iter()
                .by_vals()
                .enumerate()
            {
                let cx = coordinates.0 + x * scaling_factor;

                if self.options.clip_sprites && cx == self.size[0] {
                    break;
                }

                let cx = cx % self.size[0];

                if bit {
                    if !self.halve_resolution {
                        let mut buffer_bit = self.buffer[cy].get_mut(cx).unwrap();
                        *buffer_bit ^= true;
                        collided |= !(*buffer_bit);
                        continue;
                    }

                    for i in cy..=cy + 1 {
                        for j in cx..=cx + 1 {
                            let mut buffer_bit = self.buffer[i].get_mut(j).unwrap();
                            *buffer_bit ^= true;
                            collided |= !(*buffer_bit)
                        }
                    }
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

    pub fn new(size: [usize; 2], options: DisplayOptions) -> Self {
        Self {
            aspect_ratio: size[0] as f32 / size[1] as f32,
            buffer: vec![bitvec::vec::BitVec::repeat(false, size[0]); size[1]],
            size,
            dirty: false,
            halve_resolution: false,
            options,
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let n = if self.halve_resolution && !self.options.half_pixel_scrolling {
            2 * n
        } else {
            n
        };

        self.dirty = true;

        for i in (0..self.size[1] - n).rev() {
            let dest = ptr::from_mut(&mut self.buffer[i + n]);
            let src = &mut self.buffer[i];

            unsafe { (*dest).copy_from_bitslice(src); }

            if i < n {
                src.fill(false);
            }
        }
    }

    pub fn scroll_left(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let n = if self.halve_resolution && !self.options.half_pixel_scrolling {
            2 * n
        } else {
            n
        };

        self.dirty = true;

        for i in 0..self.size[1] {
            for j in 0..self.size[0] - n {
                self.buffer[i].copy_within(j+n..=j+n, j);

                if j + n > self.size[0] - n {
                    let mut buffer_bit = self.buffer[i].get_mut(j + n).unwrap();
                    *buffer_bit = false;
                }
            }
        }
    }

    pub fn scroll_right(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let n = if self.halve_resolution && !self.options.half_pixel_scrolling {
            2 * n
        } else {
            n
        };

        self.dirty = true;

        for i in 0..self.size[1] {
            for j in (0..self.size[0] - n).rev() {
                self.buffer[i].copy_within(j..=j, j + n);

                if j < n {
                    let mut buffer_bit = self.buffer[i].get_mut(j).unwrap();
                    *buffer_bit = false;
                }
            }
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let n = if self.halve_resolution && !self.options.half_pixel_scrolling {
            2 * n
        } else {
            n
        };

        self.dirty = true;

        for i in 0..self.size[1] - n {
            let dest = ptr::from_mut(&mut self.buffer[i]);
            let src = &mut self.buffer[i + n];

            unsafe { (*dest).copy_from_bitslice(src); }

            if i < n {
                src.fill(false);
            }
        }
    }

    #[inline]
    pub fn size(&self) -> [usize; 2] {
        self.size
    }
}
