use anyhow::Result;
use serde_json::{Value, json};

use crate::data::models::PersonPageData;
use crate::generator::kana::Kana;

/// Build person page context
pub fn build_person_context(data: &PersonPageData) -> Result<Value> {
    let person = &data.person;
    let full_name = person.full_name();

    let bgcolor = if person.copyright_flag {
        crate::tailwind::bgcolor::COPYRIGHT
    } else {
        crate::tailwind::bgcolor::DEFAULT
    };

    // Calculate kana and kana_fragment from sortkey
    let sortkey = person.sortkey.as_deref().unwrap_or("");
    let first_char = sortkey
        .chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_default();
    let kana_obj = Kana::from_kana(&first_char);
    let (column_symbol, index_in_column) = kana_obj.to_symbol_and_index();
    let kana_fragment = format!("sec{}", index_in_column + 1);

    let has_unpublished_works = !data.unpublished_works.is_empty();

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
        "name_en": person.name_en.as_deref().unwrap_or(""),
        "born_on": person.born_on.as_deref().unwrap_or(""),
        "died_on": person.died_on.as_deref().unwrap_or(""),
        "copyright_flag": person.copyright_flag,
        "description": person.description.as_deref().unwrap_or(""),
        "kana": column_symbol,
        "kana_fragment": kana_fragment,

        "other_base_people": data.other_base_people.iter().map(|p| json!({
            "id": p.id,
            "name": &p.name,
        })).collect::<Vec<_>>(),

        "works": data.works.iter().map(|w| json!({
            "id": w.id,
            "title": &w.title,
            "title_kana": w.title_kana.as_deref().unwrap_or(""),
            "subtitle": w.subtitle.as_deref().unwrap_or(""),
            "role": w.role.as_deref().unwrap_or(""),
            "role_id": w.role_id,
            "kana_type": w.kana_type.as_deref().unwrap_or(""),
            "card_person_id": w.card_person_id.as_deref().unwrap_or(""),
            "work_people": w.work_people.iter().map(|wp| json!({
                "person_id": wp.person_id,
                "name": &wp.name,
                "role_name": wp.role_name.as_deref().unwrap_or(""),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),

        "has_unpublished_works": has_unpublished_works,
        "unpublished_works": data.unpublished_works.iter().map(|w| json!({
            "id": w.id,
            "title": &w.title,
            "title_kana": w.title_kana.as_deref().unwrap_or(""),
            "subtitle": w.subtitle.as_deref().unwrap_or(""),
            "role": w.role.as_deref().unwrap_or(""),
            "role_id": w.role_id,
            "kana_type": w.kana_type.as_deref().unwrap_or(""),
            "card_person_id": w.card_person_id.as_deref().unwrap_or(""),
            "work_people": w.work_people.iter().map(|wp| json!({
                "person_id": wp.person_id,
                "name": &wp.name,
                "role_name": wp.role_name.as_deref().unwrap_or(""),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),

        "sites": data.sites.iter().map(|s| json!({
            "name": s.name.as_deref().unwrap_or(""),
            "url": s.url.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),
    });

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn person_context_matches_contract() {
        let fixture_path = format!(
            "{}/tests/fixtures/person_page_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let data: PersonPageData =
            serde_json::from_str(&std::fs::read_to_string(&fixture_path).unwrap()).unwrap();

        let ctx = build_person_context(&data).unwrap();

        let contract_source = include_str!("../../../../../contracts/people/show.ntzc");
        let contract = natsuzora_contract::parse(contract_source).unwrap();
        natsuzora_contract::validate(&contract, &ctx).unwrap();
    }
}
