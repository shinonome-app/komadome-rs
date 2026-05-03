//! `list_person_all.zip` / `list_person_all_utf8.zip` の生成。
//!
//! Ruby `CsvCreator#write_finished` (csv_creator.rb:73-109) と同じ列構成・並び順を再現する。
//! 公開作品 (`work_status_id = 1 AND started_on <= today`) を `(work, person)` 単位で展開。

use anyhow::Result;
use sqlx::PgPool;
use std::path::Path;

use super::{headers, make_csv_writer, write_header, write_pair};

#[derive(sqlx::FromRow)]
struct Row {
    person_id: i64,
    person_name: String,
    work_id: i64,
    work_title: String,
    kana_type_name: Option<String>,
    not_author_text: Option<String>,
    inputer_text: Option<String>,
    proofreader_text: Option<String>,
    work_status_name: Option<String>,
    started_on: Option<chrono::NaiveDate>,
    teihon_title: Option<String>,
    teihon_publisher: Option<String>,
    teihon_input_edition: Option<String>,
    teihon_proof_edition: Option<String>,
}

pub async fn generate(pool: &PgPool, zip_dir: &Path, today: chrono::NaiveDate) -> Result<()> {
    println!("[finished_basic] querying...");
    let rows: Vec<Row> = sqlx::query_as(
        r#"
        SELECT
            p.id AS person_id,
            CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS person_name,
            w.id AS work_id,
            w.title AS work_title,
            kt.name AS kana_type_name,
            COALESCE(
                (SELECT string_agg(
                    CONCAT(COALESCE(p2.last_name, ''), ' ', COALESCE(p2.first_name, '')),
                    '、' ORDER BY wp2.id
                 )
                 FROM work_people wp2
                 JOIN people p2 ON p2.id = wp2.person_id
                 WHERE wp2.work_id = w.id AND wp2.role_id <> 1),
                ''
            ) AS not_author_text,
            COALESCE(
                (SELECT string_agg(wkr.name, '、' ORDER BY ww.id)
                 FROM work_workers ww
                 JOIN workers wkr ON wkr.id = ww.worker_id
                 WHERE ww.work_id = w.id AND ww.worker_role_id = 1),
                ''
            ) AS inputer_text,
            COALESCE(
                (SELECT string_agg(wkr.name, '、' ORDER BY ww.id)
                 FROM work_workers ww
                 JOIN workers wkr ON wkr.id = ww.worker_id
                 WHERE ww.work_id = w.id AND ww.worker_role_id = 2),
                ''
            ) AS proofreader_text,
            ws.name AS work_status_name,
            w.started_on,
            ob.title AS teihon_title,
            ob.publisher AS teihon_publisher,
            ob.input_edition AS teihon_input_edition,
            ob.proof_edition AS teihon_proof_edition
        FROM works w
        JOIN work_statuses ws ON ws.id = w.work_status_id
        LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
        JOIN work_people wp ON wp.work_id = w.id
        JOIN people p ON p.id = wp.person_id
        LEFT JOIN LATERAL (
            SELECT title, publisher, input_edition, proof_edition
            FROM original_books
            WHERE work_id = w.id AND booktype_id = 1
            ORDER BY id
            LIMIT 1
        ) ob ON TRUE
        WHERE w.work_status_id = 1 AND w.started_on <= $1
        -- Ruby: order(:sortkey, :sortkey2, :id, 'people.sortkey')
        -- works has no sortkey2; AR が unqualified にして Postgres が people.sortkey2 に解決する。
        ORDER BY w.sortkey, p.sortkey2, w.id, p.sortkey
        "#,
    )
    .bind(today)
    .fetch_all(pool)
    .await?;

    println!("  -> {} rows", rows.len());

    let mut buf = Vec::with_capacity(rows.len() * 200);
    write_header(&mut buf, headers::FINISHED_BASIC)?;
    {
        let mut w = make_csv_writer(&mut buf);
        for r in &rows {
            let person_id = r.person_id.to_string();
            let work_id = r.work_id.to_string();
            let started_on = r.started_on.map(|d| d.to_string()).unwrap_or_default();
            w.write_record([
                person_id.as_str(),
                r.person_name.as_str(),
                work_id.as_str(),
                r.work_title.as_str(),
                r.kana_type_name.as_deref().unwrap_or(""),
                r.not_author_text.as_deref().unwrap_or(""),
                r.inputer_text.as_deref().unwrap_or(""),
                r.proofreader_text.as_deref().unwrap_or(""),
                r.work_status_name.as_deref().unwrap_or(""),
                started_on.as_str(),
                r.teihon_title.as_deref().unwrap_or(""),
                r.teihon_publisher.as_deref().unwrap_or(""),
                r.teihon_input_edition.as_deref().unwrap_or(""),
                r.teihon_proof_edition.as_deref().unwrap_or(""),
            ])?;
        }
        w.flush()?;
    }

    write_pair(zip_dir, "list_person_all", &buf)?;
    Ok(())
}
