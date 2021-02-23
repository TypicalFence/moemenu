/// Manages the whole state of the program
pub struct Menu {
    input: Vec<String>,
    search_term: String,
    items: Vec<String>,
    selection: u32,
}

impl Menu {
    pub fn new(input: Vec<String>) -> Self {
        return Menu {
            search_term: String::from(""),
            input: input.clone(),
            items: input.clone(),
            selection: 0,
        };
    }

    pub fn search(&mut self, search_term: String) -> &Vec<String> {
        self.selection = 0;
        self.search_term = search_term;
        self.items = self
            .input
            .clone()
            .into_iter()
            .filter(|x| x.starts_with(&self.search_term))
            .collect();
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
}
