use crate::contours::{Component, UCutType};
use crate::cut::{Cut, CutType};
use crate::kinematics::{
    du_dp, du_dx, dxm_dp_on_sheet, dxp_dp_on_sheet, u, u_of_x, xm, xm_on_sheet, xp, xp_on_sheet,
    CouplingConstants, SheetData, UBranch,
};
use crate::nr;
use num::complex::Complex64;

fn _c_zero() -> Complex64 {
    Complex64::from(0.0)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Point {
    pub p: Complex64,
    pub xp: Complex64,
    pub xm: Complex64,
    pub u: Complex64,
    #[serde(default = "_c_zero")]
    pub x: Complex64,
    pub sheet_data: SheetData,
}

impl Point {
    pub fn new(p: impl Into<Complex64>, consts: CouplingConstants) -> Self {
        let p: Complex64 = p.into();
        let log_branch_p: i32 = 0;
        let log_branch_m = p.re.floor() as i32;
        let log_branch_x: i32 = 0;
        let u_branch = if log_branch_m >= 0 {
            (UBranch::Outside, UBranch::Outside)
        } else if log_branch_m == -1 {
            (UBranch::Between, UBranch::Between)
        } else {
            (UBranch::Inside, UBranch::Inside)
        };

        let sheet_data = SheetData {
            log_branch_p,
            log_branch_m,
            log_branch_x,
            e_branch: 1,
            u_branch,
            im_x_sign: (1, 1),
        };

        let xp = xp(p, 1.0, consts);
        let xm = xm(p, 1.0, consts);
        let u = u(p, consts, &sheet_data);
        let x = Self::find_x(u, (xp + xm) / 2.0, consts).unwrap();
        Self {
            p,
            xp,
            xm,
            u,
            x,
            sheet_data,
        }
    }

    fn find_x(u: Complex64, guess: Complex64, consts: CouplingConstants) -> Option<Complex64> {
        nr::find_root(
            |x| u_of_x(x, consts) - u,
            |x| du_dx(x, consts),
            guess,
            1.0e-5,
            50,
        )
    }

    fn shifted(
        &self,
        p: Option<Complex64>,
        sheet_data: &SheetData,
        consts: CouplingConstants,
    ) -> Option<Self> {
        let Some(p) = p else {return None};
        let new_xp = xp_on_sheet(p, 1.0, consts, sheet_data);
        let new_xm = xm_on_sheet(p, 1.0, consts, sheet_data);
        let new_u = u(p, consts, sheet_data);

        if (self.p - p).re.abs() > 0.125 || (self.p - p).im.abs() > 0.25 {
            log::debug!(
                "p jump too large {} {}",
                (self.p - p).norm_sqr(),
                (self.p - p).re.abs()
            );
            return None;
        }

        if (p - p.re.round()).norm() < 0.005 {
            log::debug!("Too close to the origin");
            return None;
        }

        if (self.xp - new_xp).norm_sqr() > 16.0 / (consts.h * consts.h) {
            log::debug!(
                "xp jump too large: {} ({}) {} ({})",
                (self.xp - new_xp).norm_sqr(),
                (self.xp - new_xp).norm_sqr() * (consts.h * consts.h),
                self.xp.norm_sqr(),
                self.xp.norm_sqr() * (consts.h * consts.h)
            );
            // return None;
        }

        if (self.xm - new_xm).norm_sqr() > 16.0 / (consts.h * consts.h) {
            log::debug!(
                "xm jump too large: {} ({}) {} ({})",
                (self.xm - new_xm).norm_sqr(),
                (self.xm - new_xm).norm_sqr() * (consts.h * consts.h),
                self.xm.norm_sqr(),
                self.xm.norm_sqr() * (consts.h * consts.h)
            );

            // return None;
        }

        if (self.u - new_u).norm_sqr() > 16.0 / (consts.h * consts.h) {
            log::debug!("u jump too large");
            // return None;
        }

        let mut sheet_data = sheet_data.clone();
        let xp = new_xp;
        let xm = new_xm;
        let u = new_u;

        let mut x = Self::find_x(
            u + Complex64::new(
                0.0,
                -2.0 * sheet_data.log_branch_x as f64 * consts.k() as f64 / consts.h,
            ),
            self.x,
            consts,
        )
        .unwrap_or(self.x);

        if x.re < 0.0 && self.x.im * x.im < 0.0 {
            if x.im > 0.0 {
                sheet_data.log_branch_x += 1;
            } else {
                sheet_data.log_branch_x -= 1;
            }
            x = Self::find_x(
                u + Complex64::new(
                    0.0,
                    -2.0 * sheet_data.log_branch_x as f64 * consts.k() as f64 / consts.h,
                ),
                self.x.conj(),
                consts,
            )
            .unwrap_or(self.x);
        }

        Some(Self {
            p,
            xp,
            xm,
            u,
            x,
            sheet_data,
        })
    }

    fn shift_xp(
        &self,
        new_xp: Complex64,
        sheet_data: &SheetData,
        guess: Complex64,
        consts: CouplingConstants,
    ) -> Option<Complex64> {
        nr::find_root(
            |p| xp_on_sheet(p, 1.0, consts, sheet_data) - new_xp,
            |p| dxp_dp_on_sheet(p, 1.0, consts, sheet_data),
            guess,
            1.0e-6,
            50,
        )
    }

    fn shift_xm(
        &self,
        new_xm: Complex64,
        sheet_data: &SheetData,
        guess: Complex64,
        consts: CouplingConstants,
    ) -> Option<Complex64> {
        nr::find_root(
            |p| xm_on_sheet(p, 1.0, consts, sheet_data) - new_xm,
            |p| dxm_dp_on_sheet(p, 1.0, consts, sheet_data),
            guess,
            1.0e-6,
            50,
        )
    }

    fn shift_u(
        &self,
        new_u: Complex64,
        sheet_data: &SheetData,
        guess: Complex64,
        consts: CouplingConstants,
    ) -> Option<Complex64> {
        nr::find_root(
            |p| u(p, consts, sheet_data) - new_u,
            |p| du_dp(p, consts, sheet_data),
            guess,
            1.0e-6,
            50,
        )
    }

    pub fn get(&self, component: Component) -> Complex64 {
        match component {
            Component::P => self.p,
            Component::U => self.u,
            Component::Xp => self.xp,
            Component::Xm => self.xm,
            Component::X => self.x,
        }
    }

    pub fn update(
        &mut self,
        component: Component,
        new_value: Complex64,
        crossed_cuts: &[&Cut],
        consts: CouplingConstants,
    ) -> bool {
        let mut new_sheet_data = self.sheet_data.clone();
        for cut in crossed_cuts {
            match cut.typ {
                CutType::E => {
                    new_sheet_data.e_branch = -new_sheet_data.e_branch;
                }
                CutType::UShortScallion(Component::Xp) => {
                    new_sheet_data.u_branch = (
                        new_sheet_data.u_branch.0.cross_scallion(),
                        new_sheet_data.u_branch.1,
                    );
                }
                CutType::UShortScallion(Component::Xm) => {
                    new_sheet_data.u_branch = (
                        new_sheet_data.u_branch.0,
                        new_sheet_data.u_branch.1.cross_scallion(),
                    );
                }
                CutType::UShortKidney(Component::Xp) => {
                    new_sheet_data.u_branch = (
                        new_sheet_data.u_branch.0.cross_kidney(),
                        new_sheet_data.u_branch.1,
                    );
                }
                CutType::UShortKidney(Component::Xm) => {
                    new_sheet_data.u_branch = (
                        new_sheet_data.u_branch.0,
                        new_sheet_data.u_branch.1.cross_kidney(),
                    );
                }
                CutType::Log(Component::Xp) => {
                    if self.xp.im >= 0.0 {
                        new_sheet_data.log_branch_p += 1;
                    } else {
                        new_sheet_data.log_branch_p -= 1;
                    }
                }
                CutType::Log(Component::Xm) => {
                    if self.xm.im <= 0.0 {
                        new_sheet_data.log_branch_m += 1;
                    } else {
                        new_sheet_data.log_branch_m -= 1;
                    }
                }
                CutType::ULongPositive(Component::Xp) => {
                    new_sheet_data.im_x_sign.0 = -new_sheet_data.im_x_sign.0;
                }
                CutType::ULongPositive(Component::Xm) => {
                    new_sheet_data.im_x_sign.1 = -new_sheet_data.im_x_sign.1;
                }
                _ => {}
            }
            log::debug!("Intersection with {:?}: {:?}", cut.typ, new_sheet_data);
        }

        if let Some(pt) = [self.p, self.p - 0.05, self.p + 0.05]
            .into_iter()
            .filter_map(|guess| {
                let p = match component {
                    Component::P => Some(new_value),
                    Component::Xp => self.shift_xp(new_value, &new_sheet_data, guess, consts),
                    Component::Xm => self.shift_xm(new_value, &new_sheet_data, guess, consts),
                    Component::U => self.shift_u(new_value, &new_sheet_data, guess, consts),
                    Component::X => {
                        todo!("add shift_x")
                    }
                };

                self.shifted(p, &new_sheet_data, consts)
            })
            .min_by_key(|pt| {
                (((pt.xp - self.xp).norm_sqr() + (pt.xm - self.xm).norm_sqr()) * 10000.0).round()
                    as i32
            })
        {
            *self = pt;
            true
        } else {
            false
        }
    }

    pub fn same_sheet(&self, other: &Point, component: Component, u_cut_type: UCutType) -> bool {
        let sd1 = &self.sheet_data;
        let sd2 = &other.sheet_data;
        sd1.is_same(sd2, component, u_cut_type)
    }

    pub fn en(&self, consts: CouplingConstants) -> Complex64 {
        -Complex64::i() * consts.h / 2.0 * (self.xp - 1.0 / self.xp - self.xm + 1.0 / self.xm)
    }
}

impl SheetData {
    pub fn is_same(&self, other: &SheetData, component: Component, u_cut_type: UCutType) -> bool {
        let sd1 = self;
        let sd2 = other;

        match component {
            Component::P => sd1.e_branch == sd2.e_branch,
            Component::U => {
                if u_cut_type == UCutType::Short
                    && sd1.u_branch == sd2.u_branch
                    && (sd1.u_branch.0 == UBranch::Between || sd1.u_branch.1 == UBranch::Between)
                {
                    true
                } else if (sd1.log_branch_p + sd1.log_branch_m)
                    != (sd2.log_branch_p + sd2.log_branch_m)
                    || (sd1.log_branch_p - sd1.log_branch_m)
                        != (sd2.log_branch_p - sd2.log_branch_m)
                {
                    false
                } else if u_cut_type == UCutType::Long {
                    self.im_x_sign.0.signum() == other.im_x_sign.0.signum()
                        && self.im_x_sign.1.signum() == other.im_x_sign.1.signum()
                } else {
                    sd1.u_branch == sd2.u_branch
                }
            }
            Component::Xp => {
                if u_cut_type == UCutType::Short {
                    if sd1.u_branch.1 == UBranch::Between && sd2.u_branch.1 == UBranch::Between {
                        true
                    } else if sd1.u_branch.1 == sd2.u_branch.1
                        && (sd1.u_branch.0 == UBranch::Between
                            || sd2.u_branch.0 == UBranch::Between)
                    {
                        sd1.log_branch_p == sd2.log_branch_p
                    } else if (sd1.log_branch_p + sd1.log_branch_m)
                        != (sd2.log_branch_p + sd2.log_branch_m)
                    {
                        false
                    } else {
                        sd1.u_branch.1 == sd2.u_branch.1
                    }
                } else if (sd1.log_branch_p + sd1.log_branch_m)
                    != (sd2.log_branch_p + sd2.log_branch_m)
                {
                    false
                } else {
                    self.im_x_sign.1.signum() == other.im_x_sign.1.signum()
                }
            }
            Component::Xm => {
                if u_cut_type == UCutType::Short {
                    if sd1.u_branch.0 == UBranch::Between && sd2.u_branch.0 == UBranch::Between {
                        true
                    } else if sd1.u_branch.0 == sd2.u_branch.0
                        && (sd1.u_branch.1 == UBranch::Between
                            || sd2.u_branch.1 == UBranch::Between)
                    {
                        sd1.log_branch_m == sd2.log_branch_m
                    } else if (sd1.log_branch_p + sd1.log_branch_m)
                        != (sd2.log_branch_p + sd2.log_branch_m)
                    {
                        false
                    } else {
                        sd1.u_branch.0 == sd2.u_branch.0
                    }
                } else if (sd1.log_branch_p + sd1.log_branch_m)
                    != (sd2.log_branch_p + sd2.log_branch_m)
                {
                    false
                } else {
                    self.im_x_sign.0.signum() == other.im_x_sign.0.signum()
                }
            }
            Component::X => {
                todo!("figure out x branches")
            }
        }
    }
}
