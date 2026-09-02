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
use egui::color_picker;
use web_time as time;

use crate::backend;
use crate::frontend;

mod file_picker;

const ERROR_DISPLAY_DURATION: time::Duration = time::Duration::from_secs(2);
const MENU_SPACING: f32 = 5.0;
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
    path_selection: PathSelection,
}

#[derive(Clone, Copy)]
enum ColorSelection {
    Active,
    Inactive,
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

#[derive(Clone, Copy)]
enum PathSelection {
    Font,
    Program,
}

#[derive(Clone, Copy)]
enum QuirkSelection {
    CopyAndShift,
    IncrementAddress,
    QuirkyJump,
    ResetFlag,
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

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add(egui::Image::new((self.display_texture, size)));
            })
        });
    }
}

impl App {
    const FILE_PICKERS: [(&str, PathSelection); 2] = [
        ("Font", PathSelection::Font),
        ("Program", PathSelection::Program),
    ];

    const COLOR_PICKERS: [(&str, ColorSelection); 2] = [
        ("Active Color", ColorSelection::Active),
        ("Inactive Color", ColorSelection::Inactive),
    ];

    const QUIRK_TOGGLES: [(&str, &str, QuirkSelection); 4] = [
        (
            "Copy and Shift",
            "Copy the content of second operand register to the first operand register before shifting",
            QuirkSelection::CopyAndShift,
        ),
        (
            "Increment Address",
            " Increment the address register after executing SAVE and LOAD instructions",
            QuirkSelection::IncrementAddress,
        ),
        (
            "Quirky Jump",
            "The 'jump to some address plus v0' instruction (Bnnn) doesn't use v0, but vX instead where X is the highest nibble of nnn",
            QuirkSelection::QuirkyJump,
        ),
        (
            "Reset Flag",
            "Reset the flag register after executing AND, OR and XOR instructions",
            QuirkSelection::ResetFlag,
        ),
    ];

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
                return;
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

    fn show_configuration_menu(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.add_enabled_ui(
                        self.state.emulation == EmulationState::Stopped && !self.file_picker.is_open(),
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

                            for (label, selection) in Self::FILE_PICKERS {
                                menu_item(ui, label, |ui| {
                                    let file = selection.get_file_mut(&mut self.state);

                                    if file.is_some()
                                        && ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("×").color(PRIMARY_COLOR),
                                                )
                                                .frame(false)
                                            )
                                            .clicked()
                                    {
                                        *file = None;
                                    }

                                    let file_name = file
                                        .as_ref()
                                        .map(file_picker::File::name);

                                    ui.colored_label(
                                        egui::Color32::GRAY,
                                        file_name.unwrap_or("None"),
                                    );
                                });
                                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                                    if ui
                                        .selectable_label(false, format!("📂 Load {}", label))
                                        .clicked()
                                    {
                                        self.state.error.message.clear();
                                        self.file_picker.open();
                                        self.state.path_selection = selection;
                                    }
                                });

                                ui.add_space(MENU_SPACING);
                            }

                            ui.add_space(MENU_SPACING);

                            for (label, description, selection) in Self::QUIRK_TOGGLES {
                                menu_item(ui, label, |ui| {
                                    ui.checkbox(
                                            selection
                                            .get_quirk_mut(self.frontend.backend.options_mut()),
                                        "",
                                    );
                                });
                                ui.label({
                                    egui::RichText::new(description)
                                        .color(egui::Color32::GRAY)
                                        .small()
                                });

                                ui.add_space(MENU_SPACING);
                            }

                            ui.add_space(MENU_SPACING);

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

                            ui.add_space(4.0 * MENU_SPACING);

                            ui.heading("Frontend Parameters");
                            ui.separator();

                            for (label, selection) in Self::COLOR_PICKERS {
                                menu_item(ui, label, |ui| {
                                    color_picker::color_edit_button_srgba(
                                        ui,
                                        selection.get_color_mut(&mut self.frontend.colors),
                                        color_picker::Alpha::Opaque,
                                    );
                                });

                                ui.add_space(MENU_SPACING);
                            }

                            if self.state.program_file.is_some()
                                && self.state.emulation == EmulationState::Stopped
                            {
                                ui.separator();

                                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                                    if ui.button("▶ Start").clicked() {
                                        self.start_emulation();
                                    }
                                });
                            }
                        },
                    );

                    if self.state.emulation != EmulationState::Stopped {
                        ui.separator();

                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                            if ui.button("⟲ Reset").clicked() {
                                self.frontend.reset();
                                self.state.emulation = EmulationState::Running;
                                self.state.menu = MenuState::Inactive;
                            }

                            ui.add_space(MENU_SPACING);

                            if ui.button("■ Stop").clicked() {
                                self.state.emulation = EmulationState::Stopped;
                            }
                        });
                    }
            });
        });

        if self.file_picker.is_open() {
            if let Some(file) = self.file_picker.poll() {
                let selection = self.state.path_selection;
                *selection.get_file_mut(&mut self.state) = Some(file);
            }

            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, egui::Color32::from_black_alpha(100));
        }
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

impl ColorSelection {
    pub fn get_color_mut<'a>(&self, colors: &'a mut frontend::Colors) -> &'a mut egui::Color32 {
        match self {
            Self::Active => &mut colors.active,
            Self::Inactive => &mut colors.inactive,
        }
    }
}

impl PathSelection {
    pub fn get_file_mut<'a>(&self, state: &'a mut AppState) -> &'a mut Option<file_picker::File> {
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
