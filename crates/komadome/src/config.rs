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

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }
}
