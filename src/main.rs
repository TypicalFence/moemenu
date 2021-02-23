extern crate cairo;
extern crate rgb;
extern crate x11rb;

mod draw;
mod menu;
mod xorg;
mod config;

use std::io;
use std::io::{BufRead};
use std::process::exit;

pub use crate::menu::Menu;
pub use crate::config::Config;
use crate::xorg::XorgUserInterface;

pub trait UserInterface {
    fn run(&mut self, menu: &mut Menu) -> Result<String, Box<dyn std::error::Error>>;
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


fn main() {
    let config = Config::load();
    let input= read_stdin();
    let mut menu = Menu::new(input);
    let mut ui = XorgUserInterface::new(config).unwrap();
    match ui.run(&mut menu) {
        Ok(selection) => {
            println!("{}", selection);
            exit(1);
        }
        Err(_) => {
            exit(1);
        }
    }
}
