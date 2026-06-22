use serde::{Deserialize, Serialize};

use super::kana_index::{KanaIndexData, KanaIndexSection};

/// Person All index data (all registered persons, from person_all_indexes.jsonl)
pub type PersonAllIndexData = KanaIndexData<PersonAllItem>;
pub type PersonAllSection = KanaIndexSection<PersonAllItem>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonAllItem {
    pub id: i64,
    pub name: String,
    pub published_count: i64,
    pub unpublished_count: i64,
    #[serde(default)]
    pub total_count: i64,
    pub copyright_flag: bool,
}
