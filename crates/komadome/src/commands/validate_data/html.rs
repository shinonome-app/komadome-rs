//! 検証レポートを人間が目視確認するための自己完結 HTML を生成する。
//!
//! サイト本体のテンプレート (natsuzora) とは独立した開発者向けツールなので、
//! contract に縛られないよう Rust 側で直接 HTML を組み立てる。

use super::report::{FileReport, Severity};
use serde_json::Value;
use std::fmt::Write as _;

/// 全ファイルのレポートを1枚の HTML ページにまとめる。
pub fn render(reports: &[FileReport], build_date: &str) -> String {
    let mut h = String::new();

    let files_ok = reports
        .iter()
        .filter(|r| r.present && r.errors() == 0)
        .count();
    let files_err = reports.iter().filter(|r| r.errors() > 0).count();
    let files_warn = reports
        .iter()
        .filter(|r| r.errors() == 0 && r.warns() > 0)
        .count();
    let total_records: usize = reports.iter().map(|r| r.record_count).sum();

    h.push_str("<!DOCTYPE html>\n<html lang=\"ja\">\n<head>\n<meta charset=\"utf-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n");
    h.push_str("<title>komadome-rs データ検査レポート</title>\n");
    h.push_str(STYLE);
    h.push_str("</head>\n<body>\n");

    let _ = write!(
        h,
        "<header><h1>komadome-rs データ検査レポート</h1>\
         <p class=\"meta\">生成日: {date} ／ \
         <span class=\"ok\">OK {ok}</span> ・ \
         <span class=\"warn\">WARN {warn}</span> ・ \
         <span class=\"err\">ERROR {err}</span> ／ 総レコード {rec}</p></header>\n",
        date = esc(build_date),
        ok = files_ok,
        warn = files_warn,
        err = files_err,
        rec = total_records,
    );

    // 目次。
    h.push_str("<nav class=\"toc\"><ul>\n");
    for r in reports {
        let _ = write!(
            h,
            "<li>{badge} <a href=\"#{anchor}\">{name}</a> <span class=\"count\">{cnt}</span></li>\n",
            badge = badge(r),
            anchor = esc(&r.name),
            name = esc(&r.name),
            cnt = if r.present {
                format!("{} 件", r.record_count)
            } else {
                "—".to_string()
            },
        );
    }
    h.push_str("</ul></nav>\n");

    for r in reports {
        render_file(&mut h, r);
    }

    h.push_str("</body>\n</html>\n");
    h
}

fn render_file(h: &mut String, r: &FileReport) {
    let _ = write!(
        h,
        "<section id=\"{anchor}\"><h2>{badge} {name} <small>{label}</small></h2>\n",
        anchor = esc(&r.name),
        badge = badge(r),
        name = esc(&r.name),
        label = esc(&r.label),
    );

    if !r.present {
        h.push_str("<p class=\"warn\">ファイルが存在しません。</p></section>\n");
        return;
    }

    let _ = write!(
        h,
        "<p class=\"meta\">{rec} レコード ／ エラー {err} ・ 警告 {warn}</p>\n",
        rec = r.record_count,
        err = r.errors(),
        warn = r.warns(),
    );

    // 検出事項。
    if !r.findings.is_empty() {
        h.push_str("<table class=\"findings\"><thead><tr><th>種別</th><th>行</th><th>内容</th></tr></thead><tbody>\n");
        for f in &r.findings {
            let (cls, sev) = match f.severity {
                Severity::Error => ("err", "ERROR"),
                Severity::Warn => ("warn", "WARN"),
            };
            let _ = write!(
                h,
                "<tr class=\"{cls}\"><td>{sev}</td><td>{line}</td><td>{msg}</td></tr>\n",
                line = f.line.map(|l| l.to_string()).unwrap_or_default(),
                msg = esc(&f.message),
            );
        }
        h.push_str("</tbody></table>\n");
    }

    // フィールド分布。
    if !r.fields.is_empty() {
        h.push_str("<h3>フィールド分布</h3>\n");
        h.push_str("<table class=\"fields\"><thead><tr><th>field</th><th>型</th><th>present</th><th>null</th><th>true/false</th></tr></thead><tbody>\n");
        for s in &r.fields {
            let boolcol = if s.ty == "bool" {
                format!("{} / {}", s.bool_true, s.bool_false)
            } else {
                String::new()
            };
            let _ = write!(
                h,
                "<tr><td>{name}</td><td>{ty}</td><td>{present}</td><td>{null}</td><td>{boolcol}</td></tr>\n",
                name = esc(&s.name),
                ty = s.ty,
                present = s.present,
                null = s.null,
            );
        }
        h.push_str("</tbody></table>\n");
    }

    // サンプル。
    if !r.samples.is_empty() {
        let _ = write!(h, "<h3>サンプル (先頭 {} 件)</h3>\n", r.samples.len());
        render_samples(h, &r.samples);
    }

    h.push_str("</section>\n");
}

/// サンプル群をトップレベルキーを列にした表で描く。
fn render_samples(h: &mut String, samples: &[Value]) {
    // 列 = サンプル横断のトップレベルキー (初出順)。
    let mut columns: Vec<String> = Vec::new();
    for s in samples {
        if let Some(obj) = s.as_object() {
            for k in obj.keys() {
                if !columns.contains(k) {
                    columns.push(k.clone());
                }
            }
        }
    }

    h.push_str("<div class=\"scroll\"><table class=\"samples\"><thead><tr>");
    for c in &columns {
        let _ = write!(h, "<th>{}</th>", esc(c));
    }
    h.push_str("</tr></thead><tbody>\n");

    for s in samples {
        h.push_str("<tr>");
        for c in &columns {
            let cell = s.get(c).map(cell_text).unwrap_or_default();
            let _ = write!(h, "<td>{}</td>", esc(&cell));
        }
        h.push_str("</tr>\n");
    }
    h.push_str("</tbody></table></div>\n");
}

/// 1セルの表示文字列。配列/オブジェクトは要約する。
fn cell_text(v: &Value) -> String {
    match v {
        Value::Null => "—".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => truncate(s, 60),
        Value::Array(a) => format!("[{} 件]", a.len()),
        Value::Object(o) => {
            // id + name/title があれば要約、なければ {…}。
            let id = o.get("id").or_else(|| o.get("person_id"));
            let label = o
                .get("name")
                .or_else(|| o.get("title"))
                .and_then(Value::as_str);
            match (id, label) {
                (Some(id), Some(label)) => format!("{}: {}", id, truncate(label, 40)),
                (Some(id), None) => format!("id={id}"),
                (None, Some(label)) => truncate(label, 40),
                (None, None) => "{…}".to_string(),
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let head: String = s.chars().take(max).collect();
        format!("{head}…")
    }
}

fn badge(r: &FileReport) -> &'static str {
    if !r.present {
        "<span class=\"badge missing\">MISSING</span>"
    } else if r.errors() > 0 {
        "<span class=\"badge err\">ERROR</span>"
    } else if r.warns() > 0 {
        "<span class=\"badge warn\">WARN</span>"
    } else {
        "<span class=\"badge ok\">OK</span>"
    }
}

/// 最低限の HTML エスケープ。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const STYLE: &str = r#"<style>
:root { color-scheme: light dark; }
body { font-family: system-ui, sans-serif; margin: 0; padding: 1.5rem; line-height: 1.5; }
header h1 { margin: 0 0 .25rem; font-size: 1.4rem; }
.meta { color: #666; font-size: .9rem; margin: .25rem 0 1rem; }
.toc ul { list-style: none; padding: 0; columns: 2; }
.toc li { margin: .15rem 0; }
.toc .count { color: #888; font-size: .85rem; }
section { border-top: 1px solid #ccc; padding: 1rem 0; }
section h2 { font-size: 1.1rem; }
section h2 small { color: #888; font-weight: normal; font-size: .85rem; }
h3 { font-size: .95rem; margin: 1rem 0 .35rem; }
table { border-collapse: collapse; font-size: .82rem; }
th, td { border: 1px solid #d0d0d0; padding: .2rem .5rem; text-align: left; vertical-align: top; }
th { background: rgba(127,127,127,.12); }
.scroll { overflow-x: auto; }
.samples td { white-space: nowrap; max-width: 24rem; overflow: hidden; text-overflow: ellipsis; }
.badge { display: inline-block; padding: 0 .4rem; border-radius: .3rem; font-size: .72rem; font-weight: bold; color: #fff; }
.badge.ok { background: #2e7d32; }
.badge.warn { background: #ed6c02; }
.badge.err { background: #c62828; }
.badge.missing { background: #757575; }
span.ok { color: #2e7d32; } span.warn { color: #ed6c02; } span.err { color: #c62828; }
tr.err td { background: rgba(198,40,40,.10); }
tr.warn td { background: rgba(237,108,2,.10); }
</style>
"#;
