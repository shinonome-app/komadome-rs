use anyhow::Result;
use serde_json::{json, Value};

use crate::data::models::TopPageData;

/// Build context for the top page (index.html)
pub fn build_top_context(data: &TopPageData) -> Result<Value> {
    let new_works: Vec<Value> = data
        .new_works
        .iter()
        .map(|w| {
            let card_person_dir = w
                .card_person_id
                .map(|id| format!("{:06}", id))
                .unwrap_or_default();
            json!({
                "work_id": w.work_id,
                "title": &w.title,
                "subtitle": w.subtitle.as_deref().unwrap_or(""),
                "author_text": w.author_text.as_deref().unwrap_or(""),
                "card_person_dir": card_person_dir,
            })
        })
        .collect();

    let topics: Vec<Value> = data
        .topics
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "title": &t.title,
                "published_on": t.published_on.as_deref().unwrap_or(""),
                "year": t.year.unwrap_or(0),
            })
        })
        .collect();

    Ok(json!({
        "page_title": "青空文庫",
        "bgcolor": "bg-white-100",
        "new_works": new_works,
        "new_works_published_on": data.new_works_published_on.as_deref().unwrap_or(""),
        "latest_news_published_on": data.latest_news_published_on.as_deref().unwrap_or(""),
        "topics": topics,
        "works_count": data.works_count,
        "works_copyright_count": data.works_copyright_count,
        "works_noncopyright_count": data.works_noncopyright_count,
        "editable_content_html": "",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_context_matches_contract() {
        let fixture_path = format!(
            "{}/tests/fixtures/top_page_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let data: TopPageData = serde_json::from_str(
            &std::fs::read_to_string(&fixture_path).unwrap(),
        )
        .unwrap();

        let ctx = build_top_context(&data).unwrap();

        let contract_source = include_str!("../../../../../contracts/top/index.ntzc");
        let contract = subaru::parse(contract_source).unwrap();
        subaru::validate(&contract, &ctx).unwrap();
    }
}
