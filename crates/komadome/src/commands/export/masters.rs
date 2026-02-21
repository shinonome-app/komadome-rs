use anyhow::Result;
use serde::Serialize;
use sqlx::PgPool;
use std::path::Path;

#[derive(Serialize)]
struct MastersData {
    exported_on: String,
    roles: Vec<RoleRow>,
    work_statuses: Vec<WorkStatusRow>,
    kana_types: Vec<KanaTypeRow>,
    filetypes: Vec<FiletypeRow>,
    compresstypes: Vec<CompresstypeRow>,
    booktypes: Vec<BooktypeRow>,
    charsets: Vec<CharsetRow>,
    file_encodings: Vec<FileEncodingRow>,
    worker_roles: Vec<WorkerRoleRow>,
}

#[derive(Serialize, sqlx::FromRow)]
struct RoleRow {
    id: i64,
    name: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct WorkStatusRow {
    id: i64,
    name: String,
    sort_order: i32,
}

#[derive(Serialize, sqlx::FromRow)]
struct KanaTypeRow {
    id: i64,
    name: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct FiletypeRow {
    id: i64,
    name: Option<String>,
    extension: Option<String>,
    is_html: bool,
    is_text: bool,
}

#[derive(Serialize, sqlx::FromRow)]
struct CompresstypeRow {
    id: i64,
    name: Option<String>,
    extension: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct BooktypeRow {
    id: i64,
    name: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct CharsetRow {
    id: i64,
    name: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct FileEncodingRow {
    id: i64,
    name: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct WorkerRoleRow {
    id: i64,
    name: Option<String>,
}

pub async fn export(pool: &PgPool, output_dir: &Path) -> Result<usize> {
    println!("Exporting masters.json...");

    let roles: Vec<RoleRow> = sqlx::query_as("SELECT id, name FROM roles ORDER BY id")
        .fetch_all(pool)
        .await?;

    let work_statuses: Vec<WorkStatusRow> =
        sqlx::query_as("SELECT id, name, sort_order FROM work_statuses ORDER BY id")
            .fetch_all(pool)
            .await?;

    let kana_types: Vec<KanaTypeRow> =
        sqlx::query_as("SELECT id, name FROM kana_types ORDER BY id")
            .fetch_all(pool)
            .await?;

    let filetypes: Vec<FiletypeRow> =
        sqlx::query_as("SELECT id, name, extension, is_html, is_text FROM filetypes ORDER BY id")
            .fetch_all(pool)
            .await?;

    let compresstypes: Vec<CompresstypeRow> =
        sqlx::query_as("SELECT id, name, extension FROM compresstypes ORDER BY id")
            .fetch_all(pool)
            .await?;

    let booktypes: Vec<BooktypeRow> =
        sqlx::query_as("SELECT id, name FROM booktypes ORDER BY id")
            .fetch_all(pool)
            .await?;

    let charsets: Vec<CharsetRow> = sqlx::query_as("SELECT id, name FROM charsets ORDER BY id")
        .fetch_all(pool)
        .await?;

    let file_encodings: Vec<FileEncodingRow> =
        sqlx::query_as("SELECT id, name FROM file_encodings ORDER BY id")
            .fetch_all(pool)
            .await?;

    let worker_roles: Vec<WorkerRoleRow> =
        sqlx::query_as("SELECT id, name FROM worker_roles ORDER BY id")
            .fetch_all(pool)
            .await?;

    let count = roles.len()
        + work_statuses.len()
        + kana_types.len()
        + filetypes.len()
        + compresstypes.len()
        + booktypes.len()
        + charsets.len()
        + file_encodings.len()
        + worker_roles.len();

    let masters = MastersData {
        exported_on: chrono::Local::now().date_naive().format("%Y-%m-%d").to_string(),
        roles,
        work_statuses,
        kana_types,
        filetypes,
        compresstypes,
        booktypes,
        charsets,
        file_encodings,
        worker_roles,
    };

    let json = serde_json::to_string_pretty(&masters)?;
    std::fs::write(output_dir.join("masters.json"), json)?;

    println!("  -> {} records", count);
    Ok(count)
}
