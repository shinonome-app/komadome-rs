use anyhow::Result;
use serde_json::{Value, json};

/// Build context for the index_top page (総合インデックス)
pub fn build_index_top_context() -> Result<Value> {
    Ok(json!({
        "page_title": "総合インデックス | 青空文庫",
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
    }))
}

/// Build context for the index_all page (登録全作家インデックス)
pub fn build_index_all_context() -> Result<Value> {
    Ok(json!({
        "page_title": "登録全作家インデックス | 青空文庫",
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
    }))
}
