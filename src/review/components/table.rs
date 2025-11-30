use std::{
    cmp::{self, Ordering},
    sync::Arc,
    time::SystemTime,
};

use cursive::{
    Printer, Rect, Vec2, View, XY,
    direction::Direction,
    event::{Event, EventResult, Key, MouseButton, MouseEvent},
    theme::{BaseColor, ColorStyle, ColorType, Effect, Style},
    utils::markup::StyledString,
    view::CannotFocus,
};
use itertools::Itertools;

use crate::{
    cursive::{PrinterExt, view::scroll},
    impl_scroller,
};

pub struct TableView<T, C> {
    cols: Vec<TableCol<C>>,
    items: Vec<T>,
    rendered: Option<Vec<usize>>,
    sort: Option<Sort<C>>,
    active: Active,
    active_click_trigger: Option<(usize, SystemTime)>,
    scroll_core: scroll::Core,
    page_size: usize,
    screen_w: usize,
    on_select: Option<Arc<OnSelect<T>>>,
    empty_msg: StyledString,
}

pub type OnSelect<T> = dyn Fn(&T) -> EventResult + Send + Sync;

#[derive(Debug)]
struct Active {
    y: Option<usize>,
    item: Option<usize>,
}

impl Active {
    fn none() -> Self {
        Self {
            y: None,
            item: None,
        }
    }

    fn from_y(y: usize) -> Self {
        Self {
            y: Some(y),
            item: None,
        }
    }

    fn from_item(item: usize) -> Self {
        Self {
            y: None,
            item: Some(item),
        }
    }

    fn from_click(y: usize, rendered: &[usize]) -> Self {
        Self {
            y: Some(y),
            item: rendered.get(y).copied(),
        }
    }

    fn down(&mut self, n: usize, rendered: &[usize]) {
        if !rendered.is_empty() {
            let next_y = cmp::min(self.y.unwrap_or(0) + n, rendered.len() - 1);
            self.item = Some(*rendered.get(next_y).unwrap());
            self.y = Some(next_y);
        }
    }

    fn up(&mut self, n: usize, rendered: &[usize]) {
        if !rendered.is_empty() {
            let next_y = self
                .y
                .map(|y| y.saturating_sub(n))
                .unwrap_or_else(|| rendered.len() - 1);
            self.item = Some(*rendered.get(next_y).unwrap());
            self.y = Some(next_y);
        }
    }
}

pub struct TableCol<C> {
    inner: C,
    label: String,
    width: TableColWidth,
    calc_width: usize,
}

pub enum TableColWidth {
    Unset,
    // Idea is to support fixed, max, min, etc.
}

pub trait TableColExt<T> {
    fn fmt(&self, item: &T) -> impl Into<StyledString>;
    fn cmp(&self, lhs: &T, rhs: &T) -> Ordering;
}

#[derive(Clone)]
pub struct Sort<C>(C, SortDir);

impl<C> Sort<C>
where
    C: Clone,
{
    // pub fn asc(c: C) -> Self {
    //     Self(c, SortDir::Asc)
    // }

    pub fn desc(c: C) -> Self {
        Self(c, SortDir::Desc)
    }

    pub fn reversed(&self) -> Self {
        Self(self.0.clone(), self.1.reverse())
    }

    pub fn col(&self) -> &C {
        &self.0
    }

    pub fn dir(&self) -> &SortDir {
        &self.1
    }
}

pub trait DefaultSortDir {
    fn default_sort(&self) -> SortDir {
        SortDir::Asc
    }
}

impl<C> Sort<C>
where
    C: DefaultSortDir,
{
    pub fn default(c: C) -> Self {
        let sort = c.default_sort();
        Self(c, sort)
    }
}

#[derive(Clone)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

impl<T, C> TableView<T, C>
where
    C: Clone,
    C: TableColExt<T>,
{
    pub fn new() -> Self {
        Self {
            cols: Vec::new(),
            items: Vec::new(),
            rendered: None,
            sort: None,
            active: Active::none(),
            active_click_trigger: None,
            scroll_core: scroll::Core::new(),
            page_size: 1,
            screen_w: 0,
            on_select: None,
            empty_msg: StyledString::styled(
                "Empty",
                Style::from(Effect::Italic).combine(Effect::Dim),
            ),
        }
    }

    pub fn col(mut self, col: C, label: impl Into<String>) -> Self {
        self.cols.push(TableCol {
            inner: col,
            label: label.into(),
            width: TableColWidth::Unset,
            calc_width: 0,
        });
        self
    }

    pub fn sort(mut self, sort: Sort<C>) -> Self {
        self.set_sort(sort);
        self
    }

    pub fn set_sort(&mut self, sort: Sort<C>) {
        self.sort = Some(sort);
        self.invalidate_rendered();
    }

    pub fn get_sort(&self) -> Option<&Sort<C>> {
        self.sort.as_ref()
    }

    // pub fn items(mut self, items: Vec<T>) -> Self {
    //     self.set_items(items);
    //     self
    // }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.active = if self.items.is_empty() {
            Active::none()
        } else {
            Active::from_y(0)
        };
        self.invalidate_rendered();
    }

    pub fn set_items_with_active<F>(&mut self, items: Vec<T>, cb: F)
    where
        F: Fn(&T) -> bool,
    {
        self.items = items;
        if let Some((i, _)) = self.items.iter().enumerate().find(|(_, item)| cb(item)) {
            self.active = Active::from_item(i)
        } else {
            self.active = Active::none()
        };
        self.invalidate_rendered();
    }

    // pub fn get_items(&self) -> &Vec<T> {
    //     &self.items
    // }

    fn invalidate_rendered(&mut self) {
        self.rendered = None;
    }

    pub fn get_active(&self) -> Option<&T> {
        self.active.item.map(|i| self.items.get(i))?
    }

    pub fn empty_msg<S>(mut self, msg: S) -> Self
    where
        S: Into<StyledString>,
    {
        self.set_empty_msg(msg);
        self
    }

    pub fn set_empty_msg<S>(&mut self, msg: S)
    where
        S: Into<StyledString>,
    {
        self.empty_msg = msg.into();
        self.invalidate_rendered();
    }

    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> EventResult + Send + Sync + 'static,
    {
        self.set_on_select(f);
        self
    }

    pub fn set_on_select<F>(&mut self, f: F)
    where
        F: Fn(&T) -> EventResult + Send + Sync + 'static,
    {
        self.on_select = Some(Arc::new(f));
    }

    fn build_rendered(&mut self) {
        let mut rendered = (0..self.items.len()).collect_vec();

        // Sort
        if let Some(sort) = self.sort.as_ref() {
            rendered.sort_by(|lhs, rhs| {
                let order = sort.col().cmp(&self.items[*lhs], &self.items[*rhs]);
                match sort.dir() {
                    SortDir::Asc => order,
                    SortDir::Desc => order.reverse(),
                }
            })
        }

        self.rendered = Some(rendered);
    }

    fn set_active(&mut self) {
        let rendered = self.expect_rendered();
        if let Some(item) = self.active.item.as_ref() {
            // item selected - use to find y from rendered
            self.active.y = rendered
                .iter()
                .enumerate()
                .find(|(_, i)| *i == item)
                .map(|(y, _)| y);
        } else {
            // Use current y, set in bounds, or use 0 if we have rendered items
            if let Some(y) = self
                .active
                .y
                .as_ref()
                .map(|&y| cmp::min(y, rendered.len() - 1))
                .or(if rendered.is_empty() { None } else { Some(0) })
            {
                // Have y - use to resolve item
                self.active.item = rendered.get(y).copied();
                self.active.y = Some(y);
            } else {
                // Rendered is empty - nothing active
                assert!(rendered.is_empty());
                assert!(self.active.item.is_none());
                self.active.y = None;
            }
        }

        if let Some(&y) = self.active.y.as_ref() {
            self.scroll_core.scroll_to_y(y);
        }
    }

    fn expect_rendered(&self) -> &[usize] {
        self.rendered.as_ref().unwrap()
    }
}

impl_scroller!(TableView < T, H > ::scroll_core);

impl<T, C> View for TableView<T, C>
where
    T: Sync + Send + 'static,
    C: TableColExt<T>,
    C: DefaultSortDir,
    C: Sync + Send + 'static,
    C: PartialEq,
    C: Clone,
{
    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::consumed())
    }

    fn layout(&mut self, size: Vec2) {
        self.page_size = cmp::max(1, size.y.saturating_sub(5)); // Header + borders + 1
        self.screen_w = size.x;

        if self.rendered.is_none() {
            self.build_rendered();
            self.set_active();
        }

        scroll::layout(
            self,
            size.saturating_sub((2, 4)),
            true,
            Self::layout_inner,
            Self::required_size_inner,
        );
    }

    fn needs_relayout(&self) -> bool {
        self.rendered.is_none()
    }

    fn draw(&self, printer: &Printer) {
        self.draw_border(printer);
        self.draw_col_headers(&printer.offset((1, 0)).shrinked((2, 0)));
        if self.expect_rendered().is_empty() {
            self.draw_empty(&printer.offset((1, 3)).shrinked((1, 1)));
        } else {
            scroll::draw(
                self,
                &printer.offset((1, 3)).shrinked((1, 1)),
                Self::draw_inner,
            );
            self.draw_fill_to_bottom(&printer.offset((1, 0)).shrinked((2, 0)));
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if let Event::Mouse {
            position: XY { x, y: 2 },
            offset: _,
            event: MouseEvent::Press(MouseButton::Left),
        } = event
        {
            self.on_click_header(x)
        } else {
            scroll::on_event(
                self,
                event.relativized((1, 3)),
                Self::on_event_inner,
                Self::important_area_inner,
            )
        }
    }
}

// Layout/draw
impl<T, C> TableView<T, C>
where
    C: TableColExt<T>,
    C: PartialEq,
    C: Clone,
{
    fn layout_inner(&mut self, _: Vec2) {
        // Column widths
        for col in self.cols.iter_mut() {
            match col.width {
                TableColWidth::Unset => {
                    col.calc_width = cmp::max(
                        self.items
                            .iter()
                            .map(|item| col.inner.fmt(item).into().width())
                            .max()
                            .unwrap_or_default(),
                        col.label.len() + 2, // space for sort indicator
                    );
                }
            }
        }
    }

    fn required_size_inner(&mut self, req: Vec2) -> Vec2 {
        Vec2::new(req.x, cmp::max(self.expect_rendered().len(), 1)) // Use 1 for Empty message
    }

    fn draw_border(&self, printer: &Printer) {
        // Bounding box
        printer.print_box_rounded((0, 0), printer.size, false);

        // Lower header border
        printer.print((0, 2), "├");
        printer.print_hline((1, 2), printer.size.x - 2, "─");
        printer.print((printer.size.x - 1, 2), "┤");
    }

    fn draw_col_headers(&self, printer: &Printer) {
        let mut x: usize = 0;
        let is_empty = self.expect_rendered().is_empty();
        for (i, col) in self.cols.iter().enumerate() {
            x += 1; // Left padding

            // Header label
            printer.print_styled((x, 1), &StyledString::styled(&col.label, Effect::Dim));

            // Sort indicator
            if let Some(Sort(sort_col, sort_dir)) = self.sort.as_ref()
                && sort_col == &col.inner
            {
                printer.print_styled(
                    (x + col.calc_width - 1, 1),
                    &StyledString::styled(
                        match sort_dir {
                            SortDir::Desc => "↑",
                            SortDir::Asc => "↓",
                        },
                        Effect::Dim,
                    ),
                );
            }

            // Inner border
            if i < self.cols.len() - 1 {
                x += col.calc_width + 1; // Header width + right padding
                printer.print((x, 0), "┬");
                printer.print((x, 1), "│");
                printer.print((x, 2), if is_empty { "┴" } else { "┼" });
                x += 1; // Border width
            }
        }
    }

    fn draw_empty(&self, printer: &Printer) {
        let x = printer.size.x.saturating_sub(self.empty_msg.width()) / 2;
        let y = printer.size.y.saturating_sub(1) / 2;
        printer.print_styled((x, y), &self.empty_msg);
    }

    fn draw_inner(&self, printer: &Printer) {
        for (y, &i) in self.expect_rendered().iter().enumerate() {
            let item = &self.items[i];

            // Active item based on active.y
            let active = self.active.y.map(|active| active == y).unwrap_or(false);

            // Background color for active item
            let bg = if active {
                ColorType::Color(BaseColor::Blue.dark())
            } else {
                ColorType::InheritParent
            };

            // Highlight active item
            if active {
                printer.with_color(ColorStyle::back(bg), |printer| {
                    printer.print_hline((0, y), printer.size.x, " ");
                });
            }

            // Draw each item col values and inner borders
            printer.with_color(ColorStyle::back(bg), |printer| {
                let mut x = 0;
                for (col_index, col) in self.cols.iter().enumerate() {
                    // Item col value
                    x += 1; // Left padding
                    let val = col.inner.fmt(item).into();
                    let width = col.calc_width;
                    printer
                        .offset((x, y))
                        .cropped((width, 1))
                        .print_styled((0, 0), &val);

                    // If value truncated, draw ellipsis
                    if val.width() > width {
                        printer.print((x + width, y), "…");
                    }

                    // Inner border
                    if col_index < self.cols.len() - 1 {
                        x += width + 1; // Width + right padding
                        printer.print((x, y), "│");
                        x += 1; // Border
                    }
                }
            });
        }
    }

    fn draw_fill_to_bottom(&self, printer: &Printer) {
        let mut x: usize = 0;
        let fill_y = self.expect_rendered().len() + 3; // One iter per line + header height
        for (i, col) in self.cols.iter().enumerate() {
            x += 1; // Left padding

            // Inner border fill
            if i < self.cols.len() - 1 {
                x += col.calc_width + 1; // Header width + right padding
                // Fill to bottom - printer y - fill y - bottom border
                let fill_h = printer.size.y.saturating_sub(fill_y).saturating_sub(1);
                if fill_h > 0 {
                    printer.print_vline((x, fill_y), fill_h, "│");
                }

                // Bottom border
                printer.print((x, printer.size.y - 1), "┴");
                x += 1; // Border width
            }
        }
    }

    fn important_area_inner(&self, size: Vec2) -> Rect {
        Rect::from_size((0, self.active.y.unwrap_or(0)), (size.x, 1))
    }
}

// Events
impl<T, C> TableView<T, C>
where
    C: TableColExt<T>,
    C: PartialEq,
    C: Clone,
    C: DefaultSortDir,
{
    fn on_click_header(&mut self, x: usize) -> EventResult {
        let mut start = 1; // Left border
        for (i, col) in self.cols.iter().enumerate() {
            let end = if i < self.cols.len() - 1 {
                start + col.calc_width + 1 // Right space
            } else {
                self.screen_w - 2 // Left and right borders
            };
            if x >= start && x <= end {
                if let Some(Sort(sort_col, sort_dir)) = self.sort.take()
                    && sort_col == col.inner
                {
                    // Clicked current sort col - toggle direction
                    self.set_sort(Sort(sort_col, sort_dir.reverse()));
                } else {
                    // Clicked new sort col - use default
                    self.set_sort(Sort::default(col.inner.clone()));
                }
                return EventResult::consumed();
            }
            start = end + 2; // One past inner border
        }
        EventResult::Ignored
    }

    fn on_event_inner(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(Key::Down) => self.on_down(1),
            Event::Key(Key::Up) => self.on_up(1),
            Event::Key(Key::PageDown) => self.on_down(self.page_size),
            Event::Key(Key::PageUp) => self.on_up(self.page_size),
            Event::Key(Key::Enter) => self.on_active_select(),
            Event::Mouse {
                position,
                offset,
                event: MouseEvent::Press(MouseButton::Left),
            } => self.on_click_inner(position, offset),
            _ => EventResult::Ignored,
        }
    }

    fn on_down(&mut self, n: usize) -> EventResult {
        if let Some(rendered) = self.rendered.as_ref() {
            self.active.down(n, rendered);
        }
        EventResult::consumed()
    }

    fn on_up(&mut self, n: usize) -> EventResult {
        if let Some(rendered) = self.rendered.as_ref() {
            self.active.up(n, rendered);
        }
        EventResult::consumed()
    }

    fn on_active_select(&mut self) -> EventResult {
        if let Some(cb) = self.on_select.as_ref()
            && let Some(active) = self.get_active()
        {
            cb(active)
        } else {
            EventResult::consumed()
        }
    }

    fn on_click_inner(&mut self, position: Vec2, offset: Vec2) -> EventResult {
        let trigger = self.active_click_trigger.take();
        if let Some(XY { y, .. }) = position.checked_sub(offset)
            && let Some(rendered) = self.rendered.as_ref()
            && y < rendered.len()
        {
            // Set active for y
            self.active = Active::from_click(y, rendered);

            // Check for double click
            let now = SystemTime::now();
            if let Some((last_y, last_time)) = trigger
                && last_y == y
                && now.duration_since(last_time).unwrap().as_millis() < 500
            {
                return self.on_active_select();
            }

            // Arm for double click
            self.active_click_trigger = Some((y, now));
        }
        EventResult::consumed()
    }
}
