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
use std::rc;
use std::time;

use eframe::egui;
use eframe::egui::mutex;
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
    _stream: rodio::OutputStream,
    display_texture: egui::TextureId,
    file_picker: file_picker::FilePicker,
    frontend: frontend::Frontend,
    persistent_storage: rc::Rc<mutex::Mutex<[u8; 8]>>,
    state: State,
}

#[derive(PartialEq, Eq)]
enum BackendSelection {
    Chip8,
    SuperChip,
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

#[derive(PartialEq, Eq)]
enum Menu {
    BackendSelection,
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
    error: Error,
    menu: Menu,
    font_path: Option<path::PathBuf>,
    program_path: Option<path::PathBuf>,
    path_selection: PathSelection,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.state.emulation != Emulation::Stopped {
            self.handle_input(ctx);
        }

        match self.state.menu {
            Menu::BackendSelection => return self.backend_selection_menu(ctx),
            Menu::Configuration => return self.configuration_menu(ctx),
            Menu::Inactive => (),
        }

        if self.state.emulation == Emulation::Running {
            ctx.request_repaint_after(TICK_INTERVAL);

            let mut persistent_storage = self.persistent_storage.lock();
            if let Err(error) = self.frontend.tick(ctx, persistent_storage.as_mut()) {
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

            if self.frontend.backend.has_program_exited() {
                self.state.emulation = Emulation::Stopped;
                self.state.menu = Menu::Configuration;
                ctx.request_repaint();
                return;
            }
        }

        let window_size = frame.info().window_info.size;
        let size;
        let margin;

        if window_size[0] / window_size[1] <= self.frontend.backend.display_buffer_aspect_ratio()
            && window_size[0] > window_size[1]
        {
            size = window_size;
            margin = egui::style::Margin::same(0.0);
        } else {
            size = egui::vec2(
                window_size[0],
                window_size[0] / self.frontend.backend.display_buffer_aspect_ratio(),
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

    fn backend_selection_menu(&mut self, ctx: &egui::Context) {
        const BACKENDS: [(&str, &str, BackendSelection); 2] = [
            (
                "CHIP-8",
                "The original CHIP-8 interpreter",
                BackendSelection::Chip8,
            ),
            (
                "SUPER-CHIP",
                "A backwards-compatible extended version of CHIP-8 with higher resolution mode and new instructions",
                BackendSelection::SuperChip,
            ),
        ];

        let current_selection = BackendSelection::get(&self.frontend.backend);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.heading("Backends");
                    ui.separator();
                    for item_data in BACKENDS {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                            if ui
                                .selectable_label(
                                    false,
                                    egui::RichText::new(item_data.0)
                                        .color(PRIMARY_COLOR)
                                        .heading(),
                                )
                                .clicked()
                            {
                                if current_selection != item_data.2 {
                                    self.frontend.backend = item_data.2.into_backend();
                                }
                                self.state.menu = Menu::Configuration;
                            }
                            ui.label({
                                egui::RichText::new(item_data.1)
                                    .color(egui::Color32::GRAY)
                                    .small()
                            });
                        });
                    }
                })
        });
    }

    fn configuration_menu(&mut self, ctx: &egui::Context) {
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

        if let Some(path) = self.file_picker.show(ctx) {
            match self.state.path_selection {
                PathSelection::Font => self.state.font_path.insert(path.to_path_buf()),
                PathSelection::Program => self.state.program_path.insert(path.to_path_buf()),
            };
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.add_enabled_ui(
                        self.state.emulation == Emulation::Stopped && !self.file_picker.is_open(),
                        |ui| {
                            if ui.button("↩").clicked() {
                                self.state.menu = Menu::BackendSelection;
                            }

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

                            for item_data in PATH_SELECTORS {
                                menu_item(ui, item_data.0, |ui| {
                                    let selected_path = item_data.1.get_path_mut(&mut self.state);

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
                                            .get_quirk_mut(self.frontend.backend.get_options_mut()),
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
                                    &mut self.frontend.backend.get_display_options_mut().clip_sprites,
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
        persistent_storage: rc::Rc<mutex::Mutex<[u8; 8]>>,
    ) -> Self {
        let mut visuals = cc.egui_ctx.style().visuals.clone();

        visuals.selection.bg_fill = PRIMARY_COLOR;
        visuals.selection.stroke.color = egui::Color32::WHITE;

        visuals.widgets.hovered.bg_fill = PRIMARY_COLOR;

        visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::WHITE;

        visuals.window_fill = SECONDARY_COLOR;
        cc.egui_ctx.set_visuals(visuals);

        let (stream, handle) = rodio::OutputStream::try_default().unwrap();

        let frontend = frontend::Frontend::new(backend, &cc.egui_ctx, handle);
        let state = State {
            emulation: Emulation::Stopped,
            error: Error {
                message: String::with_capacity(128),
                timestamp: time::Instant::now(),
            },
            menu: Menu::BackendSelection,
            font_path: None,
            program_path: None,
            path_selection: PathSelection::Font,
        };

        Self {
            _stream: stream,
            display_texture: frontend.display_texture(),
            file_picker: file_picker::FilePicker::new(),
            frontend,
            persistent_storage,
            state,
        }
    }

    pub fn start(&mut self) {
        self.state.error.message.clear();

        let font: Option<Vec<u8>> =
            match file_picker::FilePicker::load(self.state.font_path.as_ref()) {
                Ok(Some(font)) if font.len() >= backend::MIN_FONT_SIZE => Some(font),

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

        self.frontend.reset();

        if let Err(error) = self.frontend.backend.load(font.as_deref(), &program) {
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
        self.state.menu = Menu::Inactive;
    }
}

impl BackendSelection {
    pub fn get(backend: &backend::Backend) -> Self {
        match backend {
            backend::Backend::Chip8(..) => Self::Chip8,
            backend::Backend::SuperChip(..) => Self::SuperChip,
        }
    }

    pub fn into_backend(&self) -> backend::Backend {
        match self {
            Self::Chip8 => backend::Backend::Chip8(Default::default()),
            Self::SuperChip => backend::Backend::SuperChip(Default::default()),
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

impl PathSelection {
    pub fn get_path_mut<'a>(&self, state: &'a mut State) -> &'a mut Option<path::PathBuf> {
        match self {
            Self::Font => &mut state.font_path,
            Self::Program => &mut state.program_path,
        }
    }
}

impl QuirkSelection {
    pub fn get_quirk_mut<'a>(&self, options: &'a mut backend::Options) -> &'a mut bool {
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
