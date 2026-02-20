use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::cli::{BuildArgs, CardsArgs, IndexesArgs, PeopleArgs, SoramoyouArgs, WhatsnewArgs};
use crate::config::Config;
use crate::data::loader::{self, JsonlIterator};
use crate::data::masters::Masters;
use crate::data::models::{
    CardData, ListInpData, NewsData, PersonAllIndexData, PersonIndexData, PersonPageData,
    TopPageData, WhatsnewData, WipPersonIndexData, WipWorkIndexData, WorkIndexData,
};
use crate::generator::builder::{
    card, list_inp, person, person_all_index, person_index, soramoyou, static_pages, top, whatsnew,
    wip_person_index, wip_work_index, work_index,
};
use crate::generator::templates::TemplateRegistry;

/// Build statistics
#[derive(Debug, Default)]
pub struct BuildStats {
    pub cards_built: AtomicUsize,
    pub people_built: AtomicUsize,
    pub indexes_built: AtomicUsize,
    pub whatsnew_built: AtomicUsize,
    pub soramoyou_built: AtomicUsize,
    pub wip_built: AtomicUsize,
    pub errors: AtomicUsize,
}

impl BuildStats {
    pub fn total(&self) -> usize {
        self.cards_built.load(Ordering::Relaxed)
            + self.people_built.load(Ordering::Relaxed)
            + self.indexes_built.load(Ordering::Relaxed)
            + self.whatsnew_built.load(Ordering::Relaxed)
            + self.soramoyou_built.load(Ordering::Relaxed)
            + self.wip_built.load(Ordering::Relaxed)
    }
}

pub fn run(config: &Config, args: BuildArgs) -> Result<()> {
    let jobs = args.jobs.unwrap_or(config.build.default_jobs);
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()
        .ok();

    let start = Instant::now();
    println!("Building all pages with {} threads...\n", jobs);

    // Load masters
    let masters_path = config.data.directory.join("masters.json");
    println!("Loading masters from {}...", masters_path.display());
    let masters = Masters::load(&masters_path)?;
    println!("  Masters loaded.\n");

    // Load templates
    println!("Loading templates from {}...", config.templates.directory.display());
    let templates = TemplateRegistry::load(&config.templates.directory)?;
    println!("  {} templates loaded.\n", templates.names().count());

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    // Build cards
    build_cards_internal(config, &masters, &templates, &stats, &multi)?;

    // Build people
    build_people_internal(config, &templates, &stats, &multi)?;

    // Build indexes
    build_work_indexes_internal(config, &templates, &stats, &multi)?;
    build_person_indexes_internal(config, &templates, &stats, &multi)?;

    // Build whatsnew
    build_whatsnew_internal(config, &templates, &stats, &multi)?;

    // Build soramoyou
    build_soramoyou_internal(config, &templates, &stats, &multi)?;

    // Build static pages (index_top, index_all)
    build_static_pages_internal(config, &templates)?;

    // Build top page (index.html)
    build_top_internal(config, &templates)?;

    // Build WIP pages
    build_wip_work_indexes_internal(config, &templates, &stats, &multi)?;
    build_wip_person_indexes_internal(config, &templates, &stats)?;
    build_person_all_indexes_internal(config, &templates, &stats)?;
    build_list_inp_internal(config, &templates, &stats, &multi)?;

    // Copy assets
    copy_assets(config)?;

    // Generate 404 page
    build_404_page(config)?;

    multi.clear()?;

    let elapsed = start.elapsed();
    println!("\n========================================");
    println!(
        "Build complete! {} pages in {:.2}s ({:.0} pages/sec)",
        stats.total(),
        elapsed.as_secs_f64(),
        stats.total() as f64 / elapsed.as_secs_f64()
    );

    if stats.errors.load(Ordering::Relaxed) > 0 {
        println!(
            "  {} errors occurred",
            stats.errors.load(Ordering::Relaxed)
        );
    }

    Ok(())
}

pub fn run_cards(config: &Config, args: CardsArgs) -> Result<()> {
    let jobs = args.jobs.unwrap_or(config.build.default_jobs);
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()
        .ok();

    let start = Instant::now();

    // Load masters
    let masters = Masters::load(&config.data.directory.join("masters.json"))?;

    // Load templates
    let templates = TemplateRegistry::load(&config.templates.directory)?;

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    if let Some(work_id) = args.work_id {
        build_single_card(config, &masters, &templates, work_id)?;
        println!("Built card for work {}", work_id);
    } else {
        build_cards_internal(config, &masters, &templates, &stats, &multi)?;
        multi.clear()?;

        let elapsed = start.elapsed();
        println!(
            "\nBuilt {} cards in {:.2}s",
            stats.cards_built.load(Ordering::Relaxed),
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}

pub fn run_people(config: &Config, args: PeopleArgs) -> Result<()> {
    let jobs = args.jobs.unwrap_or(config.build.default_jobs);
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()
        .ok();

    let start = Instant::now();

    // Load templates
    let templates = TemplateRegistry::load(&config.templates.directory)?;

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    if let Some(person_id) = args.person_id {
        build_single_person(config, &templates, person_id)?;
        println!("Built page for person {}", person_id);
    } else {
        build_people_internal(config, &templates, &stats, &multi)?;
        multi.clear()?;

        let elapsed = start.elapsed();
        println!(
            "\nBuilt {} person pages in {:.2}s",
            stats.people_built.load(Ordering::Relaxed),
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}

pub fn run_indexes(config: &Config, args: IndexesArgs) -> Result<()> {
    let jobs = args.jobs.unwrap_or(config.build.default_jobs);
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()
        .ok();

    let start = Instant::now();
    let index_type = args.r#type.as_deref().unwrap_or("all");
    println!("Building {} indexes with {} threads...", index_type, jobs);

    // Load templates
    let templates = TemplateRegistry::load(&config.templates.directory)?;

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    match index_type {
        "works" => {
            build_work_indexes_internal(config, &templates, &stats, &multi)?;
        }
        "people" => {
            build_person_indexes_internal(config, &templates, &stats, &multi)?;
        }
        _ => {
            build_work_indexes_internal(config, &templates, &stats, &multi)?;
            build_person_indexes_internal(config, &templates, &stats, &multi)?;
        }
    }

    multi.clear()?;

    let elapsed = start.elapsed();
    println!(
        "\nBuilt {} index pages in {:.2}s",
        stats.indexes_built.load(Ordering::Relaxed),
        elapsed.as_secs_f64()
    );

    Ok(())
}

pub fn run_whatsnew(config: &Config, args: WhatsnewArgs) -> Result<()> {
    let jobs = args.jobs.unwrap_or(config.build.default_jobs);
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()
        .ok();

    let start = Instant::now();
    println!("Building whatsnew pages with {} threads...", jobs);

    // Load templates
    let templates = TemplateRegistry::load(&config.templates.directory)?;

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    build_whatsnew_internal(config, &templates, &stats, &multi)?;
    multi.clear()?;

    let elapsed = start.elapsed();
    println!(
        "\nBuilt {} whatsnew pages in {:.2}s",
        stats.whatsnew_built.load(Ordering::Relaxed),
        elapsed.as_secs_f64()
    );

    Ok(())
}

pub fn run_soramoyou(config: &Config, args: SoramoyouArgs) -> Result<()> {
    let jobs = args.jobs.unwrap_or(config.build.default_jobs);
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()
        .ok();

    let start = Instant::now();
    println!("Building soramoyou pages with {} threads...", jobs);

    // Load templates
    let templates = TemplateRegistry::load(&config.templates.directory)?;

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    build_soramoyou_internal(config, &templates, &stats, &multi)?;
    multi.clear()?;

    let elapsed = start.elapsed();
    println!(
        "\nBuilt {} soramoyou pages in {:.2}s",
        stats.soramoyou_built.load(Ordering::Relaxed),
        elapsed.as_secs_f64()
    );

    Ok(())
}

fn build_cards_internal(
    config: &Config,
    masters: &Masters,
    templates: &TemplateRegistry,
    stats: &BuildStats,
    multi: &MultiProgress,
) -> Result<()> {
    let cards_path = config.data.directory.join("cards.jsonl");
    if !cards_path.exists() {
        println!("cards.jsonl not found, skipping card generation");
        return Ok(());
    }

    // Count lines for progress bar
    let total = crate::data::loader::count_jsonl_lines(&cards_path)?;

    let pb = multi.add(ProgressBar::new(total as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[cards] {bar:40.cyan/blue} {pos}/{len} ({per_sec})")
            .unwrap()
            .progress_chars("=> "),
    );

    // Read all cards into memory for parallel processing
    let cards: Vec<CardData> = JsonlIterator::new(&cards_path)?
        .filter_map(|r| r.ok())
        .collect();

    // Process in parallel
    cards.par_iter().for_each(|card| {
        match build_card(config, masters, templates, card) {
            Ok(_) => {
                stats.cards_built.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!("Error building card {}: {}", card.work_id, e);
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
    Ok(())
}

fn build_people_internal(
    config: &Config,
    templates: &TemplateRegistry,
    stats: &BuildStats,
    multi: &MultiProgress,
) -> Result<()> {
    let people_path = config.data.directory.join("person_pages.jsonl");
    if !people_path.exists() {
        println!("person_pages.jsonl not found, skipping person generation");
        return Ok(());
    }

    // Count lines for progress bar
    let total = crate::data::loader::count_jsonl_lines(&people_path)?;

    let pb = multi.add(ProgressBar::new(total as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[people] {bar:40.green/white} {pos}/{len} ({per_sec})")
            .unwrap()
            .progress_chars("=> "),
    );

    // Read all people into memory for parallel processing
    let people: Vec<PersonPageData> = JsonlIterator::new(&people_path)?
        .filter_map(|r| r.ok())
        .collect();

    // Process in parallel
    people.par_iter().for_each(|person_data| {
        match build_person(config, templates, person_data) {
            Ok(_) => {
                stats.people_built.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!("Error building person {}: {}", person_data.person.id, e);
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
    Ok(())
}

fn build_card(
    config: &Config,
    masters: &Masters,
    templates: &TemplateRegistry,
    card: &CardData,
) -> Result<()> {
    let ctx = card::build_card_context(card, masters)?;

    let html = templates
        .render("cards/show", ctx)
        .with_context(|| format!("Failed to render card {}", card.work_id))?;

    let output_path = config
        .output
        .directory
        .join(card.card_path());

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, html)?;

    Ok(())
}

fn build_single_card(
    config: &Config,
    masters: &Masters,
    templates: &TemplateRegistry,
    work_id: i64,
) -> Result<()> {
    let cards_path = config.data.directory.join("cards.jsonl");

    let card: CardData = JsonlIterator::new(&cards_path)?
        .filter_map(|r| r.ok())
        .find(|c: &CardData| c.work_id == work_id)
        .with_context(|| format!("Card not found: {}", work_id))?;

    build_card(config, masters, templates, &card)
}

fn build_person(
    config: &Config,
    templates: &TemplateRegistry,
    data: &PersonPageData,
) -> Result<()> {
    let ctx = person::build_person_context(data)?;

    let html = templates
        .render("people/show", ctx)
        .with_context(|| format!("Failed to render person {}", data.person.id))?;

    let output_path = config
        .output
        .directory
        .join(format!("index_pages/person{}.html", data.person.id));

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, html)?;

    Ok(())
}

fn build_single_person(
    config: &Config,
    templates: &TemplateRegistry,
    person_id: i64,
) -> Result<()> {
    let people_path = config.data.directory.join("person_pages.jsonl");

    let person_data: PersonPageData = JsonlIterator::new(&people_path)?
        .filter_map(|r| r.ok())
        .find(|p: &PersonPageData| p.person.id == person_id)
        .with_context(|| format!("Person not found: {}", person_id))?;

    build_person(config, templates, &person_data)
}

fn build_work_indexes_internal(
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

    let html = templates
        .render("indexes/works", ctx)
        .with_context(|| {
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

fn build_person_indexes_internal(
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
                eprintln!("Error building person index {}: {}", index_data.kana_column, e);
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

fn build_whatsnew_internal(
    config: &Config,
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

    let today = chrono::Local::now().date_naive();
    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    for data in &all_data {
        let result: Result<()> = (|| {
            if data.year.is_none() {
                // Current year -> index template
                let ctx = whatsnew::build_whatsnew_index_context(data, &today, &year_links)?;
                let html = templates
                    .render("whatsnew/index", ctx)
                    .with_context(|| format!("Failed to render whatsnew index page {}", data.page))?;
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

fn build_soramoyou_internal(
    config: &Config,
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

    let current_year = chrono::Datelike::year(&chrono::Local::now().date_naive());
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

fn build_static_pages_internal(
    config: &Config,
    templates: &TemplateRegistry,
) -> Result<()> {
    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    // index_top.html
    let ctx = static_pages::build_index_top_context()?;
    let html = templates
        .render("indexes/top", ctx)
        .with_context(|| "Failed to render index_top page")?;
    fs::write(index_pages_dir.join("index_top.html"), html)?;

    // index_all.html
    let ctx = static_pages::build_index_all_context()?;
    let html = templates
        .render("indexes/all", ctx)
        .with_context(|| "Failed to render index_all page")?;
    fs::write(index_pages_dir.join("index_all.html"), html)?;

    // person_all.html (consolidated all-authors page)
    let person_all_path = config.data.directory.join("person_all_indexes.jsonl");
    if person_all_path.exists() {
        let all_data: Vec<PersonAllIndexData> = loader::load_jsonl(&person_all_path)?;
        let ctx = person_all_index::build_person_all_consolidated_context(&all_data)?;
        let html = templates
            .render("indexes/person_all_consolidated", ctx)
            .with_context(|| "Failed to render person_all consolidated page")?;
        fs::write(index_pages_dir.join("person_all.html"), html)?;
    }

    // person_inp_all.html (consolidated WIP all-authors page)
    let wip_person_path = config.data.directory.join("wip_person_indexes.jsonl");
    if wip_person_path.exists() {
        let all_data: Vec<WipPersonIndexData> = loader::load_jsonl(&wip_person_path)?;
        let ctx = wip_person_index::build_wip_person_consolidated_context(&all_data)?;
        let html = templates
            .render("indexes/person_inp_all_consolidated", ctx)
            .with_context(|| "Failed to render person_inp_all consolidated page")?;
        fs::write(index_pages_dir.join("person_inp_all.html"), html)?;
    }

    println!("Built static pages: index_top.html, index_all.html, person_all.html, person_inp_all.html");
    Ok(())
}

fn build_top_internal(
    config: &Config,
    templates: &TemplateRegistry,
) -> Result<()> {
    let top_path = config.data.directory.join("top.json");
    if !top_path.exists() {
        println!("top.json not found, skipping top page generation");
        return Ok(());
    }

    let data: TopPageData = {
        let file = std::fs::File::open(&top_path)?;
        serde_json::from_reader(std::io::BufReader::new(file))?
    };

    let ctx = top::build_top_context(&data)?;
    let html = templates
        .render("top/index", ctx)
        .with_context(|| "Failed to render top page")?;

    let output_path = config.output.directory.join("index.html");
    fs::write(&output_path, html)?;

    println!("Built top page: index.html");
    Ok(())
}

fn build_wip_work_indexes_internal(
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
                    "Error building WIP work index {}/{}: {}",
                    data.kana_symbol, data.page, e
                );
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
    Ok(())
}

fn build_wip_person_indexes_internal(
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
                    format!(
                        "Failed to render WIP person index {}",
                        data.kana_column
                    )
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
                eprintln!("Error building WIP person index {}: {}", data.kana_column, e);
            }
        }
    }

    println!("Built {} WIP person index pages", built);
    Ok(())
}

fn build_person_all_indexes_internal(
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
                    format!(
                        "Failed to render person_all index {}",
                        data.kana_column
                    )
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
                eprintln!("Error building person_all index {}: {}", data.kana_column, e);
            }
        }
    }

    println!("Built {} person_all index pages", built);
    Ok(())
}

fn build_list_inp_internal(
    config: &Config,
    templates: &TemplateRegistry,
    stats: &BuildStats,
    multi: &MultiProgress,
) -> Result<()> {
    let list_path = config.data.directory.join("list_inp.jsonl");
    if !list_path.exists() {
        println!("list_inp.jsonl not found, skipping list_inp generation");
        return Ok(());
    }

    let all_data: Vec<ListInpData> = loader::load_jsonl(&list_path)?;

    let pb = multi.add(ProgressBar::new(all_data.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[list_inp] {bar:40.cyan/white} {pos}/{len} ({per_sec})")
            .unwrap()
            .progress_chars("=> "),
    );

    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    all_data.par_iter().for_each(|data| {
        let result = (|| -> Result<()> {
            let ctx = list_inp::build_list_inp_context(data)?;
            let html = templates
                .render("indexes/list_inp", ctx)
                .with_context(|| {
                    format!(
                        "Failed to render list_inp {}/{}",
                        data.person_id, data.page
                    )
                })?;
            let filename = list_inp::list_inp_filename(data.person_id, data.page);
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
                    "Error building list_inp {}/{}: {}",
                    data.person_id, data.page, e
                );
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
    Ok(())
}

fn copy_assets(config: &Config) -> Result<()> {
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

fn build_404_page(config: &Config) -> Result<()> {
    let html = include_str!("../../static/404.html");
    let output_path = config.output.directory.join("404.html");
    fs::write(&output_path, html)?;
    println!("Built 404 page: 404.html");
    Ok(())
}
