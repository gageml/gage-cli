use cursive::{
    align::HAlign,
    view::ViewWrapper,
    views::{PaddedView, TextView},
    wrap_impl,
};

use crate::cursive::{view::Padding, views::Dialog};

pub struct StatusDialog {
    inner: PaddedView<Dialog>,
}

impl StatusDialog {
    pub fn new(msg: &str) -> Self {
        let inner = Dialog::around(TextView::new(msg))
            .padding_lrtb(2, 2, 0, 0)
            .h_align(HAlign::Center)
            .pad_x(1);
        Self { inner }
    }
}

impl ViewWrapper for StatusDialog {
    wrap_impl!(self.inner: PaddedView<Dialog>);
}
