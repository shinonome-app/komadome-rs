use anyhow::Result;
use serde_json::{Value, json};

use super::build_column_nav;
use crate::data::models::PersonIndexData;

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

    // かな列フッター nav (現在列を is_current でハイライト)。people.ntzr が描画する。
    let column_nav = build_column_nav(Some(&data.kana_column));

    let ctx = json!({
        "page_title": format!("公開中　作家リスト：{}行 | 青空文庫", display_char),
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "kana_column": data.kana_column,
        "display_char": display_char,
        "kana_all": kana_all,
        "sections": sections,
        "column_nav": column_nav,
    });

    Ok(ctx)
}

/// Generate the output filename for a person index page
pub fn person_index_filename(kana_column: &str) -> String {
    format!("person_{kana_column}.html")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// people.ntzr のかな列フッター部分 (動的版)。
    const FOOTER: &str = r#"      <span class="pr-3">●作家リスト：公開中</span>{[#each column_nav as nav]}{[#if nav.is_current]}
      <span class="text-red-500 font-bold">[{[ nav.display ]}]</span>{[#else]}
      <span><a class="text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline" href="person_{[ nav.column ]}.html">[{[ nav.display ]}]</a></span>{[/if]}{[/each]}"#;

    fn render_footer(kana_column: &str) -> String {
        let tmpl =
            natsuzora::Natsuzora::parse_with_includes(FOOTER, std::path::Path::new(".")).unwrap();
        tmpl.render(json!({ "column_nav": build_column_nav(Some(kana_column)) }))
            .unwrap()
    }

    /// 「あ」ページのフッターは従来の静的ハードコードと完全に一致する (出力不変の保証)。
    #[test]
    fn footer_for_a_column_matches_legacy_static_html() {
        let expected = "      <span class=\"pr-3\">●作家リスト：公開中</span>
      <span class=\"text-red-500 font-bold\">[あ]</span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_ka.html\">[か]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_sa.html\">[さ]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_ta.html\">[た]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_na.html\">[な]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_ha.html\">[は]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_ma.html\">[ま]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_ya.html\">[や]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_ra.html\">[ら]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_wa.html\">[わ]</a></span>
      <span><a class=\"text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline\" href=\"person_zz.html\">[他]</a></span>";
        assert_eq!(render_footer("a"), expected);
    }

    /// 「か」ページでは か が current になり、あ がリンク (person_a.html) に変わる (バグ修正)。
    #[test]
    fn footer_for_ka_column_highlights_ka() {
        let footer = render_footer("ka");
        assert!(footer.contains("<span class=\"text-red-500 font-bold\">[か]</span>"));
        assert!(footer.contains("href=\"person_a.html\">[あ]</a>"));
        // か は current なので person_ka.html へのリンクは出さない
        assert!(!footer.contains("href=\"person_ka.html\""));
    }
}
