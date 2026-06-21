use anyhow::{Context, Result};
use indicatif::MultiProgress;
use std::fs;

use super::BuildStats;
use super::runner;
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

    let pb = runner::styled_bar(multi, "whatsnew", "40.red/white", all_data.len() as u64);

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

    runner::render_each(
        &all_data,
        &pb,
        stats,
        |s| &s.whatsnew_built,
        |data| {
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
        },
        |data| format!("whatsnew {:?}/{}", data.year, data.page),
    );
    Ok(())
}
