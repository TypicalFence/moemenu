extern crate x11rb;
extern crate cairo;

mod menu;
mod xorg;
mod draw;

use std::{fs, io};
pub use crate::menu::Menu;
use crate::xorg::XorgUserInterface;

pub trait UserInterface {
	fn run(&mut self, menu: &mut Menu) -> Result<(), Box<dyn std::error::Error>>;
}


fn main()  {
	let input = fs::read_dir("/usr/bin").unwrap()
		.map(|res| res.map(|e| String::from(e.file_name().to_str().unwrap())))
		.collect::<Result<Vec<_>, io::Error>>().unwrap();

	let mut menu = Menu::new(input);
	let mut ui = XorgUserInterface::new().unwrap();
	ui.run(&mut menu);
}
