use anyhow::Result;
use serde_json::{json, Value};

use crate::data::models::PersonPageData;

/// Build person page context
pub fn build_person_context(data: &PersonPageData) -> Result<Value> {
    let person = &data.person;
    let full_name = person.full_name();

    let bgcolor = if person.copyright_flag {
        "bg-rose-50"
    } else {
        "bg-sky-50"
    };

    let ctx = json!({
        "page_title": format!("作家別作品リスト：{} | 青空文庫", full_name),
        "bgcolor": bgcolor,
        "person_id": person.id,
        "last_name": &person.last_name,
        "first_name": person.first_name.as_deref().unwrap_or(""),
        "full_name": person.full_name(),
        "full_name_kana": person.full_name_kana(),
        "last_name_kana": &person.last_name_kana,
        "first_name_kana": person.first_name_kana.as_deref().unwrap_or(""),
        "born_on": person.born_on.as_deref().unwrap_or(""),
        "died_on": person.died_on.as_deref().unwrap_or(""),
        "copyright_flag": person.copyright_flag,
        "description": person.description.as_deref().unwrap_or(""),

        "works": data.works.iter().map(|w| json!({
            "id": w.id,
            "title": &w.title,
            "title_kana": w.title_kana.as_deref().unwrap_or(""),
            "subtitle": w.subtitle.as_deref().unwrap_or(""),
            "role": w.role.as_deref().unwrap_or(""),
            "role_id": w.role_id,
            "kana_type": w.kana_type.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),

        "sites": data.sites.iter().map(|s| json!({
            "name": s.name.as_deref().unwrap_or(""),
            "url": s.url.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),
    });

    Ok(ctx)
}
