mod cards;
pub mod db;
pub mod db_helpers;
pub mod export_helpers;
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
use clap::ValueEnum;
use sqlx::PgPool;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::cli::{ExportArgs, ExportTarget};
use crate::config::Config;

pub async fn run(config: &Config, args: ExportArgs) -> Result<()> {
    let start = Instant::now();

    let pool = db::connect(config).await?;

    let output_dir = &config.data.directory;
    fs::create_dir_all(output_dir)?;

    println!("Exporting to {}...\n", output_dir.display());

    match args.only {
        Some(target) => export_one(target, &pool, output_dir).await?,
        // `None` => 全種別を宣言順にエクスポート。
        None => {
            for &target in ExportTarget::value_variants() {
                export_one(target, &pool, output_dir).await?;
            }
        }
    }

    pool.close().await;

    let elapsed = start.elapsed();
    println!("\nExport complete in {:.2}s", elapsed.as_secs_f64());

    Ok(())
}

/// 1種別をエクスポートする。網羅 match なので `ExportTarget` に追加すると
/// コンパイルエラーで漏れを検知できる。
async fn export_one(target: ExportTarget, pool: &PgPool, output_dir: &Path) -> Result<()> {
    match target {
        ExportTarget::Masters => masters::export(pool, output_dir).await?,
        ExportTarget::Cards => cards::export(pool, output_dir).await?,
        ExportTarget::PersonPages => person_pages::export(pool, output_dir).await?,
        ExportTarget::WorkIndexes => work_indexes::export(pool, output_dir).await?,
        ExportTarget::PersonIndexes => person_indexes::export(pool, output_dir).await?,
        ExportTarget::Whatsnew => whatsnew::export(pool, output_dir).await?,
        ExportTarget::News => news::export(pool, output_dir).await?,
        ExportTarget::Top => top::export(pool, output_dir).await?,
        ExportTarget::WipWorkIndexes => wip_work_indexes::export(pool, output_dir).await?,
        ExportTarget::WipPersonIndexes => wip_person_indexes::export(pool, output_dir).await?,
        ExportTarget::PersonAllIndexes => person_all_indexes::export(pool, output_dir).await?,
        ExportTarget::ListInp => list_inp::export(pool, output_dir).await?,
    };
    Ok(())
}
