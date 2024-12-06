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

use std::error;

use clap::Parser;

#[derive(Parser)]
#[command(about, version)]
struct Options {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Shows license information
    License,

    /// Shows warranty information
    Warranty,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let options = Options::parse();

    match options.command {
        None => (),
        Some(Commands::License) => {
            println!(
                "
                Ruschip  Copyright (C) 2023  Segmentation Violator
                This program comes with ABSOLUTELY NO WARRANTY; for details use command `warranty'.
                This is free software, and you are welcome to redistribute it
                under certain conditions; see the source code or the GNU General Public License for copying conditions.

                You should have received a copy of the GNU General Public License
                along with this program.  If not, see <https://www.gnu.org/licenses/>.
                "
            );
            return Ok(());
        }
        Some(Commands::Warranty) => {
            println!(
                "
                THERE IS NO WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY
                APPLICABLE LAW.  EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT
                HOLDERS AND/OR OTHER PARTIES PROVIDE THE PROGRAM \"AS IS\" WITHOUT WARRANTY
                OF ANY KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT LIMITED TO,
                THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
                PURPOSE.  THE ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE PROGRAM
                IS WITH YOU.  SHOULD THE PROGRAM PROVE DEFECTIVE, YOU ASSUME THE COST OF
                ALL NECESSARY SERVICING, REPAIR OR CORRECTION.
                "
            );
            return Ok(());
        }
    }

    eframe::run_native(
        "Ruschip",
        eframe::NativeOptions {
            drag_and_drop_support: false,
            icon_data: Some(eframe::IconData::try_from_png_bytes(ICON_PNG)?),
            run_and_return: true,
            ..Default::default()
        },
        Box::new(move |cc| {
            Box::new(ruschip::ui::App::new(
                cc,
                ruschip::backend::Backend::default(),
            ))
        }),
    )?;

    unreachable!();
}
