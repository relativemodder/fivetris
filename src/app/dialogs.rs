use super::actions::AppAction;
use crate::render::{btn, text_edit_singleline};

fn render_piece_text_dialog(
    ctx: &egui::Context,
    title: &str,
    label: &str,
    is_open: &mut bool,
    text: &mut String,
    actions: &mut Vec<AppAction>,
    apply_action: fn(String) -> AppAction,
    cancel_action: AppAction,
    anchor: egui::Align2,
) {
    egui::Window::new(title)
        .anchor(anchor, egui::vec2(12.0, 64.0))
        .resizable(false)
        .open(is_open)
        .show(ctx, |ui| {
            ui.label(label);
            text_edit_singleline(ui, text);
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 40.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if btn(ui, "Cancel").clicked() {
                        actions.push(cancel_action.clone());
                    }
                    if btn(ui, "Apply").clicked() {
                        actions.push(apply_action(std::mem::take(text)));
                    }
                },
            );
        });
}

pub(crate) fn render_bag_edit_dialog(
    ctx: &egui::Context,
    is_open: &mut bool,
    text: &mut String,
    actions: &mut Vec<AppAction>,
) {
    render_piece_text_dialog(
        ctx,
        "Edit Bag",
        "Enter piece letters (I, J, L, M, O, S, T, Z):",
        is_open,
        text,
        actions,
        AppAction::ApplyBagEdit,
        AppAction::CancelBagEdit,
        egui::Align2::RIGHT_TOP,
    );
}

pub(crate) fn render_hold_edit_dialog(
    ctx: &egui::Context,
    is_open: &mut bool,
    text: &mut String,
    actions: &mut Vec<AppAction>,
) {
    render_piece_text_dialog(
        ctx,
        "Edit Hold",
        "Enter piece letter (I, J, L, M, O, S, T, Z):",
        is_open,
        text,
        actions,
        AppAction::ApplyHoldEdit,
        AppAction::CancelHoldEdit,
        egui::Align2::LEFT_TOP,
    );
}
