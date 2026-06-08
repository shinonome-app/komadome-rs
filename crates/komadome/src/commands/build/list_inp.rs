use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::Ordering;

use super::BuildStats;
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
            let html = templates.render("indexes/list_inp", ctx).with_context(|| {
                format!("Failed to render list_inp {}/{}", data.person_id, data.page)
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
