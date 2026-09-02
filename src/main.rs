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

#![cfg(not(target_arch = "wasm32"))]

const APP_NAME: &str = "ruschip";
const APP_TITLE: &str = "Ruschip";
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

fn main() -> Result<(), eframe::Error> {
    let viewport = eframe::egui::ViewportBuilder::default()
        .with_title(APP_TITLE.to_string())
        .with_icon(eframe::icon_data::from_png_bytes(ICON_PNG).expect("invalid application icon"));

    eframe::run_native(
        APP_NAME,
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(ruschip::ui::App::new),
    )
}
