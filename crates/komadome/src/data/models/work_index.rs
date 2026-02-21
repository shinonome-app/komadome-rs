use serde::{Deserialize, Serialize};

/// Work index data (for index pages)
/// From work_indexes.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkIndexData {
    pub kana_symbol: String,
    pub page: usize,
    pub total_pages: usize,
    pub works: Vec<WorkIndexItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkIndexItem {
    pub id: i64,
    pub title: String,
    pub title_kana: Option<String>,
    pub subtitle: Option<String>,
    pub author_name: Option<String>,
    pub person_id: Option<i64>,
    #[serde(default)]
    pub card_person_id: Option<String>,
    #[serde(default)]
    pub kana_type: Option<String>,
    #[serde(default)]
    pub author_text: Option<String>,
    #[serde(default)]
    pub base_author_text: Option<String>,
    #[serde(default)]
    pub translator_text: Option<String>,
}
