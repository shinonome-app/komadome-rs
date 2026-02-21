use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::Ordering;

use super::BuildStats;
use crate::config::Config;
use crate::data::loader::JsonlIterator;
use crate::data::models::PersonPageData;
use crate::generator::builder::person;
use crate::generator::templates::TemplateRegistry;

pub fn build_people_internal(
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

pub fn build_single_person(
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
