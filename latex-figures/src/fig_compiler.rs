use std::io::{BufRead, Result};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;

use indicatif::ProgressBar;

use crate::cache;
use crate::fig_writer::FigureWriter;
use crate::utils::{Settings, Size, PDF_EXT, PROGRESS_EXT, TEX_EXT};

pub struct FigureCompiler {
    pub name: String,
    pub caption: String,
    child: Child,
    plot_count: u64,
    size: Size,
    cached: bool,
}

#[derive(Debug)]
pub struct FinishedFigure {
    pub name: String,
    pub caption: String,
    pub size: Size,
    pub lualatex_error: bool,
}

impl FigureCompiler {
    pub fn new(
        figure: FigureWriter,
        cache: Arc<cache::Cache>,
        settings: &Settings,
    ) -> Result<Self> {
        let FigureWriter {
            name,
            caption,
            size,
            plot_count,
            ..
        } = figure;
        if !settings.rebuild && cache.check(&name)? {
            log::info!("[{name}]: Matches cached entry");
            let child = Command::new("/bin/true").spawn()?;
            Ok(Self {
                name,
                caption,
                child,
                plot_count: 0,
                size,
                cached: true,
            })
        } else {
            let mut path = PathBuf::from(&settings.output_dir).join(name.clone());
            path.set_extension(TEX_EXT);

            let mut cmd = Command::new(&settings.lualatex);
            cmd.arg(format!("--output-directory={}", settings.output_dir))
                .args(["--interaction=nonstopmode", "--output-format=pdf"])
                .arg(path.as_os_str())
                .stderr(Stdio::null())
                .stdout(Stdio::null());

            log::info!("[{name}]: Running Lualatex");
            let child = cmd.spawn()?;

            Ok(Self {
                name,
                caption,
                child,
                plot_count,
                size,
                cached: false,
            })
        }
    }

    pub fn get_latex_errors(&self, output_dir: &str) -> Result<Vec<String>> {
        let mut path = PathBuf::from(output_dir).join(self.name.clone());
        path.set_extension("log");

        let mut errors = vec![];

        let file = std::fs::File::open(path)?;
        for line in std::io::BufReader::new(file).lines() {
            let line = line?;
            if line.starts_with("! ") {
                errors.push(line);
            }
        }

        errors.sort();
        errors.dedup();

        Ok(errors)
    }

    pub fn wait(mut self, pb: &ProgressBar, settings: &Settings) -> Result<FinishedFigure> {
        pb.set_length(self.plot_count + 1);
        let mut progress_path = PathBuf::from(&settings.output_dir).join(&self.name);
        progress_path.set_extension(PROGRESS_EXT);
        let mut lualatex_error = false;
        loop {
            pb.tick();
            if let Ok(meta) = progress_path.metadata() {
                pb.set_position(meta.len());
            }

            if let Some(result) = self.child.try_wait()? {
                if !self.cached {
                    if result.success() {
                        log::info!("[{}]: Lualatex done.", self.name);
                    } else {
                        // TODO: check if a pdf file was generated
                        lualatex_error = true;

                        log::error!("[{}]: Lualatex failed.", self.name);
                        if let Ok(errors) = self.get_latex_errors(&settings.output_dir) {
                            let accepted_errors = ["! Dimension too large.".to_owned()];
                            if let Some(error) =
                                errors.iter().find(|err| !accepted_errors.contains(err))
                            {
                                panic!("Luatex failed for {} with {error}", self.name);
                            }
                        } else {
                            panic!("Could not read log file for {}", self.name);
                        }
                    }
                }
                break;
            }
            thread::sleep(std::time::Duration::from_millis(250));
        }
        let _ = std::fs::remove_file(progress_path);

        if !settings.no_compress && !self.cached {
            pb.set_message(format!("Compressing {}.pdf", self.name));
            log::info!("[{}]: Compressing {}.pdf", self.name, self.name);

            let mut final_path = PathBuf::from(&settings.output_dir).join(&self.name);
            final_path.set_extension(PDF_EXT);

            let temp_name = format!("{}-temp", self.name);

            let mut temp_path = PathBuf::from(&settings.output_dir).join(temp_name);
            temp_path.set_extension(PDF_EXT);

            std::fs::copy(&final_path, &temp_path)?;

            //gs -sDEVICE=pdfwrite -dCompatibilityLevel=1.5 -dPDFSETTINGS=/printer -dNOPAUSE -dQUIET -dBATCH -sOutputFile=
            let mut cmd = Command::new("gs");
            cmd.args([
                "-sDEVICE=pdfwrite",
                "-dCompatibilityLevel=1.5",
                "-dPDFSETTINGS=/printer",
                "-dNOPAUSE",
                "-dQUIET",
                "-dBATCH",
            ])
            .arg(format!(
                "-sOutputFile={}",
                final_path.as_os_str().to_str().unwrap()
            ))
            .arg(temp_path.as_os_str())
            .stderr(Stdio::null())
            .stdout(Stdio::null());

            cmd.spawn()?.wait()?;

            let _ = std::fs::remove_file(temp_path);
        }

        Ok(FinishedFigure {
            name: self.name,
            caption: self.caption,
            size: self.size,
            lualatex_error,
        })
    }
}
