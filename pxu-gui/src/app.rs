use egui::{vec2, Pos2};
use pxu::kinematics::CouplingConstants;
use pxu::path::EditablePath;
use pxu::Pxu;
use pxu::UCutType;

use crate::anim::Anim;
use crate::arguments::Arguments;
use crate::plot::Plot;
use crate::ui_state::UiState;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PxuGuiApp {
    pxu: pxu::Pxu,
    #[serde(skip)]
    p_plot: Plot,
    xp_plot: Plot,
    xm_plot: Plot,
    u_plot: Plot,
    #[serde(skip)]
    frame_history: crate::frame_history::FrameHistory,
    #[serde(skip)]
    anim_data: Anim,
    ui_state: UiState,
    editable_path: EditablePath,
    #[serde(skip)]
    path_dialog_text: Option<String>,
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
            anim_data: Default::default(),
            ui_state: Default::default(),
            editable_path: Default::default(),
            path_dialog_text: None,
        }
    }
}

impl PxuGuiApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Arguments) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.ui_state.set(settings);
            return app;
        }

        Default::default()
    }
}

impl eframe::App for PxuGuiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.ui_state.show_fps {
            self.frame_history
                .on_new_frame(ctx.input().time, frame.info().cpu_usage);
        }

        if self.ui_state.continuous_mode {
            ctx.request_repaint();
        }

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                ui.menu_button("View", |ui| self.ui_state.menu(ui, None));
            });
        });

        if ctx.input().key_pressed(egui::Key::Enter) {
            self.ui_state.show_side_panel = !self.ui_state.show_side_panel;
        }

        if ctx.input().key_pressed(egui::Key::Escape) {
            self.ui_state.close_fullscreen();
            self.ui_state.show_side_panel = true;
        }

        if self.ui_state.show_side_panel {
            self.draw_side_panel(ctx);
        }

        {
            let start = chrono::Utc::now();
            while (chrono::Utc::now() - start).num_milliseconds()
                < (1000.0 / 20.0f64).floor() as i64
            {
                if self.pxu.contours.update(
                    self.pxu.state.points[self.ui_state.active_point]
                        .p
                        .re
                        .floor() as i32,
                    self.pxu.consts,
                ) {
                    break;
                }
                ctx.request_repaint();
            }
        }

        if !self.anim_data.is_stopped() {
            if let Some(z) = self.anim_data.update() {
                self.pxu.state.update(
                    self.anim_data.active_point,
                    self.anim_data.component,
                    z,
                    &self.pxu.contours,
                    self.pxu.consts,
                );
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();

            if let Some(component) = self.ui_state.fullscreen_component {
                let plot = match component {
                    pxu::Component::P => &mut self.p_plot,
                    pxu::Component::Xp => &mut self.xp_plot,
                    pxu::Component::Xm => &mut self.xm_plot,
                    pxu::Component::U => &mut self.u_plot,
                };

                plot.show(
                    ui,
                    rect,
                    &mut self.pxu,
                    &mut self.editable_path,
                    &mut self.ui_state,
                );
            } else {
                use egui::Rect;
                const GAP: f32 = 8.0;
                let w = (rect.width() - GAP) / 2.0;
                let h = (rect.height() - GAP) / 2.0;
                let size = vec2(w, h);

                let top_left = rect.left_top();

                self.p_plot.show(
                    ui,
                    Rect::from_min_size(top_left, size),
                    &mut self.pxu,
                    &mut self.editable_path,
                    &mut self.ui_state,
                );

                self.u_plot.show(
                    ui,
                    Rect::from_min_size(top_left + vec2(w + GAP, 0.0), size),
                    &mut self.pxu,
                    &mut self.editable_path,
                    &mut self.ui_state,
                );

                self.xp_plot.show(
                    ui,
                    Rect::from_min_size(top_left + vec2(0.0, h + GAP), size),
                    &mut self.pxu,
                    &mut self.editable_path,
                    &mut self.ui_state,
                );

                self.xm_plot.show(
                    ui,
                    Rect::from_min_size(top_left + vec2(w + GAP, h + GAP), size),
                    &mut self.pxu,
                    &mut self.editable_path,
                    &mut self.ui_state,
                );
            }
        });

        let mut close_dialog = false;
        if let Some(ref mut s) = self.path_dialog_text {
            egui::Window::new("Load/save path")
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
                            let saved_path: Result<pxu::path::SavedPath, _> =
                                serde_json::from_str(s);
                            if let Ok(saved_path) = saved_path {
                                close_dialog = true;
                                self.pxu.consts = saved_path.consts;
                                self.pxu.state = saved_path.base_path.start.clone();
                                self.ui_state.active_point = saved_path.base_path.excitation;
                                self.pxu.path = pxu::Path::from_base_path(
                                    saved_path.base_path,
                                    &self.pxu.contours,
                                    self.pxu.consts,
                                );
                            }
                        }
                    });
                });
        }
        if close_dialog {
            self.path_dialog_text = None;
        }
    }
}

impl PxuGuiApp {
    fn draw_coupling_controls(&mut self, ui: &mut egui::Ui) {
        let old_consts = self.pxu.consts;
        let mut new_consts = self.pxu.consts;

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
            egui::Slider::from_get_set(1.0..=8.0, |n| {
                if let Some(n) = n {
                    let n = n as usize;
                    self.pxu.state = pxu::State::new(n, self.pxu.consts);
                    self.ui_state.active_point = n / 2;
                    self.editable_path.clear();
                    self.anim_data.stop();
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
            self.anim_data.stop();
        }
    }

    fn draw_animation_controls(&mut self, ui: &mut egui::Ui, visible: bool) {
        let enabled = false;

        ui.scope(|ui| {
            ui.set_visible(visible);

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        enabled && self.anim_data.is_paused(),
                        egui::Button::new("⏮"),
                    )
                    .clicked()
                {
                    self.pxu.state.points = self.anim_data.goto_start();
                }

                if self.anim_data.is_stopped() {
                    if ui.add_enabled(enabled, egui::Button::new("⏵")).clicked() {
                        // self.pxu.state.points = self.anim_data.start(&self.pxu.paths);
                    }
                } else if self.anim_data.is_paused() {
                    if ui.add_enabled(enabled, egui::Button::new("⏵")).clicked() {
                        self.anim_data.unpause();
                    }
                } else if ui.add_enabled(enabled, egui::Button::new("⏸")).clicked() {
                    self.anim_data.pause();
                }

                if ui
                    .add_enabled(!self.anim_data.is_stopped(), egui::Button::new("⏹"))
                    .clicked()
                {
                    self.anim_data.stop();
                }

                if ui
                    .add_enabled(
                        enabled && self.anim_data.is_paused(),
                        egui::Button::new("⏭"),
                    )
                    .clicked()
                {
                    self.pxu.state.points = self.anim_data.goto_end();
                }
            });

            ui.add_enabled(enabled, egui::Button::new("→"));
            ui.add_enabled(enabled, egui::Button::new("⇤"));
            ui.add_enabled(enabled, egui::Button::new("↔"));

            ui.add_enabled(
                enabled && self.anim_data.total_len > 0.0 && self.anim_data.is_paused(),
                egui::Slider::from_get_set(0.0..=1.0, |v| {
                    if let Some(v) = v {
                        self.anim_data.t = v * self.anim_data.total_len;
                    }
                    self.anim_data.t / self.anim_data.total_len
                })
                .show_value(false),
            );

            ui.add(
                egui::Slider::new(&mut self.anim_data.speed, 1.0..=100.0)
                    .text("Speed")
                    .show_value(false),
            );
        });
    }

    fn draw_path_editing_controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("Edit path");
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.ui_state.edit_path, egui::Button::new("Edit"))
                .clicked()
            {
                self.ui_state.edit_path = true;
            }

            if ui
                .add_enabled(self.ui_state.edit_path, egui::Button::new("Clear"))
                .clicked()
            {
                self.editable_path.clear();
            }

            if ui
                .add_enabled(self.ui_state.edit_path, egui::Button::new("Done"))
                .clicked()
            {
                self.ui_state.edit_path = false;

                if !self.editable_path.states.is_empty() {
                    let base_path = pxu::path::BasePath::from_editable_path(
                        &self.editable_path,
                        self.ui_state.edit_path_component,
                        self.ui_state.active_point,
                    );
                    self.pxu.path = pxu::path::Path::from_base_path(
                        base_path,
                        &self.pxu.contours,
                        self.pxu.consts,
                    );
                } else {
                    self.pxu.path = Default::default();
                }
            }
            // });
            // ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.ui_state.edit_path, egui::Button::new("Load/Save"))
                .clicked()
            {
                let s = if let Some(base_path) = &self.pxu.path.base_path {
                    let saved_path = pxu::path::SavedPath {
                        base_path: base_path.clone(),
                        consts: self.pxu.consts,
                    };
                    serde_json::json!(saved_path).to_string()
                } else {
                    String::new()
                };
                self.path_dialog_text = Some(s);
            }

            if ui
                .add_enabled(self.ui_state.edit_path, egui::Button::new("Cancel"))
                .clicked()
            {
                self.ui_state.edit_path = false;
            }
        });
        ui.label("Component:");
        ui.horizontal(|ui| {
            for component in [
                pxu::Component::P,
                pxu::Component::Xp,
                pxu::Component::Xm,
                pxu::Component::U,
            ] {
                ui.add_enabled_ui(self.ui_state.edit_path, |ui| {
                    ui.radio_value(
                        &mut self.ui_state.edit_path_component,
                        component,
                        format!("{component:?}"),
                    )
                });
            }
        });
    }

    fn draw_state_information(&mut self, ui: &mut egui::Ui) {
        let active_point = &self.pxu.state.points[self.ui_state.active_point];
        ui.separator();
        {
            ui.label(format!("Momentum: {:.3}", self.pxu.state.p()));
            ui.label(format!("Energy: {:.3}", self.pxu.state.en(self.pxu.consts)));
        }

        ui.separator();

        {
            ui.label("Active excitation:");

            ui.label(format!("Momentum: {:.3}", active_point.p));

            ui.label(format!("Energy: {:.3}", active_point.en(self.pxu.consts)));

            ui.add_space(10.0);
            ui.label(format!("x+: {:.3}", active_point.xp));
            ui.label(format!("x-: {:.3}", active_point.xm));
            ui.label(format!("u: {:.3}", active_point.u));

            ui.add_space(10.0);
            ui.label("Branch info:");

            ui.label(format!(
                "Log branches: {:+} {:+}",
                active_point.sheet_data.log_branch_p, active_point.sheet_data.log_branch_m
            ));

            ui.label(format!("E branch: {:+} ", active_point.sheet_data.e_branch));
            ui.label(format!(
                "U branch: ({:+},{:+}) ",
                active_point.sheet_data.u_branch.0, active_point.sheet_data.u_branch.1
            ));

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
                ui.label(format!("p = {p:.3} m = {m:.3}"));
            }
        }
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            self.draw_coupling_controls(ui);

            ui.label("U cuts: ");
            ui.horizontal(|ui| {
                for typ in UCutType::all() {
                    ui.radio_value(&mut self.ui_state.u_cut_type, typ, format!("{typ}"));
                }
            });

            if ui.add(egui::Button::new("Reset")).clicked() {
                self.pxu.consts = CouplingConstants::new(2.0, 5);
                self.pxu.state = pxu::State::new(self.pxu.state.points.len(), self.pxu.consts);
            }

            self.draw_state_information(ui);
            self.draw_animation_controls(ui, false);

            if self.ui_state.show_dev {
                self.draw_path_editing_controls(ui);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });

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
                }
            });
        });
    }
}
