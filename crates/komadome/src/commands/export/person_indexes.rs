use anyhow::Result;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use super::db_helpers::KANA_PATTERN;
use super::export_helpers::{column_display, write_jsonl_line};
use crate::data::models::{PersonIndexData, PersonIndexItem, PersonIndexSection};
use crate::generator::kana::COLUMN_CHARS;

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

    let today = crate::clock::build_date();

    let mut file = std::io::BufWriter::new(std::fs::File::create(
        output_dir.join("person_indexes.jsonl"),
    )?);
    let mut count = 0;

    for (column_key, kana_chars) in COLUMN_CHARS {
        // "zz"(その他) 列は空表示 (Rails の Kana.new(:zz).to_chars が [] を返すのに対応)。
        let column_display = column_display(column_key);

        let sections = if kana_chars.is_empty() {
            // For "zz" column, create one section with kana_char "他"
            let people = fetch_non_kana_people(pool, today).await?;
            vec![PersonIndexSection {
                kana_char: "他".to_string(),
                section_index: 1,
                people: people
                    .into_iter()
                    .map(|p| PersonIndexItem {
                        id: p.id,
                        name: p.name,
                        name_kana: p.name_kana,
                        published_works_count: p.work_count as usize,
                        work_count: p.work_count as usize,
                        copyright_flag: p.copyright_flag,
                    })
                    .collect(),
            }]
        } else {
            // For regular columns, create one section per kana character
            let mut sections = Vec::new();
            for (idx, kana_char) in kana_chars.chars().enumerate() {
                let pattern = format!("{kana_char}%");
                let people = fetch_kana_people(pool, &pattern, today).await?;
                sections.push(PersonIndexSection {
                    kana_char: kana_char.to_string(),
                    section_index: idx + 1,
                    people: people
                        .into_iter()
                        .map(|p| PersonIndexItem {
                            id: p.id,
                            name: p.name,
                            name_kana: p.name_kana,
                            published_works_count: p.work_count as usize,
                            work_count: p.work_count as usize,
                            copyright_flag: p.copyright_flag,
                        })
                        .collect(),
                });
            }
            sections
        };

        let data = PersonIndexData {
            kana_column: column_key.to_string(),
            column_display,
            sections,
        };

        write_jsonl_line(&mut file, &data)?;
        count += 1;
    }

    file.flush()?;
    println!("  -> {count} index pages");
    Ok(count)
}

async fn fetch_kana_people(
    pool: &PgPool,
    pattern: &str,
    today: chrono::NaiveDate,
) -> Result<Vec<PersonRow>> {
    let people = sqlx::query_as::<_, PersonRow>(
        r#"
        SELECT p.id,
               CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS name,
               CONCAT(COALESCE(p.last_name_kana, ''), ' ', COALESCE(p.first_name_kana, '')) AS name_kana,
               COUNT(DISTINCT CASE
                   WHEN w.work_status_id = 1 AND w.started_on <= $2
                   THEN w.id
               END) AS work_count,
               p.copyright_flag
        FROM people p
        LEFT JOIN work_people wp ON wp.person_id = p.id
        LEFT JOIN works w ON w.id = wp.work_id
        WHERE p.sortkey LIKE $1
        GROUP BY p.id, p.last_name, p.first_name, p.last_name_kana,
                 p.first_name_kana, p.copyright_flag, p.sortkey, p.sortkey2
        ORDER BY p.sortkey, p.sortkey2, p.id
        "#,
    )
    .bind(pattern)
    .bind(today)
    .fetch_all(pool)
    .await?;

    Ok(people)
}

async fn fetch_non_kana_people(pool: &PgPool, today: chrono::NaiveDate) -> Result<Vec<PersonRow>> {
    let people = sqlx::query_as::<_, PersonRow>(
        r#"
        SELECT p.id,
               CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS name,
               CONCAT(COALESCE(p.last_name_kana, ''), ' ', COALESCE(p.first_name_kana, '')) AS name_kana,
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
                 p.first_name_kana, p.copyright_flag, p.sortkey, p.sortkey2
        ORDER BY p.sortkey, p.sortkey2, p.id
        "#,
    )
    .bind(KANA_PATTERN)
    .bind(today)
    .fetch_all(pool)
    .await?;

    Ok(people)
}
