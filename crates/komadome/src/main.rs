mod cli;
mod commands;
mod config;
mod data;
mod generator;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};
use commands::{build, clean, export, stats};
use config::Config;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::load(&cli.config)?;

    match cli.command {
        Commands::Build(args) => build::run(&config, args)?,
        Commands::Cards(args) => build::run_cards(&config, args)?,
        Commands::People(args) => build::run_people(&config, args)?,
        Commands::Indexes(args) => build::run_indexes(&config, args)?,
        Commands::Whatsnew(args) => build::run_whatsnew(&config, args)?,
        Commands::Soramoyou(args) => build::run_soramoyou(&config, args)?,
        Commands::Clean(args) => clean::run(&config, args)?,
        Commands::Export(args) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(export::run(&config, args))?;
        }
        Commands::Stats => stats::run(&config)?,
    }

    Ok(())
}
