use serde::{Deserialize, Serialize};

use super::card::SiteInfo;

/// Person (人物) - embedded in PersonPageData
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: i64,
    pub last_name: String,
    pub first_name: Option<String>,
    pub last_name_kana: String,
    pub first_name_kana: Option<String>,
    #[serde(default)]
    pub name_en: Option<String>,
    pub born_on: Option<String>,
    pub died_on: Option<String>,
    pub copyright_flag: bool,
    pub description: Option<String>,
    #[serde(default)]
    pub sortkey: Option<String>,
}

impl Person {
    pub fn full_name(&self) -> String {
        match &self.first_name {
            Some(first) => format!("{} {}", self.last_name, first),
            None => self.last_name.clone(),
        }
    }

    pub fn full_name_kana(&self) -> String {
        match &self.first_name_kana {
            Some(first) => format!("{} {}", self.last_name_kana, first),
            None => self.last_name_kana.clone(),
        }
    }
}

/// Other base person (別名人物)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherBasePerson {
    pub id: i64,
    pub name: String,
}

/// Work person reference (関連著者/翻訳者)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPersonRef {
    pub person_id: i64,
    pub name: String,
    pub role_name: Option<String>,
}

/// Person page data (pre-joined)
/// From person_pages.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonPageData {
    pub person: Person,
    pub works: Vec<PersonWorkInfo>,
    #[serde(default)]
    pub unpublished_works: Vec<PersonWorkInfo>,
    pub sites: Vec<SiteInfo>,
    #[serde(default)]
    pub other_base_people: Vec<OtherBasePerson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonWorkInfo {
    pub id: i64,
    pub title: String,
    pub title_kana: Option<String>,
    pub subtitle: Option<String>,
    #[serde(default)]
    pub sortkey: Option<String>,
    #[serde(default)]
    pub subtitle_kana: Option<String>,
    pub role: Option<String>,
    pub role_id: i64,
    pub kana_type: Option<String>,
    #[serde(default)]
    pub card_person_id: Option<String>,
    #[serde(default)]
    pub work_people: Vec<WorkPersonRef>,
}
