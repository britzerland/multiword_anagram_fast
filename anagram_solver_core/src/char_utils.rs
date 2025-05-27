use std::collections::HashMap;

pub const ALPHABET_SIZE: usize = 26;

#[inline]
fn char_to_index(c: char) -> Option<usize> {
    if c.is_ascii_lowercase() {
        Some((c as u8 - b'a') as usize)
    } else {
        None
    }
}

#[inline]
pub fn index_to_char(i: usize) -> char {
    (b'a' + i as u8) as char
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharCounts([usize; ALPHABET_SIZE]);

impl CharCounts {
    pub fn new() -> Self {
        CharCounts([0; ALPHABET_SIZE])
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        let mut counts = [0; ALPHABET_SIZE];
        let mut total_chars = 0;
        for c in s.chars() {
            if c.is_alphabetic() {
                let lower_c = c.to_ascii_lowercase();
                if let Some(idx) = char_to_index(lower_c) {
                    counts[idx] += 1;
                    total_chars += 1;
                } else {
                    return Err(format!("Non-alphabetic ASCII character found: {}", c));
                }
            } else if !c.is_whitespace() { 
                // Optionally ignore non-alphabetic, non-whitespace or error out
                // For now, strict: only letters and whitespace allowed in input phrase for anagrams
                // return Err(format!("Invalid character in input string: {}", c));
            }
        }
        Ok(CharCounts(counts))
    }
    
    pub fn total(&self) -> usize {
        self.0.iter().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&count| count == 0)
    }

    pub fn get(&self, c: char) -> Option<usize> {
        char_to_index(c).map(|idx| self.0[idx])
    }

    pub fn can_subtract(&self, other: &Self) -> bool {
        for i in 0..ALPHABET_SIZE {
            if self.0[i] < other.0[i] {
                return false;
            }
        }
        true
    }

    pub fn subtract_mut(&mut self, other: &Self) -> Result<(), String> {
        if !self.can_subtract(other) {
            return Err("Cannot subtract, insufficient characters.".to_string());
        }
        for i in 0..ALPHABET_SIZE {
            self.0[i] -= other.0[i];
        }
        Ok(())
    }
    
    pub fn add_mut(&mut self, other: &Self) {
         for i in 0..ALPHABET_SIZE {
            self.0[i] += other.0[i];
        }
    }
}

pub fn normalize_word(word: &str) -> String {
    word.trim().to_ascii_lowercase().chars().filter(|c| c.is_ascii_alphabetic()).collect()
}

pub fn parse_char_list_to_set(s: Option<&str>) -> Option<std::collections::HashSet<char>> {
    s.map(|st| st.to_ascii_lowercase().chars().collect())
}

pub fn parse_char_list_to_counts(s: Option<&str>) -> Option<HashMap<char, usize>> {
    s.map(|st| {
        let mut counts = HashMap::new();
        for char_code in st.to_ascii_lowercase().chars() {
            *counts.entry(char_code).or_insert(0) += 1;
        }
        counts
    })
}