use serde::{Deserialize, Serialize};

/// Person All index data (all registered persons)
/// From person_all_indexes.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonAllIndexData {
    pub kana_column: String,
    pub column_display: String,
    pub sections: Vec<PersonAllSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonAllSection {
    pub kana_char: String,
    pub section_index: usize,
    pub people: Vec<PersonAllItem>,
}

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
