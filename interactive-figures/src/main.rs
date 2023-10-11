use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Settings {
    #[arg(short, long, default_value = "./ifigures")]
    pub output_dir: String,
    #[arg(short, long)]
    pub rebuild: bool,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[arg(short, long)]
    pub jobs: Option<usize>,
}

struct FigureSource<'a> {
    filename: &'a str,
    name: &'a str,
    description: &'a str,
    path_names: Vec<&'a str>,
    state: Option<pxu::State>,
}

fn main() -> std::io::Result<()> {
    let figures = vec![FigureSource {
        filename: "crossing-0",
        name: "Crossing from (0,2π)",
        description:
            "Two different paths that can be used for crossing starting from p in the range (0,2π)",
        path_names: vec!["p crossing a", "p crossing b"],
        state: None,
    }];

    let settings = Settings::parse();

    if settings.verbose > 0 {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_file(true)
            .with_line_number(true)
            .with_writer(std::io::stderr)
            .without_time()
            .init();
        log::set_max_level(log::LevelFilter::Debug);
    }

    let num_threads = if let Some(jobs) = settings.jobs {
        jobs
    } else {
        num_cpus::get()
    };

    let spinner_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();
    let spinner_style_no_progress =
        ProgressStyle::with_template("[{elapsed_precise}] {spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let consts = pxu::CouplingConstants::new(2.0, 5);

    let mut contours = pxu::Contours::new();

    let pb = if settings.verbose == 0 {
        println!("[1/5] Generating contours");
        ProgressBar::new(1)
    } else {
        ProgressBar::hidden()
    };

    pb.set_style(spinner_style.clone());
    loop {
        pb.set_length(contours.progress().1 as u64);
        pb.set_position(contours.progress().0 as u64);
        if contours.update(0, consts) {
            pb.finish_and_clear();
            break;
        }
    }

    let mb = Arc::new(MultiProgress::new());
    let pb = if settings.verbose == 0 {
        println!("[2/5] Loading paths");
        mb.add(ProgressBar::new(1))
    } else {
        ProgressBar::hidden()
    };

    pb.set_style(spinner_style.clone());
    pb.set_length(make_paths::INTERACTIVE_PATHS.len() as u64);

    let pool = threadpool::ThreadPool::new(num_threads);
    let (tx, rx) = std::sync::mpsc::channel();

    for path_func in make_paths::INTERACTIVE_PATHS {
        let tx = tx.clone();
        let contours = contours.clone();
        let consts = consts;
        let spinner_style = spinner_style_no_progress.clone();
        let settings = settings.clone();
        let mb = mb.clone();
        pool.execute(move || {
            let pb = if settings.verbose == 0 {
                mb.add(ProgressBar::new(1))
            } else {
                ProgressBar::hidden()
            };
            pb.set_style(spinner_style);
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            pb.set_message("Generating path");

            let saved_path = path_func(&contours, consts);
            let start = saved_path.start.clone();

            pb.set_message(saved_path.name.clone());
            pb.tick();
            let path = pxu::path::Path::from_base_path(saved_path.into(), &contours, consts);
            tx.send((path, start)).unwrap();
            pb.finish_and_clear();
        });
    }

    let paths_and_starts = rx
        .into_iter()
        .take(make_paths::INTERACTIVE_PATHS.len())
        .map(|r| {
            pb.inc(1);
            r
        })
        .collect::<Vec<_>>();

    let mut path_map: HashMap<String, pxu::Path> = HashMap::new();
    let mut start_map: HashMap<String, pxu::State> = HashMap::new();

    for (path, start) in paths_and_starts {
        start_map.insert(path.name.clone(), start);
        path_map.insert(path.name.clone(), path);
    }

    pool.join();
    pb.finish_and_clear();

    let pb = if settings.verbose == 0 {
        println!("[3/5] Generating figures");
        ProgressBar::new(1)
    } else {
        ProgressBar::hidden()
    };

    pb.set_style(spinner_style.clone());
    pb.set_length(figures.len() as u64);

    let (descriptions, filename_and_figures): (Vec<_>, Vec<_>) = figures
        .into_iter()
        .map(|fig| {
            pb.set_message(fig.filename);
            let state = if fig.state.is_some() {
                fig.state.unwrap()
            } else if !fig.path_names.is_empty() {
                start_map.get(fig.path_names[0]).unwrap().clone()
            } else {
                panic!("Figure {} is empty", fig.name);
            };

            let paths = fig
                .path_names
                .into_iter()
                .map(|name| path_map.get(name).unwrap().clone())
                .collect::<Vec<_>>();

            let figure = ::interactive_figures::Figure { paths, state };

            let filename = fig.filename.to_owned();

            let descr = ::interactive_figures::FigureDescription {
                filename: filename.clone(),
                name: fig.name.to_owned(),
                description: fig.description.to_owned(),
            };

            pb.inc(1);

            (descr, (filename, figure))
        })
        .unzip();

    pb.finish_and_clear();

    println!("[4/5] Saving figures");

    for (filename, fig) in filename_and_figures.iter() {
        let toml = ron::to_string(&fig).unwrap();

        let mut path = PathBuf::from(settings.output_dir.clone()).join(filename.clone());
        path.set_extension("toml");

        let mut file = File::create(path)?;
        file.write_all(toml.as_bytes())?;
    }

    println!("[5/5] Saving descriptions");

    let toml = ron::to_string(&descriptions).unwrap();

    let path = PathBuf::from(settings.output_dir.clone()).join("figures.toml");

    let mut file = File::create(path)?;
    file.write_all(toml.as_bytes())?;

    Ok(())
}
