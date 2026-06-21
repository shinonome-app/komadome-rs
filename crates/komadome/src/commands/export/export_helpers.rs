use anyhow::Result;
use serde::Serialize;
use std::io::Write;

use crate::generator::kana::COLUMN_CHARS;

pub const PAGE_SIZE: usize = 50;

pub fn calculate_total_pages(total_items: usize) -> usize {
    let pages = (total_items as f64 / PAGE_SIZE as f64).ceil() as usize;
    if pages == 0 { 1 } else { pages }
}

/// インデックスデータの列見出し用表示文字を返す。
/// 各列の代表カナ (あ/か/…) を返し、"zz"(その他) 列やカナを持たない列は空文字。
///
/// 注: footer ナビ用の [`crate::generator::kana::column_display`] は "zz" に "他" を
/// 返す点が異なる。こちらは export する JSONL の `column_display` フィールド用。
pub fn column_display(column: &str) -> String {
    COLUMN_CHARS
        .iter()
        .find(|(key, _)| *key == column)
        .and_then(|(_, chars)| chars.chars().next())
        .map(|c| c.to_string())
        .unwrap_or_default()
}

/// 列に属するカナ文字を 1 文字ずつ列挙する。
/// "zz"(その他) 列のように空の場合は "その他" 1 件を返す
/// (consolidated ページが利用できるようにするため)。
pub fn column_kana_chars(kana_chars: &str) -> Vec<String> {
    if kana_chars.is_empty() {
        vec!["その他".to_string()]
    } else {
        kana_chars.chars().map(|c| c.to_string()).collect()
    }
}

pub fn write_jsonl_line<T: Serialize>(writer: &mut impl Write, data: &T) -> Result<()> {
    serde_json::to_writer(&mut *writer, data)?;
    writer.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total_pages_zero() {
        assert_eq!(calculate_total_pages(0), 1);
    }

    #[test]
    fn test_calculate_total_pages_exact_page() {
        assert_eq!(calculate_total_pages(50), 1);
    }

    #[test]
    fn test_calculate_total_pages_one_over() {
        assert_eq!(calculate_total_pages(51), 2);
    }

    #[test]
    fn test_calculate_total_pages_two_pages() {
        assert_eq!(calculate_total_pages(100), 2);
    }

    #[test]
    fn test_calculate_total_pages_three_pages() {
        assert_eq!(calculate_total_pages(101), 3);
    }

    #[test]
    fn test_write_jsonl_line() {
        let mut buf = Vec::new();
        write_jsonl_line(&mut buf, &serde_json::json!({"key": "value"})).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "{\"key\":\"value\"}\n");
    }
}
