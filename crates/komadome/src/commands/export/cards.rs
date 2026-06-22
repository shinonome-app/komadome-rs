use anyhow::Result;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

use super::db_helpers;
use super::export_helpers::write_jsonl_line;
use crate::data::models::{
    AuthorInfo, BibclassInfo, CardData, OriginalBookInfo, PersonRef, SiteInfo, WorkPersonDetail,
    WorkWorkerInfo, WorkfileInfo,
};

// DB row types
#[derive(sqlx::FromRow)]
struct WorkRow {
    id: i64,
    title: String,
    title_kana: Option<String>,
    subtitle: Option<String>,
    subtitle_kana: Option<String>,
    original_title: Option<String>,
    collection: Option<String>,
    collection_kana: Option<String>,
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
    role_name: String,
    person_name: String,
    person_name_kana: String,
    person_name_en: Option<String>,
    born_on: Option<String>,
    died_on: Option<String>,
    person_description: Option<String>,
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
    is_html: bool,
    compresstype_name: Option<String>,
    charset_name: Option<String>,
    file_encoding_name: Option<String>,
    url: Option<String>,
    registered_on: Option<chrono::NaiveDate>,
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

// Row -> DTO conversions. Each関連レコードの「行→DTO」変換を1箇所に閉じ込める。
impl From<&WorkfileRow> for WorkfileInfo {
    fn from(wf: &WorkfileRow) -> Self {
        WorkfileInfo {
            id: wf.id,
            filename: wf.filename.clone(),
            filesize: wf.filesize,
            filetype: wf.filetype_name.clone(),
            filetype_id: wf.filetype_id,
            is_html: wf.is_html,
            compresstype: wf.compresstype_name.clone(),
            charset: wf.charset_name.clone(),
            file_encoding: wf.file_encoding_name.clone(),
            url: wf.url.clone(),
            registered_on: wf.registered_on.map(|d| d.to_string()),
            last_updated_on: wf.last_updated_on.map(|d| d.to_string()),
        }
    }
}

impl From<&OriginalBookRow> for OriginalBookInfo {
    fn from(ob: &OriginalBookRow) -> Self {
        OriginalBookInfo {
            title: ob.title.clone(),
            publisher: ob.publisher.clone(),
            first_pubdate: ob.first_pubdate.clone(),
            input_edition: ob.input_edition.clone(),
            proof_edition: ob.proof_edition.clone(),
            booktype: ob.booktype_name.clone(),
            booktype_id: ob.booktype_id,
        }
    }
}

impl From<&WorkWorkerRow> for WorkWorkerInfo {
    fn from(ww: &WorkWorkerRow) -> Self {
        WorkWorkerInfo {
            name: ww.worker_name.clone(),
            role: ww.worker_role_name.clone(),
        }
    }
}

impl From<&BibclassRow> for BibclassInfo {
    fn from(bc: &BibclassRow) -> Self {
        BibclassInfo {
            name: bc.name.clone(),
            num: bc.num.clone(),
            note: bc.note.clone(),
        }
    }
}

impl From<&WorkSiteRow> for SiteInfo {
    fn from(ws: &WorkSiteRow) -> Self {
        SiteInfo {
            name: ws.site_name.clone(),
            url: ws.site_url.clone(),
        }
    }
}

impl From<&WorkPersonRow> for WorkPersonDetail {
    fn from(wp: &WorkPersonRow) -> Self {
        WorkPersonDetail {
            role_name: wp.role_name.clone(),
            person_id: wp.person_id,
            name: wp.person_name.clone(),
            name_kana: wp.person_name_kana.clone(),
            name_en: wp.person_name_en.clone(),
            born_on: wp.born_on.clone(),
            died_on: wp.died_on.clone(),
            description: wp.person_description.clone(),
            copyright_flag: wp.copyright_flag,
        }
    }
}

/// `group_by` の結果から特定 work_id 分の行を取り出し、DTO へ変換する。
fn collect_infos<R, T>(grouped: &HashMap<i64, Vec<&R>>, work_id: i64) -> Vec<T>
where
    for<'r> T: From<&'r R>,
{
    grouped
        .get(&work_id)
        .map(|rows| rows.iter().map(|&r| T::from(r)).collect())
        .unwrap_or_default()
}

/// 1作品に紐づく関連データ(人物・ファイル・底本など)を組み立て済み DTO として束ねる。
/// person ごとのカード生成では、ここに集約した値を clone して使う。
struct WorkAssociations {
    authors: Vec<AuthorInfo>,
    translators: Vec<PersonRef>,
    editors: Vec<PersonRef>,
    workfiles: Vec<WorkfileInfo>,
    original_books: Vec<OriginalBookInfo>,
    work_workers: Vec<WorkWorkerInfo>,
    bibclasses: Vec<BibclassInfo>,
    sites: Vec<SiteInfo>,
    work_people_details: Vec<WorkPersonDetail>,
    note: Option<String>,
    /// カードを生成する対象 person_id (role 不問・出現順・person_id=0 と重複を除外)。
    person_ids: Vec<i64>,
}

impl WorkAssociations {
    fn build(
        work: &WorkRow,
        people: &[&WorkPersonRow],
        files_by_work: &HashMap<i64, Vec<&WorkfileRow>>,
        books_by_work: &HashMap<i64, Vec<&OriginalBookRow>>,
        workers_by_work: &HashMap<i64, Vec<&WorkWorkerRow>>,
        bibclasses_by_work: &HashMap<i64, Vec<&BibclassRow>>,
        sites_by_work: &HashMap<i64, Vec<&WorkSiteRow>>,
    ) -> Self {
        // Collect all unique person IDs related to this work (any role)
        // Preserve insertion order (by work_people.id) to match Rails' .uniq behavior
        // Skip person_id=0 ("著者なし" placeholder)
        let person_ids: Vec<i64> = {
            let mut seen = HashSet::new();
            people
                .iter()
                .filter(|wp| wp.person_id != 0)
                .filter_map(|wp| seen.insert(wp.person_id).then_some(wp.person_id))
                .collect()
        };

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

        let person_refs = |role_id: i64| -> Vec<PersonRef> {
            people
                .iter()
                .filter(|wp| wp.role_id == role_id)
                .map(|wp| PersonRef {
                    id: wp.person_id,
                    name: wp.person_name.clone(),
                    name_kana: wp.person_name_kana.clone(),
                })
                .collect()
        };

        // Rails: work.work_people.sort_by { |wp| [wp.role_id, wp.person_id] }
        let work_people_details: Vec<WorkPersonDetail> = {
            let mut sorted_people: Vec<&&WorkPersonRow> = people.iter().collect();
            sorted_people.sort_by_key(|wp| (wp.role_id, wp.person_id));
            sorted_people.iter().map(|wp| (**wp).into()).collect()
        };

        WorkAssociations {
            authors,
            translators: person_refs(2),
            editors: person_refs(3),
            workfiles: collect_infos(files_by_work, work.id),
            original_books: collect_infos(books_by_work, work.id),
            work_workers: collect_infos(workers_by_work, work.id),
            bibclasses: collect_infos(bibclasses_by_work, work.id),
            sites: collect_infos(sites_by_work, work.id),
            work_people_details,
            note: work.note.as_deref().map(remove_link_tag),
            person_ids,
        }
    }
}

/// 1作品×1人物のカード DTO を組み立てる。
fn build_card(work: &WorkRow, person_id: i64, assoc: &WorkAssociations) -> CardData {
    CardData {
        work_id: work.id,
        person_id,
        title: work.title.clone(),
        title_kana: work.title_kana.clone(),
        subtitle: work.subtitle.clone(),
        subtitle_kana: work.subtitle_kana.clone(),
        original_title: work.original_title.clone(),
        collection: work.collection.clone(),
        collection_kana: work.collection_kana.clone(),
        kana_type: work.kana_type_name.clone(),
        started_on: work.started_on.map(|d| d.to_string()),
        note: assoc.note.clone(),
        first_appearance: work.first_appearance.clone(),
        description: work.description.clone(),
        authors: assoc.authors.clone(),
        translators: assoc.translators.clone(),
        editors: assoc.editors.clone(),
        workfiles: assoc.workfiles.clone(),
        original_books: assoc.original_books.clone(),
        work_workers: assoc.work_workers.clone(),
        bibclasses: assoc.bibclasses.clone(),
        sites: assoc.sites.clone(),
        work_people_details: assoc.work_people_details.clone(),
    }
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting cards.jsonl...");

    let today = crate::clock::build_date();

    // Fetch published works with kana_type name
    let works: Vec<WorkRow> = sqlx::query_as(
        r#"
        SELECT w.id, w.title, w.title_kana, w.subtitle, w.subtitle_kana,
               w.original_title, w.collection, w.collection_kana,
               kt.name AS kana_type_name,
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
               r.name AS role_name,
               CONCAT(COALESCE(p.last_name, ''), ' ', COALESCE(p.first_name, '')) AS person_name,
               CONCAT(COALESCE(p.last_name_kana, ''), ' ', COALESCE(p.first_name_kana, '')) AS person_name_kana,
               -- Ruby Person#name_en に合わせる ("{last_en}, {first_en}"、片方 null は空文字列)
               CASE WHEN p.last_name_en IS NOT NULL OR p.first_name_en IS NOT NULL
                    THEN CONCAT(COALESCE(p.last_name_en, ''), ', ', COALESCE(p.first_name_en, ''))
                    ELSE NULL END AS person_name_en,
               p.born_on, p.died_on,
               p.description AS person_description,
               p.copyright_flag
        FROM work_people wp
        JOIN people p ON p.id = wp.person_id
        JOIN roles r ON r.id = wp.role_id
        WHERE wp.work_id = ANY($1)
        ORDER BY wp.work_id, wp.id
        "#,
    )
    .bind(&work_ids)
    .fetch_all(pool)
    .await?;

    let workfiles: Vec<WorkfileRow> = sqlx::query_as(
        r#"
        SELECT wf.id, wf.work_id, wf.filename, wf.filesize,
               ft.name AS filetype_name, wf.filetype_id,
               COALESCE(ft.is_html, false) AS is_html,
               ct.name AS compresstype_name,
               cs.name AS charset_name,
               fe.name AS file_encoding_name,
               wf.url, wf.registered_on, wf.last_updated_on
        FROM workfiles wf
        LEFT JOIN filetypes ft ON ft.id = wf.filetype_id
        LEFT JOIN compresstypes ct ON ct.id = wf.compresstype_id
        LEFT JOIN charsets cs ON cs.id = wf.charset_id
        LEFT JOIN file_encodings fe ON fe.id = wf.file_encoding_id
        WHERE wf.work_id = ANY($1)
        ORDER BY wf.work_id, wf.filetype_id, wf.id
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

    let empty_people: Vec<&WorkPersonRow> = vec![];
    for work in &works {
        let people = people_by_work.get(&work.id).unwrap_or(&empty_people);

        let assoc = WorkAssociations::build(
            work,
            people,
            &files_by_work,
            &books_by_work,
            &workers_by_work,
            &bibclasses_by_work,
            &sites_by_work,
        );

        // One card per related person (no related person -> no card)
        for &person_id in &assoc.person_ids {
            let card = build_card(work, person_id, &assoc);
            write_jsonl_line(&mut file, &card)?;
            count += 1;
        }
    }

    file.flush()?;
    println!("  -> {count} cards");
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
