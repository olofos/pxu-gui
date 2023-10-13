use indicatif::ProgressBar;
use itertools::Itertools;
use num::complex::Complex64;
use pxu::GridLine;
use pxu::{
    interpolation::{InterpolationPoint, PInterpolatorMut},
    kinematics::CouplingConstants,
};
use std::fs::File;
use std::io::{prelude::*, BufWriter, Result};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

use crate::cache;
use crate::fig_compiler::FigureCompiler;
use crate::utils::{Settings, Size, TEX_EXT};

#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    pub x_range: Range<f64>,
    pub y_range: Range<f64>,
}

impl Bounds {
    pub fn new(x_range: Range<f64>, y_range: Range<f64>) -> Self {
        Self { x_range, y_range }
    }

    fn width(&self) -> f64 {
        self.x_range.end - self.x_range.start
    }

    fn height(&self) -> f64 {
        self.y_range.end - self.y_range.start
    }

    fn inside(&self, z: &Complex64) -> bool {
        self.x_range.contains(&z.re) && self.y_range.contains(&z.im)
    }

    fn crosses(&self, z1: &Complex64, z2: &Complex64) -> bool {
        (z1.re < self.x_range.start) && (z2.re > self.x_range.end)
            || (z2.re < self.x_range.start) && (z1.re > self.x_range.end)
            || (z1.im < self.y_range.start) && (z2.im > self.y_range.end)
            || (z2.im < self.y_range.start) && (z1.im > self.y_range.end)
    }

    fn expand(self) -> Self {
        let Range { start, end } = self.x_range;
        let d = 1.1 * (end - start);
        let x_range = (start - d)..(end + d);

        let Range { start, end } = self.y_range;
        let d = 1.1 * (end - start);
        let y_range = (start - d)..(end + d);

        Self { x_range, y_range }
    }
}

#[derive(Debug)]
pub struct FigureWriter {
    pub name: String,
    pub caption: String,
    pub bounds: Bounds,
    pub size: Size,
    writer: BufWriter<File>,
    pub plot_count: u64,
    pub component: pxu::Component,
    y_shift: Option<f64>,
    no_component_indicator: bool,
}

impl FigureWriter {
    const FILE_START_1: &str = r#"
\nonstopmode
\documentclass[10pt,a4paper]{article}
\usepackage{luatextra}
\begin{luacode}
progress_file=io.open(""#;
    const FILE_START_2: &str = r#"","w")
\end{luacode}
\usepackage[svgnames]{xcolor}
\usepackage{pgfplots}
\pgfplotsset{compat=1.17}
\usepgfplotslibrary{fillbetween}
\usetikzlibrary{patterns,decorations.markings}
\usepackage[active,tightpage]{preview}
\PreviewEnvironment{tikzpicture}
\setlength\PreviewBorder{0pt}
\pdfvariable suppressoptionalinfo \numexpr 1023 \relax
\begin{document}
\pagestyle{empty}
\begin{tikzpicture}
"#;

    const FILE_END: &str = r#"
\end{tikzpicture}
\directlua{progress_file:write("!")}
\directlua{io.close(progress_file)}
\end{document}
"#;

    fn open_tex_file(name: &str, settings: &Settings, pb: &ProgressBar) -> Result<BufWriter<File>> {
        let mut path = PathBuf::from(&settings.output_dir).join(name);
        path.set_extension(TEX_EXT);

        log::info!("[{name}]: Creating file {}", path.to_string_lossy());
        pb.set_message(format!("generating {}", path.to_string_lossy()));

        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        let mut progress_path = path.clone();
        progress_path.set_extension("prg");
        writer.write_all(Self::FILE_START_1.as_bytes())?;
        write!(writer, "{}", progress_path.to_string_lossy())?;
        writer.write_all(Self::FILE_START_2.as_bytes())?;

        let _ = std::fs::remove_file(progress_path);

        Ok(writer)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        x_range: Range<f64>,
        y0: f64,
        size: Size,
        component: pxu::Component,
        settings: &Settings,
        pb: &ProgressBar,
    ) -> std::io::Result<Self> {
        let mut writer = Self::open_tex_file(name, settings, pb)?;

        let aspect_ratio = match component {
            pxu::Component::P => 1.5,
            _ => 1.0,
        };

        let y_size = (x_range.end - x_range.start) * size.height / size.width / aspect_ratio;
        let y_range = (y0 - y_size / 2.0)..(y0 + y_size / 2.0);

        let bounds = Bounds::new(x_range, y_range);

        let x_min = bounds.x_range.start;
        let x_max = bounds.x_range.end;

        let y_min = bounds.y_range.start;
        let y_max = bounds.y_range.end;

        let width = size.width;
        let height = size.height;

        writeln!(writer, "\\begin{{axis}}[hide axis,scale only axis,ticks=none,xmin={x_min},xmax={x_max},ymin={y_min},ymax={y_max},clip,width={width}cm,height={height}cm]")?;

        Ok(Self {
            name: name.to_owned(),
            writer,
            bounds,
            size,
            plot_count: 0,
            component,
            y_shift: None,
            caption: String::new(),
            no_component_indicator: false,
        })
    }

    pub fn custom_axis(
        name: &str,
        x_range: Range<f64>,
        y_range: Range<f64>,
        size: Size,
        axis_options: &[&str],
        settings: &Settings,
        pb: &ProgressBar,
    ) -> std::io::Result<Self> {
        let mut writer = Self::open_tex_file(name, settings, pb)?;

        let bounds = Bounds::new(x_range, y_range);

        let x_min = bounds.x_range.start;
        let x_max = bounds.x_range.end;

        let y_min = bounds.y_range.start;
        let y_max = bounds.y_range.end;

        let width = size.width;
        let height = size.height;

        writeln!(writer, "\\begin{{axis}}[xmin={x_min},xmax={x_max},ymin={y_min},ymax={y_max},width={width}cm,height={height}cm,{}]", axis_options.join(","))?;

        Ok(Self {
            name: name.to_owned(),
            writer,
            bounds,
            size,
            plot_count: 0,
            component: pxu::Component::P,
            y_shift: None,
            caption: String::new(),
            no_component_indicator: true,
        })
    }

    pub fn no_component_indicator(&mut self) {
        self.no_component_indicator = true;
    }

    fn format_coordinate(&self, p: Complex64) -> String {
        format!(
            "({:.5},{:.5})",
            p.re,
            p.im + self.y_shift.unwrap_or_default()
        )
    }

    fn format_contour(&self, contour: Vec<Complex64>) -> Vec<String> {
        let mut coordinates = contour
            .into_iter()
            .map(|z| self.format_coordinate(z))
            .collect::<Vec<_>>();
        coordinates.dedup();
        coordinates
    }

    pub fn crop(&self, contour: &Vec<Complex64>) -> Vec<Complex64> {
        if contour.len() < 2 {
            return vec![];
        }

        let mut coordinates: Vec<Complex64> = vec![];

        let y_shift = Complex64::new(0.0, self.y_shift.unwrap_or_default());

        let bounds = self.bounds.clone().expand();

        let include = |z1, z2| {
            let z1 = z1 + y_shift;
            let z2 = z2 + y_shift;
            bounds.inside(&z1) || bounds.inside(&z2) || bounds.crosses(&z1, &z2)
        };

        if let [z1, z2] = &contour[0..=1] {
            if include(z1, z2) {
                coordinates.push(*z1);
            }
        }

        for (z1, z2, z3) in contour.iter().tuple_windows::<(_, _, _)>() {
            if include(z1, z2) || include(z2, z3) {
                coordinates.push(*z2);
            }
        }

        if let [z1, z2] = &contour[(contour.len() - 2)..=(contour.len() - 1)] {
            if include(z1, z2) {
                coordinates.push(*z2);
            }
        }

        coordinates
    }

    pub fn add_plot(&mut self, options: &[&str], contour: &Vec<Complex64>) -> Result<()> {
        self.add_plot_all(options, self.crop(contour))
    }

    pub fn add_plot_all(&mut self, options: &[&str], contour: Vec<Complex64>) -> Result<()> {
        let coordinates = self.format_contour(contour);

        if !coordinates.is_empty() {
            writeln!(
                self.writer,
                "\\addplot [{}] coordinates {{ {} }};",
                options.join(","),
                coordinates.join(" ")
            )?;
            writeln!(self.writer, r#"\directlua{{progress_file:write(".")}}"#)?;
            writeln!(self.writer, r#"\directlua{{progress_file:flush()}}"#)?;
            self.plot_count += 1;
        }
        Ok(())
    }

    pub fn add_plot_custom(&mut self, options: &[&str], plot: &str) -> Result<()> {
        writeln!(self.writer, "\\addplot [{}] {plot};", options.join(","),)?;
        writeln!(self.writer, r#"\directlua{{progress_file:write(".")}}"#)?;
        writeln!(self.writer, r#"\directlua{{progress_file:flush()}}"#)?;
        self.plot_count += 1;
        Ok(())
    }

    pub fn add_grid_line(&mut self, grid_line: &GridLine, options: &[&str]) -> Result<()> {
        self.add_plot(
            &[&["very thin", "lightgray"], options].concat(),
            &grid_line.path,
        )?;

        Ok(())
    }

    pub fn add_grid_lines(&mut self, pxu: &pxu::Pxu, options: &[&str]) -> Result<()> {
        for contour in pxu.contours.get_grid(self.component).iter() {
            self.add_grid_line(contour, options)?;
        }
        if matches!(self.component, pxu::Component::Xp | pxu::Component::Xm) {
            self.add_plot(
                options,
                &vec![Complex64::from(-10.0), Complex64::from(10.0)],
            )?;
        }
        Ok(())
    }

    pub fn add_cut(
        &mut self,
        cut: &pxu::Cut,
        options: &[&str],
        consts: CouplingConstants,
    ) -> Result<()> {
        let straight = "very thick";
        let dashed = "very thick,densely dashed";
        let zigzag = "decorate,decoration={zigzag, segment length=1.2mm, amplitude=0.15mm},thick";
        let (color, style) = match cut.typ {
            pxu::CutType::E => ("black", straight),
            pxu::CutType::Log(pxu::Component::Xp) => ("Red", zigzag),
            pxu::CutType::Log(pxu::Component::Xm) => ("Green", zigzag),
            pxu::CutType::ULongPositive(pxu::Component::Xp) => ("Red", straight),
            pxu::CutType::ULongNegative(pxu::Component::Xp) => ("Red", dashed),
            pxu::CutType::ULongPositive(pxu::Component::Xm) => ("Green", straight),
            pxu::CutType::ULongNegative(pxu::Component::Xm) => ("Green", dashed),
            pxu::CutType::UShortScallion(pxu::Component::Xp) => ("Red", straight),
            pxu::CutType::UShortKidney(pxu::Component::Xp) => ("Red", dashed),
            pxu::CutType::UShortScallion(pxu::Component::Xm) => ("Green", straight),
            pxu::CutType::UShortKidney(pxu::Component::Xm) => ("Green", dashed),
            _ => {
                return Ok(());
            }
        };

        let shifts = if cut.component == pxu::Component::U && cut.periodic {
            let period = 2.0 * consts.k() as f64 / consts.h;
            (-5..=5).map(|n| Some(period * n as f64)).collect()
        } else {
            vec![None]
        };

        let mark_size = if options.contains(&"semithick") {
            "mark size=0.03cm"
        } else {
            "mark size=0.05cm"
        };

        for shift in shifts {
            self.y_shift = shift;

            if style == dashed {
                self.add_plot(&[&["lightgray", "very thick"], options].concat(), &cut.path)?
            }
            self.add_plot(&[&[color, style], options].concat(), &cut.path)?;

            if let Some(branch_point) = cut.branch_point {
                self.add_plot_all(
                    &[&[color, "only marks", mark_size], options].concat(),
                    vec![branch_point],
                )?;
            }
        }

        self.y_shift = None;

        Ok(())
    }

    pub fn add_cuts(&mut self, pxu: &pxu::Pxu, options: &[&str]) -> Result<()> {
        use pxu::{kinematics::UBranch, CutType::*};

        for cut in pxu
            .contours
            .get_visible_cuts(pxu, self.component, 0)
            .filter(|cut| match cut.typ {
                Log(comp) | ULongPositive(comp) => {
                    (comp == pxu::Component::Xp
                        && cut.component == pxu::Component::Xp
                        && pxu.state.points[0].sheet_data.u_branch.1 != UBranch::Between)
                        || (comp == pxu::Component::Xm
                            && cut.component == pxu::Component::Xm
                            && pxu.state.points[0].sheet_data.u_branch.0 != UBranch::Between)
                }
                ULongNegative(_) => false,
                UShortScallion(_) | UShortKidney(_) => true,
                E => true,
                DebugPath => false,
            })
        {
            self.add_cut(cut, options, pxu.consts)?;
        }
        Ok(())
    }

    pub fn add_axis(&mut self) -> Result<()> {
        let options = ["very thin", "black"];
        self.add_plot(
            &options,
            &vec![
                Complex64::new(self.bounds.x_range.start - 1.0, 0.0),
                Complex64::new(self.bounds.x_range.end + 1.0, 0.0),
            ],
        )?;
        self.add_plot(
            &options,
            &vec![
                Complex64::new(0.0, self.bounds.y_range.start - 1.0),
                Complex64::new(0.0, self.bounds.y_range.end + 1.0),
            ],
        )
    }

    pub fn add_path(
        &mut self,
        pxu: &pxu::Pxu,
        path: &pxu::path::Path,
        options: &[&str],
    ) -> Result<()> {
        let mut straight_segments = vec![];
        let mut dotted_segments = vec![];

        let mut same_branch = false;
        let mut points = vec![];

        for segment in &path.segments[0] {
            let segment_same_branch = segment
                .sheet_data
                .is_same(&pxu.state.points[0].sheet_data, self.component);

            if segment_same_branch != same_branch && !points.is_empty() {
                if same_branch {
                    straight_segments.push(points);
                } else {
                    dotted_segments.push(points);
                }
                points = vec![];
            }

            points.extend(segment.get(self.component));
            same_branch = segment_same_branch;
        }

        if same_branch {
            straight_segments.push(points);
        } else {
            dotted_segments.push(points);
        }

        for points in dotted_segments {
            self.add_plot(
                &[&["very thick", "Blue", "dotted"], options].concat(),
                &points,
            )?;
        }

        for points in straight_segments {
            self.add_plot(&[&["very thick", "Blue"], options].concat(), &points)?;
        }

        Ok(())
    }

    pub fn add_path_start_end_mark(
        &mut self,
        path: &pxu::path::Path,
        options: &[&str],
    ) -> Result<()> {
        let first_seg = path.segments[0].first().unwrap();
        let last_seg = path.segments[0].last().unwrap();

        let start = first_seg.get(self.component).first().unwrap();
        let end = last_seg.get(self.component).last().unwrap();

        let points = vec![*start, *end];

        self.add_plot(&[&["only marks"], options].concat(), &points)
    }

    pub fn add_path_arrows(
        &mut self,
        path: &pxu::path::Path,
        mark_pos: &[f64],
        options: &[&str],
    ) -> Result<()> {
        let mut lines: Vec<(Complex64, Complex64, f64)> = vec![];
        let mut len: f64 = 0.0;

        for segment in &path.segments[0] {
            for (p1, p2) in segment.get(self.component).iter().tuple_windows() {
                len += (p2 - p1).norm();
                lines.push((*p1, *p2, len)); // We store the length including the current segment
            }
        }

        let total_len = len;

        for pos in mark_pos {
            let pos = pos * total_len;
            let index = lines.partition_point(|(_, _, seg_end)| seg_end < &pos);
            if index == lines.len() {
                continue;
            }
            let (start, end, seg_end) = lines[index];
            let t = 1.0 - (seg_end - pos) / (end - start).norm();
            let points = vec![start, end];

            self.add_plot(
                &[
                    &[
                        "draw=none",
                        &format!(
                            "decoration={{markings,mark=at position {t} with {{\\arrow{{latex}}}}}}"
                        ),
                        "postaction=decorate",
                    ],
                    options,
                ]
                .concat(),
                &points,
            )?;
        }

        Ok(())
    }

    pub fn add_node(&mut self, text: &str, pos: Complex64, options: &[&str]) -> Result<()> {
        let coord = self.format_coordinate(pos);
        writeln!(
            self.writer,
            "\\node at {coord} [{}] {{{text}}};",
            options.join(",")
        )
    }

    pub fn draw(&mut self, path: &str, options: &[&str]) -> Result<()> {
        writeln!(self.writer, "\\draw [{}] {path};", options.join(","))
    }

    pub fn add_point(&mut self, point: &pxu::Point, options: &[&str]) -> Result<()> {
        let points = vec![point.get(self.component)];
        self.add_plot_all(&[&["only marks"], options].concat(), points)?;
        Ok(())
    }

    pub fn add_state(&mut self, state: &pxu::State, options: &[&str]) -> Result<()> {
        let points = state
            .points
            .iter()
            .map(|pt| pt.get(self.component))
            .collect::<Vec<_>>();
        self.add_plot_all(&[&["only marks"], options].concat(), points)?;
        Ok(())
    }

    pub fn finish(
        mut self,
        cache: Arc<cache::Cache>,
        settings: &Settings,
        pb: &ProgressBar,
    ) -> std::io::Result<FigureCompiler> {
        writeln!(self.writer, "\\end{{axis}}\n")?;

        if !self.no_component_indicator {
            let component_name = match self.component {
                pxu::Component::P => "p",
                pxu::Component::Xp => "x^+",
                pxu::Component::Xm => "x^-",
                pxu::Component::U => "u",
            };
            writeln!(
                self.writer,
                "\\node at (current bounding box.north east) [anchor=north east,fill=white,outer sep=0.1cm,draw,thin] {{$\\scriptstyle {component_name}$}};"
            )?;
        }

        self.writer.write_all(Self::FILE_END.as_bytes())?;
        self.writer.flush()?;

        pb.set_message(format!("Compiling {}.tex", self.name));
        FigureCompiler::new(self, cache, settings)
    }

    fn transform_vec(&self, v: Complex64) -> Complex64 {
        Complex64::new(
            v.re * self.size.width / self.bounds.width(),
            v.im * self.size.height / self.bounds.height(),
        )
    }

    pub fn set_caption(&mut self, caption: &str) {
        self.caption = caption.to_owned();
    }
}

pub trait Node {
    fn write_m_node(
        &mut self,
        figure: &mut FigureWriter,
        anchor: &str,
        rot_sign: i32,
        consts: CouplingConstants,
    ) -> Result<()>;
}

impl Node for PInterpolatorMut {
    fn write_m_node(
        &mut self,
        figure: &mut FigureWriter,
        anchor: &str,
        rot_sign: i32,
        consts: CouplingConstants,
    ) -> Result<()> {
        let p = match self.pt() {
            InterpolationPoint::Xp(p, _) | InterpolationPoint::Xm(p, _) => p,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Expected xp or xm, found {:?}", self.pt()),
                ));
            }
        };

        let mut p_int2 = self.clone();
        p_int2.goto_p(p - 0.001);
        let p1 = p_int2.p();
        p_int2.goto_p(p + 0.001);
        let p2 = p_int2.p();
        let dp = figure.transform_vec(p2 - p1);
        let rotation = dp.im.atan2(dp.re) * 180.0 / std::f64::consts::PI
            + if rot_sign >= 0 { 0.0 } else { 180.0 };

        let (color, m) = match self.pt().normalized(consts) {
            InterpolationPoint::Xp(_, m) => ("black", m),
            InterpolationPoint::Xm(_, m) => ("blue", m),
            _ => unreachable!(),
        };

        writeln!(figure.writer,"\\node[scale=0.5,anchor={anchor},inner sep=0.4pt,rotate={rotation:.1},{color}] at ({:.3}, {:.3}) {{$\\scriptstyle {}$}};",
                 self.p().re,
                 self.p().im,
                 m)
    }
}
