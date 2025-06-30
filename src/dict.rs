use std::{
    borrow::Borrow,
    collections::{
        BTreeMap,
        btree_map::{Iter, Keys},
    },
    ops::{Index, IndexMut},
};

#[derive(Debug)]
pub struct Dict<K, V> {
    data: BTreeMap<K, V>,
}

impl<K, V> PartialEq for Dict<K, V>
where
    K: Ord,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let mut result = true;
        if self.len() != other.len() {
            result = false;
        } else {
            for key in self.keys() {
                result = result && (self[key] == other[key]);
            }
        }
        result
    }
}

impl<K, Q, V> Index<&Q> for Dict<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        &self.data[key]
    }
}

impl<K, Q, V> IndexMut<&Q> for Dict<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    fn index_mut(&mut self, key: &Q) -> &mut Self::Output {
        self.data.get_mut(key).unwrap()
    }
}

impl<K, V> Default for Dict<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Dict<K, V> {
    pub fn new() -> Self {
        Self { data: BTreeMap::new() }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        self.data.keys()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.data.iter()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        self.data.insert(key, value)
    }

    pub fn first_key_value(&self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        self.data.first_key_value()
    }

    #[allow(dead_code)]
    pub fn from_btree_map(btree_map: BTreeMap<K, V>) -> Self {
        Dict { data: btree_map }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn btree_map_to_dict() {
        let mut b = BTreeMap::<String, i64>::new();
        b.insert("1".to_string(), 1);
        let d1 = Dict::<String, i64>::from_btree_map(b);
        let mut d2 = Dict::<String, i64>::new();
        d2.insert("1".to_string(), 1);
        assert_eq!(d1, d2)
    }
}
