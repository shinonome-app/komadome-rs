use serde::{Deserialize, Serialize};

/// Whatsnew data
/// From whatsnew.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsnewData {
    pub year: Option<i32>,
    pub page: usize,
    pub total_pages: usize,
    pub entries: Vec<WhatsnewEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsnewEntry {
    pub work_id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub card_person_id: Option<i64>,
    pub author_text: Option<String>,
    pub inputer_text: Option<String>,
    pub proofreader_text: Option<String>,
    pub translator_text: Option<String>,
    pub started_on: Option<String>,
}
