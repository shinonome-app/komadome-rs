use anyhow::Result;
use serde_json::{json, Value};

use crate::data::masters::Masters;
use crate::data::models::CardData;

/// Build card page context from card data
pub fn build_card_context(card: &CardData, _masters: &Masters) -> Result<Value> {
    let bgcolor = if card.has_copyright() {
        "bg-rose-50"
    } else {
        "bg-sky-50"
    };

    let ctx = json!({
        "page_title": format!("図書カード：{} | 青空文庫", card.title),
        "bgcolor": bgcolor,
        "work_id": card.work_id,
        "person_id": card.person_id,
        "title": &card.title,
        "title_kana": card.title_kana.as_deref().unwrap_or(""),
        "subtitle": card.subtitle.as_deref().unwrap_or(""),
        "subtitle_kana": card.subtitle_kana.as_deref().unwrap_or(""),
        "original_title": card.original_title.as_deref().unwrap_or(""),
        "kana_type": card.kana_type.as_deref().unwrap_or(""),
        "started_on": card.started_on.as_deref().unwrap_or(""),
        "note": card.note.as_ref().map(|n| cleanup_note(n)).unwrap_or_default(),
        "first_appearance": card.first_appearance.as_deref().unwrap_or(""),
        "description": card.description.as_deref().unwrap_or(""),
        "has_copyright": card.has_copyright(),
        "card_path": card.card_path(),

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

        "workfiles": card.workfiles.iter().map(|f| json!({
            "id": f.id,
            "filename": f.filename.as_deref().unwrap_or(""),
            "filesize": f.filesize,
            "filetype": f.filetype.as_deref().unwrap_or(""),
            "filetype_id": f.filetype_id,
            "compresstype": f.compresstype.as_deref().unwrap_or(""),
            "charset": f.charset.as_deref().unwrap_or(""),
            "file_encoding": f.file_encoding.as_deref().unwrap_or(""),
            "url": f.url.as_deref().unwrap_or(""),
            "last_updated_on": f.last_updated_on.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),

        "original_books": card.original_books.iter().map(|b| json!({
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

        "sites": card.sites.iter().map(|s| json!({
            "name": s.name.as_deref().unwrap_or(""),
            "url": s.url.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),
    });

    Ok(ctx)
}

/// Clean up note field (remove old link.js references, etc.)
fn cleanup_note(note: &str) -> String {
    // Remove old link.js script tags
    let re = regex::Regex::new(r#"<script[^>]*link\.js[^>]*></script>"#).unwrap();
    re.replace_all(note, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_note() {
        let note = r#"Some text <script src="link.js"></script> more text"#;
        let cleaned = cleanup_note(note);
        assert_eq!(cleaned, "Some text  more text");
    }
}
