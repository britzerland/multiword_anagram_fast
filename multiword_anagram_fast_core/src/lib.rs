use pyo3::prelude::*;
// PyList, PyString, PyDict, PySet were unused directly.
use std::collections::{HashMap, HashSet}; // These ARE needed for char_utils return types

mod char_utils;
mod trie;
mod solver;

use solver::{AnagramSolver as RustAnagramSolver, SolverConstraints as RustSolverConstraints};

#[pyclass(name = "Solver")]
struct PySolver {
    solver: RustAnagramSolver,
}

#[pymethods]
impl PySolver {
    #[new]
    fn new() -> Self {
        PySolver {
            solver: RustAnagramSolver::new(),
        }
    }

    fn load_dictionary_from_words(&mut self, words: Vec<String>) {
        self.solver.load_dictionary_from_words(&words);
    }

    fn load_dictionary_from_path(&mut self, path: String) -> PyResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read dictionary: {}", e)))?;
        self.solver.load_dictionary_from_text(&content);
        Ok(())
    }
    
    fn add_word(&mut self, word: String) {
        self.solver.add_word(&word);
    }

    #[pyo3(signature = (
        phrase,
        must_start_with=None,
        can_only_ever_start_with=None,
        must_not_start_with=None,
        max_words=None
    ))]
    fn solve(
        &self,
        phrase: String,
        must_start_with: Option<String>,
        can_only_ever_start_with: Option<String>,
        must_not_start_with: Option<String>,
        max_words: Option<usize>,
    ) -> PyResult<Vec<Vec<String>>> {
        // These parse functions return Option<HashMap/HashSet> so those types need to be in scope
        let rust_constraints = RustSolverConstraints {
            must_start_with: char_utils::parse_char_list_to_counts(must_start_with.as_deref()),
            can_only_ever_start_with: char_utils::parse_char_list_to_set(can_only_ever_start_with.as_deref()),
            must_not_start_with: char_utils::parse_char_list_to_set(must_not_start_with.as_deref()),
            max_words,
        };
        
        let solutions = self.solver.solve(&phrase, &rust_constraints);
        Ok(solutions)
    }
}

// Updated #[pymodule] signature for PyO3 >= 0.18 (approx)
#[pymodule]
fn multiword_anagram_fast_core(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySolver>()?;
    Ok(())
}