use super::*;

pub(crate) fn handle_app_action(app: &mut FourTrisApp, action: AppAction) -> bool {
    match action {
        AppAction::SetEditColor(cell) => {
            app.state.ui_state.edit_color = cell;
        }
        AppAction::BeginBrushStroke => {
            app.state.game_loop.push_snapshot();
        }
        AppAction::ClearBoard => {
            app.state.game_loop.game.board.clear_all();
            app.state.game_loop.game.highlights.clear_all();
        }
        AppAction::ToggleHighlightMode => {
            app.state.ui_state.highlight_mode = !app.state.ui_state.highlight_mode;
        }
        AppAction::ClearHighlight => {
            app.state.game_loop.game.highlights.clear_all();
        }
        AppAction::EditCell(x, y, cell) => {
            let game = &mut app.state.game_loop.game;
            if game.board.in_bounds(x, y) {
                game.board.set(x as usize, y as usize, cell);
                if game.auto_color {
                    auto_color_board(&mut game.board);
                }
            }
            app.state.game_loop.update_turn_start_snapshot();
        }
        AppAction::EditHighlightCell(x, y, alpha) => {
            let game = &mut app.state.game_loop.game;
            if game.board.in_bounds(x, y) {
                game.highlights.set(x as usize, y as usize, alpha);
            }
            app.state.game_loop.update_turn_start_snapshot();
        }
        AppAction::StartBagEdit => {
            app.bag_edit_text = app
                .state
                .game_loop
                .game
                .queue
                .visible
                .iter()
                .copied()
                .map(piece_name)
                .collect();
            app.state.ui_state.bag_edit_open = true;
        }
        AppAction::ApplyBagEdit(text) => {
            let mut pieces = Vec::new();
            let mut valid = true;
            for ch in text.chars() {
                if ch.is_whitespace() {
                    continue;
                }
                if let Some(p) = piece_from_name(ch.to_ascii_uppercase()) {
                    pieces.push(p);
                } else {
                    valid = false;
                    break;
                }
            }
            if valid && !pieces.is_empty() {
                app.state.game_loop.game.queue.visible = pieces;
                app.set_status("Bag updated");
            }
            app.state.ui_state.bag_edit_open = false;
        }
        AppAction::CancelBagEdit => {
            app.state.ui_state.bag_edit_open = false;
        }
        AppAction::StartHoldEdit(_) => {
            app.state.ui_state.hold_edit_open = true;
        }
        AppAction::ApplyHoldEdit(text) => {
            if let Some(p) = text
                .chars()
                .next()
                .and_then(|ch| piece_from_name(ch.to_ascii_uppercase()))
            {
                app.state.game_loop.game.hold.piece = Some(p);
                app.set_status("Hold updated");
            }
            app.state.ui_state.hold_edit_open = false;
        }
        AppAction::CancelHoldEdit => {
            app.state.ui_state.hold_edit_open = false;
        }
        _ => return false,
    }
    true
}
