use serde::{Deserialize, Serialize};

use super::kana_index::{KanaIndexData, KanaIndexSection};
use super::pagination::Pagination;

/// WIP Work index data (for WIP index pages)
/// From wip_work_indexes.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipWorkIndexData {
    pub kana_symbol: String,
    #[serde(flatten)]
    pub pagination: Pagination,
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

/// WIP Person index data (from wip_person_indexes.jsonl)
pub type WipPersonIndexData = KanaIndexData<WipPersonItem>;
pub type WipPersonSection = KanaIndexSection<WipPersonItem>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipPersonItem {
    pub id: i64,
    pub name: String,
    pub unpublished_count: i64,
    pub copyright_flag: bool,
}
