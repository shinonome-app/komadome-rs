use anyhow::{Context, Result};
use indicatif::MultiProgress;
use std::fs;

use super::BuildStats;
use super::runner;
use crate::config::Config;
use crate::data::loader;
use crate::data::models::ListInpData;
use crate::generator::builder::list_inp;
use crate::generator::templates::TemplateRegistry;

pub fn build_list_inp_internal(
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

    // person_id=0 ("著者なし" placeholder) は public 公開対象から除外する。
    let all_data: Vec<ListInpData> = loader::load_jsonl::<ListInpData>(&list_path)?
        .into_iter()
        .filter(|d| d.person_id != 0)
        .collect();

    let pb = runner::styled_bar(multi, "list_inp", "40.cyan/white", all_data.len() as u64);

    let index_pages_dir = config.output.directory.join("index_pages");
    fs::create_dir_all(&index_pages_dir)?;

    runner::render_each(
        &all_data,
        &pb,
        stats,
        |s| &s.wip_built,
        |data| {
            let ctx = list_inp::build_list_inp_context(data)?;
            let html = templates.render("indexes/list_inp", ctx).with_context(|| {
                format!(
                    "Failed to render list_inp {}/{}",
                    data.person_id, data.pagination.page
                )
            })?;
            let filename = list_inp::list_inp_filename(data.person_id, data.pagination.page);
            fs::write(index_pages_dir.join(&filename), html)?;
            Ok(())
        },
        |data| format!("list_inp {}/{}", data.person_id, data.pagination.page),
    );
    Ok(())
}
