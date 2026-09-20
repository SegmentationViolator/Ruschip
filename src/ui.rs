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

use std::error::Error;
use std::fmt::Write;

use eframe::egui;
use web_time as time;

use crate::backend;
use crate::frontend;

mod file_picker;
mod menu;

pub(crate) const PRIMARY_COLOR: egui::Color32 = egui::Color32::from_rgb(0x81, 0x5B, 0xA4); // #815BA4
pub(crate) const SECONDARY_COLOR: egui::Color32 = egui::Color32::from_rgb(0x1C, 0x1C, 0x1C); // #1C1C1C

pub struct App {
    display_texture: egui::TextureId,
    file_picker: file_picker::FilePicker,
    frontend: frontend::Frontend,
    state: AppState,
}

struct AppState {
    emulation: EmulationState,
    error: ErrorMessage,
    menu: menu::MenuState,
    font_file: Option<file_picker::File>,
    program_file: Option<file_picker::File>,
    path_selection: menu::PathSelection,
}

struct ErrorMessage {
    message: String,
    timestamp: time::Instant,
}

#[derive(PartialEq, Eq)]
enum EmulationState {
    Running,
    Stopped,
    Suspended,
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);

        if self.state.emulation != EmulationState::Running {
            return;
        }

        ctx.input(|input| self.frontend.keypad_state.update(input));

        match self.frontend.tick() {
            Ok(backend::ProgramState::Exited) => {
                self.frontend.suspend();
                self.state.emulation = EmulationState::Stopped;
                self.state.menu = menu::MenuState::Configuration;
            }

            Err(error) => {
                if error.is_fatal() {
                    self.state.error.timestamp = time::Instant::now();
                    self.state.error.message.clear();
                    let _ = write!(self.state.error.message, "fatal error, {}", error);

                    self.frontend.suspend();
                    self.state.emulation = EmulationState::Stopped;
                    self.state.menu = menu::MenuState::Configuration;
                }

                eprintln!("{}", error);
            }

            _ => (),
        }

        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "registers", &self.frontend.persistent_storage);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.state.menu {
            menu::MenuState::Backend => return self.show_backend_menu(ui),
            menu::MenuState::Configuration => return self.show_configuration_menu(ui),
            menu::MenuState::Inactive => (),
        }

        let available = ui.available_size();
        let buffer_ratio = self.frontend.display_buffer.aspect_ratio();

        let size = if available.x / available.y <= buffer_ratio {
            egui::vec2(available.x, available.x / buffer_ratio)
        } else {
            egui::vec2(available.y * buffer_ratio, available.y)
        };

        ui.centered_and_justified(|ui| {
            ui.add(egui::Image::new((self.display_texture, size)));
        });
    }
}

impl App {
    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input_mut(|input| {
            if self.state.emulation == EmulationState::Stopped {
                if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                    self.state.menu = menu::MenuState::Backend;
                }

                return;
            }

            if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                if self.state.menu == menu::MenuState::Inactive {
                    self.frontend.suspend();
                    self.state.emulation = EmulationState::Suspended;
                    self.state.menu = menu::MenuState::Configuration;
                } else {
                    self.state.emulation = EmulationState::Running;
                    self.state.menu = menu::MenuState::Inactive;
                }

                return;
            }

            if self.state.menu == menu::MenuState::Inactive
                && input.consume_key(egui::Modifiers::NONE, egui::Key::Space)
            {
                if self.state.emulation == EmulationState::Running {
                    self.frontend.suspend();
                    self.state.emulation = EmulationState::Suspended;
                    return;
                }

                self.state.emulation = EmulationState::Running;
            }
        });
    }

    pub fn new(
        cc: &eframe::CreationContext,
    ) -> Result<Box<dyn eframe::App>, Box<dyn Error + Send + Sync>> {
        let backend = backend::Backend::default();
        let display_buffer = backend.create_display_buffer();
        let persistent_storage = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, "registers"))
            .unwrap_or([0u8; backend::REGISTER_COUNT]);

        cc.egui_ctx.global_style_mut(|style| {
            style.spacing.button_padding.y = 8.0;

            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));

            style
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));

            style
                .text_styles
                .insert(egui::TextStyle::Small, egui::FontId::proportional(14.0));

            style
                .text_styles
                .insert(egui::TextStyle::Heading, egui::FontId::proportional(24.0));

            style
                .text_styles
                .insert(egui::TextStyle::Monospace, egui::FontId::monospace(16.0));

            style.visuals.selection.bg_fill = PRIMARY_COLOR;
            style.visuals.selection.stroke.color = egui::Color32::WHITE;

            style.visuals.widgets.hovered.bg_fill = PRIMARY_COLOR;
            style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::WHITE;

            style.visuals.window_fill = SECONDARY_COLOR;
        });

        let frontend =
            frontend::Frontend::new(&cc.egui_ctx, backend, display_buffer, persistent_storage)?;

        let state = AppState {
            emulation: EmulationState::Stopped,
            error: ErrorMessage {
                message: String::with_capacity(128),
                timestamp: time::Instant::now(),
            },
            menu: menu::MenuState::Backend,
            font_file: None,
            program_file: None,
            path_selection: menu::PathSelection::Font,
        };

        Ok(Box::new(Self {
            display_texture: frontend.display_texture(),
            file_picker: file_picker::FilePicker::new(),
            frontend,
            state,
        }))
    }

    pub fn start_emulation(&mut self) {
        self.state.error.message.clear();

        let font: Option<&[u8]> = self.state.font_file.as_ref().map(file_picker::File::bytes);
        let program = self
            .state
            .program_file
            .as_ref()
            .map(file_picker::File::bytes)
            .unwrap();

        self.frontend.reset();

        if let Err(error) = self.frontend.backend.load(program, font) {
            self.state.program_file = None;
            self.state.error.timestamp = time::Instant::now();
            let _ = write!(
                self.state.error.message,
                "couldn't load the program, {}",
                error
            );
            return;
        };

        self.state.emulation = EmulationState::Running;
        self.state.menu = menu::MenuState::Inactive;
    }
}
