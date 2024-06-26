use crate::cache;
use crate::fig_compiler::FigureCompiler;
use crate::fig_writer::FigureWriter;
use crate::utils::{error, Settings, Size};
use indicatif::ProgressBar;

use itertools::izip;
use make_paths::PxuProvider;
use num::complex::Complex64;
use num::Zero;
use pxu::{interpolation::PInterpolatorMut, kinematics::UBranch};
use pxu::{Component, CouplingConstants, Cut, CutType, GridLineComponent};
use std::io::Result;
use std::sync::Arc;

fn load_state(s: &str) -> Result<pxu::State> {
    ron::from_str(s).map_err(|_| error("Could not load state"))
}

fn load_states(state_strings: &[&str]) -> Result<Vec<pxu::State>> {
    state_strings
        .iter()
        .map(|s| load_state(s))
        .collect::<Result<Vec<_>>>()
}

const PREIMAGE_STRING: &str = include_str!("../data/preimage-data.ron");

// TODO:
// - physical u plane for various p
// - b.s. with p > 2pi in  the p plane?

fn draw_xl_preimage(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
    x_component: Component,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pt = pxu::Point::new(0.5, consts);

    #[allow(clippy::type_complexity)]
    let preimage_data: Vec<(Complex64, Complex64, (i32, f64), (i32, f64))> =
        ron::from_str(PREIMAGE_STRING).unwrap();

    let name = if x_component == Component::Xp {
        "p-xpL-preimage"
    } else {
        "p-xmL-preimage"
    };
    let mut figure = FigureWriter::new(
        name,
        -1.9..1.9,
        0.0,
        Size {
            width: 20.0,
            height: 6.5,
        },
        Component::P,
        settings,
        pb,
    )?;

    let contours = pxu_provider.get_contours(consts)?.clone();

    figure.add_grid_lines(&contours, &[])?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::P, consts)
        .filter(|cut| matches!(cut.typ, CutType::E))
    {
        figure.add_cut(cut, &[], consts)?;
    }

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::P, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::E
                    | CutType::UShortScallion(_)
                    | CutType::UShortKidney(_)
                    | CutType::Log(_)
                    | CutType::ULongPositive(_)
            )
        })
    {
        let options: &[&str] = match cut.typ {
            CutType::Log(Component::Xp) => &["Red!50!white", "very thick"],
            CutType::Log(Component::Xm) => &["Green!50!white", "very thick"],
            CutType::ULongPositive(Component::Xp) => &[
                "Red!50!white",
                "decorate,decoration={coil,aspect=0, segment length=2.4mm, amplitude=0.15mm}",
                "very thick",
            ],
            CutType::ULongPositive(Component::Xm) => &[
                "Green!50!white",
                "decorate,decoration={coil,aspect=0, segment length=2.4mm, amplitude=0.15mm}",
                "very thick",
            ],
            _ => &[],
        };
        figure.add_cut(cut, options, consts)?;
    }

    for (z, dz, (xp_sign, xp_m), (xm_sign, xm_m)) in preimage_data {
        let (sign, m) = match x_component {
            Component::Xp => (xp_sign, xp_m),
            Component::Xm => (xm_sign, xm_m),
            _ => panic!("Expected xp or xm"),
        };
        let m = m.round() as i32;
        if m % consts.k() == 0 && dz.im.abs() - dz.re.abs() > 0.0 {
            continue;
        }
        let dp = figure.transform_vec(dz);
        let rotation = dp.im.atan2(dp.re) * 180.0 / std::f64::consts::PI
            + if dz.re.abs() - dz.im.abs() > 0.0 {
                if dz.re > 0.0 {
                    0.0
                } else {
                    180.0
                }
            } else if dz.im > 0.0 {
                180.0
            } else {
                0.0
            };

        let color = if sign > 0 { "Black" } else { "Blue" };
        figure.add_node(
            &format!("$\\scriptscriptstyle{m}$"),
            z,
            &[
                color,
                "anchor=south",
                "inner sep=0.5mm",
                &format!("rotate={rotation:.1}"),
            ],
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_xpl_preimage(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    draw_xl_preimage(pxu_provider, cache, settings, pb, Component::Xp)
}

fn fig_p_xml_preimage(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    draw_xl_preimage(pxu_provider, cache, settings, pb, Component::Xm)
}

fn fig_p_plane_e_cuts(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let mut figure = FigureWriter::new(
        "p-plane-e-cuts",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let contours = pxu_provider.get_contours(consts)?.clone();

    figure.add_grid_lines(&contours, &[])?;

    let pt = pxu::Point::new(0.5, consts);

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::P, consts)
        .filter(|cut| matches!(cut.typ, CutType::E))
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.add_plot(&["black"], &[Complex64::from(-5.0), Complex64::from(5.0)])?;

    figure.add_plot(
        &["black"],
        &[Complex64::new(0.0, -5.0), Complex64::new(0.0, 5.0)],
    )?;

    for i in 0..=(2 * 5) {
        let x = -5.0 + i as f64;
        figure.add_plot(
            &["black"],
            &[Complex64::new(x, -0.03), Complex64::new(x, 0.03)],
        )?;
        figure.add_plot(
            &["black"],
            &[
                Complex64::new(x + 0.25, -0.015),
                Complex64::new(x + 0.25, 0.015),
            ],
        )?;
        figure.add_plot(
            &["black"],
            &[
                Complex64::new(x + 0.5, -0.015),
                Complex64::new(x + 0.5, 0.015),
            ],
        )?;
        figure.add_plot(
            &["black"],
            &[
                Complex64::new(x + 0.75, -0.015),
                Complex64::new(x + 0.75, 0.015),
            ],
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_scallion_and_kidney(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?.clone();

    let mut figure = FigureWriter::new(
        "scallion-and-kidney",
        -3.1..3.1,
        0.0,
        Size {
            width: 4.5,
            height: 4.5,
        },
        Component::Xp,
        settings,
        pb,
    )?;
    let pt = pxu::Point::new(0.5, consts);

    figure.no_component_indicator();
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        let mut cut = cut.clone();
        cut.branch_point = None;
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.add_node(
        "\\footnotesize Scallion",
        Complex64::new(1.5, -2.0),
        &["anchor=west"],
    )?;
    figure.add_node(
        "\\footnotesize Kidney",
        Complex64::new(-1.25, 0.5),
        &["anchor=east"],
    )?;
    figure.draw("(1.5,-2.0) to[out=180,in=-45] (0.68,-1.53)", &["->"])?;
    figure.draw("(-1.25,0.5) to[out=0,in=130] (-0.75,0.3)", &["->"])?;

    figure.finish(cache, settings, pb)
}

fn fig_scallion_and_kidney_7_10(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(1.0, 7);
    let contours = pxu_provider.get_contours(consts)?.clone();

    let mut figure = FigureWriter::new(
        "scallion-and-kidney-7-10",
        -6.2..6.2,
        0.0,
        Size {
            width: 4.5,
            height: 4.5,
        },
        Component::Xp,
        settings,
        pb,
    )?;
    let pt = pxu::Point::new(0.5, consts);

    figure.no_component_indicator();
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        let mut cut = cut.clone();
        cut.branch_point = None;
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_scallion_and_kidney_3_70(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(7.0, 3);
    let contours = pxu_provider.get_contours(consts)?.clone();

    let mut figure = FigureWriter::new(
        "scallion-and-kidney-3-70",
        -2.7..2.7,
        0.0,
        Size {
            width: 4.5,
            height: 4.5,
        },
        Component::Xp,
        settings,
        pb,
    )?;
    let pt = pxu::Point::new(0.5, consts);

    figure.no_component_indicator();
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        let mut cut = cut.clone();
        cut.branch_point = None;
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_scallion_and_kidney_r(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?.clone();

    let mut figure = FigureWriter::new(
        "scallion-and-kidney-R",
        -3.1..3.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;
    let state_string = r"(points:[(p:(-0.2498413622379303,0.000009991228580474854),xp:(-0.6478279611895327,0.6471633470693878),xm:(-0.6478494168942528,-0.6472232084111232),u:(-1.3503465619270798,-2.5000545006090906),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)";
    let state = load_state(state_string)?;
    let pt = &state.points[0];

    figure.set_r();
    figure.component_indicator(r"x_{\mbox{\tiny R}}");

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    for cut in contours
        .get_visible_cuts_from_point(pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp)
                    | CutType::UShortScallion(Component::Xp)
                    | CutType::Log(Component::Xp)
            )
        })
    {
        let mut cut = cut.clone();
        if matches!(cut.typ, CutType::Log(_)) {
            cut.path = cut.path.into_iter().map(|z| -z).collect();
        }
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    let points = vec![pt.get(Component::Xp), pt.get(Component::Xm)];

    figure.add_plot_all(&["only marks", "Blue", "mark size=0.05cm"], points)?;
    figure.add_node(
        r"$\scriptstyle x_{\mbox{\tiny R}}^+$",
        pt.get(Component::Xp),
        &["anchor=west"],
    )?;
    figure.add_node(
        r"$\scriptstyle x_{\mbox{\tiny R}}^-$",
        pt.get(Component::Xm),
        &["anchor=west"],
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_u_plane_between_between_r(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?.clone();

    let mut figure = FigureWriter::new(
        "u-plane-between-between-R",
        -5.125..5.125,
        -consts.k() as f64 / consts.h,
        Size {
            width: 5.5,
            height: 8.0,
        },
        Component::U,
        settings,
        pb,
    )?;
    let state_string = r"(points:[(p:(-0.2498413622379303,0.000009991228580474854),xp:(-0.6478279611895327,0.6471633470693878),xm:(-0.6478494168942528,-0.6472232084111232),u:(-1.3503465619270798,-2.5000545006090906),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)";
    let state = load_state(state_string)?;
    let pt = &state.points[0];
    figure.set_r();

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis_origin(Complex64::new(0.0, -consts.k() as f64 / consts.h))?;
    figure.component_indicator(r"u_{\mbox{\tiny R}}");

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(_) | CutType::UShortScallion(_) | CutType::E
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.add_state(&state, &["Blue", "mark size=0.05cm"])?;
    figure.add_node(
        r"$\scriptstyle u_{\mbox{\tiny R}}$",
        pt.get(Component::U) + Complex64::new(-0.1, 0.38 / consts.h),
        &["anchor=east"],
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_p_plane_short_cuts_r(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "p-plane-short-cuts-R",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;
    let state_string = r"(points:[(p:(-0.2498413622379303,0.000009991228580474854),xp:(-0.6478279611895327,0.6471633470693878),xm:(-0.6478494168942528,-0.6472232084111232),u:(-1.3503465619270798,-2.5000545006090906),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)";
    let state = load_state(state_string)?;
    let pt = &state.points[0];

    figure.set_r();
    figure.component_indicator(r"p_{\mbox{\tiny R}}");

    figure.add_grid_lines(&contours, &[])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, Component::P, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::E
                    | CutType::Log(_)
                    | CutType::UShortKidney(_)
                    | CutType::UShortScallion(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.add_state(&state, &["Blue", "mark size=0.05cm"])?;
    figure.add_node(
        r"$\scriptstyle p_{\mbox{\tiny R}}$",
        pt.get(Component::P),
        &["anchor=north"],
    )?;

    figure.finish(cache, settings, pb)
}

fn get_cut_path(
    contours: &pxu::Contours,
    pt: &pxu::Point,
    component: Component,
    consts: CouplingConstants,
    cut_type: CutType,
) -> Vec<Complex64> {
    let cut_paths = contours
        .get_visible_cuts_from_point(pt, component, consts)
        .filter(|cut| cut.typ == cut_type)
        .collect::<Vec<_>>();

    let mut path = cut_paths[0]
        .path
        .clone()
        .into_iter()
        .filter(|x| x.im < 0.0 && x.re > -3.5)
        .collect::<Vec<_>>();

    if path.first().unwrap().re > path.last().unwrap().re {
        path.reverse();
    }

    path
}

fn fig_x_integration_contour_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "x-integration-contour-1",
        -3.1..2.6,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let s = Complex64::from(consts.s());

    let mut bottom_scallion_path = get_cut_path(
        &contours,
        &pt,
        Component::Xp,
        consts,
        CutType::UShortScallion(Component::Xp),
    );
    bottom_scallion_path.push(s);
    bottom_scallion_path.reverse();

    let top_scallion_path = bottom_scallion_path
        .iter()
        .map(|z| z.conj())
        .collect::<Vec<_>>();

    let mut bottom_kidney_path = vec![-1.0 / s];
    bottom_kidney_path.extend(get_cut_path(
        &contours,
        &pt,
        Component::Xp,
        consts,
        CutType::UShortKidney(Component::Xp),
    ));
    bottom_kidney_path.reverse();

    let kidney_bottom = *bottom_kidney_path
        .iter()
        .min_by(|&z1, &z2| z1.im.partial_cmp(&z2.im).unwrap())
        .unwrap();
    let kidney_top = kidney_bottom.conj();

    let top_kidney_path = bottom_kidney_path
        .iter()
        .map(|z| z.conj())
        .collect::<Vec<_>>();

    let dy = Complex64::new(0.0, 0.03);
    let log_path_1t = vec![-3.1 + dy, -1.0 / s + dy];
    let log_path_1b = vec![-3.1 - dy, -1.0 / s + -dy];
    let log_path_2t = vec![-1.0 / s + dy, dy];
    let log_path_2b = vec![-1.0 / s + -dy, -dy];

    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.3 with {\arrow{latex}}}",
            r"decoration={markings,mark=at position 0.8 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &top_scallion_path,
    )?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.3 with {\arrow{latex}}}",
            r"decoration={markings,mark=at position 0.8 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &bottom_scallion_path,
    )?;
    figure.add_plot(&["Black", "thick"], &top_kidney_path)?;
    figure.add_plot(&["Black", "thick"], &bottom_kidney_path)?;
    figure.add_plot(
        &["White", "thick"],
        &[Complex64::from(-3.1), Complex64::zero()],
    )?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.6 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &log_path_1t,
    )?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.6 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &log_path_1b,
    )?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.8 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &log_path_2t,
    )?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.8 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &log_path_2b,
    )?;
    figure.add_plot(
        &["Black", "thick", "only marks", "mark size=0.04cm"],
        &[-1.0 / s, Complex64::zero(), s],
    )?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 1.0 with {\arrow{latex}}}",
            "postaction=decorate",
            "draw=none",
        ],
        &[kidney_bottom + 0.1, kidney_bottom - 0.15],
    )?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 1.0 with {\arrow{latex}}}",
            "postaction=decorate",
            "draw=none",
        ],
        &[kidney_top + 0.1, kidney_top - 0.15],
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_x_integration_contour_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "x-integration-contour-2",
        -3.1..2.6,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let s = Complex64::from(consts.s());

    let dy = Complex64::new(0.0, 0.03);
    let path_t = vec![s + dy, -1.0 / s + dy];
    let path_b = vec![s - dy, -1.0 / s - dy];

    figure.add_plot(&["White", "thick"], &[-1.0 / s, s])?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.6 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &path_t,
    )?;
    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.6 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &path_b,
    )?;
    figure.add_plot(
        &["Black", "thick", "only marks", "mark size=0.04cm"],
        &[-1.0 / s, s],
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_x_integration_contour_rr_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 0);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "x-integration-contour-RR-2",
        -2.2..2.2,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let dy = Complex64::new(0.0, 0.03);
    let path_t = vec![1.0 + dy, -1.0 + dy];
    let path_b = vec![1.0 - dy, -1.0 - dy];

    figure.add_plot(
        &["White", "thick"],
        &[Complex64::from(-1.0), Complex64::from(1.0)],
    )?;

    figure.add_plot(
        &["Black", "thick", "only marks", "mark size=0.04cm"],
        &[Complex64::from(1.0), Complex64::from(-1.0)],
    )?;

    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.3 with {\arrow{latex}}}",
            r"decoration={markings,mark=at position 0.8 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &path_t,
    )?;

    figure.add_plot(
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.3 with {\arrow{latex}}}",
            r"decoration={markings,mark=at position 0.8 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
        &path_b,
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_x_integration_contour_rr_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 0);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "x-integration-contour-RR-1",
        -2.2..2.2,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    figure.add_plot(
        &["Black", "thick", "only marks", "mark size=0.04cm"],
        &[Complex64::from(1.0), Complex64::from(-1.0)],
    )?;

    figure.draw(
        "(1,0) arc (0:180:1.0)",
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.3 with {\arrow{latex}}}",
            r"decoration={markings,mark=at position 0.8 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
    )?;

    figure.draw(
        "(1,0) arc (0:-180:1.0)",
        &[
            "Black",
            "thick",
            r"decoration={markings,mark=at position 0.3 with {\arrow{latex}}}",
            r"decoration={markings,mark=at position 0.8 with {\arrow{latex}}}",
            "postaction=decorate",
        ],
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_x_regions_outside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "x-regions-outside",
        -3.1..3.1,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let scallion_path = get_cut_path(
        &contours,
        &pt,
        Component::Xp,
        consts,
        CutType::UShortScallion(Component::Xp),
    );

    let (scallion_left, scallion_right) = scallion_path
        .split_at(scallion_path.partition_point(|x| pxu::kinematics::u_of_x(*x, consts).re < 0.0));

    let mut vertical_path: Vec<Complex64> = vec![];
    for segment in pxu_provider.get_path("u vertical outside")?.segments[0].iter() {
        vertical_path.extend(&segment.xp);
    }

    let mut q4_path = vec![consts.s().into()];

    q4_path.extend(scallion_right);
    q4_path.extend([
        Complex64::from(consts.s()),
        Complex64::from(4.0),
        Complex64::new(4.0, vertical_path.last().unwrap().im),
    ]);
    q4_path.extend(vertical_path.iter().rev());

    let mut q3_path = vec![Complex64::from(-4.0)];

    q3_path.extend(scallion_left);
    q3_path.extend(&vertical_path);
    q3_path.extend([
        Complex64::new(-4.0, vertical_path.last().unwrap().im),
        Complex64::from(-4.0),
        Complex64::from(-1.0 / consts.s()),
    ]);

    let q1_path = q4_path.iter().map(|z| z.conj()).collect::<Vec<_>>();
    let q2_path = q3_path.iter().map(|z| z.conj()).collect::<Vec<_>>();

    figure.add_plot(&["fill=yellow", "fill opacity=0.25", "draw=none"], &q1_path)?;
    figure.add_plot(&["fill=blue", "fill opacity=0.25", "draw=none"], &q2_path)?;
    figure.add_plot(&["fill=red", "fill opacity=0.25", "draw=none"], &q3_path)?;
    figure.add_plot(&["fill=green", "fill opacity=0.25", "draw=none"], &q4_path)?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp)
                    | CutType::UShortScallion(Component::Xp)
                    | CutType::Log(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_regions_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "x-regions-between",
        -3.1..3.1,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let scallion_path = get_cut_path(
        &contours,
        &pt,
        Component::Xp,
        consts,
        CutType::UShortScallion(Component::Xp),
    );

    let kidney_path = get_cut_path(
        &contours,
        &pt,
        Component::Xp,
        consts,
        CutType::UShortKidney(Component::Xp),
    );

    let (scallion_left, scallion_right) = scallion_path
        .split_at(scallion_path.partition_point(|x| pxu::kinematics::u_of_x(*x, consts).re < 0.0));

    let (kidney_left, kidney_right) = kidney_path
        .split_at(kidney_path.partition_point(|x| pxu::kinematics::u_of_x(*x, consts).re < 0.0));

    let mut vertical_path = vec![];
    for segment in pxu_provider.get_path("u vertical between")?.segments[0].iter() {
        vertical_path.extend(&segment.xp);
    }

    let mut q4_path = vec![*kidney_right.last().unwrap(), consts.s().into()];

    q4_path.extend(scallion_right.iter().rev());
    q4_path.extend(&vertical_path);
    q4_path.extend(kidney_right);

    let mut q3_path = vec![Complex64::from(-1.0 / consts.s()), Complex64::from(-4.0)];

    q3_path.extend(scallion_left);
    q3_path.extend(&vertical_path);
    q3_path.extend(kidney_left.iter().rev());

    let q1_path = q4_path.iter().map(|z| z.conj()).collect::<Vec<_>>();
    let q2_path = q3_path.iter().map(|z| z.conj()).collect::<Vec<_>>();

    figure.add_plot(&["fill=yellow", "fill opacity=0.25", "draw=none"], &q1_path)?;
    figure.add_plot(&["fill=blue", "fill opacity=0.25", "draw=none"], &q2_path)?;
    figure.add_plot(&["fill=red", "fill opacity=0.25", "draw=none"], &q3_path)?;
    figure.add_plot(&["fill=green", "fill opacity=0.25", "draw=none"], &q4_path)?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp)
                    | CutType::UShortScallion(Component::Xp)
                    | CutType::Log(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_regions_inside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "x-regions-inside",
        -1.1..1.1,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let kidney_path = get_cut_path(
        &contours,
        &pt,
        Component::Xp,
        consts,
        CutType::UShortKidney(Component::Xp),
    );

    let (kidney_left, kidney_right) = kidney_path
        .split_at(kidney_path.partition_point(|x| pxu::kinematics::u_of_x(*x, consts).re < 0.0));

    let mut vertical_path = vec![];
    for segment in pxu_provider.get_path("u vertical inside")?.segments[0].iter() {
        vertical_path.extend(&segment.xp);
    }

    let mut q4_path = vec![Complex64::zero()];

    q4_path.extend(kidney_right.iter().rev());
    q4_path.extend(&vertical_path);

    let mut q3_path = vec![Complex64::zero(), Complex64::from(-1.0 / consts.s())];

    q3_path.extend(kidney_left);
    q3_path.extend(&vertical_path);

    let q1_path = q4_path.iter().map(|z| z.conj()).collect::<Vec<_>>();
    let q2_path = q3_path.iter().map(|z| z.conj()).collect::<Vec<_>>();

    figure.add_plot(&["fill=yellow", "fill opacity=0.25", "draw=none"], &q1_path)?;
    figure.add_plot(&["fill=blue", "fill opacity=0.25", "draw=none"], &q2_path)?;
    figure.add_plot(&["fill=red", "fill opacity=0.25", "draw=none"], &q3_path)?;
    figure.add_plot(&["fill=green", "fill opacity=0.25", "draw=none"], &q4_path)?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp)
                    | CutType::UShortScallion(Component::Xp)
                    | CutType::Log(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_regions_long(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "x-regions-long",
        -3.1..3.1,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let mut vertical_path: Vec<Complex64> = vec![];

    for segment in pxu_provider.get_path("u vertical inside")?.segments[0]
        .iter()
        .rev()
    {
        vertical_path.extend(segment.xp.iter().rev());
    }

    for segment in pxu_provider.get_path("u vertical between")?.segments[0]
        .iter()
        .rev()
    {
        vertical_path.extend(segment.xp.iter().rev());
    }

    for segment in pxu_provider.get_path("u vertical outside")?.segments[0].iter() {
        vertical_path.extend(&segment.xp);
    }

    let mut q4_path = vec![Complex64::from(0.0)];

    q4_path.extend(&vertical_path);
    q4_path.extend([
        Complex64::new(4.0, vertical_path.last().unwrap().im),
        Complex64::from(4.0),
        Complex64::zero(),
    ]);

    let mut q3_path = vec![Complex64::from(0.0)];

    q3_path.extend(&vertical_path);
    q3_path.extend([
        Complex64::new(-4.0, vertical_path.last().unwrap().im),
        Complex64::from(-4.0),
        Complex64::zero(),
    ]);

    let q1_path = q4_path.iter().map(|z| z.conj()).collect::<Vec<_>>();
    let q2_path = q3_path.iter().map(|z| z.conj()).collect::<Vec<_>>();

    figure.add_plot(&["fill=yellow", "fill opacity=0.25", "draw=none"], &q1_path)?;
    figure.add_plot(&["fill=blue", "fill opacity=0.25", "draw=none"], &q2_path)?;
    figure.add_plot(&["fill=red", "fill opacity=0.25", "draw=none"], &q3_path)?;
    figure.add_plot(&["fill=green", "fill opacity=0.25", "draw=none"], &q4_path)?;

    let s = consts.s();
    let cuts = vec![
        Cut::new(
            Component::Xp,
            vec![Complex64::from(-10.0), Complex64::from(-1.0 / s)],
            Some(Complex64::from(-1.0 / s)),
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::Xp,
            vec![Complex64::from(-1.0 / s), Complex64::zero()],
            Some(Complex64::zero()),
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::Xp,
            vec![Complex64::zero(), Complex64::from(10.0)],
            Some(Complex64::from(s)),
            CutType::ULongPositive(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_outside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-outside",
        -7.25..7.25,
        -0.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Outside,
        ::pxu::kinematics::UBranch::Outside,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis_origin(Complex64::new(0.0, -0.5))?;

    figure.add_plot(
        &["fill=green", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -0.5),
            Complex64::new(20.0, -0.5),
            Complex64::new(20.0, -20.0),
            Complex64::new(0.0, -20.0),
        ],
    )?;

    figure.add_plot(
        &["fill=red", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -0.5),
            Complex64::new(-20.0, -0.5),
            Complex64::new(-20.0, -20.0),
            Complex64::new(0.0, -20.0),
        ],
    )?;

    figure.add_plot(
        &["fill=yellow", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -0.5),
            Complex64::new(20.0, -0.5),
            Complex64::new(20.0, 20.0),
            Complex64::new(0.0, 20.0),
        ],
    )?;

    figure.add_plot(
        &["fill=blue", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -0.5),
            Complex64::new(-20.0, -0.5),
            Complex64::new(-20.0, 20.0),
            Complex64::new(0.0, 20.0),
        ],
    )?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::U, consts)
        .filter(|cut| matches!(cut.typ, CutType::UShortScallion(Component::Xp)))
    {
        figure.add_cut(cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-between",
        -7.25..7.25,
        -0.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis_origin(Complex64::new(0.0, -0.5))?;

    for i in -2..=3 {
        let shift = Complex64::new(0.0, i as f64 * consts.k() as f64);

        figure.add_plot(
            &["fill=yellow", "fill opacity=0.25", "draw=none"],
            &[
                Complex64::new(0.0, -0.5) + shift,
                Complex64::new(20.0, -0.5) + shift,
                Complex64::new(20.0, -3.0) + shift,
                Complex64::new(0.0, -3.0) + shift,
            ],
        )?;

        figure.add_plot(
            &["fill=blue", "fill opacity=0.25", "draw=none"],
            &[
                Complex64::new(0.0, -0.5) + shift,
                Complex64::new(-20.0, -0.5) + shift,
                Complex64::new(-20.0, -3.0) + shift,
                Complex64::new(0.0, -3.0) + shift,
            ],
        )?;

        figure.add_plot(
            &["fill=green", "fill opacity=0.25", "draw=none"],
            &[
                Complex64::new(0.0, -0.5) + shift,
                Complex64::new(20.0, -0.5) + shift,
                Complex64::new(20.0, 2.0) + shift,
                Complex64::new(0.0, 2.0) + shift,
            ],
        )?;

        figure.add_plot(
            &["fill=red", "fill opacity=0.25", "draw=none"],
            &[
                Complex64::new(0.0, -0.5) + shift,
                Complex64::new(-20.0, -0.5) + shift,
                Complex64::new(-20.0, 2.0) + shift,
                Complex64::new(0.0, 2.0) + shift,
            ],
        )?;
    }

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::U, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_inside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-inside",
        -7.25..7.25,
        -0.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Inside,
        ::pxu::kinematics::UBranch::Inside,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis_origin(Complex64::new(0.0, -0.5))?;

    figure.add_plot(
        &["fill=yellow", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -3.0),
            Complex64::new(20.0, -3.0),
            Complex64::new(20.0, -20.0),
            Complex64::new(0.0, -20.0),
        ],
    )?;

    figure.add_plot(
        &["fill=blue", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -3.0),
            Complex64::new(-20.0, -3.0),
            Complex64::new(-20.0, -20.0),
            Complex64::new(0.0, -20.0),
        ],
    )?;

    figure.add_plot(
        &["fill=green", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -3.0),
            Complex64::new(20.0, -3.0),
            Complex64::new(20.0, 20.0),
            Complex64::new(0.0, 20.0),
        ],
    )?;

    figure.add_plot(
        &["fill=red", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -3.0),
            Complex64::new(-20.0, -3.0),
            Complex64::new(-20.0, 20.0),
            Complex64::new(0.0, 20.0),
        ],
    )?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::U, consts)
        .filter(|cut| matches!(cut.typ, CutType::UShortKidney(Component::Xp)))
    {
        figure.add_cut(cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_between_small(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-between-small",
        -7.25..7.25,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis()?;

    figure.add_plot(
        &["fill=yellow", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 0.0),
            Complex64::new(20.0, 0.0),
            Complex64::new(20.0, -2.5),
            Complex64::new(0.0, -2.5),
        ],
    )?;

    figure.add_plot(
        &["fill=blue", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 0.0),
            Complex64::new(-20.0, 0.0),
            Complex64::new(-20.0, -2.5),
            Complex64::new(0.0, -2.5),
        ],
    )?;

    figure.add_plot(
        &["fill=green", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 0.0),
            Complex64::new(20.0, 0.0),
            Complex64::new(20.0, 2.5),
            Complex64::new(0.0, 2.5),
        ],
    )?;

    figure.add_plot(
        &["fill=red", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 0.0),
            Complex64::new(-20.0, 0.0),
            Complex64::new(-20.0, 2.5),
            Complex64::new(0.0, 2.5),
        ],
    )?;

    let us = pxu::kinematics::u_of_x(consts.s(), consts);
    let ikh = Complex64::new(0.0, consts.k() as f64 / consts.h);
    let cuts = [
        Cut::new(
            Component::U,
            vec![us, Complex64::from(-20.0)],
            Some(us),
            CutType::UShortScallion(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us + ikh, Complex64::from(20.0) + ikh],
            Some(-us + ikh),
            CutType::UShortKidney(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us - ikh, Complex64::from(20.0) - ikh],
            Some(-us - ikh),
            CutType::UShortKidney(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us + ikh, Complex64::from(-20.0) + ikh],
            None,
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us - ikh, Complex64::from(-20.0) - ikh],
            None,
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_inside_small_upper(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-inside-small-upper",
        -7.25..7.25,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis()?;

    figure.add_plot(
        &["fill=green", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 20.0),
            Complex64::new(20.0, 20.0),
            Complex64::new(20.0, 2.5),
            Complex64::new(0.0, 2.5),
        ],
    )?;

    figure.add_plot(
        &["fill=red", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 20.0),
            Complex64::new(-20.0, 20.0),
            Complex64::new(-20.0, 2.5),
            Complex64::new(0.0, 2.5),
        ],
    )?;

    let us = pxu::kinematics::u_of_x(consts.s(), consts);
    let ikh = Complex64::new(0.0, consts.k() as f64 / consts.h);
    let cuts = [
        Cut::new(
            Component::U,
            vec![-us + ikh, Complex64::from(20.0) + ikh],
            Some(-us + ikh),
            CutType::UShortKidney(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us + ikh, Complex64::from(-20.0) + ikh],
            None,
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_inside_small_lower(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-inside-small-lower",
        -7.25..7.25,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis()?;

    figure.add_plot(
        &["fill=yellow", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -20.0),
            Complex64::new(20.0, -20.0),
            Complex64::new(20.0, -2.5),
            Complex64::new(0.0, -2.5),
        ],
    )?;

    figure.add_plot(
        &["fill=blue", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -20.0),
            Complex64::new(-20.0, -20.0),
            Complex64::new(-20.0, -2.5),
            Complex64::new(0.0, -2.5),
        ],
    )?;

    let us = pxu::kinematics::u_of_x(consts.s(), consts);
    let ikh = Complex64::new(0.0, consts.k() as f64 / consts.h);
    let cuts = [
        Cut::new(
            Component::U,
            vec![-us - ikh, Complex64::from(20.0) - ikh],
            Some(-us - ikh),
            CutType::UShortKidney(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us - ikh, Complex64::from(-20.0) - ikh],
            None,
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_inside_small(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-inside-small",
        -7.25..7.25,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis()?;

    figure.add_plot(
        &["fill=green", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 20.0),
            Complex64::new(20.0, 20.0),
            Complex64::new(20.0, 2.5),
            Complex64::new(0.0, 2.5),
        ],
    )?;

    figure.add_plot(
        &["fill=red", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 20.0),
            Complex64::new(-20.0, 20.0),
            Complex64::new(-20.0, 2.5),
            Complex64::new(0.0, 2.5),
        ],
    )?;

    figure.add_plot(
        &["fill=yellow", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -20.0),
            Complex64::new(20.0, -20.0),
            Complex64::new(20.0, -2.5),
            Complex64::new(0.0, -2.5),
        ],
    )?;

    figure.add_plot(
        &["fill=blue", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, -20.0),
            Complex64::new(-20.0, -20.0),
            Complex64::new(-20.0, -2.5),
            Complex64::new(0.0, -2.5),
        ],
    )?;

    let us = pxu::kinematics::u_of_x(consts.s(), consts);
    let ikh = Complex64::new(0.0, consts.k() as f64 / consts.h);
    let cuts = [
        Cut::new(
            Component::U,
            vec![-us + ikh, Complex64::from(20.0) + ikh],
            Some(-us + ikh),
            CutType::UShortKidney(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us + ikh, Complex64::from(-20.0) + ikh],
            None,
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us - ikh, Complex64::from(20.0) - ikh],
            Some(-us - ikh),
            CutType::UShortKidney(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us - ikh, Complex64::from(-20.0) - ikh],
            None,
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_long_upper(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-between-long-upper",
        -7.25..7.25,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis()?;

    figure.add_plot(
        &["fill=yellow", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 20.0),
            Complex64::new(20.0, 20.0),
            Complex64::new(20.0, -20.0),
            Complex64::new(0.0, -20.0),
        ],
    )?;

    figure.add_plot(
        &["fill=blue", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 20.0),
            Complex64::new(-20.0, 20.0),
            Complex64::new(-20.0, -20.0),
            Complex64::new(0.0, -20.0),
        ],
    )?;

    let us = pxu::kinematics::u_of_x(consts.s(), consts);
    let ikh = Complex64::new(0.0, consts.k() as f64 / consts.h);
    let cuts = [
        Cut::new(
            Component::U,
            vec![us, Complex64::from(20.0)],
            Some(us),
            CutType::ULongPositive(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us - ikh, Complex64::from(-20.0) - ikh],
            Some(-us - ikh),
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_long_lower(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-regions-between-long-lower",
        -7.25..7.25,
        0.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis()?;

    figure.add_plot(
        &["fill=green", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 20.0),
            Complex64::new(20.0, 20.0),
            Complex64::new(20.0, -20.0),
            Complex64::new(0.0, -20.0),
        ],
    )?;

    figure.add_plot(
        &["fill=red", "fill opacity=0.25", "draw=none"],
        &[
            Complex64::new(0.0, 20.0),
            Complex64::new(-20.0, 20.0),
            Complex64::new(-20.0, -20.0),
            Complex64::new(0.0, -20.0),
        ],
    )?;

    let us = pxu::kinematics::u_of_x(consts.s(), consts);
    let ikh = Complex64::new(0.0, consts.k() as f64 / consts.h);
    let cuts = [
        Cut::new(
            Component::U,
            vec![us, Complex64::from(20.0)],
            Some(us),
            CutType::ULongPositive(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![-us + ikh, Complex64::from(-20.0) + ikh],
            Some(-us + ikh),
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_long_circle(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(-0.5, consts);

    let mut figure = FigureWriter::new(
        "x-long-circle",
        -3.1..3.1,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let paths = [
        "x half circle between 1",
        "x half circle between 2",
        "x half circle between 3",
        "x half circle between 4",
    ];

    let paths = paths
        .into_iter()
        .map(|path_name| pxu_provider.get_path(path_name))
        .collect::<Result<Vec<_>>>()?;

    let first = *paths
        .first()
        .unwrap()
        .segments
        .first()
        .ok_or(error("No path?"))?
        .first()
        .ok_or(error("Empty segment?"))?
        .xp
        .first()
        .ok_or(error("Empty segment?"))?;

    let last = *paths
        .last()
        .unwrap()
        .segments
        .last()
        .ok_or(error("No path?"))?
        .last()
        .ok_or(error("Empty segment?"))?
        .xp
        .last()
        .ok_or(error("Empty segment?"))?;

    for path in paths {
        figure.add_path(&path, &pt, &["solid"])?;
        figure.add_path_arrows(&path, &[0.55], &["very thick", "Blue"])?;
    }

    figure.add_plot_all(
        &["only marks", "Blue", "mark size=0.06cm"],
        vec![first, last],
    )?;

    let s = consts.s();
    let cuts = vec![
        Cut::new(
            Component::Xp,
            vec![Complex64::from(-10.0), Complex64::from(-1.0 / s)],
            Some(Complex64::from(-1.0 / s)),
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::Xp,
            vec![Complex64::from(-1.0 / s), Complex64::zero()],
            Some(Complex64::zero()),
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::Xp,
            vec![Complex64::zero(), Complex64::from(10.0)],
            Some(Complex64::from(s)),
            CutType::ULongPositive(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.add_node("1", Complex64::new(-0.6, 0.8), &["anchor=mid", "Blue"])?;
    figure.add_node("2", Complex64::new(-0.6, -0.95), &["anchor=mid", "Blue"])?;
    figure.add_node("3", Complex64::new(-0.6, 1.7), &["anchor=mid", "Blue"])?;
    figure.add_node("4", Complex64::new(-0.6, -1.85), &["anchor=mid", "Blue"])?;

    figure.finish(cache, settings, pb)
}

enum HalfCircleMark {
    None,
    First,
    Last,
}

#[allow(clippy::too_many_arguments)]
fn draw_u_long_half_circle(
    name: &str,
    shift: f64,
    half: i32,
    label: &str,
    mark: HalfCircleMark,
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        name,
        -4.35..4.35,
        2.0 + shift,
        Size {
            width: 3.0,
            height: 5.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis_origin(Complex64::new(0.0, shift - 0.5))?;

    let path: Arc<pxu::Path> = pxu_provider.get_path(&format!("x half circle between {label}"))?;

    let first = path
        .segments
        .first()
        .ok_or(error("No path?"))?
        .first()
        .ok_or(error("Empty segment?"))?
        .u
        .first()
        .ok_or(error("Empty segment?"))?;

    let last = path
        .segments
        .last()
        .ok_or(error("No path?"))?
        .last()
        .ok_or(error("Empty segment?"))?
        .u
        .last()
        .ok_or(error("Empty segment?"))?;

    figure.add_path(&path, &pt, &["solid"])?;
    figure.add_path_arrows(&path, &[0.4, 0.7], &["very thick", "Blue"])?;

    let marks = match mark {
        HalfCircleMark::First => vec![*first],
        HalfCircleMark::Last => vec![*last],
        HalfCircleMark::None => vec![],
    };
    figure.add_plot_all(&["only marks", "Blue", "mark size=0.06cm"], marks)?;

    let shift = Complex64::new(0.0, -0.5);

    let us = pxu::kinematics::u_of_x(consts.s(), consts);
    let ikh = Complex64::new(0.0, consts.k() as f64 / consts.h);
    let cuts = [
        Cut::new(
            Component::U,
            vec![us + shift, Complex64::from(20.0) + shift],
            Some(us + shift),
            CutType::ULongPositive(Component::Xp),
            0,
            false,
            vec![],
        ),
        Cut::new(
            Component::U,
            vec![
                -us - ikh * half.signum() as f64 + shift,
                Complex64::from(-20.0) - ikh * half.signum() as f64 + shift,
            ],
            Some(-us - ikh * half.signum() as f64 + shift),
            CutType::Log(Component::Xp),
            0,
            false,
            vec![],
        ),
    ];

    for cut in cuts {
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    figure.add_node(
        label,
        us - ikh * half.signum() as f64 + shift,
        &["anchor=mid", "Blue"],
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_u_long_half_circle_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    draw_u_long_half_circle(
        "u-long-half-circle-1",
        0.0,
        1,
        "1",
        HalfCircleMark::First,
        pxu_provider,
        cache,
        settings,
        pb,
    )
}

fn fig_u_long_half_circle_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    draw_u_long_half_circle(
        "u-long-half-circle-2",
        0.0,
        -1,
        "2",
        HalfCircleMark::None,
        pxu_provider,
        cache,
        settings,
        pb,
    )
}

fn fig_u_long_half_circle_3(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    draw_u_long_half_circle(
        "u-long-half-circle-3",
        -5.0,
        1,
        "3",
        HalfCircleMark::None,
        pxu_provider,
        cache,
        settings,
        pb,
    )
}

fn fig_u_long_half_circle_4(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    draw_u_long_half_circle(
        "u-long-half-circle-4",
        -5.0,
        -1,
        "4",
        HalfCircleMark::Last,
        pxu_provider,
        cache,
        settings,
        pb,
    )
}

fn fig_x_short_circle(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(-0.5, consts);

    let mut figure = FigureWriter::new(
        "x-short-circle",
        -3.1..3.1,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;

    let paths = [
        "x half circle between 1",
        "x half circle between 2",
        "x half circle between 3",
        "x half circle between 4",
    ];

    let paths = paths
        .into_iter()
        .map(|path_name| pxu_provider.get_path(path_name))
        .collect::<Result<Vec<_>>>()?;

    let first = *paths
        .first()
        .unwrap()
        .segments
        .first()
        .ok_or(error("No path?"))?
        .first()
        .ok_or(error("Empty segment?"))?
        .xp
        .first()
        .ok_or(error("Empty segment?"))?;

    let last = *paths
        .last()
        .unwrap()
        .segments
        .last()
        .ok_or(error("No path?"))?
        .last()
        .ok_or(error("Empty segment?"))?
        .xp
        .last()
        .ok_or(error("Empty segment?"))?;

    for path in paths {
        figure.add_path(&path, &pt, &["solid"])?;
        figure.add_path_arrows(&path, &[0.55], &["very thick", "Blue"])?;
    }

    figure.add_plot_all(
        &["only marks", "Blue", "mark size=0.06cm"],
        vec![first, last],
    )?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp)
                    | CutType::UShortScallion(Component::Xp)
                    | CutType::Log(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], consts)?;
    }

    figure.add_node("1", Complex64::new(-0.6, 0.8), &["anchor=mid", "Blue"])?;
    figure.add_node("2", Complex64::new(-0.6, -0.95), &["anchor=mid", "Blue"])?;
    figure.add_node("3", Complex64::new(-0.6, 1.7), &["anchor=mid", "Blue"])?;
    figure.add_node("4", Complex64::new(-0.6, -1.85), &["anchor=mid", "Blue"])?;

    figure.finish(cache, settings, pb)
}

fn fig_u_short_circle(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "u-short-circle",
        -4.35..4.35,
        2.0,
        Size {
            width: 3.0,
            height: 5.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    figure.add_grid_lines(&contours, &[])?;
    figure.component_indicator("u");
    figure.add_axis_origin(Complex64::new(0.0, -0.5))?;

    let paths = ["x half circle between 1", "x half circle between 2"];

    let paths = paths
        .into_iter()
        .map(|path_name| pxu_provider.get_path(path_name))
        .collect::<Result<Vec<_>>>()?;

    let first = *paths
        .first()
        .unwrap()
        .segments
        .first()
        .ok_or(error("No path?"))?
        .first()
        .ok_or(error("Empty segment?"))?
        .u
        .first()
        .ok_or(error("Empty segment?"))?;

    for path in paths {
        figure.add_path(&path, &pt, &["solid"])?;
        figure.add_path_arrows(&path, &[0.55], &["very thick", "Blue"])?;
    }

    let paths = ["x half circle between 3", "x half circle between 4"];

    let paths = paths
        .into_iter()
        .map(|path_name| pxu_provider.get_path(path_name))
        .collect::<Result<Vec<_>>>()?;

    let last = Complex64::new(0.0, 2.0 * consts.k() as f64 / consts.h)
        + *paths
            .last()
            .unwrap()
            .segments
            .last()
            .ok_or(error("No path?"))?
            .last()
            .ok_or(error("Empty segment?"))?
            .u
            .last()
            .ok_or(error("Empty segment?"))?;

    for path in paths {
        let mut path = (*path).clone();
        for segs in path.segments.iter_mut() {
            for seg in segs.iter_mut() {
                for p in seg.u.iter_mut() {
                    *p += Complex64::new(0.0, 2.0 * consts.k() as f64 / consts.h);
                }
            }
        }

        figure.add_path(&path, &pt, &["solid"])?;
        figure.add_path_arrows(&path, &[0.55], &["very thick", "Blue"])?;
    }

    figure.add_plot_all(
        &["only marks", "Blue", "mark size=0.06cm"],
        vec![first, last],
    )?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::U, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xpl_cover(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "xpL-cover",
        -5.0..5.0,
        1.9,
        Size {
            width: 6.0,
            height: 3.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;
    figure.no_component_indicator();

    figure.add_axis()?;
    for contour in contours.get_grid(Component::Xp).iter().filter(
        |line| matches!(line.component, GridLineComponent::Xp(m) if (-8.0..=6.0).contains(&m)),
    ) {
        if contour.component == GridLineComponent::Xp(1.0) {
            figure.add_grid_line(contour, &["thick", "blue"])?;
        } else {
            figure.add_grid_line(contour, &["thick", "black"])?;
        }
    }

    figure.close_scope()?;
    figure.extend_left(0.25);

    for m in -4..=4 {
        figure.add_node(
            &format!(r"$\scriptstyle m={m}$"),
            Complex64::new(-5.0, 0.9 * ((consts.k() + m) as f64) / consts.h),
            &["anchor=east"],
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xml_cover(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "xmL-cover",
        -5.0..5.0,
        -1.9,
        Size {
            width: 6.0,
            height: 3.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;
    figure.no_component_indicator();

    figure.add_axis()?;
    for contour in contours.get_grid(Component::Xm).iter().filter(
        |line| matches!(line.component, GridLineComponent::Xm(m) if (-8.0..=6.0).contains(&m)),
    ) {
        if contour.component == GridLineComponent::Xm(1.0) {
            figure.add_grid_line(contour, &["thick", "blue"])?;
        } else {
            figure.add_grid_line(contour, &["thick", "black"])?;
        }
    }

    figure.close_scope()?;
    figure.extend_left(0.25);

    for m in -4..=4 {
        figure.add_node(
            &format!(r"$\scriptstyle m={m}$"),
            Complex64::new(-5.0, -0.9 * ((consts.k() + m) as f64) / consts.h),
            &["anchor=east"],
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn draw_legend(
    figure: &mut FigureWriter,
    labels: &[&str],
    styles: &[&str],
    south_west: bool,
) -> Result<()> {
    assert_eq!(labels.len(), styles.len());

    let scale = figure.bounds.height() / figure.size.height;
    let legend_step = 0.375 * scale;
    let legend_width = 1.3 * scale;
    let legend_margin = 0.25 * scale;

    let legend_se = if south_west {
        figure.bounds.south_west() + 0.1 * scale * Complex64::new(1.0, 1.0) + legend_width
    } else {
        figure.bounds.south_east() + 0.1 * scale * Complex64::new(-1.0, 1.0)
    };

    let legend_ne = legend_se + legend_step * (labels.len() as f64 + 0.5) * Complex64::i();
    let legend_nw = legend_ne - legend_width;

    figure.unset_r();

    figure.draw(
        &format!(
            "({},{}) rectangle ({},{})",
            legend_nw.re, legend_nw.im, legend_se.re, legend_se.im
        ),
        &["fill=white"],
    )?;

    for (i, (&style, label)) in izip!(styles.iter(), labels).enumerate() {
        let pos = legend_nw + legend_margin - (0.75 + i as f64) * legend_step * Complex64::i();

        let options: &[&str] = &[style];

        figure.add_plot_all(
            &[
                &["thick", "only marks", "mark=*", "mark size=0.065cm"],
                options,
            ]
            .concat(),
            vec![pos],
        )?;

        figure.add_node(
            &format!(r"$\scriptstyle {label}$"),
            pos + 0.1 * scale,
            &["anchor=west"],
        )?;
    }

    Ok(())
}

fn fig_xl_crossed_point_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "xL-crossed-point-0",
        -5.0..5.0,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator(r"x_{\mbox{\tiny L}}");

    let pt = pxu::Point::new(0.4, consts);
    let pt_r = pxu::Point::new(pt.p, CouplingConstants::new(consts.h, -consts.k()));

    for contour in contours.get_grid(Component::Xp) {
        let options: &[&str] = match contour.component {
            GridLineComponent::Xp(m) if m == 1.0 => &["thick", "Red"],
            GridLineComponent::Xm(m) if m == -1.0 => &["thick", "Red"],
            GridLineComponent::Xp(m) if m == -1.0 => &["thick", "Green"],
            GridLineComponent::Xm(m) if m == 1.0 => &["thick", "Green"],
            _ => &[],
        };

        figure.add_grid_line(contour, options)?;
    }

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        let mut cut = cut.clone();
        cut.branch_point = None;
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    let points = [
        pt.get(Component::Xp),
        1.0 / pt_r.get(Component::Xp),
        pt.get(Component::Xm),
        1.0 / pt_r.get(Component::Xm),
    ];

    let styles = [
        "Red",
        "Red,mark options={fill=white}",
        "Green",
        "Green,mark options={fill=white}",
    ];

    let labels = [
        r"x_{\mbox{\tiny L}}^+",
        r"1/x_{\mbox{\tiny R}}^+",
        r"x_{\mbox{\tiny L}}^-",
        r"1/x_{\mbox{\tiny R}}^-",
    ];

    for (&pos, &style) in izip!(points.iter(), styles.iter()) {
        let options: &[&str] = &[style];
        figure.add_plot_all(
            &[
                &["thick", "only marks", "mark=*", "mark size=0.065cm"],
                options,
            ]
            .concat(),
            vec![pos],
        )?;
    }

    draw_legend(&mut figure, &labels, &styles, false)?;

    figure.finish(cache, settings, pb)
}

fn fig_xl_crossed_point_min_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "xL-crossed-point-min-1",
        -1.4..1.4,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator(r"x_{\mbox{\tiny L}}");

    let pt = pxu::Point::new(-0.4, consts);
    let pt_r = pxu::Point::new(pt.p, CouplingConstants::new(consts.h, -consts.k()));

    for contour in contours.get_grid(Component::Xp) {
        let options: &[&str] = match contour.component {
            GridLineComponent::Xp(m) if m == -4.0 => &["thick", "Red"],
            GridLineComponent::Xm(m) if m == -6.0 => &["thick", "Red"],
            GridLineComponent::Xp(m) if m == -6.0 => &["thick", "Green"],
            GridLineComponent::Xm(m) if m == -4.0 => &["thick", "Green"],
            _ => &[],
        };

        figure.add_grid_line(contour, options)?;
    }

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        let mut cut = cut.clone();
        cut.branch_point = None;
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    let points = [
        pt.get(Component::Xp),
        1.0 / pt_r.get(Component::Xp),
        pt.get(Component::Xm),
        1.0 / pt_r.get(Component::Xm),
    ];

    let styles = [
        "Red",
        "Red,mark options={fill=white}",
        "Green",
        "Green,mark options={fill=white}",
    ];

    let labels = [
        r"x_{\mbox{\tiny L}}^+",
        r"1/x_{\mbox{\tiny R}}^+",
        r"x_{\mbox{\tiny L}}^-",
        r"1/x_{\mbox{\tiny R}}^-",
    ];

    for (&pos, &style) in izip!(points.iter(), styles.iter()) {
        let options: &[&str] = &[style];
        figure.add_plot_all(
            &[
                &["thick", "only marks", "mark=*", "mark size=0.065cm"],
                options,
            ]
            .concat(),
            vec![pos],
        )?;
    }

    draw_legend(&mut figure, &labels, &styles, false)?;

    figure.finish(cache, settings, pb)
}

fn fig_xr_crossed_point_min_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "xR-crossed-point-min-1",
        -5.0..5.0,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.set_r();

    figure.component_indicator(r"x_{\mbox{\tiny R}}");

    let pt = pxu::Point::new(0.4, consts);
    let pt_r = pxu::Point::new(pt.p, CouplingConstants::new(consts.h, -consts.k()));

    for contour in contours.get_grid(Component::Xp) {
        let options: &[&str] = match contour.component {
            GridLineComponent::Xp(m) if m == 1.0 => &["thick", "Red"],
            GridLineComponent::Xm(m) if m == -1.0 => &["thick", "Red"],
            GridLineComponent::Xp(m) if m == -1.0 => &["thick", "Green"],
            GridLineComponent::Xm(m) if m == 1.0 => &["thick", "Green"],
            _ => &[],
        };

        figure.add_grid_line(contour, options)?;
    }

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        let mut cut = cut.clone();
        cut.branch_point = None;
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    let points = [
        pt.get(Component::Xp),
        1.0 / pt_r.get(Component::Xp),
        pt.get(Component::Xm),
        1.0 / pt_r.get(Component::Xm),
    ];

    let styles = [
        "Red,mark options={fill=white}",
        "Red",
        "Green,mark options={fill=white}",
        "Green",
    ];

    let labels = [
        r"x_{\mbox{\tiny R}}^+",
        r"1/x_{\mbox{\tiny L}}^+",
        r"x_{\mbox{\tiny R}}^-",
        r"1/x_{\mbox{\tiny L}}^-",
    ];

    for (&pos, &style) in izip!(points.iter(), styles.iter()) {
        let options: &[&str] = &[style];
        figure.add_plot_all(
            &[
                &["thick", "only marks", "mark=*", "mark size=0.065cm"],
                options,
            ]
            .concat(),
            vec![pos],
        )?;
    }

    draw_legend(&mut figure, &labels, &styles, true)?;

    figure.finish(cache, settings, pb)
}

fn fig_xr_crossed_point_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "xR-crossed-point-0",
        -1.4..1.4,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.set_r();

    figure.component_indicator(r"x_{\mbox{\tiny R}}");

    let pt = pxu::Point::new(-0.4, consts);
    let pt_r = pxu::Point::new(pt.p, CouplingConstants::new(consts.h, -consts.k()));

    for contour in contours.get_grid(Component::Xp) {
        let options: &[&str] = match contour.component {
            GridLineComponent::Xp(m) if m == -4.0 => &["thick", "Red"],
            GridLineComponent::Xm(m) if m == -6.0 => &["thick", "Red"],
            GridLineComponent::Xp(m) if m == -6.0 => &["thick", "Green"],
            GridLineComponent::Xm(m) if m == -4.0 => &["thick", "Green"],
            _ => &[],
        };

        figure.add_grid_line(contour, options)?;
    }

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::Xp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortKidney(Component::Xp) | CutType::UShortScallion(Component::Xp)
            )
        })
    {
        let mut cut = cut.clone();
        cut.branch_point = None;
        figure.add_cut(&cut, &["black", "very thick"], consts)?;
    }

    let points = [
        pt.get(Component::Xp),
        1.0 / pt_r.get(Component::Xp),
        pt.get(Component::Xm),
        1.0 / pt_r.get(Component::Xm),
    ];

    let styles = [
        "Red,mark options={fill=white}",
        "Red",
        "Green,mark options={fill=white}",
        "Green",
    ];

    let labels = [
        r"x_{\mbox{\tiny R}}^+",
        r"1/x_{\mbox{\tiny L}}^+",
        r"x_{\mbox{\tiny R}}^-",
        r"1/x_{\mbox{\tiny L}}^-",
    ];

    for (&pos, &style) in izip!(points.iter(), styles.iter()) {
        let options: &[&str] = &[style];
        figure.add_plot_all(
            &[
                &["thick", "only marks", "mark=*", "mark size=0.065cm"],
                options,
            ]
            .concat(),
            vec![pos],
        )?;
    }

    draw_legend(&mut figure, &labels, &styles, true)?;

    figure.finish(cache, settings, pb)
}

fn fig_p_plane_short_cuts(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "p-plane-short-cuts",
        -2.6..2.6,
        0.0,
        Size {
            width: 25.0,
            height: 10.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::P, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::E
                    | CutType::Log(_)
                    | CutType::UShortKidney(_)
                    | CutType::UShortScallion(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_plane_short_cuts_rr_075(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(0.75, 0);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "p-plane-short-cuts-RR-075",
        -1.6..1.6,
        0.0,
        Size {
            width: 10.0,
            height: 4.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::P, consts)
        .filter(|cut| matches!(cut.typ, CutType::E | CutType::UShortScallion(_)))
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_plane_short_cuts_rr_200(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 0);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "p-plane-short-cuts-RR-200",
        -1.6..1.6,
        0.0,
        Size {
            width: 10.0,
            height: 4.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::P, consts)
        .filter(|cut| matches!(cut.typ, CutType::E | CutType::UShortScallion(_)))
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xp_cuts_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "xp-cuts-1",
        -4.0..4.0,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.add_axis()?;
    for contour in contours
        .get_grid(Component::Xp)
        .iter()
        .filter(|line| matches!(line.component, GridLineComponent::Xp(m) | GridLineComponent::Xm(m) if (-10.0..).contains(&m)))
    {
        figure.add_grid_line(contour, &[])?;
    }

    figure.add_cuts(&contours, &pt, consts, &[])?;

    figure.finish(cache, settings, pb)
}

fn fig_xm_cuts_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "xm-cuts-1",
        -4.0..4.0,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    figure.add_axis()?;
    for contour in contours
        .get_grid(Component::Xp)
        .iter()
        .filter(|line| matches!(line.component, GridLineComponent::Xp(m) | GridLineComponent::Xm(m) if (-10.0..).contains(&m)))
    {
        figure.add_grid_line(contour, &[])?;
    }

    figure.add_cuts(&contours, &pt, consts, &[])?;

    figure.finish(cache, settings, pb)
}

#[allow(clippy::too_many_arguments)]
fn draw_path_figure(
    figure: FigureWriter,
    paths: &[&str],
    pxu_provider: Arc<PxuProvider>,
    consts: CouplingConstants,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let dummy_str: &[&str] = &[];
    let dummy_f64: &[f64] = &[];

    #[allow(clippy::type_complexity)]
    let paths: Vec<(&str, &[&str], Option<&[&str]>, &[f64])> = paths
        .iter()
        .map(|&name| (name, dummy_str, None, dummy_f64))
        .collect::<Vec<_>>();

    draw_path_figure_with_options_and_start_end_marks_and_arrows_and_labels(
        figure,
        &paths,
        &[],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

#[allow(clippy::too_many_arguments)]
fn draw_path_figure_with_options(
    figure: FigureWriter,
    paths: &[(&str, &[&str])],
    pxu_provider: Arc<PxuProvider>,
    consts: CouplingConstants,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let dummy_f64: &[f64] = &[];

    #[allow(clippy::type_complexity)]
    let paths: Vec<(&str, &[&str], Option<&[&str]>, &[f64])> = paths
        .iter()
        .map(|&(name, options)| (name, options, None, dummy_f64))
        .collect::<Vec<_>>();

    draw_path_figure_with_options_and_start_end_marks_and_arrows_and_labels(
        figure,
        &paths,
        &[],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn draw_path_figure_with_options_and_start_end_marks_and_arrows_and_labels(
    mut figure: FigureWriter,
    paths: &[(&str, &[&str], Option<&[&str]>, &[f64])],
    labels: &[(&str, Complex64, &[&str])],
    pxu_provider: Arc<PxuProvider>,
    consts: CouplingConstants,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let contours = pxu_provider.get_contours(consts)?;

    let mut pt = pxu::Point::new(0.5, consts);
    pt.sheet_data = pxu_provider.get_path(paths[0].0)?.segments[0][0]
        .sheet_data
        .clone();

    figure.add_grid_lines(&contours, &[])?;
    figure.add_cuts(&contours, &pt, consts, &[])?;

    for (name, options, mark_options, arrow_pos) in paths {
        let path = pxu_provider.get_path(name)?;
        figure.add_path(&path, &pt, options)?;
        if let Some(mark_options) = mark_options {
            figure.add_path_start_end_mark(&path, mark_options)?;
        }
        figure.add_path_arrows(&path, arrow_pos, options)?;
    }

    for (text, pos, options) in labels {
        figure.add_node(text, *pos, options)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_period_between_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let mut pt = pxu::Point::new(0.5, consts);

    let figure = FigureWriter::new(
        "u-period-between-between",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    draw_path_figure(
        figure,
        &["U period between/between"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_u_band_between_outside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let mut pt = pxu::Point::new(0.5, consts);

    let figure = FigureWriter::new(
        "u-band-between-outside",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Outside,
    );

    draw_path_figure(
        figure,
        &["U band between/outside"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_u_band_between_inside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let mut pt = pxu::Point::new(0.5, consts);

    let figure = FigureWriter::new(
        "u-band-between-inside",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        Component::U,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Inside,
    );

    draw_path_figure(
        figure,
        &["U band between/inside"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_p_band_between_outside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "p-band-between-outside",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["U band between/outside"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_p_band_between_inside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "p-band-between-inside",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["U band between/inside"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_band_between_inside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let mut pt = pxu::Point::new(0.5, consts);

    let figure = FigureWriter::new(
        "xp-band-between-inside",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (UBranch::Between, UBranch::Inside);
    pt.sheet_data.log_branch_p = 0;
    pt.sheet_data.log_branch_m = -1;
    pt.sheet_data.im_x_sign = (1, -1);

    draw_path_figure_with_options(
        figure,
        &[("U band between/inside (single)", &["solid"])],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_band_between_outside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let mut pt = pxu::Point::new(0.5, consts);

    let figure = FigureWriter::new(
        "xp-band-between-outside",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (UBranch::Between, UBranch::Outside);
    pt.sheet_data.log_branch_p = 0;
    pt.sheet_data.log_branch_m = -1;
    pt.sheet_data.im_x_sign = (1, -1);

    draw_path_figure_with_options(
        figure,
        &[("U band between/outside (single)", &["solid"])],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_band_between_inside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let mut pt = pxu::Point::new(0.5, consts);

    let figure = FigureWriter::new(
        "xm-band-between-inside",
        -0.8..0.4,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (UBranch::Between, UBranch::Inside);
    pt.sheet_data.log_branch_p = 0;
    pt.sheet_data.log_branch_m = -1;
    pt.sheet_data.im_x_sign = (1, -1);

    draw_path_figure(
        figure,
        &["U band between/inside"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_band_between_outside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let mut pt = pxu::Point::new(0.5, consts);

    let figure = FigureWriter::new(
        "xm-band-between-outside",
        -7.0..7.0,
        0.0,
        Size {
            width: 8.0,
            height: 16.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    pt.sheet_data.u_branch = (UBranch::Between, UBranch::Outside);
    pt.sheet_data.log_branch_p = 0;
    pt.sheet_data.log_branch_m = -1;
    pt.sheet_data.im_x_sign = (1, -1);

    draw_path_figure(
        figure,
        &["U band between/outside"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_period_between_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-period-between-between",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    draw_path_figure_with_options(
        figure,
        &[("U period between/between (single)", &["solid"])],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_period_between_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-period-between-between",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    draw_path_figure_with_options(
        figure,
        &[("U period between/between (single)", &["solid"])],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_p_period_between_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "p-period-between-between",
        -0.15..0.15,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["U period between/between (single)"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_p_circle_between_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "p-circle-between-between",
        -0.15..0.15,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["xp circle between/between (single)"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_circle_between_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-circle-between-between",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    draw_path_figure_with_options(
        figure,
        &[("xp circle between/between (single)", &["solid"])],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_circle_between_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-circle-between-between",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    draw_path_figure_with_options(
        figure,
        &[("xp circle between/between (single)", &["solid"])],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_u_circle_between_between(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "u-circle-between-between",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        Component::U,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["xp circle between/between"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_u_circle_between_outside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "u-circle-between-outside",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        Component::U,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["xp circle between/outside L", "xp circle between/outside R"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_u_circle_between_inside(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "u-circle-between-inside",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        Component::U,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["xp circle between/inside L", "xp circle between/inside R"],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_p_crossing_all(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "p-crossing-all",
        -1.6..1.6,
        0.0,
        Size {
            width: 12.0,
            height: 5.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    draw_path_figure_with_options_and_start_end_marks_and_arrows_and_labels(
        figure,
        &[
            (
                "p crossing a",
                &["solid", "thick", "blue"],
                Some(&["Black", "mark size=0.05cm"]),
                &[0.3, 0.8],
            ),
            (
                "p crossing b",
                &["solid", "thick", "blue"],
                None,
                &[0.3, 0.8],
            ),
            (
                "p crossing c",
                &["solid", "thick", "cyan"],
                None,
                &[0.21, 0.71],
            ),
            (
                "p crossing d",
                &["solid", "thick", "magenta"],
                None,
                &[0.33, 0.83],
            ),
        ],
        &[
            (
                r"\footnotesize $1$",
                Complex64::new(0.091, 0.029),
                &["anchor=south west", "blue"],
            ),
            (
                r"\footnotesize $1'$",
                Complex64::new(0.091, -0.029),
                &["anchor=north west", "blue"],
            ),
            (
                r"\footnotesize $2$",
                Complex64::new(0.498, 0.142),
                &["anchor=north west", "cyan"],
            ),
            (
                r"\footnotesize $2'$",
                Complex64::new(-0.443, -0.172),
                &["anchor=north west", "magenta"],
            ),
        ],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_crossing_all(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-crossing-all",
        -5.0..5.0,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    draw_path_figure_with_options_and_start_end_marks_and_arrows_and_labels(
        figure,
        &[
            (
                "p crossing a",
                &["solid", "thick", "blue"],
                Some(&["Black", "mark size=0.05cm"]),
                &[0.55],
            ),
            ("p crossing b", &["solid", "thick", "blue"], None, &[0.5]),
            ("p crossing c", &["solid", "thick", "cyan"], None, &[0.5]),
            (
                "p crossing d",
                &["solid", "thick", "magenta"],
                None,
                &[0.3, 0.8],
            ),
        ],
        &[
            (
                r"\footnotesize $1$",
                Complex64::new(2.08, -0.44),
                &["anchor=north west", "blue"],
            ),
            (
                r"\footnotesize $1'$",
                Complex64::new(2.58, 1.59),
                &["anchor=west", "blue"],
            ),
            (
                r"\footnotesize $2$",
                Complex64::new(-0.80, -0.45),
                &["anchor=north east", "cyan"],
            ),
            (
                r"\footnotesize $2'$",
                Complex64::new(3.58, 2.34),
                &["anchor=west", "magenta"],
            ),
        ],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_crossing_all(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-crossing-all",
        -5.0..5.0,
        -0.7,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    draw_path_figure_with_options_and_start_end_marks_and_arrows_and_labels(
        figure,
        &[
            (
                "p crossing a",
                &["solid", "thick", "blue"],
                Some(&["Black", "mark size=0.05cm"]),
                &[0.3, 0.8],
            ),
            ("p crossing b", &["solid", "thick", "blue"], None, &[0.35]),
            (
                "p crossing c",
                &["solid", "thick", "cyan"],
                None,
                &[0.37, 0.7],
            ),
            ("p crossing d", &["solid", "thick", "magenta"], None, &[0.3]),
        ],
        &[
            (
                r"\footnotesize $1$",
                Complex64::new(1.056, -1.734),
                &["anchor=north east", "blue"],
            ),
            (
                r"\footnotesize $1'$",
                Complex64::new(1.917, 0.718),
                &["anchor=south west", "blue"],
            ),
            (
                r"\footnotesize $2$",
                Complex64::new(3.227, -2.985),
                &["anchor=west", "cyan"],
            ),
            (
                r"\footnotesize $2'$",
                Complex64::new(3.331, 1.040),
                &["anchor=west", "magenta"],
            ),
        ],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_crossing_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "xp-crossing-1",
        -2.0..3.0,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let pathname = "p crossing a";

    let path = pxu_provider.get_path(pathname)?;
    let pt = &pxu_provider.get_start(pathname)?.points[0];

    figure.add_grid_lines(&contours, &[])?;

    figure.add_path(&path, pt, &["thick", "Blue"])?;
    figure.add_path_start_end_mark(&path, &["Blue", "mark size=0.05cm"])?;
    figure.add_path_arrows(&path, &[0.55], &["thick", "Blue"])?;

    let comp = figure.component;
    for cut in contours
        .get_visible_cuts_from_point(pt, comp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            ) || matches!(cut.typ,CutType::Log(c) if c == comp)
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xm_crossing_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "xm-crossing-1",
        -2.0..3.0,
        -0.7,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    let pathname = "p crossing a";

    let path = pxu_provider.get_path(pathname)?;
    let pt = &pxu_provider.get_start(pathname)?.points[0];

    figure.add_grid_lines(&contours, &[])?;

    figure.add_path(&path, pt, &["thick", "Blue"])?;
    figure.add_path_start_end_mark(&path, &["Blue", "mark size=0.05cm"])?;
    figure.add_path_arrows(&path, &[0.55], &["thick", "Blue"])?;

    let comp = figure.component;
    for cut in contours
        .get_visible_cuts_from_point(pt, comp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            ) || matches!(cut.typ,CutType::Log(c) if c == comp)
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_crossing_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "u-crossing-1",
        0.0..3.0,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let pathname = "p crossing a";

    let path = pxu_provider.get_path(pathname)?;
    let pt = &pxu_provider.get_start(pathname)?.points[0];

    figure.add_grid_lines(&contours, &[])?;

    figure.add_path(&path, pt, &["thick", "Blue"])?;
    figure.add_path_start_end_mark(&path, &["Blue", "mark size=0.05cm"])?;
    figure.add_path_arrows(&path, &[0.5], &["thick", "Blue"])?;

    let comp = figure.component;
    for cut in contours
        .get_visible_cuts_from_point(pt, comp, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_crossing_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "u-crossing-0",
        -3.0..3.0,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &[
            "U crossing from 0-2pi path A",
            "U crossing from 0-2pi path B",
        ],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_crossing_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-crossing-0",
        -3.0..3.0,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &[
            "U crossing from 0-2pi path A",
            "U crossing from 0-2pi path B",
        ],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_crossing_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-crossing-0",
        -1.5..4.4,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &[
            "U crossing from 0-2pi path A",
            "U crossing from 0-2pi path B",
        ],
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn draw_state_figure(
    mut figure: FigureWriter,
    state_strings: &[&str],
    pxu_provider: Arc<PxuProvider>,
    consts: CouplingConstants,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let states = load_states(state_strings)?;
    let contours = pxu_provider.get_contours(consts)?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_cuts(&contours, &states[0].points[0], consts, &[])?;

    let colors = ["Blue", "MediumOrchid", "Coral", "DarkOrange", "DarkViolet"];

    let marks = [
        "mark=*",
        "mark=square*",
        "mark=diamond*",
        "mark=pentagon*",
        "mark=triangle*",
        "mark=+",
        "mark=x",
    ];

    for (state, color, mark) in itertools::izip!(
        states.into_iter(),
        colors.into_iter().cycle(),
        marks.into_iter().cycle(),
    ) {
        figure.add_state(&state, &["only marks", color, mark, "mark size=0.075cm"])?;
    }
    figure.finish(cache, settings, pb)
}

fn fig_p_two_particle_bs_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "p-two-particle-bs-0",
        -0.05..1.0,
        0.0,
        Size {
            width: 8.0,
            height: 4.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
    ];

    draw_state_figure(
        figure,
        &state_strings,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn draw_x_bound_state_figure(
    mut figure: FigureWriter,
    state_strings: &[&str],
    anchor_fn: &dyn Fn(usize) -> &'static str,
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let states: Vec<pxu::State> = state_strings
        .iter()
        .map(|s| ron::from_str(s).map_err(|_| error("Could not load state")))
        .collect::<Result<Vec<_>>>()?;

    figure.component_indicator(r"x^{\pm}");
    figure.add_grid_lines(&contours, &[])?;

    for cut in contours
        .get_visible_cuts_from_point(&states[0].points[0], figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(Component::Xp) | CutType::UShortKidney(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["Black"], consts)?;
    }

    let colors = ["Blue", "Red"];
    let marks = ["*", "o"];
    for (state, color, mark) in izip!(states, colors, marks) {
        let mut points = state
            .points
            .iter()
            .map(|pt| pt.get(Component::Xp))
            .collect::<Vec<_>>();
        points.push(state.points.last().unwrap().get(Component::Xm));

        for (i, pos) in points.iter().enumerate() {
            let text = if i == 0 {
                "$\\scriptstyle X^+ = x_1^+$".to_owned()
            } else if i == points.len() - 1 {
                format!("$\\scriptstyle X^- = x_{}^-$", i)
            } else {
                format!("$\\scriptstyle x_{}^- = x_{}^+$", i, i + 1)
            };
            let anchor = &format!("anchor={}", anchor_fn(i));
            figure.add_node(&text, *pos, &[anchor])?;
        }

        figure.add_plot_all(
            &[
                "only marks",
                color,
                &format!("mark={mark}"),
                "mark size=0.065cm",
            ],
            points,
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_typical_bound_state(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "x-typical-bound-states",
        -4.0..7.0,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_strings = [
        // "(points:[(p:(-0.01281836032081622,-0.03617430043713721),xp:(-0.5539661576009564,4.096675591673073),xm:(-0.7024897294980745,3.2176928460399083),u:(-1.7157735474931681,1.9999999999999996),x:(-0.6278118911147218,3.651492613118212),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019778339646048883,-0.041578695061571934),xp:(-0.7024897294980745,3.2176928460399083),xm:(-0.8439501836107429,2.391751872316718),u:(-1.7157735474931681,0.9999999999999993),x:(-0.7756824568522961,2.7972312015320973),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.6079768155592542,-0.000000000000000025609467106049815),xp:(-0.8439501836107431,2.3917518723167186),xm:(-0.8439501836107433,-2.3917518723167186),u:(-1.7157735474931681,-0.0000000000000004440892098500626),x:(-0.9025872691909044,-2.0021375758700994),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019778339646048887,0.04157869506157193),xp:(-0.8439501836107434,-2.391751872316718),xm:(-0.7024897294980749,-3.217692846039909),u:(-1.7157735474931686,-0.9999999999999991),x:(-0.7756824568522963,-2.7972312015320973),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.01281836032081622,0.0361743004371372),xp:(-0.7024897294980751,-3.217692846039909),xm:(-0.5539661576009569,-4.0966755916730735),u:(-1.7157735474931686,-1.9999999999999998),x:(-0.6278118911147222,-3.651492613118212),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(-0.008285099942215936,-0.03124489976444211),xp:(-0.41379014705206596,5.013730349990057),xm:(-0.5539512485108423,4.096765155780589),u:(-1.7157731060643773,3.000099539239211),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(-0.012817797608166157,-0.03617378274379514),xp:(-0.5539512485108438,4.096765155780585),xm:(-0.7024745389520475,3.217777875518938),u:(-1.7157731060643784,2.0000995392392076),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019777502854940465,-0.04157814705589314),xp:(-0.7024745389520499,3.2177778755189355),xm:(-0.8439370224593588,2.391830970565371),u:(-1.7157731060643804,1.0000995392392027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.6079767764853242,-0.000008833067157527095),xp:(-0.8439370224593605,2.391830970565368),xm:(-0.8439626423264122,-2.3916726610840278),u:(-1.7157731060643822,0.0000995392391995864),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019779171573578672,0.041579250470216406),xp:(-0.8439626423264142,-2.3916726610840273),xm:(-0.7025041652445985,-3.21760768570613),u:(-1.7157731060643844,-0.9999004607608009),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.012818918443990657,0.03617482310579956),xp:(-0.7025041652445959,-3.2176076857061333),xm:(-0.5539802718296103,-4.096585899228867),u:(-1.7157731060643822,-1.9999004607608049),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.008285809485964725,0.031245812444520096),xp:(-0.5539802718296084,-4.09658589922887),xm:(-0.4138167904094644,-5.013544938781717),u:(-1.7157731060643802,-2.9999004607608075),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))],unlocked:true)",
        "(points:[(p:(0.0369899543404076,-0.029477676458957484),xp:(3.725975442509692,2.6128313499217866),xm:(3.5128286480709265,1.3995994557612454),u:(2.7000494004152316,1.5000010188076138),x:(3.6217633112309158,2.022895894514536),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06034321575136616,-0.018323213928633217),xp:(3.512828648070947,1.3995994557612081),xm:(3.3701632658975504,0.000001507484578833207),u:(2.700049400415252,0.5000010188075885),x:(3.4147970768250535,0.7263861464447217),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06034326215107557,0.018323155770842862),xp:(3.370163265897615,0.0000015074845481910515),xm:(3.5128282084799323,-1.3995968258500417),u:(2.700049400415295,-0.49999898119243236),x:(3.4147967471340466,-0.7263832822620354),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.03698999112227798,0.029477675660386345),xp:(3.5128282084799114,-1.3995968258500804),xm:(3.7259750341536533,-2.6128289961240028),u:(2.700049400415274,-1.4999989811924586),x:(3.621762872183573,-2.0228934323008243),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1)))])"
    ];

    draw_x_bound_state_figure(
        figure,
        &state_strings,
        &|_| "west",
        pxu_provider,
        cache,
        settings,
        pb,
    )
}

fn fig_p_typical_bound_state(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "p-typical-bound-states",
        -0.05..0.85,
        0.0,
        Size {
            width: 12.0,
            height: 5.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let state_strings = [
        // "(points:[(p:(-0.01281836032081622,-0.03617430043713721),xp:(-0.5539661576009564,4.096675591673073),xm:(-0.7024897294980745,3.2176928460399083),u:(-1.7157735474931681,1.9999999999999996),x:(-0.6278118911147218,3.651492613118212),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019778339646048883,-0.041578695061571934),xp:(-0.7024897294980745,3.2176928460399083),xm:(-0.8439501836107429,2.391751872316718),u:(-1.7157735474931681,0.9999999999999993),x:(-0.7756824568522961,2.7972312015320973),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.6079768155592542,-0.000000000000000025609467106049815),xp:(-0.8439501836107431,2.3917518723167186),xm:(-0.8439501836107433,-2.3917518723167186),u:(-1.7157735474931681,-0.0000000000000004440892098500626),x:(-0.9025872691909044,-2.0021375758700994),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019778339646048887,0.04157869506157193),xp:(-0.8439501836107434,-2.391751872316718),xm:(-0.7024897294980749,-3.217692846039909),u:(-1.7157735474931686,-0.9999999999999991),x:(-0.7756824568522963,-2.7972312015320973),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.01281836032081622,0.0361743004371372),xp:(-0.7024897294980751,-3.217692846039909),xm:(-0.5539661576009569,-4.0966755916730735),u:(-1.7157735474931686,-1.9999999999999998),x:(-0.6278118911147222,-3.651492613118212),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(-0.008285099942215936,-0.03124489976444211),xp:(-0.41379014705206596,5.013730349990057),xm:(-0.5539512485108423,4.096765155780589),u:(-1.7157731060643773,3.000099539239211),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(-0.012817797608166157,-0.03617378274379514),xp:(-0.5539512485108438,4.096765155780585),xm:(-0.7024745389520475,3.217777875518938),u:(-1.7157731060643784,2.0000995392392076),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019777502854940465,-0.04157814705589314),xp:(-0.7024745389520499,3.2177778755189355),xm:(-0.8439370224593588,2.391830970565371),u:(-1.7157731060643804,1.0000995392392027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.6079767764853242,-0.000008833067157527095),xp:(-0.8439370224593605,2.391830970565368),xm:(-0.8439626423264122,-2.3916726610840278),u:(-1.7157731060643822,0.0000995392391995864),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019779171573578672,0.041579250470216406),xp:(-0.8439626423264142,-2.3916726610840273),xm:(-0.7025041652445985,-3.21760768570613),u:(-1.7157731060643844,-0.9999004607608009),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.012818918443990657,0.03617482310579956),xp:(-0.7025041652445959,-3.2176076857061333),xm:(-0.5539802718296103,-4.096585899228867),u:(-1.7157731060643822,-1.9999004607608049),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.008285809485964725,0.031245812444520096),xp:(-0.5539802718296084,-4.09658589922887),xm:(-0.4138167904094644,-5.013544938781717),u:(-1.7157731060643802,-2.9999004607608075),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))],unlocked:true)",
        "(points:[(p:(0.0369899543404076,-0.029477676458957484),xp:(3.725975442509692,2.6128313499217866),xm:(3.5128286480709265,1.3995994557612454),u:(2.7000494004152316,1.5000010188076138),x:(3.6217633112309158,2.022895894514536),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06034321575136616,-0.018323213928633217),xp:(3.512828648070947,1.3995994557612081),xm:(3.3701632658975504,0.000001507484578833207),u:(2.700049400415252,0.5000010188075885),x:(3.4147970768250535,0.7263861464447217),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06034326215107557,0.018323155770842862),xp:(3.370163265897615,0.0000015074845481910515),xm:(3.5128282084799323,-1.3995968258500417),u:(2.700049400415295,-0.49999898119243236),x:(3.4147967471340466,-0.7263832822620354),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.03698999112227798,0.029477675660386345),xp:(3.5128282084799114,-1.3995968258500804),xm:(3.7259750341536533,-2.6128289961240028),u:(2.700049400415274,-1.4999989811924586),x:(3.621762872183573,-2.0228934323008243),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1)))])"
    ];

    let states: Vec<pxu::State> = state_strings
        .iter()
        .map(|s| ron::from_str(s).map_err(|_| error("Could not load state")))
        .collect::<Result<Vec<_>>>()?;

    figure.add_grid_lines(&contours, &[])?;

    figure.add_plot(
        &["very thin", "lightgray"],
        &[Complex64::from(-10.0), Complex64::from(10.0)],
    )?;

    figure.add_cuts(&contours, &states[0].points[0], consts, &[])?;

    let colors = ["Blue", "Red"];
    let marks = ["*", "o"];
    for (state, color, mark) in izip!(states, colors, marks) {
        let points = state
            .points
            .iter()
            .map(|pt| pt.get(Component::P))
            .collect::<Vec<_>>();

        figure.add_plot_all(
            &[
                "only marks",
                color,
                &format!("mark={mark}"),
                "mark size=0.05cm",
            ],
            points,
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_bound_state_region_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "p-bound-state-region-1",
        -0.2..1.8,
        0.0,
        Size {
            width: 12.0,
            height: 5.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(1.18723732607551,-0.017900744639078304),xp:(5.343571274474835,4.112533502713208),xm:(5.227614240073456,-2.996639019704647),u:(3.942370414738855,-1.9998999607629369),sheet_data:(log_branch_p:1,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(0.02155164482525027,0.017897155757077343),xp:(5.227614240073457,-2.996639019704648),xm:(5.343548529832183,-4.1123137550256015),u:(3.9423704147388543,-2.999899960762939),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1)))],unlocked:false)",
    ];

    let states: Vec<pxu::State> = state_strings
        .iter()
        .map(|s| ron::from_str(s).map_err(|_| error("Could not load state")))
        .collect::<Result<Vec<_>>>()?;

    figure.add_grid_lines(&contours, &[])?;

    figure.add_plot(
        &["very thin", "lightgray"],
        &[Complex64::from(-10.0), Complex64::from(10.0)],
    )?;

    figure.add_cuts(&contours, &states[0].points[0], consts, &[])?;

    let colors = ["Blue", "Red"];
    let marks = ["*", "o"];
    for (state, color, mark) in izip!(states, colors, marks) {
        let points = state
            .points
            .iter()
            .map(|pt| pt.get(Component::P))
            .collect::<Vec<_>>();

        figure.add_plot_all(
            &[
                "only marks",
                color,
                &format!("mark={mark}"),
                "mark size=0.05cm",
            ],
            points,
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_bound_state_regions_min_1_min_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "p-bound-state-regions-min-1-min-2",
        -1.8..0.2,
        0.0,
        Size {
            width: 12.0,
            height: 5.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(-1.332081405906118,-0.04538049641071554),xp:(-0.17511033258995276,0.2771633748573245),xm:(-0.11636131599061295,-0.21732052778191732),u:(-0.9168424606184588,5.500100069319226),sheet_data:(log_branch_p:-2,log_branch_m:0,e_branch:1,u_branch:(Inside,Inside),im_x_sign:(1,1))),(p:(0.011437172821809637,0.04536990584917373),xp:(-0.11636131599061278,-0.2173205277819174),xm:(-0.17509442946452716,-0.2771476182033895),u:(-0.9168424606184558,4.500100069319232),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,1)))],unlocked:false)",
        "(points:[(p:(-0.10396889396070738,-0.058571065344782174),xp:(-1.1673288038094392,0.8936432232901272),xm:(-1.0174826765753087,0.0001224475526552249),u:(-2.014092443020625,3.0000999381214077),sheet_data:(log_branch_p:-1,log_branch_m:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.10399507514856618,0.05855992759638331),xp:(-1.0174826765753078,0.00012244755265466978),xm:(-1.1673151913145814,-0.8934917573729062),u:(-2.014092443020624,2.0000999381214073),sheet_data:(log_branch_p:-1,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ];

    let states: Vec<pxu::State> = state_strings
        .iter()
        .map(|s| ron::from_str(s).map_err(|_| error("Could not load state")))
        .collect::<Result<Vec<_>>>()?;

    figure.add_grid_lines(&contours, &[])?;

    figure.add_plot(
        &["very thin", "lightgray"],
        &[Complex64::from(-10.0), Complex64::from(10.0)],
    )?;

    figure.add_cuts(&contours, &states[0].points[0], consts, &[])?;

    let colors = ["Blue", "Red"];
    let marks = ["*", "o"];
    for (state, color, mark) in izip!(states, colors, marks) {
        let points_e_plus = state
            .points
            .iter()
            .filter(|pt| pt.sheet_data.e_branch > 0)
            .map(|pt| pt.get(Component::P))
            .collect::<Vec<_>>();

        figure.add_plot_all(
            &[
                "only marks",
                color,
                &format!("mark={mark}"),
                "mark size=0.05cm",
            ],
            points_e_plus,
        )?;

        let points_e_min = state
            .points
            .iter()
            .filter(|pt| pt.sheet_data.e_branch < 0)
            .map(|pt| pt.get(Component::P))
            .collect::<Vec<_>>();

        figure.add_plot_all(
            &[
                "only marks",
                "Gray",
                &format!("mark={mark}"),
                "mark size=0.05cm",
            ],
            points_e_min,
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_bound_state_region_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "x-bound-state-region-1",
        -4.0..7.0,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(1.5344982847391835,-0.03125157629093187),xp:(-0.4137901655608822,5.013730158365311),xm:(-0.5539802334816937,-4.096586081878231),u:(-1.7157730965680082,-1.9999006651456805),sheet_data:(log_branch_p:1,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(-0.00828580874234546,0.031245811489086096),xp:(-0.5539802413347306,-4.0965860869401025),xm:(-0.4138167624035101,-5.013545132940062),u:(-1.715773105953617,-2.9999006692476753),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))],unlocked:false)",
    ];

    draw_x_bound_state_figure(
        figure,
        &state_strings,
        &|_| "west",
        pxu_provider,
        cache,
        settings,
        pb,
    )
}

fn fig_x_bound_state_region_min_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "x-bound-state-region-min-1",
        -4.0..2.5,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(-0.04492676714509915,-0.023287148957676335),xp:(-2.2982685996303633,1.7011141634148028),xm:(-2.3162023933609586,0.8583601532032655),u:(-3.4154076535523155,4.000100793457268),sheet_data:(log_branch_p:-1,log_branch_m:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.0564778288751243,-0.010296000935336903),xp:(-2.316202393360959,0.8583601532032651),xm:(-2.3153985683471108,0.00008710430978264849),u:(-3.4154076535523163,3.0001007934572677),sheet_data:(log_branch_p:-1,log_branch_m:-3,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.056479445909146386,0.01029221421273873),xp:(-2.315398568347111,0.00008710430978253747),xm:(-2.3162031403629046,-0.8581889963326543),u:(-3.4154076535523172,2.000100793457267),sheet_data:(log_branch_p:-1,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04492931592095178,0.023285635921691496),xp:(-2.316203140362906,-0.8581889963326539),xm:(-2.298275528949721,-1.7009447564270626),u:(-3.415407653552319,1.000100793457268),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)",
    ];

    draw_x_bound_state_figure(
        figure,
        &state_strings,
        &|_| "west",
        pxu_provider,
        cache,
        settings,
        pb,
    )
}

fn fig_x_bound_state_region_min_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "x-bound-state-region-min-2",
        -0.9..0.4,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(-1.4606821908812262,-0.08552402227919431),xp:(-0.036494412912998445,0.3868862252151071),xm:(-0.034602130895845726,-0.2244039105108243),u:(0.47400377737283,6.000100042285478),sheet_data:(log_branch_p:-2,log_branch_m:0,e_branch:1,u_branch:(Inside,Inside),im_x_sign:(1,1))),(p:(-0.0024712590245176227,0.03841793097115144),xp:(-0.03460213089584572,-0.22440391051082456),xm:(-0.03960815630989887,-0.28631872432272015),u:(0.4740037773728304,5.000100042285471),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,1))),(p:(-0.006907346397911845,0.047095708971704085),xp:(-0.039608156309898904,-0.28631872432272),xm:(-0.036497086475895155,-0.38686051106138636),u:(0.4740037773728296,4.000100042285474),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(-1,1)))],unlocked:false)",
    ];

    draw_x_bound_state_figure(
        figure,
        &state_strings,
        &|i| if i == 1 { "south" } else { "east" },
        pxu_provider,
        cache,
        settings,
        pb,
    )
}

fn fig_x_singlet_region_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let mut figure = FigureWriter::new(
        "x-singlet-region-0",
        -4.5..6.5,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.035920572686227975,-0.0371245201982526),xp:(3.278541909565751,2.69764230683293),xm:(3.0086748709958817,1.501168090727413),u:(2.3098001480095305,1.5000993687596509),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.0736477003995048,-0.031881014951510876),xp:(3.0086748709958773,1.5011680907274152),xm:(2.752022495646597,0.00017167978252885518),u:(2.3098001480095274,0.5000993687596516),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.07365802450198924,0.031873014242525234),xp:(2.7520224956465924,0.00017167978252619065),xm:(3.008613535972122,-1.500912421713252),u:(2.3098001480095243,-0.49990063124035),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(0.035924674842931,0.03712580047228859),xp:(3.0086135359721218,-1.5009124217132535),xm:(3.2784955205790927,-2.6974165274435005),u:(2.309800148009524,-1.4999006312403511),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-1.2191509724306528,0.000006720434949787522),xp:(3.278495520579101,-2.697416527443499),xm:(3.2785419095657513,2.697642306832927),u:(2.309800148009531,2.500099368759649),sheet_data:(log_branch_p:-1,log_branch_m:0,e_branch:-1,u_branch:(Outside,Outside),im_x_sign:(1,-1)))],unlocked:true)",
        "(points:[(p:(-0.04915040522405487,-0.045791051935815626),xp:(-1.3220716930339478,1.6552562481272564),xm:(-1.3219227444059347,0.8813162555256742),u:(-2.214036050469592,4.000101180615412),sheet_data:(log_branch_p:-1,log_branch_m:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09357322668831639,-0.03991326998630673),xp:(-1.321922744405919,0.8813162555256757),xm:(-1.2363694671632584,0.00010225956113174561),u:(-2.214036050469572,3.000101180615414),sheet_data:(log_branch_p:-1,log_branch_m:-3,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.09358689247514664,0.03990349663451138),xp:(-1.2363694671632492,0.00010225956111992174),xm:(-1.3219116746778858,-0.8811569763752188),u:(-2.214036050469563,2.000101180615402),sheet_data:(log_branch_p:-1,log_branch_m:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.049155153779756815,0.045792040962502355),xp:(-1.3219116746778863,-0.8811569763752252),xm:(-1.322081015696217,-1.6550991615231962),u:(-2.214036050469563,1.0001011806153943),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7145343218327235,0.000008784325108582892),xp:(-1.3220810156962146,-1.6550991615231967),xm:(-1.3220716930339236,1.6552562481272393),u:(-2.2140360504695593,0.00010118061539343692),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1)))],unlocked:false)",
    ];

    let states: Vec<pxu::State> = state_strings
        .iter()
        .map(|s| ron::from_str(s).map_err(|_| error("Could not load state")))
        .collect::<Result<Vec<_>>>()?;

    figure.component_indicator(r"x^{\pm}");
    figure.add_grid_lines(&contours, &[])?;

    for cut in contours
        .get_visible_cuts_from_point(&states[0].points[0], figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(Component::Xp) | CutType::UShortKidney(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["Black"], consts)?;
    }

    let colors = ["Blue", "Red"];
    let marks = ["*", "o"];
    let anchors = ["west", "east"];
    for (state, color, mark, anchor) in izip!(states, colors, marks, anchors) {
        let points = state
            .points
            .iter()
            .map(|pt| pt.get(Component::Xp))
            .collect::<Vec<_>>();
        // points.push(state.points.last().unwrap().get(Component::Xm));

        for (i, pos) in points.iter().enumerate() {
            let text = if i == 0 {
                "$\\scriptstyle \\bar{x}^- = x_1^+$".to_owned()
            } else if i == points.len() - 1 {
                format!("$\\scriptstyle x_{}^- = \\bar{{x}}^+$", i)
            } else {
                format!("$\\scriptstyle x_{}^- = x_{}^+$", i, i + 1)
            };
            let anchor = &format!("anchor={anchor}");
            figure.add_node(&text, *pos, &[anchor])?;
        }

        figure.add_plot_all(
            &[
                "only marks",
                color,
                &format!("mark={mark}"),
                "mark size=0.065cm",
            ],
            points,
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xp_two_particle_bs_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-two-particle-bs-0",
        -2.2..4.8,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
    ];

    draw_state_figure(
        figure,
        &state_strings,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_two_particle_bs_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-two-particle-bs-0",
        -2.2..4.8,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",

    ];

    draw_state_figure(
        figure,
        &state_strings,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_u_two_particle_bs_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let figure = FigureWriter::new(
        "u-two-particle-bs-0",
        -2.2..4.8,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",

    ];

    draw_state_figure(
        figure,
        &state_strings,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn fig_u_bs_1_4_same_energy(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let mut figure = FigureWriter::new(
        "u-bs-1-4-same-energy",
        -5.4..5.4,
        -2.5,
        Size {
            width: 8.0,
            height: 8.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(-0.49983924627304077,0.0),xp:(-0.0003500468127455447,0.693130751982731),xm:(-0.0003500468127455447,-0.693130751982731),u:(0.29060181708478217,-2.5000000000000004),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1)))])",
        "(points:[(p:(-0.026983887446552304,-0.06765648924444852),xp:(0.0020605469306089613,1.4422316508357205),xm:(-0.15775354460012647,0.929504024735109),u:(-0.2883557081916778,-0.9999998836405168),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.022627338608906006,-0.07099139905503385),xp:(-0.15775354460012575,0.9295040247351102),xm:(-0.18427779175410938,0.5747099285634751),u:(-0.2883557081916768,-1.999999883640514),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.42385965588804475,0.07099138281105592),xp:(-0.18427779175410947,0.5747099285634747),xm:(-0.15775356577239247,-0.9295039235403522),u:(-0.2883557081916773,-2.9999998836405153),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.026983888159841367,0.06765649025461998),xp:(-0.15775356577239286,-0.9295039235403516),xm:(0.0020604953634236894,-1.4422315128632799),u:(-0.28835570819167794,-3.9999998836405135),sheet_data:(log_branch_p:1,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1)))])",
    ];

    figure.set_caption("A single particle state and a four particle bound state with the same total energy and momentum and opposite charge.");

    draw_state_figure(
        figure,
        &state_strings,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
    )
}

fn draw_p_region_plot(
    mut figure: FigureWriter,
    e_branch: i32,
    pxu_provider: Arc<PxuProvider>,
    consts: CouplingConstants,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);
    // We first extract the contours below assuming that e_branch == +1

    let mut xp_scallion_path = {
        let mut xp_scallions = contours
            .get_visible_cuts_from_point(&pt, Component::P, consts)
            .filter(|cut| matches!(cut.typ, CutType::UShortScallion(Component::Xp)))
            .map(|cut| cut.path.clone())
            .collect::<Vec<_>>();

        for path in xp_scallions.iter_mut() {
            if path.first().unwrap().re > path.last().unwrap().re {
                path.reverse();
            }
        }

        xp_scallions.sort_by_key(|path| (path.first().unwrap().re * 1000.0).round() as i64);

        let mut left_paths = vec![];
        let mut right_path = vec![];

        for path in xp_scallions {
            if path.first().unwrap().re < -0.5 {
                left_paths.push(path);
            } else {
                right_path.extend(path);
            }
        }

        let min_1_path = left_paths.pop().unwrap();

        let mut e_cuts = contours
            .get_visible_cuts_from_point(&pt, Component::P, consts)
            .filter(|cut| {
                matches!(cut.typ, CutType::E)
                    && cut.path[0].im < 0.0
                    && (-1.0..0.0).contains(&cut.path[0].re)
            })
            .map(|cut| cut.path.clone())
            .collect::<Vec<_>>();

        for path in e_cuts.iter_mut() {
            if path.first().unwrap().im > path.last().unwrap().im {
                path.reverse();
            }
        }

        e_cuts.sort_by_key(|path| (path[0].re * 1000.0) as i32);

        let e_cut_0 = e_cuts
            .pop()
            .unwrap()
            .into_iter()
            .filter(|z| z.im < right_path[0].im)
            .collect::<Vec<_>>();

        let e_cut_min_1 = e_cuts
            .pop()
            .unwrap()
            .into_iter()
            .filter(|z| z.im < min_1_path[0].im)
            .collect::<Vec<_>>();

        let mut full_path = vec![];

        for path in left_paths {
            full_path.extend(path);
        }

        full_path.extend(e_cut_min_1);
        full_path.extend(min_1_path);
        full_path.extend(e_cut_0);
        full_path.extend(right_path);

        full_path
    };

    let mut xp_kidney_path = {
        let mut xp_kidneys = contours
            .get_visible_cuts_from_point(&pt, Component::P, consts)
            .filter(|cut| matches!(cut.typ, CutType::UShortKidney(Component::Xp)))
            .map(|cut| cut.path.clone())
            .filter(|path| path[0].re > 0.0 || path[0].im < 0.2)
            .collect::<Vec<_>>();

        for path in xp_kidneys.iter_mut() {
            if path.first().unwrap().re > path.last().unwrap().re {
                path.reverse();
            }
        }

        xp_kidneys.sort_by_key(|path| (path.first().unwrap().re * 1000.0).round() as i64);

        let mut left_path = vec![];
        let mut right_path = vec![];

        for path in xp_kidneys {
            if path.first().unwrap().re < -0.5 {
                left_path.extend(path);
            } else {
                right_path.extend(path);
            }
        }

        let mut e_cut = contours
            .get_visible_cuts_from_point(&pt, Component::P, consts)
            .filter(|cut| {
                matches!(cut.typ, CutType::E)
                    && cut.path[0].im > 0.0
                    && (-0.5..0.0).contains(&cut.path[0].re)
            })
            .map(|cut| cut.path.clone())
            .next()
            .unwrap();

        if e_cut.first().unwrap().im > e_cut.last().unwrap().im {
            e_cut.reverse();
        }

        let e_cut = e_cut
            .into_iter()
            .filter(|z| z.im > left_path.last().unwrap().im)
            .collect::<Vec<_>>();

        let mut full_path = vec![];

        full_path.extend(left_path);
        full_path.extend(e_cut);
        full_path.extend(right_path);

        full_path
    };

    if e_branch < 0 {
        (xp_scallion_path, xp_kidney_path) = (
            xp_kidney_path.into_iter().map(|z| z.conj()).collect(),
            xp_scallion_path.into_iter().map(|z| z.conj()).collect(),
        );
    }

    let mut xp_between_path = xp_scallion_path;
    xp_between_path.extend(xp_kidney_path.iter().rev());

    let x0 = xp_kidney_path.first().unwrap().re;
    let x1 = xp_kidney_path.last().unwrap().re;

    xp_kidney_path.push(Complex64::new(x1, 4.0));
    xp_kidney_path.push(Complex64::new(x0, 4.0));

    figure.add_plot_all(
        &["fill=Green", "opacity=0.3", "draw=none"],
        xp_kidney_path.iter().map(|z| z.conj()).collect(),
    )?;
    figure.add_plot_all(&["fill=Red", "opacity=0.3", "draw=none"], xp_kidney_path)?;

    figure.add_plot_all(
        &[
            "pattern color=Green",
            "pattern=north east lines",
            "draw=none",
        ],
        xp_between_path.iter().map(|z| z.conj()).collect(),
    )?;
    figure.add_plot_all(
        &["pattern color=Red", "pattern=north west lines", "draw=none"],
        xp_between_path,
    )?;

    pt.sheet_data.e_branch = e_branch;
    figure.add_cuts(&contours, &pt, consts, &[])?;

    let mut node = |text1: &str, text2: &str, x: f64, y: f64| -> Result<()> {
        let options = &[
            "anchor=mid",
            "fill=white",
            "fill opacity=0.75",
            "text opacity=1",
            "outer sep=0pt",
            "inner sep=2pt",
            "rounded corners",
        ];
        figure.add_node(
            &format!("\\tiny\\sffamily {text1}/{text2}"),
            Complex64::new(x, y),
            options,
        )?;
        if text1 != text2 {
            figure.add_node(
                &format!("\\tiny\\sffamily {text2}/{text1}"),
                Complex64::new(x, -y),
                options,
            )?;
        }
        Ok(())
    };

    if e_branch > 0 {
        node("Outside", "Outside", 0.29, 0.0)?;
        node("Between", "Between", -0.37, 0.0)?;
        node("Inside", "Inside", -1.35, 0.0)?;
        node("Between", "Outside", 1.6, 0.33)?;
        node("Inside", "Between", -1.6, 0.28)?;
        node("Inside", "Outside", -0.6, 0.5)?;
    } else {
        node("Inside", "Inside", 0.25, 0.0)?;
        node("Between", "Between", -0.37, 0.0)?;
        node("Outside", "Outside", -1.35, 0.0)?;
        node("Inside", "Between", 1.6, 0.33)?;
        node("Between", "Outside", -1.6, 0.28)?;
        node("Inside", "Outside", -0.6, 0.5)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_short_cut_regions_e_plus(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "p-short-cut-regions-e-plus",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    draw_p_region_plot(figure, 1, pxu_provider, consts, cache, settings, pb)
}

fn fig_p_short_cut_regions_e_min(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "p-short-cut-regions-e-min",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    draw_p_region_plot(figure, -1, pxu_provider, consts, cache, settings, pb)
}

fn get_physical_region(consts: CouplingConstants) -> Vec<Vec<Complex64>> {
    let mut physical_region = vec![];

    for p_start in [-3, -2, 1, 2] {
        let p_start = p_start as f64;
        let p0 = p_start + 1.0 / 16.0;
        let mut p_int = PInterpolatorMut::xp(p0, consts);
        p_int.goto_m(2.0);

        let line = p_int.contour();

        let mut full_line = line.clone();
        full_line.extend(line.into_iter().rev().map(|z| z.conj()));

        physical_region.push(full_line);
    }

    {
        let mut line = vec![];

        let p_start = 0.0;
        let p0 = p_start + 1.0 / 16.0;

        let mut p_int = PInterpolatorMut::xp(p0, consts);
        p_int.goto_conj();
        p_int.goto_m(0.0);
        line.extend(p_int.contour().iter().rev());

        let mut p_int = PInterpolatorMut::xp(p0, consts);
        p_int.goto_m(0.0);
        line.extend(p_int.contour());

        let mut full_line = line.clone();
        full_line.extend(line.into_iter().rev().map(|z| z.conj()));

        physical_region.push(full_line);
    }

    {
        let mut line = vec![];

        let p_start = -1.0;
        let p0 = p_start + 1.0 / 16.0;
        let p2 = p_start + 15.0 / 16.0;

        let mut p_int = PInterpolatorMut::xp(p2, consts);
        p_int.goto_m(consts.k() as f64);
        line.extend(p_int.contour().iter().rev().map(|z| z.conj()));

        let mut p_int = PInterpolatorMut::xp(p2, consts);
        p_int.goto_conj().goto_m(0.0);
        line.extend(p_int.contour().iter());

        let mut p_int = PInterpolatorMut::xp(p0, consts);
        p_int.goto_m(0.0);
        line.extend(p_int.contour().iter().rev());

        let mut full_line = line.clone();
        full_line.extend(line.into_iter().rev().map(|z| z.conj()));

        physical_region.push(full_line);
    }

    physical_region
}

fn get_crossed_region(consts: CouplingConstants) -> Vec<Vec<Complex64>> {
    let mut crossed_region = vec![];

    {
        let mut line: Vec<Complex64> = vec![];

        let p_start = 0.0;
        let p0 = p_start + 1.0 / 16.0;

        let mut p_int = PInterpolatorMut::xp(p0, consts);
        p_int.goto_conj();
        p_int.goto_m(0.0);
        line.extend(p_int.contour().iter().rev());

        let mut p_int = PInterpolatorMut::xp(p0, consts);
        p_int.goto_m(-1.0).goto_im(0.0);
        let im_z = line.last().unwrap().im;
        line.extend(p_int.contour().into_iter().filter(|z| z.im < im_z));

        crossed_region.push(line.iter().map(|z| z.conj()).collect());
        crossed_region.push(line);
    }

    {
        let mut line = vec![];

        let p_start = -1.0;
        let p2 = p_start + 15.0 / 16.0;

        let mut p_int = PInterpolatorMut::xp(p2, consts);
        p_int.goto_m(consts.k() as f64);
        line.extend(p_int.contour().iter().rev().map(|z| z.conj()));

        let mut p_int = PInterpolatorMut::xp(p2, consts);
        p_int.goto_conj().goto_m(0.0);
        line.extend(p_int.contour().iter());

        let mut p_int = PInterpolatorMut::xp(p2, consts);
        p_int.goto_im(0.0);
        let im_z = line.last().unwrap().im;
        line.extend(p_int.contour().iter().rev().filter(|z| z.im < im_z));

        crossed_region.push(line.iter().map(|z| z.conj()).collect());
        crossed_region.push(line);
    }

    crossed_region
}

fn fig_p_physical_region_e_plus(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "p-physical-region-e-plus",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 4.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    let physical_region = get_physical_region(consts);
    let crossed_region = get_crossed_region(consts);

    for region in physical_region {
        figure.add_plot_all(&["draw=none", "fill=Blue", "opacity=0.5"], region)?;
    }

    for region in crossed_region {
        figure.add_plot_all(&["draw=none", "fill=Red", "opacity=0.5"], region)?;
    }

    figure.add_cuts(&contours, &pt, consts, &[])?;

    figure.finish(cache, settings, pb)
}

fn fig_p_physical_region_e_minus(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "p-physical-region-e-min",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 4.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    let crossed_region = get_physical_region(consts);
    let physical_region = get_crossed_region(consts);

    for region in physical_region {
        figure.add_plot_all(&["draw=none", "fill=Blue", "opacity=0.5"], region)?;
    }

    for region in crossed_region {
        figure.add_plot_all(&["draw=none", "fill=Red", "opacity=0.5"], region)?;
    }

    pt.sheet_data.e_branch = -1;

    figure.add_cuts(&contours, &pt, consts, &[])?;

    figure.finish(cache, settings, pb)
}

#[allow(clippy::too_many_arguments)]
fn draw_singlet(
    mut figure: FigureWriter,
    pxu_provider: Arc<PxuProvider>,
    consts: CouplingConstants,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
    state_string: &str,
    marked_indices: &[usize],
) -> Result<FigureCompiler> {
    let state = load_state(state_string)?;
    let pt = &state.points[0];
    let contours = pxu_provider.get_contours(consts)?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_cuts(&contours, pt, consts, &[])?;

    for (i, point) in state.points.into_iter().enumerate() {
        let color = if marked_indices.contains(&i) {
            "Black"
        } else {
            "Blue"
        };
        figure.add_point(&point, &[color, "mark size=0.075cm"])?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xp_singlet_41(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-singlet-41",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.06481769289200064,-0.04632014396084205),xp:(0.6773737156527935,0.24101679937073833),xm:(0.39355556208794307,0.3659765169104283),u:(2.2503158561824144,-0.9972640693939946),x:(0.5207960049771001,0.3382736317263967),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134065179824,-0.04287934452264521),xp:(0.3935555620861755,0.3659765169090202),xm:(0.22233500515739787,0.34507249230177073),u:(2.250315856189289,-1.997264069401408),x:(0.29603586257460585,0.36274180923791544),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216060976681002,0.042633420284661425),xp:(0.22233500515775476,0.34507249230145126),xm:(0.3923377926330045,-0.3660664539125623),u:(2.2503158561923926,-2.9972640693996655),x:(0.16710333623086243,0.3211911819475663),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.0645947551037885,0.04632338280244304),xp:(0.3923377926336257,-0.36606645391208686),xm:(0.6755998929977572,-0.24272408911183854),u:(2.2503158561943186,-3.9972640694026023),x:(0.5192267118211283,-0.33884808844761033),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.10930011368445881,0.00024268539559447655),xp:(0.6755998929977572,-0.2427240891118387),xm:(0.6773737156462706,0.24101679936958165),u:(2.2503158561943186,0.002735930597398628),x:(0.7857319077395628,-0.0016758790700285356),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[0, 1, 2, 3],
    )
}

fn fig_xm_singlet_41(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-singlet-41",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.06481769289200064,-0.04632014396084205),xp:(0.6773737156527935,0.24101679937073833),xm:(0.39355556208794307,0.3659765169104283),u:(2.2503158561824144,-0.9972640693939946),x:(0.5207960049771001,0.3382736317263967),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134065179824,-0.04287934452264521),xp:(0.3935555620861755,0.3659765169090202),xm:(0.22233500515739787,0.34507249230177073),u:(2.250315856189289,-1.997264069401408),x:(0.29603586257460585,0.36274180923791544),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216060976681002,0.042633420284661425),xp:(0.22233500515775476,0.34507249230145126),xm:(0.3923377926330045,-0.3660664539125623),u:(2.2503158561923926,-2.9972640693996655),x:(0.16710333623086243,0.3211911819475663),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.0645947551037885,0.04632338280244304),xp:(0.3923377926336257,-0.36606645391208686),xm:(0.6755998929977572,-0.24272408911183854),u:(2.2503158561943186,-3.9972640694026023),x:(0.5192267118211283,-0.33884808844761033),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.10930011368445881,0.00024268539559447655),xp:(0.6755998929977572,-0.2427240891118387),xm:(0.6773737156462706,0.24101679936958165),u:(2.2503158561943186,0.002735930597398628),x:(0.7857319077395628,-0.0016758790700285356),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[0, 1, 2, 3],
    )
}

fn fig_u_singlet_41(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "u-singlet-41",
        -3.1..4.6,
        -1.5,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let state_string ="(points:[(p:(-0.06481769289200064,-0.04632014396084205),xp:(0.6773737156527935,0.24101679937073833),xm:(0.39355556208794307,0.3659765169104283),u:(2.2503158561824144,-0.9972640693939946),x:(0.5207960049771001,0.3382736317263967),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134065179824,-0.04287934452264521),xp:(0.3935555620861755,0.3659765169090202),xm:(0.22233500515739787,0.34507249230177073),u:(2.250315856189289,-1.997264069401408),x:(0.29603586257460585,0.36274180923791544),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216060976681002,0.042633420284661425),xp:(0.22233500515775476,0.34507249230145126),xm:(0.3923377926330045,-0.3660664539125623),u:(2.2503158561923926,-2.9972640693996655),x:(0.16710333623086243,0.3211911819475663),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.0645947551037885,0.04632338280244304),xp:(0.3923377926336257,-0.36606645391208686),xm:(0.6755998929977572,-0.24272408911183854),u:(2.2503158561943186,-3.9972640694026023),x:(0.5192267118211283,-0.33884808844761033),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.10930011368445881,0.00024268539559447655),xp:(0.6755998929977572,-0.2427240891118387),xm:(0.6773737156462706,0.24101679936958165),u:(2.2503158561943186,0.002735930597398628),x:(0.7857319077395628,-0.0016758790700285356),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)";
    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[0, 1, 2, 3],
    )
}

fn fig_xp_singlet_32(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-singlet-32",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.0918635850967006,-0.037587502213391646),xp:(0.785884223705366,0.0000000000000002220446049250313),xm:(0.5200361660196523,0.3386309516954546),u:(2.2500748563450794,-0.5000000000000003),x:(0.6765622619422568,0.24195091368028965),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04931502967968751,-0.044946057622269636),xp:(0.5200361660196524,0.3386309516954545),xm:(0.29556714680693774,0.3627151161370183),u:(2.2500748563450794,-1.5),x:(0.392950187668455,0.36607556161166316),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7176427704472238,-0.000000000000000019937695239947602),xp:(0.2955671468069379,0.36271511613701846),xm:(0.29556714680693785,-0.3627151161370184),u:(2.2500748563450785,-2.499999999999999),x:(0.2219764434485283,0.34498404739256483),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931502967968751,0.044946057622269636),xp:(0.29556714680693774,-0.3627151161370183),xm:(0.5200361660196524,-0.3386309516954545),u:(2.2500748563450794,-3.4999999999999996),x:(0.39295018766845496,-0.36607556161166327),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09186358509670066,0.03758750221339164),xp:(0.5200361660196525,-0.33863095169545443),xm:(0.785884223705366,0.0000000000000003608224830031759),u:(2.2500748563450794,0.4999999999999998),x:(0.676562261942257,-0.2419509136802895),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[1, 2, 3],
    )
}

fn fig_xm_singlet_32(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-singlet-32",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.0918635850967006,-0.037587502213391646),xp:(0.785884223705366,0.0000000000000002220446049250313),xm:(0.5200361660196523,0.3386309516954546),u:(2.2500748563450794,-0.5000000000000003),x:(0.6765622619422568,0.24195091368028965),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04931502967968751,-0.044946057622269636),xp:(0.5200361660196524,0.3386309516954545),xm:(0.29556714680693774,0.3627151161370183),u:(2.2500748563450794,-1.5),x:(0.392950187668455,0.36607556161166316),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7176427704472238,-0.000000000000000019937695239947602),xp:(0.2955671468069379,0.36271511613701846),xm:(0.29556714680693785,-0.3627151161370184),u:(2.2500748563450785,-2.499999999999999),x:(0.2219764434485283,0.34498404739256483),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931502967968751,0.044946057622269636),xp:(0.29556714680693774,-0.3627151161370183),xm:(0.5200361660196524,-0.3386309516954545),u:(2.2500748563450794,-3.4999999999999996),x:(0.39295018766845496,-0.36607556161166327),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09186358509670066,0.03758750221339164),xp:(0.5200361660196525,-0.33863095169545443),xm:(0.785884223705366,0.0000000000000003608224830031759),u:(2.2500748563450794,0.4999999999999998),x:(0.676562261942257,-0.2419509136802895),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[1, 2, 3],
    )
}

fn fig_u_singlet_32(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "u-singlet-32",
        -3.1..4.6,
        -1.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.0918635850967006,-0.037587502213391646),xp:(0.785884223705366,0.0000000000000002220446049250313),xm:(0.5200361660196523,0.3386309516954546),u:(2.2500748563450794,-0.5000000000000003),x:(0.6765622619422568,0.24195091368028965),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04931502967968751,-0.044946057622269636),xp:(0.5200361660196524,0.3386309516954545),xm:(0.29556714680693774,0.3627151161370183),u:(2.2500748563450794,-1.5),x:(0.392950187668455,0.36607556161166316),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7176427704472238,-0.000000000000000019937695239947602),xp:(0.2955671468069379,0.36271511613701846),xm:(0.29556714680693785,-0.3627151161370184),u:(2.2500748563450785,-2.499999999999999),x:(0.2219764434485283,0.34498404739256483),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931502967968751,0.044946057622269636),xp:(0.29556714680693774,-0.3627151161370183),xm:(0.5200361660196524,-0.3386309516954545),u:(2.2500748563450794,-3.4999999999999996),x:(0.39295018766845496,-0.36607556161166327),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09186358509670066,0.03758750221339164),xp:(0.5200361660196525,-0.33863095169545443),xm:(0.785884223705366,0.0000000000000003608224830031759),u:(2.2500748563450794,0.4999999999999998),x:(0.676562261942257,-0.2419509136802895),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[1, 2, 3],
    )
}

fn fig_xp_singlet_23(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-singlet-23",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.064817690638922,-0.04632014058248584),xp:(0.6773736720447697,0.24101678917659286),xm:(0.39355554871074094,0.3659764991995006),u:(2.250315939687509,-0.9972641231359414),x:(0.5207959807194622,0.33827361344245904),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134011794477,-0.042879342951094745),xp:(0.39355554871074067,0.3659764991995013),xm:(0.22233500194749478,0.34507247933376406),u:(2.250315939687506,-1.9972641231359423),x:(0.2960358555274206,0.3627417937862914),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216061057006049,0.04263342355344563),xp:(0.22233500194749445,0.3450724793337641),xm:(0.3923378032288628,-0.3660664344918713),u:(2.2503159396875043,-2.9972641231359445),x:(0.16710333534746072,0.32119117129204844),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.06459475724215495,0.04632337938493029),xp:(0.39233780322886325,-0.36606643449187204),xm:(0.6755998845174871,-0.24272404535577444),u:(2.2503159396875008,1.0027358768640537),x:(0.5192267310835156,-0.3388480606808871),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.10930010734366312,0.00024268100631728482),xp:(0.6755998866881463,-0.2427240505990194),xm:(0.6773736772251796,0.2410167915569991),u:(2.2503159279047136,0.0027358814445184176),x:(0.7857318639819022,-0.0016758487182760083),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[1, 2],
    )
}

fn fig_xm_singlet_23(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-singlet-23",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.064817690638922,-0.04632014058248584),xp:(0.6773736720447697,0.24101678917659286),xm:(0.39355554871074094,0.3659764991995006),u:(2.250315939687509,-0.9972641231359414),x:(0.5207959807194622,0.33827361344245904),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134011794477,-0.042879342951094745),xp:(0.39355554871074067,0.3659764991995013),xm:(0.22233500194749478,0.34507247933376406),u:(2.250315939687506,-1.9972641231359423),x:(0.2960358555274206,0.3627417937862914),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216061057006049,0.04263342355344563),xp:(0.22233500194749445,0.3450724793337641),xm:(0.3923378032288628,-0.3660664344918713),u:(2.2503159396875043,-2.9972641231359445),x:(0.16710333534746072,0.32119117129204844),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.06459475724215495,0.04632337938493029),xp:(0.39233780322886325,-0.36606643449187204),xm:(0.6755998845174871,-0.24272404535577444),u:(2.2503159396875008,1.0027358768640537),x:(0.5192267310835156,-0.3388480606808871),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.10930010734366312,0.00024268100631728482),xp:(0.6755998866881463,-0.2427240505990194),xm:(0.6773736772251796,0.2410167915569991),u:(2.2503159279047136,0.0027358814445184176),x:(0.7857318639819022,-0.0016758487182760083),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[1, 2],
    )
}

fn fig_u_singlet_23(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "u-singlet-23",
        -3.1..4.6,
        -1.5,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.064817690638922,-0.04632014058248584),xp:(0.6773736720447697,0.24101678917659286),xm:(0.39355554871074094,0.3659764991995006),u:(2.250315939687509,-0.9972641231359414),x:(0.5207959807194622,0.33827361344245904),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134011794477,-0.042879342951094745),xp:(0.39355554871074067,0.3659764991995013),xm:(0.22233500194749478,0.34507247933376406),u:(2.250315939687506,-1.9972641231359423),x:(0.2960358555274206,0.3627417937862914),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216061057006049,0.04263342355344563),xp:(0.22233500194749445,0.3450724793337641),xm:(0.3923378032288628,-0.3660664344918713),u:(2.2503159396875043,-2.9972641231359445),x:(0.16710333534746072,0.32119117129204844),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.06459475724215495,0.04632337938493029),xp:(0.39233780322886325,-0.36606643449187204),xm:(0.6755998845174871,-0.24272404535577444),u:(2.2503159396875008,1.0027358768640537),x:(0.5192267310835156,-0.3388480606808871),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.10930010734366312,0.00024268100631728482),xp:(0.6755998866881463,-0.2427240505990194),xm:(0.6773736772251796,0.2410167915569991),u:(2.2503159279047136,0.0027358814445184176),x:(0.7857318639819022,-0.0016758487182760083),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[1, 2],
    )
}

fn fig_xp_singlet_14(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xp-singlet-14",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.09185221149636245,-0.037572722189714455),xp:(0.7857363886452503,0.0000004328254604446524),xm:(0.5200106363475369,0.3385618195950395),u:(2.2503161408013796,-0.5000007065959058),x:(0.676486747365414,0.24187289813934523),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.04931600633410893,-0.0449403973338789),xp:(0.5200106363475344,0.338561819595029),xm:(0.29557299472051746,0.3626743175215065),u:(2.2503161408014147,-1.5000007065959013),x:(0.392946068121917,0.36602187168832023),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.717663444470969,0.00000006054071687339567),xp:(0.2955729947205189,0.3626743175215076),xm:(0.2955732335644112,-0.36267435245574203),u:(2.2503161408014094,-2.500000706595892),x:(0.22198686543101423,0.3449533442179103),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931603946892371,0.044940403147529916),xp:(0.2955732335644095,-0.36267435245574087),xm:(0.5200110416414399,-0.3385616712335204),u:(2.2503161408014156,1.499999293404119),x:(0.392946382629357,-0.36602184846097735),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.09185229822963642,0.03757265583534658),xp:(0.5200110416414421,-0.33856167123353087),xm:(0.7857363886452495,0.00000043282544220923924),u:(2.250316140801381,0.4999992934041242),x:(0.6764872054840881,-0.24187245720745892),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[2],
    )
}

fn fig_xm_singlet_14(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "xm-singlet-14",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.09185221149636245,-0.037572722189714455),xp:(0.7857363886452503,0.0000004328254604446524),xm:(0.5200106363475369,0.3385618195950395),u:(2.2503161408013796,-0.5000007065959058),x:(0.676486747365414,0.24187289813934523),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.04931600633410893,-0.0449403973338789),xp:(0.5200106363475344,0.338561819595029),xm:(0.29557299472051746,0.3626743175215065),u:(2.2503161408014147,-1.5000007065959013),x:(0.392946068121917,0.36602187168832023),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.717663444470969,0.00000006054071687339567),xp:(0.2955729947205189,0.3626743175215076),xm:(0.2955732335644112,-0.36267435245574203),u:(2.2503161408014094,-2.500000706595892),x:(0.22198686543101423,0.3449533442179103),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931603946892371,0.044940403147529916),xp:(0.2955732335644095,-0.36267435245574087),xm:(0.5200110416414399,-0.3385616712335204),u:(2.2503161408014156,1.499999293404119),x:(0.392946382629357,-0.36602184846097735),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.09185229822963642,0.03757265583534658),xp:(0.5200110416414421,-0.33856167123353087),xm:(0.7857363886452495,0.00000043282544220923924),u:(2.250316140801381,0.4999992934041242),x:(0.6764872054840881,-0.24187245720745892),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[2],
    )
}

fn fig_u_singlet_14(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);

    let figure = FigureWriter::new(
        "u-singlet-14",
        -3.1..4.6,
        -1.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.09185221149636245,-0.037572722189714455),xp:(0.7857363886452503,0.0000004328254604446524),xm:(0.5200106363475369,0.3385618195950395),u:(2.2503161408013796,-0.5000007065959058),x:(0.676486747365414,0.24187289813934523),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.04931600633410893,-0.0449403973338789),xp:(0.5200106363475344,0.338561819595029),xm:(0.29557299472051746,0.3626743175215065),u:(2.2503161408014147,-1.5000007065959013),x:(0.392946068121917,0.36602187168832023),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.717663444470969,0.00000006054071687339567),xp:(0.2955729947205189,0.3626743175215076),xm:(0.2955732335644112,-0.36267435245574203),u:(2.2503161408014094,-2.500000706595892),x:(0.22198686543101423,0.3449533442179103),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931603946892371,0.044940403147529916),xp:(0.2955732335644095,-0.36267435245574087),xm:(0.5200110416414399,-0.3385616712335204),u:(2.2503161408014156,1.499999293404119),x:(0.392946382629357,-0.36602184846097735),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.09185229822963642,0.03757265583534658),xp:(0.5200110416414421,-0.33856167123353087),xm:(0.7857363886452495,0.00000043282544220923924),u:(2.250316140801381,0.4999992934041242),x:(0.6764872054840881,-0.24187245720745892),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu_provider,
        consts,
        cache,
        settings,
        pb,
        state_string,
        &[2],
    )
}

const BS_AXIS_OPTIONS: &[&str] = &[
    "axis x line=bottom",
    "axis y line=middle",
    "ytick=\\empty",
    "yticklabels=\\empty",
    "axis line style={->}",
    "xlabel={$p$}",
    "ylabel={$E$}",
    "axis line style={shorten >=-5pt, shorten <=-5pt}",
    "every axis x label/.style={at={(ticklabel* cs:1)},anchor=west,xshift=5pt}",
    "every axis y label/.style={at={(ticklabel* cs:1)},anchor=south,yshift=5pt}",
    "clip=false",
];

const BS_TICKS_2PI: &[&str] = &[
    "xtick={-4,-3,-2,-1,0,1,2,3,4}",
    r"xticklabels={$-8\pi$,$-6\pi$,$-4\pi$,$-2\pi$,$0$,$2\pi$,$4\pi$,$6\pi$,$8\pi$}",
];

const BS_TICKS_PI: &[&str] = &[
    "xtick={-3,-2.5,-2,-1.5,-1,-0.5,0,0.5,1,1.5,2,2.5,3}",
    r"xticklabels={$-6\pi$,$-5\pi$,$-4\pi$,$-3\pi$,$-2\pi$,$-\pi$,$0$,$\pi$,$2\pi$,$3\pi$,$4\pi$,$5\pi$,$6\pi$}",
];

fn fig_bs_disp_rel_large(
    _pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let width: f64 = 12.0;
    let height: f64 = 6.0;

    let x_min: f64 = -4.35;
    let x_max: f64 = 1.25;
    let y_min: f64 = 0.0;
    let y_max: f64 = (x_max - x_min).abs() * 8.0 * height / width;

    let x_range = x_min..x_max;
    let y_range = y_min..y_max;

    let restrict = format!("restrict y to domain={y_min:2}:{y_max:2}");
    let axis_options = [BS_AXIS_OPTIONS, BS_TICKS_2PI, &[&restrict]].concat();

    let mut figure = FigureWriter::custom_axis(
        "bs_disp_rel_large",
        x_range,
        y_range,
        Size { width, height },
        &axis_options,
        settings,
        pb,
    )?;

    let colors = ["Blue", "Red", "Green", "DarkViolet"];
    let mut color_it = colors.iter().cycle();

    let domain = format!("domain={x_min:.2}:{x_max:.2}");

    for m in 1..=43 {
        let mut plot = format!("{{ sqrt(({m} + 5 * x)^2+4*4*(sin(x*180))^2) }}");
        let mut options = vec![&domain, "mark=none", "samples=400"];
        if (m - 1) % 5 == 0 {
            plot.push_str(&format!(" node [pos=0,left,black] {{$\\scriptstyle {m}$}}"));
            options.extend(&[color_it.next().unwrap(), "thick"]);
            if m <= 16 {
                plot.push_str(&format!(
                    " node [pos=1,right,black] {{$\\scriptstyle {m}$}}"
                ));
                figure.add_plot_custom(&options, &plot)?;
            } else {
                options.extend(&["dashed"]);
                plot.push_str(&format!(
                    " node [pos=1,above,black] {{$\\scriptstyle {m}$}}"
                ));
                figure.add_plot_custom(&options, &plot)?;
            }
        } else {
            options.extend(&["thin", "gray"]);

            figure.add_plot_custom(&options, &plot)?;
        }
    }

    figure.finish(cache, settings, pb)
}

fn fig_bs_disp_rel_small(
    _pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let axis_options = [BS_AXIS_OPTIONS, BS_TICKS_PI].concat();

    let k = 5;

    let width: f64 = 12.0;
    let height: f64 = 4.5;

    let x_min: f64 = -1.75;
    let x_max: f64 = 0.75;
    let y_min: f64 = 0.0;
    let y_max: f64 = (x_max - x_min).abs() * 8.0 * height / width;

    let x_range = x_min..x_max;
    let y_range = y_min..y_max;

    let mut figure = FigureWriter::custom_axis(
        "bs_disp_rel_small",
        x_range,
        y_range,
        Size { width, height },
        &axis_options,
        settings,
        pb,
    )?;

    let colors = ["Blue", "Red", "Green", "DarkViolet", "DeepPink"];
    let mut color_it = colors.iter().cycle();

    let domain = format!("domain={x_min:.2}:{x_max:.2}");
    for m in 1..=(k - 1) {
        let plot = format!(
            "{{ sqrt(({m} + {k} * x)^2+4*4*(sin(x*180))^2) }} \
             node [pos=0,left,black] {{$\\scriptstyle {m}$}} \
             node [pos=1,right,black] {{$\\scriptstyle {m}$}}"
        );

        let options = [
            // "domain=-1.75:0.75",
            &domain,
            "mark=none",
            "samples=400",
            "thick",
            color_it.next().unwrap(),
        ];

        figure.add_plot_custom(&options, &plot)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_bs_disp_rel_lr(
    _pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let axis_options = [BS_AXIS_OPTIONS, BS_TICKS_2PI].concat();

    let width: f64 = 10.0;
    let height: f64 = 4.5;

    let x_min: f64 = -2.25;
    let x_max: f64 = 1.25;
    let y_min: f64 = 0.0;
    let y_max: f64 = (x_max - x_min).abs() * 8.0 * height / width;

    let x_range = x_min..x_max;
    let y_range = y_min..y_max;

    let mut figure = FigureWriter::custom_axis(
        "bs_disp_rel_lr",
        x_range,
        y_range,
        Size { width, height },
        &axis_options,
        settings,
        pb,
    )?;

    let colors = ["Blue", "Red", "Green", "DarkViolet", "DeepPink"];
    let mut color_it = colors.iter().cycle();

    let domain = format!("domain={x_min:.2}:{x_max:.2}");
    for (m, label) in [
        (4, r"X_{\mbox{\tiny L}}^{\pm}(p,k-1)"),
        (-1, r"X_{\mbox{\tiny R}}^{\pm}(p,1)"),
    ] {
        let plot = format!(
            "{{ sqrt(({m} + 5 * x)^2+4*4*(sin(x*180))^2) }} \
             node [pos=0,left,black] {{$\\scriptstyle {label}$}} \
             node [pos=1,right,black] {{$\\scriptstyle {label}$}}"
        );

        let options = [
            // "domain=-1.75:0.75",
            &domain,
            "mark=none",
            "samples=400",
            "thick",
            color_it.next().unwrap(),
        ];

        figure.add_plot_custom(&options, &plot)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_bs_disp_rel_lr0(
    _pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let width: f64 = 12.0;
    let height: f64 = 6.0;

    let x_min: f64 = -2.25;
    let x_max: f64 = 2.25;
    let y_min: f64 = 0.0;
    let y_max: f64 = (x_max - x_min).abs() * 8.0 * height / width;

    let x_range = x_min..x_max;
    let y_range = y_min..y_max;

    let restrict = format!("restrict y to domain={y_min:2}:{y_max:2}");
    let axis_options = [BS_AXIS_OPTIONS, BS_TICKS_2PI, &[&restrict]].concat();

    let mut figure = FigureWriter::custom_axis(
        "bs_disp_rel_lr0",
        x_range,
        y_range,
        Size { width, height },
        &axis_options,
        settings,
        pb,
    )?;

    let domain = format!("domain={x_min:.2}:{x_max:.2}");

    for m in 1..=29 {
        let plot = format!("{{ sqrt(({m} + 5 * x)^2+4*4*(sin(x*180))^2) }}");
        let options = [&domain, "mark=none", "samples=400", "LightSlateBlue"];

        figure.add_plot_custom(&options, &plot)?;
    }

    for m in -29..=-1 {
        let plot = format!("{{ sqrt(({m} + 5 * x)^2+4*4*(sin(x*180))^2) }}");
        let options = [
            "domain=-2.25:2.25",
            "mark=none",
            "samples=400",
            "LightCoral",
        ];

        figure.add_plot_custom(&options, &plot)?;
    }

    let plot = "{{ sqrt((5 * x)^2+4*4*(sin(x*180))^2) }}";
    let options = ["domain=-2.25:2.25", "mark=none", "samples=400", "Black"];

    figure.add_plot_custom(&options, plot)?;

    figure.finish(cache, settings, pb)
}

fn fig_u_region_min_3(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(-2.5, consts);
    pt.sheet_data.log_branch_p = -3;
    pt.sheet_data.log_branch_m = 0;

    let k = consts.k() as f64;
    let h = consts.h;

    let mut figure = FigureWriter::new(
        "u-region-min-3",
        -7.25..7.25,
        -0.0 * k / h - 6.0 * k / h,
        Size {
            width: 2.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis_origin(Complex64::new(0.0, -6.0 * k / h))?;
    figure.add_cuts(&contours, &pt, consts, &[])?;
    figure.add_point(&pt, &["Blue", "mark size=0.04cm"])?;

    figure.finish(cache, settings, pb)
}

fn fig_u_region_min_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(-1.5, consts);
    pt.sheet_data.log_branch_p = -2;
    pt.sheet_data.log_branch_m = 0;

    let k = consts.k() as f64;
    let h = consts.h;

    let mut figure = FigureWriter::new(
        "u-region-min-2",
        -7.25..7.25,
        -0.0 * k / h - 4.0 * k / h,
        Size {
            width: 2.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis_origin(Complex64::new(0.0, -4.0 * k / h))?;
    figure.add_cuts(&contours, &pt, consts, &[])?;
    figure.add_point(&pt, &["Blue", "mark size=0.04cm"])?;

    figure.finish(cache, settings, pb)
}

fn fig_u_region_min_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(-0.5, consts);
    pt.sheet_data.log_branch_p = -1;
    pt.sheet_data.log_branch_m = 0;

    let k = consts.k() as f64;
    let h = consts.h;

    let mut figure = FigureWriter::new(
        "u-region-min-1",
        -7.25..7.25,
        -0.0 * k / h - 2.0 * k / h,
        Size {
            width: 2.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis_origin(Complex64::new(0.0, -2.0 * k / h))?;
    figure.add_cuts(&contours, &pt, consts, &[])?;
    figure.add_point(&pt, &["Blue", "mark size=0.04cm"])?;

    figure.finish(cache, settings, pb)
}

fn fig_u_region_0(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let k = consts.k() as f64;
    let h = consts.h;

    let mut figure = FigureWriter::new(
        "u-region-0",
        -7.25..7.25,
        -0.0 * k / h,
        Size {
            width: 2.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;
    figure.add_cuts(&contours, &pt, consts, &[])?;
    figure.add_point(&pt, &["Blue", "mark size=0.04cm"])?;

    figure.finish(cache, settings, pb)
}

fn fig_u_region_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(1.5, consts);
    pt.sheet_data.log_branch_p = 1;
    pt.sheet_data.log_branch_m = 0;

    let k = consts.k() as f64;
    let h = consts.h;

    let mut figure = FigureWriter::new(
        "u-region-1",
        -7.25..7.25,
        -0.0 * k / h + 2.0 * k / h,
        Size {
            width: 2.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis_origin(Complex64::new(0.0, 2.0 * k / h))?;
    figure.add_cuts(&contours, &pt, consts, &[])?;
    figure.add_point(&pt, &["Blue", "mark size=0.04cm"])?;

    figure.finish(cache, settings, pb)
}

fn fig_u_region_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(2.5, consts);
    pt.sheet_data.log_branch_p = 2;
    pt.sheet_data.log_branch_m = 0;

    let k = consts.k() as f64;
    let h = consts.h;

    let mut figure = FigureWriter::new(
        "u-region-2",
        -7.25..7.25,
        -0.0 * k / h + 4.0 * k / h,
        Size {
            width: 2.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis_origin(Complex64::new(0.0, 4.0 * k / h))?;
    figure.add_cuts(&contours, &pt, consts, &[])?;
    figure.add_point(&pt, &["Blue", "mark size=0.04cm"])?;

    figure.finish(cache, settings, pb)
}

fn fig_u_region_min_1_h_01_k_5(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(0.1, 5);
    let contours = pxu_provider.get_contours(consts)?;

    let k = consts.k() as f64;
    let h = consts.h;

    let mut pt = pxu::Point::new(-1.0 / k, consts);
    pt.sheet_data.log_branch_p = -1;
    pt.sheet_data.log_branch_m = 0;

    let mut figure = FigureWriter::new(
        "u-region-min-1-h-01-k-5",
        -75.0..75.0,
        k / h,
        Size {
            width: 2.5,
            height: 5.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_axis()?;
    figure.add_cuts(&contours, &pt, consts, &[])?;

    pt.u += 2.0 * k / h * Complex64::i();
    figure.add_point(&pt, &["Blue", "mark size=0.05cm"])?;

    figure.finish(cache, settings, pb)
}

fn fig_p_plane_h_01_k_5(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(0.1, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(-1.0 / consts.k() as f64, consts);
    pt.sheet_data.log_branch_p = -1;
    pt.sheet_data.log_branch_m = 0;

    let mut figure = FigureWriter::new(
        "p-plane-h-01-k-5",
        -0.4..0.2,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;
    figure.add_cuts(&contours, &pt, consts, &[])?;
    figure.add_point(&pt, &["Blue", "mark size=0.05cm"])?;

    figure.finish(cache, settings, pb)
}

fn fig_u_region_min_1_h_0_k_5(
    _: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(0.0, 5);
    let mut figure = FigureWriter::custom_axis(
        "u-region-min-1-h-0-k-5",
        -2.0..2.0,
        -15.25..15.25,
        Size {
            width: 2.5,
            height: 5.0,
        },
        &["hide axis,scale only axis,ticks=none,clip,clip mode=individual"],
        settings,
        pb,
    )?;

    figure.component_indicator("u");

    figure.add_axis_origin(Complex64::new(0.0, 5.0))?;

    for y in (-14..=14).map(|n| n as f64) {
        figure.add_curve(
            &["very thin", "lightgray"],
            &[Complex64::new(-2.0, y), Complex64::new(2.0, y)],
        )?;
    }

    for y in [-12.5, -7.5, -2.5, 2.5, 7.5, 12.5] {
        let cut = pxu::Cut::new(
            Component::U,
            vec![Complex64::new(-2.0, y), Complex64::new(2.0, y)],
            Some(Complex64::new(0.0, y)),
            CutType::E,
            -1,
            false,
            vec![],
        );
        figure.add_cut(&cut, &[], consts)?;
    }

    figure.add_plot_all(
        &["Blue", "only marks", "mark size=0.05cm"],
        vec![Complex64::zero()],
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_p_region_min_1_h_0_k_5(
    _: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(0.0, 5);
    let mut figure = FigureWriter::custom_axis(
        "p-region-min-1-h-0-k-5",
        -2.0..2.0,
        -2.8..2.8,
        Size {
            width: 2.5,
            height: 5.0,
        },
        &["hide axis,scale only axis,ticks=none,clip,clip mode=individual"],
        settings,
        pb,
    )?;

    figure.component_indicator("p");

    for y in [-1.40496, -0.868315, 0.0, 0.868315, 1.40496] {
        figure.add_curve(
            &["very thin", "lightgray"],
            &[Complex64::new(-2.0, y), Complex64::new(2.0, y)],
        )?;
    }

    for y in [-1.47727, 1.47727] {
        let cut = pxu::Cut::new(
            Component::P,
            vec![
                Complex64::new(0.0, y),
                Complex64::new(0.0, 2.8 * y.signum()),
            ],
            Some(Complex64::new(0.0, y)),
            CutType::E,
            -1,
            false,
            vec![],
        );
        figure.add_cut(&cut, &[], consts)?;
    }

    figure.add_plot_all(
        &["Blue", "only marks", "mark size=0.05cm"],
        vec![Complex64::zero()],
    )?;

    figure.add_node(
        r"$\scriptscriptstyle -\frac{2\pi}{k}$",
        Complex64::zero(),
        &["anchor=north"],
    )?;

    figure.finish(cache, settings, pb)
}

fn fig_p_plane_path_between_regions(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "p-plane-path-between-region",
        -2.6..3.6,
        0.0,
        Size {
            width: 15.5,
            height: 5.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    for cut in contours
        .get_visible_cuts_from_point(&pt, Component::P, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::E
                    | CutType::Log(_)
                    | CutType::UShortKidney(_)
                    | CutType::UShortScallion(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    let paths = [
        ("p from region 0 to region -1", 0.45),
        ("p from region -1 to region -2", 0.6),
        ("p from region -2 to region -3", 0.6),
        ("p from region 0 to region +1", 0.6),
        ("p from region +1 to region +2", 0.6),
        ("p from region +2 to region +3", 0.6),
    ];

    for (path_name, pos) in paths {
        let path = pxu_provider.get_path(path_name)?;
        let mut path = (*path).clone();

        figure.add_path(&path, &pt, &["solid"])?;
        figure.add_path_arrows(&path, &[pos], &["very thick", "Blue"])?;

        for segs in path.segments.iter_mut() {
            for seg in segs.iter_mut() {
                for p in seg.p.iter_mut() {
                    *p = p.conj();
                }
            }
        }

        figure.add_path(&path, &pt, &["solid"])?;
        figure.add_path_arrows(&path, &[pos], &["very thick", "Blue"])?;
    }

    let centers = [-2.5, -1.5, -0.5, 0.5, 1.5, 2.5, 3.5]
        .map(Complex64::from)
        .to_vec();

    figure.add_plot_all(&["only marks", "Blue", "mark size=0.05cm"], centers)?;

    for (x, y, label) in [
        (3.2, 0.12, "3"),
        (2.2, 0.12, "2"),
        (1.1, 0.13, "1"),
        (0.2, 0.12, "-1"),
        (-0.8, -0.1, "-2"),
        (-2.0, -0.13, "-3"),
    ] {
        let (anchor, anchor_prime) = if y > 0.0 {
            ("anchor=south", "anchor=north")
        } else {
            ("anchor=north", "anchor=south")
        };
        figure.add_node(
            &format!(r"$\scriptstyle {label}$"),
            Complex64::new(x, y),
            &["Blue", anchor],
        )?;
        figure.add_node(
            &format!(r"$\scriptstyle {label}'$"),
            Complex64::new(x, -y),
            &["Blue", anchor_prime],
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_periodic_path(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(1.0, 7);
    let contours = pxu_provider.get_contours(consts)?;
    let pt = pxu::Point::new(0.5, consts);

    let mut figure = FigureWriter::new(
        "p-periodic-path",
        -0.35..0.19,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    let path_names = [
        ("p period 1", "Red"),
        ("p period 2", "Green"),
        ("p period 3", "Blue"),
        ("p period 4", "Orange"),
    ];

    for (path_name, color) in path_names {
        let path = pxu_provider.get_path(path_name)?;
        figure.add_path(&path, &pt, &[color])?;
        figure.add_path_arrows(&path, &[0.55], &[color, "very thick"])?;
    }

    figure.add_cuts(&contours, &pt, consts, &[])?;

    figure.finish(cache, settings, pb)
}

fn fig_xp_periodic_path(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(1.0, 7);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);
    pt.sheet_data.u_branch = (UBranch::Between, UBranch::Between);

    let mut figure = FigureWriter::new(
        "xp-periodic-path",
        -2.0..0.8,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    let path_names = [
        ("p period 1", "Red"),
        ("p period 2", "Green"),
        ("p period 3", "Blue"),
        ("p period 4", "Orange"),
    ];

    for (path_name, color) in path_names {
        let path = pxu_provider.get_path(path_name)?;
        figure.add_path(&path, &pt, &[color])?;
        figure.add_path_arrows(&path, &[0.55], &[color, "very thick"])?;
    }

    let cuts = contours
        .get_visible_cuts_from_point(&pt, figure.component, consts)
        .filter(|cut: &&Cut| -> bool {
            match cut.typ {
                CutType::UShortKidney(comp) => comp == figure.component || cut.p_range == -1,
                CutType::UShortScallion(comp) => comp == figure.component || cut.p_range == 0,
                CutType::E => true,
                _ => false,
            }
        })
        .collect::<Vec<_>>();

    for cut in cuts {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xm_periodic_path(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(1.0, 7);
    let contours = pxu_provider.get_contours(consts)?;
    let mut pt = pxu::Point::new(0.5, consts);
    pt.sheet_data.u_branch = (UBranch::Between, UBranch::Between);

    let mut figure = FigureWriter::new(
        "xm-periodic-path",
        -2.0..0.8,
        0.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        Component::Xm,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&contours, &[])?;

    let path_names = [
        ("p period 1", "Red"),
        ("p period 2", "Green"),
        ("p period 3", "Blue"),
        ("p period 4", "Orange"),
    ];

    for (path_name, color) in path_names {
        let path = pxu_provider.get_path(path_name)?;
        figure.add_path(&path, &pt, &[color])?;
        figure.add_path_arrows(&path, &[0.55], &[color, "very thick"])?;
    }

    let cuts = contours
        .get_visible_cuts_from_point(&pt, figure.component, consts)
        .filter(|cut: &&Cut| -> bool {
            match cut.typ {
                CutType::UShortKidney(comp) => comp == figure.component || cut.p_range == -1,
                CutType::UShortScallion(comp) => comp == figure.component || cut.p_range == 0,
                CutType::E => true,
                _ => false,
            }
        })
        .collect::<Vec<_>>();

    for cut in cuts {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_simple_path_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "u simple path 1";
    let mut figure = FigureWriter::new(
        "u-simple-path-1",
        -5.2..5.2,
        1.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let path = &pxu_provider.get_path(pathname)?;
    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;
    figure.add_path(path, pt, &[])?;
    figure.add_path_start_mark(path, &["Blue", "very thick"])?;
    figure.add_path_arrows(path, &[0.55], &["Blue", "very thick"])?;
    figure.add_node("1", Complex64::new(1.0, -1.1), &["anchor=north", "Blue"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_simple_path_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "u simple path 2";
    let mut figure = FigureWriter::new(
        "u-simple-path-2",
        -5.2..5.2,
        1.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let path = &pxu_provider.get_path(pathname)?;
    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;
    figure.add_path(path, pt, &["DarkOrchid", "very thick"])?;
    figure.add_path_arrows(path, &[0.75], &["DarkOrchid", "very thick"])?;
    figure.add_node(
        "2",
        Complex64::new(-2.0, 0.0),
        &["anchor=east", "DarkOrchid"],
    )?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_simple_path_34(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathnames = ["u simple path 3", "u simple path 4"];
    let mut figure = FigureWriter::new(
        "u-simple-path-34",
        -5.2..5.2,
        1.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let paths = pathnames
        .iter()
        .map(|pathname| pxu_provider.get_path(pathname))
        .collect::<Result<Vec<_>>>()?;

    let state = pxu_provider.get_start(pathnames[0])?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;

    figure.add_path(&paths[0], pt, &["DarkCyan"])?;
    figure.add_path(&paths[1], pt, &["FireBrick"])?;

    figure.add_path_arrows(&paths[0], &[0.55], &["DarkCyan", "very thick"])?;

    figure.add_path_end_mark(&paths[1], &["FireBrick", "very thick"])?;

    figure.add_node(
        "3",
        Complex64::new(-2.8, 1.25),
        &["anchor=east", "DarkCyan"],
    )?;
    figure.add_node(
        "4",
        Complex64::new(-3.0, 2.25),
        &["anchor=east", "FireBrick"],
    )?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_simple_path(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathnames = [
        "u simple path 1",
        "u simple path 2",
        "u simple path 3",
        "u simple path 4",
    ];

    let mut figure = FigureWriter::new(
        "x-simple-path",
        -2.9..4.6,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;
    figure.component_indicator(r"x^{\pm}");

    let xp_paths = pathnames
        .iter()
        .map(|pathname| pxu_provider.get_path(pathname))
        .collect::<Result<Vec<_>>>()?;

    let xm_paths = xp_paths
        .iter()
        .map(|xp_path| {
            let mut xm_path = (**xp_path).clone();
            xm_path.swap_xp_xm();
            xm_path
        })
        .collect::<Vec<_>>();

    let state = pxu_provider.get_start(pathnames[0])?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;

    figure.add_path_start_mark(&xp_paths[0], &["Blue", "very thick"])?;
    figure.add_path_start_mark(&xm_paths[0], &["Blue", "very thick"])?;
    figure.add_path_end_mark(&xp_paths[3], &["FireBrick", "very thick"])?;
    figure.add_path_end_mark(&xm_paths[3], &["FireBrick", "very thick"])?;

    figure.add_path(&xp_paths[0], pt, &["Blue", "solid"])?;
    figure.add_path(&xp_paths[1], pt, &["DarkOrchid", "solid"])?;
    figure.add_path(&xp_paths[2], pt, &["DarkCyan", "solid"])?;
    figure.add_path(&xp_paths[3], pt, &["FireBrick", "solid"])?;

    figure.add_path(&xm_paths[0], pt, &["Blue", "solid"])?;
    figure.add_path(&xm_paths[1], pt, &["DarkOrchid", "solid"])?;
    figure.add_path(&xm_paths[2], pt, &["DarkCyan", "solid"])?;
    figure.add_path(&xm_paths[3], pt, &["FireBrick", "solid"])?;

    figure.add_path_arrows(&xp_paths[0], &[0.75], &["Blue", "very thick"])?;
    figure.add_path_arrows(&xp_paths[1], &[0.75], &["DarkOrchid", "very thick"])?;
    figure.add_path_arrows(&xp_paths[2], &[0.75], &["DarkCyan", "very thick"])?;

    figure.add_path_arrows(&xm_paths[0], &[0.75], &["Blue", "very thick"])?;
    figure.add_path_arrows(&xm_paths[1], &[0.75], &["DarkOrchid", "very thick"])?;
    figure.add_path_arrows(&xm_paths[2], &[0.75], &["DarkCyan", "very thick"])?;

    figure.add_node("$1$", Complex64::new(2.0, -2.5), &["anchor=north", "Blue"])?;
    figure.add_node(
        "$2$",
        Complex64::new(-1.0, -2.6),
        &["anchor=east", "DarkOrchid"],
    )?;
    figure.add_node(
        "$3$",
        Complex64::new(-1.8, -1.3),
        &["anchor=east", "DarkCyan"],
    )?;
    figure.add_node(
        "$4$",
        Complex64::new(-2.0, -0.6),
        &["anchor=east", "FireBrick"],
    )?;

    figure.add_node(
        "$x^+$",
        xp_paths[0].first_coordinate(Component::Xp, 0).unwrap() + Complex64::new(0.1, 0.1),
        &["anchor=west"],
    )?;
    figure.add_node(
        r"$x^-$",
        xp_paths[0].first_coordinate(Component::Xm, 0).unwrap() + Complex64::new(0.1, 0.1),
        &["anchor=west"],
    )?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(Component::Xp)
                    | CutType::UShortKidney(Component::Xp)
                    | CutType::Log(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_simple_path(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathnames = [
        "u simple path 1",
        "u simple path 2",
        "u simple path 3",
        "u simple path 4",
    ];

    let mut figure = FigureWriter::new(
        "p-simple-path",
        -0.15..0.15,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let paths = pathnames
        .iter()
        .map(|pathname| pxu_provider.get_path(pathname))
        .collect::<Result<Vec<_>>>()?;

    let state = pxu_provider.get_start(pathnames[0])?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;

    figure.add_path_start_mark(&paths[0], &["Blue", "very thick"])?;
    figure.add_path_end_mark(&paths[3], &["FireBrick", "very thick"])?;

    figure.add_path(&paths[0], pt, &["Blue", "solid"])?;
    figure.add_path(&paths[1], pt, &["DarkOrchid", "solid"])?;
    figure.add_path(&paths[2], pt, &["DarkCyan", "solid"])?;
    figure.add_path(&paths[3], pt, &["FireBrick", "solid"])?;

    figure.add_path_arrows(&paths[0], &[0.75], &["Blue", "very thick"])?;
    figure.add_path_arrows(&paths[1], &[0.75], &["DarkOrchid", "very thick"])?;
    figure.add_path_arrows(&paths[2], &[0.75], &["DarkCyan", "very thick"])?;

    // figure.add_node("$1$", Complex64::new(2.0, -2.5), &["anchor=north", "Blue"])?;
    // figure.add_node(
    //     "$2$",
    //     Complex64::new(-1.0, -2.6),
    //     &["anchor=east", "DarkOrchid"],
    // )?;
    // figure.add_node(
    //     "$3$",
    //     Complex64::new(-1.8, -1.3),
    //     &["anchor=east", "DarkCyan"],
    // )?;
    // figure.add_node(
    //     "$4$",
    //     Complex64::new(-2.0, -0.6),
    //     &["anchor=east", "FireBrick"],
    // )?;

    // figure.add_node(
    //     "$x^+$",
    //     xp_paths[0].first_coordinate(Component::Xp, 0).unwrap() + Complex64::new(0.1, 0.1),
    //     &["anchor=west"],
    // )?;
    // figure.add_node(
    //     r"$x^-$",
    //     xp_paths[0].first_coordinate(Component::Xm, 0).unwrap() + Complex64::new(0.1, 0.1),
    //     &["anchor=west"],
    // )?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_) | CutType::E
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_large_circle(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "xp large circle";

    let mut figure = FigureWriter::new(
        "x-large-circle",
        -5.0..5.0,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;
    figure.component_indicator(r"x^{\pm}");

    let xp_path = pxu_provider.get_path(pathname).unwrap();
    let mut xm_path = (*xp_path).clone();
    xm_path.swap_xp_xm();

    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;

    figure.add_path(&xp_path, pt, &["Blue", "solid", "very thick"])?;
    figure.add_path(&xm_path, pt, &["FireBrick", "solid", "very thick"])?;

    figure.add_path_start_mark(&xp_path, &["Blue", "very thick"])?;
    figure.add_path_start_mark(&xm_path, &["FireBrick", "very thick"])?;

    figure.add_path_arrows(&xp_path, &[0.3, 0.76], &["Blue", "very thick"])?;
    figure.add_path_arrows(&xm_path, &[0.3, 0.76], &["FireBrick", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(Component::Xp)
                    | CutType::UShortKidney(Component::Xp)
                    | CutType::Log(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_large_circle(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "xp large circle";

    let mut figure = FigureWriter::new(
        "p-large-circle",
        -0.15..0.15,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let path = pxu_provider.get_path(pathname).unwrap();

    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;

    figure.add_path(&path, pt, &["Blue", "solid", "very thick"])?;

    figure.add_path_start_mark(&path, &["Blue", "very thick"])?;

    figure.add_path_arrows(&path, &[0.3, 0.76], &["Blue", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_) | CutType::E
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_large_circle_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "xp large circle";
    let mut figure = FigureWriter::new(
        "u-large-circle-1",
        -5.2..5.2,
        -2.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let path = pxu_provider.get_path(pathname).unwrap();
    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;

    for seg in path.segments[0].iter().filter(|seg| {
        (seg.sheet_data.u_branch == (UBranch::Outside, UBranch::Outside))
            && (seg.sheet_data.log_branch_p == 0)
            && (seg.sheet_data.log_branch_m == 0)
    }) {
        figure.add_curve(&["Blue", "very thick"], &seg.u)?;
    }

    figure.add_path_start_mark(&path, &["Blue", "very thick"])?;
    figure.add_path_arrows(&path, &[0.15], &["Blue", "solid", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_large_circle_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "xp large circle";
    let mut figure = FigureWriter::new(
        "u-large-circle-2",
        -5.2..5.2,
        -2.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let path = pxu_provider.get_path(pathname).unwrap();
    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let mut state = (*state).clone();
    let pt = &mut state.points[0];
    pt.sheet_data.u_branch = (UBranch::Between, UBranch::Between);

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;

    let mut paths: [Vec<Complex64>; 3] = [vec![], vec![], vec![]];

    for seg in path.segments[0].iter() {
        let index = match seg.sheet_data.u_branch {
            (UBranch::Between, UBranch::Between) => 1,
            (UBranch::Between, _) => 0,
            (_, UBranch::Between) => 2,
            _ => continue,
        };

        paths[index].extend(&seg.u);
    }

    figure.add_curve(&["Blue", "densely dashed", "very thick"], &paths[0])?;
    figure.add_curve(&["Blue", "solid", "very thick"], &paths[1])?;
    figure.add_curve(&["Blue", "densely dashed", "very thick"], &paths[2])?;

    figure.add_path_arrows(&path, &[0.45], &["Blue", "solid", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_large_circle_3(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let shift = Complex64::new(0.0, 2.0 * consts.k() as f64 / consts.h);
    let pathname = "xp large circle";
    let mut figure = FigureWriter::new(
        "u-large-circle-3",
        -5.2..5.2,
        -2.5 + shift.im,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let path = pxu_provider.get_path(pathname).unwrap();
    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let mut state = (*state).clone();
    let pt = &mut state.points[0];
    pt.sheet_data.log_branch_p = 1;
    pt.sheet_data.log_branch_m = -1;

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis_origin(shift)?;

    let mut path = (*path).clone();
    path.segments[0]
        .iter_mut()
        .for_each(|seg| seg.u.iter_mut().for_each(|u| *u += shift));

    for seg in path.segments[0].iter().filter(|seg| {
        (seg.sheet_data.u_branch == (UBranch::Outside, UBranch::Outside))
            && (seg.sheet_data.log_branch_p == 1)
            && (seg.sheet_data.log_branch_m == -1)
    }) {
        figure.add_curve(&["Blue", "very thick"], &seg.u)?;
    }

    figure.add_path_end_mark(&path, &["only marks", "Blue", "very thick"])?;
    figure.add_path_arrows(&path, &[0.8], &["Blue", "solid", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_smaller_circle(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "xp smaller circle";

    let mut figure = FigureWriter::new(
        "x-smaller-circle",
        -3.8..4.2,
        -3.0,
        Size {
            width: 4.0,
            height: 6.0,
        },
        Component::Xp,
        settings,
        pb,
    )?;
    figure.component_indicator(r"x^{\pm}");

    let xp_path = pxu_provider.get_path(pathname).unwrap();
    let mut xm_path = (*xp_path).clone();
    xm_path.swap_xp_xm();

    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;

    figure.add_path(&xp_path, pt, &["Blue", "solid", "very thick"])?;
    figure.add_path(&xm_path, pt, &["FireBrick", "solid", "very thick"])?;

    figure.add_path_start_mark(&xp_path, &["Blue", "very thick"])?;
    figure.add_path_start_end_mark(&xm_path, &["FireBrick", "very thick"])?;

    figure.add_path_arrows(&xp_path, &[0.3, 0.76], &["Blue", "very thick"])?;
    figure.add_path_arrows(&xm_path, &[0.3, 0.76], &["FireBrick", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(Component::Xp)
                    | CutType::UShortKidney(Component::Xp)
                    | CutType::Log(Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black"], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_smaller_circle_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "xp smaller circle";
    let mut figure = FigureWriter::new(
        "u-smaller-circle-1",
        -5.2..5.2,
        -2.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let path = pxu_provider.get_path(pathname).unwrap();
    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;

    for seg in path.segments[0].iter().filter(|seg| {
        (seg.sheet_data.u_branch == (UBranch::Outside, UBranch::Outside))
            && (seg.sheet_data.log_branch_p == 0)
            && (seg.sheet_data.log_branch_m == 0)
    }) {
        figure.add_curve(&["Blue", "very thick"], &seg.u)?;
    }

    figure.add_path_start_mark(&path, &["Blue", "very thick"])?;
    figure.add_path_arrows(&path, &[0.1], &["Blue", "solid", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_smaller_circle_2(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "xp smaller circle";
    let mut figure = FigureWriter::new(
        "u-smaller-circle-2",
        -5.2..5.2,
        -2.5,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let path = pxu_provider.get_path(pathname).unwrap();
    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let mut state = (*state).clone();
    let pt = &mut state.points[0];
    pt.sheet_data.u_branch = (UBranch::Between, UBranch::Outside);

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;

    for seg in path.segments[0]
        .iter()
        .filter(|seg| (seg.sheet_data.u_branch == (UBranch::Between, UBranch::Outside)))
    {
        figure.add_curve(&["Blue", "very thick"], &seg.u)?;
    }

    figure.add_path_arrows(&path, &[0.4], &["Blue", "solid", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_smaller_circle_3(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let shift = Complex64::new(0.0, 2.0 * consts.k() as f64 / consts.h);
    let pathname = "xp smaller circle";
    let mut figure = FigureWriter::new(
        "u-smaller-circle-3",
        -5.2..5.2,
        -2.5 + shift.im,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let path = pxu_provider.get_path(pathname).unwrap();
    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let mut state = (*state).clone();
    let pt = &mut state.points[0];
    pt.sheet_data.log_branch_p = 1;
    pt.sheet_data.log_branch_m = 0;

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis_origin(shift)?;

    let mut path = (*path).clone();
    path.segments[0]
        .iter_mut()
        .for_each(|seg| seg.u.iter_mut().for_each(|u| *u += shift));

    for seg in path.segments[0].iter().filter(|seg| {
        (seg.sheet_data.u_branch == (UBranch::Outside, UBranch::Outside))
            && (seg.sheet_data.log_branch_p == 1)
            && (seg.sheet_data.log_branch_m == 0)
    }) {
        figure.add_curve(&["Blue", "very thick"], &seg.u)?;
    }

    figure.add_path_end_mark(&path, &["only marks", "Blue", "very thick"])?;
    figure.add_path_arrows(&path, &[0.8], &["Blue", "solid", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_)
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_smaller_circle(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(2.0, 5);
    let pathname = "xp smaller circle";

    let mut figure = FigureWriter::new(
        "p-smaller-circle",
        -0.05..1.85,
        0.0,
        Size {
            width: 8.0,
            height: 6.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let path = pxu_provider.get_path(pathname).unwrap();

    let state = pxu_provider.get_start(pathname)?;
    let contours = &pxu_provider.get_contours(consts)?;

    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;

    figure.add_path(&path, pt, &["Blue", "solid", "very thick"])?;

    figure.add_path_start_end_mark(&path, &["Blue", "very thick"])?;

    figure.add_path_arrows(&path, &[0.4, 0.8], &["Blue", "very thick"])?;

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_) | CutType::E
            )
        })
    {
        figure.add_cut(cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_bs3_region_min_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(1.0, 7);
    let pathnames = ["bs3 region -1 1", "bs3 region -1 2"];
    let mut figure = FigureWriter::new(
        "u-bs-3-region-min-1",
        -7.25..7.25,
        7.0,
        Size {
            width: 4.0,
            height: 4.0,
        },
        Component::U,
        settings,
        pb,
    )?;

    let paths = pathnames
        .into_iter()
        .map(|pathname| pxu_provider.get_path(pathname))
        .collect::<Result<Vec<_>>>()?;
    let state = pxu_provider.get_start(pathnames[0])?;
    let contours = &pxu_provider.get_contours(consts)?;
    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;
    figure.add_path_all(&paths[0], pt, &["Blue"])?;
    figure.add_path_all(&paths[1], pt, &["FireBrick"])?;

    figure.add_path_arrows_all(&paths[0], &[0.55], &["Blue", "very thick"])?;
    figure.add_path_arrows_all(&paths[1], &[0.55], &["FireBrick", "very thick"])?;

    figure.add_path_start_mark_all(&paths[0], &["Blue", "mark size=0.05cm"])?;
    figure.add_path_end_mark_all(&paths[1], &["FireBrick", "mark size=0.05cm"])?;

    figure.add_path_start_mark_n(
        &paths[1],
        &[
            "fill=Blue",
            "mark color=FireBrick",
            "mark size=0.065cm",
            "mark=halfcircle*",
            "mark options={rotate=90}",
            "scatter",
            "scatter/use mapped color={draw opacity=0}",
        ],
        1,
    )?;

    figure.add_node(
        r"$\scriptstyle u_c$",
        paths[1].first_coordinate(figure.component, 1).unwrap()
            + Complex64::new(-0.25, 0.15) / consts.h,
        &["anchor=north west"],
    )?;

    for n in 0..3 {
        figure.add_node(
            &format!(r"$\scriptstyle u_{}$", n + 1),
            paths[0].first_coordinate(figure.component, n).unwrap(),
            &["anchor=east"],
        )?;
    }

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_) | CutType::E
            )
        })
    {
        let mut cut = cut.clone();
        cut.periodic = true;
        figure.add_cut(&cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_bs3_region_min_1(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let consts = CouplingConstants::new(1.0, 7);
    let pathnames = ["bs3 region -1 1", "bs3 region -1 2"];
    let mut figure = FigureWriter::new(
        "p-bs-3-region-min-1",
        -1.0..0.0,
        0.0,
        Size {
            width: 8.0,
            height: 4.0,
        },
        Component::P,
        settings,
        pb,
    )?;

    let paths = pathnames
        .into_iter()
        .map(|pathname| pxu_provider.get_path(pathname))
        .collect::<Result<Vec<_>>>()?;
    let state = pxu_provider.get_start(pathnames[0])?;
    let contours = &pxu_provider.get_contours(consts)?;
    let pt = &state.points[0];

    figure.add_grid_lines(contours, &[])?;
    figure.add_axis()?;
    figure.add_path_all(&paths[0], pt, &["Blue"])?;
    figure.add_path_all(&paths[1], pt, &["FireBrick"])?;

    figure.add_path_arrows_n(&paths[0], &[0.65], &["Blue", "very thick"], 1)?;
    figure.add_path_arrows_n(&paths[1], &[0.65], &["FireBrick", "very thick"], 1)?;

    figure.add_path_start_mark_all(&paths[0], &["Blue", "mark size=0.05cm"])?;
    figure.add_path_end_mark_all(&paths[1], &["FireBrick", "mark size=0.05cm"])?;

    figure.add_path_start_mark_n(
        &paths[1],
        &[
            "fill=Blue",
            "mark color=FireBrick",
            "mark size=0.065cm",
            "mark=halfcircle*",
            "mark options={rotate=-90}",
            "scatter",
            "scatter/use mapped color={draw opacity=0}",
        ],
        1,
    )?;

    figure.add_node(
        r"$\scriptstyle p_c$",
        paths[1].first_coordinate(figure.component, 1).unwrap(),
        &["anchor=north"],
    )?;

    for n in 0..3 {
        figure.add_node(
            &format!(r"$\scriptstyle p_{}$", n + 1),
            paths[0].first_coordinate(figure.component, n).unwrap(),
            if n == 1 {
                &["anchor=west"]
            } else {
                &["anchor=east"]
            },
        )?;
    }

    for cut in contours
        .get_visible_cuts_from_point(pt, figure.component, consts)
        .filter(|cut| {
            matches!(
                cut.typ,
                CutType::UShortScallion(_) | CutType::UShortKidney(_) | CutType::E
            )
        })
    {
        let mut cut = cut.clone();
        cut.periodic = true;
        figure.add_cut(&cut, &[], consts)?;
    }

    figure.finish(cache, settings, pb)
}

// Intereseting states:
// m = 5, p = -1, E = C = 0
// (points:[(p:(-0.10165672487090872,-0.05348001731440205),xp:(0.9366063608108588,-0.0000000000000015543122344752192),xm:(0.5373538000115556,0.39902207324643024),u:(2.05640778996199,4.500000000000002),x:(0.73668849857164,0.3178014188683358),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.048112372695696085,-0.049461724147602956),xp:(0.5373538000115555,0.39902207324643024),xm:(0.2888944083459811,0.39641831953822726),u:(2.05640778996199,3.5000000000000013),x:(0.39367175820818845,0.41130042259798616),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.7004618048667908,0.0),xp:(0.2888944083459809,0.3964183195382271),xm:(0.2888944083459809,-0.3964183195382271),u:(2.0564077899619906,2.5),x:(3.109957546500381,3.3102829988967026),sheet_data:(log_branch_p:-1,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.048112372695696085,0.049461724147602956),xp:(0.2888944083459811,-0.39641831953822726),xm:(0.5373538000115555,-0.39902207324643024),u:(2.0564077899619897,1.4999999999999982),x:(0.39367175820818856,-0.4113004225979862),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.10165672487090872,0.05348001731440205),xp:(0.5373538000115556,-0.39902207324643024),xm:(0.9366063608108588,0.0000000000000015543122344752192),u:(2.05640778996199,0.4999999999999982),x:(0.7366884985716402,-0.317801418868336),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1)))],lock:true)

// singlet 1 + 4
// (points:[(p:(-0.06481769289200064,-0.04632014396084205),xp:(0.6773737156527935,0.24101679937073833),xm:(0.39355556208794307,0.3659765169104283),u:(2.2503158561824144,-0.9972640693939946),x:(0.5207960049771001,0.3382736317263967),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134065179824,-0.04287934452264521),xp:(0.3935555620861755,0.3659765169090202),xm:(0.22233500515739787,0.34507249230177073),u:(2.250315856189289,-1.997264069401408),x:(0.29603586257460585,0.36274180923791544),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216060976681002,0.042633420284661425),xp:(0.22233500515775476,0.34507249230145126),xm:(0.3923377926330045,-0.3660664539125623),u:(2.2503158561923926,-2.9972640693996655),x:(0.16710333623086243,0.3211911819475663),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.0645947551037885,0.04632338280244304),xp:(0.3923377926336257,-0.36606645391208686),xm:(0.6755998929977572,-0.24272408911183854),u:(2.2503158561943186,-3.9972640694026023),x:(0.5192267118211283,-0.33884808844761033),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.10930011368445881,0.00024268539559447655),xp:(0.6755998929977572,-0.2427240891118387),xm:(0.6773737156462706,0.24101679936958165),u:(2.2503158561943186,0.002735930597398628),x:(0.7857319077395628,-0.0016758790700285356),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)
// singlet 2 + 3
// (points:[(p:(-0.0918635850967006,-0.037587502213391646),xp:(0.785884223705366,0.0000000000000002220446049250313),xm:(0.5200361660196523,0.3386309516954546),u:(2.2500748563450794,-0.5000000000000003),x:(0.6765622619422568,0.24195091368028965),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04931502967968751,-0.044946057622269636),xp:(0.5200361660196524,0.3386309516954545),xm:(0.29556714680693774,0.3627151161370183),u:(2.2500748563450794,-1.5),x:(0.392950187668455,0.36607556161166316),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7176427704472238,-0.000000000000000019937695239947602),xp:(0.2955671468069379,0.36271511613701846),xm:(0.29556714680693785,-0.3627151161370184),u:(2.2500748563450785,-2.499999999999999),x:(0.2219764434485283,0.34498404739256483),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931502967968751,0.044946057622269636),xp:(0.29556714680693774,-0.3627151161370183),xm:(0.5200361660196524,-0.3386309516954545),u:(2.2500748563450794,-3.4999999999999996),x:(0.39295018766845496,-0.36607556161166327),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09186358509670066,0.03758750221339164),xp:(0.5200361660196525,-0.33863095169545443),xm:(0.785884223705366,0.0000000000000003608224830031759),u:(2.2500748563450794,0.4999999999999998),x:(0.676562261942257,-0.2419509136802895),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)
// singlet 3 + 2
// (points:[(p:(-0.064817690638922,-0.04632014058248584),xp:(0.6773736720447697,0.24101678917659286),xm:(0.39355554871074094,0.3659764991995006),u:(2.250315939687509,-0.9972641231359414),x:(0.5207959807194622,0.33827361344245904),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134011794477,-0.042879342951094745),xp:(0.39355554871074067,0.3659764991995013),xm:(0.22233500194749478,0.34507247933376406),u:(2.250315939687506,-1.9972641231359423),x:(0.2960358555274206,0.3627417937862914),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216061057006049,0.04263342355344563),xp:(0.22233500194749445,0.3450724793337641),xm:(0.3923378032288628,-0.3660664344918713),u:(2.2503159396875043,-2.9972641231359445),x:(0.16710333534746072,0.32119117129204844),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.06459475724215495,0.04632337938493029),xp:(0.39233780322886325,-0.36606643449187204),xm:(0.6755998845174871,-0.24272404535577444),u:(2.2503159396875008,1.0027358768640537),x:(0.5192267310835156,-0.3388480606808871),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.10930010734366312,0.00024268100631728482),xp:(0.6755998866881463,-0.2427240505990194),xm:(0.6773736772251796,0.2410167915569991),u:(2.2503159279047136,0.0027358814445184176),x:(0.7857318639819022,-0.0016758487182760083),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)
// singlet 4 + 1
// (points:[(p:(-0.09185221149636245,-0.037572722189714455),xp:(0.7857363886452503,0.0000004328254604446524),xm:(0.5200106363475369,0.3385618195950395),u:(2.2503161408013796,-0.5000007065959058),x:(0.676486747365414,0.24187289813934523),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.04931600633410893,-0.0449403973338789),xp:(0.5200106363475344,0.338561819595029),xm:(0.29557299472051746,0.3626743175215065),u:(2.2503161408014147,-1.5000007065959013),x:(0.392946068121917,0.36602187168832023),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.717663444470969,0.00000006054071687339567),xp:(0.2955729947205189,0.3626743175215076),xm:(0.2955732335644112,-0.36267435245574203),u:(2.2503161408014094,-2.500000706595892),x:(0.22198686543101423,0.3449533442179103),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931603946892371,0.044940403147529916),xp:(0.2955732335644095,-0.36267435245574087),xm:(0.5200110416414399,-0.3385616712335204),u:(2.2503161408014156,1.499999293404119),x:(0.392946382629357,-0.36602184846097735),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.09185229822963642,0.03757265583534658),xp:(0.5200110416414421,-0.33856167123353087),xm:(0.7857363886452495,0.00000043282544220923924),u:(2.250316140801381,0.4999992934041242),x:(0.6764872054840881,-0.24187245720745892),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)

// Singlet with a physical 4 particle bound state with p=0.2 and a single crossed excitation with p=-1.2
// (points:[(p:(0.03697370701345617,-0.031054542242344985),xp:(3.6273956071620397,2.6302676553779873),xm:(3.4024312560818917,1.4187889153646274),u:(2.6145968437167793,1.49999978054477),x:(3.5185977035037714,2.04179131076382),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06287718000079495,-0.020435255813501682),xp:(3.4024312560818943,1.4187889153646245),xm:(3.2421939150597834,-0.00000033279749789283386),u:(2.6145968437167824,0.49999978054476846),x:(3.2939078541224145,0.741134209511953),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06287716823638237,0.020435269329719074),xp:(3.2421939150597887,-0.00000033279750100145833),xm:(3.4024313586416444,-1.4187894828446725),u:(2.614596843716786,-0.5000002194552335),x:(3.293907934922962,-0.741134834449078),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(0.036973698743170816,0.031054541890648338),xp:(3.4024313586416506,-1.4187894828446699),xm:(3.6273956983334217,-2.630268160991778),u:(2.614596843716791,-1.5000002194552322),x:(3.518597803076299,-2.0417918401049153),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-1.199701753993804,-0.000000013164520818520966),xp:(3.6273956983334235,-2.630268160991774),xm:(3.6273956071620557,2.6302676553779865),u:(2.614596843716793,-2.500000219455229),x:(3.72626001655586,-3.1968938333767136),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:-1,u_branch:(Outside,Outside),im_x_sign:(1,1)))],unlocked:false)

// h=2, k=4, p=-1.4, m=11
// (points:[(p:(-0.020683140974430327,-0.045520745607578246),xp:(-0.7328798041070045,2.9713029888519578),xm:(-0.8352053806655594,2.1420224467780176),u:(-1.523209095511133,5.00010006636237),x:(-0.7878029894955824,2.5490269820920988),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(-0.03136635310674252,-0.05424829064931387),xp:(-0.8352053806655592,2.142022446778017),xm:(-0.8807493082122642,1.377538188559913),u:(-1.5232090955111328,4.00010006636237),x:(-0.8690309426228984,1.7514712657655553),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:1,u_branch:(Outside,Between),im_x_sign:(-1,1))),(p:(-0.04474777771721691,-0.07327868330130795),xp:(-0.8807493082122642,1.3775381885599125),xm:(-0.775134383807947,0.6809095595993009),u:(-1.523209095511133,3.0001000663623696),x:(-0.85747892162112,1.019686731169598),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.008693981335154689,-0.11498443009542221),xp:(-0.7751343838079467,0.6809095595993008),xm:(-0.3938580812565977,0.30957475058111117),u:(-1.5232090955111324,2.000100066362369),x:(-0.674650218991303,3.40699219416618),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:0,e_branch:-1,u_branch:(Between,Inside),im_x_sign:(-1,-1))),(p:(0.028967391732394272,-0.06734733480397649),xp:(-0.39385808125659766,0.3095747505811114),xm:(-0.2170056023934792,0.24610829505720347),u:(-1.523209095511132,1.0001000663623696),x:(-0.8574600451888923,-1.0195467502199065),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,1))),(p:(-1.2699815160111105,-0.000010390643065416034),xp:(-0.21700560239347927,0.24610829505720358),xm:(-0.2169840620595849,-0.24609872926846416),u:(-1.523209095511131,0.00010006636237047672),x:(-0.869030942622897,1.7514712657655558),sheet_data:(log_branch_p:-1,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Inside,Inside),im_x_sign:(-1,-1))),(p:(0.028965734929305424,0.06733932971579816),xp:(-0.21698406205958481,-0.24609872926846404),xm:(-0.3938000635227753,-0.30955486922799),u:(-1.523209095511131,-0.9998999336376291),x:(-0.8574789216211187,1.0196867311695992),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,1))),(p:(-0.00867770948721811,0.11498380150546404),xp:(-0.3938000635227753,-0.3095548692279899),xm:(-0.77508385488455,-0.6807802215522121),u:(-1.5232090955111313,-1.999899933637629),x:(-0.6746502189913016,3.406992194166181),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Inside,Between),im_x_sign:(-1,-1))),(p:(-0.04475004732130248,0.07328563453707682),xp:(-0.7750838548845497,-0.6807802215522117),xm:(-0.8807480746429597,-1.3773917672943192),u:(-1.5232090955111308,-2.999899933637628),x:(-0.8574600451888913,-1.019546750219905),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.031368835582754766,0.05425048662768747),xp:(-0.8807480746429598,-1.3773917672943194),xm:(-0.8352221105330633,-2.141862783519232),u:(-1.523209095511131,-3.9998999336376286),x:(-0.8690407290991615,-1.7513182937502383),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,-1))),(p:(-0.020684940245014834,0.04552223064426885),xp:(-0.8352221105330631,-2.1418627835192328),xm:(-0.7329026772950994,-2.9711311455501623),u:(-1.523209095511131,-4.9998999336376295),x:(-0.7878238045215702,-2.548860909727752),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1)))],unlocked:false)

// M=13, p=-1.48 same as M=3, p=0.52
// (points:[(p:(-0.015981953994237602,-0.046485693541077955),xp:(-0.19710352030247388,3.119591150243588),xm:(-0.3799605865912995,2.302940972405549),u:(-1.1242161015850338,6.000098620339166),x:(-0.28967863385578146,2.7011225074939977),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(-0.02541464842136689,-0.05390885859825193),xp:(-0.37996059277279115,2.3029409811301282),xm:(-0.528319074033538,1.5773267221166196),u:(-1.1242161116045408,5.000098629349641),x:(-0.4620716073543235,1.9278370919008467),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:1,u_branch:(Outside,Between),im_x_sign:(-1,1))),(p:(-0.035014743811120554,-0.06407271568777029),xp:(-0.5283195039556788,1.5773263473851908),xm:(-0.574862966081123,0.9520823586562688),u:(-1.1242166424573128,4.0000979881409595),x:(-0.5695288940926104,1.2517530741551484),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.030543124235659032,-0.08605959312296839),xp:(-0.5748629761267917,0.9520823157456079),xm:(-0.43435692528005193,0.4803934525853911),u:(-1.1242166644169567,3.0000979142398685),x:(0.06301218509527529,4.462893814830273),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(0.008032286782501143,-0.08217309965847498),xp:(-0.43435692528153924,0.4803934525883716),xm:(-0.24439794233176473,0.2993702436234465),u:(-1.1242166644156717,2.0000979142481903),x:(-0.32486962181010887,0.36209265588731815),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:1,e_branch:-1,u_branch:(Between,Inside),im_x_sign:(-1,1))),(p:(0.015968162355591738,-0.05323688230843595),xp:(-0.2443976285210311,0.29937042284995613),xm:(-0.1525744909402503,0.2307012178359294),u:(-1.1242146315741093,1.000097008974504),x:(-0.2896777269934593,2.7011208870129164),sheet_data:(log_branch_p:-1,log_branch_m:1,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(-1,1))),(p:(-1.3140100372736065,-0.000008963246167334362),xp:(-0.15257471985610627,0.23070143769856463),xm:(-0.15256194114179822,-0.23069121403801338),u:(-1.124214277996606,0.00010090687950636834),x:(-0.46206998328873633,1.9278384561411124),sheet_data:(log_branch_p:-1,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Inside,Inside),im_x_sign:(-1,-1))),(p:(0.01596767126323757,0.05323285945084886),xp:(-0.15256194111587917,-0.23069121400591378),xm:(-0.24437168095668754,-0.29935145922500705),u:(-1.1242142781060254,-0.9998990926235782),x:(-0.5695265126645834,1.2517585437352123),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(0.008037838278110743,0.08216671497005919),xp:(-0.24437168025689937,-0.2993514583586551),xm:(-0.4343107145396756,-0.4803298130379087),u:(-1.1242142798589638,-1.999899085992567),x:(-0.5313336099821706,0.6858263218634796),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.0305377891152034,0.08606432522211882),xp:(-0.43431071413901273,-0.4803298129875953),xm:(-0.5748549233767548,-0.9519689358328741),u:(-1.1242142791056948,-2.999899085303322),x:(-0.5313042640226974,-0.6857253260353564),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Inside,Between),im_x_sign:(1,-1))),(p:(-0.03501621277823515,0.06407561196807394),xp:(-0.5748551144965012,-0.9519684277031095),xm:(-0.5283402892099497,-1.5771910653848307),u:(-1.1242146652382545,-3.9998982176176154),x:(-0.569537759480603,-1.2516279077487549),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.02541669341212124,0.053910446354621326),xp:(-0.5283402955446876,-1.5771908873974192),xm:(-0.3799944996608143,-2.302785399908366),u:(-1.1242146323727717,-4.999897959736789),x:(-0.4621008517387099,-1.9276912274938025),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,-1))),(p:(-0.01598359325365527,0.04648716726637495),xp:(-0.3799944922221864,-2.3027853986847115),xm:(-0.1971392811010283,-3.119419321596219),u:(-1.1242146228885428,-5.99989796029617),x:(-0.2897144434239505,-2.7009581485359777),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1)))],unlocked:true)

// (points:[(p:(0.008703264611073246,-0.035068810323283524),xp:(2.0561658800769416,4.596656914316539),xm:(1.848633743081469,3.591964005727913),u:(0.8508615493774978,3.000099411258928),x:(1.9593931834335436,4.0935986834524645),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(1.3574073514831153,0.03506256764453059),xp:(1.8486337430814679,3.5919640057279127),xm:(2.056129730937162,-4.596456713050234),u:(0.8508615493774965,-2.9999005887410743),x:(1.9593521653500987,-4.0933988568711746),sheet_data:(log_branch_p:1,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))],unlocked:false)
// (points:[(p:(0.008456822061412005,-0.03512740086365839),xp:(2.0250306299391014,4.602280659587191),xm:(1.8176971264103987,3.599328766047288),u:(0.8199202962937739,3.0000989431717713),x:(1.9283220791961222,4.099978631234418),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.010120189227141398,-0.045509057441523955),xp:(1.8176971264103976,3.5993287660472864),xm:(1.5347234608115312,2.6119543565246373),u:(0.819920296293773,2.0000989431717704),x:(0.13222778451172496,-0.6218049501509784),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.007825136344541624,-0.06352184735722012),xp:(1.5347234608115292,2.611954356524638),xm:(1.1145423420133296,1.6996651595351895),u:(0.8199202962937712,1.000098943171771),x:(1.346494994802847,2.137709286728024),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.31525215058612127,-0.000014911000817724449),xp:(1.1145423420133294,1.6996651595351888),xm:(1.1144416159442925,-1.6995035087835628),u:(0.819920296293771,0.0000989431717699496),x:(0.8457125556030123,1.3353135324366436),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.007823416146698096,0.06352629260682484),xp:(1.1144416159442896,-1.699503508783561),xm:(1.5346564413647352,-2.6117626866602177),u:(0.8199202962937695,-0.9999010568282269),x:(1.3464123000482295,-2.13752684392927),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(0.010120396692125598,0.04551172248174652),xp:(1.5346564413647337,-2.611762686660214),xm:(1.8176500585559867,-3.599131133848416),u:(0.819920296293769,-1.9999010568282232),x:(1.6886359129748154,-3.1018699740747815),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(0.008457185309576318,0.03512897764102424),xp:(1.8176500585559858,-3.5991311338484144),xm:(2.0249946678995734,-4.6020816651223715),u:(0.8199202962937684,-2.9999010568282216),x:(1.9282812903219275,-4.099780084479133),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))],unlocked:false)
// (points:[(p:(0.00843676807928177,-0.03513199668600366),xp:(2.0225004379092395,4.602739443684855),xm:(1.8151844324034783,3.5999287328595337),u:(0.8174037933349869,3.0001004136552805),x:(1.9257976541383772,4.100498780991189),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.010086307097622923,-0.04551165207354569),xp:(1.8151844324034787,3.5999287328595337),xm:(1.5322869268709625,2.612807837068172),u:(0.8174037933349876,2.0001004136552805),x:(0.13168703015727753,-0.622206072424072),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.007768687086187467,-0.0634999265708896),xp:(1.532286926870964,2.612807837068173),xm:(1.1124878377536462,1.700946307309016),u:(0.8174037933349889,1.000100413655282),x:(1.3441760320480471,2.1387617830721486),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(-0.012760439221813963,-0.08110003527870467),xp:(1.112487837753648,1.7009463073090156),xm:(0.5843461131739043,1.0720980362612633),u:(0.8174037933349905,0.00010041365528257185),x:(0.8440193678103955,-1.3365693652418347),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,-1))),(p:(-0.03128471282788021,-0.07223884819342559),xp:(0.5843461131739062,1.0720980362612622),xm:(0.2310100029006664,0.7403195868810634),u:(0.8174037933349938,-0.9998995863447169),x:(0.378046642973153,0.8845879443155366),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.568576186421524,-0.06102657076356458),xp:(0.23101000290066678,0.7403195868810637),xm:(0.06768228089702588,-0.5241780118357348),u:(0.8174037933349938,-1.9998995863447153),x:(-0.016668802918680562,0.169551123545851),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.027695577907200265,0.06101340663270868),xp:(0.06768228089702624,-0.5241780118357348),xm:(0.23096142338091374,-0.7402675516939287),u:(0.8174037933349955,-2.999899586344716),x:(0.13168703015727876,-0.6222060724240701),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Inside,Between),im_x_sign:(1,-1))),(p:(-0.03128565068050761,0.07223598695607951),xp:(0.23096142338091474,-0.7402675516939278),xm:(0.5842509144477248,-1.0720099109472596),u:(0.8174037933349994,-3.999899586344717),x:(0.37797625143095687,-0.8845230268536034),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.012766176787802407,0.08110093906354647),xp:(0.5842509144477261,-1.0720099109472587),xm:(1.1123857641281532,-1.7007823370614374),u:(0.8174037933350016,-4.999899586344718),x:(0.8440193678104028,-1.3365693652418285),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,-1))),(p:(0.0077669385438059015,0.06350442729062239),xp:(1.1123857641281547,-1.7007823370614383),xm:(1.5322189416726264,-2.6126133875158106),u:(0.8174037933350022,-5.999899586344719),x:(1.344092177901383,-2.138576719172685),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.01008651372735086,0.045514355358441116),xp:(1.5322189416726266,-2.6126133875158115),xm:(1.8151366715985073,-3.59972820026474),u:(0.8174037933350025,-6.9998995863447195),x:(2.8704197488363055,-13.151476411498896),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.008437134840079658,0.03513359697474945),xp:(1.815136671598508,-3.5997282002647424),xm:(2.022463942971389,-4.602537513548278),u:(0.8174037933350022,-7.999899586344722),x:(1.9257562622871796,-4.100297311354334),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1)))],unlocked:false)
// (points:[(p:(0.008504583798772088,-0.035116319700573294),xp:(2.0310602321435156,4.601192336448928),xm:(1.8236860657460943,3.5979038213284),u:(0.8259153367490619,3.0001001254211848),x:(1.9343383671092051,4.098744023547635),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.010200969845901249,-0.04550261494750871),xp:(1.8236860657460947,3.5979038213284005),xm:(1.5405330621577877,2.609924132829659),u:(0.8259153367490621,2.000100125421185),x:(0.13351168692396484,-0.6208463450691568),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.007960192001555002,-0.06357382171119046),xp:(1.5405330621577886,2.6099241328296587),xm:(1.11944336273788,1.6966093669966547),u:(0.8259153367490633,1.0001001254211848),x:(1.352025969780485,2.135202745026528),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(-0.012722182657881366,-0.08134458093418913),xp:(1.1194433627378806,1.6966093669966533),xm:(0.5880727927249432,1.0680517330150374),u:(0.8259153367490646,0.00010012542118420509),x:(0.8493750305023915,-1.3318868074347914),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,-1))),(p:(-0.03140689089138959,-0.07227848066337073),xp:(0.5880727927249434,1.0680517330150365),xm:(0.2332084780767102,0.7382567722288272),u:(0.8259153367490658,-0.9998998745788164),x:(0.380789270523584,0.8815982816018764),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Outside,Between),im_x_sign:(1,-1))),(p:(-0.02777035244194526,-0.060942522213294804),xp:(0.23320847807671063,0.7382567722288274),xm:(0.06921300804414522,0.5233589956165808),u:(0.8259153367490667,-1.999899874578815),x:(-0.016428179656077273,0.16956485311996405),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.018153512014311726,-0.05298576814605994),xp:(0.06921300804414529,0.5233589956165808),xm:(0.006592886744828705,0.37836736667440274),u:(0.825915336749067,-2.999899874578815),x:(1.9342970833496362,-4.098543038080382),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Inside),im_x_sign:(1,-1))),(p:(-0.009985077989350393,-0.04484655870383054),xp:(0.00659288674482856,0.37836736667440285),xm:(-0.01293303863452533,0.28520580704222387),u:(0.8259153367490659,-3.999899874578815),x:(-0.006223493995138852,0.3265423856854216),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-1.485574288950221,-0.000008180452225970939),xp:(-0.012933038634525393,0.28520580704222437),xm:(-0.01293489175771858,-0.28519103385426203),u:(0.8259153367490643,-4.999899874578809),x:(2.746936330172484,-11.142155843007524),sheet_data:(log_branch_p:0,log_branch_m:-2,log_branch_x:1,e_branch:1,u_branch:(Inside,Inside),im_x_sign:(1,1))),(p:(-0.009983772843950994,0.04484493726643239),xp:(-0.012934891757718534,-0.28519103385426203),xm:(0.006586037849219721,-0.37834417705667645),u:(0.8259153367490649,-5.999899874578817),x:(-0.0062271846585451925,-0.32652389514860536),sheet_data:(log_branch_p:2,log_branch_m:-2,log_branch_x:-2,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(-1,1))),(p:(-0.01815159635002861,0.05298417576951007),xp:(0.006586037849219725,-0.3783441770566764),xm:(0.06919284241842479,-0.5233235398438041),u:(0.825915336749065,-6.999899874578816),x:(0.1335437484364576,0.6208891544601038),sheet_data:(log_branch_p:2,log_branch_m:-2,log_branch_x:-1,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(-1,1))),(p:(-0.027768641516948026,0.060940799026703206),xp:(0.06919284241842472,-0.5233235398438043),xm:(0.23315986602397537,-0.7382052120657752),u:(0.8259153367490644,-7.999899874578816),x:(0.13351168692396545,-0.6208463450691566),sheet_data:(log_branch_p:2,log_branch_m:-2,log_branch_x:-2,e_branch:-1,u_branch:(Inside,Between),im_x_sign:(-1,1))),(p:(-0.03140781971284981,0.07227558488286355),xp:(0.23315986602397598,-0.7382052120657749),xm:(0.5879773385079586,-1.0679642572576211),u:(0.8259153367490659,-8.999899874578817),x:(0.3807187988870824,-0.8815339636510102),sheet_data:(log_branch_p:2,log_branch_m:-2,log_branch_x:-2,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.01272796844009084,0.0813454787332294),xp:(0.5879773385079594,-1.0679642572576202),xm:(1.1193410772171217,-1.6964455857607974),u:(0.8259153367490678,-9.999899874578816),x:(0.8493750305023934,-1.3318868074347894),sheet_data:(log_branch_p:2,log_branch_m:-2,log_branch_x:-2,e_branch:1,u_branch:(Between,Outside),im_x_sign:(-1,1))),(p:(0.007958457995764022,0.06357834537780237),xp:(1.1193410772171222,-1.6964455857607976),xm:(1.5404651704138481,-2.609730008600267),u:(0.8259153367490673,-10.999899874578817),x:(1.3519421219160495,-2.135017906474365),sheet_data:(log_branch_p:2,log_branch_m:-2,log_branch_x:-2,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(0.010201189133454311,0.04550531504147031),xp:(1.5404651704138483,-2.609730008600268),xm:(1.8236384201059015,-3.597703740169742),u:(0.8259153367490671,-11.99989987457882),x:(3.1358188803934888,-18.16733474231712),sheet_data:(log_branch_p:2,log_branch_m:-2,log_branch_x:1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(0.00850495587109817,0.035117915121075954),xp:(1.823638420105902,-3.5977037401697416),xm:(2.031023836785722,-4.600990912521743),u:(0.8259153367490675,-12.999899874578817),x:(1.934297083349637,-4.0985430380803844),sheet_data:(log_branch_p:2,log_branch_m:-2,log_branch_x:-2,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))],unlocked:false)
// (points:[(p:(0.00850451567122419,-0.0351163366428777),xp:(2.0310515783264487,4.601193769786978),xm:(1.8236774624129255,3.5979057375562773),u:(0.8259067559962097,3.000099999989294),x:(1.93432972901603,4.098745666617839),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.010200854291000724,-0.04550262603714801),xp:(1.8236774624129262,3.597905737556278),xm:(1.5405247013607068,2.6099269202774904),u:(0.8259067559962101,2.0000999999892946),x:(0.1335098726834099,-0.6208477455958303),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.007959997082149057,-0.06357375026038176),xp:(1.5405247013607066,2.60992692027749),xm:(1.1194362803928564,1.69661364718726),u:(0.8259067559962101,1.0000999999892937),x:(1.352017996815633,2.1352062219696677),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(-0.012722224785590501,-0.08134433357584302),xp:(1.1194362803928557,1.6966136471872597),xm:(0.5880689843184308,1.0680557686233254),u:(0.8259067559962098,0.0000999999892933312),x:(0.849369701941432,-1.331891622661128),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,-1))),(p:(-0.03140676738687043,-0.07227843902770795),xp:(0.5880689843184305,1.0680557686233256),xm:(0.2332062381776739,0.7382588231196434),u:(0.8259067559962091,-0.9999000000107066),x:(0.38078647020266615,0.8816012612265064),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Outside,Between),im_x_sign:(1,-1))),(p:(-0.027770277529154154,-0.06094259444272281),xp:(0.23320623817767397,0.7382588231196433),xm:(0.06921147606791347,0.5233598375865766),u:(0.8259067559962093,-1.9999000000107068),x:(-0.016428422019962932,0.1695648360578815),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.018153442581473014,-0.052985849236542905),xp:(0.06921147606791343,0.5233598375865767),xm:(0.006591888732602278,0.37836764565645814),u:(0.8259067559962092,-2.9999000000107072),x:(1.9342884969851755,-4.098544933029524),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Inside),im_x_sign:(1,-1))),(p:(-0.00998500769250728,-0.044846613616642786),xp:(0.0065918887326022226,0.378367645656458),xm:(-0.01293367285536308,0.2852058772051723),u:(0.8259067559962093,-3.999900000010708),x:(-0.006224288663879103,0.326542532266881),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.00504454849795598,-0.037273352266220736),xp:(-0.012933672855363176,0.28520587720517243),xm:(-0.01737926285524494,0.22521931099646053),u:(0.8259067559962077,-4.999900000010707),x:(-0.01613831014380371,0.25206198159240306),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.002444001225971159,-0.0311580515212133),xp:(-0.01737926285524475,0.22521931099646045),xm:(-0.01713100290273024,0.18493402813542317),u:(0.8259067559962114,-5.999900000010708),x:(0.38078647020266637,0.881601261226505),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:-1,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.001107047302755978,-0.026473566594018966),xp:(-0.017131002902730157,0.18493402813542284),xm:(-0.015594721972887067,0.15648999637430783),u:(0.8259067559962135,-6.999900000010719),x:(-0.01642842201996279,0.16956483605788117),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-2.469490518301797,0.026467846111587222),xp:(-0.015594721972887044,0.15648999637430758),xm:(-0.01713076618032053,-0.18492734608997766),u:(0.8259067559962137,-7.9999000000107285),x:(2.9374736459666977,-14.154798684335006),sheet_data:(log_branch_p:0,log_branch_m:-3,log_branch_x:1,e_branch:1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.002443635689955469,0.031156981066727794),xp:(-0.017130766180320594,-0.18492734608997763),xm:(-0.017379500127251722,-0.22520962585015392),u:(0.8259067559962111,-8.999900000010724),x:(-0.013017598277623029,0.12694382857518613),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.005043836866204629,0.03727197299065966),xp:(-0.01737950012725175,-0.22520962585015392),xm:(-0.012935523601068057,-0.2851911225008308),u:(0.825906755996211,-9.999900000010722),x:(-0.016139122690666204,-0.2520500961959419),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:-3,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.009983704183199597,0.04484499420457478),xp:(-0.012935523601068168,-0.2851911225008307),xm:(0.006585048506230251,-0.3783444850201329),u:(0.8259067559962098,-10.999900000010722),x:(-0.006227974631957518,-0.32652406485302565),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:-3,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.018151529315943035,0.052984258851600366),xp:(0.006585048506230137,-0.3783444850201332),xm:(0.06919133582369916,-0.5233244260619485),u:(0.8259067559962082,-11.999900000010726),x:(-0.01642842201996293,0.16956483605788097),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:-1,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,-1))),(p:(-0.0277685687530608,0.060940873423300734),xp:(0.06919133582369891,-0.5233244260619482),xm:(0.23315768719594876,-0.7382073272200409),u:(0.8259067559962081,-12.999900000010722),x:(0.13350987268341222,-0.6208477455958343),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:-3,e_branch:-1,u_branch:(Inside,Between),im_x_sign:(1,-1))),(p:(-0.03140769505134425,0.07227554691810695),xp:(0.23315768719594848,-0.7382073272200408),xm:(0.5879736502152035,-1.0679684020490827),u:(0.8259067559962082,-13.999900000010722),x:(0.3807160871328493,-0.8815370234323279),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:-3,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.012728003253993338,0.08134523025357689),xp:(0.5879736502152039,-1.0679684020490832),xm:(1.119334123519768,-1.6964500714143518),u:(0.8259067559962079,-14.999900000010722),x:(0.8493697019414395,-1.3318916226611388),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:-3,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,-1))),(p:(0.0079582652388391,0.06357826822367572),xp:(1.119334123519768,-1.6964500714143518),xm:(1.5404568947697497,-2.609733039471947),u:(0.8259067559962079,-15.999900000010722),x:(1.3519342542269488,-2.135021615283242),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:-3,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.010201073290449466,0.045505322744012554),xp:(1.5404568947697506,-2.6097330394719482),xm:(1.8236298764824674,-3.597705907174023),u:(0.8259067559962082,-16.999900000010726),x:(3.329264545743063,-23.178646433207852),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:1,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.008504887271023925,0.03511793006501657),xp:(1.8236298764824672,-3.5977059071740234),xm:(2.031015228567917,-4.6009925982671325),u:(0.8259067559962083,-17.999900000010726),x:(1.9342884969851784,-4.098544933029543),sheet_data:(log_branch_p:3,log_branch_m:-3,log_branch_x:-3,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1)))],unlocked:false)

type FigureFunction = fn(
    pxu_provider: Arc<PxuProvider>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler>;

pub const ALL_FIGURES: &[FigureFunction] = &[
    fig_u_region_min_1_h_0_k_5,
    fig_p_region_min_1_h_0_k_5,
    fig_u_region_min_1_h_01_k_5,
    fig_p_plane_h_01_k_5,
    fig_u_region_min_3,
    fig_u_region_min_2,
    fig_u_region_min_1,
    fig_u_region_0,
    fig_u_region_1,
    fig_u_region_2,
    fig_x_integration_contour_1,
    fig_x_integration_contour_2,
    fig_x_integration_contour_rr_1,
    fig_x_integration_contour_rr_2,
    fig_p_bs3_region_min_1,
    fig_u_bs3_region_min_1,
    fig_u_large_circle_1,
    fig_u_large_circle_2,
    fig_u_large_circle_3,
    fig_p_large_circle,
    fig_x_large_circle,
    fig_p_smaller_circle,
    fig_x_smaller_circle,
    fig_u_smaller_circle_1,
    fig_u_smaller_circle_2,
    fig_u_smaller_circle_3,
    fig_p_simple_path,
    fig_x_simple_path,
    fig_u_simple_path_1,
    fig_u_simple_path_2,
    fig_u_simple_path_34,
    fig_p_periodic_path,
    fig_xp_periodic_path,
    fig_xm_periodic_path,
    fig_p_plane_path_between_regions,
    fig_x_short_circle,
    fig_u_short_circle,
    fig_x_long_circle,
    fig_u_long_half_circle_1,
    fig_u_long_half_circle_2,
    fig_u_long_half_circle_3,
    fig_u_long_half_circle_4,
    fig_xp_circle_between_between,
    fig_p_circle_between_between,
    fig_xm_circle_between_between,
    fig_u_circle_between_between,
    fig_u_circle_between_inside,
    fig_u_circle_between_outside,
    fig_p_crossing_all,
    fig_xp_crossing_all,
    fig_xm_crossing_all,
    fig_xp_crossing_1,
    fig_xm_crossing_1,
    fig_u_crossing_1,
    fig_p_xpl_preimage,
    fig_p_xml_preimage,
    fig_p_plane_e_cuts,
    fig_xpl_cover,
    fig_xml_cover,
    fig_xl_crossed_point_0,
    fig_xr_crossed_point_0,
    fig_xl_crossed_point_min_1,
    fig_xr_crossed_point_min_1,
    fig_p_plane_short_cuts,
    fig_p_plane_short_cuts_rr_075,
    fig_p_plane_short_cuts_rr_200,
    fig_xp_cuts_1,
    fig_xm_cuts_1,
    fig_u_band_between_outside,
    fig_u_band_between_inside,
    fig_u_period_between_between,
    fig_p_band_between_outside,
    fig_p_band_between_inside,
    fig_p_period_between_between,
    fig_xp_band_between_outside,
    fig_xp_band_between_inside,
    fig_xp_period_between_between,
    fig_xm_band_between_inside,
    fig_xm_band_between_outside,
    fig_xm_period_between_between,
    fig_u_crossing_0,
    fig_xp_crossing_0,
    fig_xm_crossing_0,
    fig_x_typical_bound_state,
    fig_p_typical_bound_state,
    fig_p_bound_state_region_1,
    fig_p_bound_state_regions_min_1_min_2,
    fig_x_bound_state_region_1,
    fig_x_bound_state_region_min_1,
    fig_x_bound_state_region_min_2,
    fig_x_singlet_region_0,
    fig_p_two_particle_bs_0,
    fig_xp_two_particle_bs_0,
    fig_xm_two_particle_bs_0,
    fig_u_two_particle_bs_0,
    fig_u_bs_1_4_same_energy,
    fig_p_short_cut_regions_e_plus,
    fig_p_short_cut_regions_e_min,
    fig_p_physical_region_e_plus,
    fig_p_physical_region_e_minus,
    fig_xp_singlet_14,
    fig_xm_singlet_14,
    fig_u_singlet_14,
    fig_xp_singlet_23,
    fig_xm_singlet_23,
    fig_u_singlet_23,
    fig_xp_singlet_32,
    fig_xm_singlet_32,
    fig_u_singlet_32,
    fig_xp_singlet_41,
    fig_xm_singlet_41,
    fig_u_singlet_41,
    fig_bs_disp_rel_large,
    fig_bs_disp_rel_small,
    fig_bs_disp_rel_lr,
    fig_bs_disp_rel_lr0,
    fig_scallion_and_kidney,
    fig_scallion_and_kidney_3_70,
    fig_scallion_and_kidney_7_10,
    fig_scallion_and_kidney_r,
    fig_u_plane_between_between_r,
    fig_p_plane_short_cuts_r,
    fig_x_regions_outside,
    fig_x_regions_between,
    fig_x_regions_inside,
    fig_x_regions_long,
    fig_u_regions_outside,
    fig_u_regions_between,
    fig_u_regions_inside,
    fig_u_regions_between_small,
    fig_u_regions_inside_small_upper,
    fig_u_regions_inside_small_lower,
    fig_u_regions_inside_small,
    fig_u_regions_long_upper,
    fig_u_regions_long_lower,
];
