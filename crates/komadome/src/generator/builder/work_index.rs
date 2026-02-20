use anyhow::Result;
use serde_json::{json, Value};

use crate::data::models::WorkIndexData;
use crate::generator::kana::Kana;

/// Build work index page context
pub fn build_work_index_context(data: &WorkIndexData) -> Result<Value> {
    let kana = Kana::from_symbol(&data.kana_symbol);
    let display_char = kana
        .and_then(|k| k.display_char())
        .unwrap_or(&data.kana_symbol);

    let ctx = json!({
        "page_title": format!("作品インデックス：{} | 青空文庫", display_char),
        "bgcolor": "bg-sky-50",
        "kana_symbol": data.kana_symbol,
        "display_char": display_char,
        "page": data.page,
        "total_pages": data.total_pages,
        "has_prev": data.page > 1,
        "has_next": data.page < data.total_pages,
        "prev_page": if data.page > 1 { Some(data.page - 1) } else { None },
        "next_page": if data.page < data.total_pages { Some(data.page + 1) } else { None },

        "works": data.works.iter().map(|w| json!({
            "id": w.id,
            "title": &w.title,
            "title_kana": w.title_kana.as_deref().unwrap_or(""),
            "subtitle": w.subtitle.as_deref().unwrap_or(""),
            "author_name": w.author_name.as_deref().unwrap_or(""),
            "person_id": w.person_id,
            "has_person_id": w.person_id.is_some(),
            "card_path": format!("/cards/{:06}/card{}.html", w.person_id.unwrap_or(0), w.id),
        })).collect::<Vec<_>>(),

        // Pagination info for building URLs
        "pagination": build_pagination(data.page, data.total_pages),
    });

    Ok(ctx)
}

/// Build pagination series (shared by work_index and whatsnew)
pub fn build_pagination(current: usize, total: usize) -> Vec<Value> {
    let mut pages = Vec::new();

    // Always show first page
    if total > 0 {
        let is_current = current == 1;
        pages.push(json!({
            "page": 1,
            "is_current": is_current,
            "is_gap": false,
        }));
    }

    // Show gap if needed
    if current > 3 {
        pages.push(json!({
            "page": null,
            "is_current": false,
            "is_gap": true,
        }));
    }

    // Show pages around current
    for p in (current.saturating_sub(1))..=(current + 1).min(total) {
        if p > 1 && p < total {
            let is_current = p == current;
            pages.push(json!({
                "page": p,
                "is_current": is_current,
                "is_gap": false,
            }));
        }
    }

    // Show gap if needed
    if current < total.saturating_sub(2) {
        pages.push(json!({
            "page": null,
            "is_current": false,
            "is_gap": true,
        }));
    }

    // Always show last page
    if total > 1 {
        let is_current = current == total;
        pages.push(json!({
            "page": total,
            "is_current": is_current,
            "is_gap": false,
        }));
    }

    pages
}

/// Generate the output filename for a work index page
pub fn work_index_filename(kana_symbol: &str, page: usize) -> String {
    format!("sakuhin_{}{}.html", kana_symbol, page)
}
