use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "komadome")]
#[command(about = "Static site generator for Aozora Bunko", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "config/komadome.toml")]
    pub config: PathBuf,

    /// Pin the build date (YYYY-MM-DD) for deterministic output.
    /// Overrides the KOMADOME_BUILD_DATE env var; defaults to the system date.
    #[arg(long, global = true, value_name = "YYYY-MM-DD")]
    pub date: Option<NaiveDate>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build all pages
    Build(BuildArgs),

    /// Build card pages only
    Cards(CardsArgs),

    /// Build person pages only
    People(PeopleArgs),

    /// Build index pages only
    Indexes(IndexesArgs),

    /// Build whatsnew pages only
    Whatsnew(WhatsnewArgs),

    /// Build soramoyou (news) pages only
    Soramoyou(SoramoyouArgs),

    /// Clean output directory
    Clean(CleanArgs),

    /// Export data from PostgreSQL to JSONL files
    Export(ExportArgs),

    /// Generate downloadable CSV zip files (basic, extended, unpublished)
    GenerateZip,

    /// Show statistics
    Stats,

    /// Validate templates against contracts
    Validate,
}

#[derive(Parser)]
pub struct BuildArgs {
    /// Number of parallel jobs
    #[arg(short, long)]
    pub jobs: Option<usize>,
}

#[derive(Parser)]
pub struct CardsArgs {
    /// Number of parallel jobs
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Build only specific work ID
    #[arg(long)]
    pub work_id: Option<i64>,
}

#[derive(Parser)]
pub struct PeopleArgs {
    /// Number of parallel jobs
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Build only specific person ID
    #[arg(long)]
    pub person_id: Option<i64>,
}

#[derive(Parser)]
pub struct IndexesArgs {
    /// Number of parallel jobs
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Index type to build
    #[arg(long, value_parser = ["works", "people", "all"])]
    pub r#type: Option<String>,
}

#[derive(Parser)]
pub struct WhatsnewArgs {
    /// Number of parallel jobs
    #[arg(short, long)]
    pub jobs: Option<usize>,
}

#[derive(Parser)]
pub struct SoramoyouArgs {
    /// Number of parallel jobs
    #[arg(short, long)]
    pub jobs: Option<usize>,
}

#[derive(Parser)]
pub struct CleanArgs {
    /// Force clean without confirmation
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Parser)]
pub struct ExportArgs {
    /// Export specific type only
    #[arg(long, value_parser = ["masters", "cards", "person_pages", "work_indexes", "person_indexes", "whatsnew", "news", "top", "wip_work_indexes", "wip_person_indexes", "person_all_indexes", "list_inp"])]
    pub only: Option<String>,
}
