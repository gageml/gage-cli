use std::cmp::min;

use cursive::{
    theme::{Effect, Style},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::TextView,
    wrap_impl,
};

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::{log::Attachments, model::ModelOutput},
    review::screens::log::simplified::ChatMessageContentView,
};

fn empty() -> StyledString {
    StyledString::styled("Empty", Style::from(Effect::Italic).combine(Effect::Dim))
}

pub struct OutputView {
    inner: PageLayout,
}

impl OutputView {
    pub fn new(output: &ModelOutput, attachments: &Attachments) -> Self {
        let mut inner = PageLayout::new();
        if output.choices.is_empty() {
            inner.add_child(TextView::new(empty()));
        } else {
            for (i, choice) in output.choices.iter().enumerate() {
                inner.add_child(
                    ChatMessageContentView::new(&choice.message.base.content, attachments)
                        .pad_t(min(i, 1)),
                );
            }
        }
        Self { inner }
    }
}

impl ViewWrapper for OutputView {
    wrap_impl!(self.inner: PageLayout);
}
