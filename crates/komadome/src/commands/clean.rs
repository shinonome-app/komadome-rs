use anyhow::Result;
use std::fs;
use std::io::{self, Write};

use crate::cli::CleanArgs;
use crate::config::Config;

pub fn run(config: &Config, args: CleanArgs) -> Result<()> {
    let output_dir = &config.output.directory;

    if !output_dir.exists() {
        println!("Output directory does not exist: {}", output_dir.display());
        return Ok(());
    }

    if !args.force {
        print!(
            "Are you sure you want to delete {}? [y/N] ",
            output_dir.display()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("Cleaning {}...", output_dir.display());
    fs::remove_dir_all(output_dir)?;
    println!("Done.");

    Ok(())
}
