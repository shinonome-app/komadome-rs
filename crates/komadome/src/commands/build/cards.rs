use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::Ordering;

use super::BuildStats;
use crate::config::Config;
use crate::data::loader::JsonlIterator;
use crate::data::masters::Masters;
use crate::data::models::CardData;
use crate::generator::builder::card;
use crate::generator::templates::TemplateRegistry;

pub fn build_cards_internal(
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

    // Read all cards into memory for parallel processing.
    // first_author.id == 0 ("著者なし" placeholder) は public 公開対象から除外する。
    let cards: Vec<CardData> = JsonlIterator::new(&cards_path)?
        .filter_map(|r| r.ok())
        .filter(|c: &CardData| c.authors.first().map(|a| a.id).unwrap_or(c.person_id) != 0)
        .collect();


    // Process in parallel
    cards.par_iter().for_each(|card_data| {
        match build_card(config, masters, templates, card_data) {
            Ok(_) => {
                stats.cards_built.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                eprintln!("Error building card {}: {}", card_data.work_id, e);
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
    card_data: &CardData,
) -> Result<()> {
    let ctx = card::build_card_context(card_data, masters, &config.output.main_site_url)?;

    let html = templates
        .render("cards/show", ctx)
        .with_context(|| format!("Failed to render card {}", card_data.work_id))?;

    let output_path = config
        .output
        .directory
        .join(card_data.card_path());

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, html)?;

    Ok(())
}

pub fn build_single_card(
    config: &Config,
    masters: &Masters,
    templates: &TemplateRegistry,
    work_id: i64,
) -> Result<()> {
    let cards_path = config.data.directory.join("cards.jsonl");

    let card_data: CardData = JsonlIterator::new(&cards_path)?
        .filter_map(|r| r.ok())
        .find(|c: &CardData| c.work_id == work_id)
        .with_context(|| format!("Card not found: {work_id}"))?;

    build_card(config, masters, templates, &card_data)
}
