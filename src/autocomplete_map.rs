use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct AutocompleteSuggestion<T: Clone> {
    name: String,
    matches: Vec<Range<usize>>,
    value: T,
}

#[derive(Debug, PartialEq, Eq)]
struct CandidateSuggestion<'a> {
    name: &'a str,
    matches: Vec<Range<usize>>,
}

impl<'a> Ord for CandidateSuggestion<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("CandidateSuggestion::partial_cmp should never return None")
    }
}

impl<'a> PartialOrd for CandidateSuggestion<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // TODO: Actually take the matches into account.
        Some(self.name.cmp(other.name))
    }
}

fn get_matches(input: &str, name: &str) -> Vec<Range<usize>> {
    let mut matches = vec![];

    if input.len() == 0 {
        return matches;
    }

    let mut name_idx = 0;
    let mut curr_match: Option<Range<usize>> = None;
    let mut input_chars = input.chars();
    let mut input_char = input_chars.next().unwrap();

    for name_char in name.chars() {
        if name_char == input_char {
            curr_match = match curr_match {
                None => Some(name_idx..name_idx + 1),
                Some(range) => Some(range.start..name_idx + 1),
            };
            if let Some(next_input_char) = input_chars.next() {
                input_char = next_input_char;
            } else {
                break;
            }
        } else if let Some(match_) = curr_match {
            matches.push(match_);
            curr_match = None;
        }

        name_idx += 1;
    }

    if let Some(match_) = curr_match {
        matches.push(match_);
    }

    matches
}

fn get_best_candidates<'a, I: Iterator<Item = &'a String>>(
    input: &str,
    names: I,
    max_results: usize,
) -> Vec<CandidateSuggestion<'a>> {
    let mut candidates: Vec<CandidateSuggestion> = Vec::new();

    for name in names {
        let matches = get_matches(input, name.as_str());
        if !matches.is_empty() {
            candidates.push(CandidateSuggestion {
                name: name.as_str(),
                matches,
            });
        }
    }

    candidates.sort();
    candidates.truncate(max_results);
    candidates
}

pub struct AutocompleteMap<T: Clone> {
    entries: HashMap<String, T>,
}

impl<T: Clone> AutocompleteMap<T> {
    pub fn new() -> Self {
        AutocompleteMap {
            entries: HashMap::new(),
        }
    }

    pub fn insert<U: Into<String>>(&mut self, name: U, value: T) {
        self.entries.insert(name.into(), value);
    }

    pub fn autocomplete<U: AsRef<str>>(
        &self,
        input: U,
        max_results: usize,
    ) -> Vec<AutocompleteSuggestion<T>> {
        let mut results: Vec<AutocompleteSuggestion<T>> = Vec::with_capacity(max_results);
        let candidates = get_best_candidates(input.as_ref(), self.entries.keys(), max_results);

        for candidate in candidates.iter() {
            let name = String::from(candidate.name);
            let value = self
                .entries
                .get(&name)
                .expect("We literally just iterated past this")
                .clone();
            let matches = candidate.matches.clone();
            results.push(AutocompleteSuggestion {
                name,
                matches,
                value,
            })
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    fn strings<'a>(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| String::from(*name)).collect()
    }

    fn sugg(name: &str, matches: Vec<Range<usize>>, value: usize) -> AutocompleteSuggestion<usize> {
        AutocompleteSuggestion {
            name: String::from(name),
            matches,
            value,
        }
    }

    fn cand(name: &'static str, matches: Vec<Range<usize>>) -> CandidateSuggestion {
        CandidateSuggestion { name, matches }
    }

    #[test]
    fn test_autocomplete_map_works() {
        let mut am = AutocompleteMap::new();
        am.insert("boop", 1);
        am.insert("goop", 2);
        am.insert("boink", 3);
        assert_eq!(
            am.autocomplete("bo", 500),
            vec![sugg("boink", vec![0..2], 3), sugg("boop", vec![0..2], 1)]
        );
    }

    #[test]
    fn test_get_best_candidates_ignores_nonmatches() {
        assert_eq!(
            get_best_candidates("bo", strings(&["hi", "there"]).iter(), 500),
            vec![]
        );
    }

    #[test]
    fn test_get_best_candidates_returns_sorted_matches() {
        assert_eq!(
            get_best_candidates("bo", strings(&["boop", "boink"]).iter(), 500),
            vec![cand("boink", vec![0..2]), cand("boop", vec![0..2])]
        );
    }

    #[test]
    fn test_get_best_candidates_truncates_matches() {
        assert_eq!(
            get_best_candidates("bo", strings(&["boop", "boink"]).iter(), 1),
            vec![cand("boink", vec![0..2])]
        );
    }

    #[test]
    fn test_get_matches_returns_empty_vec() {
        assert_eq!(get_matches("boop", "goop"), vec![]);
        assert_eq!(get_matches("", "goop"), vec![]);
    }

    #[test]
    fn test_get_matches_returns_full_matches() {
        assert_eq!(get_matches("boop", "boop"), vec![0..4]);
    }

    #[test]
    fn test_get_matches_returns_multiple_matches() {
        assert_eq!(get_matches("boop", "bogus ops"), vec![0..2, 6..8]);
    }
}
