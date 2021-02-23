extern crate cairo;
extern crate rgb;
extern crate x11rb;

mod draw;
mod menu;
mod xorg;

pub use crate::menu::Menu;
use crate::xorg::XorgUserInterface;
use std::{fs, io};

pub trait UserInterface {
    fn run(&mut self, menu: &mut Menu) -> Result<(), Box<dyn std::error::Error>>;
}

fn main() {
    let input = fs::read_dir("/usr/bin")
        .unwrap()
        .map(|res| res.map(|e| String::from(e.file_name().to_str().unwrap())))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    let mut menu = Menu::new(input);
    let mut ui = XorgUserInterface::new().unwrap();
    ui.run(&mut menu);
}
