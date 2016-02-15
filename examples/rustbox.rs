extern crate rustbox;

use std::default::Default;

use rustbox::{Color, Style, RustBox};
use rustbox::Key;

struct Rectangle {
    x_min: usize,
    y_min: usize,
    x_max: usize,
    y_max: usize,
}

impl Rectangle {
    fn new(a: usize, b: usize, c: usize, d: usize) -> Rectangle {
        Rectangle {
            x_min: a,
            y_min: b,
            x_max: c,
            y_max: d,
        }
    }
}

fn draw_border(rb: &RustBox, rect: Rectangle, sty: Style, fg: Color, bg: Color) {
    let horiz = '-';
    let vert = '|';
    let corner = '+';

    for x in rect.x_min..(rect.x_max+1) {
        rb.print_char(x, rect.y_min, sty, fg, bg, horiz);
        rb.print_char(x, rect.y_max, sty, fg, bg, horiz);
    }
    for y in rect.y_min..(rect.y_max+1) {
        rb.print_char(rect.x_min, y, sty, fg, bg, vert);
        rb.print_char(rect.x_max, y, sty, fg, bg, vert);
    }

    rb.print_char(rect.x_min, rect.y_min, sty, fg, bg, corner);
    rb.print_char(rect.x_max, rect.y_min, sty, fg, bg, corner);
    rb.print_char(rect.x_max, rect.y_max, sty, fg, bg, corner);
    rb.print_char(rect.x_min, rect.y_max, sty, fg, bg, corner);
}

fn main() {
    let rustbox = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    rustbox.print(1, 1, rustbox::RB_BOLD, Color::White, Color::Black, "Hello, world!");
    rustbox.print(1, 3, rustbox::RB_BOLD, Color::White, Color::Black,
                  "Press 'q' to quit.");

    draw_border(&rustbox,
                Rectangle::new(0, 0, rustbox.width()-1, rustbox.height()-1),
                rustbox::RB_NORMAL, Color::White, Color::Blue);

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
