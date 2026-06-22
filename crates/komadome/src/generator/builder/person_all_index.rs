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

/// 全列の section をマージし section_index を振り直す (集約ページ共通)。
fn merge_all_sections(all_data: &[PersonAllIndexData]) -> Vec<Value> {
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

    all_sections
}

/// Build context for the consolidated person_all.html page (公開中, all columns merged)
pub fn build_person_all_consolidated_context(all_data: &[PersonAllIndexData]) -> Result<Value> {
    let all_sections = merge_all_sections(all_data);
    let kana_all = build_kana_all(&all_sections);
    // 集約ページのフッターも本家同様「あ」を現在列として強調する。
    let column_nav = build_column_nav(Some("a"));

    Ok(json!({
        "page_title": "公開中　作家リスト：全て | 青空文庫",
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "kana_all": kana_all,
        "sections": all_sections,
        "column_nav": column_nav,
    }))
}

/// Build context for the consolidated person_all_all.html page (登録全作家, all columns merged).
/// 本家 https://www.aozora.gr.jp/index_pages/person_all_all.html に相当する。
pub fn build_person_all_all_context(all_data: &[PersonAllIndexData]) -> Result<Value> {
    let all_sections = merge_all_sections(all_data);
    let kana_all = build_kana_all(&all_sections);
    let column_nav = build_column_nav(Some("a"));

    Ok(json!({
        "page_title": "登録全作家　作家リスト：全て | 青空文庫",
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "kana_all": kana_all,
        "sections": all_sections,
        "column_nav": column_nav,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(footer: &str, current: Option<&str>) -> String {
        let tmpl =
            natsuzora::Natsuzora::parse_with_includes(footer, std::path::Path::new(".")).unwrap();
        tmpl.render(json!({ "column_nav": build_column_nav(current) }))
            .unwrap()
    }

    /// 登録全作家 per-column フッター: ラベル「全」/ person_all_<col> リンク / 現在列強調。
    /// 本家 person_all_ka.html と同等。
    #[test]
    fn person_all_index_footer_matches_live() {
        const FOOTER: &str = r#"      <span class="pr-3">●作家リスト：全</span>{[#each column_nav as nav]}{[#if nav.is_current]}
      <span class="text-red-500 font-bold">[{[ nav.display ]}]</span>{[#else]}
      <span><a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" href="person_all_{[ nav.column ]}.html">[{[ nav.display ]}]</a></span>{[/if]}{[/each]}"#;
        let footer = render(FOOTER, Some("ka"));
        assert!(footer.contains("●作家リスト：全"));
        assert!(footer.contains("<span class=\"text-red-500 font-bold\">[か]</span>"));
        assert!(footer.contains("href=\"person_all_a.html\">[あ]</a>"));
        assert!(!footer.contains("href=\"person_all_ka.html\""));
        // 公開中ページ (person_<col>.html) ではなく登録全作家ページへリンクする
        assert!(!footer.contains("href=\"person_sa.html\""));
    }

    /// 登録全作家 集約 (person_all_all.html) のコンテキスト: タイトル・あ current・人物カウント。
    #[test]
    fn person_all_all_context_is_registered_all() {
        let path = format!(
            "{}/tests/fixtures/person_all_index_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let data: PersonAllIndexData =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let ctx = build_person_all_all_context(&[data]).unwrap();

        assert_eq!(ctx["page_title"], "登録全作家　作家リスト：全て | 青空文庫");
        assert_eq!(ctx["column_nav"][0]["column"], "a");
        assert_eq!(ctx["column_nav"][0]["is_current"], true);
        let person = &ctx["sections"][0]["people"][0];
        assert!(person.get("published_count").is_some());
        assert!(person.get("unpublished_count").is_some());
    }

    /// 実テンプレート (person_all_all.ntzr) をレンダリングし、本家相当の構造を確認する。
    #[test]
    fn person_all_all_renders_like_live() {
        let templates_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates");
        let registry = crate::generator::templates::TemplateRegistry::load(&templates_dir).unwrap();

        let path = format!(
            "{}/tests/fixtures/person_all_index_data.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let data: PersonAllIndexData =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let ctx = build_person_all_all_context(&[data]).unwrap();
        let html = registry.render("indexes/person_all_all", ctx).unwrap();

        assert!(html.contains("登録全作家　作家リスト：全て"));
        // 著者行は (公開中：N、作業中：M)
        assert!(html.contains("、作業中："));
        // フッターは登録全作家ページへリンクし「あ」を強調
        assert!(html.contains("●作家リスト：全"));
        assert!(html.contains("href=\"person_all_ka.html\""));
        assert!(html.contains("<span class=\"text-red-500 font-bold\">[あ]</span>"));
        // 公開中集約用の CSV ダウンロードリンクは持たない
        assert!(!html.contains("list_person_all.zip"));
    }

    /// 公開中 集約フッター (person_all.html): ラベル「公開中」/ person_<col> リンク / あ強調。
    /// 本家 person_all.html と同等。
    #[test]
    fn person_all_consolidated_footer_matches_live() {
        const FOOTER: &str = r#"      <span class="pr-3">●作家リスト：公開中</span>{[#each column_nav as nav]}{[#if nav.is_current]}
      <span class="text-red-500 font-bold">[{[ nav.display ]}]</span>{[#else]}
      <span><a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" href="person_{[ nav.column ]}.html">[{[ nav.display ]}]</a></span>{[/if]}{[/each]}"#;
        let footer = render(FOOTER, Some("a"));
        assert!(footer.contains("●作家リスト：公開中"));
        assert!(footer.contains("<span class=\"text-red-500 font-bold\">[あ]</span>"));
        assert!(footer.contains("href=\"person_ka.html\">[か]</a>"));
        assert!(!footer.contains("href=\"person_a.html\""));
        assert!(!footer.contains("person_all_"));
    }
}
