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
