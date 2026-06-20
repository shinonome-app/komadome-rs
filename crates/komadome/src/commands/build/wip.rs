use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::Ordering;

use super::BuildStats;
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

    let pb = multi.add(ProgressBar::new(all_data.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[wip_work] {bar:40.yellow/white} {pos}/{len} ({per_sec})")
            .unwrap()
            .progress_chars("=> "),
    );

    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    all_data.par_iter().for_each(|data| {
        let result = (|| -> Result<()> {
            let ctx = wip_work_index::build_wip_work_index_context(data)?;
            let html = templates
                .render("indexes/wip_works", ctx)
                .with_context(|| {
                    format!(
                        "Failed to render WIP work index {}/{}",
                        data.kana_symbol, data.page
                    )
                })?;
            let filename = wip_work_index::wip_work_index_filename(&data.kana_symbol, data.page);
            fs::write(index_pages_dir.join(&filename), html)?;
            Ok(())
        })();

        match result {
            Ok(_) => {
                stats.wip_built.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!(
                    "Error building WIP work index {}/{}: {:#}",
                    data.kana_symbol, data.page, e
                );
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
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

    let mut built = 0;
    for data in &all_data {
        let result: Result<()> = (|| {
            let ctx = wip_person_index::build_wip_person_index_context(data)?;
            let html = templates
                .render("indexes/wip_people", ctx)
                .with_context(|| {
                    format!("Failed to render WIP person index {}", data.kana_column)
                })?;
            let filename = wip_person_index::wip_person_index_filename(&data.kana_column);
            fs::write(index_pages_dir.join(&filename), html)?;
            Ok(())
        })();
        match result {
            Ok(_) => {
                stats.wip_built.fetch_add(1, Ordering::Relaxed);
                built += 1;
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!(
                    "Error building WIP person index {}: {}",
                    data.kana_column, e
                );
            }
        }
    }

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

    let mut built = 0;
    for data in &all_data {
        let result: Result<()> = (|| {
            let ctx = person_all_index::build_person_all_index_context(data)?;
            let html = templates
                .render("indexes/person_all_index", ctx)
                .with_context(|| {
                    format!("Failed to render person_all index {}", data.kana_column)
                })?;
            let filename = person_all_index::person_all_index_filename(&data.kana_column);
            fs::write(index_pages_dir.join(&filename), html)?;
            Ok(())
        })();
        match result {
            Ok(_) => {
                stats.wip_built.fetch_add(1, Ordering::Relaxed);
                built += 1;
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!(
                    "Error building person_all index {}: {}",
                    data.kana_column, e
                );
            }
        }
    }

    println!("Built {built} person_all index pages");
    Ok(())
}
