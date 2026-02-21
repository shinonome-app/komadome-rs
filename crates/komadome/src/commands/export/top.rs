use anyhow::Result;
use chrono::Datelike;
use serde::Serialize;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

#[derive(Serialize)]
struct TopData {
    new_works: Vec<NewWork>,
    new_works_published_on: Option<String>,
    latest_news_published_on: Option<String>,
    topics: Vec<TopicEntry>,
    works_count: i64,
    works_copyright_count: i64,
    works_noncopyright_count: i64,
}

#[derive(Serialize)]
struct NewWork {
    work_id: i64,
    title: String,
    subtitle: Option<String>,
    author_text: Option<String>,
    card_person_id: Option<i64>,
}

#[derive(Serialize)]
struct TopicEntry {
    id: i64,
    title: String,
    published_on: Option<String>,
    year: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct LatestDateRow {
    started_on: Option<chrono::NaiveDate>,
}

#[derive(sqlx::FromRow)]
struct NewWorkRow {
    id: i64,
    title: String,
    subtitle: Option<String>,
    started_on: Option<chrono::NaiveDate>,
}

#[derive(sqlx::FromRow)]
struct WorkPersonRow {
    work_id: i64,
    person_id: i64,
    person_name: String,
}

#[derive(sqlx::FromRow)]
struct NewsRow {
    id: i64,
    title: String,
    published_on: Option<chrono::NaiveDate>,
}

#[derive(sqlx::FromRow)]
struct CountRow {
    count: i64,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting top.json...");

    let today = chrono::Local::now().date_naive();
    let current_year = Datelike::year(&today);

    // Find the latest published date (try current year first, then previous year)
    let latest_date: Option<chrono::NaiveDate> = {
        let row: Option<LatestDateRow> = sqlx::query_as(
            r#"
            SELECT MAX(started_on) AS started_on
            FROM works
            WHERE work_status_id = 1
              AND started_on IS NOT NULL
              AND started_on <= $1
              AND extract(year FROM started_on) = $2
            "#,
        )
        .bind(today)
        .bind(current_year)
        .fetch_optional(pool)
        .await?;

        match row.and_then(|r| r.started_on) {
            Some(d) => Some(d),
            None => {
                // Try previous year
                let row: Option<LatestDateRow> = sqlx::query_as(
                    r#"
                    SELECT MAX(started_on) AS started_on
                    FROM works
                    WHERE work_status_id = 1
                      AND started_on IS NOT NULL
                      AND started_on <= $1
                      AND extract(year FROM started_on) = $2
                    "#,
                )
                .bind(today)
                .bind(current_year - 1)
                .fetch_optional(pool)
                .await?;
                row.and_then(|r| r.started_on)
            }
        }
    };

    // Fetch new works for that date
    let new_works_rows: Vec<NewWorkRow> = if let Some(date) = latest_date {
        sqlx::query_as(
            r#"
            SELECT id, title, subtitle, started_on
            FROM works
            WHERE work_status_id = 1
              AND started_on = $1
            ORDER BY id ASC
            "#,
        )
        .bind(date)
        .fetch_all(pool)
        .await?
    } else {
        vec![]
    };

    let work_ids: Vec<i64> = new_works_rows.iter().map(|w| w.id).collect();

    // Fetch authors for new works
    let authors: Vec<WorkPersonRow> = if !work_ids.is_empty() {
        sqlx::query_as(
            r#"
            SELECT wp.work_id, wp.person_id,
                   CONCAT_WS(' ', p.last_name, p.first_name) AS person_name
            FROM work_people wp
            JOIN people p ON p.id = wp.person_id
            WHERE wp.work_id = ANY($1) AND wp.role_id = 1
            ORDER BY wp.work_id, wp.id
            "#,
        )
        .bind(&work_ids)
        .fetch_all(pool)
        .await?
    } else {
        vec![]
    };

    let authors_by_work = super::db_helpers::group_by(&authors, |a| a.work_id);

    let new_works: Vec<NewWork> = new_works_rows
        .iter()
        .map(|w| {
            let empty = vec![];
            let work_authors = authors_by_work.get(&w.id).unwrap_or(&empty);
            let author_text = work_authors
                .iter()
                .map(|a| a.person_name.as_str())
                .collect::<Vec<_>>()
                .join("、");
            let card_person_id = work_authors.first().map(|a| a.person_id);

            NewWork {
                work_id: w.id,
                title: w.title.clone(),
                subtitle: w.subtitle.clone(),
                author_text: if author_text.is_empty() {
                    None
                } else {
                    Some(author_text)
                },
                card_person_id,
            }
        })
        .collect();

    // Latest news entry published_on
    let latest_news: Option<NewsRow> = sqlx::query_as(
        r#"
        SELECT id, title, published_on
        FROM news_entries
        WHERE published_on IS NOT NULL
        ORDER BY published_on DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    // Topics (flagged news entries, latest 10)
    let topic_rows: Vec<NewsRow> = sqlx::query_as(
        r#"
        SELECT id, title, published_on
        FROM news_entries
        WHERE flag = true AND published_on IS NOT NULL
        ORDER BY published_on DESC
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    let topics: Vec<TopicEntry> = topic_rows
        .into_iter()
        .map(|r| TopicEntry {
            id: r.id,
            title: r.title,
            year: r.published_on.map(|d| Datelike::year(&d)),
            published_on: r.published_on.map(|d| d.to_string()),
        })
        .collect();

    // Work counts
    let works_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM works WHERE work_status_id = 1 AND started_on <= $1",
    )
    .bind(today)
    .fetch_one(pool)
    .await?;

    // Copyright count: works where any associated person has copyright_flag = true
    let works_copyright_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT w.id)
        FROM works w
        JOIN work_people wp ON wp.work_id = w.id
        JOIN people p ON p.id = wp.person_id
        WHERE w.work_status_id = 1 AND w.started_on <= $1
          AND p.copyright_flag = true
        "#,
    )
    .bind(today)
    .fetch_one(pool)
    .await?;

    let works_noncopyright_count = works_count - works_copyright_count;

    let data = TopData {
        new_works,
        new_works_published_on: latest_date.map(|d| d.to_string()),
        latest_news_published_on: latest_news.and_then(|n| n.published_on.map(|d| d.to_string())),
        topics,
        works_count,
        works_copyright_count,
        works_noncopyright_count,
    };

    let mut file = std::io::BufWriter::new(std::fs::File::create(output_dir.join("top.json"))?);
    serde_json::to_writer_pretty(&mut file, &data)?;
    file.flush()?;

    println!("  -> top.json");
    Ok(1)
}
