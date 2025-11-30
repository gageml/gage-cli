use cursive::{
    view::ViewWrapper,
    views::{Layer, TextView},
    wrap_impl,
};

use crate::review::theme;

pub struct Header {
    inner: Layer<TextView>,
}

impl Header {
    pub fn new() -> Self {
        Self {
            inner: Layer::with_color(TextView::empty().center(), theme::Style::header_title()),
        }
    }

    pub fn title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.set_title(title);
        self
    }

    pub fn set_title<S>(&mut self, title: S)
    where
        S: Into<String>,
    {
        self.inner.get_inner_mut().set_content(title.into());
    }
}

impl ViewWrapper for Header {
    wrap_impl!(self.inner: Layer<TextView>);
}
