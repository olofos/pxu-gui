use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use pxu::CouplingConstants;
use std::io::Result;
use std::{collections::HashMap, sync::Arc};

fn error(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, message)
}
struct LossyHashCouplingConstants {
    consts: CouplingConstants,
}

impl LossyHashCouplingConstants {
    fn string_rep(&self) -> String {
        format!("{:.3} {}", self.consts.h, self.consts.k())
    }
}

impl std::hash::Hash for LossyHashCouplingConstants {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.string_rep().hash(state);
    }
}

impl std::cmp::Eq for LossyHashCouplingConstants {}

impl std::cmp::PartialEq for LossyHashCouplingConstants {
    fn eq(&self, other: &Self) -> bool {
        self.string_rep() == other.string_rep()
    }
}

impl From<CouplingConstants> for LossyHashCouplingConstants {
    fn from(consts: CouplingConstants) -> Self {
        Self { consts }
    }
}

#[derive(Default)]
pub struct PxuProvider {
    contours: Arc<ContourProvider>,
    paths: Arc<PathProvider>,
}

#[derive(Default)]
pub struct ContourProvider {
    contours: HashMap<LossyHashCouplingConstants, Arc<pxu::Contours>>,
}

#[derive(Default)]
pub struct PathProvider {
    paths: HashMap<String, Arc<pxu::Path>>,
    starts: HashMap<String, Arc<pxu::State>>,
}

impl ContourProvider {
    pub fn add(&mut self, consts: pxu::CouplingConstants, contours: pxu::Contours) {
        self.contours.insert(consts.into(), Arc::new(contours));
    }

    pub fn get(&self, consts: pxu::CouplingConstants) -> Result<Arc<pxu::Contours>> {
        self.contours
            .get(&consts.into())
            .cloned()
            .ok_or_else(|| error(&format!("Could not find contour for {consts:?}")))
    }
}

impl PathProvider {
    pub fn add(&mut self, name: &str, path: pxu::Path, start: pxu::State) {
        self.paths.insert(name.to_owned(), Arc::new(path));
        self.starts.insert(name.to_owned(), Arc::new(start));
    }

    pub fn get_path(&self, name: &str) -> Result<Arc<pxu::Path>> {
        self.paths
            .get(name)
            .cloned()
            .ok_or_else(|| error(&format!("Could not find path for {name}")))
    }

    pub fn get_start(&self, name: &str) -> Result<Arc<pxu::State>> {
        self.starts
            .get(name)
            .cloned()
            .ok_or_else(|| error(&format!("Could not find start for {name}")))
    }
}

impl PxuProvider {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_contours(&mut self, consts: pxu::CouplingConstants, contours: pxu::Contours) {
        Arc::get_mut(&mut self.contours)
            .unwrap()
            .add(consts, contours)
    }

    pub fn get_contours(&self, consts: pxu::CouplingConstants) -> Result<Arc<pxu::Contours>> {
        self.contours.get(consts)
    }

    pub fn add_path(&mut self, name: &str, path: pxu::Path, start: pxu::State) {
        Arc::get_mut(&mut self.paths)
            .unwrap()
            .add(name, path, start);
    }

    pub fn get_path(&self, name: &str) -> Result<Arc<pxu::Path>> {
        self.paths.get_path(name)
    }

    pub fn get_start(&self, name: &str) -> Result<Arc<pxu::State>> {
        self.paths.get_start(name)
    }

    pub fn generate_contours(
        &mut self,
        consts_list: Vec<CouplingConstants>,
        verbose: bool,
        pool: &threadpool::ThreadPool,
        spinner_style: &ProgressStyle,
    ) {
        Arc::get_mut(&mut self.contours).unwrap().generate(
            consts_list,
            verbose,
            pool,
            spinner_style,
        );
    }

    pub fn load_paths(
        &mut self,
        paths: &[crate::PathFunction],
        verbose: bool,
        pool: &threadpool::ThreadPool,
        spinner_style: &ProgressStyle,
        spinner_style_no_progress: &ProgressStyle,
    ) {
        Arc::get_mut(&mut self.paths).unwrap().load(
            paths,
            self.contours.clone(),
            verbose,
            pool,
            spinner_style,
            spinner_style_no_progress,
        );
    }
}

impl ContourProvider {
    pub fn generate(
        &mut self,
        consts_list: Vec<CouplingConstants>,
        verbose: bool,
        pool: &threadpool::ThreadPool,
        spinner_style: &ProgressStyle,
    ) {
        let consts_list_len = consts_list.len();

        let mb = Arc::new(MultiProgress::new());
        let pb = if !verbose {
            mb.add(ProgressBar::new(1))
        } else {
            ProgressBar::hidden()
        };

        pb.set_style(spinner_style.clone());
        pb.set_length(consts_list_len as u64);

        let (tx, rx) = std::sync::mpsc::channel();

        for consts in consts_list {
            let mb = mb.clone();
            let spinner_style = spinner_style.clone();
            let tx = tx.clone();
            let verbose = !verbose;

            pool.execute(move || {
                let pb = if verbose {
                    mb.add(ProgressBar::new(1))
                } else {
                    ProgressBar::hidden()
                };
                pb.set_style(spinner_style.clone());
                pb.enable_steady_tick(std::time::Duration::from_millis(100));
                pb.set_message(format!("h={:.2} k={}", consts.h, consts.k()));

                let mut contours = pxu::Contours::new();

                loop {
                    pb.set_length(contours.progress().1 as u64);
                    pb.set_position(contours.progress().0 as u64);
                    if contours.update(0, consts) {
                        tx.send((consts, contours)).unwrap();
                        pb.finish_and_clear();
                        break;
                    }
                }
            });
        }

        rx.into_iter()
            .take(consts_list_len)
            .for_each(|(consts, contours)| {
                self.add(consts, contours);
                pb.inc(1);
            });

        pool.join();
        pb.finish_and_clear();
    }
}

impl PathProvider {
    pub fn load(
        &mut self,
        paths: &[crate::PathFunction],
        contour_provider: Arc<ContourProvider>,
        verbose: bool,
        pool: &threadpool::ThreadPool,
        spinner_style: &ProgressStyle,
        spinner_style_no_progress: &ProgressStyle,
    ) {
        let mb = Arc::new(MultiProgress::new());
        let pb = if !verbose {
            mb.add(ProgressBar::new(1))
        } else {
            ProgressBar::hidden()
        };

        pb.set_style(spinner_style.clone());
        pb.set_length(paths.len() as u64);

        let (tx, rx) = std::sync::mpsc::channel();

        for path_func in paths {
            let tx = tx.clone();
            let spinner_style = spinner_style_no_progress.clone();
            let mb = mb.clone();
            let path_func = *path_func;
            let contour_provider = contour_provider.clone();

            pool.execute(move || {
                let pb = if !verbose {
                    mb.add(ProgressBar::new(1))
                } else {
                    ProgressBar::hidden()
                };
                pb.set_style(spinner_style);
                pb.enable_steady_tick(std::time::Duration::from_millis(100));

                pb.set_message("Generating path");

                let saved_path = path_func(contour_provider.clone());
                let start = saved_path.start.clone();
                let consts = saved_path.consts;

                pb.set_message(saved_path.name.clone());
                pb.tick();

                let path = pxu::path::Path::from_base_path(
                    saved_path.into(),
                    &contour_provider.get(consts).unwrap(),
                    consts,
                );
                tx.send((path, start)).unwrap();
                pb.finish_and_clear();
            });
        }

        let paths_and_starts = rx
            .into_iter()
            .take(paths.len())
            .map(|r: (pxu::Path, pxu::State)| {
                pb.inc(1);
                r
            })
            .collect::<Vec<_>>();

        pool.join();
        pb.finish_and_clear();

        for (path, start) in paths_and_starts.iter() {
            self.add(&path.name, path.clone(), start.clone());
        }
    }
}
