use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use super::trie::{Trie, TrieNode};
use super::char_utils::{CharCounts, normalize_word, index_to_char, parse_char_list_to_set, parse_char_list_to_counts};

pub struct SolverConstraints {
    pub must_start_with: Option<HashMap<char, usize>>,
    pub can_only_ever_start_with: Option<HashSet<char>>,
    pub must_not_start_with: Option<HashSet<char>>,
    pub max_words: Option<usize>,
}

impl SolverConstraints {
    // Helper to check if a character is a valid start for a word
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
            Err(_) => return Vec::new(), // Invalid input phrase
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
            &self.trie.root, // Start word search from Trie root
            constraints,
            &mut solutions_set,
        );
        
        let mut final_solutions: Vec<Vec<String>> = solutions_set.into_iter().collect();

        // Sort solutions
        final_solutions.sort_by(|a, b| {
            // 1. By number of words (ascending)
            let len_cmp = a.len().cmp(&b.len());
            if len_cmp != Ordering::Equal {
                return len_cmp;
            }

            // 2. By length of the shortest word (descending)
            let min_len_a = a.iter().map(|w| w.len()).min().unwrap_or(0);
            let min_len_b = b.iter().map(|w| w.len()).min().unwrap_or(0);
            let min_len_cmp = min_len_b.cmp(&min_len_a); // Descending
            if min_len_cmp != Ordering::Equal {
                return min_len_cmp;
            }
            
            // 3. Lexicographically (using already sorted words from HashSet insertion)
            // The HashSet stores Vec<String> where inner Vec was sorted.
            // So, direct comparison of Vec<String> works.
            a.cmp(b)
        });

        final_solutions
    }

    fn backtrack(
        &self,
        current_path: &mut Vec<String>,
        remaining_counts: &mut CharCounts,
        // previous_word_start_node: &TrieNode, // For ordering words to avoid permutations like "a b" and "b a"
                                               // Simpler: sort path before inserting into HashSet
        start_node_for_this_level: &TrieNode, // To ensure lexicographical order of words in a solution path, if needed.
                                              // For now, we sort the full path Vec<String> before inserting into HashSet.
                                              // This means start_node_for_this_level is always trie.root.
        constraints: &SolverConstraints,
        solutions_set: &mut HashSet<Vec<String>>,
    ) {
        // Pruning: Max words
        if let Some(max_w) = constraints.max_words {
            if current_path.len() > max_w {
                return;
            }
        }

        // Base Case: All characters used up
        if remaining_counts.is_empty() {
            if !current_path.is_empty() { // Must have at least one word
                // Check max_words constraint again (solution might be formed with exactly max_words)
                if let Some(max_w) = constraints.max_words {
                    if current_path.len() > max_w {
                        return;
                    }
                }

                // Check 'must_start_with' constraint
                if let Some(required_starts) = &constraints.must_start_with {
                    let mut current_starts_counts: HashMap<char, usize> = HashMap::new();
                    for word in current_path.iter() {
                        if let Some(first_char) = word.chars().next() {
                            *current_starts_counts.entry(first_char).or_insert(0) += 1;
                        }
                    }
                    for (req_char, req_count) in required_starts {
                        if current_starts_counts.get(req_char).unwrap_or(&0) < req_count {
                            return; // Constraint not met
                        }
                    }
                }

                let mut solution_candidate = current_path.clone();
                solution_candidate.sort_unstable(); // Normalize for HashSet
                solutions_set.insert(solution_candidate);
            }
            return;
        }
        
        // Pruning: If remaining characters are fewer than the shortest word in dictionary
        if remaining_counts.total() < self.trie.get_min_word_len() {
            return;
        }
        // Pruning: if current_path.len() == max_words and remaining_counts not empty
        if let Some(max_w) = constraints.max_words {
            if current_path.len() == max_w && !remaining_counts.is_empty() {
                return;
            }
        }


        // Recursive step: Find one word
        let mut word_buffer = String::new();
        self.find_one_word_recursive(
            &self.trie.root, // Always start search for a new word from the trie root
            &mut word_buffer,
            remaining_counts,
            current_path,
            constraints,
            solutions_set,
        );
    }

    fn find_one_word_recursive(
        &self,
        current_trie_node: &TrieNode,
        word_so_far: &mut String,
        current_overall_counts: &mut CharCounts, // Counts for the whole anagram phrase
        path: &mut Vec<String>, // Current list of words in the anagram
        constraints: &SolverConstraints,
        solutions_set: &mut HashSet<Vec<String>>,
    ) {
        // If current_trie_node is a word
        if current_trie_node.is_end_of_word && !word_so_far.is_empty() {
            // This word is a candidate. Add it to path and backtrack for more words.
            path.push(word_so_far.clone());
            self.backtrack(path, current_overall_counts, &self.trie.root, constraints, solutions_set);
            path.pop(); // Backtrack: remove word from path
        }
        
        // Pruning: if word_so_far is already longer than any word in dict or any remaining letters
        if word_so_far.len() > self.trie.max_word_len || word_so_far.len() > current_overall_counts.total() {
            return;
        }


        // Explore children
        for (char_code, next_node) in ¤t_trie_node.children {
            let ch = *char_code; // This is already char, not index

            // Check if character is available
            if current_overall_counts.get(ch).unwrap_or(0) > 0 {
                // Apply start-of-word constraints if this is the first letter
                if word_so_far.is_empty() {
                    if !constraints.is_valid_start_char(ch) {
                        continue; // Skip this branch for this word
                    }
                    // Lexicographical ordering for words in a path (to avoid permutations if not sorting path later)
                    // If path is not empty, this new word must be >= last word in path
                    // This is complex if not just sorting path. For now, rely on sorting path.
                    // if !path.is_empty() && &ch.to_string() < path.last().unwrap().get(0..1).unwrap_or("") {
                    //    continue;
                    // }
                }

                current_overall_counts.0[super::char_utils::char_to_index(ch).unwrap()] -= 1;
                word_so_far.push(ch);

                self.find_one_word_recursive(
                    next_node,
                    word_so_far,
                    current_overall_counts,
                    path,
                    constraints,
                    solutions_set,
                );

                word_so_far.pop(); // Backtrack char
                current_overall_counts.0[super::char_utils::char_to_index(ch).unwrap()] += 1; // Backtrack count
            }
        }
    }
}