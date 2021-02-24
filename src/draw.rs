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

    let mut position: f64 = width / 6.0;
    let spacing = 50.0;
    let current_selection = menu.get_selection();
    for (i, item )in menu.get_items().iter().enumerate() {
        if position > width {
            break;
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

        let y_pos = height/2.0 + config.font_size/2.0 - font_extents.descent * 0.7; // 0.7 for good measure
        cr.move_to(position, y_pos);
        cr.show_text(item);
        position += text_extents.width + spacing;
    }

    // print search_term
    let term = menu.get_search_term();
    let extents = cr.text_extents(&term);
    cr.move_to(10.0, extents.height + (height - extents.height) / 2.0);
    cr.show_text(&term);
}

