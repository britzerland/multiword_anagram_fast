use std::collections::{HashMap, HashSet}; // Keep these for SolverConstraints
use std::cmp::Ordering;
use std::time::Instant;

use super::trie::{Trie, TrieNode};
// Only CharCounts is directly used from char_utils in this module's scope.
// normalize_word and others are used within AnagramSolver methods but called from char_utils.
use super::char_utils::CharCounts; 

pub struct SolverInternalState {
    pub start_time: Instant,
    pub timed_out: bool,
    pub solutions_found_count: usize,
}

pub struct SolverConstraints {
    pub must_start_with: Option<HashMap<char, usize>>,
    pub can_only_ever_start_with: Option<HashSet<char>>,
    pub must_not_start_with: Option<HashSet<char>>,
    pub max_words: Option<usize>,
    pub min_word_length: Option<usize>,
    pub timeout_seconds: Option<f64>,  
    pub max_solutions: Option<usize>,  
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

        let mut internal_state = SolverInternalState { // <--- Initialize internal state
            start_time: Instant::now(),
            timed_out: false,
            solutions_found_count: 0,
        };

        self.backtrack(
            &mut current_path,
            &mut current_char_counts,
            &self.trie.root, 
            constraints,
            &mut solutions_set,
            &mut internal_state, // <--- Pass internal state
        );
        
        let mut final_solutions: Vec<Vec<String>> = solutions_set.into_iter().collect();

        final_solutions.sort_by(|a, b| {
            let len_cmp = a.len().cmp(&b.len());
            if len_cmp != Ordering::Equal {
                return len_cmp;
            }
            let min_len_a = a.iter().map(|w| w.len()).min().unwrap_or(0);
            let min_len_b = b.iter().map(|w| w.len()).min().unwrap_or(0);
            min_len_b.cmp(&min_len_a)
                .then_with(|| a.cmp(b))
        });

        final_solutions
    }

    fn backtrack(
        &self,
        current_path: &mut Vec<String>,
        remaining_counts: &mut CharCounts,
        _start_node_for_this_level: &TrieNode,
        constraints: &SolverConstraints,
        solutions_set: &mut HashSet<Vec<String>>,
        internal_state: &mut SolverInternalState, // <--- Receive internal state
    ) {
        // ---> CHECK LIMITS EARLY <---
        if internal_state.timed_out { return; }
        if let Some(timeout_sec) = constraints.timeout_seconds {
            if internal_state.start_time.elapsed().as_secs_f64() > timeout_sec {
                internal_state.timed_out = true;
                // Optionally print a message or log: eprintln!("Anagram search timed out.");
                return;
            }
        }
        if let Some(max_sol) = constraints.max_solutions {
            if internal_state.solutions_found_count >= max_sol {
                return;
            }
        }

        // Pruning: Max words
        if let Some(max_w) = constraints.max_words {
            if current_path.len() > max_w {
                return;
            }
        }

        // Base Case: All characters used up
        if remaining_counts.is_empty() {
            if !current_path.is_empty() { 
                if let Some(max_w) = constraints.max_words {
                    if current_path.len() > max_w { return; }
                }
                if let Some(required_starts) = &constraints.must_start_with {
                    let mut current_starts_counts: HashMap<char, usize> = HashMap::new();
                    for word in current_path.iter() {
                        if let Some(first_char) = word.chars().next() {
                            *current_starts_counts.entry(first_char).or_insert(0) += 1;
                        }
                    }
                    for (req_char, req_count) in required_starts {
                        if current_starts_counts.get(req_char).unwrap_or(&0) < req_count { return; }
                    }
                }
                
                let mut solution_candidate = current_path.clone();
                solution_candidate.sort_unstable(); 
                if solutions_set.insert(solution_candidate) { // Only count if it's a new unique solution
                    internal_state.solutions_found_count += 1;
                    if let Some(max_sol) = constraints.max_solutions { // Check again after adding
                        if internal_state.solutions_found_count >= max_sol {
                            return; // Reached max solutions, can stop further search from this path
                        }
                    }
                }
            }
            return;
        }
        
        if remaining_counts.total() < self.trie.get_min_word_len() { return; }
        if let Some(min_len) = constraints.min_word_length { // Check if remaining letters can form a word of min_len
            if remaining_counts.total() < min_len && !current_path.is_empty() { // if not first word
                 return;
            }
             if remaining_counts.total() < min_len && current_path.is_empty() && self.trie.max_word_len < min_len {
                // If it's the first word, and no word in the dictionary meets min_len with remaining letters
                // This specific check might be too aggressive or complex here,
                // the min_word_length check in find_one_word_recursive is more direct.
                // The main check is that remaining_counts.total() must be >= min_word_length
                // for the *next* word to be formed.
                // If remaining_counts.total() < min_len, then no next word can be formed.
            }
        }

        if let Some(max_w) = constraints.max_words {
            if current_path.len() == max_w && !remaining_counts.is_empty() { return; }
        }

        let mut word_buffer = String::new();
        self.find_one_word_recursive(
            &self.trie.root, 
            &mut word_buffer,
            remaining_counts,
            current_path,
            constraints,
            solutions_set,
            internal_state, // <--- Pass internal state
        );
    }

    fn find_one_word_recursive(
        &self,
        current_trie_node: &TrieNode,
        word_so_far: &mut String,
        current_overall_counts: &mut CharCounts, 
        path: &mut Vec<String>, 
        constraints: &SolverConstraints,
        solutions_set: &mut HashSet<Vec<String>>,
        internal_state: &mut SolverInternalState, // <--- Receive internal state
    ) {
        // ---> CHECK LIMITS <---
        if internal_state.timed_out { return; }
        // No need to check timeout again here if checked in backtrack, but doesn't hurt much
        // if let Some(timeout_sec) = constraints.timeout_seconds {
        //     if internal_state.start_time.elapsed().as_secs_f64() > timeout_sec {
        //         internal_state.timed_out = true;
        //         return;
        //     }
        // }
        if let Some(max_sol) = constraints.max_solutions {
            if internal_state.solutions_found_count >= max_sol {
                return;
            }
        }

        if current_trie_node.is_end_of_word && !word_so_far.is_empty() {
            let mut passes_min_length = true;
            if let Some(min_len) = constraints.min_word_length {
                if word_so_far.len() < min_len {
                    passes_min_length = false;
                }
            }

            if passes_min_length {
                path.push(word_so_far.clone());
                // Pass internal_state to backtrack
                self.backtrack(path, current_overall_counts, &self.trie.root, constraints, solutions_set, internal_state);
                path.pop(); 
                // After returning from backtrack, check limits again in case they were hit
                if internal_state.timed_out { return; }
                if let Some(max_sol) = constraints.max_solutions {
                    if internal_state.solutions_found_count >= max_sol {
                        return;
                    }
                }
            }
        }
        
        if word_so_far.len() > self.trie.max_word_len || word_so_far.len() > current_overall_counts.total() {
            return;
        }

        for (key_ref_char_code, value_ref_next_node) in current_trie_node.children.iter() {
            let ch: char = *key_ref_char_code;

            if current_overall_counts.get(ch).unwrap_or(0) > 0 {
                if word_so_far.is_empty() {
                    if !constraints.is_valid_start_char(ch) {
                        continue; 
                    }
                }

                current_overall_counts.decrement_char(ch).unwrap();
                word_so_far.push(ch);

                self.find_one_word_recursive( // Pass internal_state
                    value_ref_next_node,
                    word_so_far,
                    current_overall_counts,
                    path,
                    constraints,
                    solutions_set,
                    internal_state,
                );

                word_so_far.pop(); 
                current_overall_counts.increment_char(ch).unwrap();

                // After exploring a branch, check limits
                if internal_state.timed_out { return; }
                if let Some(max_sol) = constraints.max_solutions {
                    if internal_state.solutions_found_count >= max_sol {
                        return;
                    }
                }
            }
        }
    }
}