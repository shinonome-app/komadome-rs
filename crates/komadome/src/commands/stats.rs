use anyhow::Result;
use std::fs;

use crate::config::Config;
use crate::data::loader::count_jsonl_lines;

pub fn run(config: &Config) -> Result<()> {
    let data_dir = &config.data.directory;

    println!("=== komadome-rs Statistics ===\n");

    println!("Data files:");
    for file in &[
        "masters.json",
        "cards.jsonl",
        "person_pages.jsonl",
        "work_indexes.jsonl",
        "person_indexes.jsonl",
        "whatsnew.jsonl",
        "news.jsonl",
    ] {
        let path = data_dir.join(file);
        if path.exists() {
            let meta = fs::metadata(&path)?;
            let size_kb = meta.len() / 1024;

            if file.ends_with(".jsonl") {
                let lines = count_jsonl_lines(&path)?;
                println!("  {file}: {size_kb} KB ({lines} records)");
            } else {
                println!("  {file}: {size_kb} KB");
            }
        } else {
            println!("  {file}: (not found)");
        }
    }

    let output_dir = &config.output.directory;
    if output_dir.exists() {
        println!("\nOutput directory: {}", output_dir.display());
        // TODO: Count generated files
    } else {
        println!("\nOutput directory: (not built yet)");
    }

    Ok(())
}
