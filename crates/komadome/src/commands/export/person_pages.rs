use anyhow::Result;
use serde::Serialize;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use super::db_helpers;

#[derive(Serialize)]
struct PersonPageData {
    person: PersonData,
    works: Vec<PersonWorkInfo>,
    sites: Vec<SiteInfo>,
}

#[derive(Serialize)]
struct PersonData {
    id: i64,
    last_name: String,
    first_name: Option<String>,
    last_name_kana: String,
    first_name_kana: Option<String>,
    born_on: Option<String>,
    died_on: Option<String>,
    copyright_flag: bool,
    description: Option<String>,
}

#[derive(Serialize)]
struct PersonWorkInfo {
    id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    role: Option<String>,
    role_id: i64,
    kana_type: Option<String>,
}

#[derive(Serialize)]
struct SiteInfo {
    name: Option<String>,
    url: Option<String>,
}

// DB row types
#[derive(sqlx::FromRow)]
struct PersonRow {
    id: i64,
    last_name: String,
    first_name: Option<String>,
    last_name_kana: String,
    first_name_kana: Option<String>,
    born_on: Option<String>,
    died_on: Option<String>,
    copyright_flag: bool,
    description: Option<String>,
}

#[derive(sqlx::FromRow)]
struct PersonWorkRow {
    person_id: i64,
    work_id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    role_name: Option<String>,
    role_id: i64,
    kana_type_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct PersonSiteRow {
    person_id: i64,
    site_name: Option<String>,
    site_url: Option<String>,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting person_pages.jsonl...");

    let today = chrono::Local::now().date_naive();

    // Fetch all people
    let people: Vec<PersonRow> = sqlx::query_as(
        r#"
        SELECT id, last_name, first_name, last_name_kana, first_name_kana,
               born_on, died_on, copyright_flag, description
        FROM people
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await?;

    let person_ids: Vec<i64> = people.iter().map(|p| p.id).collect();

    // Fetch published works for all people
    let work_rows: Vec<PersonWorkRow> = sqlx::query_as(
        r#"
        SELECT wp.person_id, w.id AS work_id, w.title, w.title_kana, w.subtitle,
               r.name AS role_name, wp.role_id,
               kt.name AS kana_type_name
        FROM work_people wp
        JOIN works w ON w.id = wp.work_id
        LEFT JOIN roles r ON r.id = wp.role_id
        LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
        WHERE wp.person_id = ANY($1)
          AND w.work_status_id = 1 AND w.started_on <= $2
        ORDER BY wp.person_id, w.id
        "#,
    )
    .bind(&person_ids)
    .bind(today)
    .fetch_all(pool)
    .await?;

    // Fetch person sites
    let site_rows: Vec<PersonSiteRow> = sqlx::query_as(
        r#"
        SELECT ps.person_id, s.name AS site_name, s.url AS site_url
        FROM person_sites ps
        JOIN sites s ON s.id = ps.site_id
        WHERE ps.person_id = ANY($1)
        ORDER BY ps.person_id, ps.id
        "#,
    )
    .bind(&person_ids)
    .fetch_all(pool)
    .await?;

    // Group by person_id
    let works_by_person = db_helpers::group_by(&work_rows, |wr| wr.person_id);
    let sites_by_person = db_helpers::group_by(&site_rows, |sr| sr.person_id);

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(output_dir.join("person_pages.jsonl"))?);
    let mut count = 0;

    for person in &people {
        let empty_works = vec![];
        let works: Vec<PersonWorkInfo> = works_by_person
            .get(&person.id)
            .unwrap_or(&empty_works)
            .iter()
            .map(|wr| PersonWorkInfo {
                id: wr.work_id,
                title: wr.title.clone(),
                title_kana: wr.title_kana.clone(),
                subtitle: wr.subtitle.clone(),
                role: wr.role_name.clone(),
                role_id: wr.role_id,
                kana_type: wr.kana_type_name.clone(),
            })
            .collect();

        let empty_sites = vec![];
        let sites: Vec<SiteInfo> = sites_by_person
            .get(&person.id)
            .unwrap_or(&empty_sites)
            .iter()
            .map(|sr| SiteInfo {
                name: sr.site_name.clone(),
                url: sr.site_url.clone(),
            })
            .collect();

        let data = PersonPageData {
            person: PersonData {
                id: person.id,
                last_name: person.last_name.clone(),
                first_name: person.first_name.clone(),
                last_name_kana: person.last_name_kana.clone(),
                first_name_kana: person.first_name_kana.clone(),
                born_on: person.born_on.clone(),
                died_on: person.died_on.clone(),
                copyright_flag: person.copyright_flag,
                description: person.description.clone(),
            },
            works,
            sites,
        };

        serde_json::to_writer(&mut file, &data)?;
        file.write_all(b"\n")?;
        count += 1;
    }

    file.flush()?;
    println!("  -> {} people", count);
    Ok(count)
}
