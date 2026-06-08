use anyhow::{Context, Result};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::config::Config;

/// Create a database connection pool from config
pub async fn connect(config: &Config) -> Result<PgPool> {
    let db_config = config.database.as_ref().context(
        "Database configuration is required for export. Add [database] section to config.",
    )?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_config.url)
        .await
        .with_context(|| format!("Failed to connect to database: {}", db_config.url))?;

    Ok(pool)
}
