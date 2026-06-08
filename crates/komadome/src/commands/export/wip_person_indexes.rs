use anyhow::Result;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use super::export_helpers::write_jsonl_line;
use crate::data::models::{WipPersonIndexData, WipPersonItem, WipPersonSection};
use crate::generator::kana::COLUMN_CHARS;

const KANA_PATTERN: &str = "^[あいうえおか-もやゆよら-ろわをんアイウエオカ-モヤユヨラ-ロワヲンヴ]";

#[derive(sqlx::FromRow)]
struct PersonRow {
    id: i64,
    name: String,
    unpublished_count: i64,
    copyright_flag: bool,
}

/// Column display names (first kana in column)
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

/// Get individual kana characters for a column
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
    println!("Exporting wip_person_indexes.jsonl...");

    let today = crate::clock::build_date();

    let mut file = std::io::BufWriter::new(std::fs::File::create(
        output_dir.join("wip_person_indexes.jsonl"),
    )?);
    let mut count = 0;

    for (column_key, kana_chars) in COLUMN_CHARS {
        let chars = column_kana_chars(kana_chars);
        let mut sections = Vec::new();

        for (idx, kana_char) in chars.iter().enumerate() {
            let people = fetch_wip_people(pool, kana_chars, kana_char, today).await?;
            // Only include people with unpublished works
            let filtered: Vec<WipPersonItem> = people
                .into_iter()
                .filter(|p| p.unpublished_count > 0)
                .map(|p| WipPersonItem {
                    id: p.id,
                    name: p.name,
                    unpublished_count: p.unpublished_count,
                    copyright_flag: p.copyright_flag,
                })
                .collect();

            sections.push(WipPersonSection {
                kana_char: kana_char.clone(),
                section_index: idx + 1,
                people: filtered,
            });
        }

        let data = WipPersonIndexData {
            kana_column: column_key.to_string(),
            column_display: column_display(column_key).to_string(),
            sections,
        };

        write_jsonl_line(&mut file, &data)?;
        count += 1;
    }

    file.flush()?;
    println!("  -> {count} index pages");
    Ok(count)
}

async fn fetch_wip_people(
    pool: &PgPool,
    column_chars: &str,
    kana_char: &str,
    today: chrono::NaiveDate,
) -> Result<Vec<PersonRow>> {
    let people = if column_chars.is_empty() {
        // "zz" - non-kana characters
        sqlx::query_as::<_, PersonRow>(
            r#"
            SELECT p.id,
                   CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS name,
                   COUNT(DISTINCT CASE
                       WHEN w.work_status_id IN (3,4,5,6,7,8,9,10,11)
                            OR (w.work_status_id = 1 AND w.started_on > $2)
                       THEN w.id
                   END) AS unpublished_count,
                   p.copyright_flag
            FROM people p
            LEFT JOIN work_people wp ON wp.person_id = p.id
            LEFT JOIN works w ON w.id = wp.work_id
            WHERE p.sortkey !~ $1
            GROUP BY p.id, p.last_name, p.first_name, p.copyright_flag, p.sortkey, p.sortkey2
            ORDER BY p.sortkey, p.sortkey2, p.id
            "#,
        )
        .bind(KANA_PATTERN)
        .bind(today)
        .fetch_all(pool)
        .await?
    } else {
        // Match sortkey starting with specific kana character
        let pattern = format!("{kana_char}%");
        sqlx::query_as::<_, PersonRow>(
            r#"
            SELECT p.id,
                   CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS name,
                   COUNT(DISTINCT CASE
                       WHEN w.work_status_id IN (3,4,5,6,7,8,9,10,11)
                            OR (w.work_status_id = 1 AND w.started_on > $2)
                       THEN w.id
                   END) AS unpublished_count,
                   p.copyright_flag
            FROM people p
            LEFT JOIN work_people wp ON wp.person_id = p.id
            LEFT JOIN works w ON w.id = wp.work_id
            WHERE p.sortkey LIKE $1
            GROUP BY p.id, p.last_name, p.first_name, p.copyright_flag, p.sortkey, p.sortkey2
            ORDER BY p.sortkey, p.sortkey2, p.id
            "#,
        )
        .bind(&pattern)
        .bind(today)
        .fetch_all(pool)
        .await?
    };

    Ok(people)
}
