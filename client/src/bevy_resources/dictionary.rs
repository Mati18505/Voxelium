use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Default, Clone)]
pub struct Dictionary<K: Hash + Eq, V> {
    dictionary: HashMap<K, V>,
}

impl<K: Hash + Eq, V> Dictionary<K, V> {
    pub fn new(values: HashMap<K, V>) -> Dictionary<K, V> {
        Dictionary {
            dictionary: values,
        }
    }

    pub fn set(&mut self, k: K, v: V) {
        self.dictionary.insert(k, v);
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.dictionary.get(k)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.dictionary.iter()
    }
}