use std::collections::HashMap;

use crate::kinematics::{
    den2_dp, du_crossed_dp, du_dp, dxm_crossed_dp, dxm_dp, dxp_crossed_dp, dxp_dp, en2, u,
    u_crossed, xm, xm_crossed, xp, xp_crossed, CouplingConstants, SheetData,
};
use crate::nr::{self};
use crate::pxu2::{InterpolationPoint, PInterpolator, XInterpolator};
use itertools::Itertools;
use num::complex::Complex;
use num::{Integer, Zero};

type C = Complex<f64>;

struct XpFixedM {
    m: f64,
    consts: CouplingConstants,
}

impl XpFixedM {
    fn new(m: f64, consts: CouplingConstants) -> Self {
        Self { m, consts }
    }
}

impl nr::OneParameterFunction for XpFixedM {
    fn evaluate(&self, p: f64) -> C {
        xp(p, self.m, self.consts)
    }
}

struct XmFixedM {
    m: f64,
    consts: CouplingConstants,
}

impl XmFixedM {
    fn new(m: f64, consts: CouplingConstants) -> Self {
        Self { m, consts }
    }
}

impl nr::OneParameterFunction for XmFixedM {
    fn evaluate(&self, p: f64) -> C {
        xm(p, self.m, self.consts)
    }
}

struct XpFixedP {
    p: f64,
    consts: CouplingConstants,
}

impl XpFixedP {
    fn new(p: f64, consts: CouplingConstants) -> Self {
        Self { p, consts }
    }
}

impl nr::OneParameterFunction for XpFixedP {
    fn evaluate(&self, m: f64) -> C {
        xp(C::from(self.p), m, self.consts)
    }
}

struct XmFixedP {
    p: f64,
    consts: CouplingConstants,
}

impl XmFixedP {
    fn new(p: f64, consts: CouplingConstants) -> Self {
        Self { p, consts }
    }
}

impl nr::OneParameterFunction for XmFixedP {
    fn evaluate(&self, m: f64) -> C {
        xm(C::from(self.p), m, self.consts)
    }
}

struct FixedRe {
    re: f64,
}

impl FixedRe {
    fn new(re: f64) -> Self {
        Self { re }
    }
}

impl nr::OneParameterFunction for FixedRe {
    fn evaluate(&self, im: f64) -> C {
        C::new(self.re, im)
    }
}

struct FixedIm {
    im: f64,
}

impl FixedIm {
    fn new(im: f64) -> Self {
        Self { im }
    }
}

impl nr::OneParameterFunction for FixedIm {
    fn evaluate(&self, re: f64) -> C {
        C::new(re, self.im)
    }
}

// struct Lerp {
//     z0: C,
//     z1: C,
// }

// impl Lerp {
//     fn new(z0: C, z1: C) -> Self {
//         Self { z0, z1 }
//     }
// }

// impl nr::OneParameterFunction for Lerp {
//     fn evaluate(&self, t: f64) -> C {
//         self.z0 + (self.z1 - self.z0) * t
//     }
// }

struct XpFunc {
    consts: CouplingConstants,
}

impl nr::Func for XpFunc {
    fn f(&self, z: C) -> C {
        xp(z, 1.0, self.consts)
    }
    fn df(&self, z: C) -> C {
        dxp_dp(z, 1.0, self.consts)
    }
}

struct XmFunc {
    consts: CouplingConstants,
}

impl nr::Func for XmFunc {
    fn f(&self, z: C) -> C {
        xm(z, 1.0, self.consts)
    }
    fn df(&self, z: C) -> C {
        dxm_dp(z, 1.0, self.consts)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Component {
    P,
    Xp,
    Xm,
    U,
}

#[derive(Debug)]
pub struct Grid {
    data: HashMap<i32, GridLines>,
    consts: Option<CouplingConstants>,
}

impl Grid {
    pub fn new() -> Self {
        let data = HashMap::new();
        let consts = None;
        Self { data, consts }
    }

    pub fn get(&mut self, pt: &PxuPoint, component: Component) -> &Vec<Vec<C>> {
        if let Some(consts) = self.consts {
            if consts != pt.consts {
                log::info!("Clearing grid");
                self.data.clear();
            }
        }
        self.consts = Some(pt.consts);

        let line: &GridLines = self
            .data
            .entry(pt.sheet_data.log_branch)
            .or_insert_with(|| GridLines::new(pt.sheet_data.log_branch, pt.consts));
        line.get(component)
    }
}

#[derive(Debug)]
struct GridLines {
    p: Vec<Vec<C>>,
    x: Vec<Vec<C>>,
    u: Vec<Vec<C>>,
}

impl GridLines {
    fn new(p_range: i32, consts: CouplingConstants) -> Self {
        log::info!("Generating grid lines for branch {p_range}");
        let mut x = vec![];
        for i in -4..=4 {
            x.extend(Self::fill_x(p_range + i, consts));
        }
        let mut u = vec![];
        u.extend(Self::fill_u(p_range, consts));

        let mut p = vec![];
        for i in -1..=1 {
            p.extend(Self::fill_p(p_range + i, consts));
        }
        Self { p, x, u }
    }

    fn get(&self, component: Component) -> &Vec<Vec<C>> {
        match component {
            Component::P => &self.p,
            Component::Xp | Component::Xm => &self.x,
            Component::U => &self.u,
        }
    }

    fn fill_x(p_range: i32, consts: CouplingConstants) -> Vec<Vec<C>> {
        let mut lines = vec![];
        for m in (p_range * consts.k() as i32)..=((p_range + 1) * consts.k() as i32) {
            let xp_points = XInterpolator::generate_xp_full(0, m as f64, consts);

            lines.push(xp_points.iter().map(|z| z.conj()).collect::<Vec<_>>());
            lines.push(xp_points);
        }

        if p_range == 0 {
            lines.push(vec![C::from(consts.s()), C::from(1000.0)]);
        }

        if p_range == -1 {
            lines.push(vec![C::from(-1000.0), C::from(-1.0 / consts.s())]);
        }

        lines
    }

    fn fill_u(_p_range: i32, consts: CouplingConstants) -> Vec<Vec<C>> {
        let mut lines = vec![];

        let k = consts.k() as i32;
        for y in (-k)..=(k) {
            let y = y as f64 / consts.h;
            lines.push(vec![C::new(-1000.0, y), C::new(1000.0, y)]);
        }

        lines
    }

    fn fill_p(p_range: i32, consts: CouplingConstants) -> Vec<Vec<C>> {
        let mut lines = vec![];

        let p_start = p_range as f64;
        lines.push(vec![C::from(p_start), C::from(p_start + 1.0)]);

        let xp_func = XpFunc { consts };

        let p_s = {
            let p0 = 1.0 / 8.0;
            let x0 = xp(C::from(p0), 1.0, consts);
            let fixed_re = FixedRe::new(x0.re);
            let start = nr::shoot(&xp_func, &fixed_re, x0.im, 0.0, C::from(p0), x0.im / 1.0)
                .last()
                .unwrap()
                .1;
            let fixed_im = FixedIm::new(0.0);
            let t0 = x0.re;
            let t1 = consts.s();
            nr::shoot(&xp_func, &fixed_im, t0, t1, start, (t1 - t0).abs() / 4.0)
                .last()
                .unwrap()
                .1
        };

        let p_min_one_over_s = {
            let p0 = -1.0 / 64.0;
            let x0 = xp(C::from(p0), 1.0, consts);
            let fixed_re = FixedRe::new(x0.re);
            let pts = nr::shoot(&xp_func, &fixed_re, x0.im, 0.0, C::from(p0), x0.im / 1.0);

            let start = pts.last().unwrap().1;
            let fixed_im = FixedIm::new(0.0);
            let t0 = x0.re;
            let t1 = -1.0 / consts.s();
            let pts = nr::shoot(&xp_func, &fixed_im, t0, t1, start, (t1 - t0).abs() / 4.0);

            pts.last().unwrap().1
        };

        {
            // xp m = 0
            let p0 = p_start + 1.0 / 8.0;
            let xp_fixed_p = XpFixedP::new(p0, consts);

            let (m, p1) = *nr::shoot(&xp_func, &xp_fixed_p, 1.0, 0.0, C::from(p0), 0.5)
                .last()
                .unwrap();

            let fixed_m = XpFixedM::new(m, consts);

            let mut pts = vec![];
            if p_range != 0 {
                pts.push(C::from(p_start));
            } else {
                pts.push(p_s);
            }
            pts.extend(
                nr::shoot_two_sided(
                    &xp_func,
                    &fixed_m,
                    p_start + 1.0 * 1.0 / 64.0,
                    p0,
                    p_start + 63.0 * 1.0 / 64.0,
                    p1,
                    1.0 / 32.0,
                )
                .into_iter()
                .map(|(_, p)| p),
            );
            if p_range != -1 {
                pts.push(C::from(p_start + 1.0));
            } else {
                pts.push(p_min_one_over_s);
            }

            lines.push(pts.iter().map(|z| z.conj()).collect::<Vec<_>>());
            lines.push(pts);
        }

        if p_range == 0 {
            let fixed_im = FixedIm::new(0.0);
            {
                let t0 = consts.s();
                let t1 = 32.0 * consts.s();
                let mut pts = nr::shoot(&xp_func, &fixed_im, t0, t1, p_s, 1.0 / 4.0)
                    .into_iter()
                    .map(|(_, p)| p)
                    .collect::<Vec<_>>();
                pts.push(C::zero());

                lines.push(pts.iter().map(|z| z.conj()).collect::<Vec<_>>());
                lines.push(pts);
            }
        }

        if p_range == -1 {
            let fixed_im = FixedIm::new(0.0);
            {
                let t0 = -1.0 / consts.s();
                let t1 = -32.0 * consts.s();
                let mut pts = nr::shoot(&xp_func, &fixed_im, t0, t1, p_min_one_over_s, 1.0 / 8.0)
                    .into_iter()
                    .map(|(_, p)| p)
                    .collect::<Vec<_>>();
                pts.push(C::zero());
                lines.push(pts.iter().map(|z| z.conj()).collect::<Vec<_>>());
                lines.push(pts);
            }
        }

        if p_range != -1 {
            let p0 = C::from(p_start + 1.0 / 4.0);
            let x0 = xp(p0, 1.0, consts);
            let fixed_re = FixedRe::new(x0.re);
            let pts = nr::shoot(&xp_func, &fixed_re, x0.im, -x0.im, p0, x0.im / 8.0);
            let p1 = pts.last().unwrap().1;
            let xm_fixed_p = XmFixedP::new(p0.re, consts);

            let starts = nr::shoot_two_sided(
                &xp_func,
                &xm_fixed_p,
                0.0,
                1.0,
                1.0f64.max(consts.k() as f64 - 2.0),
                p1,
                1.0,
            );
            for (m, p) in starts.into_iter() {
                let xm_fixed_m = XmFixedM::new(m, consts);
                let mut pts = vec![];
                if m == 0.0 && p_range == 0 {
                    pts.push(p_s);
                } else {
                    pts.push(C::from(p_start));
                }
                pts.extend(
                    nr::shoot_two_sided(
                        &xp_func,
                        &xm_fixed_m,
                        p_start + 1.0 / 128.0,
                        p0.re,
                        p_start + 1.0 - 1.0 / 128.0,
                        p,
                        1.0 / 128.0,
                    )
                    .into_iter()
                    .map(|(_, p)| p),
                );
                pts.push(C::from(p_start));

                lines.push(pts.iter().map(|z| z.conj()).collect::<Vec<_>>());
                lines.push(pts);
            }
        }

        if p_range == -1 {
            let p0 = C::from(-1.0 / 64.0);
            let x0 = xp(p0, 1.0, consts);
            let fixed_re = FixedRe::new(x0.re);
            let pts = nr::shoot(&xp_func, &fixed_re, x0.im, -x0.im, p0, x0.im / 8.0);
            let p1 = pts.last().unwrap().1;

            let xm_fixed_p = XmFixedP::new(p0.re, consts);

            // let starts = nr::shoot(&xp_func, &xm_fixed_p, 1.0, 0.0, p1, 1.0);
            let starts = nr::shoot(&xp_func, &xm_fixed_p, 1.0, consts.k() as f64 - 1.0, p1, 1.0);
            let starts = [*starts.last().unwrap()];
            for (m, p) in starts.into_iter() {
                let xm_fixed_m = XmFixedM::new(m, consts);
                let mut pts = vec![];
                if m == 0.0 && p_range == 0 {
                    // pts.push(p_s);
                } else {
                    // pts.push(C::from(p_start));
                }
                pts.extend(
                    nr::shoot_two_sided(
                        &xp_func,
                        &xm_fixed_m,
                        p_start + 1.0 / 32.0,
                        p0.re,
                        p_start + 1.0 - 1.0 / 128.0,
                        p,
                        1.0 / 128.0,
                    )
                    .into_iter()
                    .map(|(_, p)| p),
                );

                // pts.push(C::from(p_start));

                lines.push(pts.iter().map(|z| z.conj()).collect::<Vec<_>>());
                lines.push(pts);
            }
        }

        {
            let p0 = nr::find_root(
                |p| en2(p, 1.0, consts),
                |p| den2_dp(p, 1.0, consts),
                C::new(p_start, 2.5),
                // C::new(0.0, 0.5),
                1.0e-3,
                50,
            );

            let mut cut_p = vec![p0.unwrap()];
            for i in 1..512 {
                let im = i as f64 * i as f64 / 64.0;

                let p = nr::find_root(
                    |p| en2(p, 1.0, consts) - C::new(-im, 0.001),
                    |p| den2_dp(p, 1.0, consts),
                    *cut_p.last().unwrap(),
                    1.0e-3,
                    50,
                );

                cut_p.push(p.unwrap());
            }

            cut_p.reverse();
            for i in 1..64 {
                let im = i as f64 * i as f64 / 64.0;

                let p = nr::find_root(
                    |p| en2(p, 1.0, consts) - C::new(-im, -0.001),
                    |p| den2_dp(p, 1.0, consts),
                    *cut_p.last().unwrap(),
                    1.0e-3,
                    50,
                );

                cut_p.push(p.unwrap());
            }

            let cut_p = cut_p.into_iter().map(|p| p).collect::<Vec<_>>();

            lines.push(cut_p);
        }

        lines
    }
}

fn get_branch_point(m: f64, consts: CouplingConstants, branch: f64) -> f64 {
    {
        let s = consts.s();
        let u_of_x = |x: C| -> C { x + 1.0 / x - (s - 1.0 / s) * x.ln() };
        let du_dx = |x: C| -> C { (x - s) * (x + 1.0 / s) / (x * x) };
        let sign = if branch == 0.0 { 1.0 } else { -1.0 };
        let guess = if m > 0.0 {
            C::new(0.0, 1.0)
        } else {
            C::new(0.0, 0.01)
        };
        let x = nr::find_root(
            |x| {
                u_of_x(x) - sign * u_of_x(C::from(consts.s()))
                    + branch * C::i() * consts.k() as f64 / consts.h
                    - 2.0 * m * C::i() / consts.h
            },
            du_dx,
            guess,
            1.0e-3,
            10,
        );
        let x = x.unwrap();
        x.arg() / std::f64::consts::PI
    }
}

fn get_branch_point_x(m: f64, consts: CouplingConstants, branch: f64) -> C {
    {
        let s = consts.s();
        let u_of_x = |x: C| -> C { x + 1.0 / x - (s - 1.0 / s) * x.ln() };
        let du_dx = |x: C| -> C { (x - s) * (x + 1.0 / s) / (x * x) };
        let sign = if branch == 0.0 { 1.0 } else { -1.0 };
        let guess = if m > 0.0 {
            C::new(0.0, 1.0)
        } else {
            C::new(0.0, 0.01)
        };
        let x = nr::find_root(
            |x| {
                u_of_x(x) - sign * u_of_x(C::from(consts.s()))
                    + branch * C::i() * consts.k() as f64 / consts.h
                    - 2.0 * m * C::i() / consts.h
            },
            du_dx,
            guess,
            1.0e-3,
            10,
        );
        x.unwrap()
    }
}

#[derive(Debug)]
pub enum CutType {
    U(Component),
    LogX(Component, i32),
    E,
    DebugPath,
}

#[derive(Debug)]
enum CutVisibilityCondition {
    ImXp(i8),
    ImXm(i8),
    LogBranch(i32),
    LogBranchGT(i32),
    LogBranchLE(i32),
}

impl CutVisibilityCondition {
    fn check(&self, pt: &PxuPoint) -> bool {
        match self {
            Self::ImXp(sign) => pt.xp.im.signum() as i8 == sign.signum(),
            Self::ImXm(sign) => pt.xm.im.signum() as i8 == sign.signum(),
            Self::LogBranch(b) => *b == pt.sheet_data.log_branch,
            Self::LogBranchGT(b) => pt.sheet_data.log_branch > *b,
            Self::LogBranchLE(b) => pt.sheet_data.log_branch <= *b,
        }
    }
}

#[derive(Debug)]
pub struct Cuts {
    cuts: Vec<Cut>,
    consts: Option<CouplingConstants>,
}

impl Cuts {
    pub fn new() -> Self {
        // let data = HashMap::new();
        let cuts = vec![];
        let consts = None;
        Self { cuts, consts }
    }

    fn populate(&mut self, pt: &PxuPoint) {
        if let Some(consts) = self.consts {
            if consts != pt.consts {
                log::info!("Clearing grid");
                // self.data.clear();
                self.cuts.clear();
            }
        }
        self.consts = Some(pt.consts);

        if self.cuts.is_empty() {
            // self.cuts = (-5..=5).map(|n| Cut::get(n, pt.consts)).collect();
            for p_range in -1..=1 {
                self.cuts.extend(Cut::get(p_range, pt.consts));
            }
            log::info!("Created {} cuts", self.cuts.len());
        }
    }

    pub fn visible(&mut self, pt: &PxuPoint, component: Component) -> impl Iterator<Item = &Cut> {
        self.populate(pt);

        let pt = pt.clone();

        self.cuts
            .iter()
            .filter(move |c| c.component == component && c.is_visible(&pt))
    }
    pub fn crossed(
        &mut self,
        pt: &PxuPoint,
        component: Component,
        new_value: C,
    ) -> impl Iterator<Item = &Cut> {
        self.populate(pt);

        let pt = pt.clone();

        self.cuts.iter().filter(move |c| {
            c.component == component
                && c.is_visible(&pt)
                && c.intersection(pt.get(component), new_value).is_some()
        })
    }
}

#[derive(Debug)]
pub struct Cut {
    pub component: Component,
    pub paths: Vec<Vec<C>>,
    pub branch_points: Vec<C>,
    pub typ: CutType,
    visibility: Vec<CutVisibilityCondition>,
}

impl Cut {
    fn new(component: Component, paths: Vec<Vec<C>>, branch_points: Vec<C>, typ: CutType) -> Self {
        Self {
            component,
            paths,
            branch_points,
            typ,
            visibility: vec![],
        }
    }

    fn im_xp_positive(mut self) -> Self {
        self.visibility.push(CutVisibilityCondition::ImXp(1));
        self
    }

    fn im_xp_negative(mut self) -> Self {
        self.visibility.push(CutVisibilityCondition::ImXp(-1));
        self
    }

    fn im_xm_positive(mut self) -> Self {
        self.visibility.push(CutVisibilityCondition::ImXm(1));
        self
    }

    fn im_xm_negative(mut self) -> Self {
        self.visibility.push(CutVisibilityCondition::ImXm(-1));
        self
    }

    fn log_branch(mut self, branch: i32) -> Self {
        self.visibility
            .push(CutVisibilityCondition::LogBranch(branch));
        self
    }

    fn log_branch_gt(mut self, branch: i32) -> Self {
        self.visibility
            .push(CutVisibilityCondition::LogBranchGT(branch));
        self
    }

    fn log_branch_le(mut self, branch: i32) -> Self {
        self.visibility
            .push(CutVisibilityCondition::LogBranchLE(branch));
        self
    }

    pub fn get(p_range: i32, consts: CouplingConstants) -> Vec<Cut> {
        let mut cuts = vec![];
        cuts.extend(Self::x_cuts_x(p_range, consts));
        cuts.extend(Self::x_cuts_p(p_range, consts));
        cuts.extend(Self::p_cuts_x(p_range, consts));
        cuts.extend(Self::p_cuts_p(p_range, consts));
        cuts.extend(Self::e_cuts(p_range, consts));

        cuts
    }

    pub fn intersection(&self, p1: C, p2: C) -> Option<C> {
        fn cross(v: C, w: C) -> f64 {
            v.re * w.im - v.im * w.re
        }

        let p = p1;
        let r = p2 - p1;

        for path in self.paths.iter() {
            for (q1, q2) in path.iter().tuple_windows::<(_, _)>() {
                let q = q1;
                let s = q2 - q1;

                if cross(r, s) != 0.0 {
                    let t = cross(q - p, s) / cross(r, s);
                    let u = cross(q - p, r) / cross(r, s);

                    if 0.0 <= t && t <= 1.0 && 0.0 <= u && u <= 1.0 {
                        return Some(p + t * r);
                    }
                }
            }
        }
        None
    }

    pub fn is_visible(&self, pt: &PxuPoint) -> bool {
        self.visibility.iter().all(|cond| cond.check(pt))
    }

    fn x_cuts_p(p_range: i32, consts: CouplingConstants) -> Vec<Cut> {
        let p_start = p_range as f64;

        let p_s = {
            let p0 = 1.0 / 8.0;
            let p_int = PInterpolator::xp(p0, consts)
                .goto_im(0.0)
                .goto_re(consts.s())
                .clear_path();
            *p_int.p_path.last().unwrap()
        };

        let p_min_one_over_s = {
            let p0 = -1.0 / 64.0;
            let p_int = PInterpolator::xp(p0, consts)
                .goto_im(0.0)
                .goto_re(-1.0 / consts.s())
                .clear_path();
            *p_int.p_path.last().unwrap()
        };

        let mut cuts = vec![];

        let mut p_points = vec![];
        p_points.push(C::from(p_start));
        if p_range != -1 {
            let p0 = p_start + 0.25;
            let p_int = PInterpolator::xp(p0, consts)
                .goto_xm(p0, 1.0)
                .goto_xm(p0, 0.0)
                .clear_path();

            let p_int2 = p_int.clone().goto_xm(p_start + 127.0 / 128.0, 0.0);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xm(p_start + 1.0 / 128.0, 0.0);
            p_points.extend(p_int2.p_path);

            if p_range == 0 {
                p_points.push(p_s);
            } else {
                p_points.push(C::from(p_start));
            }
        }

        {
            let p0 = p_start + 1.0 / 8.0;
            let p_int = PInterpolator::xp(p0, consts).goto_xp(p0, 0.0);
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_start + 1.0 / 64.0, 0.0);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.goto_xp(p_start + 1.0 - 1.0 / 64.0, 0.0);
            p_points.extend(p_int2.p_path);

            if p_range != -1 {
                p_points.push(C::from(p_start + 1.0));
            } else {
                p_points.push(p_min_one_over_s);
            }
        }

        cuts.push(Cut::new(
            Component::P,
            vec![p_points.clone()],
            vec![],
            CutType::U(Component::Xp),
        ));

        cuts.push(Cut::new(
            Component::P,
            vec![p_points.iter().map(|x| x.conj()).collect()],
            vec![],
            CutType::U(Component::Xm),
        ));

        if p_range == 0 {
            let mut p_points: Vec<C> = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 7.0 / 8.0;

            let p_int = PInterpolator::xp(p0, consts)
                .goto_xm(p0, 1.0)
                .goto_xm(p2, 1.0)
                .goto_xm(p2, -2.0);

            // x_cuts.push(p_int.x_path.clone());

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xm(p_start + 0.01, -2.0);
            // x_cuts.push(p_int2.x_path.clone());
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xm(p_start + 0.99, -2.0);
            // x_cuts.push(p_int2.x_path.clone());
            p_points.extend(p_int2.p_path);

            // cuts.push(
            //     Cut::new(Component::Xp, x_cuts, vec![], CutType::DebugPath).log_branch(p_range),
            // );

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.iter().map(|p| p.conj()).collect()],
                vec![],
                CutType::U(Component::Xp),
            ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.clone()],
                vec![],
                CutType::U(Component::Xm),
            ));
        }

        if p_range == -1 {
            let mut p_points: Vec<C> = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;

            // let x = get_branch_point_x(m - 1.0 - consts.k() as f64, consts, -1.0);

            let p_int = PInterpolator::xp(p0, consts)
                .goto_xp(p0, consts.k() as f64)
                .goto_xm(p0, consts.k() as f64)
                .goto_xm(p0, 0.0);

            // x_cuts.push(p_int.x_path.clone());

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xm(p_start + 0.01, 0.0);
            // x_cuts.push(p_int2.x_path.clone());
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xm(p_start + 0.9999, 0.0);
            let branch_point = *p_int2.p_path.last().unwrap();
            // x_cuts.push(p_int2.x_path.clone());
            p_points.extend(p_int2.p_path);

            // cuts.push(
            //     Cut::new(Component::Xp, x_cuts, vec![x], CutType::DebugPath)
            //         .log_branch(p_range),
            // );

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.iter().map(|p| p.conj()).collect()],
                vec![branch_point.conj()],
                CutType::U(Component::Xm),
            ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.clone()],
                vec![branch_point],
                CutType::U(Component::Xp),
            ));
        }

        if p_range > 0 {
            let mut p_points: Vec<C> = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 7.0 / 8.0;

            let m = -2.0 - 2.0 * p_range as f64 * consts.k() as f64;

            // let x = get_branch_point_x(m - 1.0 - consts.k() as f64, consts, -1.0);

            let p_int = PInterpolator::xp(p0, consts)
                .goto_xm(p0, 1.0)
                .goto_xm(p2, 1.0)
                .goto_xm(p2, m);

            // x_cuts.push(p_int.x_path.clone());

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xm(p_start + 0.01, m);
            // x_cuts.push(p_int2.x_path.clone());
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xm(p_start + 0.9999, m);
            // x_cuts.push(p_int2.x_path.clone());
            p_points.extend(p_int2.p_path);

            // cuts.push(
            //     Cut::new(Component::Xp, x_cuts, vec![], CutType::DebugPath).log_branch(p_range),
            // );

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.iter().map(|p| p.conj()).collect()],
                vec![],
                CutType::U(Component::Xp),
            ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.clone()],
                vec![],
                CutType::U(Component::Xm),
            ));
        }

        if p_range <= -1 {
            let mut p_points: Vec<C> = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 32.0;
            let p2 = p_range as f64 + 1.0 / 8.0;

            let m = -2.0 - 2.0 * p_range as f64 * consts.k() as f64;
            let m1 = 2.0 - 1.0 * p_range as f64 * consts.k() as f64;

            // let x = get_branch_point_x(m - 1.0 - consts.k() as f64, consts, -1.0);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xm(p2, 1.0)
                .goto_xm(p2, m1)
                .goto_xm(p0, m1)
                .goto_xm(p0, m);

            // x_cuts.push(p_int.x_path.clone());

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xm(p_start + 0.01, m);
            // x_cuts.push(p_int2.x_path.clone());
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xm(p_start + 0.99, m);
            // x_cuts.push(p_int2.x_path.clone());
            p_points.extend(p_int2.p_path);

            // cuts.push(
            //     Cut::new(Component::Xp, x_cuts, vec![], CutType::DebugPath).log_branch(p_range),
            // );

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.iter().map(|p| p.conj()).collect()],
                vec![],
                CutType::U(Component::Xp),
            ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.clone()],
                vec![],
                CutType::U(Component::Xm),
            ));
        }

        cuts
    }

    fn x_cuts_x(p_range: i32, consts: CouplingConstants) -> Vec<Cut> {
        let mut cuts = vec![];

        let branch_points = if p_range == 0 {
            vec![C::from(consts.s())]
        } else if p_range == -1 {
            vec![C::from(-1.0 / consts.s())]
        } else {
            vec![]
        };

        cuts.push(
            Cut::new(
                Component::Xp,
                vec![XInterpolator::generate_xp_full(p_range, 0.0, consts)],
                branch_points.clone(),
                CutType::U(Component::Xp),
            )
            .log_branch(p_range),
        );

        cuts.push(
            Cut::new(
                Component::Xm,
                vec![XInterpolator::generate_xm_full(p_range, 0.0, consts)],
                branch_points.clone(),
                CutType::U(Component::Xm),
            )
            .log_branch(p_range),
        );

        cuts.push(
            Cut::new(
                Component::Xp,
                vec![XInterpolator::generate_xm_full(p_range, 0.0, consts)],
                branch_points.clone(),
                CutType::U(Component::Xp),
            )
            .log_branch(p_range),
        );

        cuts.push(
            Cut::new(
                Component::Xm,
                vec![XInterpolator::generate_xp_full(p_range, 0.0, consts)],
                branch_points.clone(),
                CutType::U(Component::Xm),
            )
            .log_branch(p_range),
        );

        if p_range == 0 {
            let ps = get_branch_point(1.0, consts, 0.0);

            let paths = vec![XInterpolator::generate_xp(ps, 1.0, 2.0, consts)];
            let branch_points = vec![*paths[0].first().unwrap()];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range),
            );

            let paths = vec![XInterpolator::generate_xm(ps, 1.0, 2.0, consts)];
            let branch_points = vec![*paths[0].first().unwrap()];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range),
            );

            let ps = get_branch_point(-1.0, consts, 0.0);

            let paths = vec![XInterpolator::generate_xm(ps, 1.0, -2.0, consts)];
            let branch_points = vec![*paths[0].first().unwrap()];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range),
            );

            let paths = vec![XInterpolator::generate_xp(ps, 1.0, -2.0, consts)];
            let branch_points = vec![*paths[0].first().unwrap()];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range),
            );
        } else if p_range == -1 {
            let ps = get_branch_point(1.0, consts, 1.0);

            let paths = vec![XInterpolator::generate_xp(
                ps,
                0.0,
                2.0 - consts.k() as f64,
                consts,
            )];
            let branch_points = vec![*paths[0].last().unwrap()];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range)
                .im_xm_negative(),
            );

            let paths = vec![XInterpolator::generate_xm(
                ps,
                0.0,
                2.0 - consts.k() as f64,
                consts,
            )];
            let branch_points = vec![*paths[0].last().unwrap()];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range)
                .im_xp_positive(),
            );

            let m =
                1.0 + if p_range < 0 { p_range - 1 } else { p_range } as f64 * consts.k() as f64;

            let p_minus_one_over_s = get_branch_point(m, consts, m.signum());
            let m = 2.0 + (2 * p_range - 1) as f64 * consts.k() as f64;

            let paths = vec![XInterpolator::generate_xp(
                p_minus_one_over_s.floor(),
                p_minus_one_over_s,
                m,
                consts,
            )];
            let branch_points = vec![paths[0][0]];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range)
                .im_xm_positive(),
            );

            let paths = vec![XInterpolator::generate_xm(
                p_minus_one_over_s.floor(),
                p_minus_one_over_s,
                m,
                consts,
            )];
            let branch_points = vec![paths[0][0]];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range)
                .im_xp_negative(),
            );
        } else {
            let paths = vec![XInterpolator::generate_xp(
                0.0,
                1.0,
                2.0 + p_range as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range)
                .im_xm_negative(),
            );

            let paths = vec![XInterpolator::generate_xm(
                0.0,
                1.0,
                2.0 + p_range as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range)
                .im_xp_positive(),
            );
        }

        if p_range > 0 {
            let paths = vec![XInterpolator::generate_xp(
                0.0,
                1.0,
                2.0 + (3 * p_range) as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range)
                .im_xm_positive(),
            );

            let paths = vec![XInterpolator::generate_xm(
                0.0,
                1.0,
                -2.0 - p_range as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range),
            );

            let paths = vec![XInterpolator::generate_xm(
                0.0,
                1.0,
                2.0 + (3 * p_range) as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range)
                .im_xp_negative(),
            );

            let paths = vec![XInterpolator::generate_xp(
                0.0,
                1.0,
                -2.0 - p_range as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range),
            );
        }

        if p_range < -1 {
            let paths = vec![XInterpolator::generate_xp(
                0.0,
                1.0,
                2.0 + 3.0 * p_range as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range)
                .im_xm_positive(),
            );

            let paths = vec![XInterpolator::generate_xm(
                0.0,
                1.0,
                2.0 + 3.0 * p_range as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range)
                .im_xp_negative(),
            );
        }

        if p_range <= -1 {
            let paths = vec![XInterpolator::generate_xm(
                0.0,
                1.0,
                -2.0 - p_range as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xp,
                    paths,
                    branch_points,
                    CutType::U(Component::Xm),
                )
                .log_branch(p_range),
            );

            let paths = vec![XInterpolator::generate_xp(
                0.0,
                1.0,
                -2.0 - p_range as f64 * consts.k() as f64,
                consts,
            )];
            let branch_points = vec![];

            cuts.push(
                Cut::new(
                    Component::Xm,
                    paths,
                    branch_points,
                    CutType::U(Component::Xp),
                )
                .log_branch(p_range),
            );
        }

        cuts
    }

    fn p_cuts_x(p_range: i32, consts: CouplingConstants) -> Vec<Cut> {
        let mut cuts = vec![];

        // xp negative axis from above
        let paths = vec![vec![C::from(-100.0), C::zero()]];
        let branch_points = vec![C::from(-1.0 / consts.s()), C::zero()];

        cuts.push(
            Cut::new(
                Component::Xp,
                paths,
                branch_points,
                CutType::LogX(Component::Xp, 1),
            )
            .log_branch(p_range)
            .im_xp_positive(),
        );

        // xp negative axis from below
        let paths = vec![vec![C::from(-100.0), C::zero()]];
        let branch_points = vec![C::from(-1.0 / consts.s()), C::zero()];

        cuts.push(
            Cut::new(
                Component::Xp,
                paths,
                branch_points,
                CutType::LogX(Component::Xp, -1),
            )
            .log_branch(p_range)
            .im_xp_negative(),
        );

        // xm negative axis from below
        let paths = vec![vec![C::from(-100.0), C::zero()]];
        let branch_points = vec![C::from(-1.0 / consts.s()), C::zero()];

        cuts.push(
            Cut::new(
                Component::Xm,
                paths,
                branch_points,
                CutType::LogX(Component::Xm, 1),
            )
            .log_branch(p_range)
            .im_xm_negative(),
        );

        // xm negative axis from above
        let paths = vec![vec![C::from(-100.0), C::zero()]];
        let branch_points = vec![C::from(-1.0 / consts.s()), C::zero()];

        cuts.push(
            Cut::new(
                Component::Xm,
                paths,
                branch_points,
                CutType::LogX(Component::Xm, -1),
            )
            .log_branch(p_range)
            .im_xm_positive(),
        );

        // xp image of xm negative real axis from below

        let m = 1.0 + if p_range < 0 { p_range } else { p_range + 1 } as f64 * consts.k() as f64;

        let p_minus_one_over_s = get_branch_point(m, consts, m.signum());
        let m = 2.0 + (2 * p_range + 1) as f64 * consts.k() as f64;

        let paths = vec![XInterpolator::generate_xp(
            p_minus_one_over_s,
            p_minus_one_over_s.ceil(),
            m,
            consts,
        )];
        let branch_points = vec![paths[0][0]];

        cuts.push(
            Cut::new(
                Component::Xp,
                paths,
                branch_points,
                CutType::LogX(Component::Xm, 1),
            )
            .log_branch(p_range)
            .im_xm_negative(),
        );

        // xm image of xp negative real axis from above

        let paths = vec![XInterpolator::generate_xm(
            p_minus_one_over_s,
            p_minus_one_over_s.ceil(),
            m,
            consts,
        )];
        let branch_points = vec![paths[0][0]];

        cuts.push(
            Cut::new(
                Component::Xm,
                paths,
                branch_points,
                CutType::LogX(Component::Xp, 1),
            )
            .log_branch(p_range)
            .im_xp_positive(),
        );

        // xp image of xm negative real axis from above

        let m = 1.0 + if p_range < 0 { p_range - 1 } else { p_range } as f64 * consts.k() as f64;

        let p_minus_one_over_s = get_branch_point(m, consts, m.signum());
        let m = 2.0 + (2 * p_range - 1) as f64 * consts.k() as f64;

        let paths = vec![XInterpolator::generate_xp(
            p_minus_one_over_s,
            p_minus_one_over_s.ceil(),
            m,
            consts,
        )];
        let branch_points = vec![paths[0][0]];

        cuts.push(
            Cut::new(
                Component::Xp,
                paths,
                branch_points,
                CutType::LogX(Component::Xm, -1),
            )
            .log_branch(p_range)
            .im_xm_positive(),
        );

        // xm image of xp negative real axis from below

        let paths = vec![XInterpolator::generate_xm(
            p_minus_one_over_s,
            p_minus_one_over_s.ceil(),
            m,
            consts,
        )];
        let branch_points = vec![paths[0][0]];

        cuts.push(
            Cut::new(
                Component::Xm,
                paths,
                branch_points,
                CutType::LogX(Component::Xp, -1),
            )
            .log_branch(p_range)
            .im_xp_negative(),
        );

        // xp real positive axis

        let paths = vec![vec![C::zero(), C::from(100.0)]];
        let branch_points = vec![C::from(consts.s()), C::zero()];

        cuts.push(
            Cut::new(
                Component::Xp,
                paths,
                branch_points,
                CutType::LogX(Component::Xp, 0),
            )
            .log_branch(p_range),
        );

        // xm real positive axis

        let paths = vec![vec![C::zero(), C::from(100.0)]];
        let branch_points = vec![C::from(consts.s()), C::zero()];

        cuts.push(
            Cut::new(
                Component::Xm,
                paths,
                branch_points,
                CutType::LogX(Component::Xm, 0),
            )
            .log_branch(p_range),
        );

        // xp image of xm real positive axis

        let m = 2.0 + (2 * p_range) as f64 * consts.k() as f64;
        let p_s = get_branch_point(m / 2.0, consts, 0.0);

        let paths = vec![XInterpolator::generate_xp(p_s.floor(), p_s, m, consts)];
        let branch_points = vec![*paths[0].last().unwrap()];

        cuts.push(
            Cut::new(
                Component::Xp,
                paths,
                branch_points,
                CutType::LogX(Component::Xm, 0),
            )
            .log_branch(p_range),
        );

        // xm image of xp real positive axis

        let paths = vec![XInterpolator::generate_xm(p_s.floor(), p_s, m, consts)];
        let branch_points = vec![*paths[0].last().unwrap()];

        cuts.push(
            Cut::new(
                Component::Xm,
                paths,
                branch_points,
                CutType::LogX(Component::Xp, 0),
            )
            .log_branch(p_range),
        );

        cuts
    }

    fn p_cuts_p(p_range: i32, consts: CouplingConstants) -> Vec<Cut> {
        let mut cuts = vec![];
        let p_start = p_range as f64;

        if p_range == 0 {
            // Real negative axis

            let m = consts.k() as f64 + 2.0;
            let p1 = get_branch_point(m - 1.0, consts, 1.0);

            let mut p_points = vec![];
            // let mut x_cuts = vec![];

            let p0 = 1.0 / 8.0;
            let p2 = 7.0 / 8.0;

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, 3.0)
                .goto_xp(p0, 3.0)
                .goto_xp(p0, m + 1.0)
                .goto_xp(p2, m + 1.0)
                .goto_xp(p2, m);

            // x_cuts.push(p_int.x_path.clone());

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(0.99, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            let branch_point = *p_int2.p_path.last().unwrap();
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p2, consts).goto_xp(p2, m);
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(0.99, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            // cuts.push(Cut::new(
            //     Component::Xp,
            //     x_cuts,
            //      vec![],
            //      CutType::DebugPath,
            //
            // ));

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.iter().map(|p| p.conj()).collect()],
                    vec![branch_point.conj()],
                    CutType::LogX(Component::Xp, 1),
                )
                .log_branch_le(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.clone()],
                    vec![branch_point],
                    CutType::LogX(Component::Xm, 1),
                )
                .log_branch_le(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.iter().map(|p| p.conj()).collect()],
                    vec![branch_point.conj()],
                    CutType::LogX(Component::Xp, -1),
                )
                .log_branch_gt(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.clone()],
                    vec![branch_point],
                    CutType::LogX(Component::Xm, -1),
                )
                .log_branch_gt(p_range),
            );

            // Real positive axis

            let m = 2.0;
            let p1 = get_branch_point(m - 1.0, consts, 0.0);

            let mut p_points = vec![];
            // let mut x_cuts = vec![];

            let p0 = 1.0 / 8.0;
            let p2 = 7.0 / 8.0;

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m + 1.0)
                .goto_xp(p0, m + 1.0)
                .goto_xp(p0, m);

            // x_cuts.push(p_int.x_path.clone());

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(0.01, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            let branch_point = *p_int2.p_path.last().unwrap();
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p0, consts).goto_xp(p0, m);
            // x_cuts.push(p_int.x_path.clone());

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(0.01, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            // cuts.push(Cut::new(
            //     Component::Xp,
            //     x_cuts,
            //      vec![],
            //      CutType::DebugPath,
            //
            // ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.iter().map(|p| p.conj()).collect()],
                vec![branch_point.conj()],
                CutType::LogX(Component::Xp, 0),
            ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points],
                vec![branch_point],
                CutType::LogX(Component::Xm, 0),
            ));
        } else if p_range == -1 {
            // Real negative axis

            let mut p_points = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 1.0 - 1.0 / 8.0;

            let m = 2.0;

            let p1 =
                get_branch_point(m - 1.0 + p_start * consts.k() as f64, consts, -1.0) + p_start;

            let p_int = PInterpolator::xp(p2, consts).goto_xp(p2, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_start + 0.99, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            let branch_point = *p_int2.p_path.last().unwrap();
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p0, consts)
                .goto_xp(p0, m + 1.0)
                .goto_xp(p2, m + 1.0)
                .goto_xp(p2, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p_start + 0.99, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            // cuts.push(Cut::new(
            //     Component::Xp,
            //     x_cuts,
            //      vec![],
            //      CutType::DebugPath,
            //
            // ));

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.iter().map(|p| p.conj()).collect()],
                    vec![branch_point.conj()],
                    CutType::LogX(Component::Xp, 1),
                )
                .log_branch_le(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.clone()],
                    vec![branch_point],
                    CutType::LogX(Component::Xm, 1),
                )
                .log_branch_le(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.iter().map(|p| p.conj()).collect()],
                    vec![branch_point.conj()],
                    CutType::LogX(Component::Xp, -1),
                )
                .log_branch_gt(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.clone()],
                    vec![branch_point],
                    CutType::LogX(Component::Xm, -1),
                )
                .log_branch_gt(p_range),
            );

            // Real positive axis

            let mut p_points = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 1.0 - 1.0 / 8.0;

            let m = 2.0 + p_start * consts.k() as f64;

            let p1 = get_branch_point(m - 1.0, consts, 0.0) + p_start;

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m - 1.0)
                .goto_xp(p0, m - 1.0)
                .goto_xp(p0, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_start + 0.01, m);
            p_points.extend(p_int2.p_path.into_iter().rev());

            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m + 1.0)
                .goto_xp(p0, m + 1.0)
                .goto_xp(p0, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            let branch_point = *p_int2.p_path.last().unwrap();
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p_start + 0.01, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            // cuts.push(Cut::new(
            //     Component::Xp,
            //     x_cuts,
            //      vec![],
            //      CutType::DebugPath,
            //
            // ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.iter().map(|p| p.conj()).collect()],
                vec![branch_point.conj()],
                CutType::LogX(Component::Xp, 0),
            ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.clone()],
                vec![branch_point],
                CutType::LogX(Component::Xm, 0),
            ));
        } else if p_range > 0 {
            // Real negative axis

            let mut p_points = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 1.0 - 1.0 / 8.0;

            let m = (p_start + 1.0) * consts.k() as f64 + 2.0;

            let p1 = get_branch_point(m - 1.0 - consts.k() as f64, consts, -1.0) + p_start;

            // let x = get_branch_point_x(m - 1.0 - consts.k() as f64, consts, -1.0);

            let p_int = PInterpolator::xp(p2, consts).goto_xp(p2, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_start + 0.99, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            let branch_point = *p_int2.p_path.last().unwrap();
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m - 1.5)
                .goto_xp(p0, m - 1.5)
                .goto_xp(p0, m + 1.5)
                .goto_xp(p2, m + 1.5)
                .goto_xp(p2, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p_start + 0.99, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            // cuts.push(Cut::new(
            //     Component::Xp,
            //     x_cuts,
            //      vec![x],
            //      CutType::DebugPath,
            //     log_branch(p_range)
            // );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.iter().map(|p| p.conj()).collect()],
                    vec![branch_point.conj()],
                    CutType::LogX(Component::Xp, 1),
                )
                .log_branch_le(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.clone()],
                    vec![branch_point],
                    CutType::LogX(Component::Xm, 1),
                )
                .log_branch_le(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.iter().map(|p| p.conj()).collect()],
                    vec![branch_point.conj()],
                    CutType::LogX(Component::Xp, -1),
                )
                .log_branch_gt(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.clone()],
                    vec![branch_point],
                    CutType::LogX(Component::Xm, -1),
                )
                .log_branch_gt(p_range),
            );

            // Real positive axis

            let mut p_points = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 1.0 - 1.0 / 8.0;

            let m = p_start * consts.k() as f64 + 2.0;

            let p1 = get_branch_point(m - 1.0, consts, 0.0) + p_start;

            // let x = get_branch_point_x(m - 1.0, consts, 0.0);

            let p_int = PInterpolator::xp(p0, consts).goto_xp(p0, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_start + 0.01, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            let branch_point = *p_int2.p_path.last().unwrap();
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m + 1.5)
                .goto_xp(p0, m + 1.5)
                .goto_xp(p0, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p_start + 0.01, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            // cuts.push(Cut::new(
            //     Component::Xp,
            //     x_cuts,
            //      vec![x],
            //      CutType::DebugPath,
            //     log_branch(p_range)
            // );

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.iter().map(|p| p.conj()).collect()],
                vec![branch_point.conj()],
                CutType::LogX(Component::Xp, 0),
            ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.clone()],
                vec![branch_point],
                CutType::LogX(Component::Xm, 0),
            ));
        } else if p_range < -1 {
            // Real negative axis

            let mut p_points = vec![];
            // let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 1.0 - 1.0 / 8.0;

            let m = (p_start + 1.0) * consts.k() as f64 + 2.0;

            let p1 = get_branch_point(m - 1.0 - consts.k() as f64, consts, -1.0) + p_start;

            // let x = get_branch_point_x(m - 1.0 - consts.k() as f64, consts, -1.0);

            let p_int = PInterpolator::xp(p2, consts).goto_xp(p2, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_start + 0.99, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            let branch_point = *p_int2.p_path.last().unwrap();
            p_points.extend(p_int2.p_path);
            // // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m - 1.5)
                .goto_xp(p0, m - 1.5)
                .goto_xp(p0, m + 1.5)
                .goto_xp(p2, m + 1.5)
                .goto_xp(p2, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p_start + 0.99, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            // cuts.push(Cut::new(
            //     Component::Xp,
            //     x_cuts,
            //      vec![x],
            //      CutType::DebugPath,
            //     log_branch(p_range)
            // );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.iter().map(|p| p.conj()).collect()],
                    vec![branch_point.conj()],
                    CutType::LogX(Component::Xp, 1),
                )
                .log_branch_le(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.clone()],
                    vec![branch_point],
                    CutType::LogX(Component::Xm, 1),
                )
                .log_branch_le(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.iter().map(|p| p.conj()).collect()],
                    vec![branch_point.conj()],
                    CutType::LogX(Component::Xp, -1),
                )
                .log_branch_gt(p_range),
            );

            cuts.push(
                Cut::new(
                    Component::P,
                    vec![p_points.clone()],
                    vec![branch_point],
                    CutType::LogX(Component::Xm, -1),
                )
                .log_branch_gt(p_range),
            );

            // Positive real axis

            let mut p_points = vec![];
            let mut x_cuts = vec![];

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 1.0 - 1.0 / 8.0;

            let m = (p_start) * consts.k() as f64 + 2.0;

            let p1 = get_branch_point(m - 1.0, consts, 0.0) + p_start;

            let x = get_branch_point_x(m - 1.0, consts, 0.0);

            let p_int = PInterpolator::xp(p0, consts).goto_xp(p0, m);

            x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_start + 0.01, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            let branch_point = *p_int2.p_path.last().unwrap();
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p0, consts)
                .goto_xp(p0, m + 1.5)
                .goto_xp(p2, m + 1.5)
                .goto_xp(p2, m - 1.5)
                .goto_xp(p0, m - 1.5)
                .goto_xp(p0, m);

            // x_cuts.push(p_int.x_path.clone());
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p_start + 0.01, m);
            p_points.extend(p_int2.p_path);
            // // x_cuts.push(p_int2.x_path);

            // cuts.push(
            //     Cut::new(Component::Xp, x_cuts, vec![x], CutType::DebugPath).log_branch(p_range),
            // );

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.iter().map(|p| p.conj()).collect()],
                vec![branch_point.conj()],
                CutType::LogX(Component::Xp, 0),
            ));

            cuts.push(Cut::new(
                Component::P,
                vec![p_points.clone()],
                vec![branch_point],
                CutType::LogX(Component::Xm, 0),
            ));
        }

        cuts
    }

    fn e_cuts(p_range: i32, consts: CouplingConstants) -> Vec<Cut> {
        let p_start = p_range as f64;

        let p0 = nr::find_root(
            |p| en2(p, 1.0, consts),
            |p| den2_dp(p, 1.0, consts),
            C::new(p_start, 2.5),
            1.0e-3,
            50,
        );

        let Some(p0) = p0 else { return vec![] };

        let mut cut = vec![];

        cut.push((0.0, p0));
        let mut p_prev = p0;

        const STEP: i32 = 16;

        for i in 1.. {
            let im = i as f64 * i as f64 / (STEP as f64);

            let p = nr::find_root(
                |p| en2(p, 1.0, consts) - C::new(-im, 0.0),
                |p| den2_dp(p, 1.0, consts),
                p_prev,
                1.0e-3,
                50,
            );

            let Some(p) = p else {break;};

            cut.push((im, p));
            p_prev = p;

            if p.im.abs() > 2.0 {
                break;
            }
        }

        let cut = cut
            .into_iter()
            .map(|(im, p)| {
                (
                    im,
                    p,
                    xp(p + 0.001, 1.0, consts),
                    xp(p - 0.001, 1.0, consts),
                    xm(p + 0.001, 1.0, consts),
                    xm(p - 0.001, 1.0, consts),
                )
            })
            .collect::<Vec<_>>();

        let mut cuts = vec![];

        // p

        let paths = vec![cut.iter().map(|(_, p, _, _, _, _)| *p).collect()];
        let branch_points = vec![p0];

        cuts.push(Cut::new(Component::P, paths, branch_points, CutType::E));

        let paths = vec![cut.iter().map(|(_, p, _, _, _, _)| p.conj()).collect()];
        let branch_points = vec![p0.conj()];

        cuts.push(Cut::new(Component::P, paths, branch_points, CutType::E));

        // xp

        let mut paths = vec![
            cut.iter()
                .map(|(_, _, xp, _, _, _)| *xp)
                .collect::<Vec<_>>(),
            cut.iter()
                .map(|(_, _, _, xp, _, _)| *xp)
                .collect::<Vec<_>>(),
        ];
        paths.push(vec![paths[0][0], paths[1][0]]);
        let branch_points = vec![(paths[0][0] + paths[1][0]) / 2.0];
        cuts.push(if branch_points[0].im > 0.0 {
            Cut::new(Component::Xp, paths, branch_points, CutType::E)
                .log_branch(p_range)
                .im_xm_positive()
        } else {
            Cut::new(Component::Xp, paths, branch_points, CutType::E).log_branch(p_range)
        });

        let mut paths = vec![
            cut.iter()
                .map(|(_, _, _, _, xm, _)| xm.conj())
                .collect::<Vec<_>>(),
            cut.iter()
                .map(|(_, _, _, _, _, xm)| xm.conj())
                .collect::<Vec<_>>(),
        ];
        paths.push(vec![paths[0][0], paths[1][0]]);
        let branch_points = vec![(paths[0][0] + paths[1][0]) / 2.0];
        cuts.push(if branch_points[0].im > 0.0 {
            Cut::new(Component::Xp, paths, branch_points, CutType::E)
                .log_branch(p_range)
                .im_xm_positive()
        } else {
            Cut::new(Component::Xp, paths, branch_points, CutType::E).log_branch(p_range)
        });

        // xm

        let mut paths = vec![
            cut.iter()
                .map(|(_, _, xp, _, _, _)| xp.conj())
                .collect::<Vec<_>>(),
            cut.iter()
                .map(|(_, _, _, xp, _, _)| xp.conj())
                .collect::<Vec<_>>(),
        ];
        paths.push(vec![paths[0][0], paths[1][0]]);
        let branch_points = vec![(paths[0][0] + paths[1][0]) / 2.0];

        cuts.push(if branch_points[0].im < 0.0 {
            Cut::new(Component::Xm, paths, branch_points, CutType::E)
                .log_branch(p_range)
                .im_xp_negative()
        } else {
            Cut::new(Component::Xm, paths, branch_points, CutType::E).log_branch(p_range)
        });

        let mut paths = vec![
            cut.iter()
                .map(|(_, _, _, _, xm, _)| *xm)
                .collect::<Vec<_>>(),
            cut.iter()
                .map(|(_, _, _, _, _, xm)| *xm)
                .collect::<Vec<_>>(),
        ];
        paths.push(vec![paths[0][0], paths[1][0]]);
        let branch_points = vec![(paths[0][0] + paths[1][0]) / 2.0];

        cuts.push(if branch_points[0].im < 0.0 {
            Cut::new(Component::Xm, paths, branch_points, CutType::E)
                .log_branch(p_range)
                .im_xp_negative()
        } else {
            Cut::new(Component::Xm, paths, branch_points, CutType::E).log_branch(p_range)
        });

        cuts
    }
}

pub struct OldCut {
    pub cut_p: Vec<Vec<C>>,
    pub cut_xp: Vec<Vec<C>>,
    pub cut_xm: Vec<Vec<C>>,
    pub cut_u: Vec<Vec<C>>,

    pub branch_points_p: Vec<C>,
    pub branch_points_x: Vec<C>,
    pub branch_points_u: Vec<C>,
}

impl OldCut {
    pub fn get(p_range: i32, consts: CouplingConstants) -> Vec<Self> {
        vec![
            Self::x(p_range, consts),
            // Self::e(p_range, consts),
            // Self::x_log(p_range - 1, consts),
            Self::x_log(p_range, consts),
            // Self::x_log(p_range + 1, consts),
        ]
    }

    fn x(p_range: i32, consts: CouplingConstants) -> Self {
        let p_start = p_range as f64;

        let p_s = {
            let p0 = 1.0 / 8.0;
            let p_int = PInterpolator::xp(p0, consts)
                .goto_im(0.0)
                .goto_re(consts.s());
            let p_int = p_int.clear_path();
            *p_int.p_path.last().unwrap()
        };

        let p_min_one_over_s = {
            let p0 = -1.0 / 64.0;
            let p_int = PInterpolator::xp(p0, consts)
                .goto_im(0.0)
                .goto_re(-1.0 / consts.s());
            let p_int = p_int.clear_path();
            *p_int.p_path.last().unwrap()
        };

        let mut p_points = vec![];
        {
            p_points.push(C::from(p_start));
            if p_range != -1 {
                let p0 = p_start + 0.25;
                let p_int = PInterpolator::xp(p0, consts)
                    .goto_xm(p0, 1.0)
                    .goto_xm(p0, 0.0);
                let p_int = p_int.clear_path();

                let p_int2 = p_int.clone().goto_xm(p_start + 127.0 / 128.0, 0.0);
                p_points.extend(p_int2.p_path.into_iter().rev());

                let p_int2 = p_int.clone().goto_xm(p_start + 1.0 / 128.0, 0.0);
                p_points.extend(p_int2.p_path);

                if p_range == 0 {
                    p_points.push(p_s);
                } else {
                    p_points.push(C::from(p_start));
                }
            }

            {
                let p0 = p_start + 1.0 / 8.0;
                let p_int = PInterpolator::xp(p0, consts).goto_xp(p0, 0.0);
                let p_int = p_int.clear_path();

                let p_int2 = p_int.clone().goto_xp(p_start + 1.0 / 64.0, 0.0);
                p_points.extend(p_int2.p_path.into_iter().rev());

                let p_int2 = p_int.goto_xp(p_start + 1.0 - 1.0 / 64.0, 0.0);
                p_points.extend(p_int2.p_path);

                if p_range != -1 {
                    p_points.push(C::from(p_start + 1.0));
                } else {
                    p_points.push(p_min_one_over_s);
                }
            }

            if p_range == -1 {
                let p0 = p_start + 0.9;
                let p_int = PInterpolator::xp(p0, consts)
                    .goto_xm(p0, 1.0)
                    .goto_xm(p0, 0.0);
                let p_int = p_int.clear_path();
                let p_int2 = p_int.clone().goto_xm(p_start + 0.999, 0.0);
                p_points.extend(p_int2.p_path.into_iter().rev());

                let tst = p_int.clone().go_towards_xm(p_start + 0.001, 0.0);

                if let InterpolationPoint::Xm(p, _) = tst.pt {
                    // let p_int2 = p_int.goto_xm(p_start + 0.1, 0.0);
                    let p_int2 = p_int.go_towards_xm(p, 0.0);
                    p_points.extend(p_int2.p_path);
                }
            }
        }

        let mut x_points = vec![];

        if p_range == 0 {
            x_points.push(C::from(consts.s()));
        } else if p_range < 0 {
            x_points.push(C::zero());
        }

        let steps = 16;

        for i in 1..=(steps - 1) {
            let p = p_start + i as f64 / (steps as f64);
            x_points.push(xp(p, 0.0, consts));
        }

        if p_range == -1 {
            x_points.push(C::from(-1.0 / consts.s()));
        } else if p_range < -1 {
            x_points.push(C::zero());
        }

        let mut u_points = vec![];

        let u_s = {
            let s = consts.s();
            s + 1.0 / s - 2.0 * consts.kslash() / consts.h * s.ln()
        };
        if p_range == 0 {
            u_points.push(C::new(-100.0, -1.0 / consts.h));
            u_points.push(C::new(u_s, -1.0 / consts.h));
        } else if p_range == -1 {
            u_points.push(C::new(-u_s, -1.0 / consts.h));
            u_points.push(C::new(100.0, -1.0 / consts.h));
        } else {
            u_points.push(C::new(-100.0, -1.0 / consts.h));
            u_points.push(C::new(100.0, -1.0 / consts.h));
        }

        let pb = get_branch_point(1.0, consts, 0.0);

        let cut_xp = vec![
            x_points.iter().rev().map(|z| *z).collect::<Vec<_>>(),
            x_points.iter().map(|z| z.conj()).collect::<Vec<_>>(),
            // XInterpolator::generate_xp(pb, 1.0, p_start * consts.k() as f64 + 2.0, consts),
        ];

        let cut_xm = vec![
            x_points.iter().rev().map(|z| *z).collect::<Vec<_>>(),
            x_points.iter().map(|z| z.conj()).collect::<Vec<_>>(),
            // XInterpolator::generate_xm(pb, 1.0, p_start * consts.k() as f64 + 2.0, consts),
        ];

        let cut_p = vec![
            p_points.iter().map(|z| z.conj()).collect::<Vec<_>>(),
            p_points,
        ];
        let cut_u = vec![
            u_points.iter().map(|z| z.conj()).collect::<Vec<_>>(),
            u_points,
        ];

        let branch_points_p = match p_range {
            0 => vec![p_s, p_s.conj()],
            -1 => vec![p_min_one_over_s, p_min_one_over_s.conj()],
            _ => vec![],
        };
        let branch_points_x = match p_range {
            0 => vec![
                C::from(consts.s()),
                xp(pb, 2.0, consts),
                xm(pb, 2.0, consts),
            ],
            -1 => vec![
                C::from(-1.0 / consts.s()),
                xp(pb, 2.0, consts),
                xm(pb, 2.0, consts),
            ],
            _ => vec![xp(pb, 2.0, consts), xm(pb, 2.0, consts)],
        };
        let branch_points_u = match p_range {
            0 => vec![C::new(u_s, 1.0 / consts.h), C::new(u_s, -1.0 / consts.h)],
            -1 => vec![C::new(-u_s, 1.0 / consts.h), C::new(-u_s, -1.0 / consts.h)],
            _ => vec![],
        };

        Self {
            cut_p,
            cut_xp,
            cut_xm,
            cut_u,
            branch_points_p,
            branch_points_x,
            branch_points_u,
        }
    }

    fn e(p_range: i32, consts: CouplingConstants) -> Self {
        let p0 = nr::find_root(
            |p| en2(p, 1.0, consts) - C::new(0.0, 0.0),
            |p| den2_dp(p, 1.0, consts),
            C::new(0.0, 2.5),
            // C::new(0.0, 0.5),
            1.0e-5,
            50,
        )
        .unwrap();

        let mut cut_p = vec![p0];
        for i in 0..=256 {
            let im = i as f64 * i as f64 / 64.0;

            let p = nr::find_root(
                |p| en2(p, 1.0, consts) - C::new(-im, 0.0),
                |p| den2_dp(p, 1.0, consts),
                *cut_p.last().unwrap(),
                1.0e-5,
                50,
            );

            cut_p.push(p.unwrap());
        }

        let mut xp_points = vec![];

        xp_points.push(C::zero());
        xp_points.extend(
            cut_p
                .iter()
                .rev()
                .map(|p| xp(*p + C::from(1.0e-5), 1.0, consts)),
        );
        xp_points.extend(cut_p.iter().map(|p| xp(*p + C::from(-1.0e-5), 1.0, consts)));
        xp_points.push(C::zero());

        let mut xm_points = vec![];

        xm_points.extend(
            cut_p
                .iter()
                .rev()
                .map(|p| xm(*p + C::from(1.0e-5), 1.0, consts)),
        );
        xm_points.extend(cut_p.iter().map(|p| xm(*p + C::from(-1.0e-5), 1.0, consts)));

        // let mut u_points = vec![];

        // u_points.extend(
        //     cut_p
        //         .iter()
        //         .rev()
        //         .map(|p| u(*p + C::from(1.0e-5), consts, p_range)),
        // );
        // u_points.extend(
        //     cut_p
        //         .iter()
        //         .map(|p| u(*p + C::from(-1.0e-5), consts, p_range)),
        // );

        let cut_p = vec![cut_p.iter().map(|z| z.conj()).collect::<Vec<_>>(), cut_p];
        let x = vec![
            xp_points.iter().map(|z| z.conj()).collect::<Vec<_>>(),
            xp_points,
            xm_points.iter().map(|z| z.conj()).collect::<Vec<_>>(),
            xm_points,
        ];
        let cut_u = vec![
            // u_points.iter().map(|z| z.conj()).collect::<Vec<_>>(),
            // u_points,
        ];

        let branch_points_p = vec![p0, p0.conj()];
        let branch_points_x = vec![
            xp(p0, 1.0, consts),
            xm(p0, 1.0, consts),
            xp(p0, 1.0, consts).conj(),
            xm(p0, 1.0, consts).conj(),
            C::from(-1.0 / consts.s()),
        ];
        let branch_points_u = vec![
        // u(p0, consts, p_range), u(p0, consts, p_range).conj()
        ];

        Self {
            cut_p,
            cut_xp: x.clone(),
            cut_xm: x,
            cut_u,
            branch_points_p,
            branch_points_x,
            branch_points_u,
        }
    }

    fn x_log(p_range: i32, consts: CouplingConstants) -> Self {
        let x_points;
        let mut branch_points_x;
        x_points = vec![C::from(-100.0), C::zero()];
        branch_points_x = vec![C::zero(), C::from(-1.0 / consts.s())];
        // branch_points_x = vec![];

        // if p_range == 0 {
        //     x_points = vec![C::from(-100.0), C::zero()];
        //     branch_points_x = vec![C::zero(), C::from(consts.s()), C::from(-1.0 / consts.s())];
        // } else if p_range == -1 {
        //     x_points = vec![C::zero(), C::from(100.0)];
        //     branch_points_x = vec![C::zero(), C::from(consts.s()), C::from(-1.0 / consts.s())];
        // } else {
        //     x_points = vec![C::from(-100.0), C::from(100.0)];
        //     branch_points_x = vec![C::from(consts.s()), C::from(-1.0 / consts.s())]
        // }

        let mut cut_p = vec![];
        let mut cut_xp = vec![x_points.clone()];
        let mut cut_xm = vec![x_points];

        if p_range == 0 {
            let m = consts.k() as f64 + 2.0;
            let p1 = {
                let u_of_x = |x: C| -> C {
                    let s = consts.s();
                    x + 1.0 / x - (s - 1.0 / s) * x.ln()
                };
                let du_dx = |x: C| -> C {
                    let s = consts.s();
                    (x - s) * (x + 1.0 / s) / (x * x)
                };

                let x = nr::find_root(
                    |x| {
                        u_of_x(x)
                            - u_of_x(C::from(-1.0 / consts.s()))
                            - 2.0 * (m - 1.0) * C::i() / consts.h
                    },
                    du_dx,
                    C::new(0.0, 1.0),
                    1.0e-3,
                    10,
                );
                let x = x.unwrap();
                branch_points_x.push(x);
                x.arg() / std::f64::consts::PI
            };

            let p_minus_one_over_s = get_branch_point(1.0, consts, 1.0);

            cut_xp.push(XInterpolator::generate_xp(
                p1,
                p_range as f64 + 1.0,
                m,
                consts,
            ));

            cut_xm.push(XInterpolator::generate_xm(
                p1,
                p_range as f64 + 1.0,
                m,
                consts,
            ));

            cut_xp.push(XInterpolator::generate_xp(
                p_minus_one_over_s,
                p_range as f64 + 1.0,
                m - 2.0 * consts.k() as f64,
                consts,
            ));

            let p_s = get_branch_point(1.0, consts, 0.0);

            cut_xp.push(XInterpolator::generate_xp(
                p_s,
                p_range as f64 + 1.0,
                2.0,
                consts,
            ));

            let mut p_points = vec![];

            let p0 = 1.0 / 8.0;
            let p2 = 7.0 / 8.0;

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, 3.0)
                .goto_xp(p0, 3.0)
                .goto_xp(p0, m + 1.0)
                .goto_xp(p2, m + 1.0)
                .goto_xp(p2, m);

            // x_cuts.push(p_int.x_path);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(0.99, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            let p_int = PInterpolator::xp(p2, consts).goto_xp(p2, m);
            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            p_points.extend(p_int2.p_path.into_iter().rev());
            // x_cuts.push(p_int2.x_path);

            let p_int2 = p_int.clone().goto_xp(0.99, m);
            p_points.extend(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);

            cut_p.push(p_points.iter().map(|p| p.conj()).collect());
            cut_p.push(p_points);
        }

        if p_range == -1 {
            let m = 1.0;
            let p1 = {
                let u_of_x = |x: C| -> C {
                    let s = consts.s();
                    x + 1.0 / x - (s - 1.0 / s) * x.ln()
                };
                let du_dx = |x: C| -> C {
                    let s = consts.s();
                    (x - s) * (x + 1.0 / s) / (x * x)
                };

                let x = nr::find_root(
                    |x| u_of_x(x) - u_of_x(C::from(consts.s())) - 2.0 * m * C::i() / consts.h,
                    du_dx,
                    C::new(0.0, 1.0),
                    1.0e-3,
                    10,
                );
                let x = x.unwrap();
                branch_points_x.push(x);
                x.arg() / std::f64::consts::PI + p_range as f64
            };

            let mut p_points = vec![];

            let p0 = p_range as f64 + 1.0 / 32.0;
            let p2 = p_range as f64 + 1.0 - 1.0 / 32.0;

            let m = consts.k() as f64 + 2.0;

            cut_xp.push(XInterpolator::generate_xp(p_range as f64, p1, m, consts));
            cut_xm.push(XInterpolator::generate_xm(p_range as f64, p1, m, consts));

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m + 4.0)
                .goto_xp(p0, m + 4.0)
                .goto_xp(p0, m);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_range as f64 + 0.01, m);
            // p_cuts.push(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.goto_xp(p1, m);
            // p_cuts.push(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m - 0.5)
                .goto_xp(p0, m - 0.5)
                .goto_xp(p0, m);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            // p_cuts.push(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.goto_xp(p_range as f64 + 0.01, m);
            // p_cuts.push(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path);

            cut_p.push(p_points.iter().map(|p| p.conj()).collect());
            cut_p.push(p_points);

            let m = p_range as f64 * consts.k() as f64 + 1.0;
            let p1 = {
                let s = consts.s();
                let u_of_x = |x: C| -> C { x + 1.0 / x - (s - 1.0 / s) * x.ln() };
                let du_dx = |x: C| -> C {
                    let s = consts.s();
                    (x - s) * (x + 1.0 / s) / (x * x)
                };

                let x = nr::find_root(
                    |x| {
                        u_of_x(x)
                            - (s + 1.0 / s - (s - 1.0 / s) * s.ln())
                            - 2.0 * (m) * C::i() / consts.h
                    },
                    du_dx,
                    C::new(0.0, 0.1),
                    1.0e-3,
                    10,
                );
                let x = x.unwrap();
                branch_points_x.push(x);
                x.arg() / std::f64::consts::PI + p_range as f64
            };

            let mut p_points = vec![];

            let m = p_range as f64 * consts.k() as f64 + 2.0;

            cut_xp.push(XInterpolator::generate_xp(p_range as f64, p1, m, consts));
            cut_xm.push(XInterpolator::generate_xm(p_range as f64, p1, m, consts));

            let p_int = PInterpolator::xp(p0, consts).goto_xp(p0, m);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_range as f64 + 0.01, m);
            // p_cuts.push(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.goto_xp(p1, m);
            // p_cuts.push(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m - 1.0)
                .goto_xp(p0, m - 1.0)
                .goto_xp(p0, m);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            // p_cuts.push(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.goto_xp(p_range as f64 + 0.01, m);
            // p_cuts.push(p_int2.p_path);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path);

            cut_p.push(p_points.iter().map(|p| p.conj()).collect());
            cut_p.push(p_points);
        }

        if p_range > 0 {
            let m = (p_range + 1) as f64 * consts.k() as f64 + 2.0;
            let p1 = {
                let u_of_x = |x: C| -> C {
                    let s = consts.s();
                    x + 1.0 / x - (s - 1.0 / s) * x.ln()
                };
                let du_dx = |x: C| -> C {
                    let s = consts.s();
                    (x - s) * (x + 1.0 / s) / (x * x)
                };

                let x = nr::find_root(
                    |x| {
                        u_of_x(x)
                            - u_of_x(C::from(-1.0 / consts.s()))
                            - 2.0 * (m - 1.0) * C::i() / consts.h
                    },
                    du_dx,
                    C::new(0.0, 1.0),
                    1.0e-3,
                    10,
                );
                let x = x.unwrap();
                branch_points_x.push(x);
                x.arg() / std::f64::consts::PI + p_range as f64
            };

            cut_xp.push(XInterpolator::generate_xp(
                p1,
                p_range as f64 + 1.0,
                m,
                consts,
            ));
            cut_xm.push(XInterpolator::generate_xm(
                p1,
                p_range as f64 + 1.0,
                m,
                consts,
            ));

            let p0 = p_range as f64 + 1.0 / 8.0;
            let p2 = p_range as f64 + 7.0 / 8.0;

            let mut p_points: Vec<C> = vec![];

            let p_int = PInterpolator::xp(p2, consts).goto_xp(p2, m);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_range as f64 + 0.999, m);
            // x_cuts.push(p_int2.x_path);
            // p_cuts.push(p_int2.p_path);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xp(p1, m);
            // x_cuts.push(p_int2.x_path);
            // p_cuts.push(p_int2.p_path);
            p_points.extend(p_int2.p_path);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m - 1.0)
                .goto_xp(p0, m - 1.0)
                .goto_xp(p0, m + 2.0)
                .goto_xp(p2, m + 2.0)
                .goto_xp(p2, m);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            // x_cuts.push(p_int2.x_path);
            // p_cuts.push(p_int2.p_path);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xp(p_range as f64 + 0.999, m);
            // x_cuts.push(p_int2.x_path);
            // p_cuts.push(p_int2.p_path);
            p_points.extend(p_int2.p_path);

            cut_p.push(p_points.iter().map(|p| p.conj()).collect());
            cut_p.push(p_points);

            let m = (p_range) as f64 * consts.k() as f64 + 2.0;
            let p1 = {
                let u_of_x = |x: C| -> C {
                    let s = consts.s();
                    x + 1.0 / x - (s - 1.0 / s) * x.ln()
                };
                let du_dx = |x: C| -> C {
                    let s = consts.s();
                    (x - s) * (x + 1.0 / s) / (x * x)
                };

                let x = nr::find_root(
                    |x| {
                        u_of_x(x)
                            - u_of_x(C::from(consts.s()))
                            - 2.0 * (m - 1.0) * C::i() / consts.h
                    },
                    du_dx,
                    C::new(0.0, 1.0),
                    1.0e-3,
                    10,
                );
                let x = x.unwrap();
                branch_points_x.push(x);
                x.arg() / std::f64::consts::PI + p_range as f64
            };

            cut_xp.push(XInterpolator::generate_xp(p_range as f64, p1, m, consts));
            cut_xm.push(XInterpolator::generate_xm(p_range as f64, p1, m, consts));

            let mut p_points = vec![];

            let p_int = PInterpolator::xp(p0, consts).goto_xp(p0, m);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p_range as f64 + 0.001, m);
            //x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xp(p1, m);
            // x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path);

            let p_int = PInterpolator::xp(p2, consts)
                .goto_xp(p2, m + 1.0)
                .goto_xp(p0, m + 1.0)
                .goto_xp(p0, m);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_xp(p1, m);
            //x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path.into_iter().rev());

            let p_int2 = p_int.clone().goto_xp(p_range as f64 + 0.001, m);
            //x_cuts.push(p_int2.x_path);
            p_points.extend(p_int2.p_path);

            cut_p.push(p_points.iter().map(|p| p.conj()).collect());
            cut_p.push(p_points);
        }

        if p_range < -1 {
            let p0 = p_range as f64 + 8.0 / 32.0;
            let p1 = p_range as f64 + 24.0 / 32.0;

            let p_int = PInterpolator::xp(p0, consts).goto_im(0.0);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_re(10.0).goto_re(100.0);
            // x.push(p_int2.x_path);
            cut_p.push(p_int2.p_path.iter().map(|p| p.conj()).collect());
            cut_p.push(p_int2.p_path);

            let p_int2 = p_int.goto_re(0.01);
            // x.push(p_int2.x_path);
            cut_p.push(p_int2.p_path.iter().map(|p| p.conj()).collect());
            cut_p.push(p_int2.p_path);

            let p_int = PInterpolator::xp(p1, consts).goto_im(0.0);

            let p_int = p_int.clear_path();

            let p_int2 = p_int.clone().goto_re(-10.0).goto_re(-100.0);
            // x.push(p_int2.x_path);
            cut_p.push(p_int2.p_path.iter().map(|p| p.conj()).collect());
            cut_p.push(p_int2.p_path);

            let p_int2 = p_int.goto_re(-0.01);
            // x.push(p_int2.x_path);
            cut_p.push(p_int2.p_path.iter().map(|p| p.conj()).collect());
            cut_p.push(p_int2.p_path);
        }

        let cut_u = vec![];
        let branch_points_p = vec![];
        let branch_points_u = vec![];

        Self {
            cut_p,
            cut_xp,
            cut_xm,
            cut_u,
            branch_points_p,
            branch_points_x,
            branch_points_u,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PxuPoint {
    pub p: C,
    pub xp: C,
    pub xm: C,
    pub u: C,
    pub consts: CouplingConstants,
    pub sheet_data: SheetData,
}

impl PxuPoint {
    pub fn new(p: impl Into<C>, consts: CouplingConstants) -> Self {
        let p = p.into();
        let log_branch = p.re.floor() as i32;
        let log_branch_sum = if log_branch.is_odd() { 1 } else { 0 };

        let sheet_data = SheetData {
            log_branch,
            log_branch_sum,
            e_branch: 1,
        };

        let xp = xp(p, 1.0, consts);
        let xm = xm(p, 1.0, consts);
        let u = u(p, consts, &sheet_data);
        Self {
            p,
            xp,
            xm,
            u,
            consts,
            sheet_data,
        }
    }

    pub fn set_coupling_constants(&mut self, consts: CouplingConstants) {
        self.consts = consts;
        self.set(self.p);
    }

    fn set(&mut self, p: C) {
        self.p = p;
        if self.sheet_data.e_branch > 0 {
            self.xp = xp(p, 1.0, self.consts);
            self.xm = xm(p, 1.0, self.consts);

            self.u = u(p, self.consts, &self.sheet_data);
        } else {
            self.xp = xp_crossed(p, 1.0, self.consts);
            self.xm = xm_crossed(p, 1.0, self.consts);
            self.u = u_crossed(p, self.consts, &self.sheet_data);

            log::info!(
                "{:2} {:2}",
                u(p, self.consts, &self.sheet_data,),
                u_crossed(p, self.consts, &self.sheet_data)
            );
        };
    }

    fn try_set(&mut self, p: Option<C>, sheet_data: SheetData) -> bool {
        let Some(p) = p else {return false};
        let new_xp: C;
        let new_xm: C;
        let new_u: C;

        if sheet_data.e_branch > 0 {
            new_xp = xp(p, 1.0, self.consts);
            new_xm = xm(p, 1.0, self.consts);
            new_u = u(p, self.consts, &self.sheet_data);
        } else {
            new_xp = xp_crossed(p, 1.0, self.consts);
            new_xm = xm_crossed(p, 1.0, self.consts);
            new_u = u_crossed(p, self.consts, &self.sheet_data);
        }

        if (self.p - p).norm_sqr() > 4.0 || (self.p - p).re.abs() > 0.5 {
            log::info!("p jump too large");
            return false;
        }

        if (self.xp - new_xp).norm_sqr() > 4.0 / (self.consts.h * self.consts.h) {
            log::info!("xp jump too large");
            return false;
        }

        if (self.xm - new_xm).norm_sqr() > 4.0 / (self.consts.h * self.consts.h) {
            log::info!("xm jump too large");
            return false;
        }

        if (self.u - new_u).norm_sqr() > 4.0 / (self.consts.h * self.consts.h) {
            log::info!("u jump too large");
            return false;
        }

        self.sheet_data = sheet_data;
        self.p = p;
        self.xp = new_xp;
        self.xm = new_xm;
        self.u = new_u;

        true
    }

    fn shift_xp(&self, new_xp: C, sheet_data: &SheetData, guess: C) -> Option<C> {
        if sheet_data.e_branch > 0 {
            nr::find_root(
                |p| xp(p, 1.0, self.consts) - new_xp,
                |p| dxp_dp(p, 1.0, self.consts),
                guess,
                1.0e-6,
                50,
            )
        } else {
            nr::find_root(
                |p| xp_crossed(p, 1.0, self.consts) - new_xp,
                |p| dxp_crossed_dp(p, 1.0, self.consts),
                guess,
                1.0e-6,
                50,
            )
        }
    }

    fn shift_xm(&self, new_xm: C, sheet_data: &SheetData, guess: C) -> Option<C> {
        if sheet_data.e_branch > 0 {
            nr::find_root(
                |p| xm(p, 1.0, self.consts) - new_xm,
                |p| dxm_dp(p, 1.0, self.consts),
                guess,
                1.0e-6,
                50,
            )
        } else {
            nr::find_root(
                |p| xm_crossed(p, 1.0, self.consts) - new_xm,
                |p| dxm_crossed_dp(p, 1.0, self.consts),
                guess,
                1.0e-6,
                50,
            )
        }
    }

    fn shift_u(&self, new_u: C, sheet_data: &SheetData, guess: C) -> Option<C> {
        if sheet_data.e_branch > 0 {
            nr::find_root(
                |p| u(p, self.consts, &self.sheet_data) - new_u,
                |p| du_dp(p, self.consts, &self.sheet_data),
                guess,
                1.0e-6,
                50,
            )
        } else {
            nr::find_root(
                |p| u_crossed(p, self.consts, &self.sheet_data) - new_u,
                |p| du_crossed_dp(p, self.consts, &self.sheet_data),
                guess,
                1.0e-6,
                50,
            )
        }
    }

    pub fn get(&self, component: Component) -> C {
        match component {
            Component::P => self.p,
            Component::U => self.u,
            Component::Xp => self.xp,
            Component::Xm => self.xm,
        }
    }

    pub fn update(&mut self, component: Component, new_value: C, crossed_cuts: &[&Cut]) {
        let mut new_sheet_data = self.sheet_data.clone();
        for cut in crossed_cuts {
            match cut.typ {
                CutType::LogX(Component::Xp, branch) => {
                    new_sheet_data.log_branch += branch;
                    new_sheet_data.log_branch_sum += branch;
                }
                CutType::LogX(Component::Xm, branch) => {
                    new_sheet_data.log_branch += branch;
                    new_sheet_data.log_branch_sum -= branch;
                }
                CutType::E => {
                    new_sheet_data.e_branch = -new_sheet_data.e_branch;
                }
                _ => {}
            }
            log::info!("Intersection with {:?}: {:?}", cut.typ, new_sheet_data);
        }

        for guess in vec![self.p, self.p - 0.01, self.p + 0.01] {
            let p = match component {
                Component::P => Some(new_value),
                Component::Xp => self.shift_xp(new_value, &new_sheet_data, guess),
                Component::Xm => self.shift_xm(new_value, &new_sheet_data, guess),
                Component::U => self.shift_u(new_value, &new_sheet_data, guess),
            };

            let prev_p = self.p;
            let crossed_cut = new_sheet_data != self.sheet_data;

            if self.try_set(p, new_sheet_data.clone()) {
                if crossed_cut {
                    log::info!("{:.2} {:.2}", self.p - prev_p, guess - prev_p);
                }
                break;
            }
        }
    }
}
