//! データファイル1個ぶんの検証結果モデルと、構造／セマンティック検査の実装。

use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::Path;

/// 検出された問題の深刻度。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// 構造不一致など、build を壊しうる致命的な問題。
    Error,
    /// 0件・参照欠落など、目視確認を促す警告。
    Warn,
}

/// 1件の検出事項。`line` は JSONL の行番号 (1始まり)。
pub struct Finding {
    pub severity: Severity,
    pub line: Option<usize>,
    pub message: String,
}

/// データファイルの種別。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// 1行1レコードの JSONL。
    Jsonl,
    /// ファイル全体が単一の JSON オブジェクト (masters.json / top.json)。
    SingleJson,
}

/// 1ファイルの検証仕様。`validate` は1レコード(Value)を対応DTOへ
/// デシリアライズできるか試す関数。
pub struct DataFileSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub kind: Kind,
    /// 重複検出に使うユニークキー (ドット区切りで入れ子参照可)。空なら重複検査なし。
    pub unique_key: &'static [&'static str],
    pub validate: fn(&Value) -> Result<(), String>,
}

/// `Value` を型 `T` へデシリアライズできるか検証する (構造検査の実体)。
pub fn validate_as<T: DeserializeOwned>(v: &Value) -> Result<(), String> {
    serde_json::from_value::<T>(v.clone())
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// トップレベル1フィールドの分布統計。
pub struct FieldStat {
    pub name: String,
    pub ty: &'static str,
    pub present: usize,
    pub null: usize,
    pub bool_true: usize,
    pub bool_false: usize,
}

/// 1ファイルの検証レポート。
pub struct FileReport {
    pub name: String,
    pub label: String,
    pub present: bool,
    pub record_count: usize,
    pub findings: Vec<Finding>,
    pub samples: Vec<Value>,
    pub fields: Vec<FieldStat>,
}

impl FileReport {
    pub fn errors(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count()
    }

    pub fn warns(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warn)
            .count()
    }
}

/// 1ファイルを検証してレポートを返す。
pub fn check_file(dir: &Path, spec: &DataFileSpec, sample_n: usize) -> FileReport {
    let path = dir.join(spec.name);
    let mut report = FileReport {
        name: spec.name.to_string(),
        label: spec.label.to_string(),
        present: path.exists(),
        record_count: 0,
        findings: Vec::new(),
        samples: Vec::new(),
        fields: Vec::new(),
    };

    if !path.exists() {
        report.findings.push(Finding {
            severity: Severity::Warn,
            line: None,
            message: "ファイルが存在しません (未エクスポート?)".to_string(),
        });
        return report;
    }

    // 行 → Value (JSON 自体の破損はここで Error として記録)。
    let values: Vec<(usize, Value)> = match spec.kind {
        Kind::Jsonl => read_jsonl_values(&path, &mut report),
        Kind::SingleJson => read_single_value(&path, &mut report),
    };
    report.record_count = values.len();

    // 構造検査: 各レコードを DTO へデシリアライズ。
    for (line, v) in &values {
        if let Err(msg) = (spec.validate)(v) {
            push_capped(
                &mut report.findings,
                Severity::Error,
                Some(*line),
                format!("DTO 不一致: {msg}"),
                "DTO 不一致",
            );
        }
    }

    // masters.json は日付検証など独自ロジックがあるので Masters::load も通す。
    if spec.name == "masters.json" {
        if let Err(e) = crate::data::masters::Masters::load(&path) {
            report.findings.push(Finding {
                severity: Severity::Error,
                line: None,
                message: format!("Masters::load 失敗: {e}"),
            });
        }
    }

    semantic_checks(spec, &values, &mut report);

    report.samples = values
        .iter()
        .take(sample_n)
        .map(|(_, v)| v.clone())
        .collect();
    report.fields = field_stats(values.iter().map(|(_, v)| v));

    report
}

/// JSONL を1行ずつ Value へ。空行スキップ。壊れた行は Error 記録のうえ除外。
fn read_jsonl_values(path: &Path, report: &mut FileReport) -> Vec<(usize, Value)> {
    use std::io::BufRead;

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            report.findings.push(Finding {
                severity: Severity::Error,
                line: None,
                message: format!("オープン失敗: {e}"),
            });
            return Vec::new();
        }
    };

    let mut out = Vec::new();
    for (idx, line) in std::io::BufReader::new(file).lines().enumerate() {
        let line_num = idx + 1;
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    line: Some(line_num),
                    message: format!("行読み取り失敗: {e}"),
                });
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(&line) {
            Ok(v) => out.push((line_num, v)),
            Err(e) => push_capped(
                &mut report.findings,
                Severity::Error,
                Some(line_num),
                format!("JSON パース失敗: {e}"),
                "JSON パース失敗",
            ),
        }
    }
    out
}

/// 単一 JSON オブジェクトを Value に。
fn read_single_value(path: &Path, report: &mut FileReport) -> Vec<(usize, Value)> {
    match std::fs::read_to_string(path) {
        Ok(s) => match serde_json::from_str::<Value>(&s) {
            Ok(v) => vec![(1, v)],
            Err(e) => {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    line: None,
                    message: format!("JSON パース失敗: {e}"),
                });
                Vec::new()
            }
        },
        Err(e) => {
            report.findings.push(Finding {
                severity: Severity::Error,
                line: None,
                message: format!("読み取り失敗: {e}"),
            });
            Vec::new()
        }
    }
}

/// セマンティック検査: 空件数・ユニークキー重複・カウント負値・ページネーション整合。
fn semantic_checks(spec: &DataFileSpec, values: &[(usize, Value)], report: &mut FileReport) {
    if values.is_empty() {
        report.findings.push(Finding {
            severity: Severity::Warn,
            line: None,
            message: "レコードが0件です".to_string(),
        });
        return;
    }

    // ユニークキー重複。
    if !spec.unique_key.is_empty() {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (line, v) in values {
            let key: Vec<String> = spec
                .unique_key
                .iter()
                .map(|p| {
                    extract(v, p)
                        .map(value_to_key)
                        .unwrap_or_else(|| "∅".to_string())
                })
                .collect();
            let key = key.join("|");
            if !seen.insert(key.clone()) {
                push_capped(
                    &mut report.findings,
                    Severity::Error,
                    Some(*line),
                    format!("ユニークキー重複: ({}) = {key}", spec.unique_key.join(", ")),
                    "ユニークキー重複",
                );
            }
        }
    }

    // カウント系フィールドの負値 (入れ子も走査)。
    for (line, v) in values {
        let mut bad = Vec::new();
        scan_negative_counts(v, &mut bad);
        for entry in bad {
            push_capped(
                &mut report.findings,
                Severity::Error,
                Some(*line),
                format!("カウント負値: {entry}"),
                "カウント負値",
            );
        }
    }

    // ページネーション整合 (flatten された page / total_pages)。
    for (line, v) in values {
        if let (Some(page), Some(total)) = (
            v.get("page").and_then(Value::as_u64),
            v.get("total_pages").and_then(Value::as_u64),
        ) {
            if total >= 1 && (page < 1 || page > total) {
                push_capped(
                    &mut report.findings,
                    Severity::Error,
                    Some(*line),
                    format!("ページ範囲外: page={page} / total_pages={total}"),
                    "ページ範囲外",
                );
            }
        }
    }
}

/// 同種メッセージのスパムを防ぐため、種別ごとに最大件数で打ち切る。
fn push_capped(
    findings: &mut Vec<Finding>,
    severity: Severity,
    line: Option<usize>,
    message: String,
    cap_kind: &str,
) {
    const CAP: usize = 20;
    let count = findings
        .iter()
        .filter(|f| f.message.starts_with(cap_kind))
        .count();
    if count < CAP {
        findings.push(Finding {
            severity,
            line,
            message,
        });
    } else if count == CAP {
        findings.push(Finding {
            severity,
            line: None,
            message: format!("{cap_kind}: 他にも検出 (表示は{CAP}件で打ち切り)"),
        });
    }
}

/// ドット区切りパスで Value を辿る。
fn extract<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = v;
    for seg in path.split('.') {
        cur = cur.get(seg)?;
    }
    Some(cur)
}

fn value_to_key(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// "count" を含むキーで負の整数を持つものを再帰収集。
fn scan_negative_counts(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, val) in m {
                if k.contains("count") {
                    if let Some(n) = val.as_i64() {
                        if n < 0 {
                            out.push(format!("{k}={n}"));
                        }
                    }
                }
                scan_negative_counts(val, out);
            }
        }
        Value::Array(a) => {
            for val in a {
                scan_negative_counts(val, out);
            }
        }
        _ => {}
    }
}

/// 全レコードのトップレベルフィールドを集計する。
fn field_stats<'a>(values: impl Iterator<Item = &'a Value>) -> Vec<FieldStat> {
    use std::collections::BTreeMap;

    // 出現順を保ちたいので (順序保持の) Vec + index map。
    let mut order: Vec<String> = Vec::new();
    let mut idx: BTreeMap<String, usize> = BTreeMap::new();
    let mut stats: Vec<FieldStat> = Vec::new();

    for v in values {
        let Some(obj) = v.as_object() else { continue };
        for (k, val) in obj {
            let i = *idx.entry(k.clone()).or_insert_with(|| {
                order.push(k.clone());
                stats.push(FieldStat {
                    name: k.clone(),
                    ty: "scalar",
                    present: 0,
                    null: 0,
                    bool_true: 0,
                    bool_false: 0,
                });
                stats.len() - 1
            });
            let s = &mut stats[i];
            s.present += 1;
            match val {
                Value::Null => s.null += 1,
                Value::Bool(true) => {
                    s.ty = "bool";
                    s.bool_true += 1;
                }
                Value::Bool(false) => {
                    s.ty = "bool";
                    s.bool_false += 1;
                }
                Value::Array(_) => s.ty = "array",
                Value::Object(_) => s.ty = "object",
                _ => {}
            }
        }
    }

    stats
}
