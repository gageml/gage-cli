use std::cmp::min;

use cursive::{
    theme::{BaseColor, Effect, Style},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::TextView,
    wrap_impl,
};

use crate::{
    cursive::{
        view::Padding,
        views::{PageLayout, PlainTextView},
    },
    inspect::error::EvalError,
};

pub struct ErrorsView {
    inner: PageLayout,
}

impl ErrorsView {
    pub fn new(errors: Vec<&EvalError>) -> Self {
        let mut inner = PageLayout::new();
        for (i, error) in errors.into_iter().enumerate() {
            inner.add_child(
                TextView::new(StyledString::styled(
                    &error.message,
                    Style::from(BaseColor::Red.light()).combine(Effect::Bold),
                ))
                .pad_t(min(i, 1)),
            );
            inner.add_child(PlainTextView::new(error.traceback.trim_end()).pad_t(1))
        }
        Self { inner }
    }
}

impl ViewWrapper for ErrorsView {
    wrap_impl!(self.inner: PageLayout);
}
