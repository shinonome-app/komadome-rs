use anyhow::Result;
use chrono::Datelike;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use super::db_helpers;

const FIRST_YEAR: i32 = 2001;
const PAGE_SIZE: usize = 50;

#[derive(Serialize)]
struct WhatsnewData {
    year: Option<i32>,
    page: usize,
    total_pages: usize,
    entries: Vec<WhatsnewEntry>,
}

#[derive(Serialize)]
struct WhatsnewEntry {
    work_id: i64,
    title: String,
    subtitle: Option<String>,
    card_person_id: Option<i64>,
    author_text: Option<String>,
    inputer_text: Option<String>,
    proofreader_text: Option<String>,
    translator_text: Option<String>,
    started_on: Option<String>,
}

#[derive(sqlx::FromRow)]
struct WorkRow {
    id: i64,
    title: String,
    subtitle: Option<String>,
    started_on: Option<chrono::NaiveDate>,
}

#[derive(sqlx::FromRow)]
struct WorkPersonRow {
    work_id: i64,
    role_id: i64,
    person_name: String,
}

#[derive(sqlx::FromRow)]
struct WorkWorkerRow {
    work_id: i64,
    worker_role_id: i64,
    worker_name: Option<String>,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting whatsnew.jsonl...");

    let today = chrono::Local::now().date_naive();
    let current_year = Datelike::year(&today);

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(output_dir.join("whatsnew.jsonl"))?);
    let mut count = 0;

    // Current year (year=null in JSON)
    let entries = build_year_entries(pool, current_year, today).await?;
    count += write_paginated(&mut file, &entries, None)?;

    // Past years
    for year in FIRST_YEAR..(current_year) {
        let entries = build_year_entries(pool, year, today).await?;
        count += write_paginated(&mut file, &entries, Some(year))?;
    }

    file.flush()?;
    println!("  -> {} whatsnew pages", count);
    Ok(count)
}

async fn build_year_entries(
    pool: &PgPool,
    year: i32,
    until_date: chrono::NaiveDate,
) -> Result<Vec<WhatsnewEntry>> {
    // Fetch works for this year
    let works: Vec<WorkRow> = sqlx::query_as(
        r#"
        SELECT id, title, subtitle, started_on
        FROM works
        WHERE work_status_id = 1
          AND started_on IS NOT NULL
          AND extract(year FROM started_on) = $1
          AND started_on <= $2
        ORDER BY started_on DESC, id ASC
        "#,
    )
    .bind(year)
    .bind(until_date)
    .fetch_all(pool)
    .await?;

    if works.is_empty() {
        return Ok(vec![]);
    }

    let work_ids: Vec<i64> = works.iter().map(|w| w.id).collect();

    // Fetch work_people (authors role_id=1, translators role_id=2)
    let work_people: Vec<WorkPersonRow> = sqlx::query_as(
        r#"
        SELECT wp.work_id, wp.role_id,
               CONCAT_WS(' ', p.last_name, p.first_name) AS person_name
        FROM work_people wp
        JOIN people p ON p.id = wp.person_id
        WHERE wp.work_id = ANY($1)
        ORDER BY wp.work_id, wp.role_id, wp.person_id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    // Fetch first author person_id for card path
    let card_person_ids: HashMap<i64, i64> = {
        let rows: Vec<(i64, i64)> = sqlx::query_as(
            r#"
            SELECT DISTINCT ON (wp.work_id) wp.work_id, wp.person_id
            FROM work_people wp
            WHERE wp.work_id = ANY($1) AND wp.role_id = 1
            ORDER BY wp.work_id, wp.id
            "#,
        )
        .bind(&work_ids)
        .fetch_all(pool)
        .await?;
        rows.into_iter().collect()
    };

    // Fetch work_workers (inputers worker_role_id=1, proofreaders worker_role_id=2)
    let work_workers: Vec<WorkWorkerRow> = sqlx::query_as(
        r#"
        SELECT ww.work_id, ww.worker_role_id,
               w.name AS worker_name
        FROM work_workers ww
        LEFT JOIN workers w ON w.id = ww.worker_id
        WHERE ww.work_id = ANY($1)
        ORDER BY ww.work_id, ww.worker_role_id, ww.id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    let people_by_work = db_helpers::group_by(&work_people, |wp| wp.work_id);
    let workers_by_work = db_helpers::group_by(&work_workers, |ww| ww.work_id);

    let entries = works
        .iter()
        .map(|work| {
            let empty_people = vec![];
            let people = people_by_work.get(&work.id).unwrap_or(&empty_people);

            let author_text = join_names(people.iter().filter(|p| p.role_id == 1), ", ");
            let translator_text = join_names(people.iter().filter(|p| p.role_id == 2), ", ");

            let empty_workers = vec![];
            let workers = workers_by_work.get(&work.id).unwrap_or(&empty_workers);

            let inputer_text =
                join_worker_names(workers.iter().filter(|w| w.worker_role_id == 1), "、");
            let proofreader_text =
                join_worker_names(workers.iter().filter(|w| w.worker_role_id == 2), "、");

            WhatsnewEntry {
                work_id: work.id,
                title: work.title.clone(),
                subtitle: work.subtitle.clone(),
                card_person_id: card_person_ids.get(&work.id).copied(),
                author_text: non_empty(author_text),
                inputer_text: non_empty(inputer_text),
                proofreader_text: non_empty(proofreader_text),
                translator_text: non_empty(translator_text),
                started_on: work.started_on.map(|d| d.to_string()),
            }
        })
        .collect();

    Ok(entries)
}

fn join_names<'a>(people: impl Iterator<Item = &'a &'a WorkPersonRow>, sep: &str) -> String {
    people
        .map(|p| p.person_name.as_str())
        .collect::<Vec<_>>()
        .join(sep)
}

fn join_worker_names<'a>(
    workers: impl Iterator<Item = &'a &'a WorkWorkerRow>,
    sep: &str,
) -> String {
    workers
        .filter_map(|w| w.worker_name.as_deref())
        .collect::<Vec<_>>()
        .join(sep)
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn write_paginated(
    file: &mut std::io::BufWriter<std::fs::File>,
    entries: &[WhatsnewEntry],
    year: Option<i32>,
) -> Result<usize> {
    let total_pages = {
        let pages = (entries.len() as f64 / PAGE_SIZE as f64).ceil() as usize;
        if pages == 0 { 1 } else { pages }
    };

    let mut count = 0;

    for page in 1..=total_pages {
        let start_idx = (page - 1) * PAGE_SIZE;
        let page_entries: Vec<&WhatsnewEntry> =
            entries.iter().skip(start_idx).take(PAGE_SIZE).collect();

        let data = WhatsnewPageData {
            year,
            page,
            total_pages,
            entries: &page_entries,
        };

        serde_json::to_writer(&mut *file, &data)?;
        file.write_all(b"\n")?;
        count += 1;
    }

    Ok(count)
}

#[derive(Serialize)]
struct WhatsnewPageData<'a> {
    year: Option<i32>,
    page: usize,
    total_pages: usize,
    entries: &'a Vec<&'a WhatsnewEntry>,
}
