use anyhow::{Context, Result};
use std::fs;

use crate::config::Config;
use crate::data::loader;
use crate::data::models::{PersonAllIndexData, TopPageData, WipPersonIndexData};
use crate::generator::builder::{person_all_index, static_pages, top, wip_person_index};
use crate::generator::templates::TemplateRegistry;

pub fn build_static_pages_internal(config: &Config, templates: &TemplateRegistry) -> Result<()> {
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

    // person_all.html (公開中 consolidated) と person_all_all.html (登録全作家 consolidated)
    let person_all_path = config.data.directory.join("person_all_indexes.jsonl");
    if person_all_path.exists() {
        let all_data: Vec<PersonAllIndexData> = loader::load_jsonl(&person_all_path)?;

        let ctx = person_all_index::build_person_all_consolidated_context(&all_data)?;
        let html = templates
            .render("indexes/person_all_consolidated", ctx)
            .with_context(|| "Failed to render person_all consolidated page")?;
        fs::write(index_pages_dir.join("person_all.html"), html)?;

        let ctx = person_all_index::build_person_all_all_context(&all_data)?;
        let html = templates
            .render("indexes/person_all_all", ctx)
            .with_context(|| "Failed to render person_all_all page")?;
        fs::write(index_pages_dir.join("person_all_all.html"), html)?;
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

    println!(
        "Built static pages: index_top.html, index_all.html, person_all.html, person_all_all.html, person_inp_all.html"
    );
    Ok(())
}

pub fn build_top_internal(config: &Config, templates: &TemplateRegistry) -> Result<()> {
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
