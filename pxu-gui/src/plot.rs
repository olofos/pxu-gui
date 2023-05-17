use eframe::emath::RectTransform;
use egui::{vec2, Color32, Pos2, Rect, Stroke, Ui, Vec2};
use num::complex::Complex64;

use crate::ui_state::UiState;
use pxu::kinematics::UBranch;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Plot {
    pub component: pxu::Component,
    pub height: f32,
    pub width_factor: f32,
    pub origin: Pos2,
}

#[derive(Debug)]
struct InteractionPointIndices {
    hovered: Option<usize>,
    dragged: Option<usize>,
}

// enum Response {
//     None,
//     Hovered(),
//     Dragged(),
//     Zoom(),
//     Scroll(),
// }

impl Plot {
    fn interact_with_grid(&mut self, ui: &mut Ui, rect: Rect, response: &egui::Response) {
        if response.dragged() {
            let delta = response.drag_delta();
            self.origin -= Vec2::new(
                delta.x * (self.height / rect.height()) * (self.width_factor),
                delta.y * (self.height / rect.height()),
            );
        }

        if ui.rect_contains_pointer(rect) {
            let zoom = ui.input().zoom_delta();
            self.zoom(zoom);

            let scroll = ui.input().scroll_delta;
            self.origin -= Vec2::new(
                scroll.x * (self.height / rect.height()) * (self.width_factor),
                scroll.y * (self.height / rect.height()),
            );
        }
    }

    fn interact_with_points(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        pxu: &mut pxu::Pxu,
        ui_state: &mut UiState,
        response: &egui::Response,
    ) -> InteractionPointIndices {
        let to_screen = self.to_screen(rect);

        let mut hovered = None;
        let mut dragged = None;

        let state = &mut pxu.state;

        for j in 0..state.points.len() {
            let z = state.points[j].get(self.component);

            let size = egui::epaint::Vec2::splat(8.0);
            let center = to_screen * egui::pos2(z.re as f32, -z.im as f32);
            let point_rect = egui::Rect::from_center_size(center, size);

            let id = (usize::MAX, j);
            let point_id = response.id.with(id);
            let point_response = ui.interact(point_rect, point_id, egui::Sense::drag());

            if point_response.hovered() {
                hovered = Some(j);
            }

            if point_response.dragged() {
                dragged = Some(j);
            }

            if point_response.dragged() {
                let delta = point_response.drag_delta();
                let delta = if ui.input().key_down(egui::Key::E) {
                    vec2(delta.x, 0.0)
                } else if ui.input().key_down(egui::Key::W) {
                    vec2(0.0, delta.y)
                } else {
                    delta
                };
                let new_value = to_screen.inverse() * (center + delta);
                let new_value = Complex64::new(new_value.x as f64, -new_value.y as f64);

                let new_value = if ui.input().key_pressed(egui::Key::R) {
                    match self.component {
                        pxu::Component::P => Complex64::from(new_value.re),
                        pxu::Component::U => {
                            let re = new_value.re;
                            let im = (pxu.consts.h * new_value.im).round() / pxu.consts.h;
                            Complex64::new(re, im)
                        }
                        _ => new_value,
                    }
                } else {
                    new_value
                };

                ui_state.active_point = j;
                state.update(j, self.component, new_value, &pxu.contours, pxu.consts);
            }
        }

        InteractionPointIndices { hovered, dragged }
    }

    fn interact(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        pxu: &mut pxu::Pxu,
        ui_state: &mut UiState,
    ) -> InteractionPointIndices {
        let response = ui.interact(
            rect,
            ui.id().with(format!("{:?}", self.component)),
            egui::Sense::click_and_drag(),
        );

        self.interact_with_grid(ui, rect, &response);
        let interaction_points = self.interact_with_points(ui, rect, pxu, ui_state, &response);

        if response.double_clicked() {
            ui_state.toggle_fullscreen(self.component)
        }

        if ui.input().key_pressed(egui::Key::Home) {
            let z = pxu.state.points[ui_state.active_point].get(self.component);
            self.origin = egui::pos2(z.re as f32, -z.im as f32);
        }

        response.context_menu(|ui| ui_state.menu(ui, Some(self.component)));

        interaction_points
    }

    fn draw_grid(&self, rect: Rect, contours: &pxu::Contours, shapes: &mut Vec<egui::Shape>) {
        let to_screen = self.to_screen(rect);
        let visible_rect = self.visible_rect(rect);
        if self.component != pxu::Component::P {
            let origin = to_screen * egui::pos2(0.0, 0.0);

            shapes.extend([
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
            ]);
        }

        let grid_contours = contours.get_grid(self.component);

        for grid_line in grid_contours {
            if !grid_line.bounding_box.intersects(visible_rect) {
                continue;
            }
            let points = grid_line
                .path
                .iter()
                .map(|z| to_screen * egui::pos2(z.re as f32, -z.im as f32))
                .collect::<Vec<_>>();

            shapes.push(egui::epaint::Shape::line(
                points.clone(),
                Stroke::new(0.75, Color32::GRAY),
            ));
        }
    }

    fn draw_cuts(
        &self,
        rect: Rect,
        pxu: &pxu::Pxu,
        ui_state: &mut UiState,
        shapes: &mut Vec<egui::Shape>,
    ) {
        let to_screen = self.to_screen(rect);

        let mut branch_point_shapes = vec![];

        {
            let shift = if self.component == pxu::Component::U {
                2.0 * (pxu.state.points[ui_state.active_point]
                    .sheet_data
                    .log_branch_p
                    * pxu.consts.k()) as f32
                    / pxu.consts.h as f32
            } else {
                0.0
            };

            let visible_cuts = pxu
                .contours
                .get_visible_cuts(pxu, self.component, ui_state.active_point)
                .collect::<Vec<_>>();

            for cut in visible_cuts {
                let hide_log_cut = |comp| {
                    comp != cut.component
                        || (comp == pxu::Component::Xp
                            && pxu.state.points[ui_state.active_point]
                                .sheet_data
                                .u_branch
                                .1
                                == UBranch::Between)
                        || (comp == pxu::Component::Xm
                            && pxu.state.points[ui_state.active_point]
                                .sheet_data
                                .u_branch
                                .0
                                == UBranch::Between)
                };

                let color = match cut.typ {
                    pxu::CutType::E => Color32::BLACK,

                    pxu::CutType::Log(comp) => {
                        if hide_log_cut(comp) {
                            continue;
                        } else if comp == pxu::Component::Xp {
                            Color32::from_rgb(255, 128, 128)
                        } else {
                            Color32::from_rgb(128, 255, 128)
                        }
                    }

                    pxu::CutType::ULongNegative(_) => {
                        continue;
                    }

                    pxu::CutType::ULongPositive(comp) => {
                        if hide_log_cut(comp) {
                            continue;
                        } else if comp == pxu::Component::Xp {
                            Color32::from_rgb(255, 0, 0)
                        } else {
                            Color32::from_rgb(0, 192, 0)
                        }
                    }

                    pxu::CutType::UShortScallion(comp) => {
                        if comp == pxu::Component::Xp {
                            Color32::from_rgb(255, 0, 0)
                        } else {
                            Color32::from_rgb(0, 192, 0)
                        }
                    }

                    pxu::CutType::UShortKidney(comp) => {
                        if comp == pxu::Component::Xp {
                            Color32::from_rgb(255, 0, 0)
                        } else {
                            Color32::from_rgb(0, 192, 0)
                        }
                    }
                    _ => Color32::from_rgb(255, 128, 0),
                };

                let period_shifts = if cut.periodic {
                    let period = 2.0 * pxu.consts.k() as f64 / pxu.consts.h;
                    (-5..=5).map(|n| period as f32 * n as f32).collect()
                } else {
                    vec![0.0]
                };

                for period_shift in period_shifts.iter() {
                    let points = cut
                        .path
                        .iter()
                        .map(|z| {
                            to_screen
                                * egui::pos2(z.re as f32, -(z.im as f32 - shift + period_shift))
                        })
                        .collect::<Vec<_>>();

                    match cut.typ {
                        pxu::CutType::UShortKidney(_) | pxu::CutType::ULongNegative(_) => {
                            egui::epaint::Shape::dashed_line_many(
                                &points.clone(),
                                Stroke::new(3.0, color),
                                4.0,
                                4.0,
                                shapes,
                            );
                        }
                        _ => {
                            shapes.push(egui::epaint::Shape::line(
                                points.clone(),
                                Stroke::new(3.0, color),
                            ));
                        }
                    }

                    if let Some(ref z) = cut.branch_point {
                        let center = to_screen
                            * egui::pos2(z.re as f32, -(z.im as f32 - shift + period_shift));
                        branch_point_shapes.push(egui::epaint::Shape::Circle(
                            egui::epaint::CircleShape {
                                center,
                                radius: 3.5,
                                fill: color,
                                stroke: Stroke::NONE,
                            },
                        ));
                    }
                }
            }
        }

        shapes.extend(branch_point_shapes);
    }

    fn draw_points(
        &self,
        rect: Rect,
        pxu: &pxu::Pxu,
        ui_state: &UiState,
        points: InteractionPointIndices,
        shapes: &mut Vec<egui::Shape>,
    ) {
        let to_screen = self.to_screen(rect);

        for (i, pt) in pxu.state.points.iter().enumerate() {
            let is_hovered = points.hovered == Some(i);
            let is_dragged = points.dragged == Some(i);
            let is_active = ui_state.active_point == i;

            let z = pt.get(self.component);
            let center = to_screen * egui::pos2(z.re as f32, -z.im as f32);

            let radius = if is_hovered || is_dragged {
                6.0
            } else if is_active {
                5.0
            } else {
                4.0
            };

            let stroke = if is_active {
                egui::epaint::Stroke::new(2.0, Color32::LIGHT_BLUE)
            } else {
                egui::epaint::Stroke::NONE
            };

            let fill = if is_active {
                Color32::BLUE
            } else if pxu.state.points[i]
                .same_sheet(&pxu.state.points[ui_state.active_point], self.component)
            {
                Color32::BLACK
            } else {
                Color32::GRAY
            };

            shapes.push(egui::epaint::Shape::Circle(egui::epaint::CircleShape {
                center,
                radius,
                fill,
                stroke,
            }));

            if ui_state.show_x
                && (self.component == pxu::Component::Xp || self.component == pxu::Component::Xm)
            {
                let z = pt.x;
                let center = to_screen * egui::pos2(z.re as f32, -z.im as f32);
                shapes.push(egui::epaint::Shape::Circle(egui::epaint::CircleShape {
                    center,
                    radius: 4.0,
                    fill: Color32::RED,
                    stroke: egui::epaint::Stroke::NONE,
                }));
            }
        }
    }

    fn draw(
        &self,
        ui: &mut Ui,
        rect: Rect,
        pxu: &mut pxu::Pxu,
        ui_state: &mut UiState,
        interaction_points: InteractionPointIndices,
    ) {
        let to_screen = self.to_screen(rect);

        let mut shapes = vec![];

        self.draw_grid(rect, &pxu.contours, &mut shapes);
        self.draw_cuts(rect, pxu, ui_state, &mut shapes);

        if let Some(path_index) = ui_state.path_index {
            if path_index < pxu.paths.len() {
                for (active_point, segments) in pxu.paths[path_index].segments.iter().enumerate() {
                    let mut points = vec![];
                    let mut same_branch = false;

                    let color = if active_point == ui_state.active_point {
                        Color32::BLUE
                    } else {
                        Color32::GRAY
                    };
                    let width = 2.0;

                    for segment in segments.iter() {
                        let contour = match self.component {
                            pxu::Component::P => &segment.p,
                            pxu::Component::Xp => &segment.xp,
                            pxu::Component::Xm => &segment.xm,
                            pxu::Component::U => &segment.u,
                            pxu::Component::X => &segment.x,
                        };

                        let segment_points = contour
                            .iter()
                            .map(|z| to_screen * egui::pos2(z.re as f32, -(z.im as f32)))
                            .collect::<Vec<_>>();

                        let segment_same_branch = pxu.state.points[ui_state.active_point]
                            .sheet_data
                            .is_same(&segment.sheet_data, self.component);

                        if segment_same_branch != same_branch {
                            if same_branch {
                                shapes.push(egui::Shape::line(points, Stroke::new(width, color)));
                            } else {
                                shapes.extend(egui::Shape::dashed_line(
                                    &points,
                                    Stroke::new(width, color),
                                    2.5,
                                    5.0,
                                ));
                            }
                            points = vec![];
                        }

                        points.extend(segment_points);
                        same_branch = segment_same_branch;
                    }

                    if same_branch {
                        shapes.push(egui::Shape::line(points, Stroke::new(width, color)));
                    } else {
                        shapes.extend(egui::Shape::dashed_line(
                            &points,
                            Stroke::new(width, color),
                            2.5,
                            5.0,
                        ));
                    }
                }
            }
        }

        self.draw_points(rect, pxu, ui_state, interaction_points, &mut shapes);

        {
            let f = ui.fonts();

            let text = match self.component {
                pxu::Component::P => "p",
                pxu::Component::U => "u",
                pxu::Component::Xp => "X+",
                pxu::Component::Xm => "X-",
                pxu::Component::X => "X",
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

        ui.painter().extend(shapes);
    }

    fn to_screen(&self, rect: Rect) -> RectTransform {
        RectTransform::from_to(self.visible_rect(rect), rect)
    }

    fn visible_rect(&self, rect: Rect) -> Rect {
        Rect::from_center_size(
            self.origin,
            vec2(
                self.height * self.width_factor * rect.aspect_ratio(),
                self.height,
            ),
        )
    }

    pub fn show(&mut self, ui: &mut Ui, rect: Rect, pxu: &mut pxu::Pxu, ui_state: &mut UiState) {
        let old_clip_rect = ui.clip_rect();
        ui.set_clip_rect(rect);

        let interaction_points = self.interact(ui, rect, pxu, ui_state);
        self.draw(ui, rect, pxu, ui_state, interaction_points);

        ui.set_clip_rect(old_clip_rect);
        ui.painter().add(egui::epaint::Shape::rect_stroke(
            rect,
            egui::epaint::Rounding::same(4.0),
            Stroke::new(1.0, Color32::DARK_GRAY),
        ));
    }

    fn zoom(&mut self, zoom: f32) {
        self.height /= zoom;
    }
}
