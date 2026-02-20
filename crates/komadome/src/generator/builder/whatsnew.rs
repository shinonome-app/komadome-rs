use anyhow::Result;
use chrono::NaiveDate;
use serde_json::{json, Value};

use crate::data::models::WhatsnewData;

use super::work_index::build_pagination;

/// Build whatsnew index page context (current year)
pub fn build_whatsnew_index_context(
    data: &WhatsnewData,
    today: &NaiveDate,
    year_links: &[i32],
) -> Result<Value> {
    let current_year = chrono::Datelike::year(today);
    let recent_cutoff = *today - chrono::Duration::days(7);

    let ctx = json!({
        "page_title": format!("新規公開作品　{}年公開分 | 青空文庫", current_year),
        "bgcolor": "bg-sky-50",
        "current_year": current_year,
        "today": today.format("%Y.%m.%d").to_string(),
        "page": data.page,
        "total_pages": data.total_pages,
        "has_prev": data.page > 1,
        "has_next": data.page < data.total_pages,
        "prev_page": if data.page > 1 { Some(data.page - 1) } else { None },
        "next_page": if data.page < data.total_pages { Some(data.page + 1) } else { None },
        "pagination": build_pagination(data.page, data.total_pages),
        "entries": data.entries.iter().map(|e| {
            let is_recent = e.started_on.as_ref().map_or(false, |s| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_or(false, |d| d >= recent_cutoff)
            });
            json!({
                "work_id": e.work_id,
                "title": &e.title,
                "subtitle": e.subtitle.as_deref().unwrap_or(""),
                "card_person_dir": e.card_person_id.map(|id| format!("{:06}", id)).unwrap_or_default(),
                "author_text": e.author_text.as_deref().unwrap_or(""),
                "inputer_text": e.inputer_text.as_deref().unwrap_or(""),
                "proofreader_text": e.proofreader_text.as_deref().unwrap_or(""),
                "translator_text": e.translator_text.as_deref().unwrap_or(""),
                "started_on": e.started_on.as_deref().unwrap_or(""),
                "is_recent": is_recent,
            })
        }).collect::<Vec<_>>(),
        "year_links": year_links.iter().rev().map(|y| json!({"year": y})).collect::<Vec<_>>(),
    });

    Ok(ctx)
}

/// Build whatsnew year page context (past year)
pub fn build_whatsnew_year_context(
    data: &WhatsnewData,
    today: &NaiveDate,
) -> Result<Value> {
    let year = data.year.unwrap_or(0);

    let ctx = json!({
        "page_title": format!("公開作品　{}年公開分 | 青空文庫", year),
        "bgcolor": "bg-sky-50",
        "year": year,
        "today": today.format("%Y.%m.%d").to_string(),
        "page": data.page,
        "total_pages": data.total_pages,
        "has_prev": data.page > 1,
        "has_next": data.page < data.total_pages,
        "prev_page": if data.page > 1 { Some(data.page - 1) } else { None },
        "next_page": if data.page < data.total_pages { Some(data.page + 1) } else { None },
        "pagination": build_pagination(data.page, data.total_pages),
        "entries": data.entries.iter().map(|e| {
            json!({
                "work_id": e.work_id,
                "title": &e.title,
                "subtitle": e.subtitle.as_deref().unwrap_or(""),
                "card_person_dir": e.card_person_id.map(|id| format!("{:06}", id)).unwrap_or_default(),
                "author_text": e.author_text.as_deref().unwrap_or(""),
                "inputer_text": e.inputer_text.as_deref().unwrap_or(""),
                "proofreader_text": e.proofreader_text.as_deref().unwrap_or(""),
                "translator_text": e.translator_text.as_deref().unwrap_or(""),
                "started_on": e.started_on.as_deref().unwrap_or(""),
            })
        }).collect::<Vec<_>>(),
    });

    Ok(ctx)
}

/// Generate whatsnew index page filename
pub fn whatsnew_index_filename(page: usize) -> String {
    format!("whatsnew{}.html", page)
}

/// Generate whatsnew year page filename
pub fn whatsnew_year_filename(year: i32, page: usize) -> String {
    format!("whatsnew_{}_{}.html", year, page)
}
