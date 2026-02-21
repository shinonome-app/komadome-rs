use anyhow::Result;
use serde_json::{json, Value};

use crate::data::models::PersonAllIndexData;
use crate::generator::kana::COLUMN_CHARS;

const COLUMN_DISPLAY: &[(&str, &str)] = &[
    ("a", "あ"),
    ("ka", "か"),
    ("sa", "さ"),
    ("ta", "た"),
    ("na", "な"),
    ("ha", "は"),
    ("ma", "ま"),
    ("ya", "や"),
    ("ra", "ら"),
    ("wa", "わ"),
    ("zz", "他"),
];

pub fn build_person_all_index_context(data: &PersonAllIndexData) -> Result<Value> {
    // For "zz" column (empty column_display), Rails renders no sections on per-column page.
    // The sections data is still used by the consolidated page builder.
    let sections: Vec<Value> = if data.column_display.is_empty() {
        vec![]
    } else {
        data.sections
            .iter()
            .map(|s| {
                let people: Vec<Value> = s
                    .people
                    .iter()
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "name": p.name,
                            "published_count": p.published_count,
                            "unpublished_count": p.unpublished_count,
                            "copyright_flag": p.copyright_flag,
                        })
                    })
                    .collect();
                json!({
                    "kana_char": s.kana_char,
                    "section_index": s.section_index,
                    "people": people,
                })
            })
            .collect()
    };

    let column_nav: Vec<Value> = COLUMN_CHARS
        .iter()
        .map(|(col, _)| {
            let display = COLUMN_DISPLAY
                .iter()
                .find(|(k, _)| k == col)
                .map(|(_, v)| *v)
                .unwrap_or("他");
            json!({
                "column": col,
                "display": display,
                "is_current": *col == data.kana_column,
            })
        })
        .collect();

    Ok(json!({
        "page_title": format!("登録全作家　作家リスト：{}行 | 青空文庫", data.column_display),
        "bgcolor": "bg-sky-50",
        "kana_column": data.kana_column,
        "column_display": data.column_display,
        "sections": sections,
        "column_nav": column_nav,
    }))
}

pub fn person_all_index_filename(kana_column: &str) -> String {
    format!("person_all_{}.html", kana_column)
}

/// Build context for the consolidated person_all.html page (all columns merged)
pub fn build_person_all_consolidated_context(all_data: &[PersonAllIndexData]) -> Result<Value> {
    // Merge all sections from all columns, re-indexing section_index
    let mut all_sections: Vec<Value> = Vec::new();
    let mut section_index = 1;

    for data in all_data {
        for s in &data.sections {
            let people: Vec<Value> = s
                .people
                .iter()
                .map(|p| {
                    json!({
                        "id": p.id,
                        "name": p.name,
                        "published_count": p.published_count,
                        "unpublished_count": p.unpublished_count,
                        "total_count": p.total_count,
                        "copyright_flag": p.copyright_flag,
                    })
                })
                .collect();
            all_sections.push(json!({
                "kana_char": s.kana_char,
                "section_index": section_index,
                "people": people,
            }));
            section_index += 1;
        }
    }

    let column_nav: Vec<Value> = COLUMN_CHARS
        .iter()
        .map(|(col, _)| {
            let display = COLUMN_DISPLAY
                .iter()
                .find(|(k, _)| k == col)
                .map(|(_, v)| *v)
                .unwrap_or("他");
            json!({
                "column": col,
                "display": display,
            })
        })
        .collect();

    let kana_all: Vec<Value> = all_sections
        .iter()
        .map(|s| {
            json!({
                "char": s["kana_char"],
                "section_index": s["section_index"],
            })
        })
        .collect();

    Ok(json!({
        "page_title": "公開中　作家リスト：全て | 青空文庫",
        "bgcolor": "bg-sky-50",
        "kana_all": kana_all,
        "sections": all_sections,
        "column_nav": column_nav,
    }))
}
