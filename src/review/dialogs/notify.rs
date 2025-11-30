use cursive::{
    Cursive, View,
    align::HAlign,
    event::{Event, EventResult, Key},
    view::ViewWrapper,
    views::TextView,
    wrap_impl,
};

use crate::{cursive::views::Dialog, handle_wrapped_dialog_event};

pub struct NotifyDialog {
    inner: Dialog,
}

impl NotifyDialog {
    pub fn new<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        let inner = Dialog::new()
            .content(TextView::new(msg.into()))
            .padding_lrtb(2, 2, 1, 0)
            .h_align(HAlign::Center)
            .dismiss_button("OK");

        Self { inner }
    }

    fn close(siv: &mut Cursive) {
        siv.pop_layer();
    }
}

impl ViewWrapper for NotifyDialog {
    wrap_impl!(self.inner: Dialog);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb(Self::close),
            Event::Key(Key::Esc) => EventResult::with_cb(Self::close),
            _ => handle_wrapped_dialog_event!(self, event),
        }
    }
}
