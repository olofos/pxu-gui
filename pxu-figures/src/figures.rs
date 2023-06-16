use crate::cache;
use crate::fig_compiler::FigureCompiler;
use crate::fig_writer::{FigureWriter, Node};
use crate::utils::{error, Settings, Size};
use indicatif::ProgressBar;

use num::complex::Complex64;
use pxu::GridLineComponent;
use pxu::{interpolation::PInterpolatorMut, kinematics::UBranch, Pxu};
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

fn fig_p_xpl_preimage(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-xpL-preimage",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::P, 0)
        .filter(|cut| matches!(cut.typ, pxu::CutType::E))
    {
        figure.add_cut(cut, &[], pxu.consts)?;
    }

    let k = pxu.consts.k() as f64;

    for p_range in -3..=2 {
        let p_start = p_range as f64;

        let bp1 = pxu::compute_branch_point(
            p_range,
            pxu::BranchPointType::XpPositiveAxisImXmNegative,
            pxu.consts,
        )
        .unwrap();

        let bp2 = pxu::compute_branch_point(
            p_range,
            pxu::BranchPointType::XpNegativeAxisFromAboveWithImXmNegative,
            pxu.consts,
        )
        .unwrap();

        let p0 = p_start + (bp1.p + bp2.p) / 2.0;
        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);

        for m in 1..=15 {
            p_int.goto_m(m as f64);
            p_int.write_m_node(&mut figure, "south", 1, pxu.consts)?;
        }

        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);

        for m in (-15..=0).rev() {
            p_int.goto_m(m as f64);
            p_int.write_m_node(&mut figure, "south", 1, pxu.consts)?;
        }

        // let p0 = p_start + 0.5;
        let p1 = p_start + bp1.p - 0.003;

        let m = p_range * pxu.consts.k() + 2;

        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);
        p_int.goto_m(m as f64 + 1.0 * (p_start + 0.5).signum());
        p_int.goto_p(p1);

        p_int.goto_m(m as f64);
        if p_range >= 0 {
            p_int.write_m_node(&mut figure, "north", 1, pxu.consts)?;
        } else {
            p_int.write_m_node(&mut figure, "north", -1, pxu.consts)?;
        }

        let p2 = p_start + bp1.p + 0.01;
        if p_range > 0 {
            p_int.goto_m(m as f64 - 1.0).goto_p(p2);

            for dm in (1..=4).rev() {
                p_int.goto_m((m - dm) as f64);
                p_int.write_m_node(&mut figure, "south", -1, pxu.consts)?;
            }
        }

        // if p_range > 0 {
        //     let p0 = p_start + 0.4;

        //     let mut p_int = PInterpolatorMut::xp(p0, consts);
        //     p_int.goto_m(m as f64 - 1.0);
        //     p_int.goto_p(p1);
        //     p_int.goto_m(m as f64 + 1.0);

        //     let p2 = p1;
        //     p_int.goto_p(p2);

        //     for m in ((m + 1)..(m + 4)).rev() {
        //         p_int.goto_m(m as f64);
        //         p_int.write_m_node(&mut figure, "south", consts)?;
        //     }
        // }

        let p2 = p_start + 0.2;
        if p_range != 0 {
            let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);
            p_int
                .goto_m(-p_start * k + 1.0)
                .goto_p(p_start + 0.1)
                .goto_conj()
                .goto_p(p2);
            for dm in 0..=pxu.consts.k() {
                p_int.goto_m(-p_start * k - dm as f64);
                p_int.write_m_node(&mut figure, "south", -1, pxu.consts)?;
            }
        }
    }

    figure.finish(cache, settings, pb)
}

fn fig_p_plane_e_cuts(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-plane-e-cuts",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::P, 0)
        .filter(|cut| matches!(cut.typ, pxu::CutType::E))
    {
        figure.add_cut(cut, &[], pxu.consts)?;
    }

    figure.add_plot(
        &["black"],
        &vec![Complex64::from(-5.0), Complex64::from(5.0)],
    )?;

    figure.add_plot(
        &["black"],
        &vec![Complex64::new(0.0, -5.0), Complex64::new(0.0, 5.0)],
    )?;

    for i in 0..=(2 * 5) {
        let x = -5.0 + i as f64;
        figure.add_plot(
            &["black"],
            &vec![Complex64::new(x, -0.03), Complex64::new(x, 0.03)],
        )?;
        figure.add_plot(
            &["black"],
            &vec![
                Complex64::new(x + 0.25, -0.015),
                Complex64::new(x + 0.25, 0.015),
            ],
        )?;
        figure.add_plot(
            &["black"],
            &vec![
                Complex64::new(x + 0.5, -0.015),
                Complex64::new(x + 0.5, 0.015),
            ],
        )?;
        figure.add_plot(
            &["black"],
            &vec![
                Complex64::new(x + 0.75, -0.015),
                Complex64::new(x + 0.75, 0.015),
            ],
        )?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xpl_cover(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xpL-cover",
        -5.0..5.0,
        1.9,
        Size {
            width: 6.0,
            height: 3.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;
    figure.no_component_indicator();

    figure.add_axis()?;
    for contour in pxu.contours.get_grid(pxu::Component::Xp).iter().filter(
        |line| matches!(line.component, GridLineComponent::Xp(m) if (-8.0..=6.0).contains(&m)),
    ) {
        if contour.component == GridLineComponent::Xp(1.0) {
            figure.add_grid_line(contour, &["thin", "blue"])?;
        } else {
            figure.add_grid_line(contour, &["thin", "black"])?;
        }
    }
    figure.finish(cache, settings, pb)
}

fn fig_xml_cover(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xmL-cover",
        -5.0..5.0,
        -1.9,
        Size {
            width: 6.0,
            height: 3.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;
    figure.no_component_indicator();

    figure.add_axis()?;
    for contour in pxu.contours.get_grid(pxu::Component::Xm).iter().filter(
        |line| matches!(line.component, GridLineComponent::Xm(m) if (-8.0..=6.0).contains(&m)),
    ) {
        if contour.component == GridLineComponent::Xm(1.0) {
            figure.add_grid_line(contour, &["thin", "blue"])?;
        } else {
            figure.add_grid_line(contour, &["thin", "black"])?;
        }
    }
    figure.finish(cache, settings, pb)
}

fn fig_p_plane_short_cuts(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-plane-short-cuts",
        -2.6..2.6,
        0.0,
        Size {
            width: 25.0,
            height: 10.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &[])?;

    figure.finish(cache, settings, pb)
}

fn fig_xp_cuts_1(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xp-cuts-1",
        -4.0..4.0,
        0.0,
        Size {
            width: 12.0,
            height: 12.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    figure.add_axis()?;
    for contour in pxu.contours
        .get_grid(pxu::Component::Xp)
        .iter()
        .filter(|line| matches!(line.component, GridLineComponent::Xp(m) | GridLineComponent::Xm(m) if (-10.0..).contains(&m)))
    {
        figure.add_grid_line(contour, &[])?;
    }

    figure.add_cuts(&pxu, &[])?;

    figure.finish(cache, settings, pb)
}

fn draw_path_figure(
    mut figure: FigureWriter,
    paths: &[&str],
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &[])?;

    for name in paths {
        let path = pxu
            .get_path_by_name(name)
            .ok_or_else(|| error(&format!("Path \"{name}\" not found")))?;
        figure.add_path(&pxu, path, &[])?;
    }

    figure.finish(cache, settings, pb)
}

fn draw_path_figure_with_options(
    mut figure: FigureWriter,
    paths: &[(&str, &[&str])],
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &[])?;

    for (name, options) in paths {
        let path = pxu
            .get_path_by_name(name)
            .ok_or_else(|| error(&format!("Path \"{name}\" not found")))?;
        figure.add_path(&pxu, path, options)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_period_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-period-between-between",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    draw_path_figure(
        figure,
        &["U period between/between"],
        Arc::new(pxu),
        cache,
        settings,
        pb,
    )
}

fn fig_u_band_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-band-between-outside",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Outside,
    );

    draw_path_figure(
        figure,
        &["U band between/outside"],
        Arc::new(pxu),
        cache,
        settings,
        pb,
    )
}

fn fig_u_band_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-band-between-inside",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Inside,
    );

    draw_path_figure(
        figure,
        &["U band between/inside"],
        Arc::new(pxu),
        cache,
        settings,
        pb,
    )
}

fn fig_p_band_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "p-band-between-outside",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["U band between/outside"],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_p_band_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "p-band-between-inside",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    draw_path_figure(figure, &["U band between/inside"], pxu, cache, settings, pb)
}

fn fig_xp_band_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-band-between-inside",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (UBranch::Between, UBranch::Inside);
    pxu.state.points[0].sheet_data.log_branch_p = 0;
    pxu.state.points[0].sheet_data.log_branch_m = -1;
    pxu.state.points[0].sheet_data.im_x_sign = (1, -1);

    draw_path_figure_with_options(
        figure,
        &[("U band between/inside (single)", &["solid"])],
        Arc::new(pxu),
        cache,
        settings,
        pb,
    )
}

fn fig_xp_band_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-band-between-outside",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (UBranch::Between, UBranch::Outside);
    pxu.state.points[0].sheet_data.log_branch_p = 0;
    pxu.state.points[0].sheet_data.log_branch_m = -1;
    pxu.state.points[0].sheet_data.im_x_sign = (1, -1);

    draw_path_figure_with_options(
        figure,
        &[("U band between/outside (single)", &["solid"])],
        Arc::new(pxu),
        cache,
        settings,
        pb,
    )
}

fn fig_xm_band_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-band-between-inside",
        -0.8..0.4,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (UBranch::Between, UBranch::Inside);
    pxu.state.points[0].sheet_data.log_branch_p = 0;
    pxu.state.points[0].sheet_data.log_branch_m = -1;
    pxu.state.points[0].sheet_data.im_x_sign = (1, -1);

    draw_path_figure(
        figure,
        &["U band between/inside"],
        Arc::new(pxu),
        cache,
        settings,
        pb,
    )
}

fn fig_xm_band_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-band-between-outside",
        -7.0..7.0,
        0.0,
        Size {
            width: 8.0,
            height: 16.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (UBranch::Between, UBranch::Outside);
    pxu.state.points[0].sheet_data.log_branch_p = 0;
    pxu.state.points[0].sheet_data.log_branch_m = -1;
    pxu.state.points[0].sheet_data.im_x_sign = (1, -1);

    draw_path_figure(
        figure,
        &["U band between/outside"],
        Arc::new(pxu),
        cache,
        settings,
        pb,
    )
}

fn fig_xp_period_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-period-between-between",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    draw_path_figure_with_options(
        figure,
        &[("U period between/between (single)", &["solid"])],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_period_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-period-between-between",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    draw_path_figure_with_options(
        figure,
        &[("U period between/between (single)", &["solid"])],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_p_period_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "p-period-between-between",
        -0.15..0.15,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &["U period between/between (single)"],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_u_crossing_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-crossing-0",
        -3.0..3.0,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &[
            "U crossing from 0-2pi path A",
            "U crossing from 0-2pi path B",
        ],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_crossing_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-crossing-0",
        -3.0..3.0,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &[
            "U crossing from 0-2pi path A",
            "U crossing from 0-2pi path B",
        ],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_crossing_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-crossing-0",
        -1.5..4.4,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    draw_path_figure(
        figure,
        &[
            "U crossing from 0-2pi path A",
            "U crossing from 0-2pi path B",
        ],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn draw_state_figure(
    mut figure: FigureWriter,
    state_strings: &[&str],
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let states = load_states(state_strings)?;

    let mut pxu = (*pxu).clone();
    pxu.state = states[0].clone();

    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &[])?;

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
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "p-two-particle-bs-0",
        -0.05..1.0,
        0.0,
        Size {
            width: 8.0,
            height: 4.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings, pb)
}

fn fig_xp_typical_bound_state(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xp-typical-bound-states",
        -3.5..6.5,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;
    figure.no_component_indicator();

    let state_strings = [
        "(points:[(p:(-0.01281836032081622,-0.03617430043713721),xp:(-0.5539661576009564,4.096675591673073),xm:(-0.7024897294980745,3.2176928460399083),u:(-1.7157735474931681,1.9999999999999996),x:(-0.6278118911147218,3.651492613118212),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019778339646048883,-0.041578695061571934),xp:(-0.7024897294980745,3.2176928460399083),xm:(-0.8439501836107429,2.391751872316718),u:(-1.7157735474931681,0.9999999999999993),x:(-0.7756824568522961,2.7972312015320973),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.6079768155592542,-0.000000000000000025609467106049815),xp:(-0.8439501836107431,2.3917518723167186),xm:(-0.8439501836107433,-2.3917518723167186),u:(-1.7157735474931681,-0.0000000000000004440892098500626),x:(-0.9025872691909044,-2.0021375758700994),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019778339646048887,0.04157869506157193),xp:(-0.8439501836107434,-2.391751872316718),xm:(-0.7024897294980749,-3.217692846039909),u:(-1.7157735474931686,-0.9999999999999991),x:(-0.7756824568522963,-2.7972312015320973),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.01281836032081622,0.0361743004371372),xp:(-0.7024897294980751,-3.217692846039909),xm:(-0.5539661576009569,-4.0966755916730735),u:(-1.7157735474931686,-1.9999999999999998),x:(-0.6278118911147222,-3.651492613118212),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.0369899543404076,-0.029477676458957484),xp:(3.725975442509692,2.6128313499217866),xm:(3.5128286480709265,1.3995994557612454),u:(2.7000494004152316,1.5000010188076138),x:(3.6217633112309158,2.022895894514536),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06034321575136616,-0.018323213928633217),xp:(3.512828648070947,1.3995994557612081),xm:(3.3701632658975504,0.000001507484578833207),u:(2.700049400415252,0.5000010188075885),x:(3.4147970768250535,0.7263861464447217),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06034326215107557,0.018323155770842862),xp:(3.370163265897615,0.0000015074845481910515),xm:(3.5128282084799323,-1.3995968258500417),u:(2.700049400415295,-0.49999898119243236),x:(3.4147967471340466,-0.7263832822620354),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.03698999112227798,0.029477675660386345),xp:(3.5128282084799114,-1.3995968258500804),xm:(3.7259750341536533,-2.6128289961240028),u:(2.700049400415274,-1.4999989811924586),x:(3.621762872183573,-2.0228934323008243),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1)))])"
    ];

    let states: Vec<pxu::State> = state_strings
        .iter()
        .map(|s| ron::from_str(s).map_err(|_| error("Could not load state")))
        .collect::<Result<Vec<_>>>()?;

    let mut pxu = (*pxu).clone();
    pxu.state = states[0].clone();

    figure.add_grid_lines(&pxu, &[])?;

    figure.add_plot(
        &["very thin", "lightgray"],
        &vec![Complex64::from(-10.0), Complex64::from(10.0)],
    )?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, figure.component, 0)
        .filter(|cut| matches!(cut.typ, pxu::CutType::UShortScallion(pxu::Component::Xp)))
    {
        figure.add_cut(cut, &[], pxu.consts)?;
    }

    for state in states {
        let mut points = state
            .points
            .iter()
            .map(|pt| pt.get(pxu::Component::Xp))
            .collect::<Vec<_>>();
        points.push(state.points.last().unwrap().get(pxu::Component::Xm));

        for (i, pos) in points.iter().enumerate() {
            let text = if i == 0 {
                "$\\scriptstyle x_1^+$".to_owned()
            } else if i == points.len() - 1 {
                format!("$\\scriptstyle x_{}^-$", i)
            } else {
                format!("$\\scriptstyle x_{}^- = x_{}^+$", i, i + 1)
            };
            let anchor = if pos.re < 0.0 {
                "anchor=east"
            } else {
                "anchor=west"
            };
            figure.add_node(&text, *pos, &[anchor])?;
        }

        figure.add_plot_all(&["only marks", "Blue", "mark size=0.075cm"], points)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_xp_two_particle_bs_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-two-particle-bs-0",
        -2.2..4.8,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings, pb)
}

fn fig_xm_two_particle_bs_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-two-particle-bs-0",
        -2.2..4.8,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",

    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings, pb)
}

fn fig_u_two_particle_bs_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-two-particle-bs-0",
        -2.2..4.8,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",

    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings, pb)
}

fn fig_u_bs_1_4_same_energy(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "u-bs-1-4-same-energy",
        -5.4..5.4,
        -2.5,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let state_strings = [
        "(points:[(p:(-0.49983924627304077,0.0),xp:(-0.0003500468127455447,0.693130751982731),xm:(-0.0003500468127455447,-0.693130751982731),u:(0.29060181708478217,-2.5000000000000004),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1)))])",
        "(points:[(p:(-0.026983887446552304,-0.06765648924444852),xp:(0.0020605469306089613,1.4422316508357205),xm:(-0.15775354460012647,0.929504024735109),u:(-0.2883557081916778,-0.9999998836405168),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.022627338608906006,-0.07099139905503385),xp:(-0.15775354460012575,0.9295040247351102),xm:(-0.18427779175410938,0.5747099285634751),u:(-0.2883557081916768,-1.999999883640514),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.42385965588804475,0.07099138281105592),xp:(-0.18427779175410947,0.5747099285634747),xm:(-0.15775356577239247,-0.9295039235403522),u:(-0.2883557081916773,-2.9999998836405153),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.026983888159841367,0.06765649025461998),xp:(-0.15775356577239286,-0.9295039235403516),xm:(0.0020604953634236894,-1.4422315128632799),u:(-0.28835570819167794,-3.9999998836405135),sheet_data:(log_branch_p:1,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1)))])",
    ];

    figure.set_caption("A single particle state and a four particle bound state with the same total energy and momentum and opposite charge.");

    draw_state_figure(figure, &state_strings, pxu, cache, settings, pb)
}

fn draw_p_region_plot(
    mut figure: FigureWriter,
    e_branch: i32,
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut xp_scallion_path = {
        let mut xp_scallions = pxu
            .contours
            .get_visible_cuts(&pxu, pxu::Component::P, 0)
            .filter(|cut| matches!(cut.typ, pxu::CutType::UShortScallion(pxu::Component::Xp)))
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

        let mut e_cuts = pxu
            .contours
            .get_visible_cuts(&pxu, pxu::Component::P, 0)
            .filter(|cut| {
                matches!(cut.typ, pxu::CutType::E)
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
        let mut xp_kidneys = pxu
            .contours
            .get_visible_cuts(&pxu, pxu::Component::P, 0)
            .filter(|cut| matches!(cut.typ, pxu::CutType::UShortKidney(pxu::Component::Xp)))
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

        let mut e_cut = pxu
            .contours
            .get_visible_cuts(&pxu, pxu::Component::P, 0)
            .filter(|cut| {
                matches!(cut.typ, pxu::CutType::E)
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

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.e_branch = e_branch;

    figure.add_cuts(&pxu, &[])?;

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
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "p-short-cut-regions-e-plus",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    draw_p_region_plot(figure, 1, pxu, cache, settings, pb)
}

fn fig_p_short_cut_regions_e_min(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "p-short-cut-regions-e-min",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 6.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    draw_p_region_plot(figure, -1, pxu, cache, settings, pb)
}

type FigureFunction = fn(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler>;

fn get_physical_region(pxu: &Pxu) -> Vec<Vec<Complex64>> {
    let mut physical_region = vec![];

    for p_start in [-3, -2, 1, 2] {
        let p_start = p_start as f64;
        let p0 = p_start + 1.0 / 16.0;
        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);
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

        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);
        p_int.goto_conj();
        p_int.goto_m(0.0);
        line.extend(p_int.contour().iter().rev());

        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);
        p_int.goto_m(0.0);
        line.extend(p_int.contour().into_iter());

        let mut full_line = line.clone();
        full_line.extend(line.into_iter().rev().map(|z| z.conj()));

        physical_region.push(full_line);
    }

    {
        let mut line = vec![];

        let p_start = -1.0;
        let p0 = p_start + 1.0 / 16.0;
        let p2 = p_start + 15.0 / 16.0;

        let mut p_int = PInterpolatorMut::xp(p2, pxu.consts);
        p_int.goto_m(pxu.consts.k() as f64);
        line.extend(p_int.contour().iter().rev().map(|z| z.conj()));

        let mut p_int = PInterpolatorMut::xp(p2, pxu.consts);
        p_int.goto_conj().goto_m(0.0);
        line.extend(p_int.contour().iter());

        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);
        p_int.goto_m(0.0);
        line.extend(p_int.contour().iter().rev());

        let mut full_line = line.clone();
        full_line.extend(line.into_iter().rev().map(|z| z.conj()));

        physical_region.push(full_line);
    }

    physical_region
}

fn get_crossed_region(pxu: &Pxu) -> Vec<Vec<Complex64>> {
    let mut crossed_region = vec![];

    {
        let mut line: Vec<Complex64> = vec![];

        let p_start = 0.0;
        let p0 = p_start + 1.0 / 16.0;

        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);
        p_int.goto_conj();
        p_int.goto_m(0.0);
        line.extend(p_int.contour().iter().rev());

        let mut p_int = PInterpolatorMut::xp(p0, pxu.consts);
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

        let mut p_int = PInterpolatorMut::xp(p2, pxu.consts);
        p_int.goto_m(pxu.consts.k() as f64);
        line.extend(p_int.contour().iter().rev().map(|z| z.conj()));

        let mut p_int = PInterpolatorMut::xp(p2, pxu.consts);
        p_int.goto_conj().goto_m(0.0);
        line.extend(p_int.contour().iter());

        let mut p_int = PInterpolatorMut::xp(p2, pxu.consts);
        p_int.goto_im(0.0);
        let im_z = line.last().unwrap().im;
        line.extend(p_int.contour().iter().rev().filter(|z| z.im < im_z));

        crossed_region.push(line.iter().map(|z| z.conj()).collect());
        crossed_region.push(line);
    }

    crossed_region
}

fn fig_p_physical_region_e_plus(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-physical-region-e-plus",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 4.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let physical_region = get_physical_region(&pxu);
    let crossed_region = get_crossed_region(&pxu);

    for region in physical_region {
        figure.add_plot_all(&["draw=none", "fill=Blue", "opacity=0.5"], region)?;
    }

    for region in crossed_region {
        figure.add_plot_all(&["draw=none", "fill=Red", "opacity=0.5"], region)?;
    }

    figure.add_cuts(&pxu, &[])?;

    figure.finish(cache, settings, pb)
}

fn fig_p_physical_region_e_minus(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-physical-region-e-min",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.5,
            height: 4.0,
        },
        pxu::Component::P,
        settings,
        pb,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let crossed_region = get_physical_region(&pxu);
    let physical_region = get_crossed_region(&pxu);

    for region in physical_region {
        figure.add_plot_all(&["draw=none", "fill=Blue", "opacity=0.5"], region)?;
    }

    for region in crossed_region {
        figure.add_plot_all(&["draw=none", "fill=Red", "opacity=0.5"], region)?;
    }

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.e_branch = -1;

    figure.add_cuts(&pxu, &[])?;

    figure.finish(cache, settings, pb)
}

fn draw_singlet(
    mut figure: FigureWriter,
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
    state_string: &str,
    marked_indices: &[usize],
) -> Result<FigureCompiler> {
    let state = load_state(state_string)?;
    let mut pxu = (*pxu).clone();
    pxu.state = state.clone();

    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &[])?;

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
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-singlet-41",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.06481769289200064,-0.04632014396084205),xp:(0.6773737156527935,0.24101679937073833),xm:(0.39355556208794307,0.3659765169104283),u:(2.2503158561824144,-0.9972640693939946),x:(0.5207960049771001,0.3382736317263967),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134065179824,-0.04287934452264521),xp:(0.3935555620861755,0.3659765169090202),xm:(0.22233500515739787,0.34507249230177073),u:(2.250315856189289,-1.997264069401408),x:(0.29603586257460585,0.36274180923791544),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216060976681002,0.042633420284661425),xp:(0.22233500515775476,0.34507249230145126),xm:(0.3923377926330045,-0.3660664539125623),u:(2.2503158561923926,-2.9972640693996655),x:(0.16710333623086243,0.3211911819475663),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.0645947551037885,0.04632338280244304),xp:(0.3923377926336257,-0.36606645391208686),xm:(0.6755998929977572,-0.24272408911183854),u:(2.2503158561943186,-3.9972640694026023),x:(0.5192267118211283,-0.33884808844761033),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.10930011368445881,0.00024268539559447655),xp:(0.6755998929977572,-0.2427240891118387),xm:(0.6773737156462706,0.24101679936958165),u:(2.2503158561943186,0.002735930597398628),x:(0.7857319077395628,-0.0016758790700285356),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu,
        cache,
        settings,
        pb,
        state_string,
        &[0, 1, 2, 3],
    )
}

fn fig_xm_singlet_41(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-singlet-41",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.06481769289200064,-0.04632014396084205),xp:(0.6773737156527935,0.24101679937073833),xm:(0.39355556208794307,0.3659765169104283),u:(2.2503158561824144,-0.9972640693939946),x:(0.5207960049771001,0.3382736317263967),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134065179824,-0.04287934452264521),xp:(0.3935555620861755,0.3659765169090202),xm:(0.22233500515739787,0.34507249230177073),u:(2.250315856189289,-1.997264069401408),x:(0.29603586257460585,0.36274180923791544),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216060976681002,0.042633420284661425),xp:(0.22233500515775476,0.34507249230145126),xm:(0.3923377926330045,-0.3660664539125623),u:(2.2503158561923926,-2.9972640693996655),x:(0.16710333623086243,0.3211911819475663),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.0645947551037885,0.04632338280244304),xp:(0.3923377926336257,-0.36606645391208686),xm:(0.6755998929977572,-0.24272408911183854),u:(2.2503158561943186,-3.9972640694026023),x:(0.5192267118211283,-0.33884808844761033),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.10930011368445881,0.00024268539559447655),xp:(0.6755998929977572,-0.2427240891118387),xm:(0.6773737156462706,0.24101679936958165),u:(2.2503158561943186,0.002735930597398628),x:(0.7857319077395628,-0.0016758790700285356),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(
        figure,
        pxu,
        cache,
        settings,
        pb,
        state_string,
        &[0, 1, 2, 3],
    )
}

fn fig_u_singlet_41(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-singlet-41",
        -3.1..4.6,
        -1.5,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let state_string ="(points:[(p:(-0.06481769289200064,-0.04632014396084205),xp:(0.6773737156527935,0.24101679937073833),xm:(0.39355556208794307,0.3659765169104283),u:(2.2503158561824144,-0.9972640693939946),x:(0.5207960049771001,0.3382736317263967),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134065179824,-0.04287934452264521),xp:(0.3935555620861755,0.3659765169090202),xm:(0.22233500515739787,0.34507249230177073),u:(2.250315856189289,-1.997264069401408),x:(0.29603586257460585,0.36274180923791544),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216060976681002,0.042633420284661425),xp:(0.22233500515775476,0.34507249230145126),xm:(0.3923377926330045,-0.3660664539125623),u:(2.2503158561923926,-2.9972640693996655),x:(0.16710333623086243,0.3211911819475663),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.0645947551037885,0.04632338280244304),xp:(0.3923377926336257,-0.36606645391208686),xm:(0.6755998929977572,-0.24272408911183854),u:(2.2503158561943186,-3.9972640694026023),x:(0.5192267118211283,-0.33884808844761033),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.10930011368445881,0.00024268539559447655),xp:(0.6755998929977572,-0.2427240891118387),xm:(0.6773737156462706,0.24101679936958165),u:(2.2503158561943186,0.002735930597398628),x:(0.7857319077395628,-0.0016758790700285356),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)";
    draw_singlet(
        figure,
        pxu,
        cache,
        settings,
        pb,
        state_string,
        &[0, 1, 2, 3],
    )
}

fn fig_xp_singlet_32(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-singlet-32",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.0918635850967006,-0.037587502213391646),xp:(0.785884223705366,0.0000000000000002220446049250313),xm:(0.5200361660196523,0.3386309516954546),u:(2.2500748563450794,-0.5000000000000003),x:(0.6765622619422568,0.24195091368028965),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04931502967968751,-0.044946057622269636),xp:(0.5200361660196524,0.3386309516954545),xm:(0.29556714680693774,0.3627151161370183),u:(2.2500748563450794,-1.5),x:(0.392950187668455,0.36607556161166316),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7176427704472238,-0.000000000000000019937695239947602),xp:(0.2955671468069379,0.36271511613701846),xm:(0.29556714680693785,-0.3627151161370184),u:(2.2500748563450785,-2.499999999999999),x:(0.2219764434485283,0.34498404739256483),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931502967968751,0.044946057622269636),xp:(0.29556714680693774,-0.3627151161370183),xm:(0.5200361660196524,-0.3386309516954545),u:(2.2500748563450794,-3.4999999999999996),x:(0.39295018766845496,-0.36607556161166327),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09186358509670066,0.03758750221339164),xp:(0.5200361660196525,-0.33863095169545443),xm:(0.785884223705366,0.0000000000000003608224830031759),u:(2.2500748563450794,0.4999999999999998),x:(0.676562261942257,-0.2419509136802895),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[1, 2, 3])
}

fn fig_xm_singlet_32(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-singlet-32",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.0918635850967006,-0.037587502213391646),xp:(0.785884223705366,0.0000000000000002220446049250313),xm:(0.5200361660196523,0.3386309516954546),u:(2.2500748563450794,-0.5000000000000003),x:(0.6765622619422568,0.24195091368028965),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04931502967968751,-0.044946057622269636),xp:(0.5200361660196524,0.3386309516954545),xm:(0.29556714680693774,0.3627151161370183),u:(2.2500748563450794,-1.5),x:(0.392950187668455,0.36607556161166316),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7176427704472238,-0.000000000000000019937695239947602),xp:(0.2955671468069379,0.36271511613701846),xm:(0.29556714680693785,-0.3627151161370184),u:(2.2500748563450785,-2.499999999999999),x:(0.2219764434485283,0.34498404739256483),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931502967968751,0.044946057622269636),xp:(0.29556714680693774,-0.3627151161370183),xm:(0.5200361660196524,-0.3386309516954545),u:(2.2500748563450794,-3.4999999999999996),x:(0.39295018766845496,-0.36607556161166327),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09186358509670066,0.03758750221339164),xp:(0.5200361660196525,-0.33863095169545443),xm:(0.785884223705366,0.0000000000000003608224830031759),u:(2.2500748563450794,0.4999999999999998),x:(0.676562261942257,-0.2419509136802895),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[1, 2, 3])
}

fn fig_u_singlet_32(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-singlet-32",
        -3.1..4.6,
        -1.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.0918635850967006,-0.037587502213391646),xp:(0.785884223705366,0.0000000000000002220446049250313),xm:(0.5200361660196523,0.3386309516954546),u:(2.2500748563450794,-0.5000000000000003),x:(0.6765622619422568,0.24195091368028965),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04931502967968751,-0.044946057622269636),xp:(0.5200361660196524,0.3386309516954545),xm:(0.29556714680693774,0.3627151161370183),u:(2.2500748563450794,-1.5),x:(0.392950187668455,0.36607556161166316),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7176427704472238,-0.000000000000000019937695239947602),xp:(0.2955671468069379,0.36271511613701846),xm:(0.29556714680693785,-0.3627151161370184),u:(2.2500748563450785,-2.499999999999999),x:(0.2219764434485283,0.34498404739256483),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931502967968751,0.044946057622269636),xp:(0.29556714680693774,-0.3627151161370183),xm:(0.5200361660196524,-0.3386309516954545),u:(2.2500748563450794,-3.4999999999999996),x:(0.39295018766845496,-0.36607556161166327),sheet_data:(log_branch_p:1,log_branch_m:-1,log_branch_x:-1,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09186358509670066,0.03758750221339164),xp:(0.5200361660196525,-0.33863095169545443),xm:(0.785884223705366,0.0000000000000003608224830031759),u:(2.2500748563450794,0.4999999999999998),x:(0.676562261942257,-0.2419509136802895),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[1, 2, 3])
}

fn fig_xp_singlet_23(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-singlet-23",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.064817690638922,-0.04632014058248584),xp:(0.6773736720447697,0.24101678917659286),xm:(0.39355554871074094,0.3659764991995006),u:(2.250315939687509,-0.9972641231359414),x:(0.5207959807194622,0.33827361344245904),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134011794477,-0.042879342951094745),xp:(0.39355554871074067,0.3659764991995013),xm:(0.22233500194749478,0.34507247933376406),u:(2.250315939687506,-1.9972641231359423),x:(0.2960358555274206,0.3627417937862914),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216061057006049,0.04263342355344563),xp:(0.22233500194749445,0.3450724793337641),xm:(0.3923378032288628,-0.3660664344918713),u:(2.2503159396875043,-2.9972641231359445),x:(0.16710333534746072,0.32119117129204844),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.06459475724215495,0.04632337938493029),xp:(0.39233780322886325,-0.36606643449187204),xm:(0.6755998845174871,-0.24272404535577444),u:(2.2503159396875008,1.0027358768640537),x:(0.5192267310835156,-0.3388480606808871),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.10930010734366312,0.00024268100631728482),xp:(0.6755998866881463,-0.2427240505990194),xm:(0.6773736772251796,0.2410167915569991),u:(2.2503159279047136,0.0027358814445184176),x:(0.7857318639819022,-0.0016758487182760083),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[1, 2])
}

fn fig_xm_singlet_23(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-singlet-23",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.064817690638922,-0.04632014058248584),xp:(0.6773736720447697,0.24101678917659286),xm:(0.39355554871074094,0.3659764991995006),u:(2.250315939687509,-0.9972641231359414),x:(0.5207959807194622,0.33827361344245904),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134011794477,-0.042879342951094745),xp:(0.39355554871074067,0.3659764991995013),xm:(0.22233500194749478,0.34507247933376406),u:(2.250315939687506,-1.9972641231359423),x:(0.2960358555274206,0.3627417937862914),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216061057006049,0.04263342355344563),xp:(0.22233500194749445,0.3450724793337641),xm:(0.3923378032288628,-0.3660664344918713),u:(2.2503159396875043,-2.9972641231359445),x:(0.16710333534746072,0.32119117129204844),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.06459475724215495,0.04632337938493029),xp:(0.39233780322886325,-0.36606643449187204),xm:(0.6755998845174871,-0.24272404535577444),u:(2.2503159396875008,1.0027358768640537),x:(0.5192267310835156,-0.3388480606808871),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.10930010734366312,0.00024268100631728482),xp:(0.6755998866881463,-0.2427240505990194),xm:(0.6773736772251796,0.2410167915569991),u:(2.2503159279047136,0.0027358814445184176),x:(0.7857318639819022,-0.0016758487182760083),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[1, 2])
}

fn fig_u_singlet_23(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-singlet-23",
        -3.1..4.6,
        -1.5,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.064817690638922,-0.04632014058248584),xp:(0.6773736720447697,0.24101678917659286),xm:(0.39355554871074094,0.3659764991995006),u:(2.250315939687509,-0.9972641231359414),x:(0.5207959807194622,0.33827361344245904),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.03968134011794477,-0.042879342951094745),xp:(0.39355554871074067,0.3659764991995013),xm:(0.22233500194749478,0.34507247933376406),u:(2.250315939687506,-1.9972641231359423),x:(0.2960358555274206,0.3627417937862914),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7216061057006049,0.04263342355344563),xp:(0.22233500194749445,0.3450724793337641),xm:(0.3923378032288628,-0.3660664344918713),u:(2.2503159396875043,-2.9972641231359445),x:(0.16710333534746072,0.32119117129204844),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.06459475724215495,0.04632337938493029),xp:(0.39233780322886325,-0.36606643449187204),xm:(0.6755998845174871,-0.24272404535577444),u:(2.2503159396875008,1.0027358768640537),x:(0.5192267310835156,-0.3388480606808871),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.10930010734366312,0.00024268100631728482),xp:(0.6755998866881463,-0.2427240505990194),xm:(0.6773736772251796,0.2410167915569991),u:(2.2503159279047136,0.0027358814445184176),x:(0.7857318639819022,-0.0016758487182760083),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[1, 2])
}

fn fig_xp_singlet_14(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-singlet-14",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.09185221149636245,-0.037572722189714455),xp:(0.7857363886452503,0.0000004328254604446524),xm:(0.5200106363475369,0.3385618195950395),u:(2.2503161408013796,-0.5000007065959058),x:(0.676486747365414,0.24187289813934523),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.04931600633410893,-0.0449403973338789),xp:(0.5200106363475344,0.338561819595029),xm:(0.29557299472051746,0.3626743175215065),u:(2.2503161408014147,-1.5000007065959013),x:(0.392946068121917,0.36602187168832023),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.717663444470969,0.00000006054071687339567),xp:(0.2955729947205189,0.3626743175215076),xm:(0.2955732335644112,-0.36267435245574203),u:(2.2503161408014094,-2.500000706595892),x:(0.22198686543101423,0.3449533442179103),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931603946892371,0.044940403147529916),xp:(0.2955732335644095,-0.36267435245574087),xm:(0.5200110416414399,-0.3385616712335204),u:(2.2503161408014156,1.499999293404119),x:(0.392946382629357,-0.36602184846097735),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.09185229822963642,0.03757265583534658),xp:(0.5200110416414421,-0.33856167123353087),xm:(0.7857363886452495,0.00000043282544220923924),u:(2.250316140801381,0.4999992934041242),x:(0.6764872054840881,-0.24187245720745892),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[2])
}

fn fig_xm_singlet_14(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-singlet-14",
        -1.1..1.9,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xm,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.09185221149636245,-0.037572722189714455),xp:(0.7857363886452503,0.0000004328254604446524),xm:(0.5200106363475369,0.3385618195950395),u:(2.2503161408013796,-0.5000007065959058),x:(0.676486747365414,0.24187289813934523),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.04931600633410893,-0.0449403973338789),xp:(0.5200106363475344,0.338561819595029),xm:(0.29557299472051746,0.3626743175215065),u:(2.2503161408014147,-1.5000007065959013),x:(0.392946068121917,0.36602187168832023),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.717663444470969,0.00000006054071687339567),xp:(0.2955729947205189,0.3626743175215076),xm:(0.2955732335644112,-0.36267435245574203),u:(2.2503161408014094,-2.500000706595892),x:(0.22198686543101423,0.3449533442179103),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931603946892371,0.044940403147529916),xp:(0.2955732335644095,-0.36267435245574087),xm:(0.5200110416414399,-0.3385616712335204),u:(2.2503161408014156,1.499999293404119),x:(0.392946382629357,-0.36602184846097735),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.09185229822963642,0.03757265583534658),xp:(0.5200110416414421,-0.33856167123353087),xm:(0.7857363886452495,0.00000043282544220923924),u:(2.250316140801381,0.4999992934041242),x:(0.6764872054840881,-0.24187245720745892),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[2])
}

fn fig_u_singlet_14(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-singlet-14",
        -3.1..4.6,
        -1.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let state_string =
        "(points:[(p:(-0.09185221149636245,-0.037572722189714455),xp:(0.7857363886452503,0.0000004328254604446524),xm:(0.5200106363475369,0.3385618195950395),u:(2.2503161408013796,-0.5000007065959058),x:(0.676486747365414,0.24187289813934523),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.04931600633410893,-0.0449403973338789),xp:(0.5200106363475344,0.338561819595029),xm:(0.29557299472051746,0.3626743175215065),u:(2.2503161408014147,-1.5000007065959013),x:(0.392946068121917,0.36602187168832023),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.717663444470969,0.00000006054071687339567),xp:(0.2955729947205189,0.3626743175215076),xm:(0.2955732335644112,-0.36267435245574203),u:(2.2503161408014094,-2.500000706595892),x:(0.22198686543101423,0.3449533442179103),sheet_data:(log_branch_p:0,log_branch_m:-1,log_branch_x:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.04931603946892371,0.044940403147529916),xp:(0.2955732335644095,-0.36267435245574087),xm:(0.5200110416414399,-0.3385616712335204),u:(2.2503161408014156,1.499999293404119),x:(0.392946382629357,-0.36602184846097735),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.09185229822963642,0.03757265583534658),xp:(0.5200110416414421,-0.33856167123353087),xm:(0.7857363886452495,0.00000043282544220923924),u:(2.250316140801381,0.4999992934041242),x:(0.6764872054840881,-0.24187245720745892),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)"
    ;

    draw_singlet(figure, pxu, cache, settings, pb, state_string, &[2])
}

const BS_AXIS_OPTIONS: &[&str] = &[
    "axis x line=bottom",
    "axis y line=middle",
    "xtick={-4,-3,-2,-1,0,1}",
    "xticklabels={$-8\\pi$,$-6\\pi$,$-4\\pi$,$-2\\pi$,$0$,$2\\pi$}",
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

fn fig_bs_disp_rel_large(
    _pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let axis_options = [BS_AXIS_OPTIONS, &["restrict y to domain=0:22.4"]].concat();

    let mut figure = FigureWriter::custom_axis(
        "bs_disp_rel_large",
        -4.35..1.25,
        0.0..22.4,
        Size {
            width: 12.0,
            height: 6.0,
        },
        &axis_options,
        settings,
        pb,
    )?;

    let colors = ["Black", "Blue", "Red", "Green"];
    let mut color_it = colors.iter().cycle();

    for m in 1..=43 {
        let mut plot = format!("{{ sqrt(({m} + 5 * x)^2+4*4*(sin(x*180))^2) }}");
        let mut options = vec!["domain=-4.35:1.25", "mark=none", "samples=400"];
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
    _pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let axis_options = BS_AXIS_OPTIONS;

    let mut figure = FigureWriter::custom_axis(
        "bs_disp_rel_small",
        -1.75..0.75,
        0.0..10.0,
        Size {
            width: 12.0,
            height: 6.0,
        },
        &axis_options,
        settings,
        pb,
    )?;

    let colors = ["Black", "Blue", "Red", "Green", "DarkViolet"];
    let mut color_it = colors.iter().cycle();

    for m in 1..=5 {
        let plot = format!(
            "{{ sqrt(({m} + 5 * x)^2+4*4*(sin(x*180))^2) }} \
             node [pos=0,left,black] {{$\\scriptstyle {m}$}} \
             node [pos=1,right,black] {{$\\scriptstyle {m}$}}"
        );

        let options = [
            "domain=-1.75:0.75",
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
    _pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let axis_options = [BS_AXIS_OPTIONS, &["restrict y to domain=0:18"]].concat();

    let mut figure = FigureWriter::custom_axis(
        "bs_disp_rel_lr0",
        -2.25..2.25,
        0.0..18.0,
        Size {
            width: 12.0,
            height: 6.0,
        },
        &axis_options,
        settings,
        pb,
    )?;

    for m in 1..=29 {
        let plot = format!("{{ sqrt(({m} + 5 * x)^2+4*4*(sin(x*180))^2) }}");
        let options = [
            "domain=-2.25:2.25",
            "mark=none",
            "samples=400",
            "LightSlateBlue",
        ];

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

    let plot = format!("{{ sqrt((5 * x)^2+4*4*(sin(x*180))^2) }}");
    let options = ["domain=-2.25:2.25", "mark=none", "samples=400", "Black"];

    figure.add_plot_custom(&options, &plot)?;

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

pub const ALL_FIGURES: &[FigureFunction] = &[
    fig_p_xpl_preimage,
    fig_p_plane_e_cuts,
    fig_xpl_cover,
    fig_xml_cover,
    fig_p_plane_short_cuts,
    fig_xp_cuts_1,
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
    fig_xp_typical_bound_state,
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
    fig_bs_disp_rel_lr0,
];
