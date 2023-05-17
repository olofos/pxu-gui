#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .without_time()
        .init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.fullscreen = true;
    native_options.vsync = true;

    eframe::run_native(
        "presentation",
        native_options,
        Box::new(|cc| {
            let style = egui::Style {
                visuals: egui::Visuals::light(),
                ..egui::Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(app::PresentationApp::new(cc))
        }),
    );
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

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(app::PresentationApp::new(cc, arguments))),
        )
        .await
        .expect("failed to start eframe");
    });
}
