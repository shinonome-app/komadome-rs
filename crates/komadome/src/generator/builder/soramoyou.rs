use anyhow::Result;
use chrono::NaiveDate;
use serde_json::{json, Value};

use crate::data::models::NewsData;

const BEGIN_YEAR: i32 = 1997;

/// Build soramoyou index page context (current year)
pub fn build_soramoyou_index_context(
    data: &NewsData,
    current_year: i32,
) -> Result<Value> {
    let year_links: Vec<Value> = (BEGIN_YEAR..current_year)
        .map(|y| json!({"year": y}))
        .collect();

    let ctx = json!({
        "page_title": "そらもよう | 青空文庫",
        "bgcolor": "bg-white-100",
        "entries": build_entries(&data.entries),
        "year_links": year_links,
    });

    Ok(ctx)
}

/// Build soramoyou year page context (past year)
pub fn build_soramoyou_year_context(
    data: &NewsData,
) -> Result<Value> {
    let year_links: Vec<Value> = (BEGIN_YEAR..data.year)
        .map(|y| json!({"year": y}))
        .collect();

    let ctx = json!({
        "page_title": "そらもよう | 青空文庫",
        "bgcolor": "bg-white-100",
        "entries": build_entries(&data.entries),
        "year_links": year_links,
    });

    Ok(ctx)
}

fn build_entries(entries: &[crate::data::models::NewsEntry]) -> Vec<Value> {
    entries
        .iter()
        .map(|e| {
            let published_on_display = e
                .published_on
                .as_ref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .map(|d| format!("{}年{:02}月{:02}日", d.year(), d.month(), d.day()))
                .unwrap_or_default();

            // Convert newlines to <br> (HTML5 style, matching Rails sanitize output)
            let body_html = e.body.replace("\r\n", "<br>").replace('\n', "<br>");

            json!({
                "id": e.id,
                "title": e.title,
                "published_on_display": published_on_display,
                "body_html": body_html,
            })
        })
        .collect()
}

use chrono::Datelike;

/// Generate soramoyou index page filename
pub fn soramoyou_index_filename() -> String {
    "soramoyouindex.html".to_string()
}

/// Generate soramoyou year page filename
pub fn soramoyou_year_filename(year: i32) -> String {
    format!("soramoyou{}.html", year)
}
