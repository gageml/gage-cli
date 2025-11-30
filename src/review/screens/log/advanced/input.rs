use std::cmp::min;

use cursive::View;

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::log::{Attachments, SampleInput},
    review::screens::log::advanced::{
        chat_message::chat_message_view,
        util::{resolve_attachment, text_content_view},
    },
};

pub fn input_view(input: &SampleInput, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    match input {
        SampleInput::String(s) => {
            view.add_child(text_content_view(&resolve_attachment(s, attachments)))
        }
        SampleInput::ChatMessageList(l) => {
            for (i, msg) in l.iter().enumerate() {
                view.add_child(chat_message_view(msg, attachments).pad_t(min(i, 1)));
            }
        }
    }
    view
}
