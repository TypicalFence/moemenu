/*
 * This file is part of moemenu.
 * Copyright (C) 2021 fence.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use crate::SearchEngine;

/// Manages the whole state of the program
pub struct Menu {
    input: Vec<String>,
    search_term: String,
    items: Vec<String>,
    selection: usize,
    shift: usize,
    engine: Box<dyn SearchEngine>,
}

impl Menu {
    pub fn new(engine: Box<dyn SearchEngine>, input: Vec<String>) -> Self {
        return Menu {
            search_term: String::from(""),
            input: input.clone(),
            items: input.clone(),
            selection: 0,
            shift: 0,
            engine
        };
    }

    pub fn search(&mut self, search_term: String) -> &Vec<String> {
        self.selection = 0;
        self.shift = 0;
        self.search_term = search_term;
        self.items = self.engine.search(&self.search_term, &self.input);
        return &self.items;
    }

    pub fn get_search_term(&self) -> String {
        return self.search_term.clone();
    }

    pub fn get_items(&self) -> &Vec<String> {
        return &self.items;
    }

    pub fn get_selection(&self) -> usize {
        return self.selection;
    }

    pub fn select_next_item(&mut self) {
        if self.selection < self.items.len() - 1 {
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

    pub fn get_shift(&self) -> usize {
        return self.shift;
    }

    pub fn update_page(&mut self, last_first_item: usize, current_last_item: usize) -> bool {
        if self.selection > current_last_item {
            self.shift = current_last_item as usize + 1;
            return true;
        }

        // selection smaller than the current start
        if self.selection < self.shift {
            self.shift = last_first_item as usize;
            return true;
        }

        false
    }
}

