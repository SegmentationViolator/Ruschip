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

use std::io;

const SOUND_OGG: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sound.ogg"));

#[derive(Clone, Copy)]
pub struct Sound(&'static [u8]);

impl Sound {
    #[inline]
    pub fn decode(&self) -> Result<rodio::Decoder<io::Cursor<Self>>, rodio::decoder::DecoderError> {
        rodio::Decoder::new_vorbis(io::Cursor::new(*self))
    }

    pub fn new() -> Result<Self, rodio::decoder::DecoderError> {
        let sound = Self(SOUND_OGG);
        sound.decode()?;

        Ok(sound)
    }

    #[inline]
    pub fn play(&self, sink: &rodio::Sink) {
        sink.append(self.decode().unwrap());
    }
}

impl AsRef<[u8]> for Sound {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}
