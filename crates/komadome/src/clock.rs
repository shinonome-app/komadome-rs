//! Build clock: a single, injectable "today" for deterministic output.
//!
//! All export commands derive dates (current year, "latest published", page
//! counts) from [`build_date`] instead of the wall clock. The build/generator
//! phase reads the date back from `masters.json`'s `exported_on`, so pinning the
//! export date makes the whole pipeline reproducible — a prerequisite for
//! comparing output against `komadome` (Rails). See `docs/parity.md`.
//!
//! Resolution order (first match wins):
//!   1. explicit override passed to [`init`] (the `--date` CLI flag)
//!   2. the `KOMADOME_BUILD_DATE` env var (YYYY-MM-DD)
//!   3. the system local date

use std::sync::OnceLock;

use chrono::NaiveDate;

const ENV_KEY: &str = "KOMADOME_BUILD_DATE";

static BUILD_DATE: OnceLock<NaiveDate> = OnceLock::new();

/// Pin the build date at startup. Only the first call takes effect.
///
/// `explicit` is the parsed `--date` CLI flag (if any); when `None`, the env
/// var and then the system date are consulted.
pub fn init(explicit: Option<NaiveDate>) {
    let date = explicit
        .or_else(env_date)
        .unwrap_or_else(|| chrono::Local::now().date_naive());
    let _ = BUILD_DATE.set(date);
}

/// The pinned build date.
///
/// If [`init`] was never called (e.g. a unit test invoking an export helper
/// directly), this resolves lazily from the env var or the system date so call
/// sites never panic.
pub fn build_date() -> NaiveDate {
    *BUILD_DATE.get_or_init(|| env_date().unwrap_or_else(|| chrono::Local::now().date_naive()))
}

fn env_date() -> Option<NaiveDate> {
    let raw = std::env::var(ENV_KEY).ok()?;
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").ok()
}
