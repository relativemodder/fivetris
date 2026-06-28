#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn setup_custom_fonts(ctx: &egui::Context) {
    use std::sync::Arc;

    let mut fonts = egui::FontDefinitions::empty();

    let inter_data =
        fivetris::resources::builtin_font_bytes("Inter.ttf").expect("Inter.ttf embedded");
    let inter_italic = fivetris::resources::builtin_font_bytes("Inter-Italic.ttf")
        .expect("Inter-Italic.ttf embedded");
    let fa_solid = fivetris::resources::builtin_font_bytes("FontAwesome-Solid.ttf")
        .expect("FontAwesome-Solid.ttf embedded");
    let fa_regular = fivetris::resources::builtin_font_bytes("FontAwesome-Regular.ttf")
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
    monospace.push("FontAwesome-Solid".to_owned());

    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let icon = {
        let image = image::load_from_memory(fivetris::resources::BUILTIN_ICON_PNG)
            .expect("icon.png embedded")
            .into_rgba8();
        let (w, h) = image.dimensions();
        egui::IconData {
            rgba: image.into_raw(),
            width: w,
            height: h,
        }
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("io.github.relativemodder.fivetris")
            .with_icon(icon)
            .with_inner_size(egui::vec2(842.0, 650.0))
            .with_min_inner_size(egui::vec2(842.0, 650.0))
            .with_title("Fivetris"),
        ..Default::default()
    };
    eframe::run_native(
        "fivetris",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(fivetris::app::FourTrisApp::default()))
        }),
    )
}
