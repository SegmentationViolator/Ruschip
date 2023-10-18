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

use std::fs;
use std::io;
use std::path;

pub struct FilePicker {
    dialog: egui_file::FileDialog,
}

impl FilePicker {
    pub fn is_open(&self) -> bool {
        self.dialog.state() == egui_file::State::Open
    }

    pub fn load(path: Option<&path::PathBuf>) -> Result<Option<Vec<u8>>, String> {
        path.map(|path| {
            fs::read(path).map_err(|error| match error.kind() {
                io::ErrorKind::NotFound => {
                    format!(
                        "file '{}' does not exists",
                        path.file_name()
                            .and_then(|file_name| file_name.to_str())
                            .unwrap()
                    )
                }
                _ => {
                    format!("{}", error)
                }
            })
        })
        .transpose()
    }

    pub fn new() -> Self {
        Self {
            dialog: egui_file::FileDialog::open_file(None)
                .resizable(false)
                .show_new_folder(false)
                .show_rename(false),
        }
    }

    pub fn open(&mut self) {
        self.dialog.open();
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<path::PathBuf> {
        if self.dialog.show(ctx).selected() {
            return self.dialog.path();
        }

        None
    }
}
