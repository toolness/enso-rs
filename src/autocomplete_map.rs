use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct AutocompleteSuggestion<T: Clone> {
    name: String,
    matches: Vec<Range<usize>>,
    value: T,
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

        // TODO: Actually filter to input.
        for (name, value) in self.entries.iter() {
            results.push(AutocompleteSuggestion {
                name: name.clone(),
                matches: vec![0..2],
                value: value.clone(),
            });
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
        assert_eq!(am.autocomplete("bo", 1), vec![sugg("boop", vec![0..2], 1)]);
    }
}
