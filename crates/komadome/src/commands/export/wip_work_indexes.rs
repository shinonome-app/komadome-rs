use anyhow::Result;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use crate::data::models::{Pagination, WipWorkIndexData, WipWorkIndexItem};
use crate::generator::kana::ROMA2KANA;

use super::db_helpers::{KANA_PATTERN, wip_work_predicate};
use super::export_helpers::{PAGE_SIZE, calculate_total_pages, write_jsonl_line};

#[derive(sqlx::FromRow)]
struct WipWorkRow {
    id: i64,
    title: String,
    subtitle: Option<String>,
    kana_type_name: Option<String>,
    author_name: Option<String>,
    author_id: Option<i64>,
    base_author_name: Option<String>,
    translator_text: Option<String>,
    inputer_text: Option<String>,
    proofreader_text: Option<String>,
    work_status_name: Option<String>,
    started_on: Option<chrono::NaiveDate>,
    teihon_title: Option<String>,
    teihon_publisher: Option<String>,
    teihon_input_edition: Option<String>,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting wip_work_indexes.jsonl...");

    let today = crate::clock::build_date();

    let mut file = std::io::BufWriter::new(std::fs::File::create(
        output_dir.join("wip_work_indexes.jsonl"),
    )?);
    let mut count = 0;

    for (symbol, kana_char) in ROMA2KANA {
        let works = fetch_wip_works(pool, *kana_char, today).await?;
        let total_pages = calculate_total_pages(works.len());

        for page in 1..=total_pages {
            let start_idx = (page - 1) * PAGE_SIZE;
            let page_works: Vec<WipWorkIndexItem> = works
                .iter()
                .skip(start_idx)
                .take(PAGE_SIZE)
                .map(|w| WipWorkIndexItem {
                    id: w.id,
                    title: w.title.clone(),
                    subtitle: w.subtitle.clone(),
                    kana_type_name: w.kana_type_name.clone(),
                    author_name: w.author_name.clone(),
                    author_id: w.author_id,
                    base_author_name: w.base_author_name.clone(),
                    translator_text: w.translator_text.clone(),
                    inputer_text: w.inputer_text.clone(),
                    proofreader_text: w.proofreader_text.clone(),
                    work_status_name: w.work_status_name.clone(),
                    started_on: w.started_on.map(|d| d.to_string()),
                    teihon_title: w.teihon_title.clone(),
                    teihon_publisher: w.teihon_publisher.clone(),
                    teihon_input_edition: w.teihon_input_edition.clone(),
                })
                .collect();

            let data = WipWorkIndexData {
                kana_symbol: symbol.to_string(),
                pagination: Pagination { page, total_pages },
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

async fn fetch_wip_works(
    pool: &PgPool,
    kana_char: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<Vec<WipWorkRow>> {
    // 通常のカナ列は sortkey 先頭一致 (~)、"zz" 列はカナ以外 (!~) を集める。
    let (kana_op, pattern) = match kana_char {
        Some(kana) => ("~", format!("^{kana}")),
        None => ("!~", KANA_PATTERN.to_string()),
    };
    let wip = wip_work_predicate("$1");
    let sql = format!(
        r#"
            SELECT w.id, w.title, w.subtitle,
                   kt.name AS kana_type_name,
                   CONCAT(COALESCE(author_p.last_name, ''), ' ', COALESCE(author_p.first_name, '')) AS author_name,
                   author_p.id AS author_id,
                   CASE WHEN orig_p.id IS NULL THEN NULL
                        ELSE CONCAT(COALESCE(orig_p.last_name, ''), ' ', COALESCE(orig_p.first_name, ''))
                   END AS base_author_name,
                   translators.translator_text,
                   inputers.inputer_text,
                   proofreaders.proofreader_text,
                   ws.name AS work_status_name,
                   w.started_on,
                   teihon.title AS teihon_title,
                   teihon.publisher AS teihon_publisher,
                   teihon.input_edition AS teihon_input_edition
            FROM works w
            LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
            LEFT JOIN work_statuses ws ON ws.id = w.work_status_id
            LEFT JOIN LATERAL (
                SELECT pe.id, pe.last_name, pe.first_name
                FROM work_people wp2
                JOIN people pe ON pe.id = wp2.person_id
                WHERE wp2.work_id = w.id AND wp2.role_id = 1
                ORDER BY wp2.id
                LIMIT 1
            ) author_p ON true
            LEFT JOIN base_people bp ON bp.person_id = author_p.id
            LEFT JOIN people orig_p ON orig_p.id = bp.original_person_id
            LEFT JOIN LATERAL (
                SELECT string_agg(CONCAT(COALESCE(pe.last_name, ''), ' ', COALESCE(pe.first_name, '')), ', ') AS translator_text
                FROM work_people wp2
                JOIN people pe ON pe.id = wp2.person_id
                WHERE wp2.work_id = w.id AND wp2.role_id = 2
            ) translators ON true
            LEFT JOIN LATERAL (
                SELECT string_agg(wk.name, '、') AS inputer_text
                FROM work_workers ww
                JOIN workers wk ON wk.id = ww.worker_id
                WHERE ww.work_id = w.id AND ww.worker_role_id = 1
            ) inputers ON true
            LEFT JOIN LATERAL (
                SELECT string_agg(wk.name, '、') AS proofreader_text
                FROM work_workers ww
                JOIN workers wk ON wk.id = ww.worker_id
                WHERE ww.work_id = w.id AND ww.worker_role_id = 2
            ) proofreaders ON true
            LEFT JOIN LATERAL (
                SELECT ob.title, ob.publisher, ob.input_edition
                FROM original_books ob
                WHERE ob.work_id = w.id AND ob.booktype_id = 1
                ORDER BY ob.id
                LIMIT 1
            ) teihon ON true
            WHERE {wip}
              AND w.sortkey {kana_op} $2
            ORDER BY w.sortkey, w.id
            "#
    );
    let works = sqlx::query_as::<_, WipWorkRow>(&sql)
        .bind(today)
        .bind(&pattern)
        .fetch_all(pool)
        .await?;

    Ok(works)
}
