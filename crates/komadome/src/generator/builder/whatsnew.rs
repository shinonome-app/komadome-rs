use anyhow::Result;
use chrono::NaiveDate;
use serde_json::{Value, json};

use crate::data::models::WhatsnewData;

use super::work_index::{build_pagination, build_pagination_nav_html};

/// Build whatsnew index page context (current year)
pub fn build_whatsnew_index_context(
    data: &WhatsnewData,
    today: &NaiveDate,
    year_links: &[i32],
) -> Result<Value> {
    let current_year = chrono::Datelike::year(today);
    let recent_cutoff = *today - chrono::Duration::days(7);

    let pagination_nav_html = build_pagination_nav_html(data.page, data.total_pages, |p| {
        format!("/index_pages/whatsnew{p}.html")
    });

    let ctx = json!({
        "page_title": "新規公開作品 | 青空文庫",
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "current_year": current_year,
        "today": today.format("%Y.%m.%d").to_string(),
        "page": data.page,
        "total_pages": data.total_pages,
        "has_prev": data.page > 1,
        "has_next": data.page < data.total_pages,
        "prev_page": super::prev_page(data.page),
        "next_page": super::next_page(data.page, data.total_pages),
        "pagination": build_pagination(data.page, data.total_pages),
        "pagination_nav_html": &pagination_nav_html,
        "entries": data.entries.iter().map(|e| {
            let is_recent = e.started_on.as_ref().is_some_and(|s| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .is_ok_and(|d| d >= recent_cutoff)
            });
            json!({
                "work_id": e.work_id,
                "title": &e.title,
                "subtitle": e.subtitle.as_deref().unwrap_or(""),
                "card_person_dir": e.card_person_id.map(super::card_person_dir).unwrap_or_default(),
                "author_text": e.author_text.as_deref().unwrap_or(""),
                "inputer_text": e.inputer_text.as_deref().unwrap_or(""),
                "proofreader_text": e.proofreader_text.as_deref().unwrap_or(""),
                "translator_text": e.translator_text.as_deref().unwrap_or(""),
                "started_on": e.started_on.as_deref().unwrap_or(""),
                "is_recent": is_recent,
            })
        }).collect::<Vec<_>>(),
        "year_links": year_links.iter().map(|y| json!({"year": y})).collect::<Vec<_>>(),
    });

    Ok(ctx)
}

/// Build whatsnew year page context (past year)
pub fn build_whatsnew_year_context(data: &WhatsnewData, today: &NaiveDate) -> Result<Value> {
    let year = data.year.unwrap_or(0);

    let pagination_nav_html = build_pagination_nav_html(data.page, data.total_pages, |p| {
        format!("/index_pages/whatsnew_{year}_{p}.html")
    });

    let ctx = json!({
        "page_title": format!("公開作品 {}年公開分 | 青空文庫", year),
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "year": year,
        "today": today.format("%Y.%m.%d").to_string(),
        "page": data.page,
        "total_pages": data.total_pages,
        "has_prev": data.page > 1,
        "has_next": data.page < data.total_pages,
        "prev_page": super::prev_page(data.page),
        "next_page": super::next_page(data.page, data.total_pages),
        "pagination": build_pagination(data.page, data.total_pages),
        "pagination_nav_html": &pagination_nav_html,
        "entries": data.entries.iter().map(|e| {
            json!({
                "work_id": e.work_id,
                "title": &e.title,
                "subtitle": e.subtitle.as_deref().unwrap_or(""),
                "card_person_dir": e.card_person_id.map(super::card_person_dir).unwrap_or_default(),
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
    format!("whatsnew{page}.html")
}

/// Generate whatsnew year page filename
pub fn whatsnew_year_filename(year: i32, page: usize) -> String {
    format!("whatsnew_{year}_{page}.html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whatsnew_index_context_matches_contract() {
        let fixture_path = format!(
            "{}/tests/fixtures/whatsnew_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let data: WhatsnewData =
            serde_json::from_str(&std::fs::read_to_string(&fixture_path).unwrap()).unwrap();

        let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let ctx = build_whatsnew_index_context(&data, &today, &[2023, 2024]).unwrap();

        let contract_source = include_str!("../../../../../contracts/whatsnew/index.ntzc");
        let contract = natsuzora_contract::parse(contract_source).unwrap();
        natsuzora_contract::validate(&contract, &ctx).unwrap();
    }
}
