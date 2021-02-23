use rgb::{RGB8};

const PINK: RGB8 = RGB8::new(247, 168, 184);
const BLACK: RGB8 = RGB8::new(0, 0, 0);
const WHITE: RGB8 = RGB8::new(255, 255, 255);

pub enum Position {
    Top,
    Bottom
}

pub struct Colors {
    pub background: RGB8,
    pub font: RGB8,
    pub font_selected: RGB8,
    pub font_selected_background: RGB8,
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
                font_selected: PINK,
                font_selected_background: WHITE,
            }
        }
    }

    pub fn load() -> Self {
        let default = Self::default();
        return default;
    }
}
