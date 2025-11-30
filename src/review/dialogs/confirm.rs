use cursive::{
    Cursive, View,
    align::HAlign,
    event::{Event, EventResult, Key},
    view::ViewWrapper,
    views::TextView,
    wrap_impl,
};

use crate::{
    cursive::views::{Button, Dialog},
    handle_wrapped_dialog_event,
};

pub struct ConfirmDialog {
    inner: Dialog,
}

impl ConfirmDialog {
    pub fn new<S, F>(prompt: S, cb: F) -> Self
    where
        S: Into<String>,
        F: Fn(&mut Cursive) + Send + Sync + 'static,
    {
        let inner = Dialog::new()
            .title("Confirm")
            .content(TextView::new(prompt.into()))
            .padding_lrtb(2, 2, 1, 0)
            .h_align(HAlign::Center)
            .button("Yes", cb)
            .dismiss_button("No");

        Self { inner }
    }

    fn yes_button(&mut self) -> &mut Button {
        self.inner
            .buttons_mut()
            .find(|b| b.label() == " Yes ")
            .unwrap()
    }

    fn close(siv: &mut Cursive) {
        siv.pop_layer();
    }
}

impl ViewWrapper for ConfirmDialog {
    wrap_impl!(self.inner: Dialog);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('y') => self.yes_button().on_event(Event::Key(Key::Enter)),
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb(Self::close),
            Event::Char('n') => EventResult::with_cb(Self::close),
            Event::Key(Key::Esc) => EventResult::with_cb(Self::close),
            _ => handle_wrapped_dialog_event!(self, event),
        }
    }
}
