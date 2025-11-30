use cursive::{Printer, Vec2, View};
use itertools::Itertools;
use textwrap::wrap;

pub struct PlainTextView {
    wrap: bool,
    // Original text lines split by line ending (`\n`)
    lines: Vec<String>,

    // Cached wrapped text - defined if wrap is true and size has
    // changed. Would like Vec<&str> here but lazily using
    // `textwrap::wrap`, which allocates new strings.
    wrapped: Option<Vec<String>>,

    // Last calculated required size given wrap status.
    required_size: Vec2,

    // Last required size constraint to determine wrapped staleness.
    last_req: Vec2,
}

impl PlainTextView {
    pub fn new(s: &str) -> Self {
        let lines = s.split("\n").map(|s| s.to_string()).collect_vec();
        let required_size = required_for_lines(&lines);
        Self {
            wrap: false,
            lines,
            wrapped: None,
            required_size,
            last_req: Vec2::default(),
        }
    }

    pub fn wrap(s: &str) -> Self {
        Self {
            wrap: true,
            ..Self::new(s)
        }
    }
}

impl View for PlainTextView {
    fn draw(&self, printer: &Printer) {
        let lines = if self.wrap {
            self.wrapped.as_ref().expect("from layout")
        } else {
            &self.lines
        };
        for (i, s) in lines.iter().enumerate() {
            printer.print((0, i), s);
        }
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        if self.wrap && (self.wrapped.is_none() || req != self.last_req) {
            // Need new wrapped lines
            let mut wrapped = Vec::new();
            for line in self.lines.iter() {
                for part in wrap(line, req.x) {
                    wrapped.push(part.into());
                }
            }
            self.required_size = required_for_lines(&wrapped);
            self.wrapped = Some(wrapped);
            self.last_req = req;
        }
        self.required_size
    }

    fn needs_relayout(&self) -> bool {
        // View is immutable
        false
    }
}

fn required_for_lines(lines: &[String]) -> Vec2 {
    Vec2::new(
        lines.iter().fold(0, |acc, v| std::cmp::max(acc, v.len())),
        lines.len(),
    )
}
