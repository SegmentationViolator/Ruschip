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

use std::fmt;
use std::mem;

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Instruction(u16);

impl Instruction {
    #[inline]
    pub fn new(be_bytes: [u8; mem::size_of::<Self>()]) -> Self {
        Self(u16::from_be_bytes(be_bytes))
    }

    #[inline]
    pub fn operator_code(&self) -> u8 {
        (self.0 >> (u16::BITS - u8::BITS / 2)) as u8
    }

    #[inline]
    pub fn operand_n(&self) -> u8 {
        (self.0 & 0x000F) as u8
    }

    #[inline]
    pub fn operand_nn(&self) -> u8 {
        (self.0 & 0x00FF) as u8
    }

    #[inline]
    pub fn operand_nnn(&self) -> usize {
        (self.0 & 0x0FFF) as usize
    }

    #[inline]
    pub fn operand_x(&self) -> usize {
        ((self.0 & 0x0F00) >> u8::BITS) as usize
    }

    #[inline]
    pub fn operand_y(&self) -> usize {
        ((self.0 & 0x00F0) >> (u8::BITS / 2)) as usize
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}
