use anyhow::Result;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use super::db_helpers;
use super::export_helpers::write_jsonl_line;
use crate::data::models::{
    OtherBasePerson, Person, PersonPageData, PersonWorkInfo, SiteInfo, WorkPersonRef,
};

// DB row types
#[derive(sqlx::FromRow)]
struct PersonRow {
    id: i64,
    last_name: String,
    first_name: Option<String>,
    last_name_kana: String,
    first_name_kana: Option<String>,
    name_en: Option<String>,
    born_on: Option<String>,
    died_on: Option<String>,
    copyright_flag: bool,
    description: Option<String>,
    sortkey: Option<String>,
}

#[derive(sqlx::FromRow)]
struct PersonWorkRow {
    person_id: i64,
    work_id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    sortkey: Option<String>,
    subtitle_kana: Option<String>,
    role_name: Option<String>,
    role_id: i64,
    kana_type_name: Option<String>,
    card_person_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct WorkPersonRow {
    work_id: i64,
    person_id: i64,
    name: String,
    role_name: Option<String>,
    #[allow(dead_code)]
    role_id: i64,
}

#[derive(sqlx::FromRow)]
struct PersonSiteRow {
    person_id: i64,
    site_name: Option<String>,
    site_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct OtherBasePersonRow {
    person_id: i64,
    original_person_id: i64,
    name: String,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting person_pages.jsonl...");

    let today = crate::clock::build_date();

    // Fetch all people
    let people: Vec<PersonRow> = sqlx::query_as(
        r#"
        SELECT id, last_name, first_name, last_name_kana, first_name_kana,
               -- Ruby Person#name_en は last_en/first_en どちらかが non-null なら
               -- "{last_en}, {first_en}" を返す (片方 null は空文字列扱いで comma+space を残す)。
               CASE WHEN last_name_en IS NOT NULL OR first_name_en IS NOT NULL
                    THEN CONCAT(COALESCE(last_name_en, ''), ', ', COALESCE(first_name_en, ''))
                    ELSE NULL END AS name_en,
               born_on, died_on, copyright_flag, description, sortkey
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
               w.sortkey, w.subtitle_kana,
               r.name AS role_name, wp.role_id,
               kt.name AS kana_type_name,
               (SELECT LPAD(wp2.person_id::text, 6, '0') FROM work_people wp2
                WHERE wp2.work_id = w.id AND wp2.role_id = 1
                ORDER BY wp2.id LIMIT 1) AS card_person_id
        FROM work_people wp
        JOIN works w ON w.id = wp.work_id
        LEFT JOIN roles r ON r.id = wp.role_id
        LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
        WHERE wp.person_id = ANY($1)
          AND w.work_status_id = 1 AND w.started_on <= $2
        ORDER BY wp.person_id, w.sortkey, w.subtitle_kana, w.id
        "#,
    )
    .bind(&person_ids)
    .bind(today)
    .fetch_all(pool)
    .await?;

    // Fetch unpublished works for all people
    let unpublished_work_rows: Vec<PersonWorkRow> = sqlx::query_as(
        r#"
        SELECT wp.person_id, w.id AS work_id, w.title, w.title_kana, w.subtitle,
               w.sortkey, w.subtitle_kana,
               r.name AS role_name, wp.role_id,
               kt.name AS kana_type_name,
               (SELECT LPAD(wp2.person_id::text, 6, '0') FROM work_people wp2
                WHERE wp2.work_id = w.id AND wp2.role_id = 1
                ORDER BY wp2.id LIMIT 1) AS card_person_id
        FROM work_people wp
        JOIN works w ON w.id = wp.work_id
        LEFT JOIN roles r ON r.id = wp.role_id
        LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
        WHERE wp.person_id = ANY($1)
          AND (w.work_status_id IN (3,4,5,6,7,8,9,10,11)
               OR (w.work_status_id = 1 AND w.started_on > $2))
        ORDER BY wp.person_id, w.sortkey, w.subtitle_kana, w.id
        "#,
    )
    .bind(&person_ids)
    .bind(today)
    .fetch_all(pool)
    .await?;

    // Fetch all work_people for published and unpublished works (for work_people field)
    let all_work_ids: Vec<i64> = work_rows
        .iter()
        .chain(unpublished_work_rows.iter())
        .map(|wr| wr.work_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let work_people_rows: Vec<WorkPersonRow> = if all_work_ids.is_empty() {
        vec![]
    } else {
        sqlx::query_as(
            r#"
            SELECT wp.work_id, wp.person_id,
                   CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS name,
                   r.name AS role_name,
                   wp.role_id
            FROM work_people wp
            JOIN people p ON p.id = wp.person_id
            LEFT JOIN roles r ON r.id = wp.role_id
            WHERE wp.work_id = ANY($1)
            ORDER BY wp.work_id, wp.role_id, wp.person_id
            "#,
        )
        .bind(&all_work_ids)
        .fetch_all(pool)
        .await?
    };

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

    // Fetch other_base_people
    let other_base_people_rows: Vec<OtherBasePersonRow> = sqlx::query_as(
        r#"
        SELECT bp.person_id, bp.original_person_id,
               CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS name
        FROM base_people bp
        JOIN people p ON p.id = bp.original_person_id
        WHERE bp.person_id = ANY($1)
        ORDER BY bp.person_id, bp.original_person_id
        "#,
    )
    .bind(&person_ids)
    .fetch_all(pool)
    .await?;

    // Group by person_id / work_id
    let works_by_person = db_helpers::group_by(&work_rows, |wr| wr.person_id);
    let unpub_by_person = db_helpers::group_by(&unpublished_work_rows, |wr| wr.person_id);
    let wp_by_work = db_helpers::group_by(&work_people_rows, |wp| wp.work_id);
    let sites_by_person = db_helpers::group_by(&site_rows, |sr| sr.person_id);
    let obp_by_person = db_helpers::group_by(&other_base_people_rows, |r| r.person_id);

    let mut file = std::io::BufWriter::new(std::fs::File::create(
        output_dir.join("person_pages.jsonl"),
    )?);
    let mut count = 0;

    for person in &people {
        let build_works = |rows: Option<&Vec<&PersonWorkRow>>| -> Vec<PersonWorkInfo> {
            let empty = vec![];
            rows.unwrap_or(&empty)
                .iter()
                .map(|wr| {
                    let work_people: Vec<WorkPersonRef> = wp_by_work
                        .get(&wr.work_id)
                        .unwrap_or(&vec![])
                        .iter()
                        .filter(|wp| wp.person_id != person.id)
                        .map(|wp| WorkPersonRef {
                            person_id: wp.person_id,
                            name: wp.name.clone(),
                            role_name: wp.role_name.clone(),
                        })
                        .collect();

                    PersonWorkInfo {
                        id: wr.work_id,
                        title: wr.title.clone(),
                        title_kana: wr.title_kana.clone(),
                        subtitle: wr.subtitle.clone(),
                        sortkey: wr.sortkey.clone(),
                        subtitle_kana: wr.subtitle_kana.clone(),
                        role: wr.role_name.clone(),
                        role_id: wr.role_id,
                        kana_type: wr.kana_type_name.clone(),
                        card_person_id: wr.card_person_id.clone(),
                        work_people,
                    }
                })
                .collect()
        };

        let works = build_works(works_by_person.get(&person.id));
        let unpublished_works = build_works(unpub_by_person.get(&person.id));

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

        let empty_obp = vec![];
        let other_base_people: Vec<OtherBasePerson> = obp_by_person
            .get(&person.id)
            .unwrap_or(&empty_obp)
            .iter()
            .map(|r| OtherBasePerson {
                id: r.original_person_id,
                name: r.name.clone(),
            })
            .collect();

        let data = PersonPageData {
            person: Person {
                id: person.id,
                last_name: person.last_name.clone(),
                first_name: person.first_name.clone(),
                last_name_kana: person.last_name_kana.clone(),
                first_name_kana: person.first_name_kana.clone(),
                name_en: person.name_en.clone(),
                born_on: person.born_on.clone(),
                died_on: person.died_on.clone(),
                copyright_flag: person.copyright_flag,
                description: person.description.clone(),
                sortkey: person.sortkey.clone(),
            },
            works,
            unpublished_works,
            sites,
            other_base_people,
        };

        write_jsonl_line(&mut file, &data)?;
        count += 1;
    }

    file.flush()?;
    println!("  -> {count} people");
    Ok(count)
}
