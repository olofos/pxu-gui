mod path_provider;
mod paths;
mod provider;

pub use provider::ContourProvider;
pub use provider::PxuProvider;

pub type PathFunction = fn(std::sync::Arc<ContourProvider>) -> pxu::path::SavedPath;
pub use paths::INTERACTIVE_PATHS;
pub use paths::PLOT_PATHS;
