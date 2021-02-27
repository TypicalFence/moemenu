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
use crate::config::{Colors, Config, Position};
use rgb::RGB8;

const PINK: RGB8 = RGB8::new(247, 168, 184);
const BLACK: RGB8 = RGB8::new(0, 0, 0);
const WHITE: RGB8 = RGB8::new(255, 255, 255);

pub const DEFAULT_CONFIG: Config = Config {
    position: Position::Top,
    font_size: 13.0,
    height: 26,
    end_buffer: 20.0,
    item_spacing: 20.0,
    start_divisor: 6.0,
    colors: Colors {
        background: PINK,
        font: BLACK,
        selected_font: BLACK,
        selected_background: WHITE,
    },
};
