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

use clap::Parser;

#[derive(Parser)]
#[command(about, author, version)]
struct Options {
    /// Wrap the sprites drawn beyond the edge of the screen, (clips/crops them by default)
    #[arg(long)]
    wrap_sprites: bool,
}

fn main() {
    let options = Options::parse();

    eframe::run_native(
        "Ruschip",
        eframe::NativeOptions {
            drag_and_drop_support: false,
            run_and_return: false,
            ..Default::default()
        },
        Box::new(move |cc| {
            Box::new(ruschip::ui::App::new(
                cc,
                ruschip::frontend::Options {
                    wrap_sprites: options.wrap_sprites,
                },
            ))
        }),
    );
}
