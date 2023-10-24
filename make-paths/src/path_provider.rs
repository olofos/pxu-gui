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

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    path_string: String,
    saved_path_string: String,
}

const CACHE_FILENAME: &str = "path-cache";

fn load_cache(dirname: &str) -> Result<HashMap<String, CacheEntry>> {
    let path = std::path::PathBuf::from(dirname).join(CACHE_FILENAME);
    let bytes = std::fs::read(path)?;
    let s = std::str::from_utf8(&bytes).map_err(|err| error(&format!("{err}")))?;
    ron::from_str(s).map_err(|err| error(&format!("{err}")))
}

fn save_cache(cache: HashMap<String, CacheEntry>, dirname: &str) -> Result<()> {
    let s = ron::to_string(&cache).map_err(|err| error(&format!("{err}")))?;
    let path = std::path::PathBuf::from(dirname).join(CACHE_FILENAME);
    std::fs::write(path, s)
}

#[allow(clippy::too_many_arguments)]
impl PathProvider {
    pub fn load(
        &mut self,
        paths: &[crate::PathFunction],
        contour_provider: Arc<ContourProvider>,
        verbose: bool,
        pool: &threadpool::ThreadPool,
        cache_dirname: &str,
        spinner_style: &ProgressStyle,
        spinner_style_no_progress: &ProgressStyle,
    ) {
        let cache = match load_cache(cache_dirname) {
            Ok(cache) => cache,
            Err(err) => {
                if verbose {
                    eprintln!("Error loading cache: {err}");
                }
                Default::default()
            }
        };

        let mb = Arc::new(MultiProgress::new());
        let pb = if !verbose {
            mb.add(ProgressBar::new(1))
        } else {
            ProgressBar::hidden()
        };

        pb.set_style(spinner_style.clone());
        pb.set_length(paths.len() as u64);

        let (tx, rx) = std::sync::mpsc::channel();
        let cache = Arc::new(cache);

        for path_func in paths {
            let tx = tx.clone();
            let spinner_style = spinner_style_no_progress.clone();
            let mb = mb.clone();
            let path_func = *path_func;
            let contour_provider = contour_provider.clone();
            let cache = cache.clone();

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

                let mut path = None;

                if let Some(entry) = cache.get(&saved_path.name) {
                    if let Ok(saved_path_string) = ron::to_string(&saved_path) {
                        if saved_path_string == entry.saved_path_string {
                            path = ron::from_str(&entry.path_string).ok()
                        }
                    }
                }

                if path.is_none() {
                    path = Some(pxu::path::Path::from_base_path(
                        saved_path.clone().into(),
                        &contour_provider.get(consts).unwrap(),
                        consts,
                    ));
                }
                tx.send((path.unwrap(), saved_path, start)).unwrap();
                pb.finish_and_clear();
            });
        }

        let result = rx
            .into_iter()
            .take(paths.len())
            .map(|r: (pxu::Path, pxu::path::SavedPath, pxu::State)| {
                pb.inc(1);
                r
            })
            .collect::<Vec<_>>();

        pool.join();
        pb.finish_and_clear();

        let mut cache: HashMap<String, CacheEntry> = Default::default();

        for (path, saved_path, start) in result.iter() {
            self.add(&path.name, path.clone(), start.clone());
            let Ok(path_string) = ron::to_string(&path) else {
                continue;
            };
            let Ok(saved_path_string) = ron::to_string(&saved_path) else {
                continue;
            };
            cache.insert(
                saved_path.name.clone(),
                CacheEntry {
                    path_string,
                    saved_path_string,
                },
            );
        }

        if let Err(err) = save_cache(cache, cache_dirname) {
            eprintln!("{err}");
        }
    }
}
