use anyhow::Result;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use crate::data::models::{WorkIndexData, WorkIndexItem};
use crate::generator::kana::ROMA2KANA;

use super::db_helpers::{KANA_PATTERN, published_work_predicate};
use super::export_helpers::{PAGE_SIZE, calculate_total_pages, write_jsonl_line};

#[derive(sqlx::FromRow)]
struct WorkRow {
    id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    author_name: Option<String>,
    person_id: Option<i64>,
    card_person_id: Option<String>,
    kana_type: Option<String>,
    author_text: Option<String>,
    base_author_text: String,
    translator_text: String,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting work_indexes.jsonl...");

    let today = crate::clock::build_date();

    let mut file = std::io::BufWriter::new(std::fs::File::create(
        output_dir.join("work_indexes.jsonl"),
    )?);
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
                    card_person_id: w.card_person_id.clone(),
                    kana_type: w.kana_type.clone(),
                    author_text: w.author_text.clone(),
                    base_author_text: Some(w.base_author_text.clone()),
                    translator_text: Some(w.translator_text.clone()),
                })
                .collect();

            let data = WorkIndexData {
                kana_symbol: symbol.to_string(),
                page,
                total_pages,
                works: page_works,
            };

            write_jsonl_line(&mut file, &data)?;
            count += 1;
        }
    }

    file.flush()?;
    println!("  -> {count} index pages");
    Ok(count)
}

async fn fetch_works(
    pool: &PgPool,
    kana_char: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<Vec<WorkRow>> {
    // 通常のカナ列は sortkey がその文字で始まるものに一致 (~)、
    // "zz" 列はカナ以外 (KANA_PATTERN に一致しない = !~) を集める。
    let (kana_op, pattern) = match kana_char {
        Some(kana) => ("~", format!("^{kana}")),
        None => ("!~", KANA_PATTERN.to_string()),
    };
    let published = published_work_predicate("$1");
    let sql = format!(
        r#"
            SELECT w.id, w.title, w.title_kana, w.subtitle,
                   CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS author_name,
                   p.id AS person_id,
                   CASE WHEN p.id IS NOT NULL THEN LPAD(p.id::text, 6, '0') END AS card_person_id,
                   kt.name AS kana_type,
                   (SELECT string_agg(CONCAT(COALESCE(pe2.last_name, ''), ' ', COALESCE(pe2.first_name, '')), ', ' ORDER BY wp2.id)
                    FROM work_people wp2
                    JOIN people pe2 ON pe2.id = wp2.person_id
                    WHERE wp2.work_id = w.id AND wp2.role_id = 1) AS author_text,
                   COALESCE((SELECT string_agg(CONCAT(COALESCE(op.last_name, ''), ' ', COALESCE(op.first_name, '')), ', ' ORDER BY wp3.id)
                    FROM work_people wp3
                    JOIN people pe3 ON pe3.id = wp3.person_id
                    JOIN base_people bp ON bp.person_id = pe3.id
                    JOIN people op ON op.id = bp.original_person_id
                    WHERE wp3.work_id = w.id AND wp3.role_id = 1), '') AS base_author_text,
                   COALESCE((SELECT string_agg(CONCAT(COALESCE(pe4.last_name, ''), ' ', COALESCE(pe4.first_name, '')), ', ' ORDER BY wp4.id)
                    FROM work_people wp4
                    JOIN people pe4 ON pe4.id = wp4.person_id
                    WHERE wp4.work_id = w.id AND wp4.role_id = 2), '') AS translator_text
            FROM works w
            LEFT JOIN LATERAL (
                SELECT pe.id, pe.last_name, pe.first_name
                FROM work_people wp2
                JOIN people pe ON pe.id = wp2.person_id
                WHERE wp2.work_id = w.id AND wp2.role_id = 1
                ORDER BY wp2.id
                LIMIT 1
            ) p ON true
            LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
            WHERE {published}
              AND w.sortkey {kana_op} $2
            ORDER BY w.sortkey, w.id
            "#
    );
    let works = sqlx::query_as::<_, WorkRow>(&sql)
        .bind(today)
        .bind(&pattern)
        .fetch_all(pool)
        .await?;

    Ok(works)
}
