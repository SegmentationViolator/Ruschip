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

use std::fmt::Write;
use std::path;
use std::time;

use egui::color_picker;

use crate::backend;
use crate::defaults;
use crate::frontend;

mod file_picker;

const ERROR_DISPLAY_DURATION: time::Duration = time::Duration::from_secs(2);
const MENU_SPACING: f32 = 2.5;
pub(crate) const PRIMARY_COLOR: egui::Color32 = egui::Color32::from_rgb(0x81, 0x5B, 0xA4);
pub(crate) const SECONDARY_COLOR: egui::Color32 = egui::Color32::from_rgb(0x1C, 0x1C, 0x1C);
const TICK_INTERVAL: time::Duration = time::Duration::from_millis(1000 / 60);

pub struct App {
    _stream: rodio::OutputStream,
    display_texture: egui::TextureId,
    file_picker: file_picker::FilePicker,
    frontend: frontend::Frontend,
    state: State,
}

enum ColorSelection {
    Active,
    Inactive,
}

struct Error {
    message: String,
    timestamp: time::Instant,
}

#[derive(PartialEq, Eq)]
enum Emulation {
    Running,
    Stopped,
    Suspended,
}

enum PathSelection {
    Font,
    Program,
}

struct State {
    colors: frontend::Colors,
    emulation: Emulation,
    error: Error,
    menu_raised: bool,
    font_path: Option<path::PathBuf>,
    program_path: Option<path::PathBuf>,
    selection: PathSelection,
}

impl App {
    fn handle_input(&mut self, ctx: &egui::Context) {
        if self.state.emulation != Emulation::Stopped {
            let mut input = ctx.input_mut();

            if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                if !self.state.menu_raised {
                    self.state.emulation = Emulation::Suspended;
                    self.state.menu_raised = true;
                    return;
                }

                self.state.menu_raised = false;
                return;
            }

            if self.state.emulation == Emulation::Running {
                let mut state = Vec::with_capacity(backend::KEY_COUNT);

                for (index, key) in defaults::KEY_MAP.iter().enumerate() {
                    if input.key_pressed(*key) {
                        state.push(index);
                    }
                }

                self.frontend.update_keyboard_state(state)
            }
        }
    }

    fn menu(&mut self, ctx: &egui::Context) {
        const COLOR_PICKERS: [(&str, ColorSelection); 2] = [
            ("Active Color", ColorSelection::Active),
            ("Inactive Color", ColorSelection::Inactive),
        ];

        const PATH_SELECTORS: [(&str, PathSelection); 2] = [
            ("Font", PathSelection::Font),
            ("Program", PathSelection::Program),
        ];

        if let Some(path) = self.file_picker.show(ctx) {
            match self.state.selection {
                PathSelection::Font => self.state.font_path.insert(path),
                PathSelection::Program => self.state.program_path.insert(path),
            };
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(
                self.state.emulation == Emulation::Stopped && !self.file_picker.is_open(),
                |ui| {
                    ui.add_visible_ui(
                        !self.state.error.message.is_empty()
                            && self.state.error.timestamp.elapsed() < ERROR_DISPLAY_DURATION,
                        |ui| {
                            ui.vertical_centered_justified(|ui| {
                                ui.colored_label(egui::Color32::RED, &self.state.error.message)
                            });

                            ctx.request_repaint_after(ERROR_DISPLAY_DURATION);
                        },
                    );

                    ui.heading("Backend Parameters");
                    ui.separator();

                    for selector_info in PATH_SELECTORS {
                        menu_item(ui, selector_info.0, |ui| {
                            let selected_path = selector_info.1.get_path_mut(&mut self.state);

                            if selected_path.is_some()
                                && ui
                                    .add(
                                        egui::Label::new(
                                            egui::RichText::new("×").color(PRIMARY_COLOR),
                                        )
                                        .sense(egui::Sense::click()),
                                    )
                                    .clicked()
                            {
                                *selected_path = None;
                            }

                            let file_name = selected_path
                                .as_ref()
                                .and_then(|path| path.file_name())
                                .and_then(|file_name| file_name.to_str());

                            ui.colored_label(
                                egui::Color32::LIGHT_GRAY,
                                file_name.unwrap_or("None"),
                            );
                        });
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                            if ui
                                .selectable_label(false, format!("📂 Load {}", selector_info.0))
                                .clicked()
                            {
                                self.state.error.message.clear();
                                self.file_picker.open();
                                self.state.selection = selector_info.1;
                            }
                        });

                        ui.add_space(MENU_SPACING);
                    }

                    ui.add_space(MENU_SPACING.powi(3) - MENU_SPACING);

                    ui.heading("Frontend Parameters");
                    ui.separator();

                    for item_data in COLOR_PICKERS {
                        menu_item(ui, item_data.0, |ui| {
                            color_picker::color_edit_button_srgba(
                                ui,
                                item_data.1.get_color_mut(&mut self.state.colors),
                                color_picker::Alpha::Opaque,
                            );
                        });

                        ui.add_space(MENU_SPACING);
                    }

                    if self.state.program_path.is_some()
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
                    if ui.button("■ Stop").clicked() {
                        self.state.emulation = Emulation::Stopped;
                    }
                });
            }
        });
    }

    pub fn new(cc: &eframe::CreationContext, options: frontend::Options) -> Self {
        let mut visuals = cc.egui_ctx.style().visuals.clone();

        visuals.selection.bg_fill = PRIMARY_COLOR;
        visuals.selection.stroke.color = egui::Color32::WHITE;

        visuals.widgets.hovered.bg_fill = PRIMARY_COLOR;

        visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::WHITE;

        visuals.window_fill = SECONDARY_COLOR;
        cc.egui_ctx.set_visuals(visuals);

        let (stream, handle) = rodio::OutputStream::try_default().unwrap();

        let frontend = frontend::Frontend::new(&cc.egui_ctx, options, handle);
        let state = State {
            colors: frontend.colors,
            emulation: Emulation::Stopped,
            error: Error {
                message: String::with_capacity(128),
                timestamp: time::Instant::now(),
            },
            menu_raised: false,
            font_path: None,
            program_path: None,
            selection: PathSelection::Font,
        };

        Self {
            _stream: stream,
            display_texture: frontend.display_texture(),
            file_picker: file_picker::FilePicker::new(),
            frontend,
            state,
        }
    }

    pub fn start(&mut self) {
        self.state.error.message.clear();

        let boxed;
        let font: Option<&[u8; backend::FONT_SIZE]> =
            match file_picker::FilePicker::load(self.state.font_path.as_ref()) {
                Ok(Some(font)) if font.len() == backend::FONT_SIZE => {
                    boxed = font.into_boxed_slice(); // store the boxed slice so that it is not dropped immediately

                    Some(boxed.as_ref().try_into().unwrap())
                }

                Ok(Some(_)) => {
                    self.state.font_path = None;
                    self.state.error.timestamp = time::Instant::now();
                    self.state
                        .error
                        .message
                        .push_str("couldn't load the font, attempt to load invalid font");

                    return;
                }

                Ok(None) => None,

                Err(error) => {
                    self.state.font_path = None;
                    self.state.error.timestamp = time::Instant::now();
                    let _ = write!(
                        self.state.error.message,
                        "couldn't load the font, {}",
                        error
                    );
                    return;
                }
            };
        let program = match file_picker::FilePicker::load(self.state.program_path.as_ref()) {
            Ok(program) => program.unwrap(),
            Err(error) => {
                self.state.program_path = None;
                self.state.error.timestamp = time::Instant::now();
                let _ = write!(
                    self.state.error.message,
                    "couldn't load the program, {}",
                    error
                );
                return;
            }
        };

        self.frontend.colors = self.state.colors;
        self.frontend.reset();

        if let Err(error) = self.frontend.backend.load(font, &program) {
            self.state.program_path = None;
            self.state.error.timestamp = time::Instant::now();
            let _ = write!(
                self.state.error.message,
                "couldn't load the program, {}",
                error
            );
            return;
        };

        self.state.emulation = Emulation::Running;
        self.state.menu_raised = false;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.handle_input(ctx);

        if self.state.emulation == Emulation::Stopped || self.state.menu_raised {
            return self.menu(ctx);
        }

        ctx.request_repaint_after(TICK_INTERVAL);
        if let Err(error) = self.frontend.tick() {
            if error.is_fatal() {
                self.state.error.timestamp = time::Instant::now();
                self.state.error.message.clear();
                let _ = write!(self.state.error.message, "fatal error, {}", error);
                ctx.request_repaint();
            }

            eprintln!("{}", error);
        }

        let window_size = frame.info().window_info.size;
        let size;
        let margin;

        if window_size[0] / window_size[1] <= backend::DISPLAY_BUFFER_ASPECT_RATIO
            && window_size[0] > window_size[1]
        {
            size = window_size;
            margin = egui::style::Margin::same(0.0);
        } else {
            size = egui::vec2(
                window_size[0],
                window_size[0] / backend::DISPLAY_BUFFER_ASPECT_RATIO,
            );
            margin = egui::style::Margin::symmetric(0.0, (window_size[1] - size[1]) / 2.0);
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(margin))
            .show(ctx, |ui| {
                ui.add(egui::Image::new(self.display_texture, size));
            });
    }
}

impl PathSelection {
    pub fn get_path_mut<'a>(&self, state: &'a mut State) -> &'a mut Option<path::PathBuf> {
        match self {
            Self::Font => &mut state.font_path,
            Self::Program => &mut state.program_path,
        }
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
