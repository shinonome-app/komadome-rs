use serde::{Deserialize, Serialize};

use super::kana_index::{KanaIndexData, KanaIndexSection};

/// Person index data (from person_indexes.jsonl)
pub type PersonIndexData = KanaIndexData<PersonIndexItem>;
pub type PersonIndexSection = KanaIndexSection<PersonIndexItem>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonIndexItem {
    pub id: i64,
    pub name: String,
    pub name_kana: String,
    pub work_count: usize,
    pub copyright_flag: bool,
    #[serde(default)]
    pub published_works_count: usize,
}
