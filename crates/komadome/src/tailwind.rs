//! Tailwind の「動的クラス」の単一の出所。
//!
//! テンプレート(*.ntzr)にリテラルで現れず、実行時に決まるクラスを集約する。
//! builders はここの定数を参照し、`komadome tailwind-safelist` でJSONとして出力する。
//! tailwind.config.js が safelist として読む。

pub mod bgcolor {
    /// 著作権存続
    pub const COPYRIGHT: &str = "bg-rose-50";
    /// 通常
    pub const DEFAULT: &str = "bg-sky-50";
    /// トップ / そらもよう
    pub const WHITE: &str = "bg-white-100";
}

/// テンプレートにリテラルで現れない＝tailwind の safelist に出す必要がある全クラス。
pub const SAFELIST: &[&str] = &[bgcolor::COPYRIGHT, bgcolor::DEFAULT, bgcolor::WHITE];

/// `komadome tailwind-safelist`: SAFELIST を JSON 配列で標準出力する。
pub fn print_safelist() -> anyhow::Result<()> {
    println!("{}", serde_json::to_string(SAFELIST)?);
    Ok(())
}
