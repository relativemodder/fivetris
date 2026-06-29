mod clipboard;
mod edit;
mod settings;

use super::*;

#[allow(dead_code)]
pub(crate) fn import_screenshot_image(app: &mut FourTrisApp, rgba: &image::RgbaImage) {
    clipboard::import_screenshot_image(app, rgba);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn finish_screenshot_crop(app: &mut FourTrisApp, action: ScreenshotCropAction) {
    clipboard::finish_screenshot_crop(app, action);
}

pub(crate) fn handle_app_action(app: &mut FourTrisApp, action: AppAction) {
    if clipboard::handle_app_action(app, action.clone())
        || settings::handle_app_action(app, action.clone())
        || edit::handle_app_action(app, action.clone())
    {
        return;
    }
    app.controller.dispatch(&mut app.state, action);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn poll_platform_events(app: &mut FourTrisApp) {
    clipboard::poll_platform_events(app);
}
