pub mod app;
pub mod config;
pub mod core;
pub mod media;
pub mod platform;
pub mod render;
pub mod resources;

#[cfg(target_arch = "wasm32")]
pub mod web;

use std::sync::Arc;

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::empty();

    let inter_data =
        resources::builtin_font_bytes("Inter.ttf").expect("Inter.ttf embedded");
    let inter_italic = resources::builtin_font_bytes("Inter-Italic.ttf")
        .expect("Inter-Italic.ttf embedded");
    let fa_solid = resources::builtin_font_bytes("FontAwesome-Solid.ttf")
        .expect("FontAwesome-Solid.ttf embedded");
    let fa_regular = resources::builtin_font_bytes("FontAwesome-Regular.ttf")
        .expect("FontAwesome-Regular.ttf embedded");

    fonts.font_data.insert(
        "Inter".to_owned(),
        Arc::new(egui::FontData::from_static(inter_data)),
    );
    fonts.font_data.insert(
        "Inter-Italic".to_owned(),
        Arc::new(egui::FontData::from_static(inter_italic)),
    );
    fonts.font_data.insert(
        "FontAwesome-Solid".to_owned(),
        Arc::new(egui::FontData::from_static(fa_solid)),
    );
    fonts.font_data.insert(
        "FontAwesome-Regular".to_owned(),
        Arc::new(egui::FontData::from_static(fa_regular)),
    );

    let proportional = fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default();
    proportional.push("Inter".to_owned());
    proportional.push("FontAwesome-Solid".to_owned());
    proportional.push("FontAwesome-Regular".to_owned());

    let monospace = fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default();
    monospace.push("Inter".to_owned());

    ctx.set_fonts(fonts);
}
