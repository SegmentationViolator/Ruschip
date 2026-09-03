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
const TICK_INTERVAL: time::Duration = time::Duration::from_millis(1000 / 60);

pub struct App {
    display_texture: egui::TextureId,
    file_picker: file_picker::FilePicker,
    frontend: frontend::Frontend,
    last_frame: time::Instant,
    state: AppState,
}

struct AppState {
    emulation: EmulationState,
    error: ErrorMessage,
    menu: MenuState,
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

#[derive(PartialEq, Eq)]
enum MenuState {
    Configuration,
    Inactive,
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.emulation != EmulationState::Stopped {
            self.handle_input(ctx);
        }

        if self.state.emulation != EmulationState::Running {
            return;
        }

        ctx.input(|input| self.frontend.keypad_state.update(input));

        let ticks = self.last_frame.elapsed().as_millis() / TICK_INTERVAL.as_millis();

        for _ in 0..ticks {
            if let Err(error) = self.frontend.tick() {
                if error.is_fatal() {
                    self.state.error.timestamp = time::Instant::now();
                    self.state.error.message.clear();
                    let _ = write!(self.state.error.message, "fatal error, {}", error);

                    self.state.emulation = EmulationState::Stopped;
                    self.state.menu = MenuState::Configuration;
                    ctx.request_repaint();
                    return;
                }

                eprintln!("{}", error);
            }
        }

        self.last_frame = time::Instant::now();
        ctx.request_repaint_after(TICK_INTERVAL);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.state.menu {
            MenuState::Configuration => return self.show_configuration_menu(ui),
            MenuState::Inactive => (),
        }

        let viewport = ui.ctx().viewport_rect();

        let size = if viewport.aspect_ratio() <= self.frontend.display_buffer.aspect_ratio()
            && viewport.aspect_ratio() > 1.0
        {
            viewport.size()
        } else {
            egui::vec2(
                viewport.width(),
                viewport.width() / self.frontend.display_buffer.aspect_ratio(),
            )
        };

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add(egui::Image::new((self.display_texture, size)));
            })
        });
    }
}

impl App {
    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input_mut(|input| {
            if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                if self.state.menu == MenuState::Inactive {
                    self.frontend.suspend();
                    self.state.emulation = EmulationState::Suspended;
                    self.state.menu = MenuState::Configuration;
                    return;
                }

                self.state.emulation = EmulationState::Running;
                self.state.menu = MenuState::Inactive;
                return;
            }

            if self.state.menu == MenuState::Inactive
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
        let display_buffer = backend.default_display_buffer();

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

        let frontend = frontend::Frontend::new(&cc.egui_ctx, backend, display_buffer)?;

        let state = AppState {
            emulation: EmulationState::Stopped,
            error: ErrorMessage {
                message: String::with_capacity(128),
                timestamp: time::Instant::now(),
            },
            menu: MenuState::Configuration,
            font_file: None,
            program_file: None,
            path_selection: menu::PathSelection::Font,
        };

        Ok(Box::new(Self {
            display_texture: frontend.display_texture(),
            file_picker: file_picker::FilePicker::new(),
            frontend,
            last_frame: time::Instant::now(),
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
        self.state.menu = MenuState::Inactive;
    }
}
