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

use std::error;
use std::fmt;

use crate::backend;

#[derive(Debug)]
pub enum FrontendError {
    Audio(rodio::PlayError),
    Backend(backend::BackendError),
}

impl FrontendError {
    pub fn is_fatal(&self) -> bool {
        match self {
            Self::Backend(error) => matches!(
                error.kind,
                backend::BackendErrorKind::MemoryOverflow
                    | backend::BackendErrorKind::ProgramInvalid
                    | backend::BackendErrorKind::ProgramNotLoaded
            ),
            _ => true,
        }
    }
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Audio(error) => write!(f, "{}", error),
            Self::Backend(error) => write!(f, "{}", error),
        }
    }
}

impl error::Error for FrontendError {}
