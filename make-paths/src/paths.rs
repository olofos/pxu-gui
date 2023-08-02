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

// xp circle between/between
fn path_xp_circle_between_between(
    contours: &pxu::Contours,
    consts: CouplingConstants,
) -> SavedPath {
    let mut state = pxu::State::new(1, consts);
    state.follow_path(
        pxu::Component::P,
        &[[0.03, 0.03], [-0.03, 0.03], [-0.06, 0.0]],
        contours,
        consts,
    );

    let center = Complex64::new(-0.3, 0.5);
    let radius = 1.2;
    let steps = 128;

    let mut path = vec![];

    state.update(0, pxu::Component::Xp, center - radius, contours, consts);
    for i in 0..=(4 * steps) {
        let theta = TAU * (i as f64 / steps as f64 - 0.5);
        let xp = center + Complex64::from_polar(radius, theta);
        path.push(xp);
    }

    pxu::path::SavedPath::new(
        "xp circle between/between",
        path,
        state,
        pxu::Component::Xp,
        0,
        consts,
    )
}

// p circle origin not through E cut
fn path_p_circle_origin_not_e(contours: &pxu::Contours, consts: CouplingConstants) -> SavedPath {
    let center = Complex64::new(0.0, 0.0);
    let radius = 0.05;
    let steps = 128;

    let mut state = pxu::State::new(1, consts);
    state.goto(pxu::Component::P, center + radius, contours, consts, 4);

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
fn path_p_circle_origin_e(contours: &pxu::Contours, consts: CouplingConstants) -> SavedPath {
    let center = Complex64::new(0.0, 0.0);
    let radius = 0.10;
    let steps = 128;

    let mut state = pxu::State::new(1, consts);
    state.goto(pxu::Component::P, center + radius, contours, consts, 4);

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
fn path_u_band_between_outside(contours: &pxu::Contours, consts: CouplingConstants) -> SavedPath {
    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -1.5;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[0.0, 0.0], [0.0, y0]],
        contours,
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

        state.follow_path(pxu::Component::U, &path, contours, consts);
    }

    let y0 = y0 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        contours,
        consts,
        16,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        contours,
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
    contours: &pxu::Contours,
    consts: CouplingConstants,
) -> SavedPath {
    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -1.5;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[0.0, 0.0], [0.0, y0]],
        contours,
        consts,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        contours,
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
fn path_u_band_between_inside(contours: &pxu::Contours, consts: CouplingConstants) -> SavedPath {
    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -1.75;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[4.8, 0.0], [4.8, 1.0], [0.0, 1.0], [0.0, -2.5], [-x0, -2.5]],
        contours,
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

        state.follow_path(pxu::Component::U, &path, contours, consts);
    }

    let y0 = y0 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        contours,
        consts,
        16,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        contours,
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
    contours: &pxu::Contours,
    consts: CouplingConstants,
) -> SavedPath {
    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[4.8, 0.0], [4.8, 1.0], [0.0, 1.0], [0.0, -2.5], [-x0, -2.5]],
        contours,
        consts,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        contours,
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
fn path_u_periodic_between_between(
    contours: &pxu::Contours,
    consts: CouplingConstants,
) -> SavedPath {
    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -0.75;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[4.8, 0.0], [4.8, 1.0], [0.0, 1.0], [0.0, y0], [-x0, y0]],
        contours,
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

        state.follow_path(pxu::Component::U, &path, contours, consts);
    }

    let y0 = y0 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        contours,
        consts,
        16,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1 - 3.0 * k;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        contours,
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
    contours: &pxu::Contours,
    consts: CouplingConstants,
) -> SavedPath {
    let mut state = pxu::State::new(1, consts);

    let x0 = 2.7;
    let y0 = -0.75;
    let k = consts.k() as f64;
    let h = consts.h;

    state.follow_path(
        pxu::Component::U,
        &[[4.8, 0.0], [4.8, 1.0], [0.0, 1.0], [0.0, y0], [-x0, y0]],
        contours,
        consts,
    );

    let r1 = 1.0;
    let r2 = k / h - r1;
    let y0 = -r1;

    state.goto(
        pxu::Component::U,
        Complex64::new(-x0, y0),
        contours,
        consts,
        16,
    );
    state.goto(
        pxu::Component::U,
        Complex64::new(0.0, y0),
        contours,
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
fn path_u_crossing_from_0_a(contours: &pxu::Contours, consts: CouplingConstants) -> SavedPath {
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
        contours,
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
fn path_u_crossing_from_0_b(contours: &pxu::Contours, consts: CouplingConstants) -> SavedPath {
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
        contours,
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
fn path_u_crossing_from_min_1(contours: &pxu::Contours, consts: CouplingConstants) -> SavedPath {
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
        contours,
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

type PathFunction = fn(&pxu::Contours, CouplingConstants) -> SavedPath;

pub const PLOT_PATHS: &[PathFunction] = &[
    path_xp_circle_between_between,
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
];

pub const INTERACTIVE_PATHS: &[PathFunction] = &[
    path_xp_circle_between_between,
    path_p_circle_origin_e,
    path_p_circle_origin_not_e,
    path_u_band_between_inside,
    path_u_band_between_outside,
    path_u_periodic_between_between,
    path_u_crossing_from_0_b,
    path_u_crossing_from_0_a,
    path_u_crossing_from_min_1,
];
