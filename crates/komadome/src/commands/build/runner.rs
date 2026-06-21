//! ページ生成バッチの共通ランナー。
//!
//! 各 builder に散在していた「プログレスバー生成」「並列レンダリング」
//! 「成功/失敗のカウント」「エラー出力」「バー更新」という横断的処理を集約する。
//! builder 側はアイテム 1 件をレンダリングするクロージャと、失敗時の識別子だけを渡す。

use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;

use super::BuildStats;

/// `BuildStats` の特定カウンタを指す関数ポインタ (例: `|s| &s.cards_built`)。
pub type Counter = fn(&BuildStats) -> &AtomicUsize;

/// 共通スタイルのプログレスバーを `multi` に追加して返す。
/// `label` は `cards` のような先頭ラベル、`bar_style` は `40.cyan/blue` 形式の bar 装飾指定。
pub fn styled_bar(multi: &MultiProgress, label: &str, bar_style: &str, len: u64) -> ProgressBar {
    let pb = multi.add(ProgressBar::new(len));
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "[{label}] {{bar:{bar_style}}} {{pos}}/{{len}} ({{per_sec}})"
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb
}

/// 1 件分の結果を集計する。成功なら `counter`、失敗なら `stats.errors` を加算し、
/// 失敗時は `describe(item)` でアイテムを識別する 1 行を stderr に出す。
fn record<T>(
    item: &T,
    result: Result<()>,
    stats: &BuildStats,
    counter: Counter,
    describe: &impl Fn(&T) -> String,
) {
    match result {
        Ok(_) => {
            counter(stats).fetch_add(1, Ordering::Relaxed);
        }
        Err(e) => {
            stats.errors.fetch_add(1, Ordering::Relaxed);
            eprintln!("Error building {}: {:#}", describe(item), e);
        }
    }
}

/// `items` を並列にレンダリングし、結果を集計しながらプログレスバーを進める。
/// 完了時にバーを finish する。
pub fn render_each<T, R, D>(
    items: &[T],
    pb: &ProgressBar,
    stats: &BuildStats,
    counter: Counter,
    render: R,
    describe: D,
) where
    T: Sync,
    R: Fn(&T) -> Result<()> + Sync,
    D: Fn(&T) -> String + Sync,
{
    items.par_iter().for_each(|item| {
        record(item, render(item), stats, counter, &describe);
        pb.inc(1);
    });
    pb.finish_with_message("done");
}

/// `items` を逐次レンダリングし、結果を集計する。成功件数を返す。
/// プログレスバーを使わない小規模バッチ向け。
pub fn render_each_seq<T, R, D>(
    items: &[T],
    stats: &BuildStats,
    counter: Counter,
    render: R,
    describe: D,
) -> usize
where
    R: Fn(&T) -> Result<()>,
    D: Fn(&T) -> String,
{
    let before = counter(stats).load(Ordering::Relaxed);
    for item in items {
        record(item, render(item), stats, counter, &describe);
    }
    counter(stats).load(Ordering::Relaxed) - before
}
