use cursive::{
    Cursive, View,
    align::HAlign,
    event::{Event, EventResult, Key},
    view::{Resizable, ViewWrapper},
    views::{EditView, LayerPosition, PaddedView, ResizedView},
    wrap_impl,
};

use crate::{
    cursive::{view::Padding, views::Dialog},
    handle_wrapped_dialog_event,
    review::{App, AppScreen, screens::console::ConsoleScreen},
};

pub struct FilterDialog {
    inner: PaddedView<Dialog>,
}

impl FilterDialog {
    pub fn new() -> Self {
        Self {
            inner: Dialog::new()
                .title("Filter output")
                .content(EditView::new().on_submit(Self::on_submit).fixed_width(40))
                .padding_lrtb(2, 2, 1, 0)
                .h_align(HAlign::Center)
                .button("Apply", Self::apply)
                .dismiss_button("Cancel")
                .pad_x(1),
        }
    }

    pub fn filter<S>(mut self, filter: S) -> Self
    where
        S: Into<String> + Default,
    {
        self.set_filter(filter);
        self
    }

    pub fn set_filter<S>(&mut self, filter: S)
    where
        S: Into<String> + Default,
    {
        self.filter_view_mut().set_content(filter.into());
    }

    fn filter_view(&self) -> &EditView {
        self.inner
            .get_inner()
            .get_content()
            .downcast_ref::<ResizedView<EditView>>()
            .unwrap()
            .get_inner()
    }

    fn filter_view_mut(&mut self) -> &mut EditView {
        self.inner
            .get_inner_mut()
            .get_content_mut()
            .downcast_mut::<ResizedView<EditView>>()
            .unwrap()
            .get_inner_mut()
    }

    fn on_submit(siv: &mut Cursive, filter: &str) {
        siv.pop_layer();
        let filter = filter.to_string();
        App::with_screen(
            siv,
            AppScreen::Console,
            move |screen: &mut ConsoleScreen| {
                screen.set_filter(&filter);
            },
        );
    }

    fn apply(siv: &mut Cursive) {
        let view = siv.screen().get(LayerPosition::FromBack(1)).unwrap();
        let dialog = view.downcast_ref::<Self>().unwrap();
        Self::on_submit(siv, dialog.filter_view().get_content().as_str());
    }

    fn close(siv: &mut Cursive) {
        siv.pop_layer();
    }
}

impl ViewWrapper for FilterDialog {
    wrap_impl!(self.inner: PaddedView<Dialog>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb(Self::close),
            Event::Key(Key::Esc) => EventResult::with_cb(Self::close),
            _ => handle_wrapped_dialog_event!(self, event),
        }
    }
}
