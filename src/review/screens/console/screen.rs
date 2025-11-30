use cursive::{
    Cursive, View,
    event::{Event, EventResult},
    logger,
    utils::markup::StyledString,
    view::{Finder, Nameable, Resizable, ViewWrapper},
    views::{ResizedView, ViewRef},
    wrap_impl,
};

use crate::{
    cursive::views::PageLayout,
    review::{
        AppScreen,
        app::App,
        components::{footer::Footer, header::Header},
        dialogs::{confirm::ConfirmDialog, help::HelpDialog},
        screens::console::{filter::FilterDialog, view::ConsoleView},
        theme,
    },
};

pub struct ConsoleScreen {
    inner: ResizedView<PageLayout>,
}

impl ConsoleScreen {
    pub fn new() -> Self {
        let mut inner = PageLayout::new()
            .child(Header::new().title("Gage - Debug console"))
            .child(ConsoleView::new().with_name("console").full_screen())
            .child(
                Footer::new()
                    .status(Self::default_footer_status())
                    .help_keys(["?:help", "q:go back"])
                    .with_name("footer")
                    .full_width(),
            )
            .full_width();

        // Initial focus on console view
        inner.get_inner_mut().set_focus_index(1).unwrap();

        Self { inner }
    }

    fn default_footer_status() -> StyledString {
        StyledString::styled("All events", theme::Style::footer_caption())
    }

    fn filtered_footer_status(filter: &str) -> StyledString {
        StyledString::concatenate([
            StyledString::styled("Filter ", theme::Style::footer_caption()),
            StyledString::styled(filter.to_string(), theme::Style::footer_highlight()),
        ])
    }

    fn console(&mut self) -> ViewRef<ConsoleView> {
        self.find_name("console").unwrap()
    }

    fn footer(&mut self) -> ViewRef<Footer> {
        self.find_name("footer").unwrap()
    }

    pub fn set_filter(&mut self, filter: &str) {
        self.console().set_filter(filter);
        self.footer().set_status(if filter.is_empty() {
            Self::default_footer_status()
        } else {
            Self::filtered_footer_status(filter)
        });
    }
}

impl ViewWrapper for ConsoleScreen {
    wrap_impl!(self.inner: ResizedView<PageLayout>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('/') => EventResult::with_cb_once(Self::on_filter),
            Event::Char('c') => EventResult::with_cb_once(Self::on_clear),
            Event::Char('?') => EventResult::with_cb_once(Self::on_help),
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb_once(App::pop_screen),
            Event::Char('`') => EventResult::with_cb_once(App::pop_screen),
            _ => self.inner.get_inner_mut().on_event(event),
        }
    }
}

// Event handlers
impl ConsoleScreen {
    fn on_filter(siv: &mut Cursive) {
        // Get current filter used by consolve view
        let filter = App::with_screen(siv, AppScreen::Console, |screen: &mut ConsoleScreen| {
            screen.console().get_filter().to_string()
        })
        .unwrap();

        // Show dialog and update filter on submit
        siv.add_layer(FilterDialog::new().filter(filter));
    }

    fn on_clear(siv: &mut Cursive) {
        siv.add_layer(ConfirmDialog::new("Clear console events?", |siv| {
            siv.pop_layer();
            logger::LOGS.lock().unwrap().clear();
        }));
    }

    fn on_help(siv: &mut Cursive) {
        let help = vec![
            (
                None,
                vec![
                    ("Up, Down", "Scroll events".into()),
                    ("Left, Right", "".into()),
                ],
            ),
            (
                None,
                vec![("/", "Filter events".into()), ("c", "Clear events".into())],
            ),
            (None, vec![("q", "Go back to previous screen".into())]),
        ];
        siv.add_layer(HelpDialog::new(help).title("Help - Debug console"));
    }
}
