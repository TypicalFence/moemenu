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

// very simple engine based on String::contains
pub struct ContainsEngine;

impl ContainsEngine {
    pub fn new() -> Self {
        ContainsEngine {}
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
