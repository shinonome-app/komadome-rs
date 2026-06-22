use serde::{Deserialize, Serialize};

/// ページネーション情報 (1-origin の現在ページと総ページ数)。
///
/// 複数のインデックス系 DTO が共通で持つフィールド対を 1 つの値オブジェクトに集約する。
/// `#[serde(flatten)]` で埋め込むため JSONL 上は従来どおり `page` / `total_pages` が
/// トップレベルに並ぶ (表現は不変)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: usize,
    pub total_pages: usize,
}

impl Pagination {
    /// 前ページが存在するか。
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    /// 次ページが存在するか。
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    /// ページ送り UI を表示すべきか (2 ページ以上ある)。
    pub fn has_pagination(&self) -> bool {
        self.total_pages > 1
    }
}
