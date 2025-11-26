//! Consolidated fuzzy matching module using nucleo
//!
//! This module provides all fuzzy matching functionality for the application,
//! consolidating logic that was previously scattered across multiple files.

use crate::docs::Method;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Fuzzy matcher with optimized configuration
pub struct FuzzyMatcher {
    matcher: Matcher,
    buf: Vec<char>,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default configuration
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
            buf: Vec::new(),
        }
    }

    /// Match a query against a target string and return the score
    /// Higher scores indicate better matches
    pub fn fuzzy_match(&mut self, query: &str, target: &str) -> Option<u32> {
        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
        let haystack = Utf32Str::new(target, &mut self.buf);
        pattern.score(haystack, &mut self.matcher)
    }

    /// Check if a query fuzzy matches a target with a minimum score threshold
    pub fn matches(&mut self, query: &str, target: &str, min_score: u32) -> bool {
        self.fuzzy_match(query, target)
            .map(|score| score >= min_score)
            .unwrap_or(false)
    }

    /// Match methods and return them sorted by score (best matches first)
    /// Only returns methods with score >= min_score
    pub fn match_methods<'a>(
        &mut self,
        query: &str,
        methods: &'a [Method],
        min_score: u32,
    ) -> Vec<(u32, &'a Method)> {
        let mut scored: Vec<(u32, &Method)> = methods
            .iter()
            .filter_map(|method| {
                self.fuzzy_match(query, &method.name)
                    .filter(|&score| score >= min_score)
                    .map(|score| (score, method))
            })
            .collect();

        // Sort by score descending (best matches first)
        scored.sort_by(|(a, _), (b, _)| b.cmp(a));
        scored
    }

    /// Match method names and return them sorted by score
    pub fn match_method_names<'a>(
        &mut self,
        query: &str,
        methods: &'a [Method],
        min_score: u32,
    ) -> Vec<(u32, String)> {
        let mut scored: Vec<(u32, String)> = methods
            .iter()
            .filter_map(|method| {
                self.fuzzy_match(query, &method.name)
                    .filter(|&score| score >= min_score)
                    .map(|score| (score, method.name.clone()))
            })
            .collect();

        // Sort by score descending (best matches first)
        scored.sort_by(|(a, _), (b, _)| b.cmp(a));
        scored
    }

    /// Match methods and return them with signatures
    pub fn match_methods_with_signatures(
        &mut self,
        query: &str,
        methods: &[Method],
        min_score: u32,
        max_results: usize,
    ) -> Vec<(String, String)> {
        let scored = self.match_methods(query, methods, min_score);

        let mut results = Vec::new();
        for (_, method) in scored.iter().take(max_results) {
            // Add up to 2 overloads per method
            for overload in method.overloads.iter().take(2) {
                results.push((method.name.clone(), overload.signature.clone()));
            }
        }
        results
    }
}

/// Helper function for substring matching (faster than fuzzy for exact substrings)
pub fn substring_match<'a>(query: &str, methods: &'a [Method]) -> Vec<&'a Method> {
    let query_lower = query.to_lowercase();
    methods
        .iter()
        .filter(|m| m.name.to_lowercase().contains(&query_lower))
        .collect()
}

/// Check if query is a subsequence of target (all chars in order, but not necessarily consecutive)
/// Case-insensitive matching
pub fn subsequence_match(query: &str, target: &str) -> bool {
    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    let mut query_chars = query_lower.chars();
    let mut current_char = query_chars.next();

    for target_char in target_lower.chars() {
        if let Some(qc) = current_char {
            if qc == target_char {
                current_char = query_chars.next();
            }
        }
    }

    current_char.is_none()
}

/// Smart method matching that combines substring, subsequence, and fuzzy matching
/// Returns methods sorted by relevance
pub fn smart_match_methods<'a>(query: &str, methods: &'a [Method]) -> Vec<&'a Method> {
    let query_lower = query.to_lowercase().replace(' ', "");

    // First, try exact substring matches (fastest and most relevant)
    let substring_matches = substring_match(&query_lower, methods);
    if !substring_matches.is_empty() {
        return substring_matches;
    }

    // Then try subsequence matching combined with fuzzy matching
    let mut matcher = FuzzyMatcher::new();
    let mut scored: Vec<(u32, &Method)> = methods
        .iter()
        .filter_map(|method| {
            let name_lower = method.name.to_lowercase();

            // Check for subsequence match first
            if subsequence_match(&query_lower, &name_lower) {
                // Give it a bonus score for subsequence matching
                Some((u32::MAX / 2, method))
            } else {
                // Fall back to fuzzy matching with a reasonable threshold
                matcher
                    .fuzzy_match(&query_lower, &name_lower)
                    .filter(|&score| score >= 50)
                    .map(|score| (score, method))
            }
        })
        .collect();

    // Sort by score descending
    scored.sort_by(|(a, _), (b, _)| b.cmp(a));
    scored.into_iter().map(|(_, method)| method).collect()
}

/// Get method name suggestions for a given method name in a class
/// Used for error messages and "did you mean?" suggestions
pub fn get_method_suggestions(method_name: &str, class_methods: &[Method]) -> Vec<String> {
    let method_lower = method_name.to_lowercase();

    // First, look for exact case-insensitive matches
    let mut suggestions: Vec<String> = class_methods
        .iter()
        .filter(|m| m.name.to_lowercase() == method_lower && m.name != method_name)
        .map(|m| m.name.clone())
        .collect();

    // If no exact matches, use fuzzy matching
    if suggestions.is_empty() {
        let mut matcher = FuzzyMatcher::new();
        let scored = matcher.match_method_names(&method_lower, class_methods, 50);
        suggestions = scored.into_iter().map(|(_, name)| name).take(5).collect();
    }

    suggestions
}

/// Get method suggestions with signatures (for detailed error messages)
pub fn get_method_suggestions_with_signatures(
    method_name: &str,
    class_methods: &[Method],
    max_results: usize,
) -> Vec<(String, String)> {
    let method_lower = method_name.to_lowercase();

    // First, look for exact case-insensitive matches
    let matching_methods: Vec<_> = class_methods
        .iter()
        .filter(|m| m.name.to_lowercase() == method_lower && m.name != method_name)
        .collect();

    let mut suggestions: Vec<(String, String)> = Vec::new();

    if !matching_methods.is_empty() {
        // Add all overloads for exact matches
        for method in matching_methods {
            for overload in &method.overloads {
                suggestions.push((method.name.clone(), overload.signature.clone()));
            }
        }
    } else {
        // Use fuzzy matching
        let mut matcher = FuzzyMatcher::new();
        suggestions =
            matcher.match_methods_with_signatures(&method_lower, class_methods, 50, max_results);
    }

    suggestions
}

/// Fuzzy match class names and return matching (fqn, score) pairs
pub fn fuzzy_match_classes<'a>(
    query: &str,
    classes: impl Iterator<Item = &'a str>,
) -> Vec<(String, u32)> {
    let query_lower = query.to_lowercase();
    let mut matcher = FuzzyMatcher::new();

    let mut scored: Vec<(String, u32)> = classes
        .filter_map(|class_name| {
            matcher
                .fuzzy_match(&query_lower, &class_name.to_lowercase())
                .map(|score| (class_name.to_string(), score))
        })
        .collect();

    // Sort by score descending (best matches first)
    scored.sort_by(|(_, a), (_, b)| b.cmp(a));
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docs::Overload;

    fn create_test_method(name: &str) -> Method {
        Method {
            name: name.to_string(),
            overloads: vec![Overload {
                signature: format!("{}()", name),
                description: "Test method".to_string(),
                deprecated: false,
            }],
        }
    }

    #[test]
    fn test_fuzzy_match() {
        let mut matcher = FuzzyMatcher::new();

        // Should match similar strings
        assert!(matcher.fuzzy_match("test", "testing").is_some());
        assert!(matcher.fuzzy_match("str", "string").is_some());

        // Should not match completely different strings
        assert!(
            matcher.fuzzy_match("abc", "xyz").is_none()
                || matcher.fuzzy_match("abc", "xyz").unwrap() < 10
        );
    }

    #[test]
    fn test_substring_match() {
        let methods = vec![
            create_test_method("toString"),
            create_test_method("toUpperCase"),
            create_test_method("append"),
        ];

        let matches = substring_match("to", &methods);
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().any(|m| m.name == "toString"));
        assert!(matches.iter().any(|m| m.name == "toUpperCase"));
    }

    #[test]
    fn test_subsequence_match() {
        assert!(subsequence_match("tst", "testing"));
        assert!(subsequence_match("apc", "appendChild"));
        assert!(!subsequence_match("xyz", "testing"));
    }

    #[test]
    fn test_smart_match_methods() {
        let methods = vec![
            create_test_method("toString"),
            create_test_method("toUpperCase"),
            create_test_method("append"),
            create_test_method("appendLine"),
        ];

        // Substring match should take priority
        let matches = smart_match_methods("append", &methods);
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().any(|m| m.name == "append"));
        assert!(matches.iter().any(|m| m.name == "appendLine"));

        // Subsequence match
        let matches = smart_match_methods("apl", &methods);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_get_method_suggestions() {
        let methods = vec![
            create_test_method("toString"),
            create_test_method("toUpperCase"),
            create_test_method("append"),
        ];

        // Exact case-insensitive match
        let suggestions = get_method_suggestions("tostring", &methods);
        assert!(suggestions.contains(&"toString".to_string()));

        // Fuzzy match
        let suggestions = get_method_suggestions("apend", &methods);
        assert!(!suggestions.is_empty());
    }
}
