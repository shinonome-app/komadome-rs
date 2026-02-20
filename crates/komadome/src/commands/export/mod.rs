mod cards;
mod db;
pub mod db_helpers;
mod list_inp;
mod masters;
mod news;
mod person_all_indexes;
mod person_indexes;
mod person_pages;
mod top;
mod whatsnew;
mod wip_person_indexes;
mod wip_work_indexes;
mod work_indexes;

use anyhow::Result;
use std::fs;
use std::time::Instant;

use crate::cli::ExportArgs;
use crate::config::Config;

pub async fn run(config: &Config, args: ExportArgs) -> Result<()> {
    let start = Instant::now();

    let pool = db::connect(config).await?;

    let output_dir = &config.data.directory;
    fs::create_dir_all(output_dir)?;

    println!(
        "Exporting to {}...\n",
        output_dir.display()
    );

    match args.only.as_deref() {
        Some("masters") => {
            masters::export(&pool, output_dir).await?;
        }
        Some("cards") => {
            cards::export(&pool, output_dir).await?;
        }
        Some("person_pages") => {
            person_pages::export(&pool, output_dir).await?;
        }
        Some("work_indexes") => {
            work_indexes::export(&pool, output_dir).await?;
        }
        Some("person_indexes") => {
            person_indexes::export(&pool, output_dir).await?;
        }
        Some("whatsnew") => {
            whatsnew::export(&pool, output_dir).await?;
        }
        Some("news") => {
            news::export(&pool, output_dir).await?;
        }
        Some("top") => {
            top::export(&pool, output_dir).await?;
        }
        Some("wip_work_indexes") => {
            wip_work_indexes::export(&pool, output_dir).await?;
        }
        Some("wip_person_indexes") => {
            wip_person_indexes::export(&pool, output_dir).await?;
        }
        Some("person_all_indexes") => {
            person_all_indexes::export(&pool, output_dir).await?;
        }
        Some("list_inp") => {
            list_inp::export(&pool, output_dir).await?;
        }
        Some(other) => {
            anyhow::bail!("Unknown export type: {}", other);
        }
        None => {
            // Export all
            masters::export(&pool, output_dir).await?;
            cards::export(&pool, output_dir).await?;
            person_pages::export(&pool, output_dir).await?;
            work_indexes::export(&pool, output_dir).await?;
            person_indexes::export(&pool, output_dir).await?;
            whatsnew::export(&pool, output_dir).await?;
            news::export(&pool, output_dir).await?;
            top::export(&pool, output_dir).await?;
            wip_work_indexes::export(&pool, output_dir).await?;
            wip_person_indexes::export(&pool, output_dir).await?;
            person_all_indexes::export(&pool, output_dir).await?;
            list_inp::export(&pool, output_dir).await?;
        }
    }

    pool.close().await;

    let elapsed = start.elapsed();
    println!("\nExport complete in {:.2}s", elapsed.as_secs_f64());

    Ok(())
}
