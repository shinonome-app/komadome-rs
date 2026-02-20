use anyhow::Result;
use serde::Serialize;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use crate::generator::kana::COLUMN_CHARS;

// Regex pattern for all kana characters
const KANA_PATTERN: &str = "^[あいうえおか-もやゆよら-ろわをんアイウエオカ-モヤユヨラ-ロワヲンヴ]";

#[derive(Serialize)]
struct PersonIndexData {
    kana_column: String,
    people: Vec<PersonIndexItem>,
}

#[derive(Serialize)]
struct PersonIndexItem {
    id: i64,
    name: String,
    name_kana: String,
    work_count: i64,
    copyright_flag: bool,
}

#[derive(sqlx::FromRow)]
struct PersonRow {
    id: i64,
    name: String,
    name_kana: String,
    work_count: i64,
    copyright_flag: bool,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting person_indexes.jsonl...");

    let today = chrono::Local::now().date_naive();

    let mut file = std::io::BufWriter::new(std::fs::File::create(
        output_dir.join("person_indexes.jsonl"),
    )?);
    let mut count = 0;

    for (column_key, kana_chars) in COLUMN_CHARS {
        let people = fetch_people(pool, kana_chars, today).await?;

        let data = PersonIndexData {
            kana_column: column_key.to_string(),
            people: people
                .into_iter()
                .map(|p| PersonIndexItem {
                    id: p.id,
                    name: p.name,
                    name_kana: p.name_kana,
                    work_count: p.work_count,
                    copyright_flag: p.copyright_flag,
                })
                .collect(),
        };

        serde_json::to_writer(&mut file, &data)?;
        file.write_all(b"\n")?;
        count += 1;
    }

    file.flush()?;
    println!("  -> {} index pages", count);
    Ok(count)
}

async fn fetch_people(
    pool: &PgPool,
    kana_chars: &str,
    today: chrono::NaiveDate,
) -> Result<Vec<PersonRow>> {
    let people = if !kana_chars.is_empty() {
        let pattern = format!("^[{}]", kana_chars);
        sqlx::query_as::<_, PersonRow>(
            r#"
            SELECT p.id,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS name,
                   CONCAT_WS(' ', p.last_name_kana, p.first_name_kana) AS name_kana,
                   COUNT(DISTINCT CASE
                       WHEN w.work_status_id = 1 AND w.started_on <= $2
                       THEN w.id
                   END) AS work_count,
                   p.copyright_flag
            FROM people p
            LEFT JOIN work_people wp ON wp.person_id = p.id
            LEFT JOIN works w ON w.id = wp.work_id
            WHERE p.sortkey ~ $1
            GROUP BY p.id, p.last_name, p.first_name, p.last_name_kana,
                     p.first_name_kana, p.copyright_flag, p.sortkey
            ORDER BY p.sortkey, p.id
            "#,
        )
        .bind(&pattern)
        .bind(today)
        .fetch_all(pool)
        .await?
    } else {
        // "zz" - non-kana characters
        sqlx::query_as::<_, PersonRow>(
            r#"
            SELECT p.id,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS name,
                   CONCAT_WS(' ', p.last_name_kana, p.first_name_kana) AS name_kana,
                   COUNT(DISTINCT CASE
                       WHEN w.work_status_id = 1 AND w.started_on <= $2
                       THEN w.id
                   END) AS work_count,
                   p.copyright_flag
            FROM people p
            LEFT JOIN work_people wp ON wp.person_id = p.id
            LEFT JOIN works w ON w.id = wp.work_id
            WHERE p.sortkey !~ $1
            GROUP BY p.id, p.last_name, p.first_name, p.last_name_kana,
                     p.first_name_kana, p.copyright_flag, p.sortkey
            ORDER BY p.sortkey, p.id
            "#,
        )
        .bind(KANA_PATTERN)
        .bind(today)
        .fetch_all(pool)
        .await?
    };

    Ok(people)
}
