use super::*;

pub(crate) fn handle_app_action(app: &mut FourTrisApp, action: AppAction) -> bool {
    match action {
        AppAction::ToggleGhost(enabled) => {
            app.config.show_ghost = enabled;
            app.state.game_loop.game.ghost_piece = enabled;
            app.save_config();
        }
        AppAction::ToggleAutoColor(enabled) => {
            app.config.auto_color = enabled;
            app.state.game_loop.game.auto_color = enabled;
            app.save_config();
        }
        AppAction::ToggleAutoLockOnGround(enabled) => {
            app.config.auto_lock_on_ground = enabled;
            app.state.game_loop.game.auto_lock_on_ground = enabled;
            app.save_config();
        }
        AppAction::SetVolume(volume_percent) => {
            app.config.volume_percent = volume_percent.min(100);
            app.audio_manager.set_volume(app.config.volume_percent);
            app.save_config();
        }
        AppAction::SelectSkin(name) => {
            app.config.selected_skin = name;
            app.save_config();
            app.set_status("Updated skin selection hook");
        }
        AppAction::SelectTexture(name) => {
            app.config.selected_texture = name;
            app.save_config();
            app.set_status("Updated texture selection hook");
        }
        AppAction::ApplySettings => {
            if let Some(pending) = app.state.ui_state.pending_config.clone() {
                app.config = pending;
                let rebuilt = FourTrisApp::apply_config_to_state(&mut app.state, &app.config);
                app.audio_manager.set_volume(app.config.volume_percent);
                app.save_config();
                if rebuilt {
                    app.set_status("Applied settings and reset board");
                }
            }
        }
        AppAction::ConfirmSettings => {
            if let Some(pending) = app.state.ui_state.pending_config.take() {
                app.config = pending;
                let rebuilt = FourTrisApp::apply_config_to_state(&mut app.state, &app.config);
                app.audio_manager.set_volume(app.config.volume_percent);
                app.save_config();
                if rebuilt {
                    app.set_status("Applied settings and reset board");
                }
            }
            app.state.ui_state.pending_config = None;
        }
        AppAction::CancelSettings => {
            app.state.ui_state.pending_config = None;
        }
        _ => return false,
    }
    true
}
