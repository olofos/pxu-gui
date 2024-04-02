use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use make_paths::PxuProvider;
use pxu::CouplingConstants;
use std::{path::PathBuf, sync::Arc};

pub fn error(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, message)
}

fn load_state(s: &str) -> std::io::Result<pxu::State> {
    ron::from_str(s).map_err(|_| error("Could not load state"))
}

const PATH_CACHE_DIR: &str = ".cache";

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
    paper_ref: Vec<&'a str>,
}

fn main() -> std::io::Result<()> {
    let figures = vec![
    FigureSource {
        filename: "simple-path",
        name: "A simple path",
        description: "A simple path that brings x⁺ and x⁻ from the outside of the scallion to the region between the scallion and the kidney.",
        path_names: vec!["u simple path 1", "u simple path 2","u simple path 3","u simple path 4",],
        state: None,
        consts: (2.0, 5),
        paper_ref: vec!["10"]
    },
    FigureSource {
        filename: "large-circle",
        name: "A large circle",
        description: "x⁺ makes a large circle around the origin.",
        path_names: vec!["xp large circle",],
        state: None,
        consts: (2.0, 5),
        paper_ref: vec!["11"]
    },
    FigureSource {
        filename: "between-regions",
        name: "Paths between regions",
        description: "",
        path_names: vec![
            "p from region 0 to region -1", 
            "p from region -1 to region -2 conj",
            "p from region -2 to region -3 conj",
            "p from region 0 to region +1",
            "p from region +1 to region +2",
            "p from region +2 to region +3",
            ],
        state: None,
        consts: (2.0, 5),
        paper_ref: vec!["13"]
    },
    FigureSource {
        filename: "typical-bs-0-1",
        name: "m=4 state in (0,2π)",
        description:
        "",
        path_names: vec![],
        state: Some(load_state("(points:[(p:(0.0369899543404076,-0.029477676458957484),xp:(3.725975442509692,2.6128313499217866),xm:(3.5128286480709265,1.3995994557612454),u:(2.7000494004152316,1.5000010188076138),x:(3.6217633112309158,2.022895894514536),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06034321575136616,-0.018323213928633217),xp:(3.512828648070947,1.3995994557612081),xm:(3.3701632658975504,0.000001507484578833207),u:(2.700049400415252,0.5000010188075885),x:(3.4147970768250535,0.7263861464447217),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.06034326215107557,0.018323155770842862),xp:(3.370163265897615,0.0000015074845481910515),xm:(3.5128282084799323,-1.3995968258500417),u:(2.700049400415295,-0.49999898119243236),x:(3.4147967471340466,-0.7263832822620354),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.03698999112227798,0.029477675660386345),xp:(3.5128282084799114,-1.3995968258500804),xm:(3.7259750341536533,-2.6128289961240028),u:(2.700049400415274,-1.4999989811924586),x:(3.621762872183573,-2.0228934323008243),sheet_data:(log_branch_p:0,log_branch_m:0,log_branch_x:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1)))],unlocked:false)")?),
        consts: (2.0, 5),
        paper_ref: vec!["17a","18"],
    },
    FigureSource {
        filename: "typical-bs-0-2",
        name: "m=7 state in (0,2π)",
        description:
        "",
        path_names: vec![],
        state: Some(load_state("(points:[(p:(-0.008285099942215936,-0.03124489976444211),xp:(-0.41379014705206596,5.013730349990057),xm:(-0.5539512485108423,4.096765155780589),u:(-1.7157731060643773,3.000099539239211),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,-1))),(p:(-0.012817797608166157,-0.03617378274379514),xp:(-0.5539512485108438,4.096765155780585),xm:(-0.7024745389520475,3.217777875518938),u:(-1.7157731060643784,2.0000995392392076),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019777502854940465,-0.04157814705589314),xp:(-0.7024745389520499,3.2177778755189355),xm:(-0.8439370224593588,2.391830970565371),u:(-1.7157731060643804,1.0000995392392027),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.6079767764853242,-0.000008833067157527095),xp:(-0.8439370224593605,2.391830970565368),xm:(-0.8439626423264122,-2.3916726610840278),u:(-1.7157731060643822,0.0000995392391995864),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.019779171573578672,0.041579250470216406),xp:(-0.8439626423264142,-2.3916726610840273),xm:(-0.7025041652445985,-3.21760768570613),u:(-1.7157731060643844,-0.9999004607608009),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.012818918443990657,0.03617482310579956),xp:(-0.7025041652445959,-3.2176076857061333),xm:(-0.5539802718296103,-4.096585899228867),u:(-1.7157731060643822,-1.9999004607608049),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-0.008285809485964725,0.031245812444520096),xp:(-0.5539802718296084,-4.09658589922887),xm:(-0.4138167904094644,-5.013544938781717),u:(-1.7157731060643802,-2.9999004607608075),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))],unlocked:false)",)?),
        consts: (2.0, 5),
        paper_ref: vec!["17a","18"],
    },
    FigureSource {
        filename: "typical-bs-1",
        name: "m=2 state in (2π,4π)",
        description:
        "",
        path_names: vec![],
        state: Some(load_state("(points:[(p:(1.5344982847391835,-0.03125157629093187),xp:(-0.4137901655608822,5.013730158365311),xm:(-0.5539802334816937,-4.096586081878231),u:(-1.7157730965680082,-1.9999006651456805),sheet_data:(log_branch_p:1,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(-0.00828580874234546,0.031245811489086096),xp:(-0.5539802413347306,-4.0965860869401025),xm:(-0.4138167624035101,-5.013545132940062),u:(-1.715773105953617,-2.9999006692476753),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1)))],unlocked:false)",)?),
        consts: (2.0, 5),
        paper_ref: vec!["17b","19"],
    },
    FigureSource {
        filename: "typical-bs-min-1",
        name: "m=4 state in (-2π,0)",
        description:
        "",
        path_names: vec![],
        state: Some(load_state("(points:[(p:(-0.04492676714509915,-0.023287148957676335),xp:(-2.2982685996303633,1.7011141634148028),xm:(-2.3162023933609586,0.8583601532032655),u:(-3.4154076535523155,4.000100793457268),sheet_data:(log_branch_p:-1,log_branch_m:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.0564778288751243,-0.010296000935336903),xp:(-2.316202393360959,0.8583601532032651),xm:(-2.3153985683471108,0.00008710430978264849),u:(-3.4154076535523163,3.0001007934572677),sheet_data:(log_branch_p:-1,log_branch_m:-3,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.056479445909146386,0.01029221421273873),xp:(-2.315398568347111,0.00008710430978253747),xm:(-2.3162031403629046,-0.8581889963326543),u:(-3.4154076535523172,2.000100793457267),sheet_data:(log_branch_p:-1,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.04492931592095178,0.023285635921691496),xp:(-2.316203140362906,-0.8581889963326539),xm:(-2.298275528949721,-1.7009447564270626),u:(-3.415407653552319,1.000100793457268),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1)))],unlocked:false)",)?),
        consts: (2.0, 5),
        paper_ref: vec!["20a","21"],
    },
    FigureSource {
        filename: "typical-bs-min-2",
        name: "m=3 state in (-4π,-2π)",
        description:
        "",
        path_names: vec![],
        state: Some(load_state("(points:[(p:(-1.4606821908812262,-0.08552402227919431),xp:(-0.036494412912998445,0.3868862252151071),xm:(-0.034602130895845726,-0.2244039105108243),u:(0.47400377737283,6.000100042285478),sheet_data:(log_branch_p:-2,log_branch_m:0,e_branch:1,u_branch:(Inside,Inside),im_x_sign:(1,1))),(p:(-0.0024712590245176227,0.03841793097115144),xp:(-0.03460213089584572,-0.22440391051082456),xm:(-0.03960815630989887,-0.28631872432272015),u:(0.4740037773728304,5.000100042285471),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(1,1))),(p:(-0.006907346397911845,0.047095708971704085),xp:(-0.039608156309898904,-0.28631872432272),xm:(-0.036497086475895155,-0.38686051106138636),u:(0.4740037773728296,4.000100042285474),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Inside,Inside),im_x_sign:(-1,1)))],unlocked:false)",)?),
        consts: (2.0, 5),
        paper_ref: vec!["20b","21"],
    },
    FigureSource {
        filename: "bs-3-min-1",
        name: "m=3 bound state in (-2π,0)",
        description:
        "",
        path_names: vec!["bs3 region -1 1", "bs3 region -1 2"],
        state: None,
        consts: (1.0, 7),
        paper_ref: vec!["22"],
    },
    FigureSource {
        filename: "crossing-0a",
        name: "Crossing from (0,2π)",
        description:
        "Two paths that can be used for crossing starting from p in the range (0,2π)",
        path_names: vec!["p crossing a", "p crossing b"],
        state: None,
        consts: (2.0, 5),
        paper_ref: vec!["26","27","28"],
    },
    FigureSource {
        filename: "crossing-0b",
        name: "Another crossing from (0,2π)",
        description:
        "Two more less convenient paths that can be used for crossing starting from p in the range (0,2π)",
        path_names: vec!["p crossing c", "p crossing d"],
        state: None,
        consts: (2.0, 5),
        paper_ref: vec!["26"],
    },
    FigureSource {
        filename: "singlet-0",
        name: "Singlet state in (0,2π)",
        description:
        "",
        path_names: vec![],
        state: Some(load_state("(points:[(p:(0.035920572686227975,-0.0371245201982526),xp:(3.278541909565751,2.69764230683293),xm:(3.0086748709958817,1.501168090727413),u:(2.3098001480095305,1.5000993687596509),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.0736477003995048,-0.031881014951510876),xp:(3.0086748709958773,1.5011680907274152),xm:(2.752022495646597,0.00017167978252885518),u:(2.3098001480095274,0.5000993687596516),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(0.07365802450198924,0.031873014242525234),xp:(2.7520224956465924,0.00017167978252619065),xm:(3.008613535972122,-1.500912421713252),u:(2.3098001480095243,-0.49990063124035),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(-1,1))),(p:(0.035924674842931,0.03712580047228859),xp:(3.0086135359721218,-1.5009124217132535),xm:(3.2784955205790927,-2.6974165274435005),u:(2.309800148009524,-1.4999006312403511),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Outside,Outside),im_x_sign:(1,1))),(p:(-1.2191509724306528,0.000006720434949787522),xp:(3.278495520579101,-2.697416527443499),xm:(3.2785419095657513,2.697642306832927),u:(2.309800148009531,2.500099368759649),sheet_data:(log_branch_p:-1,log_branch_m:0,e_branch:-1,u_branch:(Outside,Outside),im_x_sign:(1,-1)))],unlocked:false)",)?),
        consts: (2.0, 5),
        paper_ref: vec!["32"],
    },
    FigureSource {
        filename: "singlet-min-1",
        name: "Singlet state in (-2π,0)",
        description:
        "",
        path_names: vec![],
        state: Some(load_state("(points:[(p:(-0.04915040522405487,-0.045791051935815626),xp:(-1.3220716930339478,1.6552562481272564),xm:(-1.3219227444059347,0.8813162555256742),u:(-2.214036050469592,4.000101180615412),sheet_data:(log_branch_p:-1,log_branch_m:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.09357322668831639,-0.03991326998630673),xp:(-1.321922744405919,0.8813162555256757),xm:(-1.2363694671632584,0.00010225956113174561),u:(-2.214036050469572,3.000101180615414),sheet_data:(log_branch_p:-1,log_branch_m:-3,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,1))),(p:(-0.09358689247514664,0.03990349663451138),xp:(-1.2363694671632492,0.00010225956111992174),xm:(-1.3219116746778858,-0.8811569763752188),u:(-2.214036050469563,2.000101180615402),sheet_data:(log_branch_p:-1,log_branch_m:1,e_branch:1,u_branch:(Between,Between),im_x_sign:(-1,-1))),(p:(-0.049155153779756815,0.045792040962502355),xp:(-1.3219116746778863,-0.8811569763752252),xm:(-1.322081015696217,-1.6550991615231962),u:(-2.214036050469563,1.0001011806153943),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:1,u_branch:(Between,Between),im_x_sign:(1,1))),(p:(-0.7145343218327235,0.000008784325108582892),xp:(-1.3220810156962146,-1.6550991615231967),xm:(-1.3220716930339236,1.6552562481272393),u:(-2.2140360504695593,0.00010118061539343692),sheet_data:(log_branch_p:0,log_branch_m:0,e_branch:-1,u_branch:(Between,Between),im_x_sign:(-1,-1)))],unlocked:false)",)?),
        consts: (2.0, 5),
        paper_ref: vec!["32"],
    },
    ];

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

    eprintln!("[1/5] Generating figures");
    pxu_provider.generate_contours(consts_list, verbose, &pool, &spinner_style);

    eprintln!("[2/5] Loading paths");
    pxu_provider.load_paths(
        make_paths::INTERACTIVE_PATHS,
        verbose,
        &pool,
        PATH_CACHE_DIR,
        &spinner_style,
        &spinner_style_no_progress,
    );

    let pxu_provider = Arc::new(pxu_provider);

    let pb = if !verbose {
        eprintln!("[3/5] Generating figures");
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
                if pxu_provider.get_path(name).is_err() {
                    panic!("Path {name} not found");
                }
            }

            let state = if fig.state.is_some() {
                fig.state.unwrap()
            } else if let Ok(start) = pxu_provider.get_start(fig.path_names[0]) {
                (*start).clone()
            } else {
                panic!("Figure {} is empty", fig.name);
            };

            let paths = fig
                .path_names
                .into_iter()
                .map(|name| (*pxu_provider.get_path(name).unwrap()).clone())
                .collect::<Vec<_>>();

            let consts = pxu::CouplingConstants::new(fig.consts.0, fig.consts.1);

            let figure = ::interactive_figures::Figure {
                paths,
                state,
                consts,
            };

            let filename = fig.filename.to_owned();

            let descr = ::interactive_figures::FigureDescription {
                filename: filename.clone(),
                name: fig.name.to_owned(),
                description: fig.description.to_owned(),
                consts: pxu::CouplingConstants::new(fig.consts.0, fig.consts.1),
                paper_ref: fig.paper_ref.iter().map(|s| String::from(*s)).collect(),
            };

            pb.inc(1);

            (descr, (filename, figure))
        })
        .unzip();

    pb.finish_and_clear();

    eprintln!("[4/5] Saving figures");

    let path = PathBuf::from(settings.output_dir.clone());
    std::fs::create_dir_all(path)?;

    for (filename, fig) in filename_and_figures.iter() {
        let ron = ron::to_string(&fig).unwrap();

        let mut path = PathBuf::from(settings.output_dir.clone()).join(filename);
        path.set_extension("ron");

        std::fs::write(path, ron)?;
    }

    eprintln!("[5/5] Saving descriptions");

    let ron = ron::to_string(&descriptions).unwrap();

    let path = PathBuf::from(settings.output_dir.clone()).join("figures.ron");
    std::fs::write(path, ron)?;

    pool.join();

    eprintln!();
    eprintln!("Built {} figures", descriptions.len());
    eprintln!();
    eprintln!("{}", pxu_provider.get_statistics());

    Ok(())
}
