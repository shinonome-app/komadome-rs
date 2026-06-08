use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::Ordering;

use super::BuildStats;
use crate::config::Config;
use crate::data::loader::JsonlIterator;
use crate::data::models::WorkIndexData;
use crate::generator::builder::work_index;
use crate::generator::templates::TemplateRegistry;

pub fn build_work_indexes_internal(
    config: &Config,
    templates: &TemplateRegistry,
    stats: &BuildStats,
    multi: &MultiProgress,
) -> Result<()> {
    let indexes_path = config.data.directory.join("work_indexes.jsonl");
    if !indexes_path.exists() {
        println!("work_indexes.jsonl not found, skipping work index generation");
        return Ok(());
    }

    // Count lines for progress bar
    let total = crate::data::loader::count_jsonl_lines(&indexes_path)?;

    let pb = multi.add(ProgressBar::new(total as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[work_idx] {bar:40.yellow/white} {pos}/{len} ({per_sec})")
            .unwrap()
            .progress_chars("=> "),
    );

    // Read all indexes into memory for parallel processing
    let indexes: Vec<WorkIndexData> = JsonlIterator::new(&indexes_path)?
        .filter_map(|r| r.ok())
        .collect();

    // Process in parallel
    indexes.par_iter().for_each(|index_data| {
        match build_work_index(config, templates, index_data) {
            Ok(_) => {
                stats.indexes_built.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!(
                    "Error building work index {}/{}: {}",
                    index_data.kana_symbol, index_data.page, e
                );
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
    Ok(())
}

fn build_work_index(
    config: &Config,
    templates: &TemplateRegistry,
    data: &WorkIndexData,
) -> Result<()> {
    let ctx = work_index::build_work_index_context(data)?;

    let html = templates.render("indexes/works", ctx).with_context(|| {
        format!(
            "Failed to render work index {}/{}",
            data.kana_symbol, data.page
        )
    })?;

    let filename = work_index::work_index_filename(&data.kana_symbol, data.page);
    let output_path = config.output.directory.join("index_pages").join(&filename);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, html)?;

    Ok(())
}
