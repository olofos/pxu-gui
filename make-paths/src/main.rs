use std::sync::Arc;

use clap::Parser;
use indicatif::ProgressStyle;
use make_paths::ContourProvider;
use pxu::kinematics::CouplingConstants;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Settings {
    #[arg(short, long)]
    compressed: bool,
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    path_number: Option<usize>,
}

fn main() -> std::io::Result<()> {
    let settings = Settings::parse();

    let pool = threadpool::ThreadPool::new(5);

    let spinner_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();

    let consts_list = vec![CouplingConstants::new(2.0, 5)];

    eprintln!("[1/3] Generating contours");
    let mut contour_provider = ContourProvider::default();
    contour_provider.generate(consts_list, false, &pool, &spinner_style);

    let contour_provider = Arc::new(contour_provider);

    eprintln!("[2/3] Generating paths");
    let saved_paths = make_paths::INTERACTIVE_PATHS
        .iter()
        .map(|f| f(contour_provider.clone()))
        .collect::<Vec<_>>();

    eprintln!("[3/3] Saving paths");

    let result = if settings.compressed {
        pxu::path::SavedPath::save_compressed(&saved_paths)
    } else {
        pxu::path::SavedPath::save(&saved_paths)
    }
    .unwrap();
    println!("{result}");

    eprintln!("");
    eprintln!("Built {} paths", make_paths::INTERACTIVE_PATHS.len());
    eprintln!("");
    eprintln!("{}", contour_provider.get_statistics());

    Ok(())
}
