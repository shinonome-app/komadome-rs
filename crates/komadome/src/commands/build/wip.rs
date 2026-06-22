use anyhow::{Context, Result};
use indicatif::MultiProgress;
use std::fs;

use super::BuildStats;
use super::runner;
use crate::config::Config;
use crate::data::loader;
use crate::data::models::{PersonAllIndexData, WipPersonIndexData, WipWorkIndexData};
use crate::generator::builder::{person_all_index, wip_person_index, wip_work_index};
use crate::generator::templates::TemplateRegistry;

pub fn build_wip_work_indexes_internal(
    config: &Config,
    templates: &TemplateRegistry,
    stats: &BuildStats,
    multi: &MultiProgress,
) -> Result<()> {
    let indexes_path = config.data.directory.join("wip_work_indexes.jsonl");
    if !indexes_path.exists() {
        println!("wip_work_indexes.jsonl not found, skipping WIP work index generation");
        return Ok(());
    }

    let all_data: Vec<WipWorkIndexData> = loader::load_jsonl(&indexes_path)?;

    let pb = runner::styled_bar(multi, "wip_work", "40.yellow/white", all_data.len() as u64);

    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    runner::render_each(
        &all_data,
        &pb,
        stats,
        |s| &s.wip_built,
        |data| {
            let ctx = wip_work_index::build_wip_work_index_context(data)?;
            let html = templates
                .render("indexes/wip_works", ctx)
                .with_context(|| {
                    format!(
                        "Failed to render WIP work index {}/{}",
                        data.kana_symbol, data.pagination.page
                    )
                })?;
            let filename =
                wip_work_index::wip_work_index_filename(&data.kana_symbol, data.pagination.page);
            fs::write(index_pages_dir.join(&filename), html)?;
            Ok(())
        },
        |data| {
            format!(
                "WIP work index {}/{}",
                data.kana_symbol, data.pagination.page
            )
        },
    );
    Ok(())
}

pub fn build_wip_person_indexes_internal(
    config: &Config,
    templates: &TemplateRegistry,
    stats: &BuildStats,
) -> Result<()> {
    let indexes_path = config.data.directory.join("wip_person_indexes.jsonl");
    if !indexes_path.exists() {
        println!("wip_person_indexes.jsonl not found, skipping WIP person index generation");
        return Ok(());
    }

    let all_data: Vec<WipPersonIndexData> = loader::load_jsonl(&indexes_path)?;

    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    let built = runner::render_each_seq(
        &all_data,
        stats,
        |s| &s.wip_built,
        |data| {
            let ctx = wip_person_index::build_wip_person_index_context(data)?;
            let html = templates
                .render("indexes/wip_people", ctx)
                .with_context(|| {
                    format!("Failed to render WIP person index {}", data.kana_column)
                })?;
            let filename = wip_person_index::wip_person_index_filename(&data.kana_column);
            fs::write(index_pages_dir.join(&filename), html)?;
            Ok(())
        },
        |data| format!("WIP person index {}", data.kana_column),
    );

    println!("Built {built} WIP person index pages");
    Ok(())
}

pub fn build_person_all_indexes_internal(
    config: &Config,
    templates: &TemplateRegistry,
    stats: &BuildStats,
) -> Result<()> {
    let indexes_path = config.data.directory.join("person_all_indexes.jsonl");
    if !indexes_path.exists() {
        println!("person_all_indexes.jsonl not found, skipping person_all index generation");
        return Ok(());
    }

    let all_data: Vec<PersonAllIndexData> = loader::load_jsonl(&indexes_path)?;

    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    let built = runner::render_each_seq(
        &all_data,
        stats,
        |s| &s.wip_built,
        |data| {
            let ctx = person_all_index::build_person_all_index_context(data)?;
            let html = templates
                .render("indexes/person_all_index", ctx)
                .with_context(|| {
                    format!("Failed to render person_all index {}", data.kana_column)
                })?;
            let filename = person_all_index::person_all_index_filename(&data.kana_column);
            fs::write(index_pages_dir.join(&filename), html)?;
            Ok(())
        },
        |data| format!("person_all index {}", data.kana_column),
    );

    println!("Built {built} person_all index pages");
    Ok(())
}
