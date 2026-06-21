use anyhow::Result;
use serde_json::{Value, json};

use crate::data::models::WorkIndexData;
use crate::generator::kana::Kana;

const PAGE_SIZE: usize = 50;

/// Build work index page context
pub fn build_work_index_context(data: &WorkIndexData) -> Result<Value> {
    let kana = Kana::from_symbol(&data.kana_symbol);
    let display_char = kana.and_then(|k| k.display_char()).unwrap_or("");

    let page_offset = (data.page - 1) * PAGE_SIZE;

    let ctx = json!({
        "page_title": format!("公開中　作品一覧：{} | 青空文庫", display_char),
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "kana_symbol": data.kana_symbol,
        "display_char": display_char,
        "page": data.page,
        "total_pages": data.total_pages,
        "has_prev": data.page > 1,
        "has_next": data.page < data.total_pages,
        "prev_page": super::prev_page(data.page),
        "next_page": super::next_page(data.page, data.total_pages),

        "works": data.works.iter().enumerate().map(|(i, w)| json!({
            "id": w.id,
            "title": &w.title,
            "title_kana": w.title_kana.as_deref().unwrap_or(""),
            "subtitle": w.subtitle.as_deref().unwrap_or(""),
            "author_name": w.author_name.as_deref().unwrap_or(""),
            "person_id": w.person_id,
            "has_person_id": w.person_id.is_some(),
            "card_path": format!("/cards/{}/card{}.html",
                w.card_person_id.as_deref().unwrap_or(&format!("{:06}", w.person_id.unwrap_or(0))),
                w.id),
            "row_number": page_offset + i + 1,
            "kana_type": w.kana_type.as_deref().unwrap_or(""),
            "author_text": w.author_text.as_deref().unwrap_or(""),
            "base_author_text": w.base_author_text.as_deref().unwrap_or(""),
            "translator_text": w.translator_text.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),

        // Pagination info for building URLs
        "pagination": build_pagination(data.page, data.total_pages),
    });

    Ok(ctx)
}

/// Build pagination series matching Pagy's series method with size=13 and ends=true.
///
/// This replicates the Ruby Pagy gem's series algorithm:
/// - If total <= size, show all pages
/// - Otherwise, show `size` consecutive pages centered on current, with first/last pages
///   and gaps when ends=true and size >= 7
pub fn build_pagination(current: usize, total: usize) -> Vec<Value> {
    build_pagination_series(current, total, 13)
        .iter()
        .map(PagyItem::to_json)
        .collect()
}

#[derive(Debug, Clone)]
enum PagyItem {
    Page(usize),
    Current(usize),
    Gap,
}

impl PagyItem {
    fn to_json(&self) -> Value {
        match self {
            PagyItem::Page(p) => json!({
                "page": p,
                "is_current": false,
                "is_gap": false,
            }),
            PagyItem::Current(p) => json!({
                "page": p,
                "is_current": true,
                "is_gap": false,
            }),
            PagyItem::Gap => json!({
                "page": null,
                "is_current": false,
                "is_gap": true,
            }),
        }
    }
}

const LINK_CLASS: &str =
    "text-blue-700 hover:text-gray-100 hover:bg-blue-700 visited:text-purple-600 underline";

/// Build pagination nav HTML matching Rails' PagyNav component output.
///
/// Generates the inner content of the `<nav>` element for pagination.
/// The `page_url_fn` closure takes a page number and returns the URL for that page.
pub fn build_pagination_nav_html(
    current: usize,
    total: usize,
    page_url_fn: impl Fn(usize) -> String,
) -> String {
    let series = build_pagination_series(current, total, 13);
    let mut parts = Vec::new();

    // Prev link
    if current > 1 {
        parts.push(format!(
            "  <a class=\"{}\" rel=\"prev\" aria-label=\"previous\" href=\"{}\">前の50件</a>",
            LINK_CLASS,
            page_url_fn(current - 1)
        ));
    } else {
        parts.push("  <span class=\"prev disabled\"><!-- 前の50件 --></span>".to_string());
    }

    parts.push("  <span class=\"px-1\">&nbsp;</span>".to_string());
    parts.push("  ページ:".to_string());

    // Page links
    for item in &series {
        match item {
            PagyItem::Gap => {
                parts.push("  <span class=\"page gap\">&hellip;</span>".to_string());
            }
            PagyItem::Current(p) => {
                parts.push(format!("  <span class=\"text-2xl\">{p}</span>"));
            }
            PagyItem::Page(p) => {
                parts.push(format!(
                    "  <a class=\"{}\" href=\"{}\">{}</a>",
                    LINK_CLASS,
                    page_url_fn(*p),
                    p
                ));
            }
        }
    }

    // Next link
    if current < total {
        parts.push("  <span class=\"px-1\">&nbsp;</span>".to_string());
        parts.push(format!(
            "  <a class=\"{}\" rel=\"next\" aria-label=\"next\" href=\"{}\">次の50件</a>",
            LINK_CLASS,
            page_url_fn(current + 1)
        ));
    } else {
        parts.push("  <span class=\"next disabled\"><!-- 次の50件 --></span>".to_string());
    }

    parts.join("\n")
}

/// Build pagination series (internal, returns PagyItem vec)
fn build_pagination_series(current: usize, total: usize, size: usize) -> Vec<PagyItem> {
    if total == 0 || size == 0 {
        return vec![];
    }

    let mut series: Vec<PagyItem> = Vec::new();

    if size >= total {
        for p in 1..=total {
            series.push(if p == current {
                PagyItem::Current(p)
            } else {
                PagyItem::Page(p)
            });
        }
    } else {
        let left = (size - 1) / 2;

        let start = if current <= left {
            1
        } else if current > total - size + left {
            total - size + 1
        } else {
            current - left
        };

        for p in start..start + size {
            series.push(if p == current {
                PagyItem::Current(p)
            } else {
                PagyItem::Page(p)
            });
        }

        if size >= 7 {
            series[0] = if current == 1 {
                PagyItem::Current(1)
            } else {
                PagyItem::Page(1)
            };
            if let PagyItem::Page(p) | PagyItem::Current(p) = series[1] {
                if p != 2 {
                    series[1] = PagyItem::Gap;
                }
            }
            let last_idx = series.len() - 1;
            if let PagyItem::Page(p) | PagyItem::Current(p) = series[last_idx - 1] {
                if p != total - 1 {
                    series[last_idx - 1] = PagyItem::Gap;
                }
            }
            series[last_idx] = if current == total {
                PagyItem::Current(total)
            } else {
                PagyItem::Page(total)
            };
        }
    }

    series
}

/// Generate the output filename for a work index page
pub fn work_index_filename(kana_symbol: &str, page: usize) -> String {
    format!("sakuhin_{kana_symbol}{page}.html")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract page series as a string like Pagy's series output
    /// Page numbers as integers, current page as "N" (string), :gap as :gap
    fn series_to_string(pagination: &[Value]) -> String {
        pagination
            .iter()
            .map(|item| {
                if item["is_gap"].as_bool().unwrap_or(false) {
                    ":gap".to_string()
                } else if item["is_current"].as_bool().unwrap_or(false) {
                    format!("\"{}\"", item["page"].as_u64().unwrap())
                } else {
                    format!("{}", item["page"].as_u64().unwrap())
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[test]
    fn test_pagination_all_pages_shown_when_total_lte_size() {
        // page=1, total=6 (size=13 >= 6): show all
        let result = build_pagination(1, 6);
        assert_eq!(series_to_string(&result), "\"1\", 2, 3, 4, 5, 6");

        // page=3, total=6
        let result = build_pagination(3, 6);
        assert_eq!(series_to_string(&result), "1, 2, \"3\", 4, 5, 6");
    }

    #[test]
    fn test_pagination_single_page() {
        let result = build_pagination(1, 1);
        assert_eq!(series_to_string(&result), "\"1\"");
    }

    #[test]
    fn test_pagination_exactly_13_pages() {
        let result = build_pagination(1, 13);
        assert_eq!(
            series_to_string(&result),
            "\"1\", 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13"
        );

        let result = build_pagination(7, 13);
        assert_eq!(
            series_to_string(&result),
            "1, 2, 3, 4, 5, 6, \"7\", 8, 9, 10, 11, 12, 13"
        );
    }

    #[test]
    fn test_pagination_beginning_pages() {
        // page=1, total=20: start at beginning
        let result = build_pagination(1, 20);
        assert_eq!(
            series_to_string(&result),
            "\"1\", 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, :gap, 20"
        );
    }

    #[test]
    fn test_pagination_middle_pages() {
        // page=10, total=20: intermediate
        let result = build_pagination(10, 20);
        assert_eq!(
            series_to_string(&result),
            "1, :gap, 6, 7, 8, 9, \"10\", 11, 12, 13, 14, :gap, 20"
        );
    }

    #[test]
    fn test_pagination_end_pages() {
        // page=20, total=20: end pages
        let result = build_pagination(20, 20);
        assert_eq!(
            series_to_string(&result),
            "1, :gap, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, \"20\""
        );
    }

    #[test]
    fn test_pagination_near_beginning() {
        // page=5, total=20
        // left=6, current=5 <= left=6, so start=1
        // series=[1,2,3,4,5,6,7,8,9,10,11,12,13]
        // After ends: [1,2,3,4,5,6,7,8,9,10,11,:gap,20]
        // Current=5: [1, 2, 3, 4, "5", 6, 7, 8, 9, 10, 11, :gap, 20]
        let result = build_pagination(5, 20);
        assert_eq!(
            series_to_string(&result),
            "1, 2, 3, 4, \"5\", 6, 7, 8, 9, 10, 11, :gap, 20"
        );
    }

    #[test]
    fn test_pagination_near_end() {
        // page=16, total=20
        // left=6, current=16 > total-size+left = 20-13+6 = 13, so end pages
        // start = 20 - 13 + 1 = 8
        // series = [8,9,10,11,12,13,14,15,16,17,18,19,20]
        // After ends: [1,:gap,10,11,12,13,14,15,16,17,18,19,20]
        // Current=16: [1,:gap,10,11,12,13,14,15,"16",17,18,19,20]
        let result = build_pagination(16, 20);
        assert_eq!(
            series_to_string(&result),
            "1, :gap, 10, 11, 12, 13, 14, 15, \"16\", 17, 18, 19, 20"
        );
    }

    #[test]
    fn test_pagination_14_pages() {
        // page=1, total=14: size=13 < 14
        // left=6, current=1 <= left=6, start=1
        // series=[1,2,3,4,5,6,7,8,9,10,11,12,13]
        // ends: series[0]=1, series[1]=2 (==2, no gap), series[-2]=12, 12!=13, gap!
        // series[-1]=14
        // Result: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, :gap, 14]
        // Wait but series[-2] is series[11]=12, and total-1=13, 12!=13 so gap
        let result = build_pagination(1, 14);
        assert_eq!(
            series_to_string(&result),
            "\"1\", 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, :gap, 14"
        );
    }
}
