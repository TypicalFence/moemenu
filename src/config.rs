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
use rgb::{RGB8};

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
    pub end_buffer: f64,
    pub item_spacing: f64,
    pub start_divisor: f64,
}

impl Config {
    #[cfg(not(feature = "config"))]
    pub fn get() -> Self {
        crate::defaults::DEFAULT_CONFIG
    }

    #[cfg(feature = "config")]
    pub fn get() -> Self {
        config_feature::load()
    }
}

#[cfg(feature = "config")]
mod config_feature {
    use std::fs;

    use xdg::BaseDirectories;
    use toml::Value;
    use css_color_parser::Color as CssColor;
    use toml::value::Table;
    use rgb::{RGB8};

    use crate::defaults::{DEFAULT_CONFIG};
    use super::{Config, Colors, Position};

    pub fn load() -> Config {
        let xdg = BaseDirectories::new();
        if xdg.is_err() {
            return DEFAULT_CONFIG;
        }

        let file = xdg.unwrap().find_config_file("moemenu.toml");
        if file.is_none() {
            return DEFAULT_CONFIG;
        }

        let result= fs::read_to_string(file.unwrap());
        if result.is_err() {
            return DEFAULT_CONFIG;
        }

        let config_text = result.unwrap();

        let values = config_text.parse::<Value>();

        match values {
            Ok(values) => handle_toml(values),
            Err(_) => DEFAULT_CONFIG
        }
    }

    fn handle_toml(toml: Value) -> Config {
        let position = get_str(&toml, "position");
        let font_size = get_float(&toml, "font_size");
        let height = get_int(&toml, "height");
        let end_buffer = get_float(&toml, "end_buffer");
        let item_spacing = get_float(&toml, "item_spacing");
        let start_divisor = get_float(&toml, "start_divisor");
        let colors = toml.get("colors");

        Config {
            position: match position.unwrap_or("top".to_string()).as_str() {
                "top" => Position::Top,
                "bottom" => Position::Bottom,
                _ => Position::Bottom
            },
            font_size: font_size.unwrap_or(DEFAULT_CONFIG.font_size),
            height: height.unwrap_or(DEFAULT_CONFIG.height as i64) as u16,
            end_buffer: end_buffer.unwrap_or(DEFAULT_CONFIG.end_buffer),
            item_spacing: item_spacing.unwrap_or(DEFAULT_CONFIG.item_spacing),
            start_divisor: start_divisor.unwrap_or(DEFAULT_CONFIG.start_divisor),
            colors: handle_colors(colors).unwrap_or(DEFAULT_CONFIG.colors)
        }
    }

    fn handle_colors(toml: Option<&Value>) -> Option<Colors> {
        if toml.is_none() {
            return None;
        }

        let colors = toml.unwrap().as_table().unwrap();
        let background = get_color_str(&colors, "background");
        let font= get_color_str(colors, "font");
        let selected_background = get_color_str(colors, "selected_background");
        let selected_font = get_color_str(colors, "selected_font");

        Some(Colors {
            background: parse_color(background, DEFAULT_CONFIG.colors.background),
            font: parse_color(font, DEFAULT_CONFIG.colors.font),
            selected_font: parse_color(selected_font, DEFAULT_CONFIG.colors.selected_font),
            selected_background: parse_color(selected_background, DEFAULT_CONFIG.colors.selected_background)
        })
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
            Some(val) => {
                if val.as_integer().is_none() {
                    if let Some(float) = get_float(toml, key) {
                        return Some(float as i64);
                    }
                }
                val.as_integer()
            },
            None => None
        }
    }

    fn get_float(toml: &Value, key: &str) -> Option<f64> {
        match toml.get(key) {
            Some(val) => {
                if val.as_float().is_none() {
                    if let Some(int) = get_int(toml, key) {
                        return Some(int as f64);
                    }
                }
                val.as_float()
            },
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
}


