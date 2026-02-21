use serde::{Deserialize, Serialize};

/// Person index data
/// From person_indexes.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonIndexData {
    pub kana_column: String,
    pub column_display: String,
    pub sections: Vec<PersonIndexSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonIndexSection {
    pub kana_char: String,
    pub section_index: usize,
    pub people: Vec<PersonIndexItem>,
}

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
