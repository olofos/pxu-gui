use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use make_paths::PxuProvider;
use pxu::CouplingConstants;
use std::{path::PathBuf, sync::Arc};

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Settings {
    #[arg(short, long, default_value = "./pxu-gui/dist/data/")]
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
    consts: (f64, i32),
}

fn main() -> std::io::Result<()> {
    let figures = vec![
        FigureSource {
        filename: "crossing-0a",
        name: "Crossing from (0,2π)",
        description:
            "Two paths that can be used for crossing starting from p in the range (0,2π)",
        path_names: vec!["p crossing a", "p crossing b"],
        state: None,
        consts: (2.0, 5),
    },
    FigureSource {
        filename: "crossing-0b",
        name: "Another crossing from (0,2π)",
        description:
            "Two more less convenient paths that can be used for crossing starting from p in the range (0,2π)",
        path_names: vec!["p crossing c", "p crossing d"],
        state: None,
        consts: (2.0, 5),
    },
    FigureSource {
        filename: "xp-circle-between",
        name: "x⁺ circle between/between",
        description: "x⁺ goes in a circle around the kidney with x⁻ staying between the scallion and the kidney. This path is periodic in the p, x⁺ and x⁻ planes.",
        path_names: vec!["xp circle between/between"],
        state: None,
        consts: (2.0, 5),
    },
    FigureSource {
        filename: "xp-circle-between-outside",
        name: "x⁺ circle between/outside",
        description: "x⁺ goes in a circle around the kidney with x⁻ staying outside the scallion.",
        path_names: vec!["xp circle between/outside L", "xp circle between/outside R"],
        state: None,
        consts: (2.0, 5),
    },
    FigureSource {
        filename: "xp-circle-between-inside",
        name: "x⁺ circle between/inside",
        description: "x⁺ goes in a circle around the kidney with x⁻ staying inside the scallion.",
        path_names: vec!["xp circle between/inside L", "xp circle between/inside R"],
        state: None,
        consts: (2.0, 5),
    }];

    let settings = Settings::parse();

    let verbose = settings.verbose > 0;
    if verbose {
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

    let pool = threadpool::ThreadPool::new(num_threads);

    let spinner_style: ProgressStyle = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();
    let spinner_style_no_progress =
        ProgressStyle::with_template("[{elapsed_precise}] {spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let consts_list = figures
        .iter()
        .map(|fig| CouplingConstants::new(fig.consts.0, fig.consts.1))
        .unique_by(|c| format!("h={:.3} k={}", c.h, c.k()))
        .collect::<Vec<_>>();

    let mut pxu_provider = PxuProvider::new();

    println!("[1/5] Generating figures");
    pxu_provider.generate_contours(consts_list, verbose, &pool, &spinner_style);

    println!("[2/5] Loading paths");
    pxu_provider.load_paths(
        make_paths::INTERACTIVE_PATHS,
        verbose,
        &pool,
        &spinner_style,
        &spinner_style_no_progress,
    );

    let pxu_provider = Arc::new(pxu_provider);

    let pb = if verbose {
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

            for name in fig.path_names.iter() {
                if pxu_provider.get_path(name).is_none() {
                    panic!("Path {name} not found");
                }
            }

            let state = if fig.state.is_some() {
                fig.state.unwrap()
            } else if let Some(start) = pxu_provider.get_start(fig.path_names[0]) {
                (*start).clone()
            } else {
                panic!("Figure {} is empty", fig.name);
            };

            let paths = fig
                .path_names
                .into_iter()
                .map(|name| (*pxu_provider.get_path(name).unwrap()).clone())
                .collect::<Vec<_>>();

            let figure = ::interactive_figures::Figure { paths, state };

            let filename = fig.filename.to_owned();

            let descr = ::interactive_figures::FigureDescription {
                filename: filename.clone(),
                name: fig.name.to_owned(),
                description: fig.description.to_owned(),
                consts: pxu::CouplingConstants::new(fig.consts.0, fig.consts.1),
            };

            pb.inc(1);

            (descr, (filename, figure))
        })
        .unzip();

    pb.finish_and_clear();

    println!("[4/5] Saving figures");

    let path = PathBuf::from(settings.output_dir.clone());
    std::fs::create_dir_all(&path)?;

    for (filename, fig) in filename_and_figures.iter() {
        let ron = ron::to_string(&fig).unwrap();

        let mut path = PathBuf::from(settings.output_dir.clone()).join(filename);
        path.set_extension("ron");

        std::fs::write(path, ron)?;
    }

    println!("[5/5] Saving descriptions");

    let ron = ron::to_string(&descriptions).unwrap();

    let path = PathBuf::from(settings.output_dir.clone()).join("figures.ron");
    std::fs::write(path, ron)?;

    pool.join();

    Ok(())
}
