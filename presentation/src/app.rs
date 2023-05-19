use egui::{pos2, vec2, Pos2};
use pxu::kinematics::CouplingConstants;
use pxu::Pxu;
use pxu_plot::{Plot, PlotState};
use std::collections::HashMap;

use egui_extras::RetainedImage;

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

#[derive(Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
enum RelativisticComponent {
    P,
    Theta,
}

impl std::str::FromStr for RelativisticComponent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "P" => Ok(Self::P),
            "Theta" => Ok(Self::Theta),
            _ => Err("Could not parse component".to_owned()),
        }
    }
}

impl std::fmt::Display for RelativisticComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::P => "P",
                Self::Theta => "Theta",
            },
        )
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct PlotDescription {
    pub rect: [[f32; 2]; 2],
    pub origin: Option<[f32; 2]>,
    pub height: Option<f32>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct RelativisticPlotDescription {
    pub rect: [[f32; 2]; 2],
}

use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct FrameDescription {
    pub image: String,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub plot: HashMap<pxu::Component, PlotDescription>,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub relativistic_plot: HashMap<RelativisticComponent, RelativisticPlotDescription>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PresentationDescription {
    pub frame: Vec<FrameDescription>,
}

struct Frame {
    pub image: RetainedImage,
    pub plot: HashMap<pxu::Component, PlotDescription>,
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

        let plot = value.plot;

        Ok(Self { image, plot })
    }
}

impl Frame {
    fn start(&self, plot_data: &mut PlotData) {
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

            if let Some(height) = descr.height {
                plot.height = height;
            }
        }
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
    frames: Vec<Frame>,
    #[serde(skip)]
    frame_index: usize,
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

        let toml = r#"[[frame]]
        image = "presentation-01.png"
        
        [[frame]]
        image = "presentation-02.png"
        plot.Xp.rect = [[8,4.75],[11.25,8]]
        plot.P.rect = [[8,1],[15,4.25]]
        plot.P.origin = [0,0]
        plot.Xm.rect = [[11.75,4.75],[15,8]]
        
        [[frame]]
        image = "presentation-03.png"
        
        [[frame]]
        image = "presentation-04.png"
        
        [[frame]]
        image = "presentation-05.png"
        
        [[frame]]
        image = "presentation-06.png"
        
        [[frame]]
        image = "presentation-07.png"
        
        [[frame]]
        image = "presentation-08.png"
        
        [[frame]]
        image = "presentation-09.png"
        
        [[frame]]
        image = "presentation-10.png"
        
        [[frame]]
        image = "presentation-11.png"
        
        [[frame]]
        image = "presentation-12.png"
        
        [[frame]]
        image = "presentation-13.png"
        
        [[frame]]
        image = "presentation-14.png"
        
        [[frame]]
        image = "presentation-15.png"
        
        "#;

        let presentation: Result<PresentationDescription, _> = toml::from_str(&toml);
        log::info!("{presentation:?}");

        app.frames = presentation
            .unwrap()
            .frame
            .into_iter()
            .map(|f| Frame::try_from(f).unwrap())
            .collect();

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
                // log::info!("{:?}", rect.size());

                let prev_frame_index = self.frame_index;

                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    if self.frame_index < self.frames.len() - 1 {
                        self.frame_index += 1;
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    if self.frame_index > 0 {
                        self.frame_index -= 1;
                    }
                }

                if self.frame_index != prev_frame_index {
                    self.frames[self.frame_index].start(&mut self.plot_data);
                }

                let frame = &self.frames[self.frame_index];
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

                    // plot.show(
                    //     ui,
                    //     plot_rect,
                    //     &mut self.plot_data.pxu,
                    //     &mut self.plot_data.plot_state,
                    // );

                    self.show_relativistic_plot_theta(ui, plot_rect);
                }
            });
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
