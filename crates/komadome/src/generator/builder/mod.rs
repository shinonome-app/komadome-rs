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

pub fn news_anchor(id: i64) -> String {
    format!("{id:06}")
}

/// 図書カードのディレクトリ名 (人物IDの6桁ゼロ埋め)。
///
/// `cards/{:06}/...` のパス規約を表現する単一の出所。export 側 (SQL の LPAD) を
/// 廃し、人物IDは i64 のまま持ち回って描画時にここで整形する。
pub fn card_person_dir(id: i64) -> String {
    format!("{id:06}")
}

/// 図書カード HTML の出力相対パス (例: "cards/000001/card12345.html")。
///
/// サイトのファイル配置規約はジェネレータ層の責務として一箇所に置き、
/// データ DTO (`CardData`) には持たせない。
pub fn card_relative_path(person_id: i64, work_id: i64) -> String {
    format!("cards/{}/card{}.html", card_person_dir(person_id), work_id)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn news_anchor_zero_pads_id_to_six_digits() {
        assert_eq!(news_anchor(537), "000537");
        assert_eq!(news_anchor(101), "000101");
        assert_eq!(news_anchor(1), "000001");
        assert_eq!(news_anchor(1234567), "1234567");
    }

    #[test]
    fn card_relative_path_zero_pads_person_dir() {
        assert_eq!(
            card_relative_path(100, 12345),
            "cards/000100/card12345.html"
        );
    }
}
