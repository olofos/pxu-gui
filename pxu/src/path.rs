use num::complex::Complex64;

use crate::kinematics::SheetData;
use crate::Component;
use crate::State;

pub struct PathSegment {
    pub p: Vec<Vec<Complex64>>,
    pub xp: Vec<Vec<Complex64>>,
    pub xm: Vec<Vec<Complex64>>,
    pub u: Vec<Vec<Complex64>>,
    pub sheet_data: SheetData,
}

#[derive(Default)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

pub struct EditablePath {
    pub states: Vec<State>,
    pub component: Component,
}

impl Default for EditablePath {
    fn default() -> Self {
        Self {
            states: Default::default(),
            component: Component::P,
        }
    }
}

impl EditablePath {
    pub fn clear(&mut self) {
        self.states = vec![];
    }

    pub fn get(&self, component: Component) -> Vec<Vec<Complex64>> {
        if self.states.is_empty() {
            return vec![];
        }

        let mut result = vec![vec![]; self.states[0].points.len()];

        for state in self.states.iter() {
            for (i, point) in state.points.iter().enumerate() {
                result[i].push(point.get(component));
            }
        }

        result
    }

    pub fn push(&mut self, state: &State) {
        self.states.push(state.clone());
    }
}
