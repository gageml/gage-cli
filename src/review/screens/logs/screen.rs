use std::path::Path;

use cursive::{
    Cursive, View,
    event::{Event, EventResult},
    utils::markup::StyledString,
    view::{Resizable, ViewWrapper},
    views::{LinearLayout, ResizedView},
    wrap_impl,
};
use itertools::intersperse;

use crate::review::{
    app::{App, AppScreen},
    components::{footer::Footer, header::Header, table::Sort},
    dialogs::{help::HelpDialog, notify::NotifyDialog, status::StatusDialog},
    screens::logs::{
        filter::FilterDialog,
        sort::SortDialog,
        view::{Col, Filter, LogsView},
    },
    theme,
};

pub struct LogsScreen {
    logs_pos: usize,
    footer_pos: usize,
    inner: ResizedView<LinearLayout>,
}

impl LogsScreen {
    pub fn new(log_dir: &Path) -> Self {
        let mut inner = LinearLayout::vertical()
            .child(Header::new().title("Gage - Logs"))
            .child(LogsView::new(log_dir).full_screen())
            .child(
                Footer::new()
                    .status(Self::fmt_log_dir(log_dir))
                    .help_keys(["?:help", "q:quit"])
                    .full_width(),
            )
            .full_width();

        // Children
        let logs_pos = 1;
        let footer_pos = 2;

        // Initial focus on logs
        inner.get_inner_mut().set_focus_index(1).unwrap();

        Self {
            logs_pos,
            footer_pos,
            inner,
        }
    }

    fn fmt_log_dir(log_dir: &Path) -> StyledString {
        let cwd = std::env::current_dir().unwrap();
        let path = log_dir.strip_prefix(cwd).unwrap_or(log_dir);
        StyledString::concatenate([
            StyledString::styled("Log dir ", theme::Style::footer_caption()),
            StyledString::styled(path.to_str().unwrap(), theme::Style::footer_highlight()),
        ])
    }

    fn logs(&self) -> &LogsView {
        self.inner
            .get_inner()
            .get_child(self.logs_pos)
            .unwrap()
            .downcast_ref::<ResizedView<LogsView>>()
            .unwrap()
            .get_inner()
    }

    fn logs_mut(&mut self) -> &mut LogsView {
        self.inner
            .get_inner_mut()
            .get_child_mut(self.logs_pos)
            .unwrap()
            .downcast_mut::<ResizedView<LogsView>>()
            .unwrap()
            .get_inner_mut()
    }

    fn footer_mut(&mut self) -> &mut Footer {
        self.inner
            .get_inner_mut()
            .get_child_mut(self.footer_pos)
            .unwrap()
            .downcast_mut::<ResizedView<Footer>>()
            .unwrap()
            .get_inner_mut()
    }

    pub fn get_sort(&self) -> Option<&Sort<Col>> {
        self.logs().get_sort()
    }

    pub fn set_sort(&mut self, sort: Sort<Col>) {
        self.logs_mut().set_sort(sort);
    }

    pub fn get_filter(&self) -> Option<&Filter> {
        self.logs().get_filter()
    }

    pub fn set_filter(&mut self, filter: Filter) {
        let status = StyledString::concatenate([
            Self::fmt_log_dir(self.logs().log_dir()),
            StyledString::styled(" | ", theme::Style::footer_sep()),
            Self::fmt_filter(&filter),
        ]);
        self.logs_mut().set_filter(filter);
        self.footer_mut().set_status(status);
    }

    fn fmt_filter(filter: &Filter) -> StyledString {
        let mut parts = Vec::new();
        macro_rules! maybe_part {
            ($label:literal, $attr:ident) => {
                if let Some(val) = filter.$attr.as_ref() {
                    parts.push(StyledString::concatenate([
                        StyledString::styled(
                            format!("{} ", $label),
                            theme::Style::footer_caption(),
                        ),
                        StyledString::styled(
                            if val.is_empty() { "<empty>" } else { val },
                            theme::Style::footer_highlight(),
                        ),
                    ]));
                }
            };
        }
        maybe_part!("Task", task);
        maybe_part!("Status", status);
        maybe_part!("Model", model);
        maybe_part!("Dataset", dataset);
        StyledString::concatenate([StyledString::concatenate(intersperse(
            parts,
            StyledString::styled(" | ", theme::Style::footer_sep()),
        ))])
    }

    pub fn clear_filter(&mut self) {
        let status = Self::fmt_log_dir(self.logs().log_dir());
        self.logs_mut().clear_filter();
        self.footer_mut().set_status(status);
    }
}

impl ViewWrapper for LogsScreen {
    wrap_impl!(self.inner: ResizedView<LinearLayout>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('`') => {
                EventResult::with_cb_once(|siv| App::push_screen(siv, AppScreen::Console))
            }
            Event::Char('s') => EventResult::with_cb_once(Self::on_sort),
            Event::Char('f') => EventResult::with_cb_once(Self::on_filter),
            Event::Char('r') => EventResult::with_cb_once(Self::on_refresh),
            Event::Char('?') => EventResult::with_cb_once(Self::on_help),
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb_once(App::quit),
            _ => self.inner.get_inner_mut().on_event(event),
        }
    }
}

// Event handlers
impl LogsScreen {
    fn on_sort(siv: &mut Cursive) {
        let sort = App::with_screen(siv, AppScreen::Logs, |screen: &mut Self| {
            screen.logs().get_sort().cloned()
        })
        .unwrap();
        siv.add_layer(SortDialog::new(sort));
    }

    fn on_filter(siv: &mut Cursive) {
        let filter = App::with_screen(siv, AppScreen::Logs, |screen: &mut Self| {
            screen.logs().get_active().map(Filter::from)
        })
        .unwrap();
        if let Some(filter) = filter {
            siv.add_layer(FilterDialog::new(filter));
        } else {
            siv.add_layer(NotifyDialog::new("Nothing to filter."));
        }
    }

    fn on_refresh(siv: &mut Cursive) {
        siv.add_layer(StatusDialog::new("Refreshing..."));
        // Tee up a callback that starts a thread to refresh - without
        // this indirection Cursive processes `add_layer` and the
        // refresh in the same tick.
        siv.cb_sink()
            .send(Box::new(|siv| {
                let cb = siv.cb_sink().clone();
                std::thread::spawn(move || {
                    cb.send(Box::new(|siv| {
                        App::with_screen(siv, AppScreen::Logs, |screen: &mut Self| {
                            screen.logs_mut().refresh_items();
                        });
                        siv.pop_layer();
                    }))
                    .unwrap();
                });
            }))
            .unwrap();
    }

    fn on_help(siv: &mut Cursive) {
        let help = vec![
            (
                None,
                vec![
                    ("Up, Down", "Navigate".into()),
                    ("Enter", "Open log".into()),
                ],
            ),
            (
                None,
                vec![
                    ("s", "Sort".into()),
                    ("f", "Filter".into()),
                    ("r", "Refresh".into()),
                ],
            ),
            (None, vec![("q", "Exit".into())]),
        ];
        siv.add_layer(HelpDialog::new(help).title("Help - Logs"));
    }
}
