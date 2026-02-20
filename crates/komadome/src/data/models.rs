use chrono::NaiveDate;
use serde::Deserialize;

/// Work (作品) - basic model, not used for JSONL import
#[derive(Debug, Clone, Deserialize)]
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

/// Person (人物) - embedded in PersonPageData
#[derive(Debug, Clone, Deserialize)]
pub struct Person {
    pub id: i64,
    pub last_name: String,
    pub first_name: Option<String>,
    pub last_name_kana: String,
    pub first_name_kana: Option<String>,
    pub born_on: Option<String>,
    pub died_on: Option<String>,
    pub copyright_flag: bool,
    pub description: Option<String>,
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

/// Author info embedded in card data
#[derive(Debug, Clone, Deserialize)]
pub struct AuthorInfo {
    pub id: i64,
    pub name: String,
    pub name_kana: String,
    pub copyright_flag: bool,
}

/// Translator/Editor info (no copyright_flag)
#[derive(Debug, Clone, Deserialize)]
pub struct PersonRef {
    pub id: i64,
    pub name: String,
    pub name_kana: String,
}

/// Workfile info embedded in card data
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
pub struct WorkWorkerInfo {
    pub name: Option<String>,
    pub role: Option<String>,
}

/// Bibclass info (分類)
#[derive(Debug, Clone, Deserialize)]
pub struct BibclassInfo {
    pub name: String,
    pub num: String,
    pub note: Option<String>,
}

/// Site info
#[derive(Debug, Clone, Deserialize)]
pub struct SiteInfo {
    pub name: Option<String>,
    pub url: Option<String>,
}

/// Work person detail info (for 作家データ section)
#[derive(Debug, Clone, Deserialize)]
pub struct WorkPersonDetail {
    pub role_name: String,
    pub person_id: i64,
    pub name: String,
    pub name_kana: String,
    pub name_en: Option<String>,
    pub born_on: Option<String>,
    pub died_on: Option<String>,
    pub description: Option<String>,
}

/// Card data (pre-joined for efficient page generation)
/// From cards.jsonl
#[derive(Debug, Clone, Deserialize)]
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
    /// Check if any author has copyright
    pub fn has_copyright(&self) -> bool {
        self.authors.iter().any(|a| a.copyright_flag)
    }

    /// Get primary author (first in list)
    pub fn primary_author(&self) -> Option<&AuthorInfo> {
        self.authors.first()
    }

    /// Get card path (e.g., "cards/000001/card12345.html")
    pub fn card_path(&self) -> String {
        let person_dir = format!("{:06}", self.person_id);
        format!("cards/{}/card{}.html", person_dir, self.work_id)
    }
}

/// Person page data (pre-joined)
/// From person_pages.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct PersonPageData {
    pub person: Person,
    pub works: Vec<PersonWorkInfo>,
    pub sites: Vec<SiteInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonWorkInfo {
    pub id: i64,
    pub title: String,
    pub title_kana: Option<String>,
    pub subtitle: Option<String>,
    pub role: Option<String>,
    pub role_id: i64,
    pub kana_type: Option<String>,
}

/// Work index data (for index pages)
/// From work_indexes.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct WorkIndexData {
    pub kana_symbol: String,
    pub page: usize,
    pub total_pages: usize,
    pub works: Vec<WorkIndexItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkIndexItem {
    pub id: i64,
    pub title: String,
    pub title_kana: Option<String>,
    pub subtitle: Option<String>,
    pub author_name: Option<String>,
    pub person_id: Option<i64>,
}

/// Person index data
/// From person_indexes.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct PersonIndexData {
    pub kana_column: String,
    pub people: Vec<PersonIndexItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonIndexItem {
    pub id: i64,
    pub name: String,
    pub name_kana: String,
    pub work_count: usize,
    pub copyright_flag: bool,
}

/// Whatsnew data
/// From whatsnew.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsnewData {
    pub year: Option<i32>,
    pub page: usize,
    pub total_pages: usize,
    pub entries: Vec<WhatsnewEntry>,
}

#[derive(Debug, Clone, Deserialize)]
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

/// Top page data
/// From top.json
#[derive(Debug, Clone, Deserialize)]
pub struct TopPageData {
    pub new_works: Vec<TopNewWork>,
    pub new_works_published_on: Option<String>,
    pub latest_news_published_on: Option<String>,
    pub topics: Vec<TopTopic>,
    pub works_count: i64,
    pub works_copyright_count: i64,
    pub works_noncopyright_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopNewWork {
    pub work_id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub author_text: Option<String>,
    pub card_person_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopTopic {
    pub id: i64,
    pub title: String,
    pub published_on: Option<String>,
    pub year: Option<i32>,
}

/// WIP Work index data (for WIP index pages)
/// From wip_work_indexes.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct WipWorkIndexData {
    pub kana_symbol: String,
    pub page: usize,
    pub total_pages: usize,
    pub works: Vec<WipWorkIndexItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WipWorkIndexItem {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub kana_type_name: Option<String>,
    pub author_name: Option<String>,
    pub author_id: Option<i64>,
    pub base_author_name: Option<String>,
    pub translator_text: Option<String>,
    pub inputer_text: Option<String>,
    pub proofreader_text: Option<String>,
    pub work_status_name: Option<String>,
    pub started_on: Option<String>,
    pub teihon_title: Option<String>,
    pub teihon_publisher: Option<String>,
    pub teihon_input_edition: Option<String>,
}

/// WIP Person index data
/// From wip_person_indexes.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct WipPersonIndexData {
    pub kana_column: String,
    pub column_display: String,
    pub sections: Vec<WipPersonSection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WipPersonSection {
    pub kana_char: String,
    pub section_index: usize,
    pub people: Vec<WipPersonItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WipPersonItem {
    pub id: i64,
    pub name: String,
    pub unpublished_count: i64,
    pub copyright_flag: bool,
}

/// Person All index data (all registered persons)
/// From person_all_indexes.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct PersonAllIndexData {
    pub kana_column: String,
    pub column_display: String,
    pub sections: Vec<PersonAllSection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonAllSection {
    pub kana_char: String,
    pub section_index: usize,
    pub people: Vec<PersonAllItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonAllItem {
    pub id: i64,
    pub name: String,
    pub published_count: i64,
    pub unpublished_count: i64,
    pub copyright_flag: bool,
}

/// List Inp data (per-person WIP work lists)
/// From list_inp.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct ListInpData {
    pub person_id: i64,
    pub person_name: String,
    pub page: usize,
    pub total_pages: usize,
    pub works: Vec<ListInpWorkItem>,
}

#[derive(Debug, Clone, Deserialize)]
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

/// News data
/// From news.jsonl
#[derive(Debug, Clone, Deserialize)]
pub struct NewsData {
    pub year: i32,
    pub entries: Vec<NewsEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewsEntry {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub published_on: Option<String>,
    pub flag: bool,
}
