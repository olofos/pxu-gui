#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod presentation_description;

#[cfg(not(target_arch = "wasm32"))]
mod build;

use clap::Parser;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    pub rebuild: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("eframe error: {0}")]
    EFrame(#[from] eframe::Error),

    #[error("io error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Presentation error: {0}")]
    Presentation(String),
    #[error("Toml deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("Toml serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    let arguments = Arguments::parse();

    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .without_time()
        .init();

    build::check_presentation("./presentation/images/", arguments.rebuild)?;

    let native_options = eframe::NativeOptions {
        fullscreen: true,
        vsync: true,
        ..Default::default()
    };

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
    )?;

    Ok(())
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
            Box::new(|cc| Box::new(app::PresentationApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
