use std::collections::{HashMap, HashSet}; // Keep these for SolverConstraints
use std::cmp::Ordering;
use super::trie::{Trie, TrieNode};
// Only CharCounts is directly used from char_utils in this module's scope.
// normalize_word and others are used within AnagramSolver methods but called from char_utils.
use super::char_utils::CharCounts; 


pub struct SolverConstraints {
    pub must_start_with: Option<HashMap<char, usize>>,
    pub can_only_ever_start_with: Option<HashSet<char>>,
    pub must_not_start_with: Option<HashSet<char>>,
    pub max_words: Option<usize>,
}

impl SolverConstraints {
    fn is_valid_start_char(&self, c: char) -> bool {
        if let Some(disallowed) = &self.must_not_start_with {
            if disallowed.contains(&c) {
                return false;
            }
        }
        if let Some(allowed) = &self.can_only_ever_start_with {
            if !allowed.contains(&c) {
                return false;
            }
        }
        true
    }
}

pub struct AnagramSolver {
    trie: Trie,
}

impl AnagramSolver {
    pub fn new() -> Self {
        AnagramSolver { trie: Trie::new() }
    }

    pub fn load_dictionary_from_words(&mut self, words: &[String]) {
        for word in words {
            self.trie.insert(word);
        }
    }
    
    pub fn load_dictionary_from_text(&mut self, text_content: &str) {
        for line in text_content.lines() {
            self.trie.insert(line);
        }
    }

    pub fn add_word(&mut self, word: &str) {
        self.trie.insert(word);
    }
    
    pub fn solve(&self, phrase: &str, constraints: &SolverConstraints) -> Vec<Vec<String>> {
        let target_counts = match CharCounts::from_str(phrase) {
            Ok(counts) => counts,
            Err(_) => return Vec::new(), 
        };

        if target_counts.is_empty() || self.trie.get_min_word_len() == 0 {
            return Vec::new();
        }
        
        let mut solutions_set: HashSet<Vec<String>> = HashSet::new();
        let mut current_path: Vec<String> = Vec::new();
        let mut current_char_counts = target_counts.clone();

        self.backtrack(
            &mut current_path,
            &mut current_char_counts,
            &self.trie.root, 
            constraints,
            &mut solutions_set,
        );
        
        let mut final_solutions: Vec<Vec<String>> = solutions_set.into_iter().collect();

        final_solutions.sort_by(|a, b| {
            let len_cmp = a.len().cmp(&b.len());
            if len_cmp != Ordering::Equal {
                return len_cmp;
            }
            let min_len_a = a.iter().map(|w| w.len()).min().unwrap_or(0);
            let min_len_b = b.iter().map(|w| w.len()).min().unwrap_or(0);
            min_len_b.cmp(&min_len_a) // Descending
                .then_with(|| a.cmp(b)) // Lexicographical tie-breaking
        });

        final_solutions
    }

    fn backtrack(
        &self,
        current_path: &mut Vec<String>,
        remaining_counts: &mut CharCounts,
        _start_node_for_this_level: &TrieNode, // Prefixed with _ as it's unused currently
        constraints: &SolverConstraints,
        solutions_set: &mut HashSet<Vec<String>>,
    ) {
        if let Some(max_w) = constraints.max_words {
            if current_path.len() > max_w {
                return;
            }
        }

        if remaining_counts.is_empty() {
            if !current_path.is_empty() { 
                if let Some(max_w) = constraints.max_words {
                    if current_path.len() > max_w { // Check again, as a word might have completed it
                        return;
                    }
                }
                if let Some(required_starts) = &constraints.must_start_with {
                    let mut current_starts_counts: HashMap<char, usize> = HashMap::new();
                    for word in current_path.iter() {
                        if let Some(first_char) = word.chars().next() {
                            *current_starts_counts.entry(first_char).or_insert(0) += 1;
                        }
                    }
                    for (req_char, req_count) in required_starts {
                        if current_starts_counts.get(req_char).unwrap_or(&0) < req_count {
                            return; 
                        }
                    }
                }
                let mut solution_candidate = current_path.clone();
                solution_candidate.sort_unstable(); 
                solutions_set.insert(solution_candidate);
            }
            return;
        }
        
        if remaining_counts.total() < self.trie.get_min_word_len() {
            return;
        }
        if let Some(max_w) = constraints.max_words {
            if current_path.len() == max_w && !remaining_counts.is_empty() {
                return;
            }
        }

        let mut word_buffer = String::new();
        self.find_one_word_recursive(
            &self.trie.root, 
            &mut word_buffer,
            remaining_counts,
            current_path,
            constraints,
            solutions_set,
        );
    }

    fn find_one_word_recursive(
        &self,
        current_trie_node: &TrieNode, // current_trie_node is &TrieNode
        word_so_far: &mut String,
        current_overall_counts: &mut CharCounts, 
        path: &mut Vec<String>, 
        constraints: &SolverConstraints,
        solutions_set: &mut HashSet<Vec<String>>,
    ) {
        if current_trie_node.is_end_of_word && !word_so_far.is_empty() {
            path.push(word_so_far.clone());
            self.backtrack(path, current_overall_counts, &self.trie.root, constraints, solutions_set);
            path.pop(); 
        }
        
        if word_so_far.len() > self.trie.max_word_len || word_so_far.len() > current_overall_counts.total() {
            return;
        }

        // Using .iter() for clarity on types
        for (key_ref_char_code, value_ref_next_node) in current_trie_node.children.iter() {
            // Now, key_ref_char_code is definitely &char
            // And value_ref_next_node is definitely &TrieNode

            let ch: char = *key_ref_char_code; // Dereference &char to get char

            if current_overall_counts.get(ch).unwrap_or(0) > 0 {
                if word_so_far.is_empty() {
                    if !constraints.is_valid_start_char(ch) {
                        continue; 
                    }
                }

                current_overall_counts.decrement_char(ch).unwrap();
                word_so_far.push(ch);

                // value_ref_next_node is &TrieNode, which is what the function expects.
                self.find_one_word_recursive(
                    value_ref_next_node, // Pass the reference directly
                    word_so_far,
                    current_overall_counts,
                    path,
                    constraints,
                    solutions_set,
                );

                word_so_far.pop(); 
                current_overall_counts.increment_char(ch).unwrap();
            }
        }
    }
}