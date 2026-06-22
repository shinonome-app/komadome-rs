use anyhow::Result;
use chrono::NaiveDate;
use serde_json::{Value, json};

use crate::data::models::WhatsnewData;

use super::pagination::build_pagination;

/// Build whatsnew index page context (current year)
pub fn build_whatsnew_index_context(
    data: &WhatsnewData,
    today: &NaiveDate,
    year_links: &[i32],
) -> Result<Value> {
    let current_year = chrono::Datelike::year(today);
    let recent_cutoff = *today - chrono::Duration::days(7);

    let pg = &data.pagination;

    let ctx = json!({
        "page_title": "新規公開作品 | 青空文庫",
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "current_year": current_year,
        "today": today.format("%Y.%m.%d").to_string(),
        "page": pg.page,
        "total_pages": pg.total_pages,
        "has_prev": pg.has_prev(),
        "has_next": pg.has_next(),
        "prev_page": super::prev_page(pg.page),
        "next_page": super::next_page(pg.page, pg.total_pages),
        "pagination": build_pagination(pg.page, pg.total_pages),
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
    let pg = &data.pagination;

    let ctx = json!({
        "page_title": format!("公開作品 {}年公開分 | 青空文庫", year),
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "year": year,
        "today": today.format("%Y.%m.%d").to_string(),
        "page": pg.page,
        "total_pages": pg.total_pages,
        "has_prev": pg.has_prev(),
        "has_next": pg.has_next(),
        "prev_page": super::prev_page(pg.page),
        "next_page": super::next_page(pg.page, pg.total_pages),
        "pagination": build_pagination(pg.page, pg.total_pages),
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
    use super::super::pagination::build_pagination_nav_html;
    use super::*;

    /// whatsnew index 用ページネーション nav の構造化テンプレート版。
    /// `templates/indexes/works.ntzr` の nav ブロックと同型で、URL だけ whatsnew 用。
    /// この markup を whatsnew/index.ntzr の `<nav>` にそのまま埋め込む。
    const NAV_INDEX: &str = r#"        <nav aria-label="pager" class="pagy_nav pagination" role="navigation">{[-#if has_prev]}
  <a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" rel="prev" aria-label="previous" href="/index_pages/whatsnew{[ prev_page ]}.html">前の50件</a>{[/if]}{[-#unless has_prev]}
  <span class="prev disabled"><!-- 前の50件 --></span>{[/unless]}
  <span class="px-1">&nbsp;</span>
  ページ:{[-#each pagination as item]}{[-#if item.is_gap]}
  <span class="page gap">&hellip;</span>{[/if]}{[-#unless item.is_gap]}{[-#if item.is_current]}
  <span class="text-2xl">{[ item.page ]}</span>{[/if]}{[-#unless item.is_current]}
  <a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" href="/index_pages/whatsnew{[ item.page ]}.html">{[ item.page ]}</a>{[/unless]}{[/unless]}{[/each]}{[-#if has_next]}
  <span class="px-1">&nbsp;</span>
  <a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" rel="next" aria-label="next" href="/index_pages/whatsnew{[ next_page ]}.html">次の50件</a>{[/if]}{[-#unless has_next]}
  <span class="next disabled"><!-- 次の50件 --></span>{[/unless]}
</nav>"#;

    /// whatsnew year 用。NAV_INDEX と URL パターンのみ違う (`whatsnew_{year}_{page}.html`)。
    const NAV_YEAR: &str = r#"        <nav aria-label="pager" class="pagy_nav pagination" role="navigation">{[-#if has_prev]}
  <a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" rel="prev" aria-label="previous" href="/index_pages/whatsnew_{[ year ]}_{[ prev_page ]}.html">前の50件</a>{[/if]}{[-#unless has_prev]}
  <span class="prev disabled"><!-- 前の50件 --></span>{[/unless]}
  <span class="px-1">&nbsp;</span>
  ページ:{[-#each pagination as item]}{[-#if item.is_gap]}
  <span class="page gap">&hellip;</span>{[/if]}{[-#unless item.is_gap]}{[-#if item.is_current]}
  <span class="text-2xl">{[ item.page ]}</span>{[/if]}{[-#unless item.is_current]}
  <a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" href="/index_pages/whatsnew_{[ year ]}_{[ item.page ]}.html">{[ item.page ]}</a>{[/unless]}{[/unless]}{[/each]}{[-#if has_next]}
  <span class="px-1">&nbsp;</span>
  <a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" rel="next" aria-label="next" href="/index_pages/whatsnew_{[ year ]}_{[ next_page ]}.html">次の50件</a>{[/if]}{[-#unless has_next]}
  <span class="next disabled"><!-- 次の50件 --></span>{[/unless]}
</nav>"#;

    /// テンプレート描画した nav が、旧 `build_pagination_nav_html` を `<nav>` で
    /// 包んだ出力とバイト単位で一致することを確認する（テンプレート移譲の回帰防止）。
    #[test]
    fn structured_nav_template_matches_legacy_html() {
        let cases = [
            (1, 1),
            (1, 6),
            (3, 6),
            (2, 5),
            (1, 20),
            (10, 20),
            (20, 20),
            (5, 20),
            (16, 20),
            (1, 14),
        ];

        let nav_index =
            natsuzora::Natsuzora::parse_with_includes(NAV_INDEX, std::path::Path::new("."))
                .unwrap();
        let nav_year =
            natsuzora::Natsuzora::parse_with_includes(NAV_YEAR, std::path::Path::new(".")).unwrap();

        for (cur, total) in cases {
            let mut ctx = json!({
                "has_prev": super::super::prev_page(cur).is_some(),
                "has_next": super::super::next_page(cur, total).is_some(),
                "prev_page": super::super::prev_page(cur),
                "next_page": super::super::next_page(cur, total),
                "pagination": build_pagination(cur, total),
            });

            // index URL パターン
            let rendered = nav_index.render(ctx.clone()).unwrap();
            let expected = wrap_nav(build_pagination_nav_html(cur, total, |p| {
                format!("/index_pages/whatsnew{p}.html")
            }));
            assert_eq!(rendered, expected, "index mismatch for page {cur}/{total}");

            // year URL パターン
            ctx["year"] = json!(2023);
            let rendered = nav_year.render(ctx).unwrap();
            let expected = wrap_nav(build_pagination_nav_html(cur, total, |p| {
                format!("/index_pages/whatsnew_2023_{p}.html")
            }));
            assert_eq!(rendered, expected, "year mismatch for page {cur}/{total}");
        }
    }

    fn wrap_nav(inner: String) -> String {
        format!(
            "        <nav aria-label=\"pager\" class=\"pagy_nav pagination\" role=\"navigation\">\n{inner}\n</nav>"
        )
    }

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
