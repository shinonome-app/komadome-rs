//! `list_person_all_extended.zip` / `list_person_all_extended_utf8.zip` の生成。
//!
//! Ruby `CsvCreator#write_extended` (csv_creator.rb:111-186) と同じ 55 列の構成を再現する。
//! 公開作品 (`work_status_id = 1 AND started_on <= today`) を `(work, work_person)` 単位で展開
//! (同じ人物でも別 role なら 2 行)。

use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use serde_json::Value;
use sqlx::PgPool;
use std::path::Path;

use super::{headers, make_csv_writer, write_header, write_pair};

/// 図書カード URL は本番 aozora.gr.jp で固定 (Ruby `Work#card_url` 参照)。
/// Workfile のダウンロード URL とは違って config の MAIN_SITE_URL に依存しない。
const CARD_BASE_URL: &str = "https://www.aozora.gr.jp";

#[derive(sqlx::FromRow)]
struct Row {
    work_id: i64,
    title: String,
    title_kana: Option<String>,
    work_sortkey: Option<String>,
    subtitle: Option<String>,
    subtitle_kana: Option<String>,
    original_title: Option<String>,
    first_appearance: Option<String>,
    bibclasses_num: Option<String>,
    kana_type_name: Option<String>,
    work_copyright_flag: bool,
    started_on: Option<NaiveDate>,
    work_updated_at: Option<NaiveDateTime>,
    first_author_id: Option<i64>,

    person_id: i64,
    last_name: Option<String>,
    first_name: Option<String>,
    last_name_kana: Option<String>,
    first_name_kana: Option<String>,
    person_sortkey: Option<String>,
    person_sortkey2: Option<String>,
    last_name_en: Option<String>,
    first_name_en: Option<String>,

    role_name: Option<String>,
    born_on: Option<String>,
    died_on: Option<String>,
    person_copyright_flag: bool,

    teihon_json: Option<Value>,
    oyahon_json: Option<Value>,

    inputer_text: Option<String>,
    proofreader_text: Option<String>,

    text_url: Option<String>,
    text_filename: Option<String>,
    text_updated_at: Option<NaiveDateTime>,
    text_revision_count: Option<i32>,
    text_encoding_name: Option<String>,
    text_charset_name: Option<String>,

    html_url: Option<String>,
    html_filename: Option<String>,
    html_updated_at: Option<NaiveDateTime>,
    html_revision_count: Option<i32>,
    html_encoding_name: Option<String>,
    html_charset_name: Option<String>,
}

pub async fn generate(
    pool: &PgPool,
    zip_dir: &Path,
    today: NaiveDate,
    main_site_url: &str,
) -> Result<()> {
    println!("[finished_extended] querying...");
    let rows: Vec<Row> = sqlx::query_as(
        r#"
        WITH work_order AS (
            -- 各 work の中で最小の people.sortkey2 を求める。
            -- Ruby の `Work.eager_load(:people).order(:sortkey, :sortkey2, :id, ...)`
            -- では unique work の出現位置はその work の lowest sortkey2 行に依存する。
            SELECT wp.work_id, MIN(p.sortkey2) AS first_p_sortkey2
            FROM work_people wp
            JOIN people p ON p.id = wp.person_id
            GROUP BY wp.work_id
        )
        SELECT
            w.id AS work_id,
            w.title,
            w.title_kana,
            w.sortkey AS work_sortkey,
            w.subtitle,
            w.subtitle_kana,
            w.original_title,
            w.first_appearance,
            bc.num AS bibclasses_num,
            kt.name AS kana_type_name,
            w.copyright_flag AS work_copyright_flag,
            w.started_on,
            w.updated_at AS work_updated_at,
            (SELECT person_id FROM work_people
             WHERE work_id = w.id AND role_id = 1 ORDER BY id LIMIT 1) AS first_author_id,

            p.id AS person_id,
            p.last_name,
            p.first_name,
            p.last_name_kana,
            p.first_name_kana,
            p.sortkey AS person_sortkey,
            p.sortkey2 AS person_sortkey2,
            p.last_name_en,
            p.first_name_en,

            r.name AS role_name,
            p.born_on,
            p.died_on,
            p.copyright_flag AS person_copyright_flag,

            teihon.arr AS teihon_json,
            oyahon.arr AS oyahon_json,

            COALESCE(
                (SELECT string_agg(wkr.name, '、' ORDER BY ww.id)
                 FROM work_workers ww JOIN workers wkr ON wkr.id = ww.worker_id
                 WHERE ww.work_id = w.id AND ww.worker_role_id = 1),
                ''
            ) AS inputer_text,
            COALESCE(
                (SELECT string_agg(wkr.name, '、' ORDER BY ww.id)
                 FROM work_workers ww JOIN workers wkr ON wkr.id = ww.worker_id
                 WHERE ww.work_id = w.id AND ww.worker_role_id = 2),
                ''
            ) AS proofreader_text,

            tf.url AS text_url,
            tf.filename AS text_filename,
            tf.updated_at AS text_updated_at,
            tf.revision_count AS text_revision_count,
            tf_enc.name AS text_encoding_name,
            tf_chs.name AS text_charset_name,

            hf.url AS html_url,
            hf.filename AS html_filename,
            hf.updated_at AS html_updated_at,
            hf.revision_count AS html_revision_count,
            hf_enc.name AS html_encoding_name,
            hf_chs.name AS html_charset_name

        FROM works w
        JOIN work_people wp ON wp.work_id = w.id
        JOIN people p ON p.id = wp.person_id
        JOIN roles r ON r.id = wp.role_id
        LEFT JOIN kana_types kt ON kt.id = w.kana_type_id
        LEFT JOIN work_order wo ON wo.work_id = w.id

        LEFT JOIN LATERAL (
            SELECT num FROM bibclasses WHERE work_id = w.id ORDER BY id LIMIT 1
        ) bc ON TRUE

        LEFT JOIN LATERAL (
            SELECT json_agg(json_build_object(
                'title', title,
                'publisher', publisher,
                'first_pubdate', first_pubdate,
                'input_edition', input_edition,
                'proof_edition', proof_edition
            ) ORDER BY id) AS arr
            FROM original_books WHERE work_id = w.id AND booktype_id = 1
        ) teihon ON TRUE
        LEFT JOIN LATERAL (
            SELECT json_agg(json_build_object(
                'title', title,
                'publisher', publisher,
                'first_pubdate', first_pubdate
            ) ORDER BY id) AS arr
            FROM original_books WHERE work_id = w.id AND booktype_id <> 1
        ) oyahon ON TRUE

        LEFT JOIN LATERAL (
            SELECT wf.url, wf.filename, wf.updated_at, wf.revision_count,
                   wf.file_encoding_id, wf.charset_id
            FROM workfiles wf JOIN filetypes ft ON ft.id = wf.filetype_id
            WHERE wf.work_id = w.id AND ft.is_text = TRUE
            ORDER BY wf.id LIMIT 1
        ) tf ON TRUE
        LEFT JOIN file_encodings tf_enc ON tf_enc.id = tf.file_encoding_id
        LEFT JOIN charsets tf_chs ON tf_chs.id = tf.charset_id

        LEFT JOIN LATERAL (
            SELECT wf.url, wf.filename, wf.updated_at, wf.revision_count,
                   wf.file_encoding_id, wf.charset_id
            FROM workfiles wf JOIN filetypes ft ON ft.id = wf.filetype_id
            WHERE wf.work_id = w.id AND ft.is_html = TRUE
            ORDER BY wf.id LIMIT 1
        ) hf ON TRUE
        LEFT JOIN file_encodings hf_enc ON hf_enc.id = hf.file_encoding_id
        LEFT JOIN charsets hf_chs ON hf_chs.id = hf.charset_id

        WHERE w.work_status_id = 1 AND w.started_on <= $1
        -- Ruby: outer SQL は (work.sortkey, people.sortkey2, work.id) で works を並べ、
        -- 各 work 内では `work.work_people.each` (= デフォルト has_many = work_people.id 順)。
        -- ここでは work の代表 sortkey2 を CTE で先に求めて並べることで挙動を再現する。
        ORDER BY w.sortkey, wo.first_p_sortkey2, w.id, wp.id
        "#,
    )
    .bind(today)
    .fetch_all(pool)
    .await?;

    println!("  -> {} rows", rows.len());

    let mut buf = Vec::with_capacity(rows.len() * 1000);
    write_header(&mut buf, headers::FINISHED_EXTENDED)?;
    {
        let mut w = make_csv_writer(&mut buf);
        for r in &rows {
            write_row(&mut w, r, main_site_url)?;
        }
        w.flush()?;
    }

    write_pair(zip_dir, "list_person_all_extended", &buf)?;
    Ok(())
}

fn write_row(w: &mut csv::Writer<&mut Vec<u8>>, r: &Row, main_site_url: &str) -> Result<()> {
    let work_copyright = if r.work_copyright_flag {
        "あり"
    } else {
        "なし"
    };
    let person_copyright = if r.person_copyright_flag {
        "あり"
    } else {
        "なし"
    };

    let work_id = r.work_id.to_string();
    let started_on = r
        .started_on
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    // Rails 設定が `default_timezone = :local`、`time_zone = 'Asia/Tokyo'` のため、
    // TIMESTAMP カラムは JST で保存・読み出される。NaiveDateTime をそのまま整形すれば JST。
    let work_updated_at = r
        .work_updated_at
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let card_url = r
        .first_author_id
        .map(|aid| format!("{CARD_BASE_URL}/cards/{aid:06}/card{}.html", r.work_id))
        .unwrap_or_default();

    let person_id = r.person_id.to_string();
    let born_on = r.born_on.as_deref().unwrap_or("");
    let died_on = r.died_on.as_deref().unwrap_or("");

    let teihon = r.teihon_json.as_ref().and_then(|v| v.as_array());
    let oyahon = r.oyahon_json.as_ref().and_then(|v| v.as_array());

    let teihon0_title = json_str(teihon, 0, "title");
    let teihon0_publisher = json_str(teihon, 0, "publisher");
    let teihon0_first_pubdate = json_str(teihon, 0, "first_pubdate");
    let teihon0_input_edition = json_str(teihon, 0, "input_edition");
    let teihon0_proof_edition = json_str(teihon, 0, "proof_edition");
    let oyahon0_title = json_str(oyahon, 0, "title");
    let oyahon0_publisher = json_str(oyahon, 0, "publisher");
    let oyahon0_first_pubdate = json_str(oyahon, 0, "first_pubdate");

    let teihon1_title = json_str(teihon, 1, "title");
    let teihon1_publisher = json_str(teihon, 1, "publisher");
    let teihon1_first_pubdate = json_str(teihon, 1, "first_pubdate");
    let teihon1_input_edition = json_str(teihon, 1, "input_edition");
    let teihon1_proof_edition = json_str(teihon, 1, "proof_edition");
    let oyahon1_title = json_str(oyahon, 1, "title");
    let oyahon1_publisher = json_str(oyahon, 1, "publisher");
    let oyahon1_first_pubdate = json_str(oyahon, 1, "first_pubdate");

    // text/html ファイルのダウンロード URL を Workfile#download_url と同形式で構築する。
    let text_dl_url = build_download_url(
        r.text_url.as_deref(),
        r.text_filename.as_deref(),
        r.first_author_id,
        main_site_url,
    );
    let html_dl_url = build_download_url(
        r.html_url.as_deref(),
        r.html_filename.as_deref(),
        r.first_author_id,
        main_site_url,
    );

    // Ruby の `text_file&.updated_at` は Rails の Time オブジェクトをそのまま CSV に書く。
    // CSV 出力時の `to_s` は `2024-12-01 12:34:56 +0900` 形式 (JST)。
    // shinonome は `default_timezone = :local` で TIMESTAMP を JST 保存するため変換不要。
    let text_updated_at = format_workfile_timestamp(r.text_updated_at);
    let html_updated_at = format_workfile_timestamp(r.html_updated_at);
    let text_revision = r
        .text_revision_count
        .map(|n| n.to_string())
        .unwrap_or_default();
    let html_revision = r
        .html_revision_count
        .map(|n| n.to_string())
        .unwrap_or_default();

    w.write_record([
        // 作品 (1-14)
        work_id.as_str(),
        r.title.as_str(),
        r.title_kana.as_deref().unwrap_or(""),
        r.work_sortkey.as_deref().unwrap_or(""),
        r.subtitle.as_deref().unwrap_or(""),
        r.subtitle_kana.as_deref().unwrap_or(""),
        r.original_title.as_deref().unwrap_or(""),
        r.first_appearance.as_deref().unwrap_or(""),
        r.bibclasses_num.as_deref().unwrap_or(""),
        r.kana_type_name.as_deref().unwrap_or(""),
        work_copyright,
        started_on.as_str(),
        work_updated_at.as_str(),
        card_url.as_str(),
        // 人物 (15-23)
        person_id.as_str(),
        r.last_name.as_deref().unwrap_or(""),
        r.first_name.as_deref().unwrap_or(""),
        r.last_name_kana.as_deref().unwrap_or(""),
        r.first_name_kana.as_deref().unwrap_or(""),
        r.person_sortkey.as_deref().unwrap_or(""),
        r.person_sortkey2.as_deref().unwrap_or(""),
        r.last_name_en.as_deref().unwrap_or(""),
        r.first_name_en.as_deref().unwrap_or(""),
        // 役割等 (24-27)
        r.role_name.as_deref().unwrap_or(""),
        born_on,
        died_on,
        person_copyright,
        // 底本1 + 親本1 (28-35)
        teihon0_title.as_str(),
        teihon0_publisher.as_str(),
        teihon0_first_pubdate.as_str(),
        teihon0_input_edition.as_str(),
        teihon0_proof_edition.as_str(),
        oyahon0_title.as_str(),
        oyahon0_publisher.as_str(),
        oyahon0_first_pubdate.as_str(),
        // 底本2 + 親本2 (36-43)
        teihon1_title.as_str(),
        teihon1_publisher.as_str(),
        teihon1_first_pubdate.as_str(),
        teihon1_input_edition.as_str(),
        teihon1_proof_edition.as_str(),
        oyahon1_title.as_str(),
        oyahon1_publisher.as_str(),
        oyahon1_first_pubdate.as_str(),
        // 入力者・校正者 (44-45)
        r.inputer_text.as_deref().unwrap_or(""),
        r.proofreader_text.as_deref().unwrap_or(""),
        // テキストファイル (46-50)
        text_dl_url.as_str(),
        text_updated_at.as_str(),
        r.text_encoding_name.as_deref().unwrap_or(""),
        r.text_charset_name.as_deref().unwrap_or(""),
        text_revision.as_str(),
        // HTMLファイル (51-55)
        html_dl_url.as_str(),
        html_updated_at.as_str(),
        r.html_encoding_name.as_deref().unwrap_or(""),
        r.html_charset_name.as_deref().unwrap_or(""),
        html_revision.as_str(),
    ])?;
    Ok(())
}

/// JSON array の `idx` 番目要素から文字列フィールドを取り出す。なければ空文字。
fn json_str(arr: Option<&Vec<Value>>, idx: usize, key: &str) -> String {
    arr.and_then(|a| a.get(idx))
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default()
}

/// Workfile#download_url の Rust 版。url が空でなければ url、なければ
/// `{main_site_url}/cards/{author_id:06}/files/{filename}`。
fn build_download_url(
    url: Option<&str>,
    filename: Option<&str>,
    author_id: Option<i64>,
    main_site_url: &str,
) -> String {
    if let Some(u) = url {
        if !u.is_empty() {
            return u.to_string();
        }
    }
    match (filename, author_id) {
        (Some(fname), Some(aid)) if !fname.is_empty() => {
            format!("{main_site_url}/cards/{aid:06}/files/{fname}")
        }
        _ => String::new(),
    }
}

/// Workfile.updated_at を Ruby の `Time#to_s` (Asia/Tokyo) と同じ
/// `YYYY-MM-DD HH:MM:SS +0900` 形式で書き出す。
///
/// shinonome は `default_timezone = :local` で TIMESTAMP を JST のまま保存しているため、
/// 読み出した NaiveDateTime をそのまま整形すれば JST になる。
fn format_workfile_timestamp(ts: Option<NaiveDateTime>) -> String {
    let Some(ts) = ts else { return String::new() };
    format!("{} +0900", ts.format("%Y-%m-%d %H:%M:%S"))
}
