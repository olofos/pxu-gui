use crate::cache;
use crate::fig_compiler::FigureCompiler;
use crate::fig_writer::{FigureWriter, Node};
use crate::utils::{error, Settings, Size};
use indicatif::ProgressBar;

use num::complex::Complex64;
use num::Zero;
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

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::P, 0)
        .filter(|cut| {
            matches!(
                cut.typ,
                pxu::CutType::E
                    | pxu::CutType::UShortScallion(pxu::Component::Xp)
                    | pxu::CutType::Log(pxu::Component::Xp)
                    | pxu::CutType::ULongPositive(pxu::Component::Xp)
            )
        })
    {
        let options: &[&str] = match cut.typ {
            pxu::CutType::Log(_) => &["Red!50!white", "decoration={name=none}", "very thick"],
            pxu::CutType::ULongPositive(_) => &["Red!50!white", "densely dashed", "very thick"],
            _ => &[],
        };
        figure.add_cut(cut, options, pxu.consts)?;
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

fn fig_scallion_and_kidney(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "scallion-and-kidney",
        -3.1..3.1,
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
    figure.add_grid_lines(&pxu, &[])?;
    figure.add_axis()?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::Xp, 0)
        .filter(|cut| {
            matches!(
                cut.typ,
                pxu::CutType::UShortKidney(pxu::Component::Xp)
                    | pxu::CutType::UShortScallion(pxu::Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], pxu.consts)?;
    }

    figure.add_node("Scallion", Complex64::new(1.5, -2.0), &["anchor=west"])?;
    figure.add_node("Kidney", Complex64::new(-1.25, 0.5), &["anchor=east"])?;
    figure.draw("(1.5,-2.0) to[out=180,in=-45] (0.68,-1.53)", &["->"])?;
    figure.draw("(-1.25,0.5) to[out=0,in=130] (-0.75,0.3)", &["->"])?;

    figure.finish(cache, settings, pb)
}

fn get_cut_path(
    pxu: &Arc<Pxu>,
    component: pxu::Component,
    cut_type: pxu::CutType,
) -> Vec<Complex64> {
    let cut_paths = pxu
        .contours
        .get_visible_cuts(pxu, component, 0)
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

fn fig_x_regions_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "x-regions-outside",
        -3.1..3.1,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&pxu, &[])?;
    figure.add_axis()?;

    let scallion_path = get_cut_path(
        &pxu,
        pxu::Component::Xp,
        pxu::CutType::UShortScallion(pxu::Component::Xp),
    );

    let (scallion_left, scallion_right) = scallion_path.split_at(
        scallion_path.partition_point(|x| pxu::kinematics::u_of_x(*x, pxu.consts).re < 0.0),
    );

    let mut vertical_path: Vec<Complex64> = vec![];
    for segment in pxu.get_path_by_name("u vertical outside").unwrap().segments[0].iter() {
        vertical_path.extend(&segment.xp);
    }

    let mut q4_path = vec![pxu.consts.s().into()];

    q4_path.extend(scallion_right);
    q4_path.extend([
        Complex64::from(pxu.consts.s()),
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
        Complex64::from(-1.0 / pxu.consts.s()),
    ]);

    let q1_path = q4_path.iter().map(|z| z.conj()).collect::<Vec<_>>();
    let q2_path = q3_path.iter().map(|z| z.conj()).collect::<Vec<_>>();

    figure.add_plot(&["fill=yellow", "fill opacity=0.25", "draw=none"], &q1_path)?;
    figure.add_plot(&["fill=blue", "fill opacity=0.25", "draw=none"], &q2_path)?;
    figure.add_plot(&["fill=red", "fill opacity=0.25", "draw=none"], &q3_path)?;
    figure.add_plot(&["fill=green", "fill opacity=0.25", "draw=none"], &q4_path)?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::Xp, 0)
        .filter(|cut| {
            matches!(
                cut.typ,
                pxu::CutType::UShortKidney(pxu::Component::Xp)
                    | pxu::CutType::UShortScallion(pxu::Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], pxu.consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_regions_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "x-regions-between",
        -3.1..3.1,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&pxu, &[])?;
    figure.add_axis()?;

    let scallion_path = get_cut_path(
        &pxu,
        pxu::Component::Xp,
        pxu::CutType::UShortScallion(pxu::Component::Xp),
    );

    let kidney_path = get_cut_path(
        &pxu,
        pxu::Component::Xp,
        pxu::CutType::UShortKidney(pxu::Component::Xp),
    );

    let (scallion_left, scallion_right) = scallion_path.split_at(
        scallion_path.partition_point(|x| pxu::kinematics::u_of_x(*x, pxu.consts).re < 0.0),
    );

    let (kidney_left, kidney_right) = kidney_path.split_at(
        kidney_path.partition_point(|x| pxu::kinematics::u_of_x(*x, pxu.consts).re < 0.0),
    );

    let mut vertical_path = vec![];
    for segment in pxu.get_path_by_name("u vertical between").unwrap().segments[0].iter() {
        vertical_path.extend(&segment.xp);
    }

    let mut q4_path = vec![*kidney_right.last().unwrap(), pxu.consts.s().into()];

    q4_path.extend(scallion_right.iter().rev());
    q4_path.extend(&vertical_path);
    q4_path.extend(kidney_right);

    let mut q3_path = vec![
        Complex64::from(-1.0 / pxu.consts.s()),
        Complex64::from(-4.0),
    ];

    q3_path.extend(scallion_left);
    q3_path.extend(&vertical_path);
    q3_path.extend(kidney_left.iter().rev());

    let q1_path = q4_path.iter().map(|z| z.conj()).collect::<Vec<_>>();
    let q2_path = q3_path.iter().map(|z| z.conj()).collect::<Vec<_>>();

    figure.add_plot(&["fill=green", "fill opacity=0.25", "draw=none"], &q1_path)?;
    figure.add_plot(&["fill=red", "fill opacity=0.25", "draw=none"], &q2_path)?;
    figure.add_plot(&["fill=blue", "fill opacity=0.25", "draw=none"], &q3_path)?;
    figure.add_plot(&["fill=yellow", "fill opacity=0.25", "draw=none"], &q4_path)?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::Xp, 0)
        .filter(|cut| {
            matches!(
                cut.typ,
                pxu::CutType::UShortKidney(pxu::Component::Xp)
                    | pxu::CutType::UShortScallion(pxu::Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], pxu.consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_x_regions_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "x-regions-inside",
        -1.1..1.1,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
        settings,
        pb,
    )?;

    figure.component_indicator("x");
    figure.add_grid_lines(&pxu, &[])?;
    figure.add_axis()?;

    let kidney_path = get_cut_path(
        &pxu,
        pxu::Component::Xp,
        pxu::CutType::UShortKidney(pxu::Component::Xp),
    );

    let (kidney_left, kidney_right) = kidney_path.split_at(
        kidney_path.partition_point(|x| pxu::kinematics::u_of_x(*x, pxu.consts).re < 0.0),
    );

    let mut vertical_path = vec![];
    for segment in pxu.get_path_by_name("u vertical inside").unwrap().segments[0].iter() {
        vertical_path.extend(&segment.xp);
    }

    let mut q4_path = vec![Complex64::zero()];

    q4_path.extend(kidney_right.iter().rev());
    q4_path.extend(&vertical_path);

    let mut q3_path = vec![Complex64::zero(), Complex64::from(-1.0 / pxu.consts.s())];

    q3_path.extend(kidney_left);
    q3_path.extend(&vertical_path);

    let q1_path = q4_path.iter().map(|z| z.conj()).collect::<Vec<_>>();
    let q2_path = q3_path.iter().map(|z| z.conj()).collect::<Vec<_>>();

    figure.add_plot(&["fill=green", "fill opacity=0.25", "draw=none"], &q1_path)?;
    figure.add_plot(&["fill=red", "fill opacity=0.25", "draw=none"], &q2_path)?;
    figure.add_plot(&["fill=blue", "fill opacity=0.25", "draw=none"], &q3_path)?;
    figure.add_plot(&["fill=yellow", "fill opacity=0.25", "draw=none"], &q4_path)?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::Xp, 0)
        .filter(|cut| {
            matches!(
                cut.typ,
                pxu::CutType::UShortKidney(pxu::Component::Xp)
                    | pxu::CutType::UShortScallion(pxu::Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], pxu.consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "u-regions-outside",
        -5.0..5.0,
        -0.5,
        Size {
            width: 5.0,
            height: 5.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Outside,
        ::pxu::kinematics::UBranch::Outside,
    );

    figure.add_grid_lines(&pxu, &[])?;
    figure.component_indicator("u");

    figure.add_plot(
        &["fill=green", "fill opacity=0.25"],
        &vec![
            Complex64::new(0.0, -0.5),
            Complex64::new(10.0, -0.5),
            Complex64::new(10.0, -10.0),
            Complex64::new(0.0, -10.0),
        ],
    )?;

    figure.add_plot(
        &["fill=red", "fill opacity=0.25"],
        &vec![
            Complex64::new(0.0, -0.5),
            Complex64::new(-10.0, -0.5),
            Complex64::new(-10.0, -10.0),
            Complex64::new(0.0, -10.0),
        ],
    )?;

    figure.add_plot(
        &["fill=yellow", "fill opacity=0.25"],
        &vec![
            Complex64::new(0.0, -0.5),
            Complex64::new(10.0, -0.5),
            Complex64::new(10.0, 10.0),
            Complex64::new(0.0, 10.0),
        ],
    )?;

    figure.add_plot(
        &["fill=blue", "fill opacity=0.25"],
        &vec![
            Complex64::new(0.0, -0.5),
            Complex64::new(-10.0, -0.5),
            Complex64::new(-10.0, 10.0),
            Complex64::new(0.0, 10.0),
        ],
    )?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::U, 0)
        .filter(|cut| matches!(cut.typ, pxu::CutType::UShortScallion(pxu::Component::Xp)))
    {
        figure.add_cut(cut, &["black", "very thick"], pxu.consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "u-regions-between",
        -5.0..5.0,
        0.75,
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

    figure.add_grid_lines(&pxu, &[])?;
    figure.component_indicator("u");

    for i in -2..=3 {
        let shift = Complex64::new(0.0, i as f64 * pxu.consts.k() as f64);

        figure.add_plot(
            &["fill=green", "fill opacity=0.25"],
            &vec![
                Complex64::new(0.0, -0.5) + shift,
                Complex64::new(10.0, -0.5) + shift,
                Complex64::new(10.0, -3.0) + shift,
                Complex64::new(0.0, -3.0) + shift,
            ],
        )?;

        figure.add_plot(
            &["fill=red", "fill opacity=0.25"],
            &vec![
                Complex64::new(0.0, -0.5) + shift,
                Complex64::new(-10.0, -0.5) + shift,
                Complex64::new(-10.0, -3.0) + shift,
                Complex64::new(0.0, -3.0) + shift,
            ],
        )?;

        figure.add_plot(
            &["fill=yellow", "fill opacity=0.25"],
            &vec![
                Complex64::new(0.0, -0.5) + shift,
                Complex64::new(10.0, -0.5) + shift,
                Complex64::new(10.0, 2.0) + shift,
                Complex64::new(0.0, 2.0) + shift,
            ],
        )?;

        figure.add_plot(
            &["fill=blue", "fill opacity=0.25"],
            &vec![
                Complex64::new(0.0, -0.5) + shift,
                Complex64::new(-10.0, -0.5) + shift,
                Complex64::new(-10.0, 2.0) + shift,
                Complex64::new(0.0, 2.0) + shift,
            ],
        )?;
    }

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::U, 0)
        .filter(|cut| {
            matches!(
                cut.typ,
                pxu::CutType::UShortKidney(pxu::Component::Xp)
                    | pxu::CutType::UShortScallion(pxu::Component::Xp)
            )
        })
    {
        figure.add_cut(cut, &["black", "very thick"], pxu.consts)?;
    }

    figure.finish(cache, settings, pb)
}

fn fig_u_regions_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "u-regions-inside",
        -5.0..5.0,
        -3.0,
        Size {
            width: 5.0,
            height: 5.0,
        },
        pxu::Component::U,
        settings,
        pb,
    )?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Inside,
        ::pxu::kinematics::UBranch::Inside,
    );

    figure.add_grid_lines(&pxu, &[])?;
    figure.component_indicator("u");

    figure.add_plot(
        &["fill=green", "fill opacity=0.25"],
        &vec![
            Complex64::new(0.0, -3.0),
            Complex64::new(10.0, -3.0),
            Complex64::new(10.0, -10.0),
            Complex64::new(0.0, -10.0),
        ],
    )?;

    figure.add_plot(
        &["fill=red", "fill opacity=0.25"],
        &vec![
            Complex64::new(0.0, -3.0),
            Complex64::new(-10.0, -3.0),
            Complex64::new(-10.0, -10.0),
            Complex64::new(0.0, -10.0),
        ],
    )?;

    figure.add_plot(
        &["fill=yellow", "fill opacity=0.25"],
        &vec![
            Complex64::new(0.0, -3.0),
            Complex64::new(10.0, -3.0),
            Complex64::new(10.0, 10.0),
            Complex64::new(0.0, 10.0),
        ],
    )?;

    figure.add_plot(
        &["fill=blue", "fill opacity=0.25"],
        &vec![
            Complex64::new(0.0, -3.0),
            Complex64::new(-10.0, -3.0),
            Complex64::new(-10.0, 10.0),
            Complex64::new(0.0, 10.0),
        ],
    )?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::U, 0)
        .filter(|cut| matches!(cut.typ, pxu::CutType::UShortKidney(pxu::Component::Xp)))
    {
        figure.add_cut(cut, &["black", "very thick"], pxu.consts)?;
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

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::P, 0)
        .filter(|cut| {
            matches!(
                cut.typ,
                pxu::CutType::E
                    | pxu::CutType::Log(_)
                    | pxu::CutType::UShortKidney(_)
                    | pxu::CutType::UShortScallion(_)
            )
        })
    {
        figure.add_cut(cut, &[], pxu.consts)?;
    }

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
    let mut pxu = (*pxu).clone();
    if let Some(name) = paths.first() {
        let path = pxu
            .get_path_by_name(name)
            .ok_or_else(|| error(&format!("Path \"{name}\" not found")))?;
        pxu.state.points[0].sheet_data = path.segments[0][0].sheet_data.clone();
    }

    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &["semithick"])?;

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
    let mut pxu = (*pxu).clone();
    if let Some((name, _)) = paths.first() {
        let path = pxu
            .get_path_by_name(name)
            .ok_or_else(|| error(&format!("Path \"{name}\" not found")))?;
        pxu.state.points[0].sheet_data = path.segments[0][0].sheet_data.clone();
    }
    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &["semithick"])?;

    for (name, options) in paths {
        let path = pxu
            .get_path_by_name(name)
            .ok_or_else(|| error(&format!("Path \"{name}\" not found")))?;
        figure.add_path(&pxu, path, options)?;
    }

    figure.finish(cache, settings, pb)
}

#[allow(clippy::type_complexity)]
fn draw_path_figure_with_options_and_start_end_marks_and_arrows_and_labels(
    mut figure: FigureWriter,
    paths: &[(&str, &[&str], Option<&[&str]>, &[f64])],
    labels: &[(&str, Complex64, &[&str])],
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let mut pxu = (*pxu).clone();
    if let Some((name, _, _, _)) = paths.first() {
        let path = pxu
            .get_path_by_name(name)
            .ok_or_else(|| error(&format!("Path \"{name}\" not found")))?;
        pxu.state.points[0].sheet_data = path.segments[0][0].sheet_data.clone();
    }

    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &["semithick"])?;

    for (name, options, mark_options, arrow_pos) in paths {
        let path = pxu
            .get_path_by_name(name)
            .ok_or_else(|| error(&format!("Path \"{name}\" not found")))?;
        figure.add_path(&pxu, path, options)?;
        if let Some(mark_options) = mark_options {
            figure.add_path_start_end_mark(path, mark_options)?;
        }
        figure.add_path_arrows(path, arrow_pos, options)?;
    }

    for (text, pos, options) in labels {
        figure.add_node(text, *pos, options)?;
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

fn fig_p_circle_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "p-circle-between-between",
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
        &["xp circle between/between (single)"],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_circle_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-circle-between-between",
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
        &[("xp circle between/between (single)", &["solid"])],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_circle_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-circle-between-between",
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
        &[("xp circle between/between (single)", &["solid"])],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_u_circle_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-circle-between-between",
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

    draw_path_figure(
        figure,
        &["xp circle between/between"],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_u_circle_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-circle-between-outside",
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

    draw_path_figure(
        figure,
        &["xp circle between/outside L", "xp circle between/outside R"],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_u_circle_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-circle-between-inside",
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

    draw_path_figure(
        figure,
        &["xp circle between/inside L", "xp circle between/inside R"],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_p_crossing_all(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "p-crossing-all",
        -1.1..1.1,
        0.0,
        Size {
            width: 8.0,
            height: 6.0,
        },
        pxu::Component::P,
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
                r"\footnotesize 1",
                Complex64::new(0.091, 0.029),
                &["anchor=south west", "blue"],
            ),
            (
                r"\footnotesize 2",
                Complex64::new(0.091, -0.029),
                &["anchor=north west", "blue"],
            ),
            (
                r"\footnotesize 3",
                Complex64::new(0.498, 0.142),
                &["anchor=north west", "cyan"],
            ),
            (
                r"\footnotesize 4",
                Complex64::new(-0.443, -0.172),
                &["anchor=north west", "magenta"],
            ),
        ],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_xp_crossing_all(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xp-crossing-all",
        -5.0..5.0,
        0.0,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xp,
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
                r"\footnotesize 1",
                Complex64::new(2.08, -0.44),
                &["anchor=north west", "blue"],
            ),
            (
                r"\footnotesize 2",
                Complex64::new(2.58, 1.59),
                &["anchor=west", "blue"],
            ),
            (
                r"\footnotesize 3",
                Complex64::new(-0.80, -0.45),
                &["anchor=north east", "cyan"],
            ),
            (
                r"\footnotesize 4",
                Complex64::new(3.58, 2.34),
                &["anchor=west", "magenta"],
            ),
        ],
        pxu,
        cache,
        settings,
        pb,
    )
}

fn fig_xm_crossing_all(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "xm-crossing-all",
        -5.0..5.0,
        -0.7,
        Size {
            width: 6.0,
            height: 6.0,
        },
        pxu::Component::Xm,
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
                r"\footnotesize 1",
                Complex64::new(1.056, -1.734),
                &["anchor=north east", "blue"],
            ),
            (
                r"\footnotesize 2",
                Complex64::new(1.917, 0.718),
                &["anchor=south west", "blue"],
            ),
            (
                r"\footnotesize 3",
                Complex64::new(3.227, -2.985),
                &["anchor=west", "cyan"],
            ),
            (
                r"\footnotesize 4",
                Complex64::new(3.331, 1.040),
                &["anchor=west", "magenta"],
            ),
        ],
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
    "xtick={-4,-3,-2,-1,0,1,2,3,4}",
    "xticklabels={$-8\\pi$,$-6\\pi$,$-4\\pi$,$-2\\pi$,$0$,$2\\pi$,$4\\pi$,$6\\pi$,$8\\pi$}",
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
    let width: f64 = 12.0;
    let height: f64 = 6.0;

    let x_min: f64 = -4.35;
    let x_max: f64 = 1.25;
    let y_min: f64 = 0.0;
    let y_max: f64 = (x_max - x_min).abs() * 8.0 * height / width;

    let x_range = x_min..x_max;
    let y_range = y_min..y_max;

    let restrict = format!("restrict y to domain={y_min:2}:{y_max:2}");
    let axis_options = [BS_AXIS_OPTIONS, &[&restrict]].concat();

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
    _pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
    pb: &ProgressBar,
) -> Result<FigureCompiler> {
    let axis_options = BS_AXIS_OPTIONS;

    let width: f64 = 12.0;
    let height: f64 = 4.5;

    let x_min: f64 = -2.25;
    let x_max: f64 = 1.25;
    let y_min: f64 = 0.0;
    let y_max: f64 = (x_max - x_min).abs() * 8.0 * height / width;

    let x_range = x_min..x_max;
    let y_range = y_min..y_max;

    let mut figure = FigureWriter::custom_axis(
        "bs_disp_rel_small",
        x_range,
        y_range,
        Size { width, height },
        axis_options,
        settings,
        pb,
    )?;

    let colors = ["Blue", "Red", "Green", "DarkViolet", "DeepPink"];
    let mut color_it = colors.iter().cycle();

    let domain = format!("domain={x_min:.2}:{x_max:.2}");
    for m in 1..=5 {
        let plot = format!(
            "{{ sqrt(({m} + 5 * x)^2+4*4*(sin(x*180))^2) }} \
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

fn fig_bs_disp_rel_lr0(
    _pxu: Arc<Pxu>,
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
    let axis_options = [BS_AXIS_OPTIONS, &[&restrict]].concat();

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

pub const ALL_FIGURES: &[FigureFunction] = &[
    fig_xp_circle_between_between,
    fig_p_circle_between_between,
    fig_xm_circle_between_between,
    fig_u_circle_between_between,
    fig_u_circle_between_inside,
    fig_u_circle_between_outside,
    fig_p_crossing_all,
    fig_xp_crossing_all,
    fig_xm_crossing_all,
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
    fig_scallion_and_kidney,
    fig_x_regions_outside,
    fig_x_regions_between,
    fig_x_regions_inside,
    fig_u_regions_outside,
    fig_u_regions_between,
    fig_u_regions_inside,
];
