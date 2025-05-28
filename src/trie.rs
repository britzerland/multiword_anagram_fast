use std::collections::HashMap;
// Removed CharCounts, index_to_char as they are not directly used here.
// char_to_index was also removed as it was for direct access, normalize_word handles char properties.
use super::char_utils::normalize_word; 

#[derive(Default)]
pub struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub is_end_of_word: bool,
}

pub struct Trie {
    pub root: TrieNode,
    pub min_word_len: usize, // Made public
    pub max_word_len: usize, // Made public
}

impl Trie {
    pub fn new() -> Self {
        Trie { 
            root: TrieNode::default(), 
            min_word_len: usize::MAX, 
            max_word_len: 0 
        }
    }

    pub fn insert(&mut self, word: &str) {
        let normalized = normalize_word(word);
        if normalized.is_empty() {
            return;
        }

        let len = normalized.len();
        self.min_word_len = self.min_word_len.min(len);
        self.max_word_len = self.max_word_len.max(len);

        let mut current_node = &mut self.root;
        for c in normalized.chars() {
            current_node = current_node.children.entry(c).or_default();
        }
        current_node.is_end_of_word = true;
    }

    pub fn get_min_word_len(&self) -> usize {
        if self.min_word_len == usize::MAX { 0 } else { self.min_word_len }
    }
}