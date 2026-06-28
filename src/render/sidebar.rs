use egui::{Color32, CornerRadius, Painter, Pos2, Rect, RichText, Stroke, Vec2};

use crate::app::actions::AppAction;
use crate::core::piece::{Cell, Rotation, Tetromino, cell_from_piece, piece_name, piece_shape};

use super::{GameRenderView, cell_color, cell_texture_index, textured_cell_tint};
use crate::render::{COMPACT_CONTROL_PADDING, with_control_padding};

fn draw_mini_cell(painter: &Painter, pos: Pos2, size: f32, color: Color32, corner_radius: u8) {
    if color.a() == 0 {
        return;
    }
    let rect = Rect::from_min_size(pos, Vec2::new(size, size));
    painter.rect_filled(rect, CornerRadius::same(corner_radius), color);
}

fn draw_textured_mini_cell(
    painter: &Painter,
    view: &GameRenderView<'_>,
    pos: Pos2,
    size: f32,
    cell: crate::core::Cell,
    tint: Color32,
) -> bool {
    let Some(texture_handle) = view.texture_handle else {
        return false;
    };
    let Some(texture_atlas) = view.texture_atlas else {
        return false;
    };

    let rect = Rect::from_min_size(pos, Vec2::new(size, size));
    painter.image(
        texture_handle.id(),
        rect,
        texture_atlas.tile_uv_rect(cell_texture_index(cell)),
        textured_cell_tint(cell, tint),
    );
    true
}

fn render_piece_preview(
    painter: &Painter,
    view: &GameRenderView<'_>,
    kind: Tetromino,
    origin: Pos2,
    cell_size: f32,
) {
    let shape = piece_shape(kind, Rotation::Spawn);
    let cell = cell_from_piece(kind);
    let color = cell_color(cell, view.style);
    for sx in 0..4i32 {
        for sy in 0..4i32 {
            if shape[sx as usize][sy as usize] {
                let pos = Pos2::new(
                    origin.x + sx as f32 * (cell_size + 1.0),
                    origin.y + sy as f32 * (cell_size + 1.0),
                );
                if !draw_textured_mini_cell(painter, view, pos, cell_size, cell, color) {
                    draw_mini_cell(painter, pos, cell_size, color, view.style.corner_radius);
                }
            }
        }
    }
}

fn render_stats(ui: &mut egui::Ui, view: &GameRenderView<'_>) {
    let stats = &view.game.stats;
    ui.vertical(|ui| {
        ui.label(format!("Lines: {}", stats.lines));
        ui.label(format!("Damage: {}", stats.damage));
        ui.label(format!("Combo: {}", stats.combo));
        ui.label(format!("Moves: {}", stats.moves));
    });
}

fn keycap(ui: &mut egui::Ui, text: &str) {
    egui::Frame::NONE
        .fill(ui.visuals().widgets.inactive.weak_bg_fill)
        .stroke(Stroke::new(
            0.0,
            ui.visuals().widgets.inactive.bg_stroke.color,
        ))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(8, 0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).monospace().size(13.0));
        });
}

fn key_hint(ui: &mut egui::Ui, key: &str, label: &str) {
    ui.horizontal(|ui| {
        keycap(ui, key);
        ui.label(label);
    });
}

fn render_quick_help(ui: &mut egui::Ui) {
    with_control_padding(ui, COMPACT_CONTROL_PADDING, |ui| {
        ui.collapsing("Quick Help", |ui| {
            key_hint(ui, "F1", "Training");
            key_hint(ui, "F2", "Cheese");
            key_hint(ui, "F3", "Four Wide");
            key_hint(ui, "F4", "Perfect Clear");
            key_hint(ui, "F5", "Master");
            key_hint(ui, "F6", "Screenshot");
            key_hint(ui, "R", "Restart current");
            key_hint(ui, "Ctrl+Z", "Undo");
            key_hint(ui, "Ctrl+Y", "Redo");
        });
    });
}

fn render_paint_palette(
    ui: &mut egui::Ui,
    view: &GameRenderView<'_>,
    actions: &mut Vec<AppAction>,
) {
    ui.separator();
    ui.add_space(5.0);
    ui.label(RichText::new("Drawing").size(15.0));
    ui.add_space(5.0);

    let color_size = 18.0;
    let colors = [
        (Cell::I, view.style.i_cell),
        (Cell::J, view.style.j_cell),
        (Cell::S, view.style.s_cell),
        (Cell::O, view.style.o_cell),
        (Cell::Z, view.style.z_cell),
        (Cell::L, view.style.l_cell),
        (Cell::T, view.style.t_cell),
        (Cell::Gray, view.style.gray_cell),
    ];

    ui.horizontal(|ui| {
        for (cell, color) in &colors {
            let selected = *cell == view.ui_state.edit_color;
            let stroke = if selected {
                egui::Stroke::new(2.0, if ui.visuals().dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK })
            } else {
                egui::Stroke::new(1.0, if ui.visuals().dark_mode { view.style.box_color } else { egui::Color32::WHITE })
            };
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(color_size, color_size), egui::Sense::click());
            let painter = ui.painter();
            painter.rect_filled(rect, CornerRadius::same(3), *color);
            painter.rect_stroke(
                rect,
                CornerRadius::same(3),
                stroke,
                egui::StrokeKind::Outside,
            );
            if response.clicked() {
                actions.push(AppAction::SetEditColor(*cell));
            }
        }
    });

    ui.add_space(10.0);

    let mut highlight_mode = view.ui_state.highlight_mode;
    if ui.checkbox(&mut highlight_mode, "Highlight mode").changed() {
        actions.push(AppAction::ToggleHighlightMode);
    }

    if view.ui_state.highlight_mode {
        if super::btn(ui, "CLEAR").clicked() {
            actions.push(AppAction::ClearHighlight);
        }
    }

    ui.add_space(4.0);

    let mut auto_color = view.game.auto_color;
    if ui.checkbox(&mut auto_color, "Auto-coloring").changed() {
        actions.push(AppAction::ToggleAutoColor(auto_color));
    }

    ui.add_space(4.0);

    let mut auto_lock = view.game.auto_lock_on_ground;
    if ui.checkbox(&mut auto_lock, "Piece auto-locking").changed() {
        actions.push(AppAction::ToggleAutoLockOnGround(auto_lock));
    }

    ui.add_space(5.0);
    ui.separator();
}

pub fn draw_sidebar(ui: &mut egui::Ui, view: &GameRenderView<'_>, actions: &mut Vec<AppAction>) {
    egui::ScrollArea::vertical().id_salt("sidebar").show(ui, |ui| {
        if let Some(held) = view.game.hold.piece {
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(0, 9))
                    .show(ui, |ui| {
                        ui.label("HOLD");
                    });
                ui.add_space(4.0);
                if super::fa_icon_btn_small(ui, '\u{f303}', "Edit hold").clicked() {
                    actions.push(AppAction::StartHoldEdit(piece_name(held).to_string()));
                }
            });
            let (_, painter) =
                ui.allocate_painter(egui::Vec2::new(80.0, 55.0), egui::Sense::hover());

            render_piece_preview(
                &painter,
                view,
                held,
                Pos2::new(
                    painter.clip_rect().left() + 8.0,
                    painter.clip_rect().top() + 8.0,
                ),
                view.preview_cell_size,
            );
        }

        ui.add_space(12.0);
        render_stats(ui, view);

        ui.add_space(12.0);
        render_paint_palette(ui, view, actions);

        ui.add_space(8.0);
        render_quick_help(ui);
    });
}

pub fn draw_next_panel(ui: &mut egui::Ui, view: &GameRenderView<'_>, actions: &mut Vec<AppAction>) {
    egui::ScrollArea::vertical().id_salt("next_panel").show(ui, |ui| {
        ui.horizontal(|ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(0, 9))
                .show(ui, |ui| {
                    ui.label("NEXT");
                });
            ui.add_space(4.0);
            if super::fa_icon_btn_small(ui, '\u{f303}', "Edit bag").clicked() {
                actions.push(AppAction::StartBagEdit);
            }
        });
        ui.add_space(2.0);
        for piece in view
            .game
            .queue
            .visible
            .iter()
            .copied()
            .take(view.ui_state.preview_slots)
        {
            let (_, painter) =
                ui.allocate_painter(egui::Vec2::new(70.0, 70.0), egui::Sense::hover());
            render_piece_preview(
                &painter,
                view,
                piece,
                Pos2::new(
                    painter.clip_rect().left() + 8.0,
                    painter.clip_rect().top() + 8.0,
                ),
                view.preview_cell_size,
            );
            ui.add_space(2.0);
        }
    });
}
