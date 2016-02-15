extern crate rustbox;

use std::default::Default;

use rustbox::{Color, RustBox};
use rustbox::Key;

struct Rectangle {
    x_min: usize,
    y_min: usize,
    x_max: usize,
    y_max: usize,
}

impl Rectangle {
    fn new(x_min: usize, y_min: usize, x_max: usize, y_max: usize) -> Rectangle {
        Rectangle {
            x_min: x_min,
            y_min: y_min,
            x_max: x_max,
            y_max: y_max,
        }
    }

    fn draw_border(&self, rb: &RustBox, color: Color) {
        let sty = rustbox::RB_NORMAL;
        let fg = Color::Default;
        let chr = ' ';

        for x in self.x_min..(self.x_max+1) {
            rb.print_char(x, self.y_min, sty, fg, color, chr);
            rb.print_char(x, self.y_max, sty, fg, color, chr);
        }
        for y in self.y_min..(self.y_max+1) {
            rb.print_char(self.x_min, y, sty, fg, color, chr);
            rb.print_char(self.x_max, y, sty, fg, color, chr);
        }

        rb.print_char(self.x_min, self.y_min, sty, fg, color, chr);
        rb.print_char(self.x_max, self.y_min, sty, fg, color, chr);
        rb.print_char(self.x_max, self.y_max, sty, fg, color, chr);
        rb.print_char(self.x_min, self.y_max, sty, fg, color, chr);
    }

    fn inside(&self) -> Rectangle {
        Rectangle::new(self.x_min+1, self.y_min+1, self.x_max-1, self.y_max-1)
    }

    fn print_header(&self, rb: &RustBox, fg: Color, bg: Color, s: &str) {
        let s = format!(" {} ", s.trim());
        let x = {
            let width = self.x_max - self.x_min + 1;
            let middle = width / 2;
            middle - s.len() / 2
        };
        rb.print(x, self.y_min, rustbox::RB_BOLD, fg, bg, &s);
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

    let outer_border = Rectangle::new(0, 0, rustbox.width()-1, rustbox.height()-1);
    outer_border.draw_border(&rustbox, Color::Blue);
    outer_border.print_header(&rustbox, Color::White, Color::Black, "Hello World");

    let x1 = outer_border.inside();
    x1.draw_border(&rustbox, Color::Red);
    x1.print_header(&rustbox, Color::White, Color::Blue, "Hello World");

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
