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
    DispRelPlotDescription, FrameDescription, PlotDescription, PresentationDescription,
    RelativisticComponent, RelativisticCrossingPath, RelativisticPlotDescription, Value, *,
};
struct Frame {
    pub image: RetainedImage,
    pub plot: HashMap<pxu::Component, PlotDescription>,
    pub relativistic_plot: HashMap<RelativisticComponent, RelativisticPlotDescription>,
    pub disp_rel_plot: Option<DispRelPlotDescription>,
    pub start_time: f64,
    pub duration: Option<f64>,
    pub consts: Option<CouplingConstants>,
    pub cut_filter: Option<pxu_plot::CutFilter>,
}

impl IsAnimated for Frame {
    fn is_animated(&self) -> bool {
        for description in self.plot.values() {
            if description.is_animated() {
                return true;
            }
        }

        for description in self.relativistic_plot.values() {
            if description.is_animated() {
                return true;
            }
        }

        if self.disp_rel_plot.is_animated() {
            return true;
        }

        false
    }
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
            disp_rel_plot,
            cut_filter,
            ..
        } = value;

        Ok(Self {
            image,
            plot,
            relativistic_plot,
            start_time: 0.0,
            duration,
            consts,
            disp_rel_plot,
            cut_filter,
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

            if let Some(Value::Const(origin)) = descr.origin {
                plot.origin = egui::Pos2::from(origin);
            }

            if let Some(Value::Const(height)) = descr.height {
                plot.height = height;
            }
        }

        if let Some(consts) = self.consts {
            plot_data.consts = consts;
        }

        if let Some(ref cut_filter) = self.cut_filter {
            plot_data.plot_state.cut_filter = cut_filter.clone();
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
        let frame = {
            let prev_frame_index = self.frame_index;

            if self.frames[self.frame_index].start_time == 0.0 {
                self.frames[self.frame_index].start_time = ctx.input(|i| i.time);
            }

            let next = if let Some(duration) = self.frames[self.frame_index].duration {
                let frame_end = self.frames[self.frame_index].start_time + duration;
                let now = ctx.input(|i| i.time);
                now > frame_end
            } else {
                false
            };

            if (next || ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)))
                && self.frame_index < self.frames.len() - 1
            {
                self.frame_index += 1;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
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
                self.frames[self.frame_index].start(&mut self.plot_data, ctx.input(|i| i.time));
            }

            &self.frames[self.frame_index]
        };

        let frame_time = ctx.input(|i| i.time - frame.start_time);

        let pxu = if let Some(i) = self
            .pxu
            .iter()
            .position(|pxu| pxu.consts == self.plot_data.consts)
        {
            &mut self.pxu[i]
        } else {
            let mut pxu = pxu::Pxu::new(self.plot_data.consts);
            pxu.state = pxu::State::new(1, pxu.consts);

            pxu.state
                .update(0, pxu::Component::P, 0.1.into(), &pxu.contours, pxu.consts);

            pxu.state
                .update(0, pxu::Component::P, 0.15.into(), &pxu.contours, pxu.consts);

            self.pxu.push(pxu);
            self.pxu.last_mut().unwrap()
        };

        #[cfg(not(target_arch = "wasm32"))]
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

                    if let Some(ref height) = descr.height {
                        if height.is_animated() {
                            plot.height = height.get(frame_time);
                        }
                    }

                    if let Some(ref origin) = descr.origin {
                        if origin.is_animated() {
                            plot.origin = egui::Pos2::from(origin.get(frame_time));
                        }
                    }

                    let w = rect.width();
                    let h = rect.height();

                    let descr_rect = descr.rect.get(frame_time);

                    let x1 = descr_rect[0][0] * w / 16.0;
                    let x2 = descr_rect[1][0] * w / 16.0;

                    let y1 = descr_rect[0][1] * h / 9.0;
                    let y2 = descr_rect[1][1] * h / 9.0;

                    let plot_rect = egui::Rect::from_two_pos(pos2(x1, y1), pos2(x2, y2));

                    plot.interact(ui, plot_rect, pxu, &mut self.plot_data.plot_state);
                    plot.show(ui, plot_rect, pxu, &mut self.plot_data.plot_state);
                }

                for (component, descr) in frame.relativistic_plot.iter() {
                    let plot_func: fn(
                        &mut egui::Ui,
                        egui::Rect,
                        &RelativisticPlotDescription,
                        f64,
                    ) = match component {
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

                    plot_func(ui, plot_rect, descr, frame_time);
                }

                if let Some(ref disp_rel_plot) = frame.disp_rel_plot {
                    let w = rect.width();
                    let h = rect.height();

                    let drect = disp_rel_plot.rect.get(frame_time);

                    let x1 = drect[0][0] * w / 16.0;
                    let x2 = drect[1][0] * w / 16.0;

                    let y1 = drect[0][1] * h / 9.0;
                    let y2 = drect[1][1] * h / 9.0;

                    let plot_rect = egui::Rect::from_two_pos(pos2(x1, y1), pos2(x2, y2));

                    Self::show_disp_rel_plot(
                        ui,
                        plot_rect,
                        disp_rel_plot,
                        self.plot_data.consts,
                        frame_time,
                    );
                }

                if frame.is_animated() {
                    ctx.request_repaint();
                }
            });
    }
}

impl PresentationApp {
    fn show_disp_rel_plot(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        description: &DispRelPlotDescription,
        consts: CouplingConstants,
        frame_time: f64,
    ) {
        let p_height = if let Some(ref p_height) = description.height {
            p_height.get(frame_time)
        } else {
            4.0
        };
        let width = 1.5 * p_height * rect.aspect_ratio();

        let origin = if let Some(ref origin) = description.origin {
            origin.get(frame_time)
        } else {
            0.0
        };

        let x_min = origin - width / 2.0;
        let x_max = origin + width / 2.0;

        let mut values = vec![];
        let steps = 512;

        for i in 0..((x_max - x_min) * steps as f32).ceil() as u32 {
            let p = x_min as f64 + i as f64 / steps as f64;
            let e = pxu::kinematics::en(num::complex::Complex64::from(p), 1.0, consts);
            values.push(pos2(p as f32, e.re as f32));
        }

        let y_min = -0.5;

        let y_max = 1.0
            + values
                .iter()
                .map(|pos| pos.y)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

        let height = y_max - y_min;
        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_center_size(pos2(origin, -(y_min + y_max) / 2.0), vec2(width, height)),
            rect,
        );

        let points = values
            .into_iter()
            .map(|z| to_screen * pos2(z.x, -z.y))
            .collect::<Vec<_>>();

        let old_clip_rect = ui.clip_rect();
        ui.set_clip_rect(rect);

        let mut shapes = vec![
            egui::Shape::line(
                vec![to_screen * pos2(x_min, 0.0), to_screen * pos2(x_max, 0.0)],
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            ),
            egui::Shape::line(
                vec![to_screen * pos2(0.0, -y_min), to_screen * pos2(0.0, -y_max)],
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            ),
        ];

        shapes.extend(
            (y_min.floor() as i32..=y_max.ceil() as i32)
                .filter(|y| *y != 0)
                .map(|y| {
                    egui::Shape::line(
                        vec![
                            to_screen * pos2(x_min, -y as f32),
                            to_screen * pos2(x_max, -y as f32),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                    )
                }),
        );

        shapes.extend((x_min.ceil() as i32..=-1).map(|x| {
            egui::Shape::line(
                vec![
                    to_screen * pos2(x as f32, -y_min),
                    to_screen * pos2(x as f32, -y_max),
                ],
                egui::Stroke::new(1.0, egui::Color32::GRAY),
            )
        }));

        shapes.extend((1..=x_max.floor() as i32).map(|x| {
            egui::Shape::line(
                vec![
                    to_screen * pos2(x as f32, -y_min),
                    to_screen * pos2(x as f32, -y_max),
                ],
                egui::Stroke::new(1.0, egui::Color32::GRAY),
            )
        }));

        shapes.push(egui::Shape::line(
            points,
            egui::Stroke::new(3.0, egui::Color32::BLUE),
        ));

        let text = "E";

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

    fn show_relativistic_plot_p(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        description: &RelativisticPlotDescription,
        frame_time: f64, // height: f32,
                         // m: f32,
                         // point: Option<[f32; 2]>,
    ) {
        let height = if let Some(ref height) = description.height {
            height.get(frame_time)
        } else {
            4.0
            // RelativisticComponent::Theta => 1.25,
        };
        let m = description.m.get(frame_time);
        let point = description.point.clone().map(|p| p.get(frame_time));

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

            if let Some(ref path) = description.path {
                let (start, mid, end) = match path {
                    RelativisticCrossingPath::Upper => (0, 1, 2),
                    RelativisticCrossingPath::Full | RelativisticCrossingPath::Periodic => {
                        (-1, 1, 3)
                    }
                };

                let steps = 16;

                let points_right = (0..=steps * (mid - start))
                    .map(|i| {
                        let theta = (start * steps + i) as f32 * PI / 2.0 / steps as f32;
                        let z = num::complex::Complex32::from_polar(point[0], theta);
                        to_screen * pos2(z.re, -z.im)
                    })
                    .collect::<Vec<_>>();

                let points_left = (0..=steps * (end - mid))
                    .map(|i| {
                        let theta = (mid * steps + i) as f32 * PI / 2.0 / steps as f32;
                        let z = num::complex::Complex32::from_polar(point[0], theta);
                        to_screen * pos2(z.re, -z.im)
                    })
                    .collect::<Vec<_>>();

                let (straight_points, dashed_points) = if x >= 0.0 {
                    (points_right, points_left)
                } else {
                    (points_left, points_right)
                };

                shapes.push(egui::epaint::Shape::line(
                    straight_points,
                    egui::Stroke::new(2.0, egui::Color32::BLUE),
                ));

                shapes.extend(egui::epaint::Shape::dashed_line(
                    &dashed_points,
                    egui::Stroke::new(2.0, egui::Color32::BLUE),
                    2.5,
                    5.0,
                ));
            }
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
        description: &RelativisticPlotDescription,
        frame_time: f64, // height: f32,
                         // _m: f32,
                         // point: Option<[f32; 2]>,
    ) {
        let height = if let Some(ref height) = description.height {
            height.get(frame_time)
        } else {
            1.25
        };
        let point = description.point.clone().map(|p| p.get(frame_time));

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

            if let Some(ref path) = description.path {
                let (start, end) = match path {
                    RelativisticCrossingPath::Upper => (0.0, 0.5),
                    RelativisticCrossingPath::Full => (-0.5, 0.5),
                    RelativisticCrossingPath::Periodic => (-2.0, 2.0),
                };

                shapes.push(egui::epaint::Shape::line(
                    vec![
                        to_screen * pos2(point[0], -start),
                        to_screen * pos2(point[0], -end),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::BLUE),
                ));
            }
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
