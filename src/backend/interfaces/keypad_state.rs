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

use eframe::egui;

use crate::defaults;
pub const KEY_COUNT: usize = 16; // 0..=F

pub struct KeypadState {
    state: [KeyState; KEY_COUNT],
    last_state: [KeyState; KEY_COUNT],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyState {
    Held,
    Released,
}

impl KeypadState {
    pub fn new() -> Self {
        Self {
            state: [KeyState::Released; _],
            last_state: [KeyState::Released; _],
        }
    }

    #[inline]
    pub fn pressed(&self, key: usize) -> bool {
        self.state[key] == KeyState::Held
    }

    pub fn pressed_key(&self) -> Option<usize> {
        (0..KEY_COUNT)
            .find(|&i| self.last_state[i] == KeyState::Held && self.state[i] == KeyState::Released)
    }

    pub fn update(&mut self, input: &egui::InputState) {
        self.last_state.copy_from_slice(&self.state);

        for i in 0..KEY_COUNT {
            if input.key_down(defaults::KEY_MAP[i]) {
                self.state[i] = KeyState::Held;
                continue;
            }

            self.state[i] = KeyState::Released;
        }
    }
}
