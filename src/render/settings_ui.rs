use crate::app::actions::AppAction;
use crate::app::ui_state::UiState;
use crate::config::{AppConfig, KeyName, PersistedBagMode};
use crate::render::btn;

fn drag<T: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    value: &mut T,
    range: std::ops::RangeInclusive<T>,
) {
    ui.add(egui::DragValue::new(value).range(range));
}

fn draw_key_selector(ui: &mut egui::Ui, label: &str, key_name: &mut KeyName) -> bool {
    let mut changed = false;
    egui::ComboBox::from_label(label)
        .selected_text(key_name.display_name())
        .show_ui(ui, |ui| {
            for candidate in KeyName::ALL {
                changed |= ui
                    .selectable_value(key_name, candidate, candidate.display_name())
                    .changed();
            }
        });
    changed
}

pub fn draw_settings_window(
    ctx: &egui::Context,
    state: &mut UiState,
    actions: &mut Vec<AppAction>,
) {
    if !state.settings_open {
        return;
    }

    let Some(cfg) = state.pending_config.as_mut() else {
        return;
    };

    let mut close_action: Option<AppAction> = None;

    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .min_size(egui::vec2(560.0, 420.0))
        .show(ctx, |ui| {
            let max_scroll_height = (ui.available_height() - 50.0).max(100.0);
            egui::ScrollArea::vertical()
                .max_height(max_scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if btn(ui, "Import Settings").clicked() {
                                actions.push(AppAction::ImportSettings);
                            }
                            if btn(ui, "Export Settings").clicked() {
                                actions.push(AppAction::ExportSettings);
                            }
                        });
                    });
                    ui.separator();
                    ui.checkbox(&mut cfg.show_ghost, "Show ghost piece");

                    let mut volume = i32::from(cfg.volume_percent);
                    if ui
                        .add(egui::Slider::new(&mut volume, 0..=100).text("Volume"))
                        .changed()
                    {
                        cfg.volume_percent = volume as u8;
                    }

                    let textures = AppConfig::available_textures();
                    egui::ComboBox::from_label("Texture")
                        .selected_text(cfg.selected_texture.as_str())
                        .show_ui(ui, |ui| {
                            if textures.is_empty() {
                                ui.label("No textures found");
                            }
                            for texture in &textures {
                                ui.selectable_value(&mut cfg.selected_texture, texture.clone(), texture);
                            }
                        });

                    egui::ComboBox::from_label("Theme")
                        .selected_text(match cfg.theme {
                            crate::config::ThemeMode::Auto => "Auto",
                            crate::config::ThemeMode::Light => "Light",
                            crate::config::ThemeMode::Dark => "Dark",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut cfg.theme, crate::config::ThemeMode::Auto, "Auto");
                            ui.selectable_value(&mut cfg.theme, crate::config::ThemeMode::Light, "Light");
                            ui.selectable_value(&mut cfg.theme, crate::config::ThemeMode::Dark, "Dark");
                        });

                    let mut grid_brightness = cfg.grid_brightness as i32;
                    if ui
                        .add(egui::Slider::new(&mut grid_brightness, 0..=100).text("Grid brightness"))
                        .changed()
                    {
                        cfg.grid_brightness = grid_brightness as u8;
                    }

                    ui.checkbox(&mut cfg.colored_ghost, "Colored ghost");
                    if cfg.colored_ghost {
                        let mut opacity = cfg.ghost_opacity_percent as i32;
                        if ui
                            .add(egui::Slider::new(&mut opacity, 0..=100).text("Ghost opacity"))
                            .changed()
                        {
                            cfg.ghost_opacity_percent = opacity as u8;
                        }
                    }

                    ui.separator();

                    ui.collapsing("Board", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Width");
                            drag(ui, &mut cfg.board_width, 1..=32);
                            ui.label("Visible");
                            drag(ui, &mut cfg.board_visible_height, 1..=64);
                            ui.label("Hidden");
                            drag(ui, &mut cfg.board_hidden_rows, 0..=8);
                        });
                        egui::ComboBox::from_label("Bag")
                            .selected_text(match cfg.bag_mode {
                                PersistedBagMode::SevenBag => "Seven Bag",
                                PersistedBagMode::FourteenBag => "Fourteen Bag",
                                PersistedBagMode::Random => "Random",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut cfg.bag_mode, PersistedBagMode::SevenBag, "Seven Bag");
                                ui.selectable_value(&mut cfg.bag_mode, PersistedBagMode::FourteenBag, "Fourteen Bag");
                                ui.selectable_value(&mut cfg.bag_mode, PersistedBagMode::Random, "Random");
                            });
                    });

                    ui.collapsing("Timing", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("DAS");
                            drag(ui, &mut cfg.das_ms, 0..=5000);
                            ui.label("ARR");
                            drag(ui, &mut cfg.arr_ms, 0..=5000);
                            ui.label("SDD");
                            drag(ui, &mut cfg.sdd_ms, 0..=5000);
                            ui.label("SDS");
                            drag(ui, &mut cfg.sds_ms, 0..=5000);
                        });
                        ui.checkbox(&mut cfg.das_cancel, "DAS cancel");
                    });

                    ui.collapsing("Controls", |ui| {
                        ui.checkbox(&mut cfg.bindings.hold_with_shift, "Hold on Shift");
                        draw_key_selector(ui, "Move left", &mut cfg.bindings.move_left);
                        draw_key_selector(ui, "Move right", &mut cfg.bindings.move_right);
                        draw_key_selector(ui, "Soft drop", &mut cfg.bindings.soft_drop);
                        draw_key_selector(ui, "Rotate CCW", &mut cfg.bindings.rotate_ccw);
                        draw_key_selector(ui, "Rotate CW", &mut cfg.bindings.rotate_cw);
                        draw_key_selector(ui, "Rotate 180", &mut cfg.bindings.rotate_180);
                        if cfg.bindings.hold_with_shift {
                            ui.label("Hold key is handled by Shift modifier");
                        } else {
                            draw_key_selector(ui, "Hold", &mut cfg.bindings.hold);
                        }

                        let mut primary_hard_drop = cfg.bindings.hard_drop.first().copied().unwrap_or(KeyName::Up);
                        if draw_key_selector(ui, "Hard drop primary", &mut primary_hard_drop) {
                            if cfg.bindings.hard_drop.is_empty() {
                                cfg.bindings.hard_drop.push(primary_hard_drop);
                            } else {
                                cfg.bindings.hard_drop[0] = primary_hard_drop;
                            }
                        }

                        let mut alt_enabled = cfg.bindings.hard_drop.len() > 1;
                        if ui.checkbox(&mut alt_enabled, "Hard drop alt").changed() {
                            if !alt_enabled {
                                cfg.bindings.hard_drop.truncate(1);
                            }
                        }
                        if alt_enabled {
                            let mut alt_hard_drop = cfg.bindings.hard_drop.get(1).copied().unwrap_or(KeyName::Space);
                            if draw_key_selector(ui, "Hard drop alt key", &mut alt_hard_drop) {
                                if cfg.bindings.hard_drop.len() < 2 {
                                    cfg.bindings.hard_drop.push(alt_hard_drop);
                                } else {
                                    cfg.bindings.hard_drop[1] = alt_hard_drop;
                                }
                            }
                        }
                    });

                    ui.collapsing("Runtime", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Board cell");
                            drag(ui, &mut cfg.board_cell_size, 1..=96);
                            ui.label("Preview cell");
                            drag(ui, &mut cfg.preview_cell_size, 1..=96);
                        });
                        ui.checkbox(&mut cfg.infinite_hold, "Infinite hold");
                        ui.checkbox(&mut cfg.auto_color, "Auto color");
                        ui.checkbox(&mut cfg.auto_lock_on_ground, "Auto lock on ground");
                        ui.checkbox(&mut cfg.mirror_queue, "Mirror queue");
                        ui.checkbox(&mut cfg.highlight_clear, "Highlight clear");
                        ui.checkbox(&mut cfg.static_bag, "Use static bag");
                    });
                    });
                });

            ui.separator();

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if super::btn(ui, "Cancel").clicked() {
                        close_action = Some(AppAction::CancelSettings);
                    }
                    if super::btn(ui, "Apply").clicked() {
                        actions.push(AppAction::ApplySettings);
                    }
                    if super::btn(ui, "OK").clicked() {
                        close_action = Some(AppAction::ConfirmSettings);
                    }
                });
            });
        });

    if let Some(action) = close_action {
        state.settings_open = false;
        actions.push(action);
    }
}
