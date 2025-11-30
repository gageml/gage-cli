use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use cursive::{
    Cursive, Printer, Rect, Vec2, View,
    align::HAlign,
    direction::Direction,
    event::{Callback, Event, EventResult, Key, MouseButton, MouseEvent},
    theme::PaletteStyle,
    utils::markup::StyledString,
    view::{CannotFocus, Finder, Nameable, ViewWrapper},
    views::{HideableView, LinearLayout, ViewRef},
    wrap_impl,
};

use crate::cursive::views::PageLayout;

struct State {
    expanded: bool,
    invalidated: bool,
    toggle_armed: Option<Instant>,
}

impl State {
    fn new() -> Self {
        Self {
            expanded: false,
            invalidated: true,
            toggle_armed: None,
        }
    }

    fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
        self.invalidated = true;
    }

    fn reset_invalidated(&mut self) {
        self.invalidated = false;
    }

    fn changed_expanded(&self) -> Option<bool> {
        if self.invalidated {
            Some(self.expanded)
        } else {
            None
        }
    }
}

pub struct ToggleButton {
    label: StyledString,
    callback: Callback,
    last_size: Vec2,
    invalidated: bool,
}

impl ToggleButton {
    pub fn new<F, S>(label: S, cb: F) -> Self
    where
        F: 'static + Fn(&mut Cursive) + Send + Sync,
        S: Into<StyledString>,
    {
        let label = StyledString::concatenate([
            StyledString::plain(" "),
            label.into(),
            StyledString::plain(" "),
        ]);
        Self {
            label,
            callback: Callback::from_fn(cb),
            last_size: Vec2::zero(),
            invalidated: true,
        }
    }

    pub fn set_label<S>(&mut self, label: S)
    where
        S: Into<StyledString>,
    {
        self.label = StyledString::concatenate([
            StyledString::plain(" "),
            label.into(),
            StyledString::plain(" "),
        ]);
        self.invalidated = true;
    }

    fn req_size(&self) -> Vec2 {
        Vec2::new(self.label.width(), 1)
    }
}

impl View for ToggleButton {
    fn draw(&self, printer: &Printer) {
        if printer.size.x == 0 {
            return;
        }

        let style = if printer.focused {
            PaletteStyle::Highlight
        } else {
            PaletteStyle::Primary
        };

        let offset = HAlign::Center.get_offset(self.label.width(), printer.size.x);

        printer.with_style(style, |printer| {
            printer.print_styled((offset, 0), &self.label);
        });
    }

    fn layout(&mut self, size: Vec2) {
        self.last_size = size;
        self.invalidated = false;
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.req_size()
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let width = self.label.width();
        let self_offset = HAlign::Center.get_offset(width, self.last_size.x);
        match event {
            Event::Key(Key::Enter) => EventResult::Consumed(Some(self.callback.clone())),
            Event::Mouse {
                event: MouseEvent::Release(MouseButton::Left),
                position,
                offset,
            } if position.fits_in_rect(offset + (self_offset, 0), self.req_size()) => {
                EventResult::Consumed(Some(self.callback.clone()))
            }
            _ => EventResult::Ignored,
        }
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::consumed())
    }

    fn important_area(&self, view_size: Vec2) -> Rect {
        let width = self.label.width();
        let offset = HAlign::Center.get_offset(width, view_size.x);
        Rect::from_size((offset, 0), (width, 1))
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated
    }
}

pub struct ToggleView<V> {
    state: Arc<Mutex<State>>,
    label: StyledString,
    expanded_label: Option<StyledString>,
    inner: PageLayout,
    _marker: std::marker::PhantomData<V>,
}

impl<V: View> ToggleView<V> {
    pub fn new<S: Into<StyledString>>(label: S, view: V) -> Self {
        let state = Arc::new(Mutex::new(State::new()));
        let inner = {
            let state = Arc::clone(&state);
            PageLayout::new()
                .child(
                    LinearLayout::horizontal().child(
                        ToggleButton::new("", move |_| {
                            let mut state = state.lock().unwrap();
                            state.toggle_armed = Some(Instant::now());
                            state.toggle_expanded();
                        })
                        .with_name("toggle"),
                    ),
                )
                .child(HideableView::new(view).hidden().with_name("body"))
        };
        Self {
            state,
            label: label.into(),
            expanded_label: None,
            inner,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.set_expanded(expanded);
        self
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        let mut state = self.state.lock().unwrap();
        if state.expanded != expanded {
            state.invalidated = true;
        }
        state.expanded = expanded;
    }

    pub fn expanded_label<S: Into<StyledString>>(mut self, label: S) -> Self {
        self.set_expanded_label(label);
        self
    }

    pub fn set_expanded_label<S: Into<StyledString>>(&mut self, label: S) {
        self.expanded_label = Some(label.into());
    }

    fn toggle(&mut self) -> ViewRef<ToggleButton> {
        self.find_name("toggle").unwrap()
    }

    fn body(&mut self) -> ViewRef<HideableView<V>> {
        self.find_name("body").unwrap()
    }
}

impl<V: View + 'static> ViewWrapper for ToggleView<V> {
    wrap_impl!(self.inner: PageLayout);

    fn wrap_required_size(&mut self, req: Vec2) -> cursive::Vec2 {
        // Only update components if expanded state changed
        let changed_expanded = self.state.lock().unwrap().changed_expanded();
        if let Some(expanded) = changed_expanded {
            // Toggle button label
            let toggle_glyph = if expanded { "▼ " } else { "▶ " };
            let label = if expanded {
                self.expanded_label
                    .clone()
                    .unwrap_or_else(|| self.label.clone())
            } else {
                self.label.clone()
            };
            self.toggle().set_label(StyledString::concatenate([
                StyledString::plain(toggle_glyph),
                label,
            ]));

            // Show/hide body
            self.body().set_visible(expanded);
        }

        self.inner.required_size(req)
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.inner.layout(size);
        self.state.lock().unwrap().reset_invalidated();
    }

    fn wrap_needs_relayout(&self) -> bool {
        self.state.lock().unwrap().invalidated || self.inner.needs_relayout()
    }

    fn wrap_draw(&self, printer: &Printer) {
        self.inner.draw(printer);
        let mut state = self.state.lock().unwrap();
        if let Some(toggle_armed) = state.toggle_armed {
            log::info!(
                "Toggle draw: {}",
                Instant::now().duration_since(toggle_armed).as_millis()
            );
            state.toggle_armed = None;
        }
    }
}
