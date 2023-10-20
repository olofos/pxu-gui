use std::collections::VecDeque;

use egui::{vec2, Pos2};
use pxu::kinematics::CouplingConstants;
use pxu::{CutType, Pxu};

use crate::arguments::Arguments;
use crate::ui_state::UiState;
use plot::Plot;

use std::sync::mpsc;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PxuGuiApp {
    pxu: pxu::Pxu,
    p_plot: Plot,
    xp_plot: Plot,
    xm_plot: Plot,
    u_plot: Plot,
    ui_state: UiState,
    #[serde(skip)]
    frame_history: crate::frame_history::FrameHistory,
    #[serde(skip)]
    path_dialog_text: Option<String>,
    #[serde(skip)]
    state_dialog_text: Option<String>,
    #[serde(skip)]
    show_about: bool,
    #[serde(skip)]
    show_help: bool,
    #[serde(skip)]
    show_figure_picker: bool,
    #[serde(skip)]
    fetch_queue: VecDeque<String>,
    #[serde(skip)]
    figures: Vec<interactive_figures::FigureDescription>,
    #[serde(skip)]
    figure_index: Option<usize>,
    #[serde(skip)]
    figure_response_channel: ResponseChannel,
}

type ResponseChannelTuple = (
    mpsc::Sender<(String, Result<ehttp::Response, ehttp::Error>)>,
    mpsc::Receiver<(String, Result<ehttp::Response, ehttp::Error>)>,
);
struct ResponseChannel {
    tx: mpsc::Sender<(String, Result<ehttp::Response, ehttp::Error>)>,
    rx: mpsc::Receiver<(String, Result<ehttp::Response, ehttp::Error>)>,
}

impl From<ResponseChannelTuple> for ResponseChannel {
    fn from(value: ResponseChannelTuple) -> Self {
        Self {
            tx: value.0,
            rx: value.1,
        }
    }
}

impl Default for PxuGuiApp {
    fn default() -> Self {
        let bound_state_number = 1;

        let consts = CouplingConstants::new(2.0, 5);

        let mut pxu = Pxu::new(consts);

        pxu.state = pxu::State::new(bound_state_number, consts);

        Self {
            pxu,
            p_plot: Plot {
                component: pxu::Component::P,
                height: 0.75,
                width_factor: 1.5,
                origin: Pos2::new(0.5, 0.0),
            },
            xp_plot: Plot {
                component: pxu::Component::Xp,
                height: (8.0 * consts.s()) as f32,
                width_factor: 1.0,
                origin: Pos2::ZERO,
            },
            xm_plot: Plot {
                component: pxu::Component::Xm,
                height: (8.0 * consts.s()) as f32,
                width_factor: 1.0,
                origin: Pos2::ZERO,
            },
            u_plot: Plot {
                component: pxu::Component::U,
                height: ((4 * consts.k() + 1) as f64 / consts.h) as f32,
                width_factor: 1.0,
                origin: Pos2::ZERO,
            },
            frame_history: Default::default(),
            ui_state: Default::default(),
            path_dialog_text: None,
            state_dialog_text: None,
            show_about: false,
            show_help: false,
            show_figure_picker: false,
            fetch_queue: VecDeque::from(vec!["figures".to_owned()]),
            figures: vec![],
            figure_index: None,
            figure_response_channel: mpsc::channel().into(),
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.

    fonts.font_data.insert(
        "DejaVuSansMono".to_owned(),
        egui::FontData::from_static(include_bytes!("../../fonts/DejaVuSansMono.ttf")),
    );

    // Put my font first (highest priority):
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "DejaVuSansMono".to_owned());

    // // Put my font as last fallback for monospace:
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "DejaVuSansMono".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

impl PxuGuiApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Arguments) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        setup_custom_fonts(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.ui_state.set(settings);
            return app;
        }

        let mut app: PxuGuiApp = Default::default();
        app.ui_state.set(settings);
        app
    }

    fn load_figure_descriptions(&mut self, body: &str) -> Result<(), String> {
        let figures = ron::from_str::<Vec<interactive_figures::FigureDescription>>(body)
            .map_err(|err| format!("Could not parse figure description: {err}"))?;

        self.figure_index = None;
        self.figures = figures;
        Ok(())
    }

    fn load_figure(&mut self, name: &String, body: &str) -> Result<(), String> {
        let figure = ron::from_str::<interactive_figures::Figure>(body)
            .map_err(|err| format!("Could not parse figure {name}: {err}"))?;

        log::info!("Loaded figure {name}");

        self.ui_state.plot_state.path_indices = (0..figure.paths.len()).collect();
        self.pxu.state = figure.state;
        self.pxu.paths = figure.paths;
        self.ui_state.plot_state.active_point = 0;
        Ok(())
    }

    fn load_file(&mut self, name: &String, bytes: Vec<u8>) -> Result<(), String> {
        let body = std::str::from_utf8(&bytes)
            .map_err(|err| format!("Could not parse response body: {err}"))?;

        if name == "figures" {
            self.load_figure_descriptions(body)
        } else {
            self.load_figure(name, body)
        }
    }

    fn parse_figure_download_response(
        &mut self,
        name: &String,
        response: ehttp::Response,
    ) -> Result<(), String> {
        if !response.ok {
            return Err(format!(
                "Fetch failed with {} {}",
                response.status, response.status_text
            ));
        }

        if let Some(typ) = response.headers.get("content-type") {
            if typ == "text/html" {
                return Err("Unexpected html file".into());
            }
        }

        self.load_file(name, response.bytes)
    }

    #[cfg(target_arch = "wasm32")]
    fn get_base_url(&self) -> Option<String> {
        let location: String = web_sys::window()?
            .document()?
            .location()?
            .to_string()
            .into();

        let mut url = url::Url::parse(&location).ok()?;

        url.set_fragment(None);
        url.set_query(None);
        let url = url.join("./").ok()?;

        Some(url.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_base_url(&self) -> Option<String> {
        Some("http://olofos.github.io/pxu-gui/".to_owned())
    }

    fn download_file(&mut self, ctx: &egui::Context, name: &String) -> Result<(), String> {
        let base_url = self.get_base_url().ok_or("No base URL set".to_owned())?;

        let url = format!("{base_url}data/{name}.ron");
        let request = ehttp::Request::get(url);

        let ctx = ctx.clone();
        let tx = self.figure_response_channel.tx.clone();
        let name = name.clone();
        ehttp::fetch(request, move |response| {
            if tx.send((name, response)).is_err() {
                log::info!("Could not send response!");
            }
            ctx.request_repaint(); // Wake up UI thread
        });

        Ok(())
    }

    fn receive_download(&mut self) {
        if let Ok((name, response)) = self.figure_response_channel.rx.try_recv() {
            match response {
                Err(err) => {
                    log::info!("Error: {err}")
                }
                Ok(response) => {
                    if let Err(err) = self.parse_figure_download_response(&name, response) {
                        log::error!("Error: {err}")
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_local_file(&mut self, name: &String) -> Result<(), String> {
        let mut path = std::path::Path::new("./pxu-gui/dist/data/").join(name);
        path.set_extension("ron");

        let bytes =
            std::fs::read(&path).map_err(|err| format!("Could not read {path:?}: {err}"))?;

        self.load_file(name, bytes)
    }

    fn load_files(&mut self, ctx: &egui::Context) {
        let Some(name) = self.fetch_queue.pop_front() else {
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            match self.load_local_file(&name) {
                Ok(_) => return,
                Err(err) => {
                    log::warn!("Error: {err}");
                    log::info!("Trying to download file instead");
                }
            }
        }
        match self.download_file(ctx, &name) {
            Ok(_) => {}
            Err(err) => {
                log::warn!("Error: {err}");
            }
        }
    }

    fn is_ux_mode(&self) -> bool {
        self.ui_state.plot_state.theme == plot::Theme::Black
    }

    fn set_ux_mode(&mut self) {
        self.ui_state.plot_state.theme = plot::Theme::Black;
        self.ui_state.plot_state.cut_filter = plot::CutFilter::Only(vec![
            CutType::UShortScallion(pxu::Component::Xp),
            CutType::UShortKidney(pxu::Component::Xp),
        ]);
        self.ui_state.plot_state.fullscreen_component = None;
    }

    fn set_normal_mode(&mut self) {
        self.ui_state.plot_state.theme = plot::Theme::Normal;
        self.ui_state.plot_state.cut_filter = plot::CutFilter::All;
    }
}

impl eframe::App for PxuGuiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.ui_state.show_fps {
            self.frame_history
                .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
        }

        self.load_files(ctx);
        self.receive_download();

        if self.ui_state.continuous_mode {
            ctx.request_repaint();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.ui_state.hide_side_panel = !self.ui_state.hide_side_panel;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.ui_state.plot_state.close_fullscreen();
            self.ui_state.hide_side_panel = false;
        }

        ctx.input(|i| {
            for (key, num) in [
                (egui::Key::Backspace, self.pxu.state.points.len()),
                (egui::Key::Num1, 1),
                (egui::Key::Num2, 2),
                (egui::Key::Num3, 3),
                (egui::Key::Num4, 4),
                (egui::Key::Num5, 5),
                (egui::Key::Num6, 6),
                (egui::Key::Num7, 7),
                (egui::Key::Num8, 8),
                (egui::Key::Num9, 9),
            ] {
                if i.key_pressed(key) {
                    self.pxu.state = pxu::State::new(num, self.pxu.consts);
                    self.ui_state.plot_state.active_point =
                        self.ui_state.plot_state.active_point.min(num - 1);
                }
            }

            if i.key_pressed(egui::Key::Space) {
                self.pxu.state.unlocked = !self.pxu.state.unlocked;
            }
        });

        if self.pxu.state.unlocked && ctx.input(|i| i.key_pressed(egui::Key::PlusEquals)) {
            self.pxu
                .state
                .points
                .push(pxu::Point::new(0.1, self.pxu.consts));
        }

        if self.pxu.state.unlocked
            && self.pxu.state.points.len() > 1
            && ctx.input(|i| i.key_pressed(egui::Key::Minus))
        {
            self.pxu
                .state
                .points
                .remove(self.ui_state.plot_state.active_point);
            self.ui_state.plot_state.active_point = self
                .ui_state
                .plot_state
                .active_point
                .min(self.pxu.state.points.len() - 1);
        }

        if self.pxu.state.unlocked
            && self.pxu.state.points.len() > 1
            && self.ui_state.plot_state.active_point < self.pxu.state.points.len() - 1
            && ctx.input(|i| i.key_pressed(egui::Key::ArrowUp))
        {
            let i = self.ui_state.plot_state.active_point;
            self.pxu.state.points.swap(i, i + 1);
            self.ui_state.plot_state.active_point += 1;
        }

        if self.pxu.state.unlocked
            && self.pxu.state.points.len() > 1
            && self.ui_state.plot_state.active_point > 0
            && ctx.input(|i| i.key_pressed(egui::Key::ArrowDown))
        {
            let i = self.ui_state.plot_state.active_point;
            self.pxu.state.points.swap(i, i - 1);
            self.ui_state.plot_state.active_point -= 1;
        }

        if self.pxu.state.points.len() > 1
            && self.ui_state.plot_state.active_point < self.pxu.state.points.len() - 1
            && ctx.input(|i| i.key_pressed(egui::Key::ArrowRight))
        {
            self.ui_state.plot_state.active_point += 1;
        }

        if self.pxu.state.points.len() > 1
            && self.ui_state.plot_state.active_point > 0
            && ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft))
        {
            self.ui_state.plot_state.active_point -= 1;
        }

        if !self.ui_state.hide_side_panel {
            self.draw_side_panel(ctx);
        }

        if let Some(saved_state) = self.ui_state.inital_saved_state.take() {
            self.pxu.consts = saved_state.consts;
            self.pxu.state = saved_state.state;
        }

        {
            let start = chrono::Utc::now();
            while (chrono::Utc::now() - start).num_milliseconds()
                < (1000.0 / 20.0f64).floor() as i64
            {
                if self.pxu.contours.update(
                    self.pxu.state.points[self.ui_state.plot_state.active_point]
                        .p
                        .re
                        .floor() as i32,
                    self.pxu.consts,
                ) {
                    if let Some(ref mut saved_paths) = self.ui_state.saved_paths_to_load {
                        if let Some(saved_path) = saved_paths.pop() {
                            let path = pxu::Path::from_base_path(
                                saved_path.into(),
                                &self.pxu.contours,
                                self.pxu.consts,
                            );

                            self.pxu.paths.push(path);

                            let progress = self.ui_state.path_load_progress.unwrap();
                            self.ui_state.path_load_progress = Some((progress.0 + 1, progress.1));
                        } else {
                            self.ui_state.saved_paths_to_load = None;
                            self.ui_state.path_load_progress = None;
                            if !self.pxu.paths.is_empty() {
                                self.ui_state.plot_state.path_indices = vec![];
                            }
                        }
                    }
                    break;
                }
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();

            let mut plots = if let Some(component) = self.ui_state.plot_state.fullscreen_component {
                let plot = match component {
                    pxu::Component::P => &mut self.p_plot,
                    pxu::Component::Xp => &mut self.xp_plot,
                    pxu::Component::Xm => &mut self.xm_plot,
                    pxu::Component::U => &mut self.u_plot,
                };

                vec![(plot, rect)]
            } else if self.is_ux_mode() {
                use egui::Rect;
                const GAP: f32 = 8.0;
                let w = (rect.width() - GAP) / 2.0;
                let size = vec2(w, rect.height());

                let top_left = rect.left_top();

                vec![
                    (
                        &mut self.u_plot,
                        Rect::from_min_size(top_left + vec2(w + GAP, 0.0), size),
                    ),
                    (
                        &mut self.xp_plot,
                        Rect::from_min_size(top_left + vec2(0.0, 0.0), size),
                    ),
                ]
            } else {
                use egui::Rect;
                const GAP: f32 = 8.0;
                let w = (rect.width() - GAP) / 2.0;
                let h = (rect.height() - GAP) / 2.0;
                let size = vec2(w, h);

                let top_left = rect.left_top();

                vec![
                    (&mut self.p_plot, Rect::from_min_size(top_left, size)),
                    (
                        &mut self.u_plot,
                        Rect::from_min_size(top_left + vec2(w + GAP, 0.0), size),
                    ),
                    (
                        &mut self.xp_plot,
                        Rect::from_min_size(top_left + vec2(0.0, h + GAP), size),
                    ),
                    (
                        &mut self.xm_plot,
                        Rect::from_min_size(top_left + vec2(w + GAP, h + GAP), size),
                    ),
                ]
            };

            self.ui_state.plot_state.reset();

            for (plot, rect) in plots.iter_mut() {
                plot.interact(ui, *rect, &mut self.pxu, &mut self.ui_state.plot_state);
            }

            for (plot, rect) in plots {
                plot.show(ui, rect, &mut self.pxu, &mut self.ui_state.plot_state);
            }
        });

        self.show_load_path_window(ctx);
        self.show_load_save_state_window(ctx);
        self.show_about_window(ctx);
        self.show_help_window(ctx);
        self.show_figure_window(ctx);
    }
}

impl PxuGuiApp {
    fn show_load_path_window(&mut self, ctx: &egui::Context) {
        if let Some(ref mut s) = self.path_dialog_text {
            let mut close_dialog = false;
            egui::Window::new("Load path")
                .default_height(500.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(600.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(s)
                                    .font(egui::TextStyle::Monospace) // for cursor height
                                    .code_editor()
                                    .desired_rows(10)
                                    .lock_focus(true)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::LEFT), |ui| {
                        ui.add_space(10.0);
                        if ui.button("Cancel").clicked() {
                            close_dialog = true;
                        }
                        if ui.button("OK").clicked() {
                            if let Some(saved_paths) = pxu::path::SavedPath::load(s) {
                                close_dialog = true;
                                self.pxu.consts = saved_paths[0].consts;
                                self.pxu.state = saved_paths[0].start.clone();
                                self.ui_state.plot_state.active_point = saved_paths[0].excitation;
                                self.pxu.paths = saved_paths
                                    .into_iter()
                                    .map(|saved_path| {
                                        pxu::Path::from_base_path(
                                            saved_path.into(),
                                            &self.pxu.contours,
                                            self.pxu.consts,
                                        )
                                    })
                                    .collect();
                            }
                        }
                    });
                });
            if close_dialog {
                self.path_dialog_text = None;
            }
        }
    }

    fn show_load_save_state_window(&mut self, ctx: &egui::Context) {
        if let Some(ref mut s) = self.state_dialog_text {
            let mut close_dialog = false;
            egui::Window::new("Save state")
                .default_height(500.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(600.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(s)
                                    .font(egui::TextStyle::Monospace) // for cursor height
                                    .code_editor()
                                    .desired_rows(10)
                                    .lock_focus(true)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::LEFT), |ui| {
                        ui.add_space(10.0);

                        if ui.button("Close").clicked() {
                            close_dialog = true;
                        }

                        if ui.button("Load").clicked() {
                            close_dialog = true;

                            if let Some(saved_state) = pxu::SavedState::decode(s) {
                                self.pxu.consts = saved_state.consts;
                                self.pxu.state = saved_state.state;
                            }
                        }

                        if ui.button("Compress").clicked() {
                            use base64::Engine;
                            use std::io::Write;

                            let mut enc = flate2::write::DeflateEncoder::new(
                                Vec::new(),
                                flate2::Compression::best(),
                            );
                            if enc.write_all(s.as_bytes()).is_ok() {
                                if let Ok(data) = enc.finish() {
                                    let compressed =
                                        base64::engine::general_purpose::URL_SAFE.encode(data);
                                    *s = compressed;
                                }
                            }
                        }
                    });
                });
            if close_dialog {
                self.state_dialog_text = None;
            }
        }
    }

    fn show_about_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .open(&mut self.show_about)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("PXU gui");

                const VERSION: &str = env!("CARGO_PKG_VERSION");
                ui.label(format!("Version {VERSION}"));

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    const ARXIV_ID: &str = "XXXX.XXXXX";

                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("This application is a supplement to the paper ");
                    ui.hyperlink_to(
                        format!("arXiv:{ARXIV_ID}"),
                        format!("https://arxiv.org/abs/{ARXIV_ID}"),
                    );
                    ui.label(".");
                });

                ui.add_space(8.0);

                ui.label("Copyright © 2023 Olof Ohlsson Sax");
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Licensed under the ");
                    ui.hyperlink_to(
                        "MIT license",
                        "https://github.com/olofos/pxu-gui/blob/master/LICENSE",
                    );
                    ui.label(".");
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Source code available on ");
                    ui.hyperlink_to("github", "https://github.com/olofos/pxu-gui/");
                    ui.label(".");
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
    }

    fn show_help_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Help")
            .open(&mut self.show_help)
            .resizable(true)
            .collapsible(true)
            .scroll2([false, true])
            .show(ctx, |ui| {
                use egui_commonmark::*;
                let markdown = include_str!("help.md");

                let mut cache = CommonMarkCache::default();
                CommonMarkViewer::new("viewer").show(ui, &mut cache, markdown);
            });
    }

    fn show_figure_window(&mut self, ctx: &egui::Context) {
        let mut close = false;
        egui::Window::new("Figures")
            .open(&mut self.show_figure_picker)
            .resizable(false)
            .collapsible(true)
            .show(ctx, |ui| {
                for (index, fig) in self.figures.iter().enumerate() {
                    let response = ui.selectable_label(Some(index) == self.figure_index, &fig.name);
                    if (response.clicked() || response.double_clicked())
                        && Some(index) != self.figure_index
                    {
                        self.fetch_queue.push_back(fig.filename.clone());
                        self.figure_index = Some(index);
                    };

                    if response.double_clicked() {
                        close = true;
                    }
                }
            });
        self.show_figure_picker ^= close;
    }

    fn draw_coupling_controls(&mut self, ui: &mut egui::Ui) {
        let old_consts = self.pxu.consts;
        let mut new_consts = self.pxu.consts;

        ui.add_space(6.0);
        ui.label(egui::RichText::new("Mode").strong());

        ui.horizontal(|ui| {
            if ui
                .add(egui::RadioButton::new(
                    self.ui_state.plot_state.theme != plot::Theme::Black,
                    "Full",
                ))
                .clicked()
            {
                self.set_normal_mode()
            }

            if ui
                .add(egui::RadioButton::new(self.is_ux_mode(), "u(x)"))
                .clicked()
            {
                self.set_ux_mode();
            }
        });

        ui.label(egui::RichText::new("Parameters").strong());
        ui.add_space(6.0);

        ui.add(
            egui::Slider::new(&mut new_consts.h, 0.1..=10.0)
                .text("h")
                .logarithmic(true),
        );

        ui.add(
            egui::Slider::from_get_set(0.0..=10.0, |v| new_consts.get_set_k(v))
                .integer()
                .text("k"),
        );
        ui.add(
            egui::Slider::from_get_set(1.0..=20.0, |n| {
                if let Some(n) = n {
                    let n = n as usize;
                    self.pxu.state = pxu::State::new(n, self.pxu.consts);
                    self.ui_state.plot_state.active_point = n / 2;
                }
                self.pxu.state.points.len() as f64
            })
            .integer()
            .text("M"),
        );

        if old_consts != new_consts {
            self.pxu.consts = new_consts;
            self.pxu.state = pxu::State::new(self.pxu.state.points.len(), new_consts);
            self.pxu.contours.clear();
        }
    }

    fn draw_dev_controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("Dev controls");
        ui.add_space(5.0);
        if ui.add(egui::Button::new("Load path")).clicked() {
            self.path_dialog_text = Some(String::new());
        }

        if ui.button("Load/save state").clicked() {
            let saved_state = pxu::SavedState {
                state: self.pxu.state.clone(),
                consts: self.pxu.consts,
            };
            if let Ok(s) = ron::to_string(&saved_state) {
                self.state_dialog_text = Some(s);
            } else {
                log::info!("Could not print state");
            }
        }

        if !self.pxu.paths.is_empty() {
            ui.add_space(5.0);
            ui.label("Paths");
            egui::Frame::none()
                .inner_margin(4.0)
                .stroke(egui::Stroke::new(2.0, egui::Color32::GRAY))
                .fill(egui::Color32::WHITE)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for i in 0..self.pxu.paths.len() {
                                let path = &self.pxu.paths[i];
                                let index_index = self
                                    .ui_state
                                    .plot_state
                                    .path_indices
                                    .iter()
                                    .position(|&j| j == i);
                                let selected = index_index.is_some();

                                if ui.selectable_label(selected, &path.name).clicked() {
                                    if selected {
                                        self.ui_state
                                            .plot_state
                                            .path_indices
                                            .remove(index_index.unwrap());
                                    } else {
                                        self.ui_state.plot_state.path_indices.push(i);
                                    }
                                }
                            }
                        });
                });
        }
    }

    fn draw_state_information(&mut self, ui: &mut egui::Ui) {
        let active_point = &self.pxu.state.points[self.ui_state.plot_state.active_point];
        ui.separator();
        {
            ui.label(egui::RichText::new("State").strong());

            ui.label(format!("Momentum: {:.3}", self.pxu.state.p()));
            ui.label(format!(
                "Energy:   {:.3}",
                self.pxu.state.en(self.pxu.consts)
            ));
            ui.label(format!(
                "Charge:   {:.3}",
                self.pxu.state.points.len() as f64
                    + self.pxu.consts.k() as f64 * self.pxu.state.p()
            ));
        }

        ui.separator();

        {
            ui.label(
                egui::RichText::new(format!(
                    "Active excitation (#{})",
                    self.ui_state.plot_state.active_point
                ))
                .strong(),
            );

            ui.label(format!("Momentum: {:.3}", active_point.p));

            ui.label(format!("Energy:   {:.3}", active_point.en(self.pxu.consts)));

            ui.add_space(10.0);
            ui.label(format!("x⁺: {:.3}", active_point.xp));
            ui.label(format!("x⁻: {:.3}", active_point.xm));
            ui.label(format!("u:  {:.3}", active_point.u));

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Branch info").strong());

            ui.label(format!(
                "Log branch: {:+} {:+}",
                active_point.sheet_data.log_branch_p, active_point.sheet_data.log_branch_m
            ));

            ui.label(format!(
                "E branch:   {:+}",
                active_point.sheet_data.e_branch
            ));
            ui.label(format!(
                "U branch:   ({:+},{:+})",
                active_point.sheet_data.u_branch.0, active_point.sheet_data.u_branch.1
            ));

            ui.add_space(10.0);

            {
                let xp = active_point.xp;
                let xm = xp.conj();
                let h = self.pxu.consts.h;
                let k = self.pxu.consts.k() as f64;
                let p = xp.arg() / std::f64::consts::PI;
                let m = h / 2.0
                    * (xp + 1.0 / xp
                        - xm
                        - 1.0 / xm
                        - 2.0 * num::complex::Complex64::i() * (k * p) / h)
                        .im;
                if xp.im >= 0.0 {
                    ui.label(format!("x⁺ = X⁺({:.3},{:.3})", p, m));
                } else {
                    ui.label(format!("x⁺ = X⁻({:.3},{:.3})", -p, -m));
                };
            }
            {
                let xm = active_point.xm;
                let xp = xm.conj();
                let h = self.pxu.consts.h;
                let k = self.pxu.consts.k() as f64;
                let p = xp.arg() / std::f64::consts::PI;
                let m = h / 2.0
                    * (xp + 1.0 / xp
                        - xm
                        - 1.0 / xm
                        - 2.0 * num::complex::Complex64::i() * (k * p) / h)
                        .im;
                if xm.im >= 0.0 {
                    ui.label(format!("x⁻ = X⁺({:.3},{:.3})", -p, -m));
                } else {
                    ui.label(format!("x⁻ = X⁻({:.3},{:.3})", p, m));
                };
            }
        }
    }

    fn draw_state_information_ux(&mut self, ui: &mut egui::Ui) {
        let active_point = &self.pxu.state.points[self.ui_state.plot_state.active_point];
        ui.separator();

        {
            ui.label(
                egui::RichText::new(format!(
                    "Active excitation (#{})",
                    self.ui_state.plot_state.active_point
                ))
                .strong(),
            );

            ui.label(format!("x: {:.3}", active_point.xp));
            ui.label(format!("u: {:.3}", active_point.u));

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Branch info").strong());

            ui.label(format!(
                "Log branch: {:+}",
                active_point.sheet_data.log_branch_p
            ));
            ui.label(format!(
                "U branch:   {:+}",
                active_point.sheet_data.u_branch.0
            ));
        }
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            self.draw_coupling_controls(ui);

            if ui.add(egui::Button::new("Reset State")).clicked() {
                self.pxu.state = pxu::State::new(self.pxu.state.points.len(), self.pxu.consts);
            }

            ui.checkbox(&mut self.pxu.state.unlocked, "Unlock bound state");

            if self.is_ux_mode() {
                self.draw_state_information_ux(ui);
            } else {
                self.draw_state_information(ui);
            }

            ui.separator();
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(!self.figures.is_empty(), egui::Button::new("Figures"))
                    .on_disabled_hover_text("No figures loaded")
                    .on_hover_text("Pick a figure")
                    .clicked()
                {
                    self.show_figure_picker = true;
                }

                if ui.button("Help").clicked() {
                    self.show_help = true;
                }

                if ui.button("About").clicked() {
                    self.show_about = true;
                }
            });

            if let Some(index) = self.figure_index {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Figure: {}", self.figures[index].name)).strong(),
                );
                ui.add_space(5.0);
                ui.label(&self.figures[index].description);
                ui.add_space(5.0);

                if ui.button("Close").clicked() {
                    self.figure_index = None;
                    self.pxu.paths.clear();
                }
            }

            if self.ui_state.show_dev {
                self.draw_dev_controls(ui);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                egui::warn_if_debug_build(ui);

                if self.ui_state.show_fps {
                    ui.separator();

                    ui.label(format!("Framerate: {:.0} fps", self.frame_history.fps()));
                    ui.label(format!(
                        "CPU usage: {:.1} ms/frame",
                        1000.0 * self.frame_history.mean_frame_time()
                    ));

                    if !self.ui_state.continuous_mode {
                        ui.label(
                            egui::RichText::new("⚠ Not running in continuous mode ⚠")
                                .small()
                                .color(ui.visuals().warn_fg_color),
                        )
                        .on_hover_text("The screen will only be redrawn when it receives input");
                    }
                    ui.separator();
                }

                ui.add_space(10.0);
                let (current, total) = self.pxu.contours.progress();
                if total > 1 && current != total {
                    let progress = current as f32 / total as f32;
                    ui.add(
                        egui::ProgressBar::new(progress)
                            .text(format!("Generating contours   {:.0}%", 100.0 * progress)),
                    );
                } else if let Some((curret, total)) = self.ui_state.path_load_progress {
                    let progress = current as f32 / total as f32;
                    ui.add(
                        egui::ProgressBar::new(progress)
                            .text(format!("Loading paths: {}/{}", curret, total)),
                    );
                }
            });
        });
    }
}
