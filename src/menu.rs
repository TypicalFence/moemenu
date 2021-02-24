use crate::SearchEngine;

/// Manages the whole state of the program
pub struct Menu {
    input: Vec<String>,
    search_term: String,
    items: Vec<String>,
    selection: u32,
    engine: Box<dyn SearchEngine>,
}

impl Menu {
    pub fn new(engine: Box<dyn SearchEngine>, input: Vec<String>) -> Self {
        return Menu {
            search_term: String::from(""),
            input: input.clone(),
            items: input.clone(),
            selection: 0,
            engine
        };
    }

    pub fn search(&mut self, search_term: String) -> &Vec<String> {
        self.selection = 0;
        self.search_term = search_term;
        self.items = self.engine.search(&self.search_term, &self.input);
        return &self.items;
    }

    pub fn get_search_term(&self) -> &String {
        return &self.search_term;
    }

    pub fn get_items(&self) -> &Vec<String> {
        return &self.items;
    }

    pub fn get_selection(&self) -> u32 {
        return self.selection;
    }

    pub fn select_next_item(&mut self) {
        if self.selection < self.items.len() as u32 {
            self.selection += 1;
        }
    }

    pub fn select_previous_item(&mut self) {
        if self.selection > 0 {
            self.selection -= 1;
        }
    }

    pub fn get_selected_item(&self) -> Option<String> {
        match self.items.get(self.selection as usize) {
            Some(s) => Some(s.clone()),
            None => None
        }
    }
}

