use super::*;
use crate::app::screenshot_crop::render_screenshot_crop_window;

pub(crate) fn render_screenshot_crop_window_ui(app: &mut FourTrisApp, ctx: &egui::Context) {
    let Some(crop) = app.screenshot_crop.as_mut() else {
        return;
    };
    if let Some(action) = render_screenshot_crop_window(ctx, crop) {
        super::commands::finish_screenshot_crop(app, action);
    }
}

pub(crate) fn collect_actions(
    app: &mut FourTrisApp,
    ctx: &egui::Context,
    now: Instant,
) -> Vec<AppAction> {
    let mut actions = Vec::new();

    if app.state.ui_state.settings_open
        || ctx.egui_wants_keyboard_input()
        || app.screenshot_crop.is_some()
    {
        app.state.ui_state.previous_keys.clear();
        return actions;
    }

    collect_keyboard_actions(
        ctx,
        now,
        &app.config,
        &mut actions,
        &mut app.state.ui_state.previous_keys,
    );
    let shift_down = ctx.input(|input| input.modifiers.shift);
    if app.config.bindings.hold_with_shift && shift_down && !app.state.ui_state.shift_was_down {
        actions.push(AppAction::Hold);
    }
    app.state.ui_state.shift_was_down = shift_down;
    actions
}

pub(crate) fn render_ui(app: &mut FourTrisApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    app.sync_texture_atlas(ctx);

    let mode_name = match app.state.game_loop.game.mode {
        GameMode::Training => "Training",
        GameMode::Cheese => "Cheese",
        GameMode::FourWide => "FourWide",
        GameMode::PerfectClear => "PerfectClear",
        GameMode::Master => "Master",
        GameMode::Edit => "Edit",
    };
    let mode_icon = match app.state.game_loop.game.mode {
        GameMode::Training => '\u{f44b}',
        GameMode::Cheese => '\u{f7ef}',
        GameMode::FourWide => '\u{f0db}',
        GameMode::PerfectClear => '\u{f058}',
        GameMode::Master => '\u{f521}',
        GameMode::Edit => '\u{f303}',
    };

    let mut actions = Vec::<AppAction>::new();

    ui.horizontal(|ui| {
        if fa_btn(ui, '\u{f013}', "Settings").clicked() {
            app.state.ui_state.was_paused_before_settings = app.state.paused;
            app.state.paused = true;
            app.state.ui_state.settings_open = true;
            app.state.ui_state.pending_config = Some(app.config.clone());
        }
        if fa_btn(ui, '\u{f0c5}', "Copy State").clicked() {
            actions.push(AppAction::CopyState);
        }
        if fa_btn(ui, '\u{f0ea}', "Paste State").clicked() {
            actions.push(AppAction::PasteState);
        }
        if fa_btn(ui, '\u{f030}', "Screenshot").clicked() {
            actions.push(AppAction::RequestScreenshot);
        }
        if let Some(message) = &app.state.ui_state.status_message {
            ui.label(message);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if fa_btn(ui, mode_icon, &format!("Mode: {mode_name}")).clicked() {
                actions.push(AppAction::CycleMode);
            }
        });
    });
    ui.separator();

    let available_width = ui.available_width();
    let available_height = (ui.available_height() - 12.0).max(120.0);
    let sidebar_width = 220.0;
    let next_panel_width = 220.0;
    let content_gap = 16.0;
    let separator_width = 1.0;
    let sidebar_padding = 8.0;
    let board_width_cells = app.state.game_loop.game.board.width as f32;
    let board_height_cells = app.state.game_loop.game.board.visible_height as f32;
    let board_available_width = (available_width
        - sidebar_width
        - next_panel_width
        - content_gap * 2.0
        - separator_width * 2.0)
        .max(120.0);
    let fitted_board_cell_size = ((board_available_width / board_width_cells) - 1.0)
        .min((available_height / board_height_cells) - 1.0)
        .floor()
        .max(8.0);
    let fitted_preview_cell_size = app
        .state
        .ui_state
        .preview_cell_size
        .min((fitted_board_cell_size * 0.6).max(8.0));

    let view = GameRenderView {
        game: &app.state.game_loop.game,
        ghost: app
            .state
            .game_loop
            .game
            .ghost_piece
            .then(|| app.state.game_loop.ghost_position()),
        mode_name,
        paused: app.state.paused,
        style: RenderStyle::from_config(&app.config),
        texture_atlas: app.texture_atlas.as_ref(),
        texture_handle: app.texture_handle.as_ref(),
        board_cell_size: fitted_board_cell_size,
        preview_cell_size: fitted_preview_cell_size,
        ui_state: &app.state.ui_state,
    };

    let content_height = ui.available_height();
    let top_y = ui.cursor().min.y;
    let board_pixel_width = board_width_cells * (fitted_board_cell_size + 1.0);
    let board_region_start =
        sidebar_width + content_gap * 0.5 + separator_width + content_gap * 0.5;
    let board_region_end = available_width
        - next_panel_width
        - content_gap * 0.5
        - separator_width
        - content_gap * 0.5;
    let board_leading =
        ((board_region_end - board_region_start - board_pixel_width) * 0.5).max(0.0);
    let right_gutter = available_width - next_panel_width;

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_size(
                egui::pos2(0.0, top_y),
                egui::vec2(sidebar_width, content_height),
            ))
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(sidebar_padding as i8, 0))
                .show(ui, |ui| {
                    draw_sidebar(ui, &view, &mut actions);
                });
        },
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_size(
                egui::pos2(board_region_start + board_leading, top_y),
                egui::vec2(board_pixel_width, content_height),
            ))
            .layout(egui::Layout::top_down(egui::Align::Center)),
        |ui| {
            draw_board(ui, &view, &mut actions);
        },
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_size(
                egui::pos2(right_gutter, top_y),
                egui::vec2(next_panel_width, content_height),
            ))
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(sidebar_padding as i8, 0))
                .show(ui, |ui| {
                    draw_next_panel(ui, &view, &mut actions);
                });
        },
    );

    ui.painter().vline(
        sidebar_width + content_gap * 0.5,
        top_y..=(top_y + content_height),
        ui.visuals().widgets.noninteractive.bg_stroke,
    );
    ui.painter().vline(
        available_width - next_panel_width - content_gap * 0.5 - separator_width,
        top_y..=(top_y + content_height),
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    let was_settings_open = app.state.ui_state.settings_open;
    draw_settings_window(ctx, &mut app.state.ui_state, &mut actions);
    if was_settings_open && !app.state.ui_state.settings_open {
        app.state.paused = app.state.ui_state.was_paused_before_settings;
    }

    if app.state.ui_state.bag_edit_open {
        render_bag_edit_dialog(
            ctx,
            &mut app.state.ui_state.bag_edit_open,
            &mut app.bag_edit_text,
            &mut actions,
        );
    }

    if app.state.ui_state.hold_edit_open {
        render_hold_edit_dialog(
            ctx,
            &mut app.state.ui_state.hold_edit_open,
            &mut app.hold_edit_text,
            &mut actions,
        );
    }

    for action in actions {
        super::commands::handle_app_action(app, action);
    }
}
