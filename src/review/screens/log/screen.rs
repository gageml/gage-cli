use std::sync::Arc;

use cursive::{
    Cursive, ScreenId, View,
    event::{Event, EventResult, Key},
    theme::{BaseColor, Effect},
    utils::markup::StyledString,
    view::{Finder, Nameable, Resizable, ViewWrapper},
    views::{BoxedView, LinearLayout, ResizedView, TextView, ViewRef},
    wrap_impl,
};
use itertools::intersperse;
use pyo3::Python;

use crate::{
    cursive::{view::Padding, views::ScreensView},
    inspect::log::{EvalLog, read_log},
    py,
    review::{
        App, AppScreen,
        components::{footer::Footer, header::Header},
        dialogs::help::HelpDialog,
        screens::log::{advanced::AdvancedView, simplified::SampleView},
        theme,
    },
};

pub struct LogScreen {
    log: Option<Arc<EvalLog>>,
    active_sample: Option<usize>,
    sample_view: ScreenId,
    advanced_view: ScreenId,
    error_view: ScreenId,
    screens_pos: usize,
    inner: ResizedView<LinearLayout>,
}

impl LogScreen {
    pub fn new() -> Self {
        let mut screens = ScreensView::new();

        // Screen views
        let sample_view = screens.add_screen(BoxedView::new(Box::new(SampleView::new())));
        let advanced_view = screens.add_screen(BoxedView::new(Box::new(AdvancedView::new())));
        let error_view = screens.add_screen(BoxedView::new(Box::new(TextView::empty())));

        // Layout
        let mut inner = LinearLayout::vertical()
            .child(Header::new().title("Gage - Review").with_name("header"))
            .child(screens.full_screen()) // pos 1
            .child(
                Footer::new()
                    .help_keys(["?:help", "q:close"])
                    .with_name("footer")
                    .full_width()
                    .pad_t(1),
            )
            .full_width();

        // Screens child pos
        let screens_pos = 1;

        // Initial focus on screens
        inner.get_inner_mut().set_focus_index(screens_pos).unwrap();

        Self {
            log: None,
            active_sample: None,
            sample_view,
            advanced_view,
            error_view,
            screens_pos,
            inner,
        }
    }

    pub fn set_log_location(&mut self, location: &str) {
        py::init();
        Python::attach(|py| match read_log(py, location) {
            Ok(log) => {
                let log = Arc::new(log);
                let active_sample = if let Some(samples) = log.samples.as_ref()
                    && !samples.is_empty()
                {
                    Some(0)
                } else {
                    None
                };
                self.sample_view().set_log_sample(&log, active_sample, true);
                self.advanced_view().set_log_sample(&log, active_sample);
                self.active_sample = active_sample;
                self.log = Some(log);
                self.set_active(self.sample_view);
            }
            Err(err) => {
                self.sample_view().clear();
                self.advanced_view().clear();
                self.error_view().set_content(StyledString::concatenate([
                    StyledString::styled("Error loading log", Effect::Bold),
                    StyledString::styled(format!("\n\n{location}\n\n"), Effect::Dim),
                    StyledString::styled(format!("{err:?}"), BaseColor::Red.light()),
                ]));
                self.log = None;
                self.active_sample = None;
                self.set_active(self.error_view);
            }
        });
    }

    fn prev_sample(&mut self) {
        let next = if let Some(active_sample) = self.active_sample {
            active_sample.saturating_sub(1)
        } else {
            // No active sample, use last sample
            let sample_count = self
                .log
                .as_ref()
                .map(|l| l.samples.as_ref().map(|s| s.len()).unwrap_or(0))
                .unwrap_or(0);
            sample_count.saturating_sub(1)
        };
        self.goto_sample(next);
    }

    fn next_sample(&mut self) {
        let next = if let Some(active_sample) = self.active_sample {
            active_sample.saturating_add(1)
        } else {
            0
        };
        self.goto_sample(next);
    }

    fn goto_sample(&mut self, sample: usize) -> bool {
        if let Some(log) = self.log.as_ref() {
            let sample_count = log.samples.as_ref().map(|s| s.len()).unwrap_or(0);
            if sample < sample_count {
                let log = Arc::clone(log);
                self.sample_view().set_log_sample(&log, Some(sample), false);
                self.advanced_view().set_log_sample(&log, Some(sample));
                self.active_sample = Some(sample);
                self.refresh_footer();
                return true;
            }
        }
        false
    }

    fn header(&mut self) -> ViewRef<Header> {
        self.inner.find_name("header").unwrap()
    }

    fn screens(&self) -> &ScreensView {
        self.inner
            .get_inner()
            .get_child(self.screens_pos)
            .unwrap()
            .downcast_ref::<ResizedView<ScreensView>>()
            .unwrap()
            .get_inner()
    }

    fn screens_mut(&mut self) -> &mut ScreensView {
        self.inner
            .get_inner_mut()
            .get_child_mut(self.screens_pos)
            .unwrap()
            .downcast_mut::<ResizedView<ScreensView>>()
            .unwrap()
            .get_inner_mut()
    }

    fn set_active(&mut self, screen: ScreenId) {
        // Header
        if screen == self.sample_view {
            self.header().set_title("Gage - Review sample");
        } else if screen == self.advanced_view {
            self.header().set_title("Gage - Advanced view");
        } else {
            assert!(screen == self.error_view, "{screen}");
            self.header().set_title("Gage - Error");
        }

        // Active screen
        self.screens_mut().set_active_screen(screen);

        // Footer (requires updated active screen)
        self.refresh_footer();
    }

    fn refresh_footer(&mut self) {
        let mut footer_sections = Vec::new();
        if let Some(log) = self.log.as_ref() {
            // Short log Id
            let log_id = log.short_log_id();
            footer_sections.push(StyledString::concatenate([
                StyledString::styled("Log ", theme::Style::footer_caption()),
                StyledString::styled(log_id, theme::Style::footer_highlight()),
            ]));

            // Active sample
            if let Some(active_sample) = self.active_sample {
                let sample_count = log.samples.as_ref().expect("sample count > 0").len();
                footer_sections.push(StyledString::concatenate([
                    StyledString::styled("Sample ", theme::Style::footer_caption()),
                    StyledString::styled(
                        format!("{}", active_sample + 1),
                        theme::Style::footer_highlight(),
                    ),
                    StyledString::styled(" of ", theme::Style::footer_caption()),
                    StyledString::styled(
                        format!("{sample_count}"),
                        theme::Style::footer_highlight(),
                    ),
                ]));
            }
        }
        if self.active_screen() == self.error_view {
            footer_sections.push(StyledString::styled(
                "Error",
                theme::Style::footer_caption(),
            ));
        }
        self.footer()
            .set_status(StyledString::concatenate(intersperse(
                footer_sections,
                StyledString::styled(" | ", theme::Style::footer_sep()),
            )));
    }

    fn active_screen(&mut self) -> ScreenId {
        self.screens().active_screen()
    }

    fn sample_view(&mut self) -> &mut SampleView {
        let screen = self.sample_view;
        self.screens_mut()
            .get_screen_mut(screen)
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    fn advanced_view(&mut self) -> &mut AdvancedView {
        let screen = self.advanced_view;
        self.screens_mut()
            .get_screen_mut(screen)
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    fn error_view(&mut self) -> &mut TextView {
        let screen = self.error_view;
        self.screens_mut()
            .get_screen_mut(screen)
            .unwrap()
            .downcast_mut::<TextView>()
            .unwrap()
    }

    fn footer(&mut self) -> ViewRef<Footer> {
        self.inner.find_name("footer").unwrap()
    }
}

impl ViewWrapper for LogScreen {
    wrap_impl!(self.inner: ResizedView<LinearLayout>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            // Sample view
            Event::Char('1') => {
                if self.log.is_some() {
                    self.set_active(self.sample_view);
                }
                EventResult::consumed()
            }

            // Advanced view
            Event::Char('2') => {
                if self.log.is_some() {
                    self.set_active(self.advanced_view);
                }
                EventResult::consumed()
            }

            // Show debug console
            Event::Char('`') => {
                EventResult::with_cb_once(|siv| App::push_screen(siv, AppScreen::Console))
            }

            // Left key - previous sample
            Event::Key(Key::Left) => {
                self.prev_sample();
                EventResult::consumed()
            }

            // Right key - next sample
            Event::Key(Key::Right) => {
                self.next_sample();
                EventResult::consumed()
            }

            // Help
            Event::Char('?') => {
                let active_screen = self.active_screen();
                if active_screen == self.sample_view {
                    EventResult::with_cb_once(Self::on_sample_view_help)
                } else if active_screen == self.advanced_view {
                    EventResult::with_cb_once(Self::on_advanced_view_help)
                } else {
                    assert!(active_screen == self.error_view, "{active_screen}");
                    EventResult::with_cb_once(Self::on_error_view_help)
                }
            }
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb_once(App::pop_screen),
            _ => self.inner.on_event(event),
        }
    }
}

// Event handlers
impl LogScreen {
    fn on_sample_view_help(siv: &mut Cursive) {
        let help = vec![
            (
                None,
                vec![
                    ("Left, Right", "View next and previous samples".into()),
                    ("Down, Up", "Highlight next and previous sections".into()),
                ],
            ),
            (
                Some("View"),
                vec![
                    ("1", "Simplified view (current)".into()),
                    ("2", "Advanced view".into()),
                ],
            ),
            (None, vec![("q", "Close log".into())]),
        ];
        siv.add_layer(HelpDialog::new(help).title("Help - Simplified view"));
    }

    fn on_advanced_view_help(siv: &mut Cursive) {
        let help = vec![
            (None, vec![("Up, Down", "Scroll log info".into())]),
            (
                Some("View"),
                vec![
                    ("1", "Simplified view".into()),
                    ("2", "Advanced view (current)".into()),
                ],
            ),
            (None, vec![("q", "Close log".into())]),
        ];
        siv.add_layer(HelpDialog::new(help).title("Help - Advanced view"));
    }

    fn on_error_view_help(siv: &mut Cursive) {
        let help = vec![(None, vec![("q", "Close log".into())])];
        siv.add_layer(HelpDialog::new(help).title("Help - Review"));
    }
}
