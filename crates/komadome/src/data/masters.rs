use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// All master tables loaded from masters.json
#[derive(Debug, Deserialize)]
pub struct Masters {
    /// The date when JSONL was exported (ISO 8601 format)
    #[serde(default)]
    pub exported_on: Option<String>,
    pub roles: Vec<Role>,
    pub work_statuses: Vec<WorkStatus>,
    pub kana_types: Vec<KanaType>,
    pub filetypes: Vec<Filetype>,
    pub compresstypes: Vec<Compresstype>,
    pub booktypes: Vec<Booktype>,
    pub charsets: Vec<Charset>,
    pub file_encodings: Vec<FileEncoding>,
    pub worker_roles: Vec<WorkerRole>,

    // Lookup maps (built after load)
    #[serde(skip)]
    roles_map: HashMap<i64, String>,
    #[serde(skip)]
    work_statuses_map: HashMap<i64, String>,
    #[serde(skip)]
    kana_types_map: HashMap<i64, String>,
    #[serde(skip)]
    filetypes_map: HashMap<i64, String>,
    #[serde(skip)]
    compresstypes_map: HashMap<i64, String>,
    #[serde(skip)]
    booktypes_map: HashMap<i64, String>,
    #[serde(skip)]
    worker_roles_map: HashMap<i64, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkStatus {
    pub id: i64,
    pub name: String,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KanaType {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filetype {
    pub id: i64,
    pub name: String,
    pub extension: Option<String>,
    pub is_html: Option<bool>,
    pub is_text: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Compresstype {
    pub id: i64,
    pub name: String,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Booktype {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Charset {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileEncoding {
    pub id: i64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerRole {
    pub id: i64,
    pub name: Option<String>,
}

impl Masters {
    /// Load masters from JSON file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read masters file: {}", path.display()))?;

        let mut masters: Masters = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse masters file: {}", path.display()))?;

        // exported_on が「存在するが不正フォーマット」の場合はビルド再現性が静かに
        // 壊れるため、ここでエラーにする (欠落は exported_date() で wall-clock に
        // フォールバックして良いが、不正値の黙殺は許さない)。
        if let Some(s) = masters.exported_on.as_deref() {
            NaiveDate::parse_from_str(s, "%Y-%m-%d").with_context(|| {
                format!("Invalid `exported_on` in masters file {}: {s:?}", path.display())
            })?;
        }

        masters.build_lookup_maps();

        Ok(masters)
    }

    fn build_lookup_maps(&mut self) {
        self.roles_map = self.roles.iter().map(|r| (r.id, r.name.clone())).collect();

        self.work_statuses_map = self
            .work_statuses
            .iter()
            .map(|s| (s.id, s.name.clone()))
            .collect();

        self.kana_types_map = self
            .kana_types
            .iter()
            .map(|k| (k.id, k.name.clone()))
            .collect();

        self.filetypes_map = self
            .filetypes
            .iter()
            .map(|f| (f.id, f.name.clone()))
            .collect();

        self.compresstypes_map = self
            .compresstypes
            .iter()
            .map(|c| (c.id, c.name.clone()))
            .collect();

        self.booktypes_map = self
            .booktypes
            .iter()
            .map(|b| (b.id, b.name.clone()))
            .collect();

        self.worker_roles_map = self
            .worker_roles
            .iter()
            .filter_map(|w| w.name.as_ref().map(|n| (w.id, n.clone())))
            .collect();
    }

    /// Get the export date as NaiveDate, falling back to today if not set
    pub fn exported_date(&self) -> NaiveDate {
        self.exported_on
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .unwrap_or_else(|| crate::clock::build_date())
    }

    pub fn role_name(&self, id: i64) -> Option<&str> {
        self.roles_map.get(&id).map(|s| s.as_str())
    }

    pub fn work_status_name(&self, id: i64) -> Option<&str> {
        self.work_statuses_map.get(&id).map(|s| s.as_str())
    }

    pub fn kana_type_name(&self, id: i64) -> Option<&str> {
        self.kana_types_map.get(&id).map(|s| s.as_str())
    }

    pub fn filetype_name(&self, id: i64) -> Option<&str> {
        self.filetypes_map.get(&id).map(|s| s.as_str())
    }

    pub fn compresstype_name(&self, id: i64) -> Option<&str> {
        self.compresstypes_map.get(&id).map(|s| s.as_str())
    }

    pub fn booktype_name(&self, id: i64) -> Option<&str> {
        self.booktypes_map.get(&id).map(|s| s.as_str())
    }

    pub fn worker_role_name(&self, id: i64) -> Option<&str> {
        self.worker_roles_map.get(&id).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masters_load_and_lookup() {
        let path = std::path::PathBuf::from(format!(
            "{}/tests/fixtures/masters_data.json",
            env!("CARGO_MANIFEST_DIR")
        ));
        let masters = Masters::load(&path).unwrap();

        assert_eq!(masters.exported_on.as_deref(), Some("2024-06-15"));
        assert_eq!(
            masters.exported_date(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()
        );
        assert_eq!(masters.roles.len(), 3);
        assert_eq!(masters.role_name(1), Some("著者"));
        assert_eq!(masters.role_name(2), Some("翻訳者"));
        assert_eq!(masters.role_name(99), None);
        assert_eq!(masters.work_status_name(1), Some("公開"));
        assert_eq!(masters.kana_type_name(1), Some("新字新仮名"));
        assert_eq!(masters.filetype_name(1), Some("テキストファイル(ルビあり)"));
        assert_eq!(masters.compresstype_name(1), Some("zip"));
        assert_eq!(masters.booktype_name(1), Some("底本"));
        assert_eq!(masters.worker_role_name(1), Some("入力"));
        assert_eq!(masters.worker_role_name(2), Some("校正"));
    }
}
