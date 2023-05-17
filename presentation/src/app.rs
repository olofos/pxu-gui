use egui::{vec2, Pos2};
use pxu::kinematics::CouplingConstants;
use pxu::Pxu;
use pxu_plot::{Plot, PlotState};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PresentationApp {
    pxu: pxu::Pxu,
    p_plot: Plot,
    xp_plot: Plot,
    xm_plot: Plot,
    u_plot: Plot,
    #[serde(skip)]
    plot_state: PlotState,
    size: f32,
}

impl Default for PresentationApp {
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
            plot_state: Default::default(),
            size: 0.0,
        }
    }
}

impl PresentationApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            return app;
        }

        Default::default()
    }
}

impl eframe::App for PresentationApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:
        //     egui::menu::bar(ui, |ui| {
        //         ui.menu_button("File", |ui| {
        //             if ui.button("Quit").clicked() {
        //                 frame.close();
        //             }
        //         });
        //     });
        // });
        if ctx.input().key_pressed(egui::Key::Q) {
            _frame.close();
        }
        {
            let start = chrono::Utc::now();
            while (chrono::Utc::now() - start).num_milliseconds()
                < (1000.0 / 20.0f64).floor() as i64
            {
                if self.pxu.contours.update(
                    self.pxu.state.points[0].p.re.floor() as i32,
                    self.pxu.consts,
                ) {
                    break;
                }
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();

            let mut plots = {
                use egui::Rect;
                const GAP: f32 = 8.0;
                let w = (rect.width() - GAP) / 2.0;
                let h = (rect.height() - GAP) / 2.0;
                let size = vec2(w, h);

                let top_left = rect.left_top();

                let t = ctx.input().time;
                let x = 0.5 + 0.5 * (t.cos() * t.cos()) as f32;
                ctx.request_repaint_after(std::time::Duration::from_millis((1000.0 / 20.0) as u64));

                vec![
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
                    (
                        &mut self.p_plot,
                        Rect::from_min_size(top_left, rect.size() * x),
                    ),
                ]
            };

            self.plot_state.reset();

            for (plot, rect) in plots.iter_mut() {
                plot.interact(ui, *rect, &mut self.pxu, &mut self.plot_state);
            }

            for (plot, rect) in plots {
                plot.show(ui, rect, &mut self.pxu, &mut self.plot_state);
            }
        });
    }
}
