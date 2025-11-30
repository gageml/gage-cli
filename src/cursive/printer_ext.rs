use cursive::{Printer, Vec2};

pub trait PrinterExt {
    fn print_box_rounded<T: Into<Vec2>, S: Into<Vec2>>(&self, start: T, size: S, invert: bool);
}

impl<'a, 'b> PrinterExt for Printer<'a, 'b> {
    fn print_box_rounded<T: Into<Vec2>, S: Into<Vec2>>(&self, start: T, size: S, invert: bool) {
        let start = start.into();
        let size = size.into();

        if size.x < 2 || size.y < 2 {
            return;
        }
        let size = size - (1, 1);

        self.with_high_border(invert, |s| {
            s.print(start, "╭");
            s.print(start + size.keep_y(), "╰");
            s.print_hline(start + (1, 0), size.x - 1, "─");
            s.print_vline(start + (0, 1), size.y - 1, "│");
        });

        self.with_low_border(invert, |s| {
            s.print(start + size.keep_x(), "╮");
            s.print(start + size, "╯");
            s.print_hline(start + (1, 0) + size.keep_y(), size.x - 1, "─");
            s.print_vline(start + (0, 1) + size.keep_x(), size.y - 1, "│");
        });
    }
}
