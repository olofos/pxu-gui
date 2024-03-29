#![warn(clippy::all, rust_2018_idioms)]

mod contours;
mod cut;
pub mod interpolation;
pub mod kinematics;
mod nr;
pub mod path;
mod point;
mod state;

pub use contours::{
    compute_branch_point, BranchPointType, Component, Contours, GridLine, GridLineComponent,
};
pub use cut::{Cut, CutType};
pub use kinematics::CouplingConstants;
pub use path::Path;
pub use point::Point;
pub use state::SavedState;
pub use state::State;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Pxu {
    pub consts: CouplingConstants,
    #[serde(skip)]
    pub contours: Contours,
    pub state: State,
    #[serde(skip)]
    pub paths: Vec<Path>,
}

impl Pxu {
    pub fn new(consts: CouplingConstants) -> Self {
        Self {
            consts,
            contours: Default::default(),
            state: Default::default(),
            paths: Default::default(),
        }
    }

    pub fn get_path_by_name(&self, name: &str) -> Option<&Path> {
        self.paths.iter().find(|path| path.name == name)
    }
}
