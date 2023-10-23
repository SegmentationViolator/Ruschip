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
    /// Clip the sprites drawn beyond the edge of the screen [Default: OFF (wraps the sprites)]
    #[arg(long, default_value_t = true)]
    clip_sprites: bool,

    /// Copy the content of second operand register to the first operand register before shifting [DEFAULT: ON]
    #[arg(long, default_value_t = true)]
    copy_and_shift: bool,

    /// Increment the address register after executing SAVE and LOAD instructions [DEFAULT: ON]
    #[arg(long, default_value_t = true)]
    increment_address: bool,

    /// The "jump to some address plus v0" instruction (Bnnn) doesn't use v0, but vX instead where X is the highest nibble of nnn [DEFAULT: OFF]
    #[arg(long)]
    quirky_jump: bool,

    /// Reset the flag register after executing AND, OR and XOR instructions [DEFAULT: ON]
    #[arg(long, default_value_t = true)]
    reset_flag: bool,
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
                ruschip::backend::Options {
                    copy_and_shift: options.copy_and_shift,
                    reset_flag: options.reset_flag,
                    increment_address: options.increment_address,
                    quirky_jump: options.quirky_jump,
                },
                ruschip::frontend::Options {
                    clip_sprites: options.clip_sprites,
                },
            ))
        }),
    );
}
