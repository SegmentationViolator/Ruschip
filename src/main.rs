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

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
const APP_NAME: &str = "ruschip";
#[cfg(not(target_arch = "wasm32"))]
const APP_TITLE: &str = "Ruschip";
#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "ruschip-canvas";
#[cfg(not(target_arch = "wasm32"))]
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

#[cfg(not(target_arch = "wasm32"))]
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


#[cfg(target_arch = "wasm32")]
fn main() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document is unavailable"))?;
    let canvas = document
        .get_element_by_id(CANVAS_ID)
        .ok_or_else(|| JsValue::from_str("Ruschip canvas is missing"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = ruschip::web::start(canvas).await {
            web_sys::console::error_1(&error);
        }
    });

    Ok(())
}
