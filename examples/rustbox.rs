extern crate rustbox;

use std::default::Default;

use rustbox::{Color, RustBox};
use rustbox::Key;

struct Rectangle<'a> {
    x_min: usize,
    y_min: usize,
    x_max: usize,
    y_max: usize,
    rb: &'a RustBox
}

impl<'a> Rectangle<'a> {
    fn new(x_min: usize, y_min: usize, x_max: usize, y_max: usize, rb: &'a RustBox) -> Rectangle {
        Rectangle {
            x_min: x_min,
            y_min: y_min,
            x_max: x_max,
            y_max: y_max,
            rb: rb,
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
        Rectangle::new(self.x_min+1, self.y_min+1, self.x_max-1, self.y_max-1, self.rb)
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

// TODO: Move toward Drawable model. Make specialized Drawable structs
// as necessary. Make a collection of Drawables Drawable.

// TODO: Support a notion of a rectangular drawable region, that can
// own Drawables. The owned Drawables can only draw within the
// region's borders.

trait Drawable<'a> {
    fn rustbox(&self) -> &'a RustBox;
    fn draw(&self);
}

impl<'a> Drawable<'a> for Rectangle<'a> {
    fn rustbox(&self) -> &'a RustBox {
        self.rb
    }

    fn draw(&self) {
        self.draw_border(self.rustbox(), Color::Red);
    }
}

fn main() {
    let rb = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    rb.print(3, 3, rustbox::RB_BOLD, Color::White, Color::Black,
             "Press 'q' to quit.");

    let outer_border = Rectangle::new(0, 0, rb.width()-1, rb.height()-1, &rb);
    outer_border.draw_border(&rb, Color::Blue);
    outer_border.print_header(&rb, Color::White, Color::Black, "Hello World");
    outer_border.draw();

    let start_y = 3;
    let addrs = (0..255).cycle().take(100*4-1).collect::<Vec<u8>>();
    for j in 0..addrs.len()/4 {
        let i = j * 4;
        let v: u32 = ((addrs[i] as u32) << 24) +
            ((addrs[i+1] as u32) << 16) +
            ((addrs[i+2] as u32) << 8) +
            addrs[i+4] as u32;
        rb.print(3, start_y+j, rustbox::RB_NORMAL, Color::White, Color::Black,
                 &format!("{:#08x}: ", i));
        rb.print(13, start_y+j, rustbox::RB_NORMAL, Color::White, Color::Blue,
                 &format!("{}", v));
    }

    loop {
        rb.present();
        match rb.poll_event(false) {
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
