use anyhow::Result;
use serde_json::{json, Value};

use crate::data::models::PersonIndexData;
use crate::generator::kana::COLUMN_CHARS;

/// Build person index page context
pub fn build_person_index_context(data: &PersonIndexData) -> Result<Value> {
    // Get display character for the column
    let display_char = COLUMN_CHARS
        .iter()
        .find(|(sym, _)| *sym == data.kana_column)
        .map(|(sym, chars)| {
            if *sym == "zz" {
                "他".to_string()
            } else {
                chars.chars().next().unwrap_or('あ').to_string()
            }
        })
        .unwrap_or_else(|| data.kana_column.clone());

    let ctx = json!({
        "page_title": format!("人物インデックス：{}行 | 青空文庫", display_char),
        "bgcolor": "bg-sky-50",
        "kana_column": data.kana_column,
        "display_char": display_char,

        "people": data.people.iter().map(|p| json!({
            "id": p.id,
            "name": p.name,
            "name_kana": p.name_kana,
            "work_count": p.work_count,
            "copyright_flag": p.copyright_flag,
            "person_path": format!("/index_pages/person{}.html", p.id),
        })).collect::<Vec<_>>(),

        // Navigation columns
        "columns": COLUMN_CHARS.iter().map(|(sym, chars)| {
            let display = if *sym == "zz" {
                "他".to_string()
            } else {
                chars.chars().next().unwrap_or('あ').to_string()
            };
            let is_current = *sym == data.kana_column;
            json!({
                "symbol": sym,
                "display": display,
                "is_current": is_current,
            })
        }).collect::<Vec<_>>(),
    });

    Ok(ctx)
}

/// Generate the output filename for a person index page
pub fn person_index_filename(kana_column: &str) -> String {
    format!("person_{}.html", kana_column)
}
