use crate::arguments::Arguments;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct UiState {
    pub plot_state: plot::PlotState,
    #[serde(skip)]
    pub hide_side_panel: bool,
    #[serde(skip)]
    pub show_fps: bool,
    #[serde(skip)]
    pub show_dev: bool,
    #[serde(skip)]
    pub continuous_mode: bool,
    #[serde(skip)]
    pub saved_paths_to_load: Option<Vec<pxu::path::SavedPath>>,
    #[serde(skip)]
    pub path_load_progress: Option<(usize, usize)>,
    #[serde(skip)]
    pub inital_saved_state: Option<pxu::SavedState>,
}

impl UiState {
    pub fn set(&mut self, arguments: Arguments) {
        self.show_fps = arguments.show_fps;
        self.show_dev = arguments.show_dev;
        self.continuous_mode = arguments.continuous_mode;

        if let Some(ref paths) = arguments.paths {
            let mut saved_paths_to_load = pxu::path::SavedPath::load(paths);
            if let Some(ref mut paths) = saved_paths_to_load {
                self.path_load_progress = Some((0, paths.len()));
                paths.reverse();
            }
            self.saved_paths_to_load = saved_paths_to_load
        }

        if let Some(ref s) = arguments.state {
            self.inital_saved_state = pxu::SavedState::decode(s);
        }
    }
}
