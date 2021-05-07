use std::collections::HashMap;

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

    pub fn autocomplete<U: Into<String>>(&self, input: U, max_results: usize) -> Vec<(String, T)> {
        let mut results: Vec<(String, T)> = Vec::with_capacity(max_results);

        // TODO: Actually filter to input.
        for (key, value) in self.entries.iter() {
            results.push((key.clone(), value.clone()));
        }

        results
    }
}

#[test]
fn test_it_works() {
    let mut am = AutocompleteMap::new();
    am.insert("boop", 1);
    assert_eq!(am.autocomplete("bo", 1), vec![(String::from("boop"), 1)]);
}
