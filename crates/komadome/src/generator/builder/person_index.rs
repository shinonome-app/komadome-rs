use anyhow::Result;
use serde_json::{Value, json};

use crate::data::models::PersonIndexData;
use crate::generator::kana::COLUMN_CHARS;

/// Build person index page context
pub fn build_person_index_context(data: &PersonIndexData) -> Result<Value> {
    let display_char = &data.column_display;

    // For "zz" column, Rails renders empty kana_all and no sections
    // because Kana.new(:zz).to_chars returns []
    let is_zz = data.kana_column == "zz";

    // Build kana_all (array of kana characters for anchor nav)
    let kana_all: Vec<Value> = if is_zz {
        vec![]
    } else {
        data.sections
            .iter()
            .map(|s| {
                json!({
                    "char": &s.kana_char,
                    "section_index": s.section_index,
                })
            })
            .collect()
    };

    // Build sections with people
    let sections: Vec<Value> = if is_zz {
        vec![]
    } else {
        data.sections
            .iter()
            .map(|s| {
                let people: Vec<Value> = s
                    .people
                    .iter()
                    .filter(|p| p.published_works_count > 0)
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "name": &p.name,
                            "name_kana": &p.name_kana,
                            "work_count": p.work_count,
                            "copyright_flag": p.copyright_flag,
                            "published_works_count": p.published_works_count,
                            "person_path": format!("person{}.html#sakuhin_list_1", p.id),
                        })
                    })
                    .collect();
                json!({
                    "kana_char": &s.kana_char,
                    "section_index": s.section_index,
                    "people": people,
                })
            })
            .collect()
    };

    // Build footer columns nav - highlight current
    let footer_columns: Vec<Value> = COLUMN_CHARS
        .iter()
        .map(|(sym, chars)| {
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
        })
        .collect();

    let ctx = json!({
        "page_title": format!("公開中　作家リスト：{}行 | 青空文庫", display_char),
        "bgcolor": "bg-sky-50",
        "kana_column": data.kana_column,
        "display_char": display_char,
        "kana_all": kana_all,
        "sections": sections,
        "footer_columns": footer_columns,
    });

    Ok(ctx)
}

/// Generate the output filename for a person index page
pub fn person_index_filename(kana_column: &str) -> String {
    format!("person_{kana_column}.html")
}
