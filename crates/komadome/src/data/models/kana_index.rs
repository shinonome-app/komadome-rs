use serde::{Deserialize, Serialize};

/// カナ列で区切られたセクション付き人物索引データ。
///
/// 公開人物索引 (`person_indexes.jsonl`)・全人物索引 (`person_all_indexes.jsonl`)・
/// WIP人物索引 (`wip_person_indexes.jsonl`) は、いずれも「カナ列 → セクション →
/// 人物項目」という同一構造で、違いは項目 (`I`) の型だけ。共通の器をここに置き、
/// 各ファイルは項目型を当てた型エイリアスとして定義する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanaIndexData<I> {
    pub kana_column: String,
    pub column_display: String,
    pub sections: Vec<KanaIndexSection<I>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanaIndexSection<I> {
    pub kana_char: String,
    pub section_index: usize,
    pub people: Vec<I>,
}
