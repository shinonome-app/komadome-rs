use serde::{Deserialize, Serialize};

/// Author info embedded in card data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub id: i64,
    pub name: String,
    pub name_kana: String,
    pub copyright_flag: bool,
}

/// Translator/Editor info (no copyright_flag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonRef {
    pub id: i64,
    pub name: String,
    pub name_kana: String,
}

/// Workfile info embedded in card data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkfileInfo {
    pub id: i64,
    pub filename: Option<String>,
    pub filesize: Option<i32>,
    pub filetype: Option<String>,
    pub filetype_id: i64,
    #[serde(default)]
    pub is_html: bool,
    pub compresstype: Option<String>,
    pub charset: Option<String>,
    pub file_encoding: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub registered_on: Option<String>,
    pub last_updated_on: Option<String>,
}

/// Original book info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginalBookInfo {
    pub title: String,
    pub publisher: Option<String>,
    pub first_pubdate: Option<String>,
    pub input_edition: Option<String>,
    pub proof_edition: Option<String>,
    pub booktype: Option<String>,
    pub booktype_id: Option<i64>,
}

/// Work worker info (入力者・校正者)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkWorkerInfo {
    pub name: Option<String>,
    pub role: Option<String>,
}

/// Bibclass info (分類)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibclassInfo {
    pub name: String,
    pub num: String,
    pub note: Option<String>,
}

/// Site info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteInfo {
    pub name: Option<String>,
    pub url: Option<String>,
}

/// Work person detail info (for 作家データ section)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPersonDetail {
    pub role_name: String,
    pub person_id: i64,
    pub name: String,
    pub name_kana: String,
    pub name_en: Option<String>,
    pub born_on: Option<String>,
    pub died_on: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub copyright_flag: bool,
}

/// Card data (pre-joined for efficient page generation)
/// From cards.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardData {
    pub work_id: i64,
    pub person_id: i64,
    pub title: String,
    pub title_kana: Option<String>,
    pub subtitle: Option<String>,
    pub subtitle_kana: Option<String>,
    pub original_title: Option<String>,
    #[serde(default)]
    pub collection: Option<String>,
    #[serde(default)]
    pub collection_kana: Option<String>,
    pub kana_type: Option<String>,
    pub started_on: Option<String>,
    pub note: Option<String>,
    pub first_appearance: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<AuthorInfo>,
    pub translators: Vec<PersonRef>,
    pub editors: Vec<PersonRef>,
    pub workfiles: Vec<WorkfileInfo>,
    pub original_books: Vec<OriginalBookInfo>,
    pub work_workers: Vec<WorkWorkerInfo>,
    pub bibclasses: Vec<BibclassInfo>,
    pub sites: Vec<SiteInfo>,
    #[serde(default)]
    pub work_people_details: Vec<WorkPersonDetail>,
}

impl CardData {
    /// Check if any related person (of any role) has copyright.
    /// `work_people_details` holds every author/translator/editor for the work.
    pub fn has_copyright(&self) -> bool {
        self.work_people_details.iter().any(|wp| wp.copyright_flag)
    }

    /// Get primary author (first in list)
    pub fn primary_author(&self) -> Option<&AuthorInfo> {
        self.authors.first()
    }
}
