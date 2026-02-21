use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fs;
use std::sync::atomic::Ordering;

use super::BuildStats;
use crate::config::Config;
use crate::data::loader;
use crate::data::masters::Masters;
use crate::data::models::NewsData;
use crate::generator::builder::soramoyou;
use crate::generator::templates::TemplateRegistry;

pub fn build_soramoyou_internal(
    config: &Config,
    masters: &Masters,
    templates: &TemplateRegistry,
    stats: &BuildStats,
    multi: &MultiProgress,
) -> Result<()> {
    let news_path = config.data.directory.join("news.jsonl");
    if !news_path.exists() {
        println!("news.jsonl not found, skipping soramoyou generation");
        return Ok(());
    }

    let all_data: Vec<NewsData> = loader::load_jsonl(&news_path)?;

    let pb = multi.add(ProgressBar::new(all_data.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[soramoyou] {bar:40.blue/white} {pos}/{len} ({per_sec})")
            .unwrap()
            .progress_chars("=> "),
    );

    let current_year = chrono::Datelike::year(&masters.exported_date());
    let soramoyou_dir = config.output.directory.join("soramoyou");
    fs::create_dir_all(&soramoyou_dir)?;

    for data in &all_data {
        let result: Result<()> = (|| {
            if data.year == current_year {
                // Current year -> index template
                let ctx = soramoyou::build_soramoyou_index_context(data, current_year)?;
                let html = templates
                    .render("soramoyou/index", ctx)
                    .with_context(|| "Failed to render soramoyou index page")?;
                let filename = soramoyou::soramoyou_index_filename();
                fs::write(soramoyou_dir.join(&filename), &html)?;
                // Also write as soramoyou{year}.html for the current year
                let year_filename = soramoyou::soramoyou_year_filename(data.year);
                fs::write(soramoyou_dir.join(&year_filename), html)?;
            } else {
                // Past year -> year template
                let ctx = soramoyou::build_soramoyou_year_context(data)?;
                let html = templates.render("soramoyou/year", ctx).with_context(|| {
                    format!("Failed to render soramoyou year page {}", data.year)
                })?;
                let filename = soramoyou::soramoyou_year_filename(data.year);
                fs::write(soramoyou_dir.join(&filename), html)?;
            }
            Ok(())
        })();

        match result {
            Ok(_) => {
                stats.soramoyou_built.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!("Error building soramoyou {}: {}", data.year, e);
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message("done");
    Ok(())
}
