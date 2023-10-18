use crate::ContourProvider;
use num::complex::Complex64;
use pxu::{kinematics::CouplingConstants, path::SavedPath};
use std::f64::consts::{PI, TAU};

trait Goto {
    fn goto(
        &mut self,
        component: pxu::Component,
        new_value: impl Into<Complex64>,
        contours: &pxu::Contours,
        consts: CouplingConstants,
        steps: usize,
    );

    fn follow_path(
        &mut self,
        component: pxu::Component,
        path: &[[f64; 2]],
        contours: &pxu::Contours,
        consts: CouplingConstants,
    );
}

impl Goto for pxu::State {
    fn goto(
        &mut self,
        component: pxu::Component,
        new_value: impl Into<Complex64>,
        contours: &pxu::Contours,
        consts: CouplingConstants,
        steps: usize,
    ) {
        let z0 = self.points[0].get(component);
        let z1 = new_value.into();

        for i in 0..=steps {
            let z = z0 + (i as f64 / steps as f64) * (z1 - z0);
            self.update(0, component, z, contours, consts);
        }

        if (self.points[0].get(component) - z1).norm() > 1.0e-6 {
            eprintln!(
                "Could not goto ({})",
                (self.points[0].get(component) - z1).norm()
            );
        }
    }

    fn follow_path(
        &mut self,
        component: pxu::Component,
        path: &[[f64; 2]],
        contours: &pxu::Contours,
        consts: CouplingConstants,
    ) {
        for &[re, im] in path {
            self.goto(component, Complex64::new(re, im), contours, consts, 15);
        }
    }
}

fn create_xp_circle_between_path(
    name: &str,
    mut start: pxu::State,
    start_rev: f64,
    end_rev: f64,
    contours: &pxu::Contours,
    consts: CouplingConstants,
) -> SavedPath {
    let center = Complex64::new(-0.458742, 0.20995);
    let radius = 0.907159 * 1.03;

    let steps = 256.0;

    let mut path = vec![];

    for i in 0..=(start_rev.abs() * steps) as i32 {
        let theta = start_rev.signum() * TAU * (i as f64 / steps - 0.5);
        let xp = center + Complex64::from_polar(radius, theta);
        start.update(0, pxu::Component::Xp, xp, contours, consts);
    }

    let steps = 256.0;

    for i in 0..=((end_rev - start_rev).abs() * steps) as i32 {
        let theta = TAU * (start_rev + (end_rev - start_rev).signum() * i as f64 / steps - 0.5);
        let xp = center + Complex64::from_polar(radius, theta);
        path.push(xp);
    }

    pxu::path::SavedPath::new(name, path, start, pxu::Component::Xp, 0, consts)
}

// xp circle between/between
fn path_xp_circle_between_between(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);
    state.follow_path(
        pxu::Component::P,
        &[[0.03, 0.03], [-0.03, 0.03], [-0.06, 0.0]],
        &contours,
        consts,
    );

    create_xp_circle_between_path(
        "xp circle between/between",
        state,
        -2.5,
        3.5,
        &contours,
        consts,
    )
}

// xp circle between/inside
fn path_xp_circle_between_inside_left(
    contour_provider: std::sync::Arc<ContourProvider>,
) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);
    state.follow_path(
        pxu::Component::P,
        &[[0.03, 0.03], [-0.03, 0.03], [-0.06, 0.0], [-0.06, -0.2]],
        &contours,
        consts,
    );

    create_xp_circle_between_path(
        "xp circle between/inside L",
        state,
        0.0,
        -2.5,
        &contours,
        consts,
    )
}

// xp circle between/inside
fn path_xp_circle_between_inside_right(
    contour_provider: std::sync::Arc<ContourProvider>,
) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);
    state.follow_path(
        pxu::Component::P,
        &[[0.03, 0.03], [-0.03, 0.03], [-0.06, 0.0], [-0.06, -0.2]],
        &contours,
        consts,
    );

    create_xp_circle_between_path(
        "xp circle between/inside R",
        state,
        0.0,
        3.5,
        &contours,
        consts,
    )
}

// xp circle between/outside
fn path_xp_circle_between_outside_left(
    contour_provider: std::sync::Arc<ContourProvider>,
) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);
    state.follow_path(
        pxu::Component::P,
        &[[0.2, 0.0], [0.2, 0.2], [0.78, 0.2]],
        &contours,
        consts,
    );

    create_xp_circle_between_path(
        "xp circle between/outside L",
        state,
        0.0,
        -2.5,
        &contours,
        consts,
    )
}

// xp circle between/outside
fn path_xp_circle_between_outside_right(
    contour_provider: std::sync::Arc<ContourProvider>,
) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);
    state.follow_path(
        pxu::Component::P,
        &[[0.2, 0.0], [0.2, 0.2], [0.78, 0.2]],
        &contours,
        consts,
    );

    create_xp_circle_between_path(
        "xp circle between/outside R",
        state,
        0.0,
        3.5,
        &contours,
        consts,
    )
}

// xp circle between/between single
fn path_xp_circle_between_between_single(
    contour_provider: std::sync::Arc<ContourProvider>,
) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);
    state.follow_path(
        pxu::Component::P,
        &[[0.03, 0.03], [-0.03, 0.03], [-0.06, 0.0]],
        &contours,
        consts,
    );

    create_xp_circle_between_path(
        "xp circle between/between (single)",
        state,
        0.0,
        1.0,
        &contours,
        consts,
    )
}

// p circle origin not through E cut
fn path_p_circle_origin_not_e(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let center = Complex64::new(0.0, 0.0);
    let radius = 0.05;
    let steps = 128;

    let mut state = pxu::State::new(1, consts);
    state.goto(pxu::Component::P, center + radius, &contours, consts, 4);

    let mut path = vec![];

    for i in 0..=(steps) {
        let theta = TAU * (i as f64 / steps as f64);
        let z = center + Complex64::from_polar(radius, theta);
        path.push(z);
    }

    pxu::path::SavedPath::new(
        "p circle origin not through E cut",
        path,
        state,
        pxu::Component::P,
        0,
        consts,
    )
}

// P circle around origin through E cuts
fn path_p_circle_origin_e(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let center = Complex64::new(0.0, 0.0);
    let radius = 0.10;
    let steps = 128;

    let mut state = pxu::State::new(1, consts);
    state.goto(pxu::Component::P, center + radius, &contours, consts, 4);

    let mut path = vec![];

    for i in 0..=(steps) {
        let theta = TAU * (i as f64 / steps as f64);
        let z = center + Complex64::from_polar(radius, theta);
        path.push(z);
    }

    pxu::path::SavedPath::new(
        "P circle around origin through E cuts",
        path,
        state,
        pxu::Component::P,
        0,
        consts,
    )
}

// U band between/outside
fn path_u_band_between_outside(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -1.55;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[0.0, 0.0], [0.0, y0]],
        &contours,
        consts,
    );

    for y in (-2..=0).rev() {
        let y = y as f64;
        let path = [
            [-x0, y0 + k * y],
            [-x0, y0 + k * (y - 0.5)],
            [x0, y0 + k * (y - 0.5)],
            [x0, y0 + k * (y - 1.0)],
        ];

        state.follow_path(pxu::Component::U, &path, &contours, consts);
    }

    let y0 = y0 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        &contours,
        consts,
        16,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        &contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        &contours,
        consts,
        16,
    );

    let mut path = vec![state.points[0].u];

    let steps = 16;
    let steps = (0..=steps)
        .map(|n| PI * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    for y in 0..=5 {
        let y = y as f64;

        let c = Complex64::new(x0, y0 + k * y + r1);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r1, -PI / 2.0 + *theta));
        }

        let c = Complex64::new(-x0, y0 + k * y + 2.0 * r1 + r2);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r2, -PI / 2.0 - *theta));
        }
    }

    path.push(Complex64::new(0.0, y0 + 6.0 * k));

    pxu::path::SavedPath::new(
        "U band between/outside",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// U band between/outside (single)
fn path_u_band_between_outside_single(
    contour_provider: std::sync::Arc<ContourProvider>,
) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -1.55;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[0.0, 0.0], [0.0, y0]],
        &contours,
        consts,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        &contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        &contours,
        consts,
        16,
    );

    let mut path = vec![state.points[0].u];

    let steps = 32;
    let steps = (0..=steps)
        .map(|n| PI * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    for y in 0..=0 {
        let y = y as f64;

        let c = Complex64::new(x0, y0 + k * y + r1);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r1, -PI / 2.0 + *theta));
        }

        let c = Complex64::new(-x0, y0 + k * y + 2.0 * r1 + r2);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r2, -PI / 2.0 - *theta));
        }
    }

    path.push(Complex64::new(0.0, y0 + 1.0 * k));

    pxu::path::SavedPath::new(
        "U band between/outside (single)",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// U band between/inside
fn path_u_band_between_inside(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -1.75;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[4.8, 0.0], [4.8, 1.0], [0.0, 1.0], [0.0, -2.5], [-x0, -2.5]],
        &contours,
        consts,
    );

    for y in (-2..=0).rev() {
        let y = y as f64;
        let path = [
            [-x0, y0 + k * y],
            [-x0, y0 + k * (y - 0.5)],
            [x0, y0 + k * (y - 0.5)],
            [x0, y0 + k * (y - 1.0)],
        ];

        state.follow_path(pxu::Component::U, &path, &contours, consts);
    }

    let y0 = y0 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        &contours,
        consts,
        16,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        &contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        &contours,
        consts,
        16,
    );

    let mut path = vec![state.points[0].u];

    let steps = 16;
    let steps = (0..=steps)
        .map(|n| PI * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    for y in 0..=5 {
        let y = y as f64;

        let c = Complex64::new(x0, y0 + k * y + r1);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r1, -PI / 2.0 + *theta));
        }

        let c = Complex64::new(-x0, y0 + k * y + 2.0 * r1 + r2);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r2, -PI / 2.0 - *theta));
        }
    }

    path.push(Complex64::new(0.0, y0 + 6.0 * k));

    pxu::path::SavedPath::new(
        "U band between/inside",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// U band between/inside (single)
fn path_u_band_between_inside_single(
    contour_provider: std::sync::Arc<ContourProvider>,
) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[4.8, 0.0], [4.8, 1.0], [0.0, 1.0], [0.0, -2.5], [-x0, -2.5]],
        &contours,
        consts,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        &contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        &contours,
        consts,
        16,
    );

    let mut path = vec![state.points[0].u];

    let steps = 32;
    let steps = (0..=steps)
        .map(|n| PI * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    for y in 0..=0 {
        let y = y as f64;

        let c = Complex64::new(x0, y0 + k * y + r1);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r1, -PI / 2.0 + *theta));
        }

        let c = Complex64::new(-x0, y0 + k * y + 2.0 * r1 + r2);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r2, -PI / 2.0 - *theta));
        }
    }

    path.push(Complex64::new(0.0, y0 + 1.0 * k));

    pxu::path::SavedPath::new(
        "U band between/inside (single)",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// U period between/between
fn path_u_periodic_between_between(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -0.75;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[4.8, 0.0], [4.8, 1.0], [0.0, 1.0], [0.0, y0], [-x0, y0]],
        &contours,
        consts,
    );

    for y in (-2..=0).rev() {
        let y = y as f64;
        let path = [
            [-x0, y0 + k * y],
            [-x0, y0 + k * (y - 0.5)],
            [x0, y0 + k * (y - 0.5)],
            [x0, y0 + k * (y - 1.0)],
        ];

        state.follow_path(pxu::Component::U, &path, &contours, consts);
    }

    let y0 = y0 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        &contours,
        consts,
        16,
    );

    let r1 = 0.75;
    let r2 = k / h - r1;
    let y0 = -r1 - 3.0 * k;
    let x0 = 1.8;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        &contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        &contours,
        consts,
        16,
    );

    let mut path = vec![state.points[0].u];

    let steps = 16;
    let steps = (0..=steps)
        .map(|n| PI * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    for y in 0..=5 {
        let y = y as f64;

        let c = Complex64::new(x0, y0 + k * y + r1);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r1, -PI / 2.0 + *theta));
        }

        let c = Complex64::new(-x0, y0 + k * y + 2.0 * r1 + r2);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r2, -PI / 2.0 - *theta));
        }
    }

    path.push(Complex64::new(0.0, y0 + 6.0 * k));

    pxu::path::SavedPath::new(
        "U period between/between",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// U period between/between single
fn path_u_periodic_between_between_single(
    contour_provider: std::sync::Arc<ContourProvider>,
) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -0.75;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[4.8, 0.0], [4.8, 1.0], [0.0, 1.0], [0.0, y0], [-x0, y0]],
        &contours,
        consts,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        &contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        &contours,
        consts,
        16,
    );

    let mut path = vec![state.points[0].u];

    let steps = 32;
    let steps = (0..=steps)
        .map(|n| PI * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    for y in 0..=0 {
        let y = y as f64;

        let c = Complex64::new(x0, y0 + k * y + r1);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r1, -PI / 2.0 + *theta));
        }

        let c = Complex64::new(-x0, y0 + k * y + 2.0 * r1 + r2);
        for theta in steps.iter() {
            path.push(c + Complex64::from_polar(r2, -PI / 2.0 - *theta));
        }
    }

    path.push(Complex64::new(0.0, y0 + 1.0 * k));

    pxu::path::SavedPath::new(
        "U period between/between (single)",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// U crossing from 0-2pi
fn path_u_crossing_from_0_a(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let h = consts.h;
    let x0 = -0.19224596334559135;
    let x1 = 2.6;
    let x2 = 0.08077185856988384;
    let y = 2.2 / h;
    let r = 1.0 / h;

    state.goto(
        pxu::Component::U,
        Complex64::new(x0, 0.0),
        &contours,
        consts,
        16,
    );

    let steps = 8;
    let steps = (0..=steps)
        .map(|n| PI / 2.0 * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    let mut path = vec![state.points[0].u];

    for theta in steps.iter() {
        path.push(Complex64::new(x0 + r, -y + r) + Complex64::from_polar(r, -PI + theta));
    }

    for theta in steps.iter() {
        path.push(Complex64::new(x1 - r, -y + r) + Complex64::from_polar(r, -PI / 2.0 + theta));
    }

    for theta in steps.iter() {
        path.push(Complex64::new(x1 - r, y - r) + Complex64::from_polar(r, *theta));
    }

    for theta in steps.iter() {
        path.push(Complex64::new(x2 + r, y - r) + Complex64::from_polar(r, PI / 2.0 + theta));
    }

    path.push(Complex64::new(x2, 0.0));

    pxu::path::SavedPath::new(
        "U crossing from 0-2pi path A",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// U crossing from 0-2pi
fn path_u_crossing_from_0_b(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let h = consts.h;
    let x0 = 1.9235122885022853;
    let x1 = 0.45;
    let x2 = 2.0535118654142286;
    let y = 1.8 / h;
    let r = 1.0 / h;

    state.goto(
        pxu::Component::U,
        Complex64::new(x0, 0.0),
        &contours,
        consts,
        16,
    );

    let steps = 8;
    let steps = (0..=steps)
        .map(|n| PI / 2.0 * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    let mut path = vec![state.points[0].u];

    for theta in steps.iter() {
        path.push(Complex64::new(x0 - r, -y + r) + Complex64::from_polar(r, -theta));
    }

    for theta in steps.iter() {
        path.push(Complex64::new(x1 + r, -y + r) + Complex64::from_polar(r, -PI / 2.0 - theta));
    }

    for theta in steps.iter() {
        path.push(Complex64::new(x1 + r, y - r) + Complex64::from_polar(r, PI - theta));
    }

    for theta in steps.iter() {
        path.push(Complex64::new(x2 - r, y - r) + Complex64::from_polar(r, PI / 2.0 - theta));
    }

    path.push(Complex64::new(x2, 0.0));

    pxu::path::SavedPath::new(
        "U crossing from 0-2pi path B",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// U crossing from -2pi to 0
fn path_u_crossing_from_min_1(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let k = consts.k() as f64;
    let h = consts.h;
    let x0 = -0.4319489724735624;
    let x1 = -2.4;
    let x2 = -0.21090292930603183;
    let y = 2.2 / h;
    let r = 1.0 / h;

    state.follow_path(
        pxu::Component::U,
        &[
            [3.0, 0.0],
            [3.0, -2.0 / h],
            [-3.0, -2.0 / h],
            [-3.0, k / h],
            [x0, k / h],
        ],
        &contours,
        consts,
    );

    let steps = 8;
    let steps = (0..=steps)
        .map(|n| PI / 2.0 * n as f64 / steps as f64)
        .collect::<Vec<_>>();

    let mut path = vec![state.points[0].u];

    for theta in steps.iter() {
        path.push(Complex64::new(x0 - r, k / h - y + r) + Complex64::from_polar(r, -theta));
    }

    for theta in steps.iter() {
        path.push(
            Complex64::new(x1 + r, k / h - y + r) + Complex64::from_polar(r, -PI / 2.0 - theta),
        );
    }

    for theta in steps.iter() {
        path.push(Complex64::new(x1 + r, k / h + y - r) + Complex64::from_polar(r, PI - theta));
    }

    for theta in steps.iter() {
        path.push(
            Complex64::new(x2 - r, k / h + y - r) + Complex64::from_polar(r, PI / 2.0 - theta),
        );
    }

    path.push(Complex64::new(x2, k / h));

    pxu::path::SavedPath::new(
        "U crossing from -2pi to 0",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

// p crossing a
fn path_p_crossing_a(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let p0 = 0.15;
    let y0 = 0.08;
    let steps = 100;

    let mut state = pxu::State::new(1, consts);
    state.goto(pxu::Component::P, p0, &contours, consts, 4);

    let mut path = vec![];
    for i in 0..=steps {
        let x = 1.0 - (i as f64 / steps as f64) * 2.0;
        let y = y0 * (1.0 - x * x);
        let p = Complex64::new(x * p0, y);

        path.push(p);
    }

    pxu::path::SavedPath::new("p crossing a", path, state, pxu::Component::P, 0, consts)
}

// p crossing b
fn path_p_crossing_b(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let p0 = 0.15;
    let y0 = 0.08;
    let steps = 100;

    let mut state = pxu::State::new(1, consts);
    state.goto(pxu::Component::P, p0, &contours, consts, 4);

    let mut path = vec![];
    for i in 0..=steps {
        let x = 1.0 - (i as f64 / steps as f64) * 2.0;
        let y = -y0 * (1.0 - x * x);
        let p = Complex64::new(x * p0, y);

        path.push(p);
    }

    pxu::path::SavedPath::new("p crossing b", path, state, pxu::Component::P, 0, consts)
}

// p crossing c
fn path_p_crossing_c(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let p0 = 0.15;
    let bp = Complex64::new(0.915, 0.370);

    let mut state = pxu::State::new(1, consts);
    state.goto(pxu::Component::P, p0, &contours, consts, 4);

    let dp = (bp / bp.norm()) * p0 * (bp.re / bp.norm());

    let mut path = vec![Complex64::from(p0), bp + p0 - dp];

    let steps = 32;

    for i in 1..(steps - 1) {
        let theta = PI * i as f64 / steps as f64;
        path.push(bp + (p0 - dp) * (Complex64::i() * theta).exp());
    }

    path.extend([bp - p0 + dp, Complex64::from(-p0)]);

    pxu::path::SavedPath::new("p crossing c", path, state, pxu::Component::P, 0, consts)
}

// p crossing d
fn path_p_crossing_d(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let p0 = 0.15;
    let bp = Complex64::new(-0.922, -0.265);

    let mut state = pxu::State::new(1, consts);
    state.goto(pxu::Component::P, p0, &contours, consts, 4);

    let dp = (bp / bp.norm()) * p0 * (bp.re / bp.norm());

    let mut path = vec![Complex64::from(p0), bp + p0 - dp];

    let steps = 32;

    for i in 1..(steps - 1) {
        let theta = -PI * i as f64 / steps as f64;
        path.push(bp + (p0 - dp) * (Complex64::i() * theta).exp());
    }

    path.extend([bp - p0 + dp, Complex64::from(-p0)]);

    pxu::path::SavedPath::new("p crossing d", path, state, pxu::Component::P, 0, consts)
}

fn path_u_vertical_outside(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let steps = 67;
    let y0 = -0.51;
    let y1 = -8.0;

    state.follow_path(
        pxu::Component::U,
        &[[3.0, 0.0], [3.0, -2.0], [0.0, -2.0], [0.0, y0]],
        &contours,
        consts,
    );

    let p1 = Complex64::new(0.0, y0);
    let p2 = Complex64::new(0.0, y1);

    let path = (0..=steps)
        .map(|i| p1 + (i as f64 / steps as f64) * (p2 - p1))
        .collect::<Vec<_>>();

    pxu::path::SavedPath::new(
        "u vertical outside",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

fn path_u_vertical_between(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let steps = 67;
    let y0 = -0.49;
    let y1 = 2.0;

    state.follow_path(
        pxu::Component::U,
        &[[3.0, 0.0], [3.0, -2.0], [0.0, -2.0], [0.0, y0]],
        &contours,
        consts,
    );

    let p1 = Complex64::new(0.0, y0);
    let p2 = Complex64::new(0.0, y1);

    let path = (0..=steps)
        .map(|i| p1 + (i as f64 / steps as f64) * (p2 - p1))
        .collect::<Vec<_>>();

    pxu::path::SavedPath::new(
        "u vertical between",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

fn path_u_vertical_inside(contour_provider: std::sync::Arc<ContourProvider>) -> SavedPath {
    let consts = CouplingConstants::new(2.0, 5);
    let contours = contour_provider.get(consts).unwrap();

    let mut state = pxu::State::new(1, consts);

    let steps = 67;
    let y0 = 2.0;
    let y1 = 50.0;

    state.follow_path(
        pxu::Component::U,
        &[
            [3.0, 0.0],
            [3.0, -2.0],
            [0.0, -2.0],
            [0.0, -0.49],
            [0.0, 0.0],
            [0.0, y0],
        ],
        &contours,
        consts,
    );

    let p1 = Complex64::new(0.0, y0);
    let p2 = Complex64::new(0.0, y1);

    let path = (0..=steps)
        .map(|i| p1 + (i as f64 / steps as f64) * (p2 - p1))
        .collect::<Vec<_>>();

    pxu::path::SavedPath::new(
        "u vertical inside",
        path,
        state,
        pxu::Component::U,
        0,
        consts,
    )
}

pub const PLOT_PATHS: &[crate::PathFunction] = &[
    path_xp_circle_between_between,
    path_xp_circle_between_between_single,
    path_xp_circle_between_inside_left,
    path_xp_circle_between_inside_right,
    path_xp_circle_between_outside_left,
    path_xp_circle_between_outside_right,
    path_p_circle_origin_e,
    path_p_circle_origin_not_e,
    path_u_band_between_inside,
    path_u_band_between_inside_single,
    path_u_band_between_outside,
    path_u_band_between_outside_single,
    path_u_periodic_between_between,
    path_u_periodic_between_between_single,
    path_u_crossing_from_0_a,
    path_u_crossing_from_0_b,
    path_u_crossing_from_min_1,
    path_p_crossing_a,
    path_p_crossing_b,
    path_p_crossing_c,
    path_p_crossing_d,
    path_u_vertical_outside,
    path_u_vertical_between,
    path_u_vertical_inside,
];

pub const INTERACTIVE_PATHS: &[crate::PathFunction] = &[
    path_xp_circle_between_between,
    path_xp_circle_between_between_single,
    path_xp_circle_between_inside_left,
    path_xp_circle_between_inside_right,
    path_xp_circle_between_outside_left,
    path_xp_circle_between_outside_right,
    path_p_circle_origin_e,
    path_p_circle_origin_not_e,
    path_u_band_between_inside,
    path_u_band_between_outside,
    path_u_periodic_between_between,
    path_u_crossing_from_0_b,
    path_u_crossing_from_0_a,
    path_u_crossing_from_min_1,
    path_p_crossing_a,
    path_p_crossing_b,
    path_p_crossing_c,
    path_p_crossing_d,
    path_u_vertical_between,
];
