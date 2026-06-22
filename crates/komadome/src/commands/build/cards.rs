use anyhow::{Context, Result};
use indicatif::MultiProgress;
use std::fs;

use super::BuildStats;
use super::runner;
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

    let pb = runner::styled_bar(multi, "cards", "40.cyan/blue", total as u64);

    // Read all cards into memory for parallel processing.
    // first_author.id == 0 ("著者なし" placeholder) は public 公開対象から除外する。
    let cards: Vec<CardData> = JsonlIterator::new(&cards_path)?
        .filter_map(|r| r.ok())
        .filter(|c: &CardData| c.authors.first().map(|a| a.id).unwrap_or(c.person_id) != 0)
        .collect();

    runner::render_each(
        &cards,
        &pb,
        stats,
        |s| &s.cards_built,
        |card_data| build_card(config, masters, templates, card_data),
        |card_data| format!("card {}", card_data.work_id),
    );
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

    let output_path = config.output.directory.join(crate::generator::builder::card_relative_path(
        card_data.person_id,
        card_data.work_id,
    ));

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
