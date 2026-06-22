pub mod build;
pub mod clean;
pub mod export;
pub mod generate_zip;
pub mod stats;
pub mod validate;

/// whatsnew の最初の対象年。年ページの生成範囲(export)と footer の年リンク(build)で共有する。
pub const WHATSNEW_FIRST_YEAR: i32 = 1997;
