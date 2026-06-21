use anyhow::{Context, Result};
use indicatif::MultiProgress;
use std::fs;

use super::BuildStats;
use super::runner;
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

    let pb = runner::styled_bar(multi, "person_idx", "40.magenta/white", total as u64);

    // Read all indexes into memory for parallel processing
    let indexes: Vec<PersonIndexData> = JsonlIterator::new(&indexes_path)?
        .filter_map(|r| r.ok())
        .collect();

    runner::render_each(
        &indexes,
        &pb,
        stats,
        |s| &s.indexes_built,
        |index_data| build_person_index(config, templates, index_data),
        |index_data| format!("person index {}", index_data.kana_column),
    );
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
