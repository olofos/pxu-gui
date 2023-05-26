use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Value<T> {
    Const(T),
    Transition(T, T, f64),
}

fn ease(s: f64) -> f64 {
    s * s * (3.0 - 2.0 * s)
}

pub trait Interpolate {
    fn lerp(&self, other: &Self, s: f64) -> Self;
}

impl Interpolate for f32 {
    fn lerp(&self, other: &Self, s: f64) -> Self {
        let s = s as f32;
        (1.0 - s) * self + s * other
    }
}

impl Interpolate for f64 {
    fn lerp(&self, other: &Self, s: f64) -> Self {
        (1.0 - s) * self + s * other
    }
}

impl Interpolate for num::complex::Complex64 {
    fn lerp(&self, other: &Self, s: f64) -> Self {
        (1.0 - s) * self + s * other
    }
}

impl Interpolate for [f32; 2] {
    fn lerp(&self, other: &Self, s: f64) -> Self {
        [self[0].lerp(&other[0], s), self[1].lerp(&other[1], s)]
    }
}

impl Interpolate for [[f32; 2]; 2] {
    fn lerp(&self, other: &Self, s: f64) -> Self {
        [self[0].lerp(&other[0], s), self[1].lerp(&other[1], s)]
    }
}

impl<T> Value<T>
where
    T: Interpolate + Clone,
{
    pub fn get(&self, t: f64) -> T {
        match self {
            Self::Const(v) => v.clone(),
            Self::Transition(start, end, duration) => {
                // let s = (t / duration).clamp(0.0, 1.0) as f32;
                let s = (t / duration).rem_euclid(2.0);
                let s = if s > 1.0 { 2.0 - s } else { s };
                start.lerp(end, ease(s))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum RelativisticComponent {
    P,
    Theta,
}

impl std::str::FromStr for RelativisticComponent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "P" => Ok(Self::P),
            "Theta" => Ok(Self::Theta),
            _ => Err("Could not parse component".to_owned()),
        }
    }
}

impl std::fmt::Display for RelativisticComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::P => "P",
                Self::Theta => "Theta",
            },
        )
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PlotDescription {
    pub rect: [[f32; 2]; 2],
    pub origin: Option<[f32; 2]>,
    pub height: Option<Value<f32>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
// #[serde(default)]
pub struct RelativisticPlotDescription {
    pub rect: Value<[[f32; 2]; 2]>,
    pub m: Value<f32>,
    pub point: Option<Value<[f32; 2]>>,
    pub height: Option<Value<f32>>,
}

use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FrameDescription {
    pub image: String,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub plot: HashMap<pxu::Component, PlotDescription>,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub relativistic_plot: HashMap<RelativisticComponent, RelativisticPlotDescription>,
    pub duration: Option<f64>,
    pub consts: Option<[f64; 2]>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PresentationDescription {
    pub frame: Vec<FrameDescription>,
}
