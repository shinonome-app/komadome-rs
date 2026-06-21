use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::cli::CleanArgs;
use crate::config::Config;

pub fn run(config: &Config, args: CleanArgs) -> Result<()> {
    let mut targets: Vec<PathBuf> = vec![config.output.directory.clone()];
    if args.data {
        targets.push(config.data.directory.clone());
    }

    let existing: Vec<PathBuf> = targets.into_iter().filter(|d| d.exists()).collect();
    if existing.is_empty() {
        println!("Nothing to clean (target directories do not exist).");
        return Ok(());
    }

    if !args.force {
        let names = existing
            .iter()
            .map(|d| d.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        print!("Are you sure you want to delete the contents of {names}? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    for dir in &existing {
        println!("Cleaning {}...", dir.display());
        fs::remove_dir_all(dir)?;
        // do not remove (empty) directories
        fs::create_dir_all(dir)?;
    }
    println!("Done.");

    Ok(())
}
