use anyhow::{Context, Result};
use indicatif::MultiProgress;
use std::fs;

use super::BuildStats;
use super::runner;
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

    let pb = runner::styled_bar(multi, "work_idx", "40.yellow/white", total as u64);

    // Read all indexes into memory for parallel processing
    let indexes: Vec<WorkIndexData> = JsonlIterator::new(&indexes_path)?
        .filter_map(|r| r.ok())
        .collect();

    runner::render_each(
        &indexes,
        &pb,
        stats,
        |s| &s.indexes_built,
        |index_data| build_work_index(config, templates, index_data),
        |index_data| format!("work index {}/{}", index_data.kana_symbol, index_data.page),
    );
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
