use super::*;
#[cfg(not(target_arch = "wasm32"))]
use image::imageops;
use image::RgbaImage;

pub(crate) fn replace_game_state(app: &mut FourTrisApp, game: GameState) {
    app.state.game_loop.game = game;
    app.state.game_loop.history = crate::core::History::new(100);
    app.state.game_loop.queue_generator = match load_static_queue() {
        Ok(Some(static_sequence)) if app.config.static_bag && !static_sequence.is_empty() => {
            QueueGenerator::with_static_sequence(
                app.state.game_loop.game.queue.mode,
                app.state.game_loop.game.queue.seed,
                static_sequence,
            )
        }
        _ => QueueGenerator::new(
            app.state.game_loop.game.queue.mode,
            app.state.game_loop.game.queue.seed,
        ),
    };
    app.state.game_loop.gravity_accum = 0;
    app.state.game_loop.pending_sounds.clear();
    FourTrisApp::apply_runtime_config(&mut app.state, &app.config);
}

pub(crate) fn screenshot_analysis_config(app: &FourTrisApp) -> ScreenshotAnalysisConfig {
    ScreenshotAnalysisConfig {
        board_width: app.state.game_loop.game.board.width,
        board_visible_height: app.state.game_loop.game.board.visible_height,
        ..ScreenshotAnalysisConfig::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn start_screenshot_crop(app: &mut FourTrisApp, image: RgbaImage) {
    if app.screenshot_crop.is_some() {
        return;
    }
    let was_paused = app.state.paused;
    app.state.paused = true;
    app.controller.clear_repeat_state();
    app.screenshot_crop = Some(ScreenshotCropState::new(image, was_paused));
    app.set_status("Select an area to crop, or import the full screenshot");
}

pub(crate) fn import_screenshot_image(app: &mut FourTrisApp, rgba: &RgbaImage) {
    let config = screenshot_analysis_config(app);
    match analyze_board_image(rgba, &config) {
        Ok(analysis) => match import_board_from_analysis(&mut app.state.game_loop.game, analysis) {
            Ok(()) => app.set_status("Imported board from screenshot"),
            Err(error) => {
                app.set_status(format!("Screenshot import failed: {error:?}"));
            }
        },
        Err(error) => {
            app.set_status(format!("Screenshot analysis failed: {error:?}"));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn finish_screenshot_crop(app: &mut FourTrisApp, action: ScreenshotCropAction) {
    let Some(crop_state) = app.screenshot_crop.take() else {
        return;
    };

    app.state.paused = crop_state.was_paused;
    app.controller.clear_repeat_state();

    match action {
        ScreenshotCropAction::Cancel => {
            app.set_status("Screenshot import cancelled");
        }
        ScreenshotCropAction::UseFullImage => {
            import_screenshot_image(app, &crop_state.image);
        }
        ScreenshotCropAction::UseSelection => {
            let Some((x, y, width, height)) = crop_state.selected_pixel_rect() else {
                app.set_status("Select a non-empty crop area first");
                return;
            };
            let cropped = imageops::crop_imm(&crop_state.image, x, y, width, height).to_image();
            import_screenshot_image(app, &cropped);
        }
    }
}

pub(crate) fn handle_app_action(app: &mut FourTrisApp, action: AppAction) -> bool {
    match action {
        AppAction::CopyState => match encode_json_state(&app.state.game_loop.game) {
            Ok(serialized) => match SystemClipboard::write_text(&serialized) {
                Ok(()) => app.set_status("Copied state to clipboard"),
                Err(error) => app.set_status(format!("Copy failed: {error}")),
            },
            Err(error) => app.set_status(format!("Copy failed: {error}")),
        },
        AppAction::PasteState => match SystemClipboard::read_text() {
            Ok(text) => {
                match decode_json_state(&text).or_else(|_| decode_legacy_clipboard(&text)) {
                    Ok(game) => {
                        replace_game_state(app, game);
                        app.set_status("Pasted state from clipboard");
                    }
                    Err(error) => app.set_status(format!("Paste failed: {error}")),
                }
            }
            Err(error) => app.set_status(format!("Paste failed: {error}")),
        },
        #[cfg(not(target_arch = "wasm32"))]
        AppAction::RequestScreenshot => {
            if !app.waiting_for_screenshot {
                app.waiting_for_screenshot = true;
                app.request_interactive_capture();
            }
            app.set_status("Requested interactive screenshot");
        }
        _ => return false,
    }
    true
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn poll_platform_events(app: &mut FourTrisApp) {
    while let Ok(event) = app.platform_events.try_recv() {
        match event {
            PlatformEvent::ScreenshotReady(result) => {
                app.waiting_for_screenshot = false;
                match result {
                    Ok(image) => match screenshot_analyzer::screenshot_image_to_rgba(&image) {
                        Ok(rgba) => start_screenshot_crop(app, rgba),
                        Err(error) => {
                            app.set_status(format!("Screenshot analysis failed: {error:?}"))
                        }
                    },
                    Err(error) => {
                        app.set_status(format!("Screenshot capture failed: {error}"));
                    }
                }
            }
        }
    }
}
