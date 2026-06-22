//! `validate-data` サブコマンド。
//!
//! export が吐く JSONL/JSON を、対応 DTO への構造検査＋セマンティック検査（空件数・
//! ユニークキー重複・カウント負値・ページネーション整合）にかける。`--report` 指定時は
//! 人間が目視確認できる HTML レポートも書き出す。

mod html;
mod report;

use anyhow::Result;
use std::fs;

use crate::cli::ValidateDataArgs;
use crate::config::Config;
use crate::data::masters::Masters;
use crate::data::models::{
    CardData, ListInpData, NewsData, PersonAllIndexData, PersonIndexData, PersonPageData,
    TopPageData, WhatsnewData, WipPersonIndexData, WipWorkIndexData, WorkIndexData,
};
use report::{DataFileSpec, FileReport, Kind, Severity, check_file, validate_as};

/// 検証対象ファイルとその DTO/ユニークキーの定義一覧 (export 全種別を網羅)。
fn specs() -> Vec<DataFileSpec> {
    vec![
        DataFileSpec {
            name: "masters.json",
            label: "マスタ",
            kind: Kind::SingleJson,
            unique_key: &[],
            validate: validate_as::<Masters>,
        },
        DataFileSpec {
            name: "top.json",
            label: "トップページ",
            kind: Kind::SingleJson,
            unique_key: &[],
            validate: validate_as::<TopPageData>,
        },
        DataFileSpec {
            name: "cards.jsonl",
            label: "図書カード",
            kind: Kind::Jsonl,
            unique_key: &["work_id", "person_id"],
            validate: validate_as::<CardData>,
        },
        DataFileSpec {
            name: "person_pages.jsonl",
            label: "人物ページ",
            kind: Kind::Jsonl,
            unique_key: &["person.id"],
            validate: validate_as::<PersonPageData>,
        },
        DataFileSpec {
            name: "work_indexes.jsonl",
            label: "公開作品索引",
            kind: Kind::Jsonl,
            unique_key: &["kana_symbol", "page"],
            validate: validate_as::<WorkIndexData>,
        },
        DataFileSpec {
            name: "person_indexes.jsonl",
            label: "公開人物索引",
            kind: Kind::Jsonl,
            unique_key: &["kana_column"],
            validate: validate_as::<PersonIndexData>,
        },
        DataFileSpec {
            name: "person_all_indexes.jsonl",
            label: "登録全作家索引",
            kind: Kind::Jsonl,
            unique_key: &["kana_column"],
            validate: validate_as::<PersonAllIndexData>,
        },
        DataFileSpec {
            name: "wip_work_indexes.jsonl",
            label: "作業中作品索引",
            kind: Kind::Jsonl,
            unique_key: &["kana_symbol", "page"],
            validate: validate_as::<WipWorkIndexData>,
        },
        DataFileSpec {
            name: "wip_person_indexes.jsonl",
            label: "作業中人物索引",
            kind: Kind::Jsonl,
            unique_key: &["kana_column"],
            validate: validate_as::<WipPersonIndexData>,
        },
        DataFileSpec {
            name: "list_inp.jsonl",
            label: "作業中作家別一覧",
            kind: Kind::Jsonl,
            unique_key: &["person_id", "page"],
            validate: validate_as::<ListInpData>,
        },
        DataFileSpec {
            name: "whatsnew.jsonl",
            label: "新着公開",
            kind: Kind::Jsonl,
            unique_key: &["year", "page"],
            validate: validate_as::<WhatsnewData>,
        },
        DataFileSpec {
            name: "news.jsonl",
            label: "そらもよう",
            kind: Kind::Jsonl,
            unique_key: &["year"],
            validate: validate_as::<NewsData>,
        },
    ]
}

pub fn run(config: &Config, args: ValidateDataArgs) -> Result<()> {
    let dir = &config.data.directory;
    println!("Validating data files in {}...\n", dir.display());

    let specs = specs();
    let mut reports: Vec<FileReport> = Vec::new();
    let mut total_errors = 0usize;
    let mut total_warns = 0usize;

    for spec in &specs {
        let report = check_file(dir, spec, args.sample);
        print_report_line(&report);
        total_errors += report.errors();
        total_warns += report.warns();
        reports.push(report);
    }

    println!(
        "\n{} file(s): {} error(s), {} warning(s).",
        specs.len(),
        total_errors,
        total_warns,
    );

    if let Some(path) = &args.report {
        let html = html::render(&reports, &crate::clock::build_date().to_string());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, html)?;
        println!("HTML report written to {}", path.display());
    }

    if total_errors > 0 {
        anyhow::bail!("{total_errors} error(s) found in data files");
    }

    Ok(())
}

fn print_report_line(r: &FileReport) {
    let badge = if !r.present {
        "[MISSING]"
    } else if r.errors() > 0 {
        "[ERROR]"
    } else if r.warns() > 0 {
        "[WARN] "
    } else {
        "[OK]   "
    };

    let count = if r.present {
        format!("{} records", r.record_count)
    } else {
        String::new()
    };
    println!("  {badge} {:<26} {count}", r.name);

    for f in &r.findings {
        let sev = match f.severity {
            Severity::Error => "ERROR",
            Severity::Warn => "WARN",
        };
        let line = f.line.map(|l| format!("L{l} ")).unwrap_or_default();
        println!("      {line}{sev}: {}", f.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_for(name: &str) -> DataFileSpec {
        specs().into_iter().find(|s| s.name == name).unwrap()
    }

    /// fixture の CardData を1行 JSONL 化して返す。
    fn card_line() -> String {
        let raw = std::fs::read_to_string(format!(
            "{}/tests/fixtures/card_data.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        serde_json::to_string(&v).unwrap()
    }

    #[test]
    fn flags_structural_and_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let line = card_line();
        // 同一行2本 (= (work_id, person_id) 重複) + 壊れたレコード。
        std::fs::write(
            dir.path().join("cards.jsonl"),
            format!("{line}\n{line}\n{{\"broken\":true}}\n"),
        )
        .unwrap();

        let report = check_file(dir.path(), &spec_for("cards.jsonl"), 5);

        assert_eq!(report.record_count, 3);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("DTO 不一致"))
        );
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("ユニークキー重複"))
        );
        assert!(!report.fields.is_empty());
        assert!(!report.samples.is_empty());
    }

    #[test]
    fn empty_file_warns_and_missing_file_warns() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("news.jsonl"), "").unwrap();

        let empty = check_file(dir.path(), &spec_for("news.jsonl"), 5);
        assert_eq!(empty.record_count, 0);
        assert!(empty.warns() >= 1);
        assert_eq!(empty.errors(), 0);

        // 存在しないファイル → MISSING 警告。
        let missing = check_file(dir.path(), &spec_for("cards.jsonl"), 5);
        assert!(!missing.present);
        assert!(missing.warns() >= 1);
    }

    #[test]
    fn html_report_contains_badges_and_names() {
        let dir = tempfile::tempdir().unwrap();
        let report = check_file(dir.path(), &spec_for("news.jsonl"), 5);
        let html = html::render(&[report], "2026-06-22");
        assert!(html.contains("データ検査レポート"));
        assert!(html.contains("news.jsonl"));
        assert!(html.contains("MISSING"));
        assert!(html.contains("2026-06-22"));
    }
}
