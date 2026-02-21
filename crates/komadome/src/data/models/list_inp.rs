use serde::{Deserialize, Serialize};

/// List Inp data (per-person WIP work lists)
/// From list_inp.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInpData {
    pub person_id: i64,
    pub person_name: String,
    pub page: usize,
    pub total_pages: usize,
    pub works: Vec<ListInpWorkItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInpWorkItem {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub kana_type_name: Option<String>,
    pub translator_text: Option<String>,
    pub inputer_text: Option<String>,
    pub proofreader_text: Option<String>,
    pub work_status_name: Option<String>,
    pub started_on: Option<String>,
    pub teihon_title: Option<String>,
    pub teihon_publisher: Option<String>,
    pub teihon_input_edition: Option<String>,
}
