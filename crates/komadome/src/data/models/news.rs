use serde::{Deserialize, Serialize};

/// News data
/// From news.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsData {
    pub year: i32,
    pub entries: Vec<NewsEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsEntry {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub published_on: Option<String>,
    pub flag: bool,
}
