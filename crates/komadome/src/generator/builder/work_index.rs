use anyhow::Result;
use serde_json::{Value, json};

use super::pagination::{PAGE_SIZE, build_pagination};
use crate::data::models::WorkIndexData;
use crate::generator::kana::Kana;

/// Build work index page context
pub fn build_work_index_context(data: &WorkIndexData) -> Result<Value> {
    let kana = Kana::from_symbol(&data.kana_symbol);
    let display_char = kana.and_then(|k| k.display_char()).unwrap_or("");

    let pg = &data.pagination;
    let page_offset = (pg.page - 1) * PAGE_SIZE;

    let ctx = json!({
        "page_title": format!("公開中　作品一覧：{} | 青空文庫", display_char),
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "kana_symbol": data.kana_symbol,
        "display_char": display_char,
        "page": pg.page,
        "total_pages": pg.total_pages,
        "has_prev": pg.has_prev(),
        "has_next": pg.has_next(),
        "prev_page": super::prev_page(pg.page),
        "next_page": super::next_page(pg.page, pg.total_pages),

        "works": data.works.iter().enumerate().map(|(i, w)| json!({
            "id": w.id,
            "title": &w.title,
            "title_kana": w.title_kana.as_deref().unwrap_or(""),
            "subtitle": w.subtitle.as_deref().unwrap_or(""),
            "author_name": w.author_name.as_deref().unwrap_or(""),
            "person_id": w.person_id,
            "has_person_id": w.person_id.is_some(),
            "card_path": format!("/cards/{}/card{}.html",
                super::card_person_dir(w.card_person_id.or(w.person_id).unwrap_or(0)),
                w.id),
            "row_number": page_offset + i + 1,
            "kana_type": w.kana_type.as_deref().unwrap_or(""),
            "author_text": w.author_text.as_deref().unwrap_or(""),
            "base_author_text": w.base_author_text.as_deref().unwrap_or(""),
            "translator_text": w.translator_text.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),

        // Pagination info for building URLs
        "pagination": build_pagination(pg.page, pg.total_pages),
    });

    Ok(ctx)
}

/// Generate the output filename for a work index page
pub fn work_index_filename(kana_symbol: &str, page: usize) -> String {
    format!("sakuhin_{kana_symbol}{page}.html")
}
