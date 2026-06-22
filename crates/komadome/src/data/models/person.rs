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
    // first_name が無い場合でも末尾に空白を残す
    pub fn full_name(&self) -> String {
        format!(
            "{} {}",
            self.last_name,
            self.first_name.as_deref().unwrap_or("")
        )
    }

    pub fn full_name_kana(&self) -> String {
        format!(
            "{} {}",
            self.last_name_kana,
            self.first_name_kana.as_deref().unwrap_or("")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(last: &str, first: Option<&str>) -> Person {
        Person {
            id: 1,
            last_name: last.to_string(),
            first_name: first.map(str::to_string),
            last_name_kana: last.to_string(),
            first_name_kana: first.map(str::to_string),
            name_en: None,
            born_on: None,
            died_on: None,
            copyright_flag: false,
            description: None,
            sortkey: None,
        }
    }

    #[test]
    fn full_name_keeps_trailing_space_without_first_name() {
        // 姓のみのときも末尾に空白を残す。
        assert_eq!(person("紫式部", None).full_name(), "紫式部 ");
        assert_eq!(person("太宰", Some("治")).full_name(), "太宰 治");
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
    pub card_person_id: Option<i64>,
    #[serde(default)]
    pub work_people: Vec<WorkPersonRef>,
}
