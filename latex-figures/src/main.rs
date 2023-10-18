use make_paths::PxuProvider;
use pxu::kinematics::CouplingConstants;
use std::io::Result;
use std::sync::Arc;

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

mod cache;
mod fig_compiler;
mod fig_writer;
mod figures;
mod utils;

use crate::figures::ALL_FIGURES;
use crate::utils::{error, Settings, Summary, SUMMARY_NAME};

fn check_for_gs() -> bool {
    let mut cmd = std::process::Command::new("gs");
    cmd.arg("--version")
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null());
    match cmd.spawn() {
        Ok(mut child) => {
            if child.wait().is_err() {
                log::info!("Could not run \"gs\"");
                false
            } else {
                true
            }
        }
        Err(_) => {
            log::info!("Could not run \"gs\"");
            false
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut settings = Settings::parse();
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

    if !settings.no_compress {
        settings.no_compress = !check_for_gs();
    }

    let num_threads = if let Some(jobs) = settings.jobs {
        jobs
    } else {
        num_cpus::get()
    };

    let pool = threadpool::ThreadPool::new(num_threads);

    if settings.rebuild {
        println!(" ---  Rebuilding all figures");
    }

    let spinner_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();
    let spinner_style_no_progress =
        ProgressStyle::with_template("[{elapsed_precise}] {spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let cache = cache::Cache::load(&settings.output_dir)?;

    let consts_list = vec![
        CouplingConstants::new(2.0, 5),
        CouplingConstants::new(7.0, 3),
    ];

    let mut pxu_provider = PxuProvider::new();

    println!("[1/5] Generating figures");
    pxu_provider.generate_contours(consts_list, verbose, &pool, &spinner_style);

    println!("[2/5] Loading paths");
    pxu_provider.load_paths(
        make_paths::PLOT_PATHS,
        verbose,
        &pool,
        &spinner_style,
        &spinner_style_no_progress,
    );

    let pxu_provider = Arc::new(pxu_provider);
    let cache = Arc::new(cache);

    if !verbose {
        if settings.rebuild {
            println!("[3/5] Building figures (ignoring cache)");
        } else {
            println!("[3/5] Building figures");
        }
    }
    let mb = Arc::new(MultiProgress::new());

    let (tx, rx) = std::sync::mpsc::channel();

    let pb = if !verbose {
        mb.add(ProgressBar::new_spinner())
    } else {
        ProgressBar::hidden()
    };

    pb.set_style(spinner_style.clone());
    pb.set_message("Building figures");
    pb.set_length(ALL_FIGURES.len() as u64);
    pb.enable_steady_tick(std::time::Duration::from_millis(250));

    for (i, f) in ALL_FIGURES.iter().enumerate() {
        let pxu_provider = pxu_provider.clone();
        let cache_ref = cache.clone();
        let spinner_style = spinner_style.clone();
        let settings = settings.clone();
        let mb = mb.clone();
        let tx = tx.clone();
        pool.execute(move || {
            let pb = if !verbose {
                mb.add(ProgressBar::new_spinner())
            } else {
                ProgressBar::hidden()
            };
            pb.set_style(spinner_style);

            match f(pxu_provider, cache_ref, &settings, &pb) {
                Ok(figure) => {
                    let result = figure.wait(&pb, &settings);
                    pb.finish_and_clear();
                    tx.send(result.map(|r| (i, r))).unwrap();
                }
                Err(e) => {
                    tx.send(Err(e)).unwrap();
                }
            }
        });
    }

    let mut finished_figures = rx
        .into_iter()
        .take(ALL_FIGURES.len())
        .map(|r| {
            pb.inc(1);
            r
        })
        .collect::<Result<Vec<_>>>()?;
    pool.join();
    pb.finish_and_clear();

    finished_figures.sort_by_key(|&(n, _)| n);
    let finished_figures = finished_figures.into_iter().map(|(_, r)| r);

    let mut new_cache = cache::Cache::new(&settings.output_dir);
    let mut summary = Summary::default();

    for finished_figure in finished_figures {
        new_cache.update(&finished_figure.name)?;
        summary.add(finished_figure);
    }

    if !verbose {
        println!("[4/5] Saving cache");
    }
    new_cache.save()?;

    if !verbose {
        println!("[5/5] Building summary");
    }

    let pb = if !verbose {
        ProgressBar::new_spinner()
    } else {
        ProgressBar::hidden()
    };

    pb.set_style(spinner_style_no_progress);
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    if summary.finish(&settings, &pb)?.wait()?.success() {
        log::info!("[{SUMMARY_NAME}] Done.");
    } else {
        log::error!("[{SUMMARY_NAME}] Error.");
        return Err(error("Error compiling summary"));
    }

    pb.finish_and_clear();

    Ok(())
}
