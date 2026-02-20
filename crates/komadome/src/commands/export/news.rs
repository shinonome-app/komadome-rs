use anyhow::Result;
use chrono::Datelike;
use serde::Serialize;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

const BEGIN_YEAR: i32 = 1997;

#[derive(Serialize)]
struct NewsData {
    year: i32,
    entries: Vec<NewsEntryData>,
}

#[derive(Serialize)]
struct NewsEntryData {
    id: i64,
    title: String,
    body: String,
    published_on: Option<String>,
    flag: bool,
}

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
                .map(|e| NewsEntryData {
                    id: e.id,
                    title: e.title,
                    body: e.body,
                    published_on: e.published_on.map(|d| d.to_string()),
                    flag: e.flag,
                })
                .collect(),
        };

        serde_json::to_writer(&mut file, &data)?;
        file.write_all(b"\n")?;
        count += 1;
    }

    file.flush()?;
    println!("  -> {} news years", count);
    Ok(count)
}
