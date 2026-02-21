use anyhow::Result;
use std::fs;

use crate::config::Config;

pub fn copy_assets(config: &Config) -> Result<()> {
    let assets_config = match &config.assets {
        Some(c) => c,
        None => {
            println!("No [assets] config, skipping asset copy");
            return Ok(());
        }
    };

    let output_dir = &config.output.directory;
    let mut copied = 0usize;

    // Copy CSS files and create fingerprint-free aliases
    if let Some(css_dir) = &assets_config.css_dir {
        if css_dir.exists() {
            let assets_out = output_dir.join("assets");
            fs::create_dir_all(&assets_out)?;
            for entry in fs::read_dir(css_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let filename = path.file_name().unwrap();
                    let dest = assets_out.join(filename);
                    fs::copy(&path, &dest)?;
                    copied += 1;

                    // Create fingerprint-free alias (e.g., tailwind-abc123.css -> tailwind.css)
                    let fname = filename.to_string_lossy();
                    if let Some(base) = strip_fingerprint(&fname) {
                        let alias = assets_out.join(base);
                        if !alias.exists() {
                            fs::copy(&dest, &alias)?;
                        }
                    }
                }
            }
        } else {
            println!("  CSS dir not found: {}", css_dir.display());
        }
    }

    // Copy public images
    if let Some(images_dir) = &assets_config.images_dir {
        if images_dir.exists() {
            let images_out = output_dir.join("images");
            fs::create_dir_all(&images_out)?;
            copy_dir_recursive(images_dir, &images_out, &mut copied)?;
        } else {
            println!("  Images dir not found: {}", images_dir.display());
        }
    }

    // Copy card images
    if let Some(card_images_dir) = &assets_config.card_images_dir {
        if card_images_dir.exists() {
            let cards_images_out = output_dir.join("cards").join("images");
            fs::create_dir_all(&cards_images_out)?;
            copy_dir_recursive(card_images_dir, &cards_images_out, &mut copied)?;
        } else {
            println!("  Card images dir not found: {}", card_images_dir.display());
        }
    }

    // Copy ZIP files
    if let Some(zip_dir) = &assets_config.zip_dir {
        if zip_dir.exists() {
            let index_pages_out = output_dir.join("index_pages");
            fs::create_dir_all(&index_pages_out)?;
            for entry in fs::read_dir(zip_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "zip") {
                    let filename = path.file_name().unwrap();
                    fs::copy(&path, index_pages_out.join(filename))?;
                    copied += 1;
                }
            }
        } else {
            println!("  ZIP dir not found: {}", zip_dir.display());
        }
    }

    if copied > 0 {
        println!("Copied {} asset files", copied);
    }
    Ok(())
}

/// Strip Rails asset fingerprint from filename.
/// e.g., "tailwind-ffccb42b.css" -> "tailwind.css"
///       "inter-font-8c3e82af.css" -> "inter-font.css"
fn strip_fingerprint(filename: &str) -> Option<String> {
    // Match pattern: name-hexhash.ext (hash is 8+ hex chars)
    let re = regex::Regex::new(r"^(.+)-[0-9a-f]{8,}(\.css(?:\.gz)?|\.js(?:\.gz)?)$").unwrap();
    re.captures(filename).map(|caps| format!("{}{}", &caps[1], &caps[2]))
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path, count: &mut usize) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path, count)?;
        } else {
            fs::copy(&path, &dest_path)?;
            *count += 1;
        }
    }
    Ok(())
}

pub fn build_404_page(config: &Config) -> Result<()> {
    let html = include_str!("../../../static/404.html");
    let output_path = config.output.directory.join("404.html");
    fs::write(&output_path, html)?;
    println!("Built 404 page: 404.html");
    Ok(())
}
