//! `generate-zip` サブコマンド: shinonome の `CsvCreator` 相当を Rust に移植したもの。
//!
//! 公開作品 (基本/拡充)・未公開作品 (基本) の 3 系統 × SJIS/UTF-8 の合計 6 個の zip
//! を `assets.zip_dir` (デフォルト `data/csv_zip/`) に生成する。
//! ファイル名規約は本番 aozora.gr.jp 準拠:
//!   - list_person_all.zip / list_person_all_utf8.zip
//!   - list_person_all_extended.zip / list_person_all_extended_utf8.zip
//!   - list_inp_person_all.zip / list_inp_person_all_utf8.zip

mod basic;
mod finished_extended;
mod headers;

use anyhow::{Context, Result};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::commands::export::db;
use crate::config::Config;

/// UTF-8 BOM を CSV 先頭に付加する際に使うバイト列。
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// `generate-zip` のエントリーポイント。
pub async fn run(config: &Config) -> Result<()> {
    let start = Instant::now();

    let zip_dir = resolve_zip_dir(config)?;
    fs::create_dir_all(&zip_dir)
        .with_context(|| format!("Failed to create zip dir: {}", zip_dir.display()))?;

    println!("Generating CSV zip files in {}...\n", zip_dir.display());

    let pool = db::connect(config).await?;
    let today = crate::clock::build_date();

    basic::generate(&pool, &zip_dir, today, basic::BasicKind::Finished).await?;
    finished_extended::generate(&pool, &zip_dir, today, &config.output.main_site_url).await?;
    basic::generate(&pool, &zip_dir, today, basic::BasicKind::Inp).await?;

    pool.close().await;

    let elapsed = start.elapsed();
    println!("\nGenerated 6 zip files in {:.2}s", elapsed.as_secs_f64());
    Ok(())
}

/// `[assets] zip_dir` 設定を取得する。設定がなければエラー。
fn resolve_zip_dir(config: &Config) -> Result<PathBuf> {
    config
        .assets
        .as_ref()
        .and_then(|a| a.zip_dir.clone())
        .context("`[assets] zip_dir` must be configured to generate zip files")
}

/// 生成した CSV を SJIS と UTF-8 の 2 種類で zip 化して書き出す。
///
/// `base_name` は zip ファイル名のベース (例: `list_person_all`)。
/// 拡張子なしで渡す。
/// `csv_body_utf8` は BOM 付き UTF-8 の CSV バイト列。
pub(super) fn write_pair(zip_dir: &Path, base_name: &str, csv_body_utf8: &[u8]) -> Result<()> {
    // UTF-8 zip
    let utf8_zip_path = zip_dir.join(format!("{base_name}_utf8.zip"));
    let utf8_csv_name = format!("{base_name}_utf8.csv");
    write_zip(&utf8_zip_path, &utf8_csv_name, csv_body_utf8)
        .with_context(|| format!("Failed to write {}", utf8_zip_path.display()))?;

    // SJIS zip: UTF-8 (BOM 付) を CP932 にエンコードし直す。BOM は SJIS では除去。
    let csv_body_sjis = encode_sjis(csv_body_utf8)?;
    let sjis_zip_path = zip_dir.join(format!("{base_name}.zip"));
    let sjis_csv_name = format!("{base_name}.csv");
    write_zip(&sjis_zip_path, &sjis_csv_name, &csv_body_sjis)
        .with_context(|| format!("Failed to write {}", sjis_zip_path.display()))?;

    println!(
        "  -> {} ({} bytes), {} ({} bytes)",
        utf8_zip_path.file_name().unwrap().to_string_lossy(),
        csv_body_utf8.len(),
        sjis_zip_path.file_name().unwrap().to_string_lossy(),
        csv_body_sjis.len(),
    );
    Ok(())
}

/// UTF-8 の CSV (BOM 付) を CP932 (Shift_JIS) に変換する。BOM は除去する。
fn encode_sjis(utf8_with_bom: &[u8]) -> Result<Vec<u8>> {
    let stripped: &[u8] = utf8_with_bom
        .strip_prefix(UTF8_BOM)
        .unwrap_or(utf8_with_bom);

    let utf8_str = std::str::from_utf8(stripped)
        .context("CSV body is not valid UTF-8 (this is a programming error)")?;

    let (cow, _, had_unmappable) = encoding_rs::SHIFT_JIS.encode(utf8_str);
    if had_unmappable {
        // CP932 で表現できない文字があった場合、`encoding_rs` は HTML 数値文字参照
        // (`&#NNNN;`) に置換する。Ruby 側の `String#encode('cp932')` のデフォルト
        // 動作に合わせるとここで例外を投げるべきだが、本番運用上は警告に留める。
        eprintln!(
            "  warning: some characters were not representable in Shift_JIS \
             and replaced with HTML numeric character references"
        );
    }
    Ok(cow.into_owned())
}

/// 単一 CSV を含む zip を作成する。
fn write_zip(zip_path: &Path, csv_filename: &str, csv_body: &[u8]) -> Result<()> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buf);
        let options: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer.start_file(csv_filename, options)?;
        writer.write_all(csv_body)?;
        writer.finish()?;
    }
    fs::write(zip_path, buf.into_inner())?;
    Ok(())
}

/// CSV writer ファクトリ (force_quotes、CRLF terminator)。
pub(super) fn make_csv_writer(buf: &mut Vec<u8>) -> csv::Writer<&mut Vec<u8>> {
    csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .terminator(csv::Terminator::CRLF)
        .from_writer(buf)
}

/// CSV body を組み立てる際の共通プリアンブル: BOM + ヘッダー行 (CRLF 区切り)。
///
/// ヘッダーは Ruby の `csv_header` 文字列 (`...\r\n` 終端) と完全一致させたいので、
/// `csv` クレートを使わず手動で BOM とヘッダー文字列をそのまま書き出す。
pub(super) fn write_header(buf: &mut Vec<u8>, header_line: &str) -> Result<()> {
    buf.extend_from_slice(UTF8_BOM);
    buf.extend_from_slice(header_line.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_header_produces_bom_and_header() {
        let mut buf = Vec::new();
        write_header(&mut buf, "a,b,c\r\n").unwrap();
        assert_eq!(&buf[..3], b"\xEF\xBB\xBF");
        assert_eq!(&buf[3..], b"a,b,c\r\n");
    }

    #[test]
    fn encode_sjis_strips_bom() {
        let mut buf = Vec::new();
        write_header(&mut buf, "abc\r\n").unwrap();
        let sjis = encode_sjis(&buf).unwrap();
        assert_eq!(sjis, b"abc\r\n");
    }

    #[test]
    fn encode_sjis_handles_japanese() {
        let mut buf = Vec::new();
        write_header(&mut buf, "テスト\r\n").unwrap();
        let sjis = encode_sjis(&buf).unwrap();
        // Shift_JIS では「テ」=0x83 0x65, 「ス」=0x83 0x58, 「ト」=0x83 0x67
        assert_eq!(sjis, b"\x83\x65\x83\x58\x83\x67\r\n");
    }

    #[test]
    fn make_csv_writer_quotes_and_uses_crlf() {
        let mut buf = Vec::new();
        {
            let mut w = make_csv_writer(&mut buf);
            w.write_record(["1", "abc", ""]).unwrap();
            w.flush().unwrap();
        }
        assert_eq!(&buf[..], b"\"1\",\"abc\",\"\"\r\n");
    }

    #[test]
    fn write_zip_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        write_zip(&zip_path, "test.csv", b"a,b,c\r\n").unwrap();

        let bytes = std::fs::read(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 1);
        let mut entry = archive.by_index(0).unwrap();
        assert_eq!(entry.name(), "test.csv");
        let mut content = Vec::new();
        std::io::copy(&mut entry, &mut content).unwrap();
        assert_eq!(content, b"a,b,c\r\n");
    }

    #[test]
    fn write_pair_creates_both_zips() {
        let dir = tempfile::tempdir().unwrap();
        let mut buf = Vec::new();
        write_header(&mut buf, "id,名前\r\n").unwrap();
        {
            let mut w = make_csv_writer(&mut buf);
            w.write_record(["1", "テスト"]).unwrap();
            w.flush().unwrap();
        }
        write_pair(dir.path(), "demo", &buf).unwrap();

        assert!(dir.path().join("demo.zip").is_file());
        assert!(dir.path().join("demo_utf8.zip").is_file());

        // UTF-8 zip should retain BOM
        let utf8_bytes = std::fs::read(dir.path().join("demo_utf8.zip")).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(utf8_bytes)).unwrap();
        let mut entry = archive.by_index(0).unwrap();
        let mut content = Vec::new();
        std::io::copy(&mut entry, &mut content).unwrap();
        assert!(content.starts_with(UTF8_BOM));
    }
}
