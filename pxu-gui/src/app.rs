// use std::f64::consts::PI;

use egui::style::Margin;
use egui::{vec2, Color32, Pos2, Rect, Stroke, Ui, Vec2};
use num::complex::Complex64;

use ::pxu::kinematics::CouplingConstants;
use ::pxu::pxu;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq, Default)]
enum UCutType {
    #[default]
    Long,
    Short,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state

pub struct TemplateApp {
    #[serde(skip)]
    consts: CouplingConstants,
    #[serde(skip)]
    pxu: pxu::State,
    #[serde(skip)]
    z: num::complex::Complex64,
    #[serde(skip)]
    branch: i32,
    #[serde(skip)]
    contour_generator: pxu::ContourGenerator,
    #[serde(skip)]
    p_plot: Plot,
    #[serde(skip)]
    xp_plot: Plot,
    #[serde(skip)]
    xm_plot: Plot,
    #[serde(skip)]
    u_plot: Plot,
    #[serde(skip)]
    p_range: i32,
    show_dots: bool,
    show_cuts: bool,
    u_cut_type: UCutType,
    #[serde(skip)]
    frame_history: crate::frame_history::FrameHistory,
}

struct Plot {
    component: pxu::Component,
    height: f32,
    width_factor: f32,
    origin: Pos2,
}

#[allow(clippy::too_many_arguments)]
impl Plot {
    fn draw(
        &mut self,
        ui: &mut Ui,
        desired_size: Vec2,
        contour_generator: &mut pxu::ContourGenerator,
        show_dots: bool,
        show_cuts: bool,
        u_cut_type: UCutType,
        pxu: &mut pxu::State,
    ) {
        egui::Frame::canvas(ui.style())
            .outer_margin(Margin::same(0.0))
            .inner_margin(Margin::same(0.0))
            .show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::drag());

                if response.hovered() {
                    let zoom = ui.input().zoom_delta();
                    self.zoom(zoom);

                    if response.dragged() {
                        let delta = response.drag_delta();
                        self.origin -= Vec2::new(
                            delta.x * (self.height / desired_size.y) * (self.width_factor),
                            delta.y * (self.height / desired_size.y),
                        );
                    }
                }

                let rect = response.rect;

                let visible_rect = Rect::from_center_size(
                    self.origin,
                    vec2(
                        self.height * self.width_factor * desired_size.x / desired_size.y,
                        self.height,
                    ),
                );

                let to_screen = eframe::emath::RectTransform::from_to(visible_rect, rect);

                ui.set_clip_rect(rect);

                let origin = to_screen * egui::pos2(0.0, 0.0);

                let mut shapes = if self.component != pxu::Component::P {
                    vec![
                        egui::epaint::Shape::line(
                            vec![
                                egui::pos2(rect.left(), origin.y),
                                egui::pos2(rect.right(), origin.y),
                            ],
                            Stroke::new(0.75, Color32::DARK_GRAY),
                        ),
                        egui::epaint::Shape::line(
                            vec![
                                egui::pos2(origin.x, rect.bottom()),
                                egui::pos2(origin.x, rect.top()),
                            ],
                            Stroke::new(0.75, Color32::DARK_GRAY),
                        ),
                    ]
                } else {
                    vec![]
                };

                let mut stroke = egui::epaint::Stroke::NONE;

                let mut hovered_point = None;

                for j in 0..pxu.points.len() {
                    let z = pxu.points[j].get(self.component);

                    let size = egui::epaint::Vec2::splat(8.0);
                    let center = to_screen * egui::pos2(z.re as f32, -z.im as f32);
                    let point_rect = egui::Rect::from_center_size(center, size);
                    let point_id = response.id.with(j);
                    let point_response = ui.interact(point_rect, point_id, egui::Sense::drag());

                    if point_response.hovered() {
                        hovered_point = Some(j);
                    }

                    if point_response.dragged() {
                        pxu.active_point = j;
                    }

                    if point_response.hovered() || point_response.dragged() {
                        stroke = egui::epaint::Stroke::new(2.0, Color32::LIGHT_BLUE)
                    }

                    if point_response.dragged() {
                        let new_value =
                            to_screen.inverse() * (center + point_response.drag_delta());
                        let new_value = Complex64::new(new_value.x as f64, -new_value.y as f64);

                        let crossed_cuts = contour_generator
                            .get_crossed_cuts(
                                &pxu.points[pxu.active_point],
                                self.component,
                                new_value,
                                u_cut_type == UCutType::Long,
                            )
                            .collect::<Vec<_>>();

                        pxu.points[pxu.active_point].update(
                            self.component,
                            new_value,
                            &crossed_cuts,
                        );

                        for i in (pxu.active_point + 1)..pxu.points.len() {
                            let new_value = if pxu.points[i - 1].sheet_data.e_branch > 0 {
                                ::pxu::kinematics::xm(
                                    pxu.points[i - 1].p,
                                    1.0,
                                    pxu.points[i - 1].consts,
                                )
                            } else {
                                ::pxu::kinematics::xm_crossed(
                                    pxu.points[i - 1].p,
                                    1.0,
                                    pxu.points[i - 1].consts,
                                )
                            };
                            let crossed_cuts = contour_generator
                                .get_crossed_cuts(
                                    &pxu.points[i],
                                    pxu::Component::Xp,
                                    new_value,
                                    u_cut_type == UCutType::Long,
                                )
                                .collect::<Vec<_>>();
                            pxu.points[i].update(pxu::Component::Xp, new_value, &crossed_cuts);
                        }
                    }

                    for i in (0..pxu.active_point).rev() {
                        let new_value = if pxu.points[i + 1].sheet_data.e_branch > 0 {
                            ::pxu::kinematics::xp(
                                pxu.points[i + 1].p,
                                1.0,
                                pxu.points[i + 1].consts,
                            )
                        } else {
                            ::pxu::kinematics::xp_crossed(
                                pxu.points[i + 1].p,
                                1.0,
                                pxu.points[i + 1].consts,
                            )
                        };
                        let crossed_cuts = contour_generator
                            .get_crossed_cuts(
                                &pxu.points[i],
                                pxu::Component::Xm,
                                new_value,
                                u_cut_type == UCutType::Long,
                            )
                            .collect::<Vec<_>>();
                        pxu.points[i].update(pxu::Component::Xm, new_value, &crossed_cuts);
                    }
                }

                let grid_contours = contour_generator.get_grid(self.component);

                for grid_line in grid_contours {
                    let points = grid_line
                        .path
                        .iter()
                        .map(|z| to_screen * egui::pos2(z.re as f32, -z.im as f32))
                        .collect::<Vec<_>>();

                    shapes.push(egui::epaint::Shape::line(
                        points.clone(),
                        Stroke::new(0.75, Color32::GRAY),
                    ));

                    if show_dots {
                        for center in points {
                            shapes.push(egui::epaint::Shape::Circle(egui::epaint::CircleShape {
                                center,
                                radius: 1.5,
                                fill: Color32::RED,
                                stroke: Stroke::NONE,
                            }));
                        }
                    }
                }

                for (i, pt) in pxu.points.iter().enumerate() {
                    let z = pt.get(self.component);
                    let center = to_screen * egui::pos2(z.re as f32, -z.im as f32);

                    let radius = if let Some(hovered_point) = hovered_point {
                        if hovered_point == i {
                            6.0
                        } else {
                            4.0
                        }
                    } else {
                        4.0
                    };

                    shapes.push(egui::epaint::Shape::Circle(egui::epaint::CircleShape {
                        center,
                        radius,
                        fill: if i == pxu.active_point {
                            Color32::BLUE
                        } else {
                            Color32::GRAY
                        },
                        stroke: if i == pxu.active_point {
                            stroke
                        } else {
                            egui::epaint::Stroke::NONE
                        },
                    }));
                }

                let mut branch_point_shapes = vec![];

                if show_cuts {
                    let shift = if self.component == pxu::Component::U {
                        2.0 * (pxu.points[pxu.active_point].sheet_data.log_branch_p
                            * pxu.points[pxu.active_point].consts.k())
                            as f32
                            / pxu.points[pxu.active_point].consts.h as f32
                    } else {
                        0.0
                    };

                    let long_cuts = match u_cut_type {
                        UCutType::Long => true,
                        UCutType::Short => false,
                    };

                    let visible_cuts = contour_generator
                        .get_visible_cuts(&pxu.points[pxu.active_point], self.component, long_cuts)
                        .collect::<Vec<_>>();

                    for cut in visible_cuts {
                        let color = match cut.typ {
                            pxu::CutType::E => Color32::BLACK,

                            pxu::CutType::Log(comp) => {
                                if comp == pxu::Component::Xp {
                                    Color32::from_rgb(255, 128, 128)
                                } else {
                                    Color32::from_rgb(128, 255, 128)
                                }
                            }

                            pxu::CutType::ULongNegative(comp) => {
                                if !long_cuts {
                                    continue;
                                }

                                if comp == pxu::Component::Xp {
                                    Color32::from_rgb(255, 0, 0)
                                } else {
                                    Color32::from_rgb(0, 192, 0)
                                }
                            }

                            pxu::CutType::ULongPositive(comp) => {
                                if !long_cuts {
                                    continue;
                                }
                                if comp == pxu::Component::Xp {
                                    Color32::from_rgb(255, 0, 0)
                                } else {
                                    Color32::from_rgb(0, 192, 0)
                                }
                            }

                            pxu::CutType::UShortScallion(comp) => {
                                if long_cuts {
                                    continue;
                                }
                                if comp == pxu::Component::Xp {
                                    Color32::from_rgb(255, 0, 0)
                                } else {
                                    Color32::from_rgb(0, 192, 0)
                                }
                            }

                            pxu::CutType::UShortKidney(comp) => {
                                if long_cuts {
                                    continue;
                                }

                                if comp == pxu::Component::Xp {
                                    Color32::from_rgb(255, 0, 0)
                                } else {
                                    Color32::from_rgb(0, 192, 0)
                                }
                            }
                            _ => Color32::from_rgb(255, 128, 0),
                        };

                        for points in cut.paths.iter() {
                            let points = points
                                .iter()
                                .map(|z| {
                                    to_screen * egui::pos2(z.re as f32, -(z.im as f32 - shift))
                                })
                                .collect::<Vec<_>>();

                            match cut.typ {
                                pxu::CutType::UShortKidney(_) | pxu::CutType::ULongNegative(_) => {
                                    egui::epaint::Shape::dashed_line_many(
                                        &points.clone(),
                                        Stroke::new(3.0, color),
                                        4.0,
                                        4.0,
                                        &mut shapes,
                                    );
                                }
                                _ => {
                                    shapes.push(egui::epaint::Shape::line(
                                        points.clone(),
                                        Stroke::new(3.0, color),
                                    ));
                                }
                            }

                            if show_dots {
                                for center in points {
                                    shapes.push(egui::epaint::Shape::Circle(
                                        egui::epaint::CircleShape {
                                            center,
                                            radius: 2.5,
                                            fill: Color32::RED,
                                            stroke: Stroke::NONE,
                                        },
                                    ));
                                }
                            }
                        }

                        if let Some(ref z) = cut.branch_point {
                            let center =
                                to_screen * egui::pos2(z.re as f32, -(z.im as f32 - shift));
                            branch_point_shapes.push(egui::epaint::Shape::Circle(
                                egui::epaint::CircleShape {
                                    center,
                                    radius: 4.0,
                                    fill: color,
                                    stroke: Stroke::NONE,
                                },
                            ));
                        }
                    }
                }

                {
                    let f = ui.fonts();

                    let text = match self.component {
                        pxu::Component::P => "p",
                        pxu::Component::U => "u",
                        pxu::Component::Xp => "X+",
                        pxu::Component::Xm => "X-",
                    };

                    let text_shape = egui::epaint::Shape::text(
                        &f,
                        rect.right_top() + vec2(-10.0, 10.0),
                        egui::Align2::RIGHT_TOP,
                        text,
                        egui::TextStyle::Monospace.resolve(ui.style()),
                        Color32::BLACK,
                    );
                    shapes.push(egui::epaint::Shape::rect_filled(
                        text_shape.visual_bounding_rect().expand(6.0),
                        egui::Rounding::none(),
                        Color32::WHITE,
                    ));
                    shapes.push(egui::epaint::Shape::rect_stroke(
                        text_shape.visual_bounding_rect().expand(4.0),
                        egui::Rounding::none(),
                        egui::Stroke::new(0.5, Color32::BLACK),
                    ));
                    shapes.push(text_shape);
                }

                painter.extend(shapes);
                painter.extend(branch_point_shapes);
            });
    }

    fn zoom(&mut self, zoom: f32) {
        self.height /= zoom;
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        let bound_state_number = 3;

        let consts = CouplingConstants::new(2.0, 5);
        let p_range = 0;

        let state = pxu::State::new(bound_state_number, consts);
        log::info!(
            "\nxp         xm         u\n{}",
            state
                .points
                .iter()
                .map(|pt| format!("{:.2} {:.2} {:.2}", pt.xp, pt.xm, pt.u))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Self {
            consts,
            // pxu: vec![pxu::Point::new(p0, consts), pxu::Point::new(p_conj, consts)],
            pxu: state,
            z: num::complex::Complex::new(0.0, 0.5),
            branch: 1,
            contour_generator: pxu::ContourGenerator::new(),
            p_plot: Plot {
                component: pxu::Component::P,
                height: 0.75,
                width_factor: 1.5,
                origin: Pos2::new(((2 * p_range + 1) as f32) * 0.5, 0.0),
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
            p_range,
            show_dots: false,
            show_cuts: true,
            u_cut_type: Default::default(),
            frame_history: Default::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_history
            .on_new_frame(ctx.input().time, _frame.info().cpu_usage);

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        let old_consts = self.consts;
        let old_p_range = self.p_range;

        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.add(
                egui::Slider::new(&mut self.consts.h, 0.1..=10.0)
                    .text("h")
                    .logarithmic(true),
            );

            ui.add(
                egui::Slider::from_get_set(0.0..=10.0, |v| self.consts.get_set_k(v))
                    .integer()
                    .text("k"),
            );
            ui.add(egui::Slider::new(&mut self.p_range, -10..=5).text("Range"));
            ui.add(
                egui::Slider::from_get_set(1.0..=((2 * self.pxu.consts.k()) as f64), |n| {
                    if let Some(n) = n {
                        self.pxu = pxu::State::new(n as usize, self.pxu.consts);
                    }
                    self.pxu.points.len() as f64
                })
                .integer()
                .text("M"),
            );

            ui.add(egui::Checkbox::new(&mut self.show_dots, "Show dots"));
            ui.add(egui::Checkbox::new(&mut self.show_cuts, "Show cuts"));

            ui.horizontal(|ui| {
                ui.label("U cuts: ");
                ui.radio_value(&mut self.u_cut_type, UCutType::Long, "Long");
                ui.radio_value(&mut self.u_cut_type, UCutType::Short, "Short");
            });

            if ui.add(egui::Button::new("Reset")).clicked() {
                *self = Self::default();
            }

            ui.separator();

            {
                ui.label(format!(
                    "Momentum: {:.2}",
                    self.pxu.points.iter().map(|pxu| pxu.p).sum::<Complex64>()
                ));

                {
                    let en = self
                        .pxu
                        .points
                        .iter()
                        .map(|pxu| {
                            let xp = pxu.xp;
                            let xm = pxu.xm;
                            -Complex64::i() * pxu.consts.h / 2.0 * (xp - 1.0 / xp - xm + 1.0 / xm)
                        })
                        .sum::<Complex64>();
                    ui.label(format!("Energy: {:.2}", en));
                }

                ui.label(format!(
                    "Log branches: {:+} {:+}",
                    self.pxu.points[self.pxu.active_point]
                        .sheet_data
                        .log_branch_p,
                    self.pxu.points[self.pxu.active_point]
                        .sheet_data
                        .log_branch_m
                ));

                ui.label(format!(
                    "E branch: {:+} ",
                    self.pxu.points[self.pxu.active_point].sheet_data.e_branch
                ));
                ui.label(format!(
                    "U branch: ({:+},{:+}) ",
                    self.pxu.points[self.pxu.active_point].sheet_data.u_branch.0,
                    self.pxu.points[self.pxu.active_point].sheet_data.u_branch.1
                ));
            }

            #[cfg(debug_assertions)]
            {
                ui.separator();
                {
                    ui.label(format!("FPS: {}", self.frame_history.fps()));

                    self.frame_history.ui(ui);

                    let (current, total) = self.contour_generator.progress();
                    ui.add(egui::ProgressBar::new(current as f32 / total as f32).show_percentage());
                }
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
            });
        });

        if old_consts != self.consts || old_p_range != self.p_range {
            // self.pxu = PxuPoint::new(
            //     self.pxu.p + 2.0 * PI * (self.p_range - old_p_range) as f64,
            //     self.consts,
            // );
        }

        if old_consts != self.consts {
            self.pxu = pxu::State::new(self.pxu.points.len(), self.consts);
            // self.pxu
            //     .points
            //     .iter_mut()
            //     .for_each(|pxu| pxu.set_coupling_constants(self.consts));

            // self.xp_plot.height *= (self.consts.s() / old_consts.s()) as f32;

            // self.u_plot.height /= (self.consts.h / old_consts.h) as f32;
            // if self.consts.k() > 1 && old_consts.k() > 1 {
            //     self.xp_plot.height *= (2 * self.consts.k()) as f32 / (2 * old_consts.k()) as f32;
            //     self.u_plot.height *= (2 * self.consts.k()) as f32 / (2 * old_consts.k()) as f32;
            // }
        }

        if old_p_range != self.p_range {
            self.p_plot.origin.x += (self.p_range - old_p_range) as f32;
        }

        {
            let start = chrono::Utc::now();
            while (chrono::Utc::now() - start).num_milliseconds()
                < (1000.0 / 20.0f64).floor() as i64
            {
                if self
                    .contour_generator
                    .update(&self.pxu.points[self.pxu.active_point])
                {
                    break;
                }
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            ui.horizontal(|ui| {
                self.p_plot.draw(
                    ui,
                    available_size * vec2(0.49, 0.49),
                    &mut self.contour_generator,
                    self.show_dots,
                    self.show_cuts,
                    self.u_cut_type,
                    &mut self.pxu,
                );

                self.u_plot.draw(
                    ui,
                    available_size * vec2(0.49, 0.49),
                    &mut self.contour_generator,
                    self.show_dots,
                    self.show_cuts,
                    self.u_cut_type,
                    &mut self.pxu,
                );
            });
            ui.horizontal(|ui| {
                self.xp_plot.draw(
                    ui,
                    available_size * vec2(0.49, 0.49),
                    &mut self.contour_generator,
                    self.show_dots,
                    self.show_cuts,
                    self.u_cut_type,
                    &mut self.pxu,
                );

                self.xm_plot.draw(
                    ui,
                    available_size * vec2(0.49, 0.49),
                    &mut self.contour_generator,
                    self.show_dots,
                    self.show_cuts,
                    self.u_cut_type,
                    &mut self.pxu,
                );
            });
        });
    }
}
