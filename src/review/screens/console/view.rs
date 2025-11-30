use std::cmp;

use cursive::{
    Printer, Rect, Vec2, View,
    direction::Direction,
    event::{Event, EventResult},
    logger::{self, Record},
    theme::{BaseColor, Effect, Style},
    utils::markup::StyledString,
    view::CannotFocus,
};
use log::Level;

use crate::{
    cursive::view::scroll::{self, ScrollStrategy},
    impl_scroller,
};

pub struct ConsoleView {
    filter: String,
    last_log_count: usize,
    invalidated: bool,
    scroll: scroll::Core,
    lines: Vec<StyledString>,
}

impl ConsoleView {
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            last_log_count: 0,
            invalidated: true,
            scroll: scroll::Core::new(),
            lines: Vec::new(),
        }
    }

    pub fn set_filter<S>(&mut self, filter: S)
    where
        S: Into<String>,
    {
        self.filter = filter.into();
        self.invalidate();
    }

    pub fn get_filter(&self) -> &str {
        &self.filter
    }

    fn filter_item(&self, item: &Record) -> bool {
        self.filter.is_empty()
            || item
                .message
                .to_lowercase()
                .contains(&self.filter.to_lowercase())
    }

    fn invalidate(&mut self) {
        self.invalidated = true;
    }
}

impl_scroller!(ConsoleView::scroll);

impl View for ConsoleView {
    fn draw(&self, printer: &Printer) {
        scroll::draw(self, printer, Self::draw_inner);
    }

    fn layout(&mut self, size: Vec2) {
        scroll::layout(
            self,
            size,
            self.needs_relayout(),
            Self::layout_inner,
            Self::required_size_inner,
        );

        self.invalidated = false;
        self.last_log_count = logger::LOGS.lock().unwrap().len();
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated || logger::LOGS.lock().unwrap().len() != self.last_log_count
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        scroll::on_event(
            self,
            event,
            Self::on_event_inner,
            Self::inner_important_area,
        )
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::consumed())
    }
}

// Scroll inner
impl ConsoleView {
    fn draw_inner(&self, printer: &Printer) {
        // Work already done in lines generate (used for sizing)
        for (i, line) in self.lines.iter().enumerate() {
            printer.print_styled((0, i), line);
        }
    }

    fn layout_inner(&mut self, _size: Vec2) {
        // Nothing to layout - all out content is scrollable
    }

    fn required_size_inner(&mut self, size: Vec2) -> Vec2 {
        // Pre-format lines to get required size - used also for draws
        let logs = logger::LOGS.try_lock().unwrap();
        let (mut lines, mut max_line) =
            logs.iter()
                .fold((Vec::new(), 0), |(mut lines_acc, mut max_len), item| {
                    if self.filter_item(item) {
                        let record_lines = format_record(item);
                        max_len = cmp::max(
                            max_len,
                            record_lines.iter().map(|s| s.width()).max().unwrap_or(0),
                        );
                        lines_acc.extend(record_lines);
                    }
                    (lines_acc, max_len)
                });

        // If nothing to show, show waiting message
        if lines.is_empty() {
            let msg = StyledString::styled(
                "Waiting for events…",
                Style::from(Effect::Italic).combine(Effect::Dim),
            );
            max_line = msg.width();
            lines.push(msg);
        }

        self.lines = lines;

        // Workaround what appears to be a bug in scroll support, where
        // scrollbars that appear without their corresponding side (i.e.
        // hor but not ver; ver but not hor) do not respond to mouse
        // events. This behavior can be circumvented by disabling the
        // not-displayed scrollbar. Along with this, we need to update
        // the scroll strategy to keep the scroll position tracking at
        // bottom.
        self.scroll.set_scroll_x(max_line > size.x);
        self.scroll.set_scroll_y(self.lines.len() > size.y);
        self.scroll
            .set_scroll_strategy(ScrollStrategy::StickToBottom);

        // Required size is max line width and line count
        Vec2::new(max_line, self.lines.len())
    }

    fn inner_important_area(&self, size: Vec2) -> Rect {
        // View doesn't implement "selected" state so it doesn't have an
        // important area - new items are made visible only when scroll
        // is in sticky position (buttom)
        Rect::from_size((0, size.y), (size.x, size.y))
    }

    fn on_event_inner(&mut self, _event: Event) -> EventResult {
        EventResult::Ignored
    }
}

fn format_record(r: &Record) -> Vec<StyledString> {
    let time = StyledString::styled(
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            r.time.hour(),
            r.time.minute(),
            r.time.second(),
            r.time.millisecond()
        ),
        Effect::Dim,
    );
    let level = StyledString::styled(
        format!("{:5}", r.level),
        match r.level {
            Level::Error => Style::from_color_style(BaseColor::Red.light().into()),
            Level::Warn => Style::from_color_style(BaseColor::Red.dark().into()),
            Level::Debug | Level::Trace => Style::from_color_style(BaseColor::Yellow.dark().into()),
            Level::Info => Style::from_color_style(BaseColor::Cyan.dark().into()),
        },
    );
    let message = format!(" {}", r.message).into();
    vec![StyledString::concatenate([
        time,
        StyledString::plain(" "),
        level,
        message,
    ])]
}
