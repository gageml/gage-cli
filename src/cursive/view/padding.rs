use cursive::{View, view::Margins, views::PaddedView};

pub trait Padding: View + Sized {
    fn pad(self, p: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::lrtb(p, p, p, p), self)
    }

    fn pad_lrtb(self, l: usize, r: usize, t: usize, b: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::lrtb(l, r, t, b), self)
    }

    #[allow(dead_code)]
    fn pad_xy(self, x: usize, y: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::lrtb(x, x, y, y), self)
    }

    fn pad_x(self, x: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::lr(x, x), self)
    }

    fn pad_y(self, y: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::tb(y, y), self)
    }

    fn pad_lr(self, l: usize, r: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::lr(l, r), self)
    }

    fn pad_l(self, l: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::lr(l, 0), self)
    }

    fn pad_r(self, r: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::lr(0, r), self)
    }

    fn pad_t(self, t: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::tb(t, 0), self)
    }

    #[allow(dead_code)]
    fn pad_b(self, b: usize) -> PaddedView<Self> {
        PaddedView::new(Margins::tb(0, b), self)
    }
}

impl<T: View> Padding for T {}
