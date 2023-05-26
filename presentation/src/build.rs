use crate::presentation_description::PresentationDescription;
use crate::{Error, Result};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

const TOML_NAME: &str = "presentation.toml";
const PDF_NAME: &str = "presentation.pdf";
const CACHE_NAME: &str = "cache.toml";

fn calculate_md5(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let md5 = md5::compute(data);
    Ok(format!("{:x}", md5))
}

fn read_presentation(path: &Path) -> Result<PresentationDescription> {
    let presentation_toml = std::fs::read_to_string(path)?;
    let presentation: PresentationDescription = toml::from_str(&presentation_toml)?;
    Ok(presentation)
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct PresentationCache {
    pdf_hash: String,
    toml_hash: String,
}

fn read_cache(path: &Path) -> Result<PresentationCache> {
    let cache_toml = std::fs::read_to_string(path)?;
    let cache: PresentationCache = toml::from_str(&cache_toml)?;
    Ok(cache)
}

pub fn check_presentation(dirname: &str, force_rebuild: bool) -> Result<()> {
    use std::{collections::BTreeMap, process::Command};

    let mut rebuild = force_rebuild;
    let mut rebuild_pdf = force_rebuild;

    let dir = std::path::Path::new(dirname);

    let toml_path = dir.join(TOML_NAME);
    let pdf_path = dir.join(PDF_NAME);
    let cache_path = dir.join(CACHE_NAME);

    let pdf_hash = calculate_md5(&pdf_path)?;

    if let Ok(cache) = read_cache(&cache_path) {
        let toml_hash = calculate_md5(&toml_path)?;

        if toml_hash != cache.toml_hash {
            log::info!(
                "toml hash does not match. Found '{}' expected '{}'",
                toml_hash,
                cache.toml_hash
            );
            rebuild = true;
        }
        if pdf_hash != cache.pdf_hash {
            log::info!(
                "PDF hash does not match. Found '{}' expected '{}'",
                pdf_hash,
                cache.pdf_hash
            );
            rebuild = true;
            rebuild_pdf = true;
        }
    } else {
        log::info!("Cache not found");
        rebuild = true;
        rebuild_pdf = true;
    }

    let mut presentation = read_presentation(&dir.join(TOML_NAME))?;

    for frame in presentation.frame.iter() {
        if !dir.join(&frame.image).exists() {
            log::info!("Image {} not found", frame.image);
            rebuild = true;
            rebuild_pdf = true;
        }
    }

    if !rebuild {
        return Ok(());
    }

    log::info!("Rebuilding");

    if rebuild_pdf {
        let presentation_pdf_path = dir.join(PDF_NAME);
        let presentation_pdf_name = presentation_pdf_path.as_os_str();

        let presentation_image_template_path = dir.join("presentation");
        let presentation_image_template_name = presentation_image_template_path.as_os_str();

        let mut cmd = Command::new("pdftoppm");
        cmd.args(["-png", "-scale-to-x", "-1", "-scale-to-y", "1024"])
            .args([presentation_pdf_name, presentation_image_template_name]);

        log::info!("Running pdftoppm");
        if !cmd.spawn()?.wait()?.success() {
            return Err(Error::Presentation(String::from("pdfroppm failed")));
        }
    }

    let mut image_to_image = BTreeMap::<String, String>::new();
    {
        let mut image_to_md5 = BTreeMap::<String, String>::new();
        let mut md5_to_image = BTreeMap::<String, String>::new();

        for frame in presentation.frame.iter() {
            let path = dir.join(&frame.image);
            let md5 = calculate_md5(&path)?;

            image_to_md5.insert(frame.image.clone(), md5.clone());

            if !md5_to_image.contains_key(&md5) {
                md5_to_image.insert(md5.clone(), frame.image.clone());
            }

            image_to_image.insert(frame.image.clone(), md5_to_image.get(&md5).unwrap().clone());
        }

        let values = image_to_image.values().collect::<Vec<_>>();

        for name in image_to_image.keys() {
            if !values.contains(&name) {
                log::info!("Duplicate image {name}");
            }
        }
    }

    for frame in presentation.frame.iter_mut() {
        frame.image = image_to_image.get(&frame.image).unwrap().clone();
    }

    let toml = toml::to_string(&presentation)?;

    std::fs::write(toml_path.clone(), toml)?;

    let toml_hash = calculate_md5(&toml_path)?;

    let cache = PresentationCache {
        toml_hash,
        pdf_hash,
    };

    let cache_toml = toml::to_string(&cache)?;

    std::fs::write(cache_path.clone(), cache_toml)?;

    Ok(())
}
