use std::collections::HashMap;
use std::hash::Hash;

/// kana 文字すべてにマッチする正規表現パターン (Ruby の KANA_PATTERN と同一)。
/// sortkey が kana で始まるか (= 五十音のどの列に属するか) の判定に使う。
pub const KANA_PATTERN: &str =
    "^[あいうえおか-もやゆよら-ろわをんアイウエオカ-モヤユヨラ-ロワヲンヴ]";

/// 作業中(WIP=未公開)とみなす work_status_id の一覧。
/// 公開済み (work_status_id = 1 かつ started_on <= 基準日) 以外の進行中ステータス。
pub const WIP_WORK_STATUS_IDS: &str = "3,4,5,6,7,8,9,10,11";

/// 作業中(未公開)作品を表す SQL 述語を組み立てる。
/// `date_param` には基準日のバインドプレースホルダ (例: "$1") を渡す。
/// works テーブルが `w` エイリアスである前提。全体が括弧で囲まれるため
/// `AND <predicate>` や `CASE WHEN <predicate> THEN ...` にそのまま埋め込める。
pub fn wip_work_predicate(date_param: &str) -> String {
    format!(
        "(w.work_status_id IN ({WIP_WORK_STATUS_IDS}) \
         OR (w.work_status_id = 1 AND w.started_on > {date_param}))"
    )
}

/// 公開済み作品を表す SQL 述語を組み立てる。
/// `date_param` には基準日のバインドプレースホルダを渡す。works テーブルが `w` エイリアス前提。
pub fn published_work_predicate(date_param: &str) -> String {
    format!("w.work_status_id = 1 AND w.started_on <= {date_param}")
}

/// Group a slice of items by a key extracted by the given function
pub fn group_by<T, K, F>(items: &[T], key_fn: F) -> HashMap<K, Vec<&T>>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    let mut map: HashMap<K, Vec<&T>> = HashMap::new();
    for item in items {
        map.entry(key_fn(item)).or_default().push(item);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_empty() {
        let items: Vec<(i32, &str)> = vec![];
        let result = group_by(&items, |item| item.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_group_by_single_element() {
        let items = vec![(1, "a")];
        let result = group_by(&items, |item| item.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&1].len(), 1);
        assert_eq!(result[&1][0].1, "a");
    }

    #[test]
    fn test_group_by_same_key() {
        let items = vec![(1, "a"), (1, "b"), (1, "c")];
        let result = group_by(&items, |item| item.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&1].len(), 3);
    }

    #[test]
    fn test_group_by_mixed_keys() {
        let items = vec![(1, "a"), (2, "b"), (1, "c"), (3, "d"), (2, "e")];
        let result = group_by(&items, |item| item.0);
        assert_eq!(result.len(), 3);
        assert_eq!(result[&1].len(), 2);
        assert_eq!(result[&2].len(), 2);
        assert_eq!(result[&3].len(), 1);
    }
}
