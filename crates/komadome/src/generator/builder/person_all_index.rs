use anyhow::Result;
use serde_json::{Value, json};

use super::{build_column_nav, build_kana_all};
use crate::data::models::PersonAllIndexData;

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

    let column_nav = build_column_nav(Some(&data.kana_column));

    Ok(json!({
        "page_title": format!("登録全作家　作家リスト：{}行 | 青空文庫", data.column_display),
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "kana_column": data.kana_column,
        "column_display": data.column_display,
        "sections": sections,
        "column_nav": column_nav,
    }))
}

pub fn person_all_index_filename(kana_column: &str) -> String {
    format!("person_all_{kana_column}.html")
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

    let column_nav = build_column_nav(None);
    let kana_all = build_kana_all(&all_sections);

    Ok(json!({
        "page_title": "公開中　作家リスト：全て | 青空文庫",
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "kana_all": kana_all,
        "sections": all_sections,
        "column_nav": column_nav,
    }))
}
