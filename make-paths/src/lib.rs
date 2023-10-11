mod paths;

pub fn get_plot_paths(
    contours: &pxu::Contours,
    consts: pxu::CouplingConstants,
) -> Vec<pxu::path::SavedPath> {
    paths::PLOT_PATHS
        .iter()
        .map(|f| f(contours, consts))
        .collect::<Vec<_>>()
}

pub fn get_interactive_paths(
    contours: &pxu::Contours,
    consts: pxu::CouplingConstants,
) -> Vec<pxu::path::SavedPath> {
    paths::INTERACTIVE_PATHS
        .iter()
        .map(|f| f(contours, consts))
        .collect::<Vec<_>>()
}

pub use paths::INTERACTIVE_PATHS;
pub use paths::PLOT_PATHS;
