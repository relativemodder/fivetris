#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![cfg(not(target_arch = "wasm32"))]

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
            fivetris::setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(fivetris::app::FourTrisApp::default()))
        }),
    )
}
