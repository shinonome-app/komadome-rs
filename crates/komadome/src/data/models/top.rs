use serde::{Deserialize, Serialize};

/// Top page data
/// From top.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopPageData {
    pub new_works: Vec<TopNewWork>,
    pub new_works_published_on: Option<String>,
    pub latest_news_published_on: Option<String>,
    pub topics: Vec<TopTopic>,
    pub works_count: i64,
    pub works_copyright_count: i64,
    pub works_noncopyright_count: i64,
    /// Natsuzora fragment edited in shinonome admin (editable_contents table).
    /// `serde(default)` keeps older top.json files deserializable.
    #[serde(default)]
    pub editable_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopNewWork {
    pub work_id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub author_text: Option<String>,
    pub card_person_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopTopic {
    pub id: i64,
    pub title: String,
    pub published_on: Option<String>,
    pub year: Option<i32>,
}
