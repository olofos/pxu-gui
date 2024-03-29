use crate::contours::{Component, Contours};
use crate::interpolation::PInterpolatorMut;
use crate::kinematics::{xm_on_sheet, xp_on_sheet, CouplingConstants};
use crate::point::Point;
use num::complex::Complex64;

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct State {
    pub points: Vec<Point>,
    pub unlocked: bool,
}

impl State {
    pub fn new(m: usize, consts: CouplingConstants) -> Self {
        let mut points = vec![];

        let mut p_int = PInterpolatorMut::xp(0.025, consts);
        p_int
            .goto_m(m as f64)
            .goto_p(0.025 + 0.022 * (m - 1) as f64);
        let mut pt = Point::new(p_int.p(), consts);

        let s = consts.s();
        let us = s + 1.0 / s - (s - 1.0 / s) * s.ln();

        let u0 = us + 3.0;
        let step_size = 1.0 / 4.0;
        let max_steps = 2 * ((u0 - pt.u.re).abs() / step_size) as usize;
        for _ in 0..max_steps {
            let du = u0 - pt.u.re;
            let u = pt.u.re + du.abs().min(step_size).copysign(du);
            pt.update(Component::U, Complex64::new(u, pt.u.im), &[], consts);
            if (u0 - pt.u.re).abs() < 0.01 {
                break;
            }
        }
        if (u0 - pt.u.re).abs() >= 0.01 {
            log::warn!(
                "Could not find u (h={} k={} du={})",
                consts.h,
                consts.k(),
                u0 - pt.u.re
            );
        }
        points.push(pt);

        for i in 1..m {
            let mut pt = points[i - 1].clone();
            let xm = pt.xm;
            let steps = 4;
            for _ in 1..=steps {
                pt.update(Component::Xp, xm, &[], consts);
            }
            points.push(pt);
        }

        Self {
            points,
            unlocked: false,
        }
    }

    fn update_point(
        pt: &mut Point,
        component: Component,
        final_value: Complex64,
        contours: &Contours,
        consts: CouplingConstants,
    ) -> bool {
        loop {
            let current_value = pt.get(component);

            let crossings = contours.get_crossed_cuts(pt, component, final_value, consts);

            let next_value = if crossings.len() > 1 {
                let t = (crossings[0].0 + crossings[1].0) / 2.0;
                current_value + t * (final_value - current_value)
            } else {
                final_value
            };

            if crossings.is_empty() {
                if !pt.update(component, next_value, &[], consts) {
                    return false;
                }
            } else if !pt.update(component, next_value, &crossings[0].1, consts) {
                return false;
            }

            if next_value == final_value {
                return true;
            }
        }
    }

    pub fn update_points(
        &mut self,
        active_point: usize,
        component: Component,
        new_value: Complex64,
        contours: &Contours,
        consts: CouplingConstants,
    ) -> bool {
        let mut result = true;

        result &= Self::update_point(
            &mut self.points[active_point],
            component,
            new_value,
            contours,
            consts,
        );

        if !self.unlocked {
            for i in (active_point + 1)..self.points.len() {
                let new_value = xm_on_sheet(
                    self.points[i - 1].p,
                    1.0,
                    consts,
                    &self.points[i - 1].sheet_data,
                );
                result &= Self::update_point(
                    &mut self.points[i],
                    Component::Xp,
                    new_value,
                    contours,
                    consts,
                );
            }

            for i in (0..active_point).rev() {
                let new_value: num::Complex<f64> = xp_on_sheet(
                    self.points[i + 1].p,
                    1.0,
                    consts,
                    &self.points[i + 1].sheet_data,
                );
                result &= Self::update_point(
                    &mut self.points[i],
                    Component::Xm,
                    new_value,
                    contours,
                    consts,
                );
            }
        }
        result
    }

    pub fn update(
        &mut self,
        active_point: usize,
        component: Component,
        new_value: Complex64,
        contours: &Contours,
        consts: CouplingConstants,
    ) -> bool {
        self.update_points(active_point, component, new_value, contours, consts)
    }

    pub fn p(&self) -> Complex64 {
        self.points.iter().map(|pxu| pxu.p).sum::<Complex64>()
    }

    pub fn en(&self, consts: CouplingConstants) -> Complex64 {
        self.points
            .iter()
            .map(|pt| pt.en(consts))
            .sum::<Complex64>()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SavedState {
    pub consts: CouplingConstants,
    pub state: State,
}

impl SavedState {
    pub fn decode(input: &str) -> Option<Self> {
        use base64::Engine;
        use std::io::Write;

        let input = input.trim();

        if let Ok(saved_state) = ron::from_str(input) {
            return Some(saved_state);
        }
        log::info!("Could not decode RON, trying JSON");
        if let Ok(saved_state) = serde_json::from_str(input) {
            return Some(saved_state);
        }
        log::info!("Could not decode JSON, trying base64");

        let Ok(data) = base64::engine::general_purpose::URL_SAFE.decode(input) else {
            log::warn!("Could not decode base64");
            return None;
        };

        let mut dec = flate2::write::DeflateDecoder::new(Vec::new());
        let Ok(()) = dec.write_all(&data[..]) else {
            log::warn!("Could not deflate");
            return None;
        };
        let Ok(data) = dec.finish() else {
            log::warn!("Could not deflate");
            return None;
        };
        let Ok(input) = String::from_utf8(data) else {
            log::warn!("Resulting data is not a string");
            return None;
        };
        if let Ok(saved_state) = ron::from_str::<SavedState>(&input) {
            return Some(saved_state);
        }
        log::warn!("Could not decode RON");
        None
    }
}
