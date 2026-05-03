use anyhow::Result;
use chrono::Datelike;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use super::export_helpers::write_jsonl_line;
use crate::data::models::{NewsData, NewsEntry};

const BEGIN_YEAR: i32 = 1997;

#[derive(sqlx::FromRow)]
struct NewsEntryRow {
    id: i64,
    title: String,
    body: String,
    published_on: Option<chrono::NaiveDate>,
    flag: bool,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting news.jsonl...");

    let current_year = Datelike::year(&chrono::Local::now().date_naive());

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(output_dir.join("news.jsonl"))?);
    let mut count = 0;

    for year in BEGIN_YEAR..=current_year {
        let entries: Vec<NewsEntryRow> = sqlx::query_as(
            r#"
            SELECT id, title, body, published_on, flag
            FROM news_entries
            WHERE extract(year FROM published_on) = $1
            ORDER BY published_on DESC, id DESC
            "#,
        )
        .bind(year)
        .fetch_all(pool)
        .await?;

        let data = NewsData {
            year,
            entries: entries
                .into_iter()
                .map(|e| NewsEntry {
                    id: e.id,
                    title: e.title,
                    body: e.body,
                    published_on: e.published_on.map(|d| d.to_string()),
                    flag: e.flag,
                })
                .collect(),
        };

        write_jsonl_line(&mut file, &data)?;
        count += 1;
    }

    file.flush()?;
    println!("  -> {count} news years");
    Ok(count)
}
