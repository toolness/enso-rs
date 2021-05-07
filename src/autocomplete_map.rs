use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct AutocompleteSuggestion<T: Clone> {
    name: String,
    matches: Vec<Range<usize>>,
    value: T,
}

fn get_matches(input: &str, name: &str) -> Vec<Range<usize>> {
    let mut matches = vec![];

    if name.len() == 0 {
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

    pub fn autocomplete<U: Into<String>>(
        &self,
        input: U,
        max_results: usize,
    ) -> Vec<AutocompleteSuggestion<T>> {
        let mut results = Vec::with_capacity(max_results);
        let input_string = input.into();

        for (name, value) in self.entries.iter() {
            let matches = get_matches(input_string.as_str(), name.as_str());
            if !matches.is_empty() {
                results.push(AutocompleteSuggestion {
                    name: name.clone(),
                    matches,
                    value: value.clone(),
                })
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::{AutocompleteMap, AutocompleteSuggestion};
    use std::ops::Range;

    fn sugg(name: &str, matches: Vec<Range<usize>>, value: usize) -> AutocompleteSuggestion<usize> {
        AutocompleteSuggestion {
            name: String::from(name),
            matches,
            value,
        }
    }

    #[test]
    fn test_it_works() {
        let mut am = AutocompleteMap::new();
        am.insert("boop", 1);
        am.insert("goop", 1);
        assert_eq!(am.autocomplete("bo", 1), vec![sugg("boop", vec![0..2], 1)]);
    }
}
