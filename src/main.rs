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
mod config;
mod defaults;
mod draw;
mod menu;
mod search;
mod xorg;

use std::io;
use std::io::BufRead;
use std::process::exit;

pub use crate::config::Config;
pub use crate::menu::Menu;
use crate::search::ContainsEngine;
use crate::xorg::XorgUserInterface;

pub trait SearchEngine {
    fn search(&mut self, needle: &String, haystack: &Vec<String>) -> Vec<String>;
}

pub trait UserInterface {
    fn run(&mut self, menu: &mut Menu) -> Result<(String, bool), Box<dyn std::error::Error>>;
}

fn read_stdin() -> Vec<String> {
    let stdin = io::stdin();
    let mut input = Vec::new();
    for line in stdin.lock().lines() {
        if line.is_ok() {
            input.push(line.unwrap())
        }
    }
    return input;
}

fn run_ui(ui: &mut dyn UserInterface, menu: &mut Menu) {
    match ui.run(menu) {
        Ok((selection, should_continue)) => {
            println!("{}", selection);
            if should_continue {
                run_ui(ui, menu);
            }
            exit(0);
        }
        Err(_) => {
            exit(1);
        }
    }
}

fn main() {
    let config = Config::get();
    let input = read_stdin();
    let mut menu = Menu::new(Box::from(ContainsEngine::new()), input);
    let mut ui = XorgUserInterface::new(config).unwrap();
    run_ui(&mut ui, &mut menu);
}
