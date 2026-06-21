mod assets;
mod cards;
mod list_inp;
mod people;
mod person_indexes;
mod runner;
mod soramoyou;
mod static_pages;
mod whatsnew;
mod wip;
mod work_indexes;

use anyhow::Result;
use indicatif::MultiProgress;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::cli::{BuildArgs, CardsArgs, IndexesArgs, PeopleArgs, SoramoyouArgs, WhatsnewArgs};
use crate::config::Config;
use crate::data::masters::Masters;
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
    println!("Building all pages with {jobs} threads...\n");

    // Load masters
    let masters_path = config.data.directory.join("masters.json");
    println!("Loading masters from {}...", masters_path.display());
    let masters = Masters::load(&masters_path)?;
    println!("  Masters loaded.\n");

    // Load templates
    println!(
        "Loading templates from {}...",
        config.templates.directory.display()
    );
    let templates = TemplateRegistry::load(&config.templates.directory)?;
    println!("  {} templates loaded.\n", templates.names().count());

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    // Build cards
    cards::build_cards_internal(config, &masters, &templates, &stats, &multi)?;

    // Build people
    people::build_people_internal(config, &templates, &stats, &multi)?;

    // Build indexes
    work_indexes::build_work_indexes_internal(config, &templates, &stats, &multi)?;
    person_indexes::build_person_indexes_internal(config, &templates, &stats, &multi)?;

    // Build whatsnew
    whatsnew::build_whatsnew_internal(config, &masters, &templates, &stats, &multi)?;

    // Build soramoyou
    soramoyou::build_soramoyou_internal(config, &masters, &templates, &stats, &multi)?;

    // Build static pages (index_top, index_all)
    static_pages::build_static_pages_internal(config, &templates)?;

    // Build top page (index.html)
    static_pages::build_top_internal(config, &templates)?;

    // Build WIP pages
    wip::build_wip_work_indexes_internal(config, &templates, &stats, &multi)?;
    wip::build_wip_person_indexes_internal(config, &templates, &stats)?;
    wip::build_person_all_indexes_internal(config, &templates, &stats)?;
    list_inp::build_list_inp_internal(config, &templates, &stats, &multi)?;

    // Copy assets
    assets::copy_assets(config)?;

    // 404.html は Ruby (komadome) では生成しないため、互換性維持のためここでは呼ばない。
    // 静的配信に必要なら nginx 等のレイヤーで補う。

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
        println!("  {} errors occurred", stats.errors.load(Ordering::Relaxed));
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
        cards::build_single_card(config, &masters, &templates, work_id)?;
        println!("Built card for work {work_id}");
    } else {
        cards::build_cards_internal(config, &masters, &templates, &stats, &multi)?;
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
        people::build_single_person(config, &templates, person_id)?;
        println!("Built page for person {person_id}");
    } else {
        people::build_people_internal(config, &templates, &stats, &multi)?;
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
    println!("Building {index_type} indexes with {jobs} threads...");

    // Load templates
    let templates = TemplateRegistry::load(&config.templates.directory)?;

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    match index_type {
        "works" => {
            work_indexes::build_work_indexes_internal(config, &templates, &stats, &multi)?;
        }
        "people" => {
            person_indexes::build_person_indexes_internal(config, &templates, &stats, &multi)?;
        }
        _ => {
            work_indexes::build_work_indexes_internal(config, &templates, &stats, &multi)?;
            person_indexes::build_person_indexes_internal(config, &templates, &stats, &multi)?;
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
    println!("Building whatsnew pages with {jobs} threads...");

    // Load masters and templates
    let masters = Masters::load(&config.data.directory.join("masters.json"))?;
    let templates = TemplateRegistry::load(&config.templates.directory)?;

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    whatsnew::build_whatsnew_internal(config, &masters, &templates, &stats, &multi)?;
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
    println!("Building soramoyou pages with {jobs} threads...");

    // Load masters and templates
    let masters = Masters::load(&config.data.directory.join("masters.json"))?;
    let templates = TemplateRegistry::load(&config.templates.directory)?;

    // Prepare output directory
    fs::create_dir_all(&config.output.directory)?;

    let stats = BuildStats::default();
    let multi = MultiProgress::new();

    soramoyou::build_soramoyou_internal(config, &masters, &templates, &stats, &multi)?;
    multi.clear()?;

    let elapsed = start.elapsed();
    println!(
        "\nBuilt {} soramoyou pages in {:.2}s",
        stats.soramoyou_built.load(Ordering::Relaxed),
        elapsed.as_secs_f64()
    );

    Ok(())
}
