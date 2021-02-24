use crate::{Menu, Config};
use rgb::{RGB, RGB8};

pub fn set_color(cr: &cairo::Context, rgb: RGB8) {
    let convert = |x| 1.0 / 255.0 * (x as f64);
    cr.set_source_rgb(convert(rgb.r), convert(rgb.g), convert(rgb.b));
}

pub fn do_draw(cr: &cairo::Context, (width, height): (f64, f64), transparency: bool, config: &Config, menu: &Menu) {
    // draw bar
    set_color(cr, config.colors.background);
    cr.set_operator(cairo::Operator::Source);
    cr.paint();
    cr.set_operator(cairo::Operator::Over);

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
    let extents = cr.text_extents(menu.get_search_term());
    cr.move_to(10.0, extents.height + (height - extents.height) / 2.0);
    cr.show_text(menu.get_search_term());
}

