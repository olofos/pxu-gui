#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod arguments;
mod frame_history;
mod ui_state;

use crate::arguments::Arguments;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .without_time()
        .init();

    let arguments = Arguments::parse();

    let icon_bytes = include_bytes!("../assets/icon-256.png");
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(eframe::icon_data::from_png_bytes(icon_bytes).ok().unwrap()),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "pxu gui",
        native_options,
        Box::new(|cc| {
            let style = egui::Style {
                visuals: egui::Visuals::light(),
                ..egui::Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(app::PxuGuiApp::new(cc, arguments))
        }),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn get_url() -> Option<url::Url> {
    let location: String = web_sys::window()?
        .document()?
        .location()?
        .to_string()
        .into();

    url::Url::parse(&location).ok()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));

    let arguments = Arguments::from(get_url());

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(app::PxuGuiApp::new(cc, arguments))),
            )
            .await
            .expect("failed to start eframe");
    });
}
