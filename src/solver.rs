use std::collections::{HashMap, HashSet}; // Keep these for SolverConstraints
use std::cmp::Ordering;
use std::time::Instant;

use super::trie::{Trie, TrieNode};
use super::char_utils::CharCounts; 

// Preprocessed pattern structure
#[derive(Clone, Debug)] // Added Clone and Debug
pub struct ProcessedPattern {
    pub text: String, // Normalized text of the pattern
    pub counts: CharCounts,
    // original_index: usize, // If needed for mapping back
}

pub struct SolverInternalState {
    pub start_time: Instant,
    pub timed_out: bool,
    pub solutions_found_count: usize,
    pub patterns_satisfied_mask: Option<Vec<bool>>,
}

pub struct SolverConstraints {
    pub must_start_with: Option<HashMap<char, usize>>,
    pub can_only_ever_start_with: Option<HashSet<char>>,
    pub must_not_start_with: Option<HashSet<char>>,
    pub max_words: Option<usize>,
    pub min_word_length: Option<usize>,
    pub timeout_seconds: Option<f64>,  
    pub max_solutions: Option<usize>,  
    pub contains_patterns: Option<Vec<ProcessedPattern>>,
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
        
        let initial_patterns_mask = constraints.contains_patterns.as_ref().map(|patterns| {
            vec![false; patterns.len()]
        });

        let mut internal_state = SolverInternalState { 
            start_time: Instant::now(),
            timed_out: false,
            solutions_found_count: 0,
            patterns_satisfied_mask: initial_patterns_mask,
        };

        self.backtrack(
            &mut current_path,
            &mut current_char_counts,
            &self.trie.root, 
            constraints,
            &mut solutions_set,
            &mut internal_state, 
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
        // Pattern-based pruning
        if let Some(patterns_to_satisfy) = &constraints.contains_patterns {
            if let Some(satisfied_mask) = &internal_state.patterns_satisfied_mask {
                let mut num_unsatisfied = 0;
                for (i, pattern_proc) in patterns_to_satisfy.iter().enumerate() {
                    if !satisfied_mask[i] {
                        num_unsatisfied += 1;
                        // Prune if remaining letters cannot form this specific unsatisfied pattern
                        if !remaining_counts.can_subtract(&pattern_proc.counts) {
                            return; 
                        }
                    }
                }
                // If there are unsatisfied patterns but no letters left, or no more words allowed.
                if num_unsatisfied > 0 && remaining_counts.is_empty() { return; }
                if num_unsatisfied > 0 && constraints.max_words.is_some() && current_path.len() >= constraints.max_words.unwrap() {
                    return;
                }
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
                // Check constraints that apply to the full solution
                if let Some(max_w) = constraints.max_words { if current_path.len() > max_w { return; } }
                if let Some(required_starts) = &constraints.must_start_with { /* ... check ... */ }

                // ---> FINAL PATTERN CHECK FOR SOLUTION <---
                if let Some(satisfied_mask) = &internal_state.patterns_satisfied_mask {
                    if !satisfied_mask.iter().all(|&s| s) { // Check if all patterns are satisfied
                        return; // Not a valid solution if patterns aren't met
                    }
                }
                
                let mut solution_candidate = current_path.clone();
                solution_candidate.sort_unstable(); 
                if solutions_set.insert(solution_candidate) { 
                    internal_state.solutions_found_count += 1;
                    if let Some(max_sol) = constraints.max_solutions {
                        if internal_state.solutions_found_count >= max_sol { return; }
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
            internal_state,
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
        internal_state: &mut SolverInternalState,
    ) {
        // Limit checks
        if internal_state.timed_out { return; }
        if let Some(max_sol) = constraints.max_solutions {
            if internal_state.solutions_found_count >= max_sol { return; }
        }

        if current_trie_node.is_end_of_word && !word_so_far.is_empty() {
            let mut passes_min_length = true;
            if let Some(min_len) = constraints.min_word_length {
                if word_so_far.len() < min_len { passes_min_length = false; }
            }

            if passes_min_length {
                // ---> Pattern Satisfaction Logic for the current word_so_far <---
                let mut original_mask_states_for_changed_indices = Vec::new(); // Stores (index, original_bool_value)
                
                if let Some(patterns_to_satisfy) = &constraints.contains_patterns {
                    if let Some(current_mask) = internal_state.patterns_satisfied_mask.as_mut() {
                        for (idx, pattern_proc) in patterns_to_satisfy.iter().enumerate() {
                            if !current_mask[idx] && word_so_far.contains(&pattern_proc.text) {
                                original_mask_states_for_changed_indices.push((idx, current_mask[idx])); // Store original (false)
                                current_mask[idx] = true; // Set to true for deeper search
                            }
                        }
                    }
                }

                path.push(word_so_far.clone());
                self.backtrack(path, current_overall_counts, &self.trie.root, constraints, solutions_set, internal_state);
                path.pop();

                // Revert mask changes to their original state before this word was considered
                if let Some(current_mask) = internal_state.patterns_satisfied_mask.as_mut() {
                    for (idx, original_state) in original_mask_states_for_changed_indices {
                        current_mask[idx] = original_state;
                    }
                }
                
                // Check limits again
                if internal_state.timed_out { return; }
                if let Some(max_sol) = constraints.max_solutions {
                    if internal_state.solutions_found_count >= max_sol { return; }
                }
            }
        }
        
        if word_so_far.len() > self.trie.max_word_len || word_so_far.len() > current_overall_counts.total() { return; }

        for (key_ref_char_code, value_ref_next_node) in current_trie_node.children.iter() {
            let ch: char = *key_ref_char_code;
            if current_overall_counts.get(ch).unwrap_or(0) > 0 {
                if word_so_far.is_empty() && !constraints.is_valid_start_char(ch) { continue; }

                current_overall_counts.decrement_char(ch).unwrap();
                word_so_far.push(ch);

                self.find_one_word_recursive(
                    value_ref_next_node, word_so_far, current_overall_counts, path,
                    constraints, solutions_set, internal_state,
                );

                word_so_far.pop(); 
                current_overall_counts.increment_char(ch).unwrap();

                if internal_state.timed_out { return; }
                if let Some(max_sol) = constraints.max_solutions {
                    if internal_state.solutions_found_count >= max_sol { return; }
                }
            }
        }
    }
}