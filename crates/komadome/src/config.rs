use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub output: OutputConfig,
    pub templates: TemplatesConfig,
    pub data: DataConfig,
    pub build: BuildConfig,
    pub database: Option<DatabaseConfig>,
    pub assets: Option<AssetsConfig>,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub directory: PathBuf,
    pub main_site_url: String,
}

#[derive(Debug, Deserialize)]
pub struct TemplatesConfig {
    pub directory: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct DataConfig {
    pub directory: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_jobs")]
    pub default_jobs: usize,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct AssetsConfig {
    /// Directory containing precompiled CSS files (e.g., tailwind*.css)
    pub css_dir: Option<PathBuf>,
    /// Directory containing public images (top_logo.png, etc.)
    pub images_dir: Option<PathBuf>,
    /// Directory containing card images
    pub card_images_dir: Option<PathBuf>,
    /// Directory containing CSV ZIP files
    pub zip_dir: Option<PathBuf>,
}

fn default_jobs() -> usize {
    8
}

fn default_page_size() -> usize {
    50
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let mut config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.apply_env_overrides();

        Ok(config)
    }

    /// 環境変数で設定を上書きする。
    /// 本番(サーバ/Kamalアクセサリ)では DB 認証情報や公開URLを toml にコミットせず env で注入する。
    fn apply_env_overrides(&mut self) {
        if let Ok(url) = std::env::var("KOMADOME_DATABASE_URL") {
            if !url.is_empty() {
                self.database = Some(DatabaseConfig { url });
            }
        }
        if let Ok(site_url) = std::env::var("KOMADOME_MAIN_SITE_URL") {
            if !site_url.is_empty() {
                self.output.main_site_url = site_url;
            }
        }
    }
}
