use std::collections::HashMap;
use std::hash::Hash;

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
