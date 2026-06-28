use egui::{Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, Vec2};

use crate::app::actions::AppAction;
use crate::core::board::HighlightBoard;
use crate::core::game_state::{ClearFlashState, GameState};
use crate::core::piece::{Cell, PieceState, Tetromino, cell_from_piece, piece_shape};

use super::{GAP, GameRenderView, RenderStyle, cell_color, cell_texture_index, textured_cell_tint};

fn piece_color(kind: Tetromino, style: RenderStyle) -> Color32 {
    cell_color(cell_from_piece(kind), style)
}

fn draw_cell(painter: &Painter, pos: Pos2, size: f32, color: Color32, style: RenderStyle) {
    let rect = Rect::from_min_size(pos, Vec2::new(size, size));
    let cr = CornerRadius::same(style.corner_radius);
    if color == style.empty_cell {
        if style.empty_stroke.a() > 0 {
            painter.rect_stroke(
                rect,
                cr,
                Stroke::new(1.0, style.empty_stroke),
                egui::StrokeKind::Middle,
            );
        }
    } else {
        painter.rect_filled(rect, cr, color);
        painter.rect_stroke(
            rect,
            cr,
            Stroke::new(1.0, style.filled_stroke),
            egui::StrokeKind::Middle,
        );
    }
}

fn draw_textured_cell(
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

fn render_board_cells(
    painter: &Painter,
    view: &GameRenderView<'_>,
    game: &GameState,
    current: PieceState,
    ghost: Option<PieceState>,
    origin: Pos2,
    cell_size: f32,
    style: RenderStyle,
) {
    let board = &game.board;
    let hidden = board.hidden_rows;
    let total = board.total_height();
    let step = cell_size + GAP;

    let no_grid = style.empty_stroke.a() == 0;

    for y in hidden..total {
        for x in 0..board.width {
            let cell = board.get(x, y);
            if no_grid && cell == crate::core::Cell::Empty {
                continue;
            }
            let color = cell_color(cell, style);
            let pos = Pos2::new(
                origin.x + x as f32 * step,
                origin.y + (y - hidden) as f32 * step,
            );
            if !draw_textured_cell(painter, view, pos, cell_size, cell, color) {
                draw_cell(painter, pos, cell_size, color, style);
            } else if cell == crate::core::Cell::Empty {
                let rect = Rect::from_min_size(pos, Vec2::new(cell_size, cell_size));
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(style.corner_radius),
                    egui::Stroke::new(1.0, style.empty_stroke),
                    egui::StrokeKind::Middle,
                );
            }
        }
    }

    if let Some(ghost_piece) = ghost {
        let shape = piece_shape(ghost_piece.kind, ghost_piece.rotation);
        for sx in 0..4i32 {
            for sy in 0..4i32 {
                if shape[sx as usize][sy as usize] {
                    let bx = ghost_piece.x + sx;
                    let by = ghost_piece.y + sy;
                    if by >= hidden as i32 && by < total as i32 {
                        let pos = Pos2::new(
                            origin.x + bx as f32 * step,
                            origin.y + (by - hidden as i32) as f32 * step,
                        );
                        if style.colored_ghost {
                            if let (Some(texture_handle), Some(atlas)) =
                                (view.texture_handle, view.texture_atlas)
                            {
                                let rect = Rect::from_min_size(pos, Vec2::new(cell_size, cell_size));
                                painter.image(
                                    texture_handle.id(),
                                    rect,
                                    atlas.tile_uv_rect(cell_texture_index(
                                        cell_from_piece(ghost_piece.kind),
                                    )),
                                    Color32::from_white_alpha(style.ghost_alpha),
                                );
                            } else {
                                let c = piece_color(ghost_piece.kind, style);
                                let ghost_color = Color32::from_rgba_premultiplied(
                                    c.r(), c.g(), c.b(), style.ghost_alpha,
                                );
                                draw_cell(painter, pos, cell_size, ghost_color, style);
                            }
                        } else {
                            draw_cell(painter, pos, cell_size, style.ghost_cell, style);
                            let _ = draw_textured_cell(
                                painter,
                                view,
                                pos,
                                cell_size,
                                crate::core::Cell::Ghost,
                                style.ghost_cell,
                            );
                        }
                    }
                }
            }
        }
    }

    let shape = piece_shape(current.kind, current.rotation);
    let color = piece_color(current.kind, style);
    for sx in 0..4i32 {
        for sy in 0..4i32 {
            if shape[sx as usize][sy as usize] {
                let bx = current.x + sx;
                let by = current.y + sy;
                if by >= hidden as i32 && by < total as i32 {
                    let pos = Pos2::new(
                        origin.x + bx as f32 * step,
                        origin.y + (by - hidden as i32) as f32 * step,
                    );
                    if !draw_textured_cell(
                        painter,
                        view,
                        pos,
                        cell_size,
                        cell_from_piece(current.kind),
                        color,
                    ) {
                        draw_cell(painter, pos, cell_size, color, style);
                    }
                }
            }
        }
    }
}

fn render_clear_flash(
    painter: &Painter,
    game: &GameState,
    flash: &ClearFlashState,
    origin: Pos2,
    cell_size: f32,
) {
    let step = cell_size + GAP;
    let hidden = game.board.hidden_rows;
    let progress = (flash.timer_ms as f32 / flash.duration_ms as f32).min(1.0);
    let alpha = (255.0 * (1.0 - progress)) as u8;
    let color = Color32::from_rgba_premultiplied(255, 255, 255, alpha);

    for &row in &flash.lines {
        if row >= hidden {
            let y = (row - hidden) as f32 * step;
            for x in 0..game.board.width {
                let pos = Pos2::new(origin.x + x as f32 * step, origin.y + y);
                let rect = Rect::from_min_size(pos, Vec2::new(cell_size, cell_size));
                painter.rect_filled(rect, CornerRadius::same(2), color);
            }
        }
    }
}

fn render_highlights(
    painter: &Painter,
    highlights: &HighlightBoard,
    style: RenderStyle,
    hidden_rows: usize,
    origin: Pos2,
    cell_size: f32,
) {
    let step = cell_size + GAP;
    let edge = (cell_size / 8.0).clamp(2.0, 4.0);

    for y in hidden_rows..highlights.total_height {
        for x in 0..highlights.width {
            let alpha = highlights.get(x, y);
            if alpha == 0 {
                continue;
            }

            let pos = Pos2::new(
                origin.x + x as f32 * step,
                origin.y + (y - hidden_rows) as f32 * step,
            );
            let rect = Rect::from_min_size(pos, Vec2::new(cell_size, cell_size));
            let fill = Color32::from_rgba_premultiplied(
                style.text_color.r(),
                style.text_color.g(),
                style.text_color.b(),
                alpha.saturating_div(5),
            );
            let edge_color = Color32::from_rgba_premultiplied(
                style.text_color.r(),
                style.text_color.g(),
                style.text_color.b(),
                alpha,
            );
            painter.rect_filled(rect, CornerRadius::same(2), fill);

            if x == 0 || highlights.get(x - 1, y) != alpha {
                painter.rect_filled(
                    Rect::from_min_size(pos, Vec2::new(edge, cell_size)),
                    CornerRadius::ZERO,
                    edge_color,
                );
            }
            if x + 1 >= highlights.width || highlights.get(x + 1, y) != alpha {
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(pos.x + cell_size - edge, pos.y),
                        Vec2::new(edge, cell_size),
                    ),
                    CornerRadius::ZERO,
                    edge_color,
                );
            }
            if y == hidden_rows || highlights.get(x, y - 1) != alpha {
                painter.rect_filled(
                    Rect::from_min_size(pos, Vec2::new(cell_size, edge)),
                    CornerRadius::ZERO,
                    edge_color,
                );
            }
            if y + 1 >= highlights.total_height || highlights.get(x, y + 1) != alpha {
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(pos.x, pos.y + cell_size - edge),
                        Vec2::new(cell_size, edge),
                    ),
                    CornerRadius::ZERO,
                    edge_color,
                );
            }
        }
    }
}

fn overlay_status_stack(painter: &Painter, rect: Rect, view: &GameRenderView<'_>) {
    let mut y = rect.top() + 10.0;

    if view.paused {
        painter.text(
            Pos2::new(rect.center().x, y),
            Align2::CENTER_TOP,
            "PAUSED",
            FontId::proportional(22.0),
            view.style.text_color,
        );
        y += 28.0;
    } else if view.game.stats.lost {
        painter.text(
            Pos2::new(rect.center().x, y),
            Align2::CENTER_TOP,
            "GAME OVER",
            FontId::proportional(22.0),
            Color32::RED,
        );
        y += 28.0;
    }

    if let Some(text) = &view.game.stats.attack_text {
        painter.text(
            Pos2::new(rect.center().x, y),
            Align2::CENTER_TOP,
            text,
            FontId::proportional(18.0),
            view.style.theme_colors.warning,
        );
        y += 23.0;
    } else if view.game.stats.perfect_clear {
        painter.text(
            Pos2::new(rect.center().x, y),
            Align2::CENTER_TOP,
            "PERFECT CLEAR",
            FontId::proportional(18.0),
            view.style.theme_colors.success,
        );
        y += 23.0;
    }

    if let Some(text) = &view.game.stats.b2b_text {
        painter.text(
            Pos2::new(rect.center().x, y),
            Align2::CENTER_TOP,
            text,
            FontId::proportional(18.0),
            view.style.theme_colors.success,
        );
    } else if view.game.stats.b2b {
        painter.text(
            Pos2::new(rect.center().x, y),
            Align2::CENTER_TOP,
            "B2B",
            FontId::proportional(18.0),
            view.style.theme_colors.success,
        );
    }
}

pub fn board_rect(ui: &egui::Ui, game: &GameState) -> Rect {
    Rect::from_min_size(
        ui.cursor().min,
        super::board_size(&game.board, ui.spacing().interact_size.y.max(1.0) - GAP),
    )
}

fn pointer_to_cell(
    pointer_pos: Pos2,
    origin: Pos2,
    cell_size: f32,
    hidden_rows: usize,
) -> Option<(usize, usize)> {
    let step = cell_size + GAP;
    let dx = pointer_pos.x - origin.x;
    let dy = pointer_pos.y - origin.y;
    if dx < 0.0 || dy < 0.0 {
        return None;
    }
    let x = (dx / step) as usize;
    let y = (dy / step) as usize + hidden_rows;
    Some((x, y))
}

const EDIT_ALPHA: u8 = 200;

pub fn draw_board(ui: &mut egui::Ui, view: &GameRenderView<'_>, actions: &mut Vec<AppAction>) {
    let rect = Rect::from_min_size(
        ui.cursor().min,
        super::board_size(&view.game.board, view.board_cell_size),
    );
    let sense = egui::Sense::click_and_drag();
    let (response, painter) = ui.allocate_painter(rect.size(), sense);

    painter.rect_filled(
        response.rect.expand(6.0),
        CornerRadius::same(8),
        view.style.background,
    );
    painter.rect_stroke(
        response.rect.expand(6.0),
        CornerRadius::same(8),
        Stroke::new(2.0, view.style.box_color),
        egui::StrokeKind::Outside,
    );

    render_board_cells(
        &painter,
        view,
        view.game,
        view.game.current,
        view.ghost,
        response.rect.min,
        view.board_cell_size,
        view.style,
    );

    if view.game.clear_flash.active {
        render_clear_flash(
            &painter,
            view.game,
            &view.game.clear_flash,
            response.rect.min,
            view.board_cell_size,
        );
    }

    render_highlights(
        &painter,
        &view.game.highlights,
        view.style,
        view.game.board.hidden_rows,
        response.rect.min,
        view.board_cell_size,
    );

    overlay_status_stack(&painter, response.rect, view);

    let ctrl = ui.input(|input| input.modifiers.ctrl);
    let primary_drag = response.drag_started_by(egui::PointerButton::Primary)
        || response.dragged_by(egui::PointerButton::Primary);
    let secondary_drag = response.drag_started_by(egui::PointerButton::Secondary)
        || response.dragged_by(egui::PointerButton::Secondary)
        || response.clicked_by(egui::PointerButton::Secondary);

    if primary_drag || secondary_drag {
        if let Some(pos) = response.interact_pointer_pos() {
            if let Some((cx, cy)) = pointer_to_cell(
                pos,
                response.rect.min,
                view.board_cell_size,
                view.game.board.hidden_rows,
            ) {
                if cy < view.game.board.total_height() && cx < view.game.board.width {
                    if view.ui_state.highlight_mode {
                        let alpha = if ctrl || secondary_drag { 0 } else { EDIT_ALPHA };
                        actions.push(AppAction::EditHighlightCell(cx as i32, cy as i32, alpha));
                    } else {
                        let cell = if ctrl || secondary_drag {
                            Cell::Empty
                        } else {
                            view.ui_state.edit_color
                        };
                        actions.push(AppAction::EditCell(cx as i32, cy as i32, cell));
                    }
                }
            }
        }
    }
}
