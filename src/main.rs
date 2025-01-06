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

const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

use std::cell;
use std::error;
use std::fs;
use std::io::Read;
use std::rc;

fn main() -> Result<(), Box<dyn error::Error>> {
    let data_dir = dirs::data_dir()
        .or(dirs::data_dir())
        .expect("couldn't find a data directory")
        .join("ruschip");
    let data_file = data_dir.join("rpl_user_flags.dat");

    fs::create_dir_all(&data_dir)?;

    let mut rpl_user_flags = [0; ruschip::backend::superchip::PERSISTENT_STORAGE_SIZE];
    let _ = fs::File::open(&data_file).and_then(|mut file| file.read(&mut rpl_user_flags));

    let persistent_storage = rc::Rc::new(cell::RefCell::new(rpl_user_flags));
    let persistent_storage_clone = persistent_storage.clone();

    eframe::run_native(
        "Ruschip",
        eframe::NativeOptions {
            drag_and_drop_support: false,
            icon_data: Some(eframe::IconData::try_from_png_bytes(ICON_PNG)?),
            ..Default::default()
        },
        Box::new(move |cc| {
            Box::new(ruschip::ui::App::new(
                cc,
                ruschip::backend::Backend::default(),
                persistent_storage_clone,
            ))
        }),
    )?;

    fs::create_dir_all(data_dir)?;

    let rpl_user_flags = persistent_storage.borrow();
    fs::write(data_file, rpl_user_flags.as_ref())?;

    Ok(())
}
