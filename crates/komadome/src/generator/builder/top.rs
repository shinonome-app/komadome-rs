use anyhow::Result;
use serde_json::{Value, json};

use crate::data::models::TopPageData;

/// Build context for the top page (index.html)
pub fn build_top_context(data: &TopPageData) -> Result<Value> {
    let new_works: Vec<Value> = data
        .new_works
        .iter()
        .map(|w| {
            let card_person_dir = w
                .card_person_id
                .map(|id| format!("{id:06}"))
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

    let mut ctx = json!({
        "page_title": "青空文庫",
        "bgcolor": crate::tailwind::bgcolor::WHITE,
        "new_works": new_works,
        "new_works_published_on": data.new_works_published_on.as_deref().unwrap_or(""),
        "latest_news_published_on": data.latest_news_published_on.as_deref().unwrap_or(""),
        "topics": topics,
        "works_count": data.works_count,
        "works_copyright_count": data.works_copyright_count,
        "works_noncopyright_count": data.works_noncopyright_count,
        "editable_content_html": "",
    });

    // Render the natsuzora fragment edited in shinonome admin with the base
    // context. `parse` (not `parse_with_includes`) keeps {[!include]} disabled,
    // matching shinonome's EditableContentRenderer. On error, fall back to ""
    // so the template renders the default /top/body instead of breaking.
    if let Some(src) = data
        .editable_content
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        match natsuzora::Natsuzora::parse(src).and_then(|t| t.render(ctx.clone())) {
            Ok(html) => ctx["editable_content_html"] = Value::String(html),
            Err(e) => {
                eprintln!("WARN: editable_content render failed; falling back to /top/body: {e}")
            }
        }
    }

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_data() -> TopPageData {
        let fixture_path = format!(
            "{}/tests/fixtures/top_page_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).unwrap()).unwrap()
    }

    fn validate_contract(ctx: &Value) {
        let contract_source = include_str!("../../../../../contracts/top/index.ntzc");
        let contract = natsuzora_contract::parse(contract_source).unwrap();
        natsuzora_contract::validate(&contract, ctx).unwrap();
    }

    #[test]
    fn top_context_matches_contract() {
        let ctx = build_top_context(&fixture_data()).unwrap();
        validate_contract(&ctx);
    }

    #[test]
    fn renders_editable_content_fragment() {
        let mut data = fixture_data();
        data.editable_content = Some("<p>total: {[ works_count ]}</p>".to_string());

        let ctx = build_top_context(&data).unwrap();

        let expected = format!("<p>total: {}</p>", data.works_count);
        assert_eq!(ctx["editable_content_html"], Value::String(expected));
        validate_contract(&ctx);
    }

    #[test]
    fn falls_back_to_empty_on_invalid_fragment() {
        let mut data = fixture_data();
        data.editable_content = Some("{[#if".to_string());

        let ctx = build_top_context(&data).unwrap();

        assert_eq!(ctx["editable_content_html"], Value::String(String::new()));
    }

    #[test]
    fn falls_back_to_empty_when_fragment_uses_include() {
        let mut data = fixture_data();
        data.editable_content = Some("{[!include /top/body]}".to_string());

        let ctx = build_top_context(&data).unwrap();

        assert_eq!(ctx["editable_content_html"], Value::String(String::new()));
    }
}
