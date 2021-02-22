use crate::Menu;

pub fn do_draw(cr: &cairo::Context, (width, height): (f64, f64), transparency: bool, menu: &Menu) {
    use std::f64::consts::PI;

    cr.set_source_rgba(1.0, 0.0, 0.0, 0.3);
    cr.set_operator(cairo::Operator::Source);
    cr.paint();
    cr.set_operator(cairo::Operator::Over);

    cr.set_source_rgb(0.1, 0.1, 0.7);
    cr.move_to(10.0, 30.0);
    cr.set_font_size(30.0);
    let mut position: f64 = 100.0;
    let spacing = 50.0;
    for item in menu.get_items() {
        if position > width {
            break;
        }
        cr.move_to(position, height/2.0 + 5.0);
        let extents = cr.text_extents(item);
        cr.show_text(item);
        position += extents.width + spacing;
    }
}
