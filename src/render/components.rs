use egui::Color32;

pub const CONTROL_PADDING: egui::Vec2 = egui::vec2(20.0, 7.0);
pub const COMPACT_CONTROL_PADDING: egui::Vec2 = egui::Vec2::ZERO;

fn input_margin() -> egui::Margin {
    let padding = CONTROL_PADDING.y as i8;
    egui::Margin::symmetric(padding, padding)
}

pub fn with_control_padding<R>(
    ui: &mut egui::Ui,
    padding: egui::Vec2,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let saved = ui.style().spacing.button_padding;
    ui.style_mut().spacing.button_padding = padding;
    let result = add_contents(ui);
    ui.style_mut().spacing.button_padding = saved;
    result
}

pub fn apply_global_style(ctx: &egui::Context, theme: egui::Theme) {
    let mut style = (*ctx.style_of(theme)).clone();
    style.spacing.button_padding = CONTROL_PADDING;
    style.spacing.interact_size.y = style.spacing.interact_size.y.max(32.0);
    ctx.set_style_of(theme, style);
}

fn darken(color: Color32, amount: u8) -> Color32 {
    Color32::from_rgba_premultiplied(
        color.r().saturating_sub(amount),
        color.g().saturating_sub(amount),
        color.b().saturating_sub(amount),
        color.a(),
    )
}

pub fn styled_button(ui: &mut egui::Ui, button: egui::Button<'_>) -> egui::Response {
    let saved_hovered = ui.style().visuals.widgets.hovered;
    let saved_active = ui.style().visuals.widgets.active;
    let base_fill = ui.style().visuals.widgets.inactive.weak_bg_fill;
    let hover_fill = darken(base_fill, 18);
    let active_fill = darken(base_fill, 34);
    ui.style_mut().visuals.widgets.hovered = ui.style().visuals.widgets.inactive;
    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = hover_fill;
    ui.style_mut().visuals.widgets.hovered.bg_fill = hover_fill;
    ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
    ui.style_mut().visuals.widgets.active = ui.style().visuals.widgets.hovered;
    ui.style_mut().visuals.widgets.active.weak_bg_fill = active_fill;
    ui.style_mut().visuals.widgets.active.bg_fill = active_fill;
    ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    ui.style_mut().visuals.widgets.active.expansion = 0.0;
    let r = ui.add(button);
    ui.style_mut().visuals.widgets.hovered = saved_hovered;
    ui.style_mut().visuals.widgets.active = saved_active;
    r
}

pub fn text_edit_singleline(ui: &mut egui::Ui, text: &mut String) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .desired_width(f32::INFINITY)
            .margin(input_margin()),
    )
}

pub fn btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    styled_button(ui, egui::Button::new(text))
}

pub fn fa_btn(ui: &mut egui::Ui, icon: char, text: &str) -> egui::Response {
    styled_button(ui, egui::Button::new(format!("{icon}  {text}")))
}

pub fn fa_icon_btn(ui: &mut egui::Ui, icon: char, tooltip: &str) -> egui::Response {
    styled_button(ui, egui::Button::new(icon.to_string()).min_size(egui::vec2(34.0, 34.0)))
        .on_hover_text(tooltip)
}

pub fn fa_icon_btn_small(ui: &mut egui::Ui, icon: char, tooltip: &str) -> egui::Response {
    let saved_hovered = ui.style().visuals.widgets.hovered;
    let saved_active = ui.style().visuals.widgets.active;
    let base_fill = ui.style().visuals.widgets.inactive.weak_bg_fill;
    let hover_fill = darken(base_fill, 18);
    let active_fill = darken(base_fill, 34);
    ui.style_mut().visuals.widgets.hovered = ui.style().visuals.widgets.inactive;
    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = hover_fill;
    ui.style_mut().visuals.widgets.hovered.bg_fill = hover_fill;
    ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
    ui.style_mut().visuals.widgets.active = ui.style().visuals.widgets.hovered;
    ui.style_mut().visuals.widgets.active.weak_bg_fill = active_fill;
    ui.style_mut().visuals.widgets.active.bg_fill = active_fill;
    ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    ui.style_mut().visuals.widgets.active.expansion = 0.0;
    let r = with_control_padding(ui, egui::vec2(6.0, 6.0), |ui| {
        ui.add(egui::Button::new(icon.to_string()).min_size(egui::vec2(28.0, 28.0)))
    })
    .on_hover_text(tooltip);
    ui.style_mut().visuals.widgets.hovered = saved_hovered;
    ui.style_mut().visuals.widgets.active = saved_active;
    r
}
