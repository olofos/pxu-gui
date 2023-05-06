use crate::cache;
use crate::fig_compiler::FigureCompiler;
use crate::fig_writer::{FigureWriter, Node};
use crate::utils::{error, Settings, Size};

use num::complex::Complex64;
use pxu::GridLineComponent;
use pxu::{interpolation::PInterpolatorMut, kinematics::UBranch, Pxu};
use std::io::Result;
use std::sync::Arc;

fn fig_p_xpl_preimage(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-xpL-preimage",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.0,
            height: 6.0,
        },
        pxu::Component::P,
        pxu::UCutType::Long,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::P, pxu::UCutType::Long, 0)
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

    figure.finish(cache, settings)
}

fn fig_xpl_cover(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
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
        pxu::UCutType::Long,
        settings,
    )?;

    figure.add_axis()?;
    for contour in pxu.contours.get_grid(pxu::Component::Xp).iter().filter(
        |line| matches!(line.component, GridLineComponent::Xp(m) if (-8.0..=6.0).contains(&m)),
    ) {
        figure.add_grid_line(contour, &["thin", "black"])?;
    }
    figure.finish(cache, settings)
}

fn fig_p_plane_long_cuts_regions(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-plane-long-cuts-regions",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.0,
            height: 6.0,
        },
        pxu::Component::P,
        pxu::UCutType::Long,
        settings,
    )?;

    let color_physical = "blue!10";
    let color_mirror_p = "red!10";
    let color_mirror_m = "green!10";

    {
        let x1 = figure.bounds.x_range.start - 0.25;
        let x2 = figure.bounds.x_range.end + 0.25;
        let y1 = figure.bounds.y_range.start - 0.25;
        let y2 = figure.bounds.y_range.end + 0.25;

        figure.add_plot_all(
            &[format!("fill={color_physical}").as_str()],
            vec![
                Complex64::new(x1, y1),
                Complex64::new(x1, y2),
                Complex64::new(x2, y2),
                Complex64::new(x2, y1),
            ],
        )?;
    }

    for cut in pxu
        .contours
        .get_visible_cuts(&pxu, pxu::Component::P, pxu::UCutType::Long, 0)
    {
        let color_mirror = match cut.typ {
            pxu::CutType::ULongPositive(pxu::Component::Xp)
            | pxu::CutType::ULongNegative(pxu::Component::Xp) => color_mirror_p,
            pxu::CutType::ULongPositive(pxu::Component::Xm)
            | pxu::CutType::ULongNegative(pxu::Component::Xm) => color_mirror_m,
            _ => {
                continue;
            }
        };

        let mut cropped_path = figure.crop(&cut.path);
        if cropped_path.len() >= 2 {
            let len = cropped_path.len();
            let start = cropped_path[0];
            let mid = cropped_path[len / 2];
            let end = cropped_path[len - 1];

            cropped_path.push(Complex64::new(mid.re.round(), end.im));
            cropped_path.push(Complex64::new(mid.re.round(), start.im));

            figure.add_plot_all(
                &["draw=none", format!("fill={color_mirror}").as_str()],
                cropped_path,
            )?;
        }
    }

    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &[])?;

    figure.finish(cache, settings)
}

fn fig_p_plane_short_cuts(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
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
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;
    figure.add_cuts(&pxu, &[])?;

    figure.finish(cache, settings)
}

fn fig_xp_cuts_1(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
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
        pxu::UCutType::Short,
        settings,
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

    figure.finish(cache, settings)
}

fn fig_u_period_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "u-period-between-between",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        pxu::Component::U,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Between,
    );

    let path = pxu
        .get_path_by_name("U period between/between")
        .ok_or_else(|| error("Path not found"))?;

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_u_band_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "u-band-between-outside",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        pxu::Component::U,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Outside,
    );

    let path = pxu
        .get_path_by_name("U band between/outside")
        .ok_or_else(|| error("Path not found"))?;

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_u_band_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "u-band-between-inside",
        -6.0..4.0,
        0.25,
        Size {
            width: 5.0,
            height: 12.5,
        },
        pxu::Component::U,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (
        ::pxu::kinematics::UBranch::Between,
        ::pxu::kinematics::UBranch::Inside,
    );

    let path = pxu
        .get_path_by_name("U band between/inside")
        .ok_or_else(|| error("Path not found"))?;

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_p_band_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-band-between-outside",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.0,
            height: 6.0,
        },
        pxu::Component::P,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let path = pxu
        .get_path_by_name("U band between/outside")
        .ok_or_else(|| error("Path not found"))?;

    let mut pxu = (*pxu).clone();
    pxu.state = path.base_path.start.clone();

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_p_band_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-band-between-inside",
        -2.6..2.6,
        0.0,
        Size {
            width: 15.0,
            height: 6.0,
        },
        pxu::Component::P,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;
    let path = pxu
        .get_path_by_name("U band between/inside")
        .ok_or_else(|| error("Path not found"))?;

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_xp_band_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xp-band-between-inside",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xp,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;
    let path = pxu
        .get_path_by_name("U band between/inside (single)")
        .ok_or_else(|| error("Path not found"))?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (UBranch::Between, UBranch::Inside);
    pxu.state.points[0].sheet_data.log_branch_p = 0;
    pxu.state.points[0].sheet_data.log_branch_m = -1;
    pxu.state.points[0].sheet_data.im_x_sign = (1, -1);

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &["solid"])?;

    figure.finish(cache, settings)
}

fn fig_xp_band_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xp-band-between-outside",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xp,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let path = pxu
        .get_path_by_name("U band between/outside (single)")
        .ok_or_else(|| error("Path not found"))?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (UBranch::Between, UBranch::Outside);
    pxu.state.points[0].sheet_data.log_branch_p = 0;
    pxu.state.points[0].sheet_data.log_branch_m = -1;
    pxu.state.points[0].sheet_data.im_x_sign = (1, -1);

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &["solid"])?;

    figure.finish(cache, settings)
}

fn fig_xm_band_between_inside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xm-band-between-inside",
        -0.8..0.4,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xm,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let path = pxu
        .get_path_by_name("U band between/inside")
        .ok_or_else(|| error("Path not found"))?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (UBranch::Between, UBranch::Inside);
    pxu.state.points[0].sheet_data.log_branch_p = 0;
    pxu.state.points[0].sheet_data.log_branch_m = -1;
    pxu.state.points[0].sheet_data.im_x_sign = (1, -1);

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_xm_band_between_outside(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xm-band-between-outside",
        -7.0..7.0,
        0.0,
        Size {
            width: 8.0,
            height: 16.0,
        },
        pxu::Component::Xm,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;

    let path = pxu
        .get_path_by_name("U band between/outside")
        .ok_or_else(|| error("Path not found"))?;

    let mut pxu = (*pxu).clone();
    pxu.state.points[0].sheet_data.u_branch = (UBranch::Between, UBranch::Outside);
    pxu.state.points[0].sheet_data.log_branch_p = 0;
    pxu.state.points[0].sheet_data.log_branch_m = -1;
    pxu.state.points[0].sheet_data.im_x_sign = (1, -1);

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_xp_period_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xp-period-between-between",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xp,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;
    let path = pxu
        .get_path_by_name("U period between/between (single)")
        .ok_or_else(|| error("Path not found"))?;

    let mut pxu = (*pxu).clone();
    pxu.state = path.base_path.start.clone();

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_xm_period_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "xm-period-between-between",
        -3.1..2.1,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::Xm,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;
    let path = pxu
        .get_path_by_name("U period between/between (single)")
        .ok_or_else(|| error("Path not found"))?;

    let mut pxu = (*pxu).clone();
    pxu.state = path.base_path.start.clone();

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn fig_p_period_between_between(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let mut figure = FigureWriter::new(
        "p-period-between-between",
        -0.15..0.15,
        0.0,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::P,
        pxu::UCutType::Short,
        settings,
    )?;

    figure.add_grid_lines(&pxu, &[])?;
    let path = pxu
        .get_path_by_name("U period between/between (single)")
        .ok_or_else(|| error("Path not found"))?;

    let mut pxu = (*pxu).clone();
    pxu.state = path.base_path.start.clone();

    figure.add_cuts(&pxu, &[])?;
    figure.add_path(&pxu, path, &[])?;

    figure.finish(cache, settings)
}

fn draw_state_figure(
    mut figure: FigureWriter,
    state_strings: &[&str],
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let states: Vec<pxu::State> = state_strings
        .iter()
        .map(|s| ron::from_str(s).map_err(|_| error("Could not load state")))
        .collect::<Result<Vec<_>>>()?;

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
        let points = state
            .points
            .iter()
            .map(|pt| pt.get(figure.component))
            .collect::<Vec<_>>();

        figure.add_plot_all(&["only marks", color, mark, "mark size=0.075cm"], points)?;
    }
    figure.finish(cache, settings)
}

fn fig_p_two_particle_bs_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
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
        pxu::UCutType::Short,
        settings,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings)
}

fn fig_xp_two_particle_bs_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
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
        pxu::UCutType::Short,
        settings,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",

    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings)
}

fn fig_xm_two_particle_bs_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
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
        pxu::UCutType::Short,
        settings,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",

    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings)
}

fn fig_u_two_particle_bs_0(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
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
        pxu::UCutType::Short,
        settings,
    )?;

    let state_strings = [
        "(points:[(p:(0.049906029903425714,-0.011317561918482518),xp:(4.075425564166025,1.3215262509273769),xm:(3.990254347756956,-0.00000000000008060219158778636),u:(3.139628139566713,0.49999999999994027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(0.04990602990342423,0.011317561918484643),xp:(3.990254347756972,-0.00000000000007505107646466058),xm:(4.075425564166056,-1.321526250927521),u:(3.1396281395667245,-0.5000000000000554),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))])",
        "(points:[(p:(0.004107548537993523,-0.07848376696376784),xp:(1.5017763385170317,2.066585116519383),xm:(0.9494180269531781,1.238002479091183),u:(0.9855333457443732,0.4999999999459174),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.29586076213838275,0.07848376697071423),xp:(0.9494180269531776,1.2380024790911828),xm:(1.5017763385645666,-2.0665851166226674),u:(0.9855333457443731,-0.5000000000540827),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",
        "(points:[(p:(0.2955484673695275,-0.07853446096510001),xp:(1.503716303147816,2.0656922379697886),xm:(0.9506849827846514,-1.236725796907908),u:(0.9875645002911329,0.49999999999534983),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Between),im_x_sign:(1,1))),(p:(0.0041589403041424845,0.07853446096569741),xp:(0.9506849827846514,-1.2367257969079077),xm:(1.5037163031519056,-2.0656922379786726),u:(0.9875645002911335,-0.5000000000046495),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Outside),im_x_sign:(1,1)))])",

    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings)
}

fn fig_u_bs_1_4_same_energy(
    pxu: Arc<Pxu>,
    cache: Arc<cache::Cache>,
    settings: &Settings,
) -> Result<FigureCompiler> {
    let figure = FigureWriter::new(
        "u-bs-1-4-same-energy",
        -5.4..5.4,
        -2.5,
        Size {
            width: 8.0,
            height: 8.0,
        },
        pxu::Component::U,
        pxu::UCutType::Short,
        settings,
    )?;

    let state_strings = [
        "(points:[(p:(-0.49983924627304077,0.0),xp:(-0.0003500468127455447,0.693130751982731),xm:(-0.0003500468127455447,-0.693130751982731),u:(0.29060181708478217,-2.5000000000000004),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,-1)))])",
        "(points:[(p:(-0.026983887446552304,-0.06765648924444852),xp:(0.0020605469306089613,1.4422316508357205),xm:(-0.15775354460012647,0.929504024735109),u:(-0.2883557081916778,-0.9999998836405168),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.022627338608906006,-0.07099139905503385),xp:(-0.15775354460012575,0.9295040247351102),xm:(-0.18427779175410938,0.5747099285634751),u:(-0.2883557081916768,-1.999999883640514),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(1,-1))),(p:(-0.42385965588804475,0.07099138281105592),xp:(-0.18427779175410947,0.5747099285634747),xm:(-0.15775356577239247,-0.9295039235403522),u:(-0.2883557081916773,-2.9999998836405153),sheet_data:(log_branch_p:0,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.026983888159841367,0.06765649025461998),xp:(-0.15775356577239286,-0.9295039235403516),xm:(0.0020604953634236894,-1.4422315128632799),u:(-0.28835570819167794,-3.9999998836405135),sheet_data:(log_branch_p:1,log_branch_m:-1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1)))])",
    ];

    draw_state_figure(figure, &state_strings, pxu, cache, settings)
}

type FigureFunction =
    fn(pxu: Arc<Pxu>, cache: Arc<cache::Cache>, settings: &Settings) -> Result<FigureCompiler>;

pub const ALL_FIGURES: &[FigureFunction] = &[
    fig_p_xpl_preimage,
    fig_xpl_cover,
    fig_p_plane_long_cuts_regions,
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
    fig_p_two_particle_bs_0,
    fig_xp_two_particle_bs_0,
    fig_xm_two_particle_bs_0,
    fig_u_two_particle_bs_0,
    fig_u_bs_1_4_same_energy,
];
