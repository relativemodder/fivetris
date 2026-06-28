use egui::load::SizedTexture;
use egui::{ColorImage, TextureHandle, TextureOptions};
use image::RgbaImage;

use crate::render::{btn, styled_button};

pub(crate) struct ScreenshotCropState {
    pub(crate) image: RgbaImage,
    texture: Option<TextureHandle>,
    drag_start_uv: Option<egui::Pos2>,
    selection_uv: Option<egui::Rect>,
    pub(crate) was_paused: bool,
}

impl ScreenshotCropState {
    pub(crate) fn new(image: RgbaImage, was_paused: bool) -> Self {
        Self {
            image,
            texture: None,
            drag_start_uv: None,
            selection_uv: None,
            was_paused,
        }
    }

    fn ensure_texture(&mut self, ctx: &egui::Context) {
        if self.texture.is_some() {
            return;
        }
        let size = [self.image.width() as usize, self.image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, self.image.as_raw());
        self.texture = Some(ctx.load_texture(
            "screenshot-crop-preview",
            color_image,
            TextureOptions::LINEAR,
        ));
    }

    fn texture(&self) -> Option<SizedTexture> {
        self.texture.as_ref().map(SizedTexture::from_handle)
    }

    fn pointer_to_uv(image_rect: egui::Rect, pointer_pos: egui::Pos2) -> egui::Pos2 {
        egui::pos2(
            ((pointer_pos.x - image_rect.min.x) / image_rect.width()).clamp(0.0, 1.0),
            ((pointer_pos.y - image_rect.min.y) / image_rect.height()).clamp(0.0, 1.0),
        )
    }

    fn update_selection_from_pointer(&mut self, image_rect: egui::Rect, pointer_pos: egui::Pos2) {
        let uv = Self::pointer_to_uv(image_rect, pointer_pos);
        if let Some(start) = self.drag_start_uv {
            self.selection_uv = Some(egui::Rect::from_two_pos(start, uv));
        }
    }

    fn selection_screen_rect(&self, image_rect: egui::Rect) -> Option<egui::Rect> {
        let selection = self.selection_uv?;
        Some(egui::Rect::from_min_max(
            egui::pos2(
                image_rect.left() + selection.min.x * image_rect.width(),
                image_rect.top() + selection.min.y * image_rect.height(),
            ),
            egui::pos2(
                image_rect.left() + selection.max.x * image_rect.width(),
                image_rect.top() + selection.max.y * image_rect.height(),
            ),
        ))
    }

    pub(crate) fn selected_pixel_rect(&self) -> Option<(u32, u32, u32, u32)> {
        let selection = self.selection_uv?;
        let width = self.image.width();
        let height = self.image.height();
        let min_x = (selection.min.x * width as f32).floor().clamp(0.0, width as f32) as u32;
        let min_y = (selection.min.y * height as f32).floor().clamp(0.0, height as f32) as u32;
        let max_x = (selection.max.x * width as f32).ceil().clamp(0.0, width as f32) as u32;
        let max_y = (selection.max.y * height as f32).ceil().clamp(0.0, height as f32) as u32;
        let crop_width = max_x.saturating_sub(min_x);
        let crop_height = max_y.saturating_sub(min_y);
        if crop_width == 0 || crop_height == 0 {
            return None;
        }
        Some((min_x, min_y, crop_width, crop_height))
    }
}

pub(crate) enum ScreenshotCropAction {
    Cancel,
    UseFullImage,
    UseSelection,
}

pub(crate) fn render_screenshot_crop_window(
    ctx: &egui::Context,
    crop: &mut ScreenshotCropState,
) -> Option<ScreenshotCropAction> {
    let escape_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    if escape_pressed {
        return Some(ScreenshotCropAction::Cancel);
    }

    crop.ensure_texture(ctx);
    let Some(texture) = crop.texture() else {
        return None;
    };

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let mut action = None;

    egui::Window::new("screenshot_crop")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .fixed_rect(screen_rect)
        .order(egui::Order::Foreground)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(12, 12, 12))
                .inner_margin(egui::Margin::same(8)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Crop Screenshot");
                ui.separator();
                ui.label("Drag to select the board area, or import the full capture.");
            });
            ui.add_space(8.0);

            let footer_height = 110.0;
            let available_size = ui.available_size();
            let image_area = egui::vec2(
                available_size.x,
                (available_size.y - footer_height).max(120.0),
            );

            let image_response = ui.add(
                egui::Image::new(texture)
                    .shrink_to_fit()
                    .sense(egui::Sense::click_and_drag())
                    .max_size(image_area),
            );

            if image_response.drag_started()
                && let Some(ptr) = image_response.interact_pointer_pos()
            {
                let uv = ScreenshotCropState::pointer_to_uv(image_response.rect, ptr);
                crop.drag_start_uv = Some(uv);
                crop.selection_uv = Some(egui::Rect::from_two_pos(uv, uv));
            }
            if image_response.dragged()
                && let Some(ptr) = image_response.interact_pointer_pos()
            {
                crop.update_selection_from_pointer(image_response.rect, ptr);
            }
            if image_response.drag_stopped() {
                crop.drag_start_uv = None;
            }

            let painter = ui.painter_at(image_response.rect);
            painter.rect_filled(
                image_response.rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 24),
            );
            if let Some(sel) = crop.selection_screen_rect(image_response.rect) {
                painter.rect_stroke(
                    sel,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN),
                    egui::StrokeKind::Outside,
                );
                painter.rect_filled(
                    sel,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(64, 255, 64, 24),
                );
            }

            ui.add_space(8.0);
            let crop_summary = if let Some((_, _, w, h)) = crop.selected_pixel_rect() {
                format!("Selected crop: {w} x {h}")
            } else {
                format!("Using full image: {} x {}", crop.image.width(), crop.image.height())
            };
            ui.label(crop_summary);
            ui.label("Press Esc to cancel.");

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if btn(ui, "Use Full Image").clicked() {
                    action = Some(ScreenshotCropAction::UseFullImage);
                }
                let can_crop = crop.selected_pixel_rect().is_some();
                let resp = ui.add_enabled_ui(can_crop, |ui| {
                    styled_button(ui, egui::Button::new("Crop && Import"))
                });
                if resp.inner.clicked() {
                    action = Some(ScreenshotCropAction::UseSelection);
                }
                if btn(ui, "Cancel").clicked() {
                    action = Some(ScreenshotCropAction::Cancel);
                }
            });
        });

    action
}
