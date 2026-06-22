use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;
use serde_json::{Value, json};

use crate::data::masters::Masters;
use crate::data::models::{CardData, WorkfileInfo};
use crate::generator::builder::nl2br;

/// Old `link.js` script tags to strip from note fields.
static LINK_JS_SCRIPT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(<br\s*/?>)?<div id=?"?link"?></div><script[^>]*src="[^"]*link\.js"[^>]*></script>"#,
    )
    .unwrap()
});

/// Build card page context from card data
pub fn build_card_context(
    card: &CardData,
    _masters: &Masters,
    main_site_url: &str,
) -> Result<Value> {
    let bgcolor = if card.has_copyright() {
        crate::tailwind::bgcolor::COPYRIGHT
    } else {
        crate::tailwind::bgcolor::DEFAULT
    };

    // First author ID for header nav link
    let first_author_id = card.authors.first().map(|a| a.id).unwrap_or(card.person_id);

    // Find XHTML workfile URL for "いますぐ XHTML 版で読む" link
    let xhtml_url = card
        .workfiles
        .iter()
        .find(|f| f.is_html)
        .and_then(|f| workfile_top_xhtml_url(f, first_author_id, main_site_url));

    // Format bibclasses as comma-separated string
    let bibclasses_text = card
        .bibclasses
        .iter()
        .map(|b| match b.note.as_deref() {
            Some(note) if !note.is_empty() => format!("{}{} {}", b.name, b.num, note),
            _ => format!("{} {}", b.name, b.num),
        })
        .collect::<Vec<_>>()
        .join(", ");

    // External link URLs for LinkComponent
    let booklog_url = format!("http://booklog.jp/item/7/{}", card.work_id);
    let voyger_url = format!(
        "http://aozora.binb.jp/reader/main.html?cid={}",
        card.work_id
    );
    let airzoshi_url = format!(
        "https://www.satokazzz.com/airzoshi/reader.php?action=aozora&id={}",
        card.work_id
    );
    let rodoku_url = format!(
        "https://www.google.co.jp/search?hl=ja&source=hp&q=青空文庫+朗読+{}",
        card.title
    );

    let ctx = json!({
        "page_title": format!("図書カード：{} | 青空文庫", card.title),
        "bgcolor": bgcolor,
        "work_id": card.work_id,
        "person_id": card.person_id,
        "first_author_id": first_author_id,
        "title": &card.title,
        "booklog_url": booklog_url,
        "voyger_url": voyger_url,
        "airzoshi_url": airzoshi_url,
        "rodoku_url": rodoku_url,
        "title_kana": card.title_kana.as_deref().unwrap_or(""),
        "subtitle": card.subtitle.as_deref().unwrap_or(""),
        "subtitle_kana": card.subtitle_kana.as_deref().unwrap_or(""),
        "original_title": card.original_title.as_deref().unwrap_or(""),
        "collection": card.collection.as_deref().unwrap_or(""),
        "collection_kana": card.collection_kana.as_deref().unwrap_or(""),
        "kana_type": card.kana_type.as_deref().unwrap_or(""),
        "started_on": card.started_on.as_deref().unwrap_or(""),
        "note": card.note.as_ref().map(|n| cleanup_note(n)).unwrap_or_default(),
        "first_appearance": card.first_appearance.as_deref().unwrap_or(""),
        "description": card.description.as_deref().map(nl2br).unwrap_or_default(),
        "has_copyright": card.has_copyright(),
        "card_path": super::card_relative_path(card.person_id, card.work_id),
        "xhtml_url": xhtml_url,
        "bibclasses_text": bibclasses_text,

        "authors": card.authors.iter().map(|a| json!({
            "id": a.id,
            "name": &a.name,
            "name_kana": &a.name_kana,
            "copyright_flag": a.copyright_flag,
        })).collect::<Vec<_>>(),

        "translators": card.translators.iter().map(|t| json!({
            "id": t.id,
            "name": &t.name,
            "name_kana": &t.name_kana,
        })).collect::<Vec<_>>(),

        "editors": card.editors.iter().map(|e| json!({
            "id": e.id,
            "name": &e.name,
            "name_kana": &e.name_kana,
        })).collect::<Vec<_>>(),

        "workfiles": card.workfiles.iter().map(|f| {
            let download_filename = f.filename.as_deref().unwrap_or("");
            json!({
                "id": f.id,
                "filename": download_filename,
                "filesize": f.filesize,
                "filetype": f.filetype.as_deref().unwrap_or(""),
                "filetype_id": f.filetype_id,
                "is_html": f.is_html,
                "compresstype": f.compresstype.as_deref().unwrap_or(""),
                "charset": f.charset.as_deref().unwrap_or(""),
                "file_encoding": f.file_encoding.as_deref().unwrap_or(""),
                "url": f.url.as_deref().unwrap_or(""),
                "download_url": workfile_download_url(f),
                "download_display": workfile_download_display(f),
                "registered_on": f.registered_on.as_deref().unwrap_or(""),
                "last_updated_on": f.last_updated_on.as_deref().unwrap_or(""),
            })
        }).collect::<Vec<_>>(),

        "original_books": card.original_books.iter().enumerate().map(|(i, b)| json!({
            "is_first": i == 0,
            "title": &b.title,
            "publisher": b.publisher.as_deref().unwrap_or(""),
            "first_pubdate": b.first_pubdate.as_deref().unwrap_or(""),
            "input_edition": b.input_edition.as_deref().unwrap_or(""),
            "proof_edition": b.proof_edition.as_deref().unwrap_or(""),
            "booktype": b.booktype.as_deref().unwrap_or(""),
            "booktype_id": b.booktype_id,
        })).collect::<Vec<_>>(),

        "work_workers": card.work_workers.iter().map(|w| json!({
            "name": w.name.as_deref().unwrap_or(""),
            "role": w.role.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),

        "bibclasses": card.bibclasses.iter().map(|b| json!({
            "name": &b.name,
            "num": &b.num,
            "note": b.note.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),

        "work_people_details": card.work_people_details.iter().enumerate().map(|(i, wp)| json!({
            "index": i,
            "is_first": i == 0,
            "role_name": &wp.role_name,
            "person_id": wp.person_id,
            "name": &wp.name,
            "name_kana": &wp.name_kana,
            "name_en": wp.name_en.as_deref().unwrap_or(""),
            "born_on": wp.born_on.as_deref().unwrap_or(""),
            "died_on": wp.died_on.as_deref().unwrap_or(""),
            "description": wp.description.as_deref().map(nl2br).unwrap_or_default(),
        })).collect::<Vec<_>>(),

        "sites": card.sites.iter().map(|s| json!({
            "name": s.name.as_deref().unwrap_or(""),
            "url": s.url.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),
    });

    Ok(ctx)
}

/// Clean up note field (remove old link.js references, etc.)
fn cleanup_note(note: &str) -> String {
    LINK_JS_SCRIPT_RE.replace_all(note, "").to_string()
}

/// Build the href for the top "いますぐ XHTML 版で読む" link.
/// Ruby Workfile#download_url に合わせ、`{main_site_url}{download_path}` の絶対 URL。
fn workfile_top_xhtml_url(f: &WorkfileInfo, person_id: i64, main_site_url: &str) -> Option<String> {
    if let Some(url) = f.url.as_deref() {
        if !url.is_empty() {
            return Some(url.to_string());
        }
    }
    f.filename
        .as_deref()
        .map(|fname| format!("{main_site_url}/cards/{person_id:06}/files/{fname}"))
}

/// Build the href for the download table link.
/// Ruby template の `./files/{filename}` (page-relative path) と同じ形。
fn workfile_download_url(f: &WorkfileInfo) -> String {
    if let Some(url) = f.url.as_deref() {
        if !url.is_empty() {
            return url.to_string();
        }
    }
    match f.filename.as_deref() {
        Some(fname) => format!("./files/{fname}"),
        None => String::new(),
    }
}

/// Build the display text for the download table link.
/// Ruby template と同じ: url 指定があればそれ、無ければ filename のみ。
fn workfile_download_display(f: &WorkfileInfo) -> String {
    if let Some(url) = f.url.as_deref() {
        if !url.is_empty() {
            return url.to_string();
        }
    }
    f.filename.as_deref().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_note() {
        // 実データの書式（空 div + script、引用符欠け・相対 src 込み）を丸ごと除去する。
        let note = r#"本文<br><div id=link"></div><script type="text/javascript" src="../link.js"></script>後ろ"#;
        let cleaned = cleanup_note(note);
        assert_eq!(cleaned, "本文後ろ");
    }

    #[test]
    fn description_newlines_become_br() {
        let fixture_path = format!(
            "{}/tests/fixtures/card_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let masters_path = format!(
            "{}/tests/fixtures/masters_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut card: CardData =
            serde_json::from_str(&std::fs::read_to_string(&fixture_path).unwrap()).unwrap();
        let masters = Masters::load(std::path::Path::new(&masters_path)).unwrap();

        card.description = Some("一行目\r\n二行目\n三行目".to_string());
        if let Some(wp) = card.work_people_details.first_mut() {
            wp.description = Some("人物\r\n説明\n行".to_string());
        }
        let ctx = build_card_context(&card, &masters, "https://www.aozora.gr.jp").unwrap();

        assert_eq!(ctx["description"], "一行目<br>二行目<br>三行目");
        assert_eq!(
            ctx["work_people_details"][0]["description"],
            "人物<br>説明<br>行"
        );
    }

    #[test]
    fn card_context_matches_contract() {
        let fixture_path = format!(
            "{}/tests/fixtures/card_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let masters_path = format!(
            "{}/tests/fixtures/masters_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let card: CardData =
            serde_json::from_str(&std::fs::read_to_string(&fixture_path).unwrap()).unwrap();
        let masters = Masters::load(std::path::Path::new(&masters_path)).unwrap();

        let ctx = build_card_context(&card, &masters, "https://www.aozora.gr.jp").unwrap();

        let contract_source = include_str!("../../../../../contracts/cards/show.ntzc");
        let contract = natsuzora_contract::parse(contract_source).unwrap();
        natsuzora_contract::validate(&contract, &ctx).unwrap();
    }
}
