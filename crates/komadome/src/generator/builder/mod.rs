pub mod card;
pub mod list_inp;
pub mod person;
pub mod person_all_index;
pub mod person_index;
pub mod soramoyou;
pub mod static_pages;
pub mod top;
pub mod whatsnew;
pub mod wip_person_index;
pub mod wip_work_index;
pub mod work_index;

use serde_json::{Value, json};

use crate::generator::kana::{COLUMN_CHARS, column_display};

/// Previous page number, or `None` on the first page.
pub fn prev_page(page: usize) -> Option<usize> {
    (page > 1).then(|| page - 1)
}

/// Next page number, or `None` on the last page.
pub fn next_page(page: usize, total_pages: usize) -> Option<usize> {
    (page < total_pages).then(|| page + 1)
}

/// Convert newlines to `<br>` (HTML5 style, matching Rails sanitize output).
pub fn nl2br(s: &str) -> String {
    s.replace("\r\n", "<br>").replace('\n', "<br>")
}

/// Build the kana column footer navigation shared by the 作家リスト pages.
///
/// Pass `Some(column)` to mark the current column with `is_current`; pass `None`
/// for the consolidated pages that have no single "current" column.
pub fn build_column_nav(current: Option<&str>) -> Vec<Value> {
    COLUMN_CHARS
        .iter()
        .map(|(col, _)| {
            let display = column_display(col);
            match current {
                Some(cur) => json!({
                    "column": col,
                    "display": display,
                    "is_current": *col == cur,
                }),
                None => json!({
                    "column": col,
                    "display": display,
                }),
            }
        })
        .collect()
}

/// Build the kana anchor list (`kana_all`) from already-built section values.
pub fn build_kana_all(sections: &[Value]) -> Vec<Value> {
    sections
        .iter()
        .map(|s| {
            json!({
                "char": s["kana_char"],
                "section_index": s["section_index"],
            })
        })
        .collect()
}
