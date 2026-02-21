use anyhow::Result;
use serde::Serialize;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use crate::generator::kana::COLUMN_CHARS;

const KANA_PATTERN: &str = "^[あいうえおか-もやゆよら-ろわをんアイウエオカ-モヤユヨラ-ロワヲンヴ]";

#[derive(Serialize)]
struct PersonAllIndexData {
    kana_column: String,
    column_display: String,
    sections: Vec<PersonAllSection>,
}

#[derive(Serialize)]
struct PersonAllSection {
    kana_char: String,
    section_index: usize,
    people: Vec<PersonAllItem>,
}

#[derive(Serialize)]
struct PersonAllItem {
    id: i64,
    name: String,
    published_count: i64,
    unpublished_count: i64,
    total_count: i64,
    copyright_flag: bool,
}

#[derive(sqlx::FromRow)]
struct PersonRow {
    id: i64,
    name: String,
    published_count: i64,
    unpublished_count: i64,
    total_count: i64,
    copyright_flag: bool,
}

const COLUMN_DISPLAY: &[(&str, &str)] = &[
    ("a", "あ"),
    ("ka", "か"),
    ("sa", "さ"),
    ("ta", "た"),
    ("na", "な"),
    ("ha", "は"),
    ("ma", "ま"),
    ("ya", "や"),
    ("ra", "ら"),
    ("wa", "わ"),
    ("zz", ""),
];

fn column_display(col: &str) -> &'static str {
    COLUMN_DISPLAY
        .iter()
        .find(|(k, _)| *k == col)
        .map(|(_, v)| *v)
        .unwrap_or("他")
}

fn column_kana_chars(kana_chars: &str) -> Vec<String> {
    if kana_chars.is_empty() {
        // For "zz" column, include "その他" section so consolidated page can use it.
        // Per-column page builder will filter this out (Rails renders no sections for zz).
        vec!["その他".to_string()]
    } else {
        kana_chars.chars().map(|c| c.to_string()).collect()
    }
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting person_all_indexes.jsonl...");

    let today = chrono::Local::now().date_naive();

    let mut file = std::io::BufWriter::new(std::fs::File::create(
        output_dir.join("person_all_indexes.jsonl"),
    )?);
    let mut count = 0;

    for (column_key, kana_chars) in COLUMN_CHARS {
        let chars = column_kana_chars(kana_chars);
        let mut sections = Vec::new();

        for (idx, kana_char) in chars.iter().enumerate() {
            let people = fetch_all_people(pool, kana_chars, kana_char, today).await?;
            let items: Vec<PersonAllItem> = people
                .into_iter()
                .map(|p| PersonAllItem {
                    id: p.id,
                    name: p.name,
                    published_count: p.published_count,
                    unpublished_count: p.unpublished_count,
                    total_count: p.total_count,
                    copyright_flag: p.copyright_flag,
                })
                .collect();

            sections.push(PersonAllSection {
                kana_char: kana_char.clone(),
                section_index: idx + 1,
                people: items,
            });
        }

        let data = PersonAllIndexData {
            kana_column: column_key.to_string(),
            column_display: column_display(column_key).to_string(),
            sections,
        };

        serde_json::to_writer(&mut file, &data)?;
        file.write_all(b"\n")?;
        count += 1;
    }

    file.flush()?;
    println!("  -> {} index pages", count);
    Ok(count)
}

async fn fetch_all_people(
    pool: &PgPool,
    column_chars: &str,
    kana_char: &str,
    today: chrono::NaiveDate,
) -> Result<Vec<PersonRow>> {
    // Use subqueries for counts to match Rails ordering behavior.
    // Rails does Person.where(...) without ORDER BY, which returns results
    // in physical/ctid order. Using a simple SELECT with correlated subqueries
    // preserves this ordering, unlike GROUP BY which uses hash aggregation.
    let people = if column_chars.is_empty() {
        sqlx::query_as::<_, PersonRow>(
            r#"
            SELECT p.id,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS name,
                   (SELECT COUNT(DISTINCT w.id)
                    FROM work_people wp
                    JOIN works w ON w.id = wp.work_id
                    WHERE wp.person_id = p.id
                      AND w.work_status_id = 1 AND w.started_on <= $2
                   ) AS published_count,
                   (SELECT COUNT(DISTINCT w.id)
                    FROM work_people wp
                    JOIN works w ON w.id = wp.work_id
                    WHERE wp.person_id = p.id
                      AND (w.work_status_id IN (3,4,5,6,7,8,9,10,11)
                           OR (w.work_status_id = 1 AND w.started_on > $2))
                   ) AS unpublished_count,
                   (SELECT COUNT(DISTINCT w.id)
                    FROM work_people wp
                    JOIN works w ON w.id = wp.work_id
                    WHERE wp.person_id = p.id
                   ) AS total_count,
                   p.copyright_flag
            FROM people p
            WHERE p.sortkey !~ $1
            "#,
        )
        .bind(KANA_PATTERN)
        .bind(today)
        .fetch_all(pool)
        .await?
    } else {
        let pattern = format!("{}%", kana_char);
        sqlx::query_as::<_, PersonRow>(
            r#"
            SELECT p.id,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS name,
                   (SELECT COUNT(DISTINCT w.id)
                    FROM work_people wp
                    JOIN works w ON w.id = wp.work_id
                    WHERE wp.person_id = p.id
                      AND w.work_status_id = 1 AND w.started_on <= $2
                   ) AS published_count,
                   (SELECT COUNT(DISTINCT w.id)
                    FROM work_people wp
                    JOIN works w ON w.id = wp.work_id
                    WHERE wp.person_id = p.id
                      AND (w.work_status_id IN (3,4,5,6,7,8,9,10,11)
                           OR (w.work_status_id = 1 AND w.started_on > $2))
                   ) AS unpublished_count,
                   (SELECT COUNT(DISTINCT w.id)
                    FROM work_people wp
                    JOIN works w ON w.id = wp.work_id
                    WHERE wp.person_id = p.id
                   ) AS total_count,
                   p.copyright_flag
            FROM people p
            WHERE p.sortkey LIKE $1
            "#,
        )
        .bind(&pattern)
        .bind(today)
        .fetch_all(pool)
        .await?
    };

    Ok(people)
}
