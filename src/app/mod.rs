pub mod actions;
mod commands;
pub mod controller;
mod dialogs;
mod screenshot_crop;
pub mod ui_state;
mod view;

use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use egui::{ColorImage, TextureHandle, TextureOptions};
use image::RgbaImage;

use self::controller::GameController;
use self::dialogs::{render_bag_edit_dialog, render_hold_edit_dialog};
use self::screenshot_crop::{ScreenshotCropAction, ScreenshotCropState};
use self::ui_state::{AppAction, UiState};
use crate::config::{AppConfig, PersistedBagMode, ThemeMode, load_static_queue};
use crate::core::QueueGenerator;
use crate::core::game_loop::GameLoop;
use crate::core::game_state::GameState;
use crate::core::highlight::auto_color_board;
use crate::core::input::collect_keyboard_actions;
use crate::core::{BagMode, GameMode};
use crate::core::{
    decode_json_state, decode_legacy_clipboard, encode_json_state, piece_from_name, piece_name,
};
use crate::media::audio::{AudioCommand, AudioManagerHandle, AudioSink, RodioAudioManager};
use crate::platform::{
    PlatformEvent, ScreenshotAnalysisConfig, ScreenshotPortalService, ScreenshotRequester,
    SystemClipboard, analyze_board_image, import_board_from_analysis, screenshot_analyzer,
};
use crate::render::{
    GameRenderView, RenderStyle, TextureAtlas, draw_board, draw_next_panel, draw_settings_window,
    draw_sidebar, fa_btn, load_texture_atlas, load_texture_atlas_bytes,
};
use crate::resources;

pub struct AppState {
    pub game_loop: GameLoop,
    pub ui_state: UiState,
    pub paused: bool,
}

impl AppState {
    pub fn is_gameplay_active(&self) -> bool {
        if self.paused {
            return false;
        }
        !self.game_loop.game.stats.lost
    }
}

pub struct FourTrisApp {
    state: AppState,
    config: AppConfig,
    controller: GameController,
    audio_manager: RodioAudioManager,
    audio_handle: AudioManagerHandle,
    audio_commands: crossbeam_channel::Receiver<AudioCommand>,
    platform_events: Receiver<PlatformEvent>,
    screenshot_service: ScreenshotPortalService,
    texture_atlas: Option<TextureAtlas>,
    texture_handle: Option<TextureHandle>,
    loaded_texture_name: Option<String>,
    last_time: Instant,
    screenshot_crop: Option<ScreenshotCropState>,
    waiting_for_screenshot: bool,
    bag_edit_text: String,
    hold_edit_text: String,
    #[cfg(target_os = "windows")]
    last_dark_mode: Option<bool>,
}

impl Default for FourTrisApp {
    fn default() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let mut state = AppState {
            game_loop: GameLoop::new_with_board(
                GameMode::Training,
                BagMode::SevenBag,
                0,
                true,
                config.static_bag,
                config.board_width,
                config.board_visible_height,
                config.board_hidden_rows,
            ),
            ui_state: UiState::default(),
            paused: false,
        };
        state.game_loop.spawn_next();
        FourTrisApp::apply_config_to_state(&mut state, &config);
        let (platform_tx, platform_events) = crossbeam_channel::unbounded();
        let screenshot_service = match ScreenshotPortalService::new(platform_tx.clone()) {
            Ok(service) => service,
            Err(_) => ScreenshotPortalService::unavailable(platform_tx),
        };
        let audio_manager =
            RodioAudioManager::load_default_audio_assets(&resources::data_audio_dir());
        let (audio_handle, audio_commands) = RodioAudioManager::command_channel();
        audio_manager.set_volume(config.volume_percent);
        let mut app = FourTrisApp {
            state,
            config,
            controller: GameController::default(),
            audio_manager,
            audio_handle,
            audio_commands,
            platform_events,
            screenshot_service,
            texture_atlas: None,
            texture_handle: None,
            loaded_texture_name: None,
            last_time: Instant::now(),
            screenshot_crop: None,
            waiting_for_screenshot: false,
            bag_edit_text: String::new(),
            hold_edit_text: String::new(),
            #[cfg(target_os = "windows")]
            last_dark_mode: None,
        };
        if !app.screenshot_service.is_available() {
            app.set_status("Screenshot service unavailable");
        }
        app
    }
}

impl FourTrisApp {
    pub fn request_interactive_capture(&self) {
        self.screenshot_service.request_interactive_capture();
    }

    fn set_status(&mut self, message: impl Into<String>) {
        self.state.ui_state.status_message = Some(message.into());
    }

    fn save_config(&mut self) {
        if let Err(error) = self.config.save() {
            self.set_status(format!("Failed to save settings: {error}"));
        }
    }

    fn resolve_theme(&self, ctx: &egui::Context) -> egui::Theme {
        match self.config.theme {
            ThemeMode::Light => egui::Theme::Light,
            ThemeMode::Dark => egui::Theme::Dark,
            ThemeMode::Auto => ctx.system_theme().unwrap_or(egui::Theme::Dark),
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) -> egui::Theme {
        let theme = self.resolve_theme(ctx);
        ctx.set_theme(theme);
        theme
    }

    #[cfg(target_os = "windows")]
    fn apply_dark_mode(&mut self, frame: &eframe::Frame, theme: egui::Theme) {
        let dark = theme == egui::Theme::Dark;
        if self.last_dark_mode == Some(dark) {
            return;
        }
        self.last_dark_mode = Some(dark);

        use raw_window_handle::HasWindowHandle;

        let Some(window) = frame.winit_window() else {
            return;
        };
        let Ok(handle) = window.window_handle() else {
            return;
        };
        let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() else {
            return;
        };
        crate::platform::set_window_dark_mode(win32.hwnd.get() as *mut std::ffi::c_void, dark);
    }

    fn bag_mode_from_config(config: &AppConfig) -> BagMode {
        match config.bag_mode {
            PersistedBagMode::SevenBag => BagMode::SevenBag,
            PersistedBagMode::FourteenBag => BagMode::FourteenBag,
            PersistedBagMode::Random => BagMode::Random,
        }
    }

    fn apply_runtime_config(state: &mut AppState, config: &AppConfig) {
        state.game_loop.game.gravity.das_ms = config.das_ms;
        state.game_loop.game.gravity.arr_ms = config.arr_ms;
        state.game_loop.game.gravity.sdd_ms = config.sdd_ms;
        state.game_loop.game.gravity.sds_ms = config.sds_ms;
        state.game_loop.game.gravity.das_cancel = config.das_cancel;
        state.game_loop.game.ghost_piece = config.show_ghost;
        state.game_loop.game.hold.infinite_hold = config.infinite_hold;
        state.game_loop.game.auto_color = config.auto_color;
        state.game_loop.game.auto_lock_on_ground = config.auto_lock_on_ground;
        state.game_loop.game.mirror_queue_with_field = config.mirror_queue;
        state.game_loop.game.highlight_clear = config.highlight_clear;
        state.ui_state.board_cell_size = f32::from(config.board_cell_size);
        state.ui_state.preview_cell_size = f32::from(config.preview_cell_size);
    }

    fn apply_config_to_state(state: &mut AppState, config: &AppConfig) -> bool {
        let desired_bag_mode = Self::bag_mode_from_config(config);
        let needs_rebuild = state.game_loop.game.board.width != config.board_width
            || state.game_loop.game.board.visible_height != config.board_visible_height
            || state.game_loop.game.board.hidden_rows != config.board_hidden_rows
            || state.game_loop.game.queue.mode != desired_bag_mode;

        if needs_rebuild {
            let mode = state.game_loop.game.mode;
            let seed = state.game_loop.game.queue.seed;
            state.game_loop = GameLoop::new_with_board(
                mode,
                desired_bag_mode,
                seed,
                config.infinite_hold,
                config.static_bag,
                config.board_width,
                config.board_visible_height,
                config.board_hidden_rows,
            );
            state.game_loop.spawn_next();
        }

        Self::apply_runtime_config(state, config);
        needs_rebuild
    }

    fn sync_texture_atlas(&mut self, ctx: &egui::Context) {
        if self.loaded_texture_name.as_deref() == Some(self.config.selected_texture.as_str()) {
            return;
        }

        let data_path = resources::data_textures_dir().join(&self.config.selected_texture);
        let atlas = if data_path.is_file() {
            load_texture_atlas(&data_path)
        } else if let Some(bytes) = resources::builtin_texture_bytes(&self.config.selected_texture)
        {
            load_texture_atlas_bytes(&self.config.selected_texture, bytes)
        } else {
            Err(crate::render::TextureError::Io(format!(
                "missing texture {}",
                self.config.selected_texture
            )))
        };

        match atlas {
            Ok(atlas) => {
                let size = [atlas.image.width() as usize, atlas.image.height() as usize];
                let color_image = ColorImage::from_rgba_unmultiplied(size, atlas.image.as_raw());
                let texture_handle = ctx.load_texture(
                    format!("texture-atlas:{}", self.config.selected_texture),
                    color_image,
                    TextureOptions::LINEAR,
                );
                self.texture_atlas = Some(atlas);
                self.texture_handle = Some(texture_handle);
                self.loaded_texture_name = Some(self.config.selected_texture.clone());
            }
            Err(error) => {
                self.texture_atlas = None;
                self.texture_handle = None;
                self.loaded_texture_name = None;
                self.set_status(format!("Texture load failed: {error}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, FourTrisApp, UiState};
    use crate::config::{AppConfig, PersistedBagMode};
    use crate::core::game_loop::GameLoop;
    use crate::core::{BagMode, GameMode};

    fn test_state() -> AppState {
        let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 0, true);
        game_loop.spawn_next();
        AppState {
            game_loop,
            ui_state: UiState::default(),
            paused: false,
        }
    }

    #[test]
    fn apply_config_to_state_updates_runtime_fields() {
        let mut state = test_state();
        let config = AppConfig {
            show_ghost: false,
            das_ms: 85,
            arr_ms: 1,
            sdd_ms: 2,
            sds_ms: 3,
            das_cancel: true,
            board_cell_size: 32,
            preview_cell_size: 16,
            infinite_hold: false,
            auto_color: true,
            mirror_queue: true,
            highlight_clear: true,
            ..AppConfig::default()
        };

        let rebuilt = FourTrisApp::apply_config_to_state(&mut state, &config);

        assert!(!rebuilt);
        assert_eq!(state.game_loop.game.gravity.das_ms, 85);
        assert_eq!(state.game_loop.game.gravity.arr_ms, 1);
        assert_eq!(state.game_loop.game.gravity.sdd_ms, 2);
        assert_eq!(state.game_loop.game.gravity.sds_ms, 3);
        assert!(state.game_loop.game.gravity.das_cancel);
        assert!(!state.game_loop.game.ghost_piece);
        assert!(!state.game_loop.game.hold.infinite_hold);
        assert!(state.game_loop.game.auto_color);
        assert!(state.game_loop.game.mirror_queue_with_field);
        assert!(state.game_loop.game.highlight_clear);
        assert_eq!(state.ui_state.board_cell_size, 32.0);
        assert_eq!(state.ui_state.preview_cell_size, 16.0);
    }

    #[test]
    fn apply_config_to_state_rebuilds_board_and_bag_mode() {
        let mut state = test_state();
        let config = AppConfig {
            board_width: 12,
            board_visible_height: 22,
            board_hidden_rows: 3,
            bag_mode: PersistedBagMode::Random,
            ..AppConfig::default()
        };

        let rebuilt = FourTrisApp::apply_config_to_state(&mut state, &config);

        assert!(rebuilt);
        assert_eq!(state.game_loop.game.board.width, 12);
        assert_eq!(state.game_loop.game.board.visible_height, 22);
        assert_eq!(state.game_loop.game.board.hidden_rows, 3);
        assert_eq!(state.game_loop.game.queue.mode, BagMode::Random);
    }
}

impl eframe::App for FourTrisApp {
    fn ui(&mut self, app_ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = app_ui.ctx().clone();
        let _theme = self.apply_theme(&ctx);
        #[cfg(target_os = "windows")]
        self.apply_dark_mode(frame, _theme);
        commands::poll_platform_events(self);

        let now = Instant::now();
        let dt = (now - self.last_time).as_secs_f64();
        self.last_time = now;
        let capped_dt = dt.min(0.1);
        let capped_now = now - Duration::from_secs_f64(dt - capped_dt);

        let focused = ctx.input(|i| i.focused);
        if !focused {
            self.controller.clear_repeat_state();
            self.state.ui_state.previous_keys.clear();
        }

        let actions = view::collect_actions(self, &ctx, now);
        for action in actions {
            commands::handle_app_action(self, action);
        }

        self.controller.advance_timers(&mut self.state, capped_now);

        if self.state.is_gameplay_active() {
            let dt_ms = (capped_dt * 1000.0) as u32;
            self.state.game_loop.tick(dt_ms);
        }

        for effect in self.state.game_loop.pending_sounds.drain(..) {
            self.audio_handle.send(AudioCommand::Play(effect));
        }
        self.audio_manager.process_commands(&self.audio_commands);

        egui::CentralPanel::default().show(app_ui, |ui| {
            view::render_ui(self, &ctx, ui);
        });

        view::render_screenshot_crop_window_ui(self, &ctx);

        if let Some(window) = frame.winit_window() {
            let native_wayland =
                std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("DISPLAY").is_err();
            if !native_wayland {
                window.set_visible(!self.waiting_for_screenshot);
            }
        }

        if self.screenshot_crop.is_none() {
            ctx.request_repaint();
        }
    }
}
