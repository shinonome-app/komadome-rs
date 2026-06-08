use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fs;
use std::sync::atomic::Ordering;

use super::BuildStats;
use crate::config::Config;
use crate::data::loader;
use crate::data::masters::Masters;
use crate::data::models::WhatsnewData;
use crate::generator::builder::whatsnew;
use crate::generator::templates::TemplateRegistry;

pub fn build_whatsnew_internal(
    config: &Config,
    masters: &Masters,
    templates: &TemplateRegistry,
    stats: &BuildStats,
    multi: &MultiProgress,
) -> Result<()> {
    let whatsnew_path = config.data.directory.join("whatsnew.jsonl");
    if !whatsnew_path.exists() {
        println!("whatsnew.jsonl not found, skipping whatsnew generation");
        return Ok(());
    }

    let all_data: Vec<WhatsnewData> = loader::load_jsonl(&whatsnew_path)?;

    let pb = multi.add(ProgressBar::new(all_data.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[whatsnew] {bar:40.red/white} {pos}/{len} ({per_sec})")
            .unwrap()
            .progress_chars("=> "),
    );

    // Collect all past years for year_links
    let year_links: Vec<i32> = {
        let mut years: Vec<i32> = all_data
            .iter()
            .filter_map(|d| d.year)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        years.sort();
        years
    };

    // 「最終更新日」は JSONL export 時刻 (= DB を読み出した時点) を表す。
    // build を再実行しても export し直さない限り表示が変わらない (idempotent)。
    // Ruby (komadome) は build 時刻を使うため、再 export しないと両者の表示日付は
    // ズレることがあるが、Rust 側の意味は data freshness なので意図的にこちらを優先する。
    let today = masters.exported_date();
    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    for data in &all_data {
        let result: Result<()> = (|| {
            if data.year.is_none() {
                // Current year -> index template
                let ctx = whatsnew::build_whatsnew_index_context(data, &today, &year_links)?;
                let html = templates.render("whatsnew/index", ctx).with_context(|| {
                    format!("Failed to render whatsnew index page {}", data.page)
                })?;
                let filename = whatsnew::whatsnew_index_filename(data.page);
                fs::write(index_pages_dir.join(&filename), html)?;
            } else {
                // Past year -> year template
                let year = data.year.unwrap();
                let ctx = whatsnew::build_whatsnew_year_context(data, &today)?;
                let html = templates.render("whatsnew/year", ctx).with_context(|| {
                    format!("Failed to render whatsnew year page {}/{}", year, data.page)
                })?;
                let filename = whatsnew::whatsnew_year_filename(year, data.page);
                fs::write(index_pages_dir.join(&filename), html)?;
            }
            Ok(())
        })();

        match result {
            Ok(_) => {
                stats.whatsnew_built.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!(
                    "Error building whatsnew {:?}/{}: {}",
                    data.year, data.page, e
                );
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message("done");
    Ok(())
}
