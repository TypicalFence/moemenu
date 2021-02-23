use std::fs;
use xdg::BaseDirectories;
use toml::Value;
use rgb::{RGB8};
use css_color_parser::Color as CssColor;
use toml::value::Table;

const PINK: RGB8 = RGB8::new(247, 168, 184);
const BLACK: RGB8 = RGB8::new(0, 0, 0);
const WHITE: RGB8 = RGB8::new(255, 255, 255);

#[derive(Debug, Copy, Clone)]
pub enum Position {
    Top,
    Bottom
}

pub struct Colors {
    pub background: RGB8,
    pub font: RGB8,
    pub selected_font: RGB8,
    pub selected_background: RGB8,
}

pub struct Config {
    pub position: Position,
    pub font_size: f64,
    pub height: u16,
    pub colors: Colors,
}

impl Config {
    pub fn default() -> Self {
        Config {
            position: Position::Top,
            font_size: 30.0,
            height: 50,
            colors: Colors {
                background: PINK,
                font: BLACK,
                selected_font: PINK,
                selected_background: WHITE,
            }
        }
    }

    pub fn load() -> Self {
        let default = Self::default();
        let xdg = BaseDirectories::new();
        if xdg.is_err() {
            return default;
        }

        let file = xdg.unwrap().find_config_file("moemenu.toml");
        if file.is_none() {
            return default;
        }

        let result= fs::read_to_string(file.unwrap());
        if result.is_err() {
            return default;
        }

        let config_text = result.unwrap();

        let values = config_text.parse::<Value>();

        match values {
            Ok(values) => Self::handle_toml(values),
            Err(_) => default
        }
    }

    fn handle_toml(toml: Value) -> Self {
        let default= Self::default();
        let position = get_str(&toml, "position");
        let font_size = get_float(&toml, "fon_size");
        let height = get_int(&toml, "height");
        let colors = toml.get("colors");

        Config {
            position: match position.unwrap_or("top".to_string()).as_str() {
                "top" => Position::Top,
                "bottom" => Position::Bottom,
                _ => Position::Bottom
            },
            font_size: font_size.unwrap_or(default.font_size),
            height: height.unwrap_or(default.height as i64) as u16,
            colors: Self::handle_colors(colors).unwrap_or(default.colors)
        }
    }

    fn handle_colors(toml: Option<&Value>) -> Option<Colors> {
        if toml.is_none() {
            return None;
        }

        let default= Self::default();
        let colors = toml.unwrap().as_table().unwrap();
        let background = get_color_str(&colors, "background");
        let font= get_color_str(colors, "font");
        let selected_background = get_color_str(colors, "selected_background");
        let selected_font = get_color_str(colors, "selected_font");

        Some(Colors {
            background: parse_color(background, default.colors.background),
            font: parse_color(font, default.colors.font),
            selected_font: parse_color(selected_font, default.colors.selected_font),
            selected_background: parse_color(selected_background, default.colors.selected_background)
        })
    }
}

// pile of util functions, LETS GO!
fn parse_color(maybe: Option<String>, fallback: RGB8) -> RGB8 {
    if maybe.is_none() {
        return fallback;
    }
    let color = maybe.unwrap().parse::<CssColor>();
    match color {
        Ok(c) => RGB8::new(c.r, c.g, c.b),
        Err(_) => fallback
    }
}

fn get_str(toml: &Value, key: &str) -> Option<String> {
    match toml.clone().get(key) {
        Some(val) => match val.as_str() {
            Some(s) => Some(s.to_string()),
            None => None,
        },
        None => None
    }
}

fn get_int(toml: &Value, key: &str) -> Option<i64> {
    match toml.get(key) {
        Some(val) => val.as_integer(),
        None => None
    }
}

fn get_float(toml: &Value, key: &str) -> Option<f64> {
    match toml.get(key) {
        Some(val) => val.as_float(),
        None => None
    }
}

fn get_color_str(toml: &Table, key: &str) -> Option<String> {
    match toml.clone().get(key) {
        Some(val) => match val.as_str() {
            Some(s) => Some(s.to_string()),
            None => None,
        },
        None => None
    }
}
