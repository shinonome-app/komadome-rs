use anyhow::Result;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use crate::data::models::{ListInpData, ListInpWorkItem};

use super::db_helpers::{group_by, wip_work_predicate};
use super::export_helpers::{PAGE_SIZE, calculate_total_pages, write_jsonl_line};

#[derive(sqlx::FromRow)]
struct PersonWithCountRow {
    id: i64,
    name: String,
    #[allow(dead_code)]
    work_count: i64,
}

#[derive(sqlx::FromRow)]
struct WorkRow {
    person_id: i64,
    id: i64,
    title: String,
    subtitle: Option<String>,
    kana_type_name: Option<String>,
    translator_text: Option<String>,
    inputer_text: Option<String>,
    proofreader_text: Option<String>,
    work_status_name: Option<String>,
    started_on: Option<chrono::NaiveDate>,
    teihon_title: Option<String>,
    teihon_publisher: Option<String>,
    teihon_input_edition: Option<String>,
    #[allow(dead_code)]
    sortkey: Option<String>,
    #[allow(dead_code)]
    subtitle_kana: Option<String>,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting list_inp.jsonl...");

    let today = crate::clock::build_date();

    // Find all persons who have unpublished works (any role)
    let wip = wip_work_predicate("$1");
    let persons_sql = format!(
        r#"
        SELECT p.id,
               CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS name,
               COUNT(DISTINCT w.id) AS work_count
        FROM people p
        JOIN work_people wp ON wp.person_id = p.id
        JOIN works w ON w.id = wp.work_id
        WHERE {wip}
        GROUP BY p.id, p.last_name, p.first_name
        HAVING COUNT(DISTINCT w.id) > 0
        ORDER BY p.id
        "#
    );
    let persons: Vec<PersonWithCountRow> = sqlx::query_as(&persons_sql)
        .bind(today)
        .fetch_all(pool)
        .await?;

    // Fetch every person's unpublished works in a single batched query (avoids N+1),
    // then group in memory by person_id. Rows are ordered by person_id first so each
    // person's works keep their sortkey order within the grouped Vec.
    let person_ids: Vec<i64> = persons.iter().map(|p| p.id).collect();
    let all_works = fetch_persons_wip_works(pool, &person_ids, today).await?;
    let works_by_person = group_by(&all_works, |w| w.person_id);
    let empty: Vec<&WorkRow> = Vec::new();

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(output_dir.join("list_inp.jsonl"))?);
    let mut count = 0;

    for person in &persons {
        let works = works_by_person.get(&person.id).unwrap_or(&empty);
        let total_pages = calculate_total_pages(works.len());

        for page in 1..=total_pages {
            let start_idx = (page - 1) * PAGE_SIZE;
            let page_works: Vec<ListInpWorkItem> = works
                .iter()
                .skip(start_idx)
                .take(PAGE_SIZE)
                .map(|w| ListInpWorkItem {
                    id: w.id,
                    title: w.title.clone(),
                    subtitle: w.subtitle.clone(),
                    kana_type_name: w.kana_type_name.clone(),
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

            let data = ListInpData {
                person_id: person.id,
                person_name: person.name.clone(),
                page,
                total_pages,
                works: page_works,
            };

            write_jsonl_line(&mut file, &data)?;
            count += 1;
        }
    }

    file.flush()?;
    println!("  -> {} list pages for {} persons", count, persons.len());
    Ok(count)
}

async fn fetch_persons_wip_works(
    pool: &PgPool,
    person_ids: &[i64],
    today: chrono::NaiveDate,
) -> Result<Vec<WorkRow>> {
    let wip = wip_work_predicate("$2");
    let sql = format!(
        r#"
        SELECT DISTINCT wp.person_id, w.id, w.title, w.subtitle,
               kt.name AS kana_type_name,
               translators.translator_text,
               inputers.inputer_text,
               proofreaders.proofreader_text,
               ws.name AS work_status_name,
               w.started_on,
               teihon.title AS teihon_title,
               teihon.publisher AS teihon_publisher,
               teihon.input_edition AS teihon_input_edition,
               w.sortkey, w.subtitle_kana
        FROM works w
        JOIN work_people wp ON wp.work_id = w.id AND wp.person_id = ANY($1)
        LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
        LEFT JOIN work_statuses ws ON ws.id = w.work_status_id
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
        ORDER BY wp.person_id, w.sortkey, w.subtitle_kana, w.id
        "#
    );
    let works = sqlx::query_as::<_, WorkRow>(&sql)
        .bind(person_ids)
        .bind(today)
        .fetch_all(pool)
        .await?;

    Ok(works)
}
