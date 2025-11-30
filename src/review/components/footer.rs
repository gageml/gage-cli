use cursive::{Vec2, View, utils::markup::StyledString};
use itertools::intersperse;

use crate::review::theme;

pub struct Footer {
    status: StyledString,
    help: StyledString,
}

pub enum HelpKey {
    Key(&'static str, &'static str),
    Sep,
}

impl From<&'static str> for HelpKey {
    fn from(s: &'static str) -> Self {
        if s == "|" {
            Self::Sep
        } else {
            let (key, caption) = s.split_once(":").expect("format 'key:caption'");
            Self::Key(key, caption)
        }
    }
}

impl Footer {
    pub fn new() -> Self {
        Self {
            status: StyledString::new(),
            help: StyledString::new(),
        }
    }

    pub fn status<S>(mut self, status: S) -> Self
    where
        S: Into<StyledString>,
    {
        self.set_status(status);
        self
    }

    pub fn set_status<S>(&mut self, status: S)
    where
        S: Into<StyledString>,
    {
        self.status = status.into();
    }

    pub fn help_keys<K, I>(mut self, keys: K) -> Self
    where
        K: IntoIterator<Item = I>,
        I: Into<HelpKey>,
    {
        self.set_help_keys(keys);
        self
    }

    pub fn set_help_keys<K, I>(&mut self, keys: K)
    where
        K: IntoIterator<Item = I>,
        I: Into<HelpKey>,
    {
        self.help = StyledString::concatenate(intersperse(
            keys.into_iter().flat_map(|hk| match hk.into() {
                HelpKey::Key(key, help) => vec![
                    StyledString::styled(key, theme::Style::footer_key()),
                    StyledString::styled(help, theme::Style::footer_caption()),
                ],
                HelpKey::Sep => vec![StyledString::styled("|", theme::Style::footer_sep())],
            }),
            StyledString::plain(" "),
        ));
    }
}

impl View for Footer {
    fn needs_relayout(&self) -> bool {
        // TODO not sure here do I?
        false
    }

    fn layout(&mut self, _size: Vec2) {
        // Anything to do??
    }

    fn draw(&self, printer: &cursive::Printer) {
        printer.with_color(theme::Style::panel(), |printer| {
            // Footer background
            printer.print_hline((0, 0), printer.size.x, " ");

            // Status
            printer.print_styled((1, 0), &self.status);

            // Move to help start leaving room for 1 right pad, 2 spaces
            let x = printer.size.x.saturating_sub(self.help.width() + 4);

            // Use 2 spaces to separate from status
            printer.print((x + 1, 0), "  ");
            printer.print_styled((x + 3, 0), &self.help);

            // If truncated status, print ellipsis
            if x < self.status.width() {
                printer.print_styled(
                    (x, 0),
                    &StyledString::styled("…", theme::Style::footer_caption()),
                );
            }
        });
    }
}
