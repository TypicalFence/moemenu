use crate::Menu;
use rgb::{RGB, RGB8};

const pink: RGB8 = RGB::new(247, 168, 184);
const black: RGB8 = RGB::new(0, 0, 0);

fn set_color(cr: &cairo::Context, rgb: RGB8) {
    let convert = |x| 1.0 / 255.0 * (x as f64);
    cr.set_source_rgb(convert(rgb.r), convert(rgb.g), convert(rgb.b));
}

pub fn do_draw(cr: &cairo::Context, (width, height): (f64, f64), transparency: bool, menu: &Menu) {
    // draw bar
    set_color(cr, pink);
    cr.set_operator(cairo::Operator::Source);
    cr.paint();
    cr.set_operator(cairo::Operator::Over);

    // print items
    set_color(cr, black);
    cr.set_font_size(30.0);

    let mut position: f64 = width / 4.0;
    let spacing = 50.0;
    for item in menu.get_items() {
        if position > width {
            break;
        }
        let extents = cr.text_extents(item);
        cr.move_to(position, extents.height + (height - extents.height) / 2.0);
        cr.show_text(item);
        position += extents.width + spacing;
    }

    // print search_term
    let extents = cr.text_extents(menu.get_search_term());
    cr.move_to(10.0, extents.height + (height - extents.height) / 2.0);
    cr.show_text(menu.get_search_term());
}
