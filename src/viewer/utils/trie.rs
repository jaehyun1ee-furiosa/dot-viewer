#![allow(dead_code)]

use std::str;

use trie_rs::TrieBuilder;

pub(crate) struct Trie {
    items: Vec<String>,
    trie: trie_rs::Trie<u8>,
}

impl FromIterator<String> for Trie {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let mut builder = TrieBuilder::new();

        let mut items = Vec::new();
        for id in iter {
            items.push(id.clone());
            builder.push(id);
        }
        let trie = builder.build();

        Trie { items, trie }
    }
}

impl Trie {
    pub fn autocomplete(&self, key: &str) -> Option<String> {
        let predictions = if key.is_empty() { self.items.clone() } else { self.predict(key) };

        longest_common_prefix(&predictions)
    }

    fn predict(&self, key: &str) -> Vec<String> {
        let trie_search_result = self.trie.predictive_search(key);
        (trie_search_result.into_iter()).map(|s| String::from_utf8(s).unwrap()).collect()
    }
}

// https://leetcode.com/problems/longest-common-prefix/solutions/1134124/faster-than-100-in-memory-and-runtime-by-rust/
fn longest_common_prefix(strs: &[String]) -> Option<String> {
    if strs.is_empty() {
        return None;
    }

    let mut str_iters = strs.iter().map(|s| s.chars()).collect::<Vec<_>>();

    for (i, c) in strs[0].char_indices() {
        for str_iter in &mut str_iters {
            if str_iter.next().filter(|&x| x == c).is_none() {
                return Some(strs[0][..i].to_string());
            }
        }
    }

    Some(strs[0].clone())
}
