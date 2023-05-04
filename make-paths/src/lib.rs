mod paths;

pub fn get_plot_paths(
    contours: &pxu::Contours,
    consts: pxu::CouplingConstants,
) -> Vec<pxu::path::SavedPath> {
    paths::ALL_PATHS
        .iter()
        .map(|f| f(&contours, consts))
        .collect::<Vec<_>>()
}

pub fn get_interactive_paths(
    contours: &pxu::Contours,
    consts: pxu::CouplingConstants,
) -> Vec<pxu::path::SavedPath> {
    paths::ALL_PATHS
        .iter()
        .map(|f| f(&contours, consts))
        .collect::<Vec<_>>()
}
