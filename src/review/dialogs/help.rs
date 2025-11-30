use cursive::{
    Cursive, View, With,
    align::HAlign,
    event::{Event, EventResult, Key},
    theme::{ColorType, Effect},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::{DummyView, PaddedView, TextView},
    wrap_impl,
};

use crate::{
    cursive::{
        view::{Padding, Scrollable},
        views::{Dialog, PageLayout},
    },
    handle_wrapped_dialog_event,
    review::app::Help,
};

pub struct HelpView {
    inner: PageLayout,
}

impl HelpView {
    pub fn new(help: Help) -> Self {
        let max_key_width = Self::max_key_width(&help);
        let inner = PageLayout::new().with(|v| {
            for (i, (title, keys)) in help.into_iter().enumerate() {
                if i > 0 {
                    v.add_child(DummyView);
                }
                if let Some(title) = title {
                    v.add_child(TextView::new(StyledString::styled(title, Effect::Dim)));
                }
                for (key, key_help) in keys {
                    let pad = " ".repeat(max_key_width - key.len());
                    v.add_child(TextView::new(StyledString::concatenate([
                        StyledString::styled(key, ColorType::highlight()),
                        StyledString::plain(format!("{pad}  ")),
                        key_help,
                    ])));
                }
            }
        });
        Self { inner }
    }

    fn max_key_width(help: &Help) -> usize {
        help.iter()
            .map(|(_, keys)| keys.iter().map(|(key, _)| key.len()).max().unwrap_or(0))
            .max()
            .unwrap_or(0)
    }
}

impl ViewWrapper for HelpView {
    wrap_impl!(self.inner: PageLayout);
}

pub struct HelpDialog {
    inner: PaddedView<Dialog>,
}

impl HelpDialog {
    pub fn new(help: Help) -> Self {
        let help_view = HelpView::new(help);
        Self {
            inner: Dialog::around(help_view.scrollable())
                .padding_lrtb(2, 2, 1, 0)
                .title("Help")
                .button("Close", Self::on_close)
                .h_align(HAlign::Center)
                .pad_x(1),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_title(&mut self, title: &str) {
        self.inner.get_inner_mut().set_title(title);
    }
}

impl ViewWrapper for HelpDialog {
    wrap_impl!(self.inner: PaddedView<Dialog>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb(Self::on_close),
            Event::Key(Key::Esc) => EventResult::with_cb(Self::on_close),
            _ => handle_wrapped_dialog_event!(self, event),
        }
    }
}

// Event handlers
impl HelpDialog {
    fn on_close(s: &mut Cursive) {
        s.pop_layer();
    }
}
