use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::io::Result;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use crate::paths::error;
use crate::ContourProvider;

#[derive(Default)]
pub struct PathProvider {
    paths: HashMap<String, Arc<pxu::Path>>,
    starts: HashMap<String, Arc<pxu::State>>,
    seen_paths: Arc<Mutex<HashSet<String>>>,
}

impl PathProvider {
    pub fn add(&mut self, name: &str, path: pxu::Path, start: pxu::State) {
        self.paths.insert(name.to_owned(), Arc::new(path));
        self.starts.insert(name.to_owned(), Arc::new(start));
    }

    pub fn get_path(&self, name: &str) -> Result<Arc<pxu::Path>> {
        self.seen_paths.lock().unwrap().insert(name.to_owned());

        self.paths
            .get(name)
            .cloned()
            .ok_or_else(|| error(&format!("Could not find path for {name}")))
    }

    pub fn get_start(&self, name: &str) -> Result<Arc<pxu::State>> {
        self.seen_paths.lock().unwrap().insert(name.to_owned());

        self.starts
            .get(name)
            .cloned()
            .ok_or_else(|| error(&format!("Could not find start for {name}")))
    }

    pub fn get_statistics(&self) -> String {
        let unused_paths = {
            let seen_paths = &self.seen_paths.lock().unwrap();

            self.paths
                .keys()
                .filter(|k| !seen_paths.contains(*k))
                .collect::<Vec<_>>()
        };

        let mut lines: Vec<String> = vec![];

        if unused_paths.is_empty() {
            lines.push("All paths were used.".into());
        } else {
            lines.push("The following paths were not used:".into());
            for p in unused_paths {
                lines.push(format!("- \"{p}\""));
            }
        }

        lines.join("\n")
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

                let saved_path: pxu::path::SavedPath = path_func(contour_provider.clone());
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
