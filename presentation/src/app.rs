use egui::{vec2, Pos2};
use pxu::kinematics::CouplingConstants;
use pxu::Pxu;
use pxu_plot::{Plot, PlotState};

use egui_extras::RetainedImage;

// type FrameSetupFunction = fn(&mut PlotData);

// enum Frame {
//     Images,
//     Plot(FrameSetupFunction),
// }

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct PlotData {
    pxu: pxu::Pxu,
    p_plot: Plot,
    xp_plot: Plot,
    xm_plot: Plot,
    u_plot: Plot,
    #[serde(skip)]
    plot_state: PlotState,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PresentationApp {
    plot_data: PlotData,
    #[serde(skip)]
    images: Vec<Vec<RetainedImage>>,
    image_index: (usize, usize),
    // #[serde(skip)]
    // frames: Vec<Frame>,
}

impl Default for PlotData {
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
        }
    }
}

impl PresentationApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     let app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //     return app;
        // }

        let mut app: PresentationApp = Default::default();

        let mut paths = std::fs::read_dir("./presentation/images")
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        paths.sort_by_key(|p| p.file_name());

        let mut images = vec![];
        for path in paths {
            log::info!("Name: {}", path.path().display());

            let image_buffer = image::open(path.path()).unwrap().to_rgba8();
            let pixels = image_buffer.as_flat_samples();

            if pixels.as_slice().iter().all(|p| *p == 0xFF) {
                log::info!("Empty image");
                if !images.is_empty() {
                    app.images.push(images);
                    images = vec![];
                }
            } else {
                let size = [image_buffer.width() as _, image_buffer.height() as _];
                let img = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let img = egui_extras::RetainedImage::from_color_image(
                    path.path().display().to_string(),
                    img,
                );
                images.push(img);
            }
        }

        app
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
        if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
            _frame.close();
        }
        {
            let start = chrono::Utc::now();
            while (chrono::Utc::now() - start).num_milliseconds()
                < (1000.0 / 20.0f64).floor() as i64
            {
                if self.plot_data.pxu.contours.update(
                    self.plot_data.pxu.state.points[0].p.re.floor() as i32,
                    self.plot_data.pxu.consts,
                ) {
                    break;
                }
                ctx.request_repaint();
            }
        }

        // let mut style: egui::Style = (*ctx.style()).clone();
        // style.spacing.item_spacing = vec2(0.0, 0.0);
        // ctx.set_style(style.clone());

        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .inner_margin(egui::Margin::same(0.0))
                    .outer_margin(egui::Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();

                let mut plots = {
                    use egui::Rect;
                    const GAP: f32 = 8.0;
                    let w = (rect.width() - 3.0 * GAP) / 2.0;
                    let h = (rect.height() - 3.0 * GAP) / 2.0;
                    let size = vec2(w, h);

                    let top_left = rect.left_top();

                    vec![
                        (
                            &mut self.plot_data.u_plot,
                            Rect::from_min_size(top_left + vec2(w + 2.0 * GAP, GAP), size),
                        ),
                        (
                            &mut self.plot_data.xp_plot,
                            Rect::from_min_size(top_left + vec2(GAP, h + 2.0 * GAP), size),
                        ),
                        (
                            &mut self.plot_data.xm_plot,
                            Rect::from_min_size(
                                top_left + vec2(w + 2.0 * GAP, h + 2.0 * GAP),
                                size,
                            ),
                        ),
                        (
                            &mut self.plot_data.p_plot,
                            Rect::from_min_size(top_left + vec2(GAP, GAP), size),
                        ),
                    ]
                };

                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    let (mut group_index, mut image_index) = self.image_index;

    }
}

impl PresentationApp {
    fn show_relativistic_plot_p(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_two_pos(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            rect,
        );

        let old_clip_rect = ui.clip_rect();
        ui.set_clip_rect(rect);

        let mut shapes = vec![
            egui::Shape::line_segment(
                [to_screen * pos2(0.0, 0.5), to_screen * pos2(1.0, 0.5)],
                egui::Stroke::new(0.75, egui::Color32::DARK_GRAY),
            ),
            egui::Shape::line_segment(
                [to_screen * pos2(0.5, 1.0), to_screen * pos2(0.5, 0.75)],
                egui::Stroke::new(3.0, egui::Color32::BLACK),
            ),
            egui::Shape::line_segment(
                [to_screen * pos2(0.5, 0.0), to_screen * pos2(0.5, 0.25)],
                egui::Stroke::new(3.0, egui::Color32::BLACK),
            ),
            egui::Shape::circle_filled(to_screen * pos2(0.5, 0.25), 3.5, egui::Color32::BLACK),
            egui::Shape::circle_filled(to_screen * pos2(0.5, 0.75), 3.5, egui::Color32::BLACK),
        ];

        ui.painter().extend(shapes);

        ui.set_clip_rect(old_clip_rect);
        ui.painter().add(egui::epaint::Shape::rect_stroke(
            rect,
            egui::epaint::Rounding::same(4.0),
            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
        ));
    }

    fn show_relativistic_plot_theta(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_two_pos(pos2(0.0, -0.125), pos2(1.0, 1.125)),
            rect,
        );

        let old_clip_rect = ui.clip_rect();
        ui.set_clip_rect(rect);

        let mut shapes = vec![];

        for y in [0.25, 0.75] {
            shapes.push(egui::Shape::line_segment(
                [to_screen * pos2(0.0, y), to_screen * pos2(1.0, y)],
                egui::Stroke::new(0.75, egui::Color32::DARK_GRAY),
            ));
        }

        for y in [0.0, 0.5, 1.0] {
            shapes.push(egui::Shape::line_segment(
                [to_screen * pos2(0.0, y), to_screen * pos2(1.0, y)],
                egui::Stroke::new(3.0, egui::Color32::BLACK),
            ));
            shapes.push(egui::Shape::circle_filled(
                to_screen * pos2(0.5, y),
                3.5,
                egui::Color32::BLACK,
            ));
        }

        ui.painter().extend(shapes);

        ui.set_clip_rect(old_clip_rect);
        ui.painter().add(egui::epaint::Shape::rect_stroke(
            rect,
            egui::epaint::Rounding::same(4.0),
            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
        ));
    }
}
