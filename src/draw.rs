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
use crate::{Menu, Config};
use rgb::{RGB8};

pub fn set_color(cr: &cairo::Context, rgb: RGB8) {
    let convert = |x| 1.0 / 255.0 * (x as f64);
    cr.set_source_rgb(convert(rgb.r), convert(rgb.g), convert(rgb.b));
}

// TODO put in config
const END_BUFFER: f64 = 100.0;
const SPACING: f64 = 50.0;
const START_DIVISOR: f64 = 6.0;

pub fn find_last_item_that_fits(cr: &cairo::Context, width: f64, start: usize, items: &Vec<String>) -> usize {
    let mut position: f64 = width / START_DIVISOR;


    for (i, item) in items.clone().iter().enumerate() {
        // is the item to the left of the starting point?
        if i < start {
            continue;
        }

        let text_extents = cr.text_extents(item);
        let next_width = match items.get(i + 1) {
            Some(word) => cr.text_extents(word).width,
            None => 0.0
        };
        let next_is_off_screen = position + text_extents.width + next_width + 2.0 * SPACING > width - END_BUFFER;

        if next_is_off_screen {
            return i;
        }


        position += text_extents.width + SPACING;
    }

    items.len()
}

pub fn find_first_item_that_fits(cr: &cairo::Context, width: f64, end: usize, items: &Vec<String>) -> usize {
    let start: f64 = width / START_DIVISOR;
    let mut position: f64 = width - END_BUFFER;
    let len = items.len();

    for i in (0..len).rev() {
        // is the item to the left of the starting point?
        if i > end {
            continue;
        }

        let item = items.get(i).unwrap();
        let text_extents = cr.text_extents(item);
        let previous_width = match items.get((i + len - 1) % len) {
            Some(word) => cr.text_extents(word).width,
            None => 0.0
        };
        let prev_is_off_screen = position - text_extents.width - previous_width - 2.0 * SPACING < start;

        if prev_is_off_screen {
            return i;
        }

        position -= text_extents.width - SPACING;
    }

    0
}

pub fn do_draw(cr: &cairo::Context, (width, height): (f64, f64), transparency: bool, config: &Config, menu: &Menu) {
    // draw bar
    if transparency {
        cr.set_operator(cairo::Operator::Source);
    }
    set_color(cr, config.colors.background);
    cr.paint();

    if transparency {
        cr.set_operator(cairo::Operator::Over);
    }

    // print items
    set_color(cr, config.colors.font);
    cr.set_font_size(config.font_size);
    let font_extents = cr.font_extents();

    let start: f64 = width / START_DIVISOR;
    let mut position: f64 = start;
    let spacing = SPACING;
    let current_selection = menu.get_selection();

    // has previous page
    if menu.get_shift() > 0 {
        let prev_page_indicator = "<";
        let ppi_extents = cr.text_extents(prev_page_indicator);
        cr.move_to(start - ppi_extents.width - spacing, ppi_extents.height + (height - ppi_extents.height) / 2.0);
        cr.show_text(prev_page_indicator);
    }

    let items = menu.get_items();
    let mut has_next_page = false;
    for (i, item ) in items.clone().iter().enumerate() {
        // don't draw elements that have been scrolled away
        if i < menu.get_shift() as usize {
            continue;
        }

        let text_extents = cr.text_extents(item);

        // draw background for selected item
        if i == current_selection as usize {
            set_color(cr, config.colors.selected_background);
            cr.move_to(position, 0.0);
            cr.rectangle(position - spacing/2.0, 0.0, text_extents.width + spacing, height);
            cr.fill();
        }

        if i == current_selection as usize {
            set_color(cr, config.colors.selected_font);
        } else {
            set_color(cr, config.colors.font);
        }

        let next_width = match items.get(i + 1) {
            Some(word) => cr.text_extents(word).width,
            None => 0.0
        };
        let next_is_off_screen = position + text_extents.width + next_width + 2.0 * spacing > width - END_BUFFER;

        let y_pos = height/2.0 + config.font_size/2.0 - font_extents.descent * 0.7; // 0.7 for good measure
        cr.move_to(position, y_pos);
        cr.show_text(item);

        if next_is_off_screen {
            has_next_page = true;
            break;
        }

        position += text_extents.width + spacing;
    }

    if has_next_page {
        let next_page_indicator = ">";
        let npi_extents = cr.text_extents(next_page_indicator);
        cr.move_to(width - END_BUFFER, npi_extents.height + (height - npi_extents.height) / 2.0);
        cr.show_text(next_page_indicator);
    }

    // print search_term
    let term = menu.get_search_term();
    let term_extents = cr.text_extents(&term);
    cr.move_to(10.0, term_extents.height + (height - term_extents.height) / 2.0);
    cr.show_text(&term);
}

