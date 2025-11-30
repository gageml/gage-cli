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
    inspect::log::{Attachments, SampleInput},
    review::screens::log::simplified::ChatMessageContentView,
};

pub struct InputView {
    inner: PageLayout,
}

impl InputView {
    pub fn new(input: &SampleInput, attachments: &Attachments) -> Self {
        let mut inner = PageLayout::new();
        match input {
            SampleInput::String(s) => {
                let s = if s.is_empty() { empty() } else { s.into() };
                inner.add_child(TextView::new(s));
            }
            SampleInput::ChatMessageList(l) => {
                if l.is_empty() {
                    inner.add_child(TextView::new(empty()))
                } else {
                    for (i, chat_msg) in l.iter().enumerate() {
                        inner.add_child(
                            ChatMessageContentView::new(&chat_msg.base().content, attachments)
                                .pad_t(min(i, 0)),
                        );
                    }
                }
            }
        }
        Self { inner }
    }
}

fn empty() -> StyledString {
    StyledString::styled("Empty", Style::from(Effect::Italic).combine(Effect::Dim))
}

impl ViewWrapper for InputView {
    wrap_impl!(self.inner: PageLayout);
}
