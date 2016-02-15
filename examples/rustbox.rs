extern crate rustbox;

use std::default::Default;

use rustbox::{Color, Style, RustBox};
use rustbox::Key;

fn draw_border(rb: &RustBox, sty: Style, fg: Color, bg: Color) {
    let horiz = '-';
    let vert = '|';
    for x in 0..rb.width() {
        rb.print_char(x, 0, sty, fg, bg, horiz);
        rb.print_char(x, rb.height()-1, sty, fg, bg, horiz);
    }
    for y in 0..rb.height() {
        rb.print_char(0, y, sty, fg, bg, vert);
        rb.print_char(rb.width()-1, y, sty, fg, bg, vert);
    }
}

fn main() {
    let rustbox = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    rustbox.print(1, 1, rustbox::RB_BOLD, Color::White, Color::Black, "Hello, world!");
    rustbox.print(1, 3, rustbox::RB_BOLD, Color::White, Color::Black,
                  "Press 'q' to quit.");

    draw_border(&rustbox, rustbox::RB_NORMAL, Color::White, Color::Blue);

    loop {
        rustbox.present();
        match rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char('q') => { break; }
                    _ => { }
                }
            },
            Err(e) => panic!("{}", e),
            _ => { }
        }
    }
}
