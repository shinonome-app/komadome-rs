/// Kana processing utilities for Japanese text sorting
///
/// Ported from Ruby komadome/app/lib/kana.rb

/// Roman letter to kana mapping
pub const ROMA2KANA: &[(&str, Option<&str>)] = &[
    ("a", Some("あ")),
    ("i", Some("い")),
    ("u", Some("う")),
    ("e", Some("え")),
    ("o", Some("お")),
    ("ka", Some("か")),
    ("ki", Some("き")),
    ("ku", Some("く")),
    ("ke", Some("け")),
    ("ko", Some("こ")),
    ("sa", Some("さ")),
    ("si", Some("し")),
    ("su", Some("す")),
    ("se", Some("せ")),
    ("so", Some("そ")),
    ("ta", Some("た")),
    ("ti", Some("ち")),
    ("tu", Some("つ")),
    ("te", Some("て")),
    ("to", Some("と")),
    ("na", Some("な")),
    ("ni", Some("に")),
    ("nu", Some("ぬ")),
    ("ne", Some("ね")),
    ("no", Some("の")),
    ("ha", Some("は")),
    ("hi", Some("ひ")),
    ("hu", Some("ふ")),
    ("he", Some("へ")),
    ("ho", Some("ほ")),
    ("ma", Some("ま")),
    ("mi", Some("み")),
    ("mu", Some("む")),
    ("me", Some("め")),
    ("mo", Some("も")),
    ("ya", Some("や")),
    ("yu", Some("ゆ")),
    ("yo", Some("よ")),
    ("ra", Some("ら")),
    ("ri", Some("り")),
    ("ru", Some("る")),
    ("re", Some("れ")),
    ("ro", Some("ろ")),
    ("wa", Some("わ")),
    ("wo", Some("を")),
    ("nn", Some("ん")),
    ("zz", None), // その他
];

/// Characters in each kana column
pub const COLUMN_CHARS: &[(&str, &str)] = &[
    ("a", "あいうえお"),
    ("ka", "かきくけこ"),
    ("sa", "さしすせそ"),
    ("ta", "たちつてと"),
    ("na", "なにぬねの"),
    ("ha", "はひふへほ"),
    ("ma", "まみむめも"),
    ("ya", "やゆよ"),
    ("ra", "らりるれろ"),
    ("wa", "わをん"),
    ("zz", ""), // その他
];

/// All kana symbols in order
pub const SYMBOLS: &[&str] = &[
    "a", "i", "u", "e", "o", "ka", "ki", "ku", "ke", "ko", "sa", "si", "su", "se", "so", "ta",
    "ti", "tu", "te", "to", "na", "ni", "nu", "ne", "no", "ha", "hi", "hu", "he", "ho", "ma", "mi",
    "mu", "me", "mo", "ya", "yu", "yo", "ra", "ri", "ru", "re", "ro", "wa", "wo", "nn", "zz",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kana {
    pub symbol: String,
    pub index: usize,
}

impl Kana {
    /// Create a Kana from a symbol string
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        SYMBOLS.iter().position(|&s| s == symbol).map(|index| Self {
            symbol: symbol.to_string(),
            index,
        })
    }

    /// Create a Kana from a hiragana character
    pub fn from_kana(kana: &str) -> Self {
        let first_char = kana.chars().next();

        for (symbol, kana_char) in ROMA2KANA {
            if let Some(k) = kana_char {
                if Some(*k) == first_char.map(|c| c.to_string()).as_deref() {
                    if let Some(idx) = SYMBOLS.iter().position(|&s| s == *symbol) {
                        return Self {
                            symbol: symbol.to_string(),
                            index: idx,
                        };
                    }
                }
            }
        }

        // Default to "zz" (other)
        Self {
            symbol: "zz".to_string(),
            index: SYMBOLS.len() - 1,
        }
    }

    /// Get the display kana character for this symbol
    pub fn display_char(&self) -> Option<&'static str> {
        ROMA2KANA
            .iter()
            .find(|(s, _)| *s == self.symbol)
            .and_then(|(_, k)| *k)
    }

    /// Get column symbol for this kana
    pub fn column_symbol(&self) -> &'static str {
        for (col_symbol, chars) in COLUMN_CHARS {
            if chars.contains(self.display_char().unwrap_or("")) {
                return col_symbol;
            }
        }
        "zz"
    }

    /// Get column symbol and index within that column for this kana
    /// Equivalent to Ruby's Kana#to_symbol_and_index
    /// Returns (column_symbol, index_within_column)
    pub fn to_symbol_and_index(&self) -> (&'static str, usize) {
        let kana_char = self.display_char().unwrap_or("");
        if kana_char.is_empty() {
            return ("zz", 0);
        }
        for (col_symbol, chars) in COLUMN_CHARS {
            if let Some(idx) = chars.find(kana_char) {
                // find returns byte offset; convert to char index
                let char_idx = chars[..idx].chars().count();
                return (col_symbol, char_idx);
            }
        }
        ("zz", 0)
    }

    /// Generate sortkey pattern for database queries (if needed)
    pub fn sortkey_pattern(&self) -> String {
        if self.symbol == "zz" {
            return "^[^あ-ん]".to_string();
        }

        if let Some(kana) = self.display_char() {
            format!("^{kana}")
        } else {
            "^[^あ-ん]".to_string()
        }
    }
}

/// Get all symbols for a column (a, ka, sa, ...)
pub fn symbols_in_column(column: &str) -> Vec<&'static str> {
    match column {
        "a" => vec!["a", "i", "u", "e", "o"],
        "ka" => vec!["ka", "ki", "ku", "ke", "ko"],
        "sa" => vec!["sa", "si", "su", "se", "so"],
        "ta" => vec!["ta", "ti", "tu", "te", "to"],
        "na" => vec!["na", "ni", "nu", "ne", "no"],
        "ha" => vec!["ha", "hi", "hu", "he", "ho"],
        "ma" => vec!["ma", "mi", "mu", "me", "mo"],
        "ya" => vec!["ya", "yu", "yo"],
        "ra" => vec!["ra", "ri", "ru", "re", "ro"],
        "wa" => vec!["wa", "wo", "nn"],
        "zz" => vec!["zz"],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_symbol() {
        let kana = Kana::from_symbol("ka").unwrap();
        assert_eq!(kana.symbol, "ka");
        assert_eq!(kana.display_char(), Some("か"));
    }

    #[test]
    fn test_column_symbol() {
        let kana = Kana::from_symbol("ki").unwrap();
        assert_eq!(kana.column_symbol(), "ka");
    }

    #[test]
    fn test_symbols_in_column() {
        let syms = symbols_in_column("ka");
        assert_eq!(syms, vec!["ka", "ki", "ku", "ke", "ko"]);
    }
}
