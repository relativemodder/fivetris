use super::*;

#[cfg(target_os = "linux")]
fn portal_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().expect("failed to create Tokio runtime"))
}

#[cfg(target_os = "linux")]
fn export_via_portal(app: &mut FourTrisApp, raw: &str) {
    use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
    portal_runtime().block_on(async {
        let result = SelectedFiles::save_file()
            .modal(true)
            .current_name("fivetris-settings.json")
            .filter(FileFilter::new("JSON").glob("*.json"))
            .send()
            .await;
        match result {
            Ok(response) => match response.response() {
                Ok(files) => {
                    if let Some(uri) = files.uris().first() {
                        if let Some(path) = uri.to_file_path().ok() {
                            if let Err(e) = std::fs::write(&path, raw) {
                                app.set_status(format!("Export failed: {e}"));
                            } else {
                                app.set_status(format!("Exported to {}", path.display()));
                            }
                        }
                    }
                }
                Err(e) => app.set_status(format!("Export cancelled: {e}")),
            },
            Err(e) => app.set_status(format!("Export failed: {e}")),
        }
    });
}

#[cfg(target_os = "linux")]
fn import_via_portal(app: &mut FourTrisApp) {
    use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
    portal_runtime().block_on(async {
        let result = SelectedFiles::open_file()
            .modal(true)
            .filter(FileFilter::new("JSON").glob("*.json"))
            .send()
            .await;
        match result {
            Ok(response) => match response.response() {
                Ok(files) => {
                    if let Some(uri) = files.uris().first() {
                        if let Some(path) = uri.to_file_path().ok() {
                            match std::fs::read_to_string(&path) {
                                Ok(raw) => match serde_json::from_str(&raw) {
                                    Ok(config) => {
                                        app.state.ui_state.pending_config = Some(config);
                                        app.set_status("Settings imported");
                                    }
                                    Err(e) => app.set_status(format!("Invalid settings file: {e}")),
                                },
                                Err(e) => app.set_status(format!("Failed to read file: {e}")),
                            }
                        }
                    }
                }
                Err(e) => app.set_status(format!("Import cancelled: {e}")),
            },
            Err(e) => app.set_status(format!("Import failed: {e}")),
        }
    });
}

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
        AppAction::ExportSettings => {
            let cfg = app
                .state
                .ui_state
                .pending_config
                .as_ref()
                .unwrap_or(&app.config);
            let raw = serde_json::to_string_pretty(cfg)
                .unwrap_or_else(|_| "{}".to_string());
            #[cfg(target_os = "linux")]
            export_via_portal(app, &raw);
            #[cfg(not(any(target_os = "linux", target_arch = "wasm32")))]
            {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("fivetris-settings.json")
                    .add_filter("JSON", &["json"])
                    .save_file()
                {
                    match std::fs::write(&path, &raw) {
                        Ok(()) => app.set_status(format!("Exported to {}", path.display())),
                        Err(e) => app.set_status(format!("Export failed: {e}")),
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            crate::web::export_settings(&raw);
        }
        AppAction::ImportSettings => {
            #[cfg(target_os = "linux")]
            import_via_portal(app);
            #[cfg(not(any(target_os = "linux", target_arch = "wasm32")))]
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
                {
                    match std::fs::read_to_string(&path) {
                        Ok(raw) => match serde_json::from_str(&raw) {
                            Ok(config) => {
                                app.state.ui_state.pending_config = Some(config);
                                app.set_status("Settings imported");
                            }
                            Err(e) => app.set_status(format!("Invalid settings file: {e}")),
                        },
                        Err(e) => app.set_status(format!("Failed to read file: {e}")),
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            crate::web::prompt_import_settings();
        }
        _ => return false,
    }
    true
}
