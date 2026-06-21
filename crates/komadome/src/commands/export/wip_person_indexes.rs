use anyhow::Result;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use super::db_helpers::{KANA_PATTERN, wip_work_predicate};
use super::export_helpers::{column_display, column_kana_chars, write_jsonl_line};
use crate::data::models::{WipPersonIndexData, WipPersonItem, WipPersonSection};
use crate::generator::kana::COLUMN_CHARS;

#[derive(sqlx::FromRow)]
struct PersonRow {
    id: i64,
    name: String,
    unpublished_count: i64,
    copyright_flag: bool,
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
            column_display: column_display(column_key),
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
    // "zz" 列はカナ以外 (!~)、通常の列は sortkey が指定文字で始まる (LIKE) ものを集める。
    let (kana_op, pattern) = if column_chars.is_empty() {
        ("!~", KANA_PATTERN.to_string())
    } else {
        ("LIKE", format!("{kana_char}%"))
    };
    let wip = wip_work_predicate("$2");
    let sql = format!(
        r#"
            SELECT p.id,
                   CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS name,
                   COUNT(DISTINCT CASE
                       WHEN {wip}
                       THEN w.id
                   END) AS unpublished_count,
                   p.copyright_flag
            FROM people p
            LEFT JOIN work_people wp ON wp.person_id = p.id
            LEFT JOIN works w ON w.id = wp.work_id
            WHERE p.sortkey {kana_op} $1
            GROUP BY p.id, p.last_name, p.first_name, p.copyright_flag, p.sortkey, p.sortkey2
            ORDER BY p.sortkey, p.sortkey2, p.id
            "#
    );
    let people = sqlx::query_as::<_, PersonRow>(&sql)
        .bind(&pattern)
        .bind(today)
        .fetch_all(pool)
        .await?;

    Ok(people)
}
