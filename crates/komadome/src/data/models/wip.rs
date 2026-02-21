use serde::{Deserialize, Serialize};

/// WIP Work index data (for WIP index pages)
/// From wip_work_indexes.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipWorkIndexData {
    pub kana_symbol: String,
    pub page: usize,
    pub total_pages: usize,
    pub works: Vec<WipWorkIndexItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipWorkIndexItem {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub kana_type_name: Option<String>,
    pub author_name: Option<String>,
    pub author_id: Option<i64>,
    pub base_author_name: Option<String>,
    pub translator_text: Option<String>,
    pub inputer_text: Option<String>,
    pub proofreader_text: Option<String>,
    pub work_status_name: Option<String>,
    pub started_on: Option<String>,
    pub teihon_title: Option<String>,
    pub teihon_publisher: Option<String>,
    pub teihon_input_edition: Option<String>,
}

/// WIP Person index data
/// From wip_person_indexes.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipPersonIndexData {
    pub kana_column: String,
    pub column_display: String,
    pub sections: Vec<WipPersonSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipPersonSection {
    pub kana_char: String,
    pub section_index: usize,
    pub people: Vec<WipPersonItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipPersonItem {
    pub id: i64,
    pub name: String,
    pub unpublished_count: i64,
    pub copyright_flag: bool,
}
