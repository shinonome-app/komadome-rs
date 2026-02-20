use anyhow::Result;
use serde::Serialize;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use crate::generator::kana::ROMA2KANA;

const PAGE_SIZE: usize = 50;

// Regex pattern for all kana characters (same as Ruby's KANA_PATTERN)
const KANA_PATTERN: &str = "^[あいうえおか-もやゆよら-ろわをんアイウエオカ-モヤユヨラ-ロワヲンヴ]";

#[derive(Serialize)]
struct WorkIndexData {
    kana_symbol: String,
    page: usize,
    total_pages: usize,
    works: Vec<WorkIndexItem>,
}

#[derive(Serialize)]
struct WorkIndexItem {
    id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    author_name: Option<String>,
    person_id: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct WorkRow {
    id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    author_name: Option<String>,
    person_id: Option<i64>,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting work_indexes.jsonl...");

    let today = chrono::Local::now().date_naive();

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(output_dir.join("work_indexes.jsonl"))?);
    let mut count = 0;

    for (symbol, kana_char) in ROMA2KANA {
        let works = fetch_works(pool, *kana_char, today).await?;
        let total_pages = calculate_total_pages(works.len());

        for page in 1..=total_pages {
            let start_idx = (page - 1) * PAGE_SIZE;
            let page_works: Vec<WorkIndexItem> = works
                .iter()
                .skip(start_idx)
                .take(PAGE_SIZE)
                .map(|w| WorkIndexItem {
                    id: w.id,
                    title: w.title.clone(),
                    title_kana: w.title_kana.clone(),
                    subtitle: w.subtitle.clone(),
                    author_name: w.author_name.clone(),
                    person_id: w.person_id,
                })
                .collect();

            let data = WorkIndexData {
                kana_symbol: symbol.to_string(),
                page,
                total_pages,
                works: page_works,
            };

            serde_json::to_writer(&mut file, &data)?;
            file.write_all(b"\n")?;
            count += 1;
        }
    }

    file.flush()?;
    println!("  -> {} index pages", count);
    Ok(count)
}

async fn fetch_works(
    pool: &PgPool,
    kana_char: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<Vec<WorkRow>> {
    let works = if let Some(kana) = kana_char {
        // Regular kana character - match sortkey starting with this char
        let pattern = format!("^{}", kana);
        sqlx::query_as::<_, WorkRow>(
            r#"
            SELECT w.id, w.title, w.title_kana, w.subtitle,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS author_name,
                   p.id AS person_id
            FROM works w
            LEFT JOIN LATERAL (
                SELECT pe.id, pe.last_name, pe.first_name
                FROM work_people wp2
                JOIN people pe ON pe.id = wp2.person_id
                WHERE wp2.work_id = w.id AND wp2.role_id = 1
                ORDER BY wp2.id
                LIMIT 1
            ) p ON true
            WHERE w.work_status_id = 1 AND w.started_on <= $1
              AND w.sortkey ~ $2
            ORDER BY w.sortkey, w.id
            "#,
        )
        .bind(today)
        .bind(&pattern)
        .fetch_all(pool)
        .await?
    } else {
        // "zz" - non-kana characters
        sqlx::query_as::<_, WorkRow>(
            r#"
            SELECT w.id, w.title, w.title_kana, w.subtitle,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS author_name,
                   p.id AS person_id
            FROM works w
            LEFT JOIN LATERAL (
                SELECT pe.id, pe.last_name, pe.first_name
                FROM work_people wp2
                JOIN people pe ON pe.id = wp2.person_id
                WHERE wp2.work_id = w.id AND wp2.role_id = 1
                ORDER BY wp2.id
                LIMIT 1
            ) p ON true
            WHERE w.work_status_id = 1 AND w.started_on <= $1
              AND w.sortkey !~ $2
            ORDER BY w.sortkey, w.id
            "#,
        )
        .bind(today)
        .bind(KANA_PATTERN)
        .fetch_all(pool)
        .await?
    };

    Ok(works)
}

fn calculate_total_pages(total_items: usize) -> usize {
    let pages = (total_items as f64 / PAGE_SIZE as f64).ceil() as usize;
    if pages == 0 { 1 } else { pages }
}
