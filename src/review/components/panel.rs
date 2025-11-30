use std::sync::Arc;

use cursive::{
    Printer, Rect, Vec2, View,
    direction::Direction,
    event::{Event, EventResult},
    theme::Style,
    utils::markup::StyledString,
    view::CannotFocus,
};

use crate::review::theme;

pub struct Panel<V> {
    view: V,
    view_size: Vec2,
    title: Option<StyledString>,
    color: Style,
    border: Style,
    focus_border: Style,
    focus_cb: Option<FocusCb<V>>,
    invalidated: bool,
}

type FocusCb<V> = Arc<dyn Fn(&mut Panel<V>) -> EventResult + Send + Sync>;

impl<V> Panel<V> {
    pub fn new(view: V) -> Self {
        Panel {
            view,
            view_size: Vec2::default(),
            title: None,
            focus_cb: None,
            color: theme::Style::panel().into(),
            border: theme::Style::panel_border(),
            focus_border: theme::Style::panel_focus_border(),
            invalidated: true,
        }
    }

    pub fn color<S: Into<Style>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn border<S: Into<Style>>(mut self, border: S) -> Self {
        self.border = border.into();
        self
    }

    pub fn focus_border<S: Into<Style>>(mut self, focus_border: S) -> Self {
        self.focus_border = focus_border.into();
        self
    }

    // pub fn title<S: Into<String>>(mut self, title: S) -> Self {
    //     self.set_title(title);
    //     self
    // }

    // pub fn set_title<S: Into<String>>(&mut self, title: S) {
    //     self.title = Some(StyledString::styled(title, theme::Style::panel_caption()));
    //     self.invalidate();
    // }

    pub fn on_focus<F>(mut self, cb: F) -> Self
    where
        F: Fn(&mut Self) -> EventResult + Send + Sync + 'static,
    {
        self.set_on_focus(cb);
        self
    }

    pub fn set_on_focus<F>(&mut self, cb: F)
    where
        F: Fn(&mut Self) -> EventResult + Send + Sync + 'static,
    {
        self.focus_cb = Some(Arc::new(cb));
    }

    // pub fn get_inner_mut(&mut self) -> &mut V {
    //     &mut self.view
    // }

    // fn invalidate(&mut self) {
    //     self.invalidated = true;
    // }
}

impl<V: View> View for Panel<V> {
    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        if let Some(cb) = self.focus_cb.as_ref().cloned() {
            Ok(cb(self))
        } else {
            Ok(EventResult::consumed())
        }
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated || self.view.needs_relayout()
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        let title_req = if let Some(title) = self.title.as_ref() {
            (
                title.width() + 2, // Left and right margins
                1,
            )
        } else {
            (0, 0)
        };
        let padding_req = (2, 2); // inset plus border
        // See note in `draw` for rationale for storing view size
        self.view_size = self.view.required_size(req.saturating_sub((2, 0)));
        self.view_size + title_req + padding_req
    }

    fn layout(&mut self, size: Vec2) {
        let title = if self.title.is_some() { (0, 1) } else { (0, 0) };
        let padding = (2, 2);
        self.view
            .layout(size.saturating_sub(title).saturating_sub(padding));
        self.invalidated = false;
    }

    fn draw(&self, printer: &Printer) {
        // Drawing scheme uses cached self.view_size as a more reliable
        // extent of the panel size. `printer.size.y` can be 1 space
        // greater than it should be based on `self.required_size`,
        // which results in an extra space in the fill/border rendering.
        // As a workaround, we draw the fill/border based on the view
        // size (see below).

        printer.with_style(self.color, |printer| {
            let title_y;
            if let Some(title) = self.title.as_ref() {
                // Title fill
                printer.print_rect(
                    Rect::from_size((0, 0), (title.width() + 2, 1)), // 2 = title margins
                    " ",
                );
                // Title text
                printer.print_styled((1, 0), title); // 1 = left margin
                title_y = 1;
            } else {
                title_y = 0;
            }

            // Body fill and border
            {
                let printer = printer.offset((0, title_y));
                let fill_x = printer.size.x;
                let fill_y = self.view_size.y + 2; // Using view size for y (see above)

                // Fill
                printer.print_rect(Rect::from_size((0, 0), (fill_x, fill_y)), " ");

                // Border
                printer.with_style(
                    if printer.focused {
                        self.focus_border
                    } else {
                        self.border
                    },
                    |printer| {
                        // Top
                        printer.print_hline((1, 0), fill_x.saturating_sub(2), "─");
                        // Bottom
                        printer.print_hline(
                            (1, fill_y.saturating_sub(1)),
                            fill_x.saturating_sub(2),
                            "─",
                        );
                        // Left
                        printer.print_vline((0, 1), fill_y.saturating_sub(2), "│");
                        // Right
                        printer.print_vline(
                            (fill_x.saturating_sub(1), 1),
                            fill_y.saturating_sub(2),
                            "│",
                        );
                        // Corners
                        printer.print((0, 0), "╭");
                        printer.print((fill_x.saturating_sub(1), 0), "╮");
                        printer.print((0, fill_y.saturating_sub(1)), "╰");
                        printer.print((fill_x.saturating_sub(1), fill_y.saturating_sub(1)), "╯");
                    },
                );
            }

            // Body content (view)
            {
                let printer = printer.offset((2, title_y + 1)).shrinked((2, 0));
                self.view.draw(&printer);
            }
        });
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let inset = if self.title.is_some() {
            (1, 2) // Padding + title height
        } else {
            (1, 1) // Padding
        };
        self.view.on_event(event.relativized(inset))
    }
}
