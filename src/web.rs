use std::sync::Mutex;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

static PENDING_IMPORT: Mutex<Option<Vec<u8>>> = Mutex::new(None);
static PENDING_SETTINGS_IMPORT: Mutex<Option<String>> = Mutex::new(None);

pub fn take_pending_import() -> Option<Vec<u8>> {
    PENDING_IMPORT.lock().unwrap().take()
}

pub fn setup_clipboard_paste() {
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let on_paste =
        Closure::<dyn Fn(web_sys::ClipboardEvent)>::new(move |event: web_sys::ClipboardEvent| {
            let Some(data) = event.clipboard_data() else { return };
            let items = data.items();
            let len = items.length();
            for i in 0..len {
                let Some(item) = items.get(i) else { continue };
                if item.kind() == "file" && item.type_().starts_with("image/") {
                    if let Ok(Some(file)) = item.get_as_file() {
                        let reader = web_sys::FileReader::new().expect("create FileReader");
                        let reader_c = reader.clone();
                        let on_load = Closure::<dyn Fn()>::new(move || {
                            let result = reader_c
                                .result()
                                .expect("file read result")
                                .dyn_into::<js_sys::ArrayBuffer>()
                                .ok()
                                .map(|ab| {
                                    let array = js_sys::Uint8Array::new(&ab);
                                    array.to_vec()
                                });
                            if let Some(bytes) = result {
                                *PENDING_IMPORT.lock().unwrap() = Some(bytes);
                            }
                        });
                        reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
                        on_load.forget();
                        reader.read_as_array_buffer(&file).expect("read file");
                    }
                    break;
                }
            }
        });

    document
        .add_event_listener_with_callback("paste", on_paste.as_ref().unchecked_ref())
        .expect("add paste listener");
    on_paste.forget();
}

pub fn take_pending_settings_import() -> Option<String> {
    PENDING_SETTINGS_IMPORT.lock().unwrap().take()
}

pub fn prompt_import_settings() {
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let input = document
        .create_element("input")
        .expect("create input")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("cast input");
    input.set_type("file");
    input.set_accept(".json,application/json");

    let input_clone = input.clone();
    let on_change = Closure::<dyn Fn()>::new(move || {
        if let Some(file) = input_clone.files().and_then(|f| f.item(0)) {
            let reader = web_sys::FileReader::new().expect("create FileReader");
            let reader_c = reader.clone();
            let on_load = Closure::<dyn Fn()>::new(move || {
                let result = reader_c.result().expect("file read result").as_string();
                if let Some(json) = result {
                    *PENDING_SETTINGS_IMPORT.lock().unwrap() = Some(json);
                }
            });
            reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
            on_load.forget();
            reader.read_as_text(&file).expect("read file");
        }
    });
    input
        .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
        .expect("add change listener");
    on_change.forget();

    let body = document.body().expect("no body");
    body.append_child(&input).expect("append input");
    input.click();
    body.remove_child(&input).expect("remove input");
}

pub fn export_settings(json: &str) {
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let encoded = js_sys::encode_uri_component(json);
    let data_url = format!("data:application/json,{encoded}");

    let link = document
        .create_element("a")
        .expect("create link")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("cast HtmlElement");
    link.set_attribute("href", &data_url).expect("set href");
    link.set_attribute("download", "fivetris-settings.json")
        .expect("set download");
    link.set_attribute("style", "display: none")
        .expect("set style");

    let body = document.body().expect("no body");
    body.append_child(&link).expect("append link");
    link.click();
    body.remove_child(&link).expect("remove link");
}

pub fn prompt_import_screenshot() {
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let input = document
        .create_element("input")
        .expect("create input")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("cast input");
    input.set_type("file");
    input.set_accept("image/*");

    let input_clone = input.clone();
    let on_change = Closure::<dyn Fn()>::new(move || {
        if let Some(file) = input_clone.files().and_then(|f| f.item(0)) {
            let reader = web_sys::FileReader::new().expect("create FileReader");
            let reader_c = reader.clone();
            let on_load = Closure::<dyn Fn()>::new(move || {
                let result = reader_c
                    .result()
                    .expect("file read result")
                    .dyn_into::<js_sys::ArrayBuffer>()
                    .ok()
                    .map(|ab| {
                        let array = js_sys::Uint8Array::new(&ab);
                        array.to_vec()
                    });
                if let Some(bytes) = result {
                    *PENDING_IMPORT.lock().unwrap() = Some(bytes);
                }
            });
            reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
            on_load.forget();
            reader.read_as_array_buffer(&file).expect("read file");
        }
    });
    input
        .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
        .expect("add change listener");
    on_change.forget();

    let body = document.body().expect("no body");
    body.append_child(&input).expect("append input");
    input.click();
    body.remove_child(&input).expect("remove input");
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();

    let canvas = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("fivetris_canvas"))
        .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .expect("canvas #fivetris_canvas not found");

    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|cc| {
                crate::setup_custom_fonts(&cc.egui_ctx);
                Ok(Box::new(crate::app::FourTrisApp::default()))
            }),
        )
        .await
        .expect("failed to start eframe web app");

    setup_clipboard_paste();

    Ok(())
}
