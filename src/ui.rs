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
use std::time;

use eframe::egui;
use egui::color_picker;

use crate::backend;
use crate::frontend;

mod file_picker;

const ERROR_DISPLAY_DURATION: time::Duration = time::Duration::from_secs(2);
const MENU_SPACING: f32 = 2.5;
pub(crate) const PRIMARY_COLOR: egui::Color32 = egui::Color32::from_rgb(0x81, 0x5B, 0xA4); // #815BA4
pub(crate) const SECONDARY_COLOR: egui::Color32 = egui::Color32::from_rgb(0x1C, 0x1C, 0x1C); // #1C1C1C
const TICK_INTERVAL: time::Duration = time::Duration::from_millis(1000 / 60);

pub struct App {
    display_texture: egui::TextureId,
    file_picker: file_picker::FilePicker,
    frontend: frontend::Frontend,
    last_frame: time::Instant,
    state: State,
}

enum ColorSelection {
    Active,
    Inactive,
}

struct ErrorMessage {
    message: String,
    timestamp: time::Instant,
}

#[derive(PartialEq, Eq)]
enum Emulation {
    Running,
    Stopped,
    Suspended,
}

#[derive(PartialEq, Eq)]
enum Menu {
    Configuration,
    Inactive,
}

enum PathSelection {
    Font,
    Program,
}

enum QuirkSelection {
    CopyAndShift,
    IncrementAddress,
    QuirkyJump,
    ResetFlag,
}

struct State {
    emulation: Emulation,
    error: ErrorMessage,
    menu: Menu,
    font_file: Option<file_picker::SelectedFile>,
    program_file: Option<file_picker::SelectedFile>,
    path_selection: PathSelection,
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.emulation != Emulation::Stopped {
            self.handle_input(ctx);
        }

        if self.state.emulation != Emulation::Running {
            return;
        }

        ctx.input(|input| self.frontend.keypad_state.update(input));

        dbg!(self.last_frame.elapsed());
        let ticks = self.last_frame.elapsed().as_millis() / TICK_INTERVAL.as_millis();

        for _ in 0..ticks {
            if let Err(error) = self.frontend.tick() {
                if error.is_fatal() {
                    self.state.error.timestamp = time::Instant::now();
                    self.state.error.message.clear();
                    let _ = write!(self.state.error.message, "fatal error, {}", error);

                    self.state.emulation = Emulation::Stopped;
                    self.state.menu = Menu::Configuration;
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
            Menu::Configuration => return self.configuration_menu(ui),
            Menu::Inactive => (),
        }

        let viewport = ui.ctx().viewport_rect();
        let size;

        if viewport.aspect_ratio() <= self.frontend.display_buffer.aspect_ratio()
            && viewport.aspect_ratio() > 1.0
        {
            size = viewport.size();
        } else {
            size = egui::vec2(
                viewport.width(),
                viewport.width() / self.frontend.display_buffer.aspect_ratio(),
            );
        };

        egui::CentralPanel::default()
            .show(ui, |ui| {
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
                if self.state.menu == Menu::Inactive {
                    self.frontend.suspend();
                    self.state.emulation = Emulation::Suspended;
                    self.state.menu = Menu::Configuration;
                    return;
                }

                self.state.emulation = Emulation::Running;
                self.state.menu = Menu::Inactive;
                return;
            }

            if self.state.menu == Menu::Inactive
                && input.consume_key(egui::Modifiers::NONE, egui::Key::Space)
            {
                if self.state.emulation == Emulation::Running {
                    self.frontend.suspend();
                    self.state.emulation = Emulation::Suspended;
                    return;
                }

                self.state.emulation = Emulation::Running;
                return;
            }
        });
    }

    fn configuration_menu(&mut self, ui: &mut egui::Ui) {
        const COLOR_PICKERS: [(&str, ColorSelection); 2] = [
            ("Active Color", ColorSelection::Active),
            ("Inactive Color", ColorSelection::Inactive),
        ];

        const PATH_SELECTORS: [(&str, PathSelection); 2] = [
            ("Font", PathSelection::Font),
            ("Program", PathSelection::Program),
        ];

        const QUIRK_TOGGLES: [(&str, &str, QuirkSelection); 4] = [
            ("Copy and Shift", "Copy the content of second operand register to the first operand register before shifting", QuirkSelection::CopyAndShift),
            ("Increment Address", " Increment the address register after executing SAVE and LOAD instructions", QuirkSelection::IncrementAddress),
            ("Quirky Jump", "The 'jump to some address plus v0' instruction (Bnnn) doesn't use v0, but vX instead where X is the highest nibble of nnn", QuirkSelection::QuirkyJump),
            ("Reset Flag", "Reset the flag register after executing AND, OR and XOR instructions", QuirkSelection::ResetFlag),
        ];

        if let Some(file) = self.file_picker.poll() {
            match self.state.path_selection {
                PathSelection::Font => self.state.font_file.insert(file),
                PathSelection::Program => self.state.program_file.insert(file),
            };
        }

        if self.file_picker.is_open() {
            ui.ctx().request_repaint_after(time::Duration::from_millis(16));
        }

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.add_enabled_ui(
                        self.state.emulation == Emulation::Stopped && !self.file_picker.is_open(),
                        |ui| {
                            if !self.state.error.message.is_empty()
                                && self.state.error.timestamp.elapsed() < ERROR_DISPLAY_DURATION
                            {
                                ui.vertical_centered_justified(|ui| {
                                    ui.colored_label(egui::Color32::RED, &self.state.error.message)
                                });

                                ui.request_repaint_after(ERROR_DISPLAY_DURATION);
                            }

                            ui.heading("Backend Parameters");
                            ui.separator();

                            for item_data in PATH_SELECTORS {
                                menu_item(ui, item_data.0, |ui| {
                                    let selected_file = item_data.1.get_file_mut(&mut self.state);

                                    if selected_file.is_some()
                                        && ui
                                            .add(
                                                egui::Label::new(
                                                    egui::RichText::new("×").color(PRIMARY_COLOR),
                                                )
                                                .sense(egui::Sense::click()),
                                            )
                                            .clicked()
                                    {
                                        *selected_file = None;
                                    }

                                    let file_name = selected_file
                                        .as_ref()
                                        .map(file_picker::SelectedFile::name);

                                    ui.colored_label(
                                        egui::Color32::GRAY,
                                        file_name.unwrap_or("None"),
                                    );
                                });
                                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                                    if ui
                                        .selectable_label(false, format!("📂 Load {}", item_data.0))
                                        .clicked()
                                    {
                                        self.state.error.message.clear();
                                        self.file_picker.open();
                                        self.state.path_selection = item_data.1;
                                    }
                                });

                                ui.add_space(MENU_SPACING);
                            }

                            for item_data in QUIRK_TOGGLES {
                                menu_item(ui, item_data.0, |ui| {
                                    ui.checkbox(
                                        item_data
                                            .2
                                            .get_quirk_mut(self.frontend.backend.options_mut()),
                                        "",
                                    );
                                });
                                ui.label({
                                    egui::RichText::new(item_data.1)
                                        .color(egui::Color32::GRAY)
                                        .small()
                                });

                                ui.add_space(MENU_SPACING);
                            }

                            menu_item(ui, "Clip Sprites", |ui| {
                                ui.checkbox(
                                    &mut self.frontend.display_buffer.options.clip_sprites,
                                    "",
                                );
                            });
                            ui.label({
                                egui::RichText::new("Clip the sprites drawn beyond the edge of the screen (wrap around if off)")
                                    .color(egui::Color32::GRAY)
                                    .small()
                            });

                            ui.add_space(MENU_SPACING);

                            ui.add_space(4.0 * MENU_SPACING);

                            ui.heading("Frontend Parameters");
                            ui.separator();

                            for item_data in COLOR_PICKERS {
                                menu_item(ui, item_data.0, |ui| {
                                    color_picker::color_edit_button_srgba(
                                        ui,
                                        item_data.1.get_color_mut(&mut self.frontend.colors),
                                        color_picker::Alpha::Opaque,
                                    );
                                });

                                ui.add_space(MENU_SPACING);
                            }

                            if self.state.program_file.is_some()
                                && self.state.emulation == Emulation::Stopped
                            {
                                ui.separator();

                                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                                    if ui.button("▶ Start").clicked() {
                                        self.start();
                                    }
                                });
                            }
                        },
                    );

                    if self.state.emulation != Emulation::Stopped {
                        ui.separator();

                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                            if ui.button("⟲ Reset").clicked() {
                                self.frontend.reset();
                                self.state.emulation = Emulation::Running;
                                self.state.menu = Menu::Inactive;
                            }

                            ui.add_space(MENU_SPACING);

                            if ui.button("■ Stop").clicked() {
                                self.state.emulation = Emulation::Stopped;
                            }
                        });
                    }
            });
        });
    }

    pub fn new(
        cc: &eframe::CreationContext,
        backend: backend::Backend,
        display_buffer: backend::interfaces::display_buffer::DisplayBuffer,
    ) -> Result<Box<Self>, Box<dyn Error + Send + Sync>> {
        cc.egui_ctx.global_style_mut(|style| {
            style.visuals.selection.bg_fill = PRIMARY_COLOR;
            style.visuals.selection.stroke.color = egui::Color32::WHITE;

            style.visuals.widgets.hovered.bg_fill = PRIMARY_COLOR;
            style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::WHITE;

            style.visuals.window_fill = SECONDARY_COLOR;

            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::proportional(16.0),
            );

            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(16.0),
            );

            style.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::proportional(13.0),
            );

            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::proportional(22.0),
            );

            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::monospace(16.0),
            );
        });

        let frontend = frontend::Frontend::new(&cc.egui_ctx, backend, display_buffer)?;

        let state = State {
            emulation: Emulation::Stopped,
            error: ErrorMessage {
                message: String::with_capacity(128),
                timestamp: time::Instant::now(),
            },
            menu: Menu::Configuration,
            font_file: None,
            program_file: None,
            path_selection: PathSelection::Font,
        };

        Ok(Box::new(Self {
            display_texture: frontend.display_texture(),
            file_picker: file_picker::FilePicker::new(),
            frontend,
            last_frame: time::Instant::now(),
            state,
        }))
    }

    pub fn start(&mut self) {
        self.state.error.message.clear();

        let font: Option<Vec<u8>> =
            match file_picker::FilePicker::load(self.state.font_file.as_ref()) {
                Some(font) if font.len() >= backend::MIN_FONT_SIZE => Some(font),

                Some(_) => {
                    self.state.font_file = None;
                    self.state.error.timestamp = time::Instant::now();
                    self.state
                        .error
                        .message
                        .push_str("couldn't load the font, attempt to load invalid font");

                    return;
                }

                None => None,
            };

        let program = file_picker::FilePicker::load(self.state.program_file.as_ref()).unwrap();

        self.frontend.reset();

        if let Err(error) = self.frontend.backend.load(font.as_deref(), &program) {
            self.state.program_file = None;
            self.state.error.timestamp = time::Instant::now();
            let _ = write!(
                self.state.error.message,
                "couldn't load the program, {}",
                error
            );
            return;
        };

        self.state.emulation = Emulation::Running;
        self.state.menu = Menu::Inactive;
    }
}

impl ColorSelection {
    pub fn get_color_mut<'a>(&self, colors: &'a mut frontend::Colors) -> &'a mut egui::Color32 {
        match self {
            Self::Active => &mut colors.active,
            Self::Inactive => &mut colors.inactive,
        }
    }
}

impl PathSelection {
    pub fn get_file_mut<'a>(
        &self,
        state: &'a mut State,
    ) -> &'a mut Option<file_picker::SelectedFile> {
        match self {
            Self::Font => &mut state.font_file,
            Self::Program => &mut state.program_file,
        }
    }
}

impl QuirkSelection {
    pub fn get_quirk_mut<'a>(&self, options: &'a mut backend::BackendOptions) -> &'a mut bool {
        match self {
            Self::CopyAndShift => &mut options.copy_and_shift,
            Self::IncrementAddress => &mut options.increment_address,
            Self::QuirkyJump => &mut options.quirky_jump,
            Self::ResetFlag => &mut options.reset_flag,
        }
    }
}

pub fn menu_item(
    ui: &mut egui::Ui,
    text: impl Into<egui::WidgetText>,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.label(text)
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), add_contents);
    });
}
