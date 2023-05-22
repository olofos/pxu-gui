use egui::{pos2, vec2, Pos2};
use pxu::kinematics::CouplingConstants;
use pxu_plot::{Plot, PlotState};
use std::collections::HashMap;

use egui_extras::RetainedImage;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct PlotData {
    consts: CouplingConstants,
    p_plot: Plot,
    xp_plot: Plot,
    xm_plot: Plot,
    u_plot: Plot,
    #[serde(skip)]
    plot_state: PlotState,
}

use crate::presentation_description::{
    FrameDescription, PlotDescription, PresentationDescription, RelativisticComponent,
    RelativisticPlotDescription, Value,
};
struct Frame {
    pub image: RetainedImage,
    pub plot: HashMap<pxu::Component, PlotDescription>,
    pub relativistic_plot: HashMap<RelativisticComponent, RelativisticPlotDescription>,
    pub start_time: f64,
    pub duration: Option<f64>,
    pub consts: Option<CouplingConstants>,
}

impl TryFrom<FrameDescription> for Frame {
    type Error = String;

    fn try_from(value: FrameDescription) -> Result<Self, Self::Error> {
        let path = std::path::Path::new("./presentation/images/").join(&value.image);

        let image_buffer = image::open(path.clone())
            .map_err(|_| format!("Could not open image {}", path.display()))?
            .to_rgba8();
        let rgba = image_buffer.as_flat_samples();
        let rgba = rgba.as_slice();

        let size = [image_buffer.width() as _, image_buffer.height() as _];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba);
        let image = egui_extras::RetainedImage::from_color_image(value.image, color_image);

        let consts = value
            .consts
            .map(|[h, k]| CouplingConstants::new(h, k as i32));

        let FrameDescription {
            plot,
            relativistic_plot,
            duration,
            ..
        } = value;

        Ok(Self {
            image,
            plot,
            relativistic_plot,
            start_time: 0.0,
            duration,
            consts,
        })
    }
}

impl Frame {
    fn start(&mut self, plot_data: &mut PlotData, start_time: f64) {
        for (component, descr) in self.plot.iter() {
            let plot = match component {
                pxu::Component::P => &mut plot_data.p_plot,
                pxu::Component::Xp => &mut plot_data.xp_plot,
                pxu::Component::Xm => &mut plot_data.xm_plot,
                pxu::Component::U => &mut plot_data.u_plot,
                _ => unimplemented!(),
            };

            if let Some(origin) = descr.origin {
                plot.origin = egui::Pos2::from(origin);
            }

            if let Some(Value::Const(height)) = descr.height {
                plot.height = height;
            }

            if let Some(consts) = self.consts {
                plot_data.consts = consts;
            }
        }
        self.start_time = start_time;
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state

pub struct PresentationApp {
    plot_data: PlotData,
    // #[serde(skip)]
    // images: Vec<Vec<RetainedImage>>,
    // image_index: (usize, usize),
    #[serde(skip)]
    pxu: Vec<pxu::Pxu>,
    #[serde(skip)]
    frames: Vec<Frame>,
    #[serde(skip)]
    frame_index: usize,
    #[serde(skip)]
    frame_start: f64,
}

impl Default for PlotData {
    fn default() -> Self {
        let consts = CouplingConstants::new(2.0, 5);

        Self {
            consts,
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

        let path = std::path::Path::new("./presentation/images/presentation.toml");
        let presentation_toml = std::fs::read_to_string(path).unwrap();
        let presentation: Result<PresentationDescription, _> = toml::from_str(&presentation_toml);

        app.frames = presentation
            .unwrap()
            .frame
            .into_iter()
            .map(|f| Frame::try_from(f).unwrap())
            .collect();

        app.frames[0].start(&mut app.plot_data, 0.0);

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
        let pxu = if let Some(i) = self
            .pxu
            .iter()
            .position(|pxu| pxu.consts == self.plot_data.consts)
        {
            &mut self.pxu[i]
        } else {
            let mut pxu = pxu::Pxu::new(self.plot_data.consts);
            pxu.state = pxu::State::new(1, pxu.consts);
            self.pxu.push(pxu);
            self.pxu.last_mut().unwrap()
        };

        if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
            _frame.close();
        }
        {
            let start = chrono::Utc::now();
            while (chrono::Utc::now() - start).num_milliseconds()
                < (1000.0 / 20.0f64).floor() as i64
            {
                if pxu
                    .contours
                    .update(pxu.state.points[0].p.re.floor() as i32, pxu.consts)
                {
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
                // log::info!("{:?}", rect.size());

                let prev_frame_index = self.frame_index;

                if self.frames[self.frame_index].start_time == 0.0 {
                    self.frames[self.frame_index].start_time = ui.input(|i| i.time);
                }

                let next = if let Some(duration) = self.frames[self.frame_index].duration {
                    let frame_end = self.frames[self.frame_index].start_time + duration;
                    let now = ui.input(|i| i.time);
                    now > frame_end
                } else {
                    false
                };

                if (next || ui.input(|i| i.key_pressed(egui::Key::ArrowRight)))
                    && self.frame_index < self.frames.len() - 1
                {
                    self.frame_index += 1;
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    loop {
                        if self.frame_index > 0 {
                            self.frame_index -= 1;
                        } else {
                            break;
                        }
                        if self.frames[self.frame_index].duration.is_none() {
                            break;
                        }
                    }
                }

                if self.frame_index != prev_frame_index {
                    self.frames[self.frame_index].start(&mut self.plot_data, ui.input(|i| i.time));
                }

                let frame = &self.frames[self.frame_index];

                let frame_time = ui.input(|i| i.time - frame.start_time);

                ui.vertical_centered(|ui| {
                    // let image_size = image.size_vec2();
                    // image.show_size(ui, image_size * (rect.height() / image_size.y));
                    frame.image.show_size(ui, rect.size());
                });

                for (component, descr) in frame.plot.iter() {
                    let plot = match component {
                        pxu::Component::P => &mut self.plot_data.p_plot,
                        pxu::Component::Xp => &mut self.plot_data.xp_plot,
                        pxu::Component::Xm => &mut self.plot_data.xm_plot,
                        pxu::Component::U => &mut self.plot_data.u_plot,
                        _ => unimplemented!(),
                    };

                    let w = rect.width();
                    let h = rect.height();

                    let x1 = descr.rect[0][0] * w / 16.0;
                    let x2 = descr.rect[1][0] * w / 16.0;

                    let y1 = descr.rect[0][1] * h / 9.0;
                    let y2 = descr.rect[1][1] * h / 9.0;

                    let plot_rect = egui::Rect::from_two_pos(pos2(x1, y1), pos2(x2, y2));

                    plot.show(ui, plot_rect, pxu, &mut self.plot_data.plot_state);
                }

                for (component, descr) in frame.relativistic_plot.iter() {
                    let plot_func: fn(&mut egui::Ui, egui::Rect, f32, f32, Option<[f32; 2]>) =
                        match component {
                            RelativisticComponent::P => Self::show_relativistic_plot_p,
                            RelativisticComponent::Theta => Self::show_relativistic_plot_theta,
                        };

                    let w = rect.width();
                    let h = rect.height();

                    let drect = descr.rect.get(frame_time);

                    let x1 = drect[0][0] * w / 16.0;
                    let x2 = drect[1][0] * w / 16.0;

                    let y1 = drect[0][1] * h / 9.0;
                    let y2 = drect[1][1] * h / 9.0;

                    let plot_rect = egui::Rect::from_two_pos(pos2(x1, y1), pos2(x2, y2));

                    let height = if let Some(ref height) = descr.height {
                        height.get(frame_time)
                    } else {
                        match component {
                            RelativisticComponent::P => 4.0,
                            RelativisticComponent::Theta => 1.25,
                        }
                    };

                    let m = descr.m.get(frame_time);
                    plot_func(
                        ui,
                        plot_rect,
                        height,
                        m,
                        descr.point.clone().map(|p| p.get(frame_time)),
                    );
                }
            });
        ctx.request_repaint();
    }
}

impl PresentationApp {
    fn show_relativistic_plot_p(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        height: f32,
        m: f32,
        point: Option<[f32; 2]>,
    ) {
        let width = height * rect.aspect_ratio();
        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_center_size(pos2(0.0, 0.0), vec2(width, height)),
            rect,
        );

        let old_clip_rect = ui.clip_rect();
        ui.set_clip_rect(rect);

        let mut shapes = vec![
            egui::Shape::line_segment(
                [
                    to_screen * pos2(-width / 2.0, 0.0),
                    to_screen * pos2(width / 2.0, 0.0),
                ],
                egui::Stroke::new(0.75, egui::Color32::DARK_GRAY),
            ),
            egui::Shape::line_segment(
                [
                    to_screen * pos2(0.0, m),
                    to_screen * pos2(0.0, height / 2.0),
                ],
                egui::Stroke::new(3.0, egui::Color32::BLACK),
            ),
            egui::Shape::line_segment(
                [
                    to_screen * pos2(0.0, -m),
                    to_screen * pos2(0.0, -height / 2.0),
                ],
                egui::Stroke::new(3.0, egui::Color32::BLACK),
            ),
            egui::Shape::circle_filled(to_screen * pos2(0.0, m), 3.5, egui::Color32::BLACK),
            egui::Shape::circle_filled(to_screen * pos2(0.0, -m), 3.5, egui::Color32::BLACK),
        ];

        if let Some(point) = point {
            use std::f32::consts::PI;

            let x = point[0] * (point[1] * 2.0 * PI).cos();
            let y = point[0] * (point[1] * 2.0 * PI).sin();

            let center = to_screen * pos2(x, -y);

            shapes.push(egui::epaint::Shape::Circle(egui::epaint::CircleShape {
                center,
                radius: 5.0,
                fill: egui::Color32::BLUE,
                stroke: egui::Stroke::NONE,
            }));
        }

        let text = "p";

        ui.fonts(|f| {
            let text_shape = egui::epaint::Shape::text(
                f,
                rect.right_top() + vec2(-10.0, 10.0),
                egui::Align2::RIGHT_TOP,
                text,
                egui::TextStyle::Monospace.resolve(ui.style()),
                egui::Color32::BLACK,
            );

            shapes.push(egui::epaint::Shape::rect_filled(
                text_shape.visual_bounding_rect().expand(6.0),
                egui::Rounding::none(),
                egui::Color32::WHITE,
            ));
            shapes.push(egui::epaint::Shape::rect_stroke(
                text_shape.visual_bounding_rect().expand(4.0),
                egui::Rounding::none(),
                egui::Stroke::new(0.5, egui::Color32::BLACK),
            ));
            shapes.push(text_shape);
        });

        ui.painter().extend(shapes);

        ui.set_clip_rect(old_clip_rect);
        ui.painter().add(egui::epaint::Shape::rect_stroke(
            rect,
            egui::epaint::Rounding::same(4.0),
            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
        ));
    }

    fn show_relativistic_plot_theta(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        height: f32,
        _m: f32,
        point: Option<[f32; 2]>,
    ) {
        let width = 4.0 * height * rect.aspect_ratio();

        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_center_size(pos2(0.0, 0.0), vec2(width, height)),
            rect,
        );

        let old_clip_rect = ui.clip_rect();
        ui.set_clip_rect(rect);

        let mut shapes = vec![];

        for i in 0..=(4 * height.ceil() as i32) {
            let y = -height.ceil() + 0.5 * i as f32;

            shapes.push(egui::Shape::line_segment(
                [
                    to_screen * pos2(-width / 2.0, y),
                    to_screen * pos2(width / 2.0, y),
                ],
                egui::Stroke::new(0.75, egui::Color32::DARK_GRAY),
            ));
        }

        for i in 0..=(4 * height.ceil() as i32) {
            let y = -height.ceil() - 0.25 + 0.5 * i as f32;

            shapes.push(egui::Shape::line_segment(
                [
                    to_screen * pos2(-width / 2.0, y),
                    to_screen * pos2(width / 2.0, y),
                ],
                egui::Stroke::new(3.0, egui::Color32::BLACK),
            ));
            shapes.push(egui::Shape::circle_filled(
                to_screen * pos2(0.0, y),
                3.5,
                egui::Color32::BLACK,
            ));
        }

        if let Some(point) = point {
            let center = to_screen * pos2(point[0], -point[1]);

            shapes.push(egui::epaint::Shape::Circle(egui::epaint::CircleShape {
                center,
                radius: 5.0,
                fill: egui::Color32::BLUE,
                stroke: egui::Stroke::NONE,
            }));
        }

        let text = "θ";

        ui.fonts(|f| {
            let text_shape = egui::epaint::Shape::text(
                f,
                rect.right_top() + vec2(-10.0, 10.0),
                egui::Align2::RIGHT_TOP,
                text,
                egui::TextStyle::Monospace.resolve(ui.style()),
                egui::Color32::BLACK,
            );

            shapes.push(egui::epaint::Shape::rect_filled(
                text_shape.visual_bounding_rect().expand(6.0),
                egui::Rounding::none(),
                egui::Color32::WHITE,
            ));
            shapes.push(egui::epaint::Shape::rect_stroke(
                text_shape.visual_bounding_rect().expand(4.0),
                egui::Rounding::none(),
                egui::Stroke::new(0.5, egui::Color32::BLACK),
            ));
            shapes.push(text_shape);
        });

        ui.painter().extend(shapes);

        ui.set_clip_rect(old_clip_rect);
        ui.painter().add(egui::epaint::Shape::rect_stroke(
            rect,
            egui::epaint::Rounding::same(4.0),
            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
        ));
    }
}
