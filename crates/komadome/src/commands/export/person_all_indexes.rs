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
    copyright_flag: bool,
}

#[derive(sqlx::FromRow)]
struct PersonRow {
    id: i64,
    name: String,
    published_count: i64,
    unpublished_count: i64,
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
    ("zz", "他"),
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
    let people = if column_chars.is_empty() {
        sqlx::query_as::<_, PersonRow>(
            r#"
            SELECT p.id,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS name,
                   COUNT(DISTINCT CASE
                       WHEN w.work_status_id = 1 AND w.started_on <= $2
                       THEN w.id
                   END) AS published_count,
                   COUNT(DISTINCT CASE
                       WHEN w.work_status_id IN (3,4,5,6,7,8,9,10,11)
                            OR (w.work_status_id = 1 AND w.started_on > $2)
                       THEN w.id
                   END) AS unpublished_count,
                   p.copyright_flag
            FROM people p
            LEFT JOIN work_people wp ON wp.person_id = p.id AND wp.role_id = 1
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
        let pattern = format!("{}%", kana_char);
        sqlx::query_as::<_, PersonRow>(
            r#"
            SELECT p.id,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS name,
                   COUNT(DISTINCT CASE
                       WHEN w.work_status_id = 1 AND w.started_on <= $2
                       THEN w.id
                   END) AS published_count,
                   COUNT(DISTINCT CASE
                       WHEN w.work_status_id IN (3,4,5,6,7,8,9,10,11)
                            OR (w.work_status_id = 1 AND w.started_on > $2)
                       THEN w.id
                   END) AS unpublished_count,
                   p.copyright_flag
            FROM people p
            LEFT JOIN work_people wp ON wp.person_id = p.id AND wp.role_id = 1
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
