use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

/// Context for query evaluation that can cache expensive computations
/// like compiled regexes across multiple row evaluations
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Cache of compiled regexes for LIKE patterns
    /// Key is the SQL LIKE pattern, value is the compiled regex
    regex_cache: HashMap<String, Arc<Regex>>,

    /// Case insensitive mode
    case_insensitive: bool,

    /// Statistics for debugging
    regex_compilations: usize,
    regex_cache_hits: usize,
}

impl EvaluationContext {
    pub fn new(case_insensitive: bool) -> Self {
        Self {
            regex_cache: HashMap::new(),
            case_insensitive,
            regex_compilations: 0,
            regex_cache_hits: 0,
        }
    }

    /// Get or compile a regex for a SQL LIKE pattern
    pub fn get_or_compile_like_regex(&mut self, like_pattern: &str) -> Result<Arc<Regex>, String> {
        // Check cache first
        if let Some(regex) = self.regex_cache.get(like_pattern) {
            self.regex_cache_hits += 1;
            return Ok(Arc::clone(regex));
        }

        // Convert SQL LIKE pattern to regex pattern
        let regex_pattern = like_pattern.replace('%', ".*").replace('_', ".");

        // Compile the regex
        let regex = regex::RegexBuilder::new(&format!("^{}$", regex_pattern))
            .case_insensitive(self.case_insensitive)
            .build()
            .map_err(|e| format!("Invalid LIKE pattern '{}': {}", like_pattern, e))?;

        let regex_arc = Arc::new(regex);
        self.regex_cache
            .insert(like_pattern.to_string(), Arc::clone(&regex_arc));
        self.regex_compilations += 1;

        Ok(regex_arc)
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.regex_compilations, self.regex_cache_hits)
    }

    pub fn is_case_insensitive(&self) -> bool {
        self.case_insensitive
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new(false)
    }
}
