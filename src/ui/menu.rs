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

use eframe::egui;
use egui::color_picker;
use web_time as time;

use crate::backend;
use crate::frontend;

use super::file_picker;
use super::{App, AppState, EmulationState, MenuState, PRIMARY_COLOR};

const ERROR_DISPLAY_DURATION: time::Duration = time::Duration::from_secs(2);
const MENU_SPACING: f32 = 5.0;
const MENU_SCALE_VIEWPORT_WIDTH: f32 = 1_280.0;
const MENU_BASE_CARD_WIDTH: f32 = 760.0;
const MENU_BASE_GUTTER: f32 = 20.0;
const MENU_MIN_SCALE: f32 = 1.2;
const MENU_MAX_SCALE: f32 = 1.5;

#[derive(Clone, Copy)]
enum ColorSelection {
    Active,
    Inactive,
}

#[derive(Clone, Copy)]
pub(super) enum PathSelection {
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

#[derive(Debug, PartialEq)]
struct MenuLayout {
    card_width: f32,
    scale: f32,
}

impl MenuLayout {
    fn for_viewport(viewport_width: f32) -> Self {
        let scale =
            (viewport_width / MENU_SCALE_VIEWPORT_WIDTH).clamp(MENU_MIN_SCALE, MENU_MAX_SCALE);
        let gutter = MENU_BASE_GUTTER * scale;
        let card_width = (viewport_width - 2.0 * gutter)
            .max(0.0)
            .min(MENU_BASE_CARD_WIDTH * scale);

        Self { card_width, scale }
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

    pub(super) fn show_configuration_menu(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            let layout = MenuLayout::for_viewport(ui.available_width());
            let style = scaled_menu_style(ui.style(), layout.scale);
            let menu_spacing = MENU_SPACING * layout.scale;

            ui.set_style(style);
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    let side_margin = ((ui.available_width() - layout.card_width) / 2.0).max(0.0);
                    ui.horizontal(|ui| {
                        ui.add_space(side_margin);
                        ui.allocate_ui_with_layout(
                            egui::vec2(layout.card_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.add_enabled_ui(
                                    self.state.emulation == EmulationState::Stopped
                                        && !self.file_picker.is_open(),
                                    |ui| {
                                        if !self.state.error.message.is_empty()
                                            && self.state.error.timestamp.elapsed()
                                                < ERROR_DISPLAY_DURATION
                                        {
                                            ui.vertical_centered_justified(|ui| {
                                                ui.colored_label(
                                                    egui::Color32::RED,
                                                    &self.state.error.message,
                                                )
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
                                                                egui::RichText::new("×")
                                                                    .color(PRIMARY_COLOR),
                                                            )
                                                            .frame(false),
                                                        )
                                                        .clicked()
                                                {
                                                    *file = None;
                                                }

                                                let file_name =
                                                    file.as_ref().map(file_picker::File::name);

                                                ui.colored_label(
                                                    egui::Color32::GRAY,
                                                    file_name.unwrap_or("None"),
                                                );
                                            });
                                            ui.with_layout(
                                                egui::Layout::top_down_justified(egui::Align::Min),
                                                |ui| {
                                                    if ui
                                                        .selectable_label(
                                                            false,
                                                            format!("📂 Load {label}"),
                                                        )
                                                        .clicked()
                                                    {
                                                        self.state.error.message.clear();
                                                        self.file_picker.open();
                                                        self.state.path_selection = selection;
                                                    }
                                                },
                                            );

                                            ui.add_space(menu_spacing);
                                        }

                                        ui.add_space(menu_spacing);

                                        for (label, description, selection) in Self::QUIRK_TOGGLES {
                                            menu_item(ui, label, |ui| {
                                                ui.checkbox(
                                                    selection.get_quirk_mut(
                                                        self.frontend.backend.options_mut(),
                                                    ),
                                                    "",
                                                );
                                            });
                                            ui.label(
                                                egui::RichText::new(description)
                                                    .color(egui::Color32::GRAY)
                                                    .small(),
                                            );

                                            ui.add_space(menu_spacing);
                                        }

                                        ui.add_space(menu_spacing);

                                        menu_item(ui, "Clip Sprites", |ui| {
                                            ui.checkbox(
                                                &mut self.frontend.display_buffer.options.clip_sprites,
                                                "",
                                            );
                                        });
                                        ui.label(
                                            egui::RichText::new(
                                                "Clip the sprites drawn beyond the edge of the screen (wrap around if off)",
                                            )
                                            .color(egui::Color32::GRAY)
                                            .small(),
                                        );

                                        ui.add_space(4.0 * menu_spacing);

                                        ui.heading("Frontend Parameters");
                                        ui.separator();

                                        for (label, selection) in Self::COLOR_PICKERS {
                                            menu_item(ui, label, |ui| {
                                                color_picker::color_edit_button_srgba(
                                                    ui,
                                                    selection
                                                        .get_color_mut(&mut self.frontend.colors),
                                                    color_picker::Alpha::Opaque,
                                                );
                                            });

                                            ui.add_space(menu_spacing);
                                        }

                                        if self.state.program_file.is_some()
                                            && self.state.emulation == EmulationState::Stopped
                                        {
                                            ui.separator();

                                            ui.with_layout(
                                                egui::Layout::top_down_justified(egui::Align::Min),
                                                |ui| {
                                                    if ui.button("▶ Start").clicked() {
                                                        self.start_emulation();
                                                    }
                                                },
                                            );
                                        }
                                    },
                                );

                                if self.state.emulation != EmulationState::Stopped {
                                    ui.separator();

                                    ui.with_layout(
                                        egui::Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui.button("⟲ Reset").clicked() {
                                                self.frontend.reset();
                                                self.state.emulation = EmulationState::Running;
                                                self.state.menu = MenuState::Inactive;
                                            }

                                            ui.add_space(menu_spacing);

                                            if ui.button("■ Stop").clicked() {
                                                self.state.emulation = EmulationState::Stopped;
                                            }
                                        },
                                    );
                                }
                            },
                        );
                    });
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
}

fn scaled_menu_style(style: &egui::Style, scale: f32) -> egui::Style {
    let mut style = style.clone();

    for font in style.text_styles.values_mut() {
        font.size *= scale;
    }

    style.spacing.button_padding *= scale;
    style.spacing.item_spacing *= scale;
    style.spacing.interact_size *= scale;
    style.spacing.indent *= scale;
    style.spacing.slider_width *= scale;
    style.spacing.slider_rail_height *= scale;
    style.spacing.combo_width *= scale;
    style.spacing.text_edit_width *= scale;
    style.spacing.icon_width *= scale;
    style.spacing.icon_width_inner *= scale;
    style.spacing.icon_spacing *= scale;

    style
}

impl ColorSelection {
    fn get_color_mut<'a>(&self, colors: &'a mut frontend::Colors) -> &'a mut egui::Color32 {
        match self {
            Self::Active => &mut colors.active,
            Self::Inactive => &mut colors.inactive,
        }
    }
}

impl PathSelection {
    fn get_file_mut<'a>(&self, state: &'a mut AppState) -> &'a mut Option<file_picker::File> {
        match self {
            Self::Font => &mut state.font_file,
            Self::Program => &mut state.program_file,
        }
    }
}

impl QuirkSelection {
    fn get_quirk_mut<'a>(&self, options: &'a mut backend::BackendOptions) -> &'a mut bool {
        match self {
            Self::CopyAndShift => &mut options.copy_and_shift,
            Self::IncrementAddress => &mut options.increment_address,
            Self::QuirkyJump => &mut options.quirky_jump,
            Self::ResetFlag => &mut options.reset_flag,
        }
    }
}

fn menu_item(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_layout_preserves_a_readable_minimum_scale() {
        let layout = MenuLayout::for_viewport(320.0);

        assert_eq!(layout.scale, MENU_MIN_SCALE);
        assert_eq!(layout.card_width, 272.0);
    }

    #[test]
    fn menu_layout_delays_growth_on_standard_desktop_viewports() {
        let layout = MenuLayout::for_viewport(MENU_SCALE_VIEWPORT_WIDTH);

        assert_eq!(layout.scale, MENU_MIN_SCALE);
        assert_eq!(layout.card_width, MENU_BASE_CARD_WIDTH * MENU_MIN_SCALE);
    }

    #[test]
    fn menu_layout_caps_growth_on_large_viewports() {
        let layout = MenuLayout::for_viewport(3_000.0);

        assert_eq!(layout.scale, MENU_MAX_SCALE);
        assert_eq!(layout.card_width, MENU_BASE_CARD_WIDTH * MENU_MAX_SCALE);
    }
}
