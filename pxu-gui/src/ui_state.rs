use crate::arguments::Arguments;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UiState {
    pub plot_state: crate::plot::PlotState,
    #[serde(skip)]
    pub show_side_panel: bool,
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
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            plot_state: Default::default(),
            show_side_panel: true,
            show_fps: false,
            show_dev: false,
            continuous_mode: false,
            saved_paths_to_load: None,
            path_load_progress: None,
        }
    }
}

impl UiState {
    pub fn set(&mut self, arguments: Arguments) {
        self.show_fps = arguments.show_fps;
        self.show_dev = arguments.show_dev;
        self.continuous_mode = arguments.continuous_mode;
        self.plot_state.show_x = arguments.show_x;

        if let Some(ref paths) = arguments.paths {
            let mut saved_paths_to_load = pxu::path::SavedPath::load(paths);
            if let Some(ref mut paths) = saved_paths_to_load {
                self.path_load_progress = Some((0, paths.len()));
                paths.reverse();
            }
            self.saved_paths_to_load = saved_paths_to_load
        }
    }
}
