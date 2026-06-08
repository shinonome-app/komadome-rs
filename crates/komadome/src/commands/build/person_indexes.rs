use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::Ordering;

use super::BuildStats;
use crate::config::Config;
use crate::data::loader::JsonlIterator;
use crate::data::models::PersonIndexData;
use crate::generator::builder::person_index;
use crate::generator::templates::TemplateRegistry;

pub fn build_person_indexes_internal(
    config: &Config,
    templates: &TemplateRegistry,
    stats: &BuildStats,
    multi: &MultiProgress,
) -> Result<()> {
    let indexes_path = config.data.directory.join("person_indexes.jsonl");
    if !indexes_path.exists() {
        println!("person_indexes.jsonl not found, skipping person index generation");
        return Ok(());
    }

    // Count lines for progress bar
    let total = crate::data::loader::count_jsonl_lines(&indexes_path)?;

    let pb = multi.add(ProgressBar::new(total as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[person_idx] {bar:40.magenta/white} {pos}/{len} ({per_sec})")
            .unwrap()
            .progress_chars("=> "),
    );

    // Read all indexes into memory for parallel processing
    let indexes: Vec<PersonIndexData> = JsonlIterator::new(&indexes_path)?
        .filter_map(|r| r.ok())
        .collect();

    // Process in parallel
    indexes.par_iter().for_each(|index_data| {
        match build_person_index(config, templates, index_data) {
            Ok(_) => {
                stats.indexes_built.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!(
                    "Error building person index {}: {}",
                    index_data.kana_column, e
                );
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
    Ok(())
}

fn build_person_index(
    config: &Config,
    templates: &TemplateRegistry,
    data: &PersonIndexData,
) -> Result<()> {
    let ctx = person_index::build_person_index_context(data)?;

    let html = templates
        .render("indexes/people", ctx)
        .with_context(|| format!("Failed to render person index {}", data.kana_column))?;

    let filename = person_index::person_index_filename(&data.kana_column);
    let output_path = config.output.directory.join("index_pages").join(&filename);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, html)?;

    Ok(())
}
