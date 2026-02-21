use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Work (作品) - basic model, not used for JSONL import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    pub id: i64,
    pub title: String,
    pub title_kana: Option<String>,
    pub subtitle: Option<String>,
    pub original_title: Option<String>,
    pub author_display_name: Option<String>,
    pub copyright_flag: bool,
    pub description: Option<String>,
    pub note: Option<String>,
    pub sortkey: Option<String>,
    pub started_on: Option<NaiveDate>,
    pub work_status_id: i64,
    pub kana_type_id: i64,
}
