use crate::SearchEngine;

// very simple engine based on String::contains
pub struct ContainsEngine;

impl ContainsEngine {
    pub fn new() -> Self {
        ContainsEngine{}
    }
}

impl SearchEngine for ContainsEngine {
    fn search(&mut self, needle: &String, haystack: &Vec<String>) -> Vec<String> {
        haystack
            .clone()
            .into_iter()
            .filter(|x| x.contains(needle))
            .collect()
    }
}
