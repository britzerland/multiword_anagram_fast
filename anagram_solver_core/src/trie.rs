use std::collections::HashMap;
use super::char_utils::{CharCounts, normalize_word, char_to_index, index_to_char};

#[derive(Default)]
pub struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub is_end_of_word: bool,
    // Store CharCounts for each word to quickly check if it can be formed
    // This is only useful if words are short. For anagramming, we build words char by char.
    // So, is_end_of_word is enough. If we want to store the word itself:
    // pub word: Option<String> 
}

pub struct Trie {
    pub root: TrieNode,
    min_word_len: usize, // Minimum length of a word in the trie
    max_word_len: usize, // Maximum length of a word in the trie
}

impl Trie {
    pub fn new() -> Self {
        Trie { root: TrieNode::default(), min_word_len: usize::MAX, max_word_len: 0 }
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

