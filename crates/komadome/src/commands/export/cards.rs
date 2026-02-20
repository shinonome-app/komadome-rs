use anyhow::Result;
use serde::Serialize;
use sqlx::PgPool;
use std::io::Write;
use std::path::Path;

use super::db_helpers;

#[derive(Serialize)]
struct CardData {
    work_id: i64,
    person_id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    subtitle_kana: Option<String>,
    original_title: Option<String>,
    kana_type: Option<String>,
    started_on: Option<String>,
    note: Option<String>,
    first_appearance: Option<String>,
    description: Option<String>,
    authors: Vec<AuthorInfo>,
    translators: Vec<PersonRef>,
    editors: Vec<PersonRef>,
    workfiles: Vec<WorkfileInfo>,
    original_books: Vec<OriginalBookInfo>,
    work_workers: Vec<WorkWorkerInfo>,
    bibclasses: Vec<BibclassInfo>,
    sites: Vec<SiteInfo>,
}

#[derive(Serialize)]
struct AuthorInfo {
    id: i64,
    name: String,
    name_kana: String,
    copyright_flag: bool,
}

#[derive(Serialize)]
struct PersonRef {
    id: i64,
    name: String,
    name_kana: String,
}

#[derive(Serialize)]
struct WorkfileInfo {
    id: i64,
    filename: Option<String>,
    filesize: Option<i32>,
    filetype: Option<String>,
    filetype_id: i64,
    compresstype: Option<String>,
    charset: Option<String>,
    file_encoding: Option<String>,
    url: Option<String>,
    last_updated_on: Option<String>,
}

#[derive(Serialize)]
struct OriginalBookInfo {
    title: String,
    publisher: Option<String>,
    first_pubdate: Option<String>,
    input_edition: Option<String>,
    proof_edition: Option<String>,
    booktype: Option<String>,
    booktype_id: Option<i64>,
}

#[derive(Serialize)]
struct WorkWorkerInfo {
    name: Option<String>,
    role: Option<String>,
}

#[derive(Serialize)]
struct BibclassInfo {
    name: String,
    num: String,
    note: Option<String>,
}

#[derive(Serialize)]
struct SiteInfo {
    name: Option<String>,
    url: Option<String>,
}

// DB row types
#[derive(sqlx::FromRow)]
struct WorkRow {
    id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    subtitle_kana: Option<String>,
    original_title: Option<String>,
    kana_type_name: Option<String>,
    started_on: Option<chrono::NaiveDate>,
    note: Option<String>,
    first_appearance: Option<String>,
    description: Option<String>,
}

#[derive(sqlx::FromRow)]
struct WorkPersonRow {
    work_id: i64,
    person_id: i64,
    role_id: i64,
    person_name: String,
    person_name_kana: String,
    copyright_flag: bool,
}

#[derive(sqlx::FromRow)]
struct WorkfileRow {
    id: i64,
    work_id: i64,
    filename: Option<String>,
    filesize: Option<i32>,
    filetype_name: Option<String>,
    filetype_id: i64,
    compresstype_name: Option<String>,
    charset_name: Option<String>,
    file_encoding_name: Option<String>,
    url: Option<String>,
    last_updated_on: Option<chrono::NaiveDate>,
}

#[derive(sqlx::FromRow)]
struct OriginalBookRow {
    work_id: i64,
    title: String,
    publisher: Option<String>,
    first_pubdate: Option<String>,
    input_edition: Option<String>,
    proof_edition: Option<String>,
    booktype_name: Option<String>,
    booktype_id: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct WorkWorkerRow {
    work_id: i64,
    worker_name: Option<String>,
    worker_role_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct BibclassRow {
    work_id: i64,
    name: String,
    num: String,
    note: Option<String>,
}

#[derive(sqlx::FromRow)]
struct WorkSiteRow {
    work_id: i64,
    site_name: Option<String>,
    site_url: Option<String>,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting cards.jsonl...");

    let today = chrono::Local::now().date_naive();

    // Fetch published works with kana_type name
    let works: Vec<WorkRow> = sqlx::query_as(
        r#"
        SELECT w.id, w.title, w.title_kana, w.subtitle, w.subtitle_kana,
               w.original_title, kt.name AS kana_type_name,
               w.started_on, w.note, w.first_appearance, w.description
        FROM works w
        LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
        WHERE w.work_status_id = 1 AND w.started_on <= $1
        ORDER BY w.id
        "#,
    )
    .bind(today)
    .fetch_all(pool)
    .await?;

    let work_ids: Vec<i64> = works.iter().map(|w| w.id).collect();

    if work_ids.is_empty() {
        let file = std::fs::File::create(output_dir.join("cards.jsonl"))?;
        drop(file);
        println!("  -> 0 cards");
        return Ok(0);
    }

    // Fetch all related data in bulk
    let work_people: Vec<WorkPersonRow> = sqlx::query_as(
        r#"
        SELECT wp.work_id, wp.person_id, wp.role_id,
               CONCAT_WS(' ', p.last_name, p.first_name) AS person_name,
               CONCAT_WS(' ', p.last_name_kana, p.first_name_kana) AS person_name_kana,
               p.copyright_flag
        FROM work_people wp
        JOIN people p ON p.id = wp.person_id
        WHERE wp.work_id = ANY($1)
        ORDER BY wp.work_id, wp.role_id, wp.person_id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    let workfiles: Vec<WorkfileRow> = sqlx::query_as(
        r#"
        SELECT wf.id, wf.work_id, wf.filename, wf.filesize,
               ft.name AS filetype_name, wf.filetype_id,
               ct.name AS compresstype_name,
               cs.name AS charset_name,
               fe.name AS file_encoding_name,
               wf.url, wf.last_updated_on
        FROM workfiles wf
        LEFT JOIN filetypes ft ON ft.id = wf.filetype_id
        LEFT JOIN compresstypes ct ON ct.id = wf.compresstype_id
        LEFT JOIN charsets cs ON cs.id = wf.charset_id
        LEFT JOIN file_encodings fe ON fe.id = wf.file_encoding_id
        WHERE wf.work_id = ANY($1)
        ORDER BY wf.work_id, wf.id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    let original_books: Vec<OriginalBookRow> = sqlx::query_as(
        r#"
        SELECT ob.work_id, ob.title, ob.publisher, ob.first_pubdate,
               ob.input_edition, ob.proof_edition,
               bt.name AS booktype_name, ob.booktype_id
        FROM original_books ob
        LEFT JOIN booktypes bt ON bt.id = ob.booktype_id
        WHERE ob.work_id = ANY($1)
        ORDER BY ob.work_id, ob.id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    let work_workers: Vec<WorkWorkerRow> = sqlx::query_as(
        r#"
        SELECT ww.work_id,
               w.name AS worker_name,
               wr.name AS worker_role_name
        FROM work_workers ww
        LEFT JOIN workers w ON w.id = ww.worker_id
        LEFT JOIN worker_roles wr ON wr.id = ww.worker_role_id
        WHERE ww.work_id = ANY($1)
        ORDER BY ww.work_id, ww.id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    let bibclasses: Vec<BibclassRow> = sqlx::query_as(
        r#"
        SELECT work_id, name, num, note
        FROM bibclasses
        WHERE work_id = ANY($1)
        ORDER BY work_id, id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    let work_sites: Vec<WorkSiteRow> = sqlx::query_as(
        r#"
        SELECT ws.work_id, s.name AS site_name, s.url AS site_url
        FROM work_sites ws
        JOIN sites s ON s.id = ws.site_id
        WHERE ws.work_id = ANY($1)
        ORDER BY ws.work_id, ws.id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    // Group related data by work_id
    let people_by_work = db_helpers::group_by(&work_people, |wp| wp.work_id);
    let files_by_work = db_helpers::group_by(&workfiles, |wf| wf.work_id);
    let books_by_work = db_helpers::group_by(&original_books, |ob| ob.work_id);
    let workers_by_work = db_helpers::group_by(&work_workers, |ww| ww.work_id);
    let bibclasses_by_work = db_helpers::group_by(&bibclasses, |bc| bc.work_id);
    let sites_by_work = db_helpers::group_by(&work_sites, |ws| ws.work_id);

    // Build cards - one card per related person per work
    // Ruby generates cards for all person-work combinations (authors, translators, editors)
    let mut file = std::io::BufWriter::new(std::fs::File::create(output_dir.join("cards.jsonl"))?);
    let mut count = 0;

    for work in &works {
        let empty_people = vec![];
        let people = people_by_work.get(&work.id).unwrap_or(&empty_people);

        // Collect all unique person IDs related to this work (any role)
        let all_person_ids: Vec<i64> = {
            let mut ids: Vec<i64> = people.iter().map(|wp| wp.person_id).collect();
            ids.sort();
            ids.dedup();
            ids
        };

        if all_person_ids.is_empty() {
            continue;
        }

        let authors: Vec<AuthorInfo> = people
            .iter()
            .filter(|wp| wp.role_id == 1)
            .map(|wp| AuthorInfo {
                id: wp.person_id,
                name: wp.person_name.clone(),
                name_kana: wp.person_name_kana.clone(),
                copyright_flag: wp.copyright_flag,
            })
            .collect();

        let translators: Vec<PersonRef> = people
            .iter()
            .filter(|wp| wp.role_id == 2)
            .map(|wp| PersonRef {
                id: wp.person_id,
                name: wp.person_name.clone(),
                name_kana: wp.person_name_kana.clone(),
            })
            .collect();

        let editors: Vec<PersonRef> = people
            .iter()
            .filter(|wp| wp.role_id == 3)
            .map(|wp| PersonRef {
                id: wp.person_id,
                name: wp.person_name.clone(),
                name_kana: wp.person_name_kana.clone(),
            })
            .collect();

        let empty_files = vec![];
        let wf_list: Vec<WorkfileInfo> = files_by_work
            .get(&work.id)
            .unwrap_or(&empty_files)
            .iter()
            .map(|wf| WorkfileInfo {
                id: wf.id,
                filename: wf.filename.clone(),
                filesize: wf.filesize,
                filetype: wf.filetype_name.clone(),
                filetype_id: wf.filetype_id,
                compresstype: wf.compresstype_name.clone(),
                charset: wf.charset_name.clone(),
                file_encoding: wf.file_encoding_name.clone(),
                url: wf.url.clone(),
                last_updated_on: wf.last_updated_on.map(|d| d.to_string()),
            })
            .collect();

        let empty_books = vec![];
        let ob_list: Vec<OriginalBookInfo> = books_by_work
            .get(&work.id)
            .unwrap_or(&empty_books)
            .iter()
            .map(|ob| OriginalBookInfo {
                title: ob.title.clone(),
                publisher: ob.publisher.clone(),
                first_pubdate: ob.first_pubdate.clone(),
                input_edition: ob.input_edition.clone(),
                proof_edition: ob.proof_edition.clone(),
                booktype: ob.booktype_name.clone(),
                booktype_id: ob.booktype_id,
            })
            .collect();

        let empty_workers = vec![];
        let ww_list: Vec<WorkWorkerInfo> = workers_by_work
            .get(&work.id)
            .unwrap_or(&empty_workers)
            .iter()
            .map(|ww| WorkWorkerInfo {
                name: ww.worker_name.clone(),
                role: ww.worker_role_name.clone(),
            })
            .collect();

        let empty_bib = vec![];
        let bc_list: Vec<BibclassInfo> = bibclasses_by_work
            .get(&work.id)
            .unwrap_or(&empty_bib)
            .iter()
            .map(|bc| BibclassInfo {
                name: bc.name.clone(),
                num: bc.num.clone(),
                note: bc.note.clone(),
            })
            .collect();

        let empty_sites = vec![];
        let site_list: Vec<SiteInfo> = sites_by_work
            .get(&work.id)
            .unwrap_or(&empty_sites)
            .iter()
            .map(|ws| SiteInfo {
                name: ws.site_name.clone(),
                url: ws.site_url.clone(),
            })
            .collect();

        let note = work.note.as_deref().map(remove_link_tag);

        // One card per related person
        for person_id in &all_person_ids {
            let card = CardData {
                work_id: work.id,
                person_id: *person_id,
                title: work.title.clone(),
                title_kana: work.title_kana.clone(),
                subtitle: work.subtitle.clone(),
                subtitle_kana: work.subtitle_kana.clone(),
                original_title: work.original_title.clone(),
                kana_type: work.kana_type_name.clone(),
                started_on: work.started_on.map(|d| d.to_string()),
                note: note.clone(),
                first_appearance: work.first_appearance.clone(),
                description: work.description.clone(),
                authors: authors.clone(),
                translators: translators.clone(),
                editors: editors.clone(),
                workfiles: wf_list.clone(),
                original_books: ob_list.clone(),
                work_workers: ww_list.clone(),
                bibclasses: bc_list.clone(),
                sites: site_list.clone(),
            };

            serde_json::to_writer(&mut file, &card)?;
            file.write_all(b"\n")?;
            count += 1;
        }
    }

    file.flush()?;
    println!("  -> {} cards", count);
    Ok(count)
}

/// Remove legacy link.js tags from note HTML
/// Ported from Ruby: Work#note_without_link_tag
fn remove_link_tag(note: &str) -> String {
    let re = regex::Regex::new(
        r#"(<br\s*/?>)?<div id=?"?link"?></div><script[^>]*src="[^"]*link\.js"[^>]*></script>"#,
    )
    .unwrap();
    re.replace_all(note, "").to_string()
}

// Make AuthorInfo, PersonRef, WorkfileInfo etc. clonable for multi-author cards
impl Clone for AuthorInfo {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            name_kana: self.name_kana.clone(),
            copyright_flag: self.copyright_flag,
        }
    }
}

impl Clone for PersonRef {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            name_kana: self.name_kana.clone(),
        }
    }
}

impl Clone for WorkfileInfo {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            filename: self.filename.clone(),
            filesize: self.filesize,
            filetype: self.filetype.clone(),
            filetype_id: self.filetype_id,
            compresstype: self.compresstype.clone(),
            charset: self.charset.clone(),
            file_encoding: self.file_encoding.clone(),
            url: self.url.clone(),
            last_updated_on: self.last_updated_on.clone(),
        }
    }
}

impl Clone for OriginalBookInfo {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            publisher: self.publisher.clone(),
            first_pubdate: self.first_pubdate.clone(),
            input_edition: self.input_edition.clone(),
            proof_edition: self.proof_edition.clone(),
            booktype: self.booktype.clone(),
            booktype_id: self.booktype_id,
        }
    }
}

impl Clone for WorkWorkerInfo {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            role: self.role.clone(),
        }
    }
}

impl Clone for BibclassInfo {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            num: self.num.clone(),
            note: self.note.clone(),
        }
    }
}

impl Clone for SiteInfo {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            url: self.url.clone(),
        }
    }
}
