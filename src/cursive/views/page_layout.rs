#![allow(dead_code)]

/// Event fired when the view is about to lose focus.
use cursive::{
    Printer, Rect, Vec2, With, XY,
    direction::{self, Direction, Orientation},
    event::{AnyCb, Event, EventResult, Key},
    view::{CannotFocus, IntoBoxedView, Selector, SizeCache, View, ViewNotFound},
};
use std::cmp::min;
use std::ops::Deref;

/// Simple vertical linear layout.
///
/// Makes no attempt to fit children and so is more performance than
/// LinearLayout.
pub struct PageLayout {
    children: Vec<Child>,
    focus: usize,
    cache: Option<XY<SizeCache>>,
}

struct Child {
    view: Box<dyn View>,
    required_size: Vec2,
    last_size: Vec2,
}

impl Child {
    fn required_size(&mut self, req: Vec2) -> Vec2 {
        self.required_size = self.view.required_size(req);
        self.required_size
    }

    fn layout(&mut self, size: Vec2) {
        self.last_size = size;
        self.view.layout(size);
    }

    fn as_view(&self) -> &dyn View {
        &*self.view
    }
}

struct ChildIterator<I> {
    inner: I,
    offset: usize,
    available: usize,
}

struct ChildItem<T> {
    child: T,
    offset: usize,
    height: usize,
}

impl<T> ChildIterator<T> {
    fn new(inner: T, available: usize) -> Self {
        ChildIterator {
            inner,
            available,
            offset: 0,
        }
    }
}

impl<T: Deref<Target = Child>, I: Iterator<Item = T>> Iterator for ChildIterator<I> {
    type Item = ChildItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|child| {
            let offset = self.offset;
            let length = min(self.available, child.required_size.y);
            self.available = self.available.saturating_sub(length);
            self.offset += length;
            ChildItem {
                child,
                offset,
                height: length,
            }
        })
    }
}

fn cap<'a, I: Iterator<Item = &'a mut usize>>(iter: I, max: usize) {
    let mut available = max;
    for item in iter {
        if *item > available {
            *item = available;
        }

        available -= *item;
    }
}

impl PageLayout {
    pub fn new() -> Self {
        PageLayout {
            children: Vec::new(),
            focus: 0,
            cache: None,
        }
    }

    /// Adds a child to the layout.
    ///
    /// Chainable variant.
    #[must_use]
    pub fn child<V: IntoBoxedView + 'static>(self, view: V) -> Self {
        self.with(|s| s.add_child(view))
    }

    /// Adds a child to the layout.
    pub fn add_child<V: IntoBoxedView + 'static>(&mut self, view: V) {
        self.children.push(Child {
            view: view.into_boxed_view(),
            required_size: Vec2::zero(),
            last_size: Vec2::zero(),
        });
        self.invalidate();
    }

    /// Inserts a child at the given position.
    ///
    /// # Panics
    ///
    /// Panics if `i > self.len()`.
    pub fn insert_child<V: IntoBoxedView + 'static>(&mut self, i: usize, view: V) {
        self.children.insert(
            i,
            Child {
                view: view.into_boxed_view(),
                required_size: Vec2::zero(),
                last_size: Vec2::zero(),
            },
        );
        self.invalidate();
    }

    /// Swaps two children.
    pub fn swap_children(&mut self, i: usize, j: usize) {
        self.children.swap(i, j);
    }

    /// Returns the number of children.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns `true` if this view has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Returns index of focused inner view
    pub fn get_focus_index(&self) -> usize {
        self.focus
    }

    /// Attempts to set the focus on the given child.
    ///
    /// Returns `Err(ViewNotFound)` if `index >= self.len()`, or if the view at the
    /// given index does not accept focus.
    pub fn set_focus_index(&mut self, index: usize) -> Result<EventResult, ViewNotFound> {
        self.children
            .get_mut(index)
            .and_then(|child| child.view.take_focus(Direction::none()).ok())
            .map(|res| res.and(self.set_focus_unchecked(index)))
            .ok_or(ViewNotFound)
    }

    fn set_focus_unchecked(&mut self, index: usize) -> EventResult {
        if index != self.focus {
            let result = self.children[self.focus].view.on_event(Event::FocusLost);
            self.focus = index;
            result
        } else {
            EventResult::Consumed(None)
        }
    }

    // Invalidate the view, to request a layout next time !!!
    fn invalidate(&mut self) {
        self.cache = None;
    }

    /// Returns a reference to a child.
    pub fn get_child(&self, i: usize) -> Option<&dyn View> {
        self.children.get(i).map(|child| &*child.view)
    }

    /// Returns a mutable reference to a child.
    pub fn get_child_mut(&mut self, i: usize) -> Option<&mut dyn View> {
        // Anything could happen to the child view, so bust the cache.
        self.invalidate();
        self.children.get_mut(i).map(|child| &mut *child.view)
    }

    /// Removes all children from this view.
    pub fn clear(&mut self) {
        self.invalidate();
        self.children.clear();
        self.focus = 0;
    }

    /// Removes a child.
    ///
    /// If `i` is within bounds, the removed child will be returned.
    pub fn remove_child(&mut self, i: usize) -> Option<Box<dyn View>> {
        if i < self.children.len() {
            // Any alteration means we should invalidate the cache.
            self.invalidate();

            // Keep the same view focused.
            if self.focus > i || (self.focus != 0 && self.focus == self.children.len() - 1) {
                self.focus -= 1;
            }

            // Return the wrapped view
            Some(self.children.remove(i).view)
        } else {
            // This includes empty list
            None
        }
    }

    /// Looks for the child containing a view with the given name.
    ///
    /// Returns `Some(i)` if `self.get_child(i)` has the given name, or
    /// contains a view with the given name.
    ///
    /// Returns `None` if the given name was not found.
    pub fn find_child_from_name(&mut self, name: &str) -> Option<usize> {
        let selector = Selector::Name(name);
        for (i, c) in self.children.iter_mut().enumerate() {
            let mut found = false;
            c.view.call_on_any(&selector, &mut |_| found = true);
            if found {
                return Some(i);
            }
        }
        None
    }

    // If the cache can be used, return the cached size.
    // Otherwise, return None.
    fn get_cache(&self, req: Vec2) -> Option<Vec2> {
        match self.cache {
            None => None,
            Some(ref cache) => {
                // Is our cache even valid?
                // Also, is any child invalidating the layout?
                if cache.zip_map(req, SizeCache::accept).both() && self.children_are_sleeping() {
                    Some(cache.map(|s| s.value))
                } else {
                    None
                }
            }
        }
    }

    fn children_are_sleeping(&self) -> bool {
        !self
            .children
            .iter()
            .map(Child::as_view)
            .any(View::needs_relayout)
    }

    /// Returns a mutable iterator.
    ///
    /// If `from_focus` is true, starts with starting with the child in
    /// focus, otherwise starts with the first or last child depending
    /// on `source`.
    fn iter_mut<'a>(
        &'a mut self,
        from_focus: bool,
        source: direction::Relative,
    ) -> Box<dyn Iterator<Item = (usize, &'a mut Child)> + 'a> {
        match source {
            direction::Relative::Front => {
                let start = if from_focus { self.focus } else { 0 };
                Box::new(self.children.iter_mut().enumerate().skip(start))
            }
            direction::Relative::Back => {
                let end = if from_focus {
                    self.focus + 1
                } else {
                    self.children.len()
                };
                Box::new(self.children[..end].iter_mut().enumerate().rev())
            }
        }
    }

    // Attempt to move the focus, coming from the given direction.
    //
    // Consumes the event if the focus was moved, otherwise ignores it.
    fn move_focus(&mut self, source: Direction) -> EventResult {
        source
            .relative(Orientation::Vertical)
            .and_then(|rel| {
                self.iter_mut(true, rel)
                    .skip(1)
                    .find_map(|p| try_focus(p, source))
            })
            .map_or(EventResult::Ignored, |(i, res)| {
                res.and(self.set_focus_unchecked(i))
            })
    }

    // Move the focus to the selected view if needed.
    //
    // Does nothing if the event is not a `MouseEvent`.
    fn check_focus_grab(&mut self, event: &Event) -> Option<EventResult> {
        if let Event::Mouse {
            offset,
            position,
            event,
        } = *event
        {
            if !event.grabs_focus() {
                return None;
            }

            let position = position.checked_sub(offset)?;

            // Iterate on the views and find the one
            // We need a mutable ref to call take_focus later on.
            for (i, item) in ChildIterator::new(
                self.children.iter_mut(),
                // TODO: get actual width (not super important)
                usize::MAX,
            )
            .enumerate()
            {
                // Get the child size:
                // this will give us the allowed window for a click.
                let child_size = item.child.last_size.y;

                if item.offset + child_size <= position.y {
                    continue;
                }

                return item
                    .child
                    .view
                    .take_focus(Direction::none())
                    .ok()
                    .map(|res| res.and(self.set_focus_unchecked(i)));
            }
        }
        None
    }
}

fn try_focus((i, child): (usize, &mut Child), source: Direction) -> Option<(usize, EventResult)> {
    child.view.take_focus(source).ok().map(|res| (i, res))
}

impl View for PageLayout {
    fn draw(&self, printer: &Printer) {
        for (i, item) in ChildIterator::new(self.children.iter(), printer.size.y).enumerate() {
            let printer = &printer
                .offset((0, item.offset))
                .cropped(item.child.last_size)
                .focused(i == self.focus);
            item.child.view.draw(printer);
        }
    }

    fn needs_relayout(&self) -> bool {
        if self.cache.is_none() {
            return true;
        }

        !self.children_are_sleeping()
    }

    fn layout(&mut self, size: Vec2) {
        if self.get_cache(size).is_none() {
            // Build cache
            self.required_size(size);
        }

        for item in ChildIterator::new(self.children.iter_mut(), size.y) {
            item.child.layout(Vec2::new(size.x, item.height));
        }
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        if let Some(size) = self.get_cache(req) {
            return size;
        }
        Orientation::Vertical.stack(self.children.iter_mut().map(|c| c.required_size(req)))
    }

    fn take_focus(&mut self, source: Direction) -> Result<EventResult, CannotFocus> {
        // In what order will we iterate on the children?
        let rel = source.relative(Orientation::Vertical);

        // We activate from_focus only if coming from the "sides".
        let focus_res = self
            .iter_mut(rel.is_none(), rel.unwrap_or(direction::Relative::Front))
            .find_map(|p| try_focus(p, source));

        if let Some((next_focus, res)) = focus_res {
            // No "FocusLost" here, since we didn't have focus before.
            self.focus = next_focus;
            Ok(res)
        } else {
            Err(CannotFocus)
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if self.is_empty() {
            return EventResult::Ignored;
        }

        let res = self
            .check_focus_grab(&event)
            .unwrap_or(EventResult::Ignored);

        let result = {
            let mut iterator = ChildIterator::new(self.children.iter_mut(), usize::MAX);
            let item = iterator.nth(self.focus).unwrap();
            item.child
                .view
                .on_event(event.relativized((0, item.offset)))
        };
        res.and(match result {
            EventResult::Ignored => match event {
                Event::Shift(Key::Tab) if self.focus > 0 => self.move_focus(Direction::back()),
                Event::Key(Key::Tab) if self.focus + 1 < self.children.len() => {
                    self.move_focus(Direction::front())
                }
                Event::Key(Key::Up) if self.focus > 0 => self.move_focus(Direction::down()),
                Event::Key(Key::Down) if self.focus + 1 < self.children.len() => {
                    self.move_focus(Direction::up())
                }
                _ => EventResult::Ignored,
            },
            res => res,
        })
    }

    fn call_on_any(&mut self, selector: &Selector, callback: AnyCb) {
        for child in &mut self.children {
            child.view.call_on_any(selector, callback);
        }
    }

    fn focus_view(&mut self, selector: &Selector) -> Result<EventResult, ViewNotFound> {
        for (i, child) in self.children.iter_mut().enumerate() {
            if child.view.focus_view(selector).is_ok() {
                return Ok(self.set_focus_unchecked(i));
            }
        }

        Err(ViewNotFound)
    }

    fn important_area(&self, size: Vec2) -> Rect {
        if self.is_empty() {
            // Return dummy area if we are empty.
            return Rect::from_size(Vec2::zero(), size);
        }

        // Pick the focused item, with its offset
        let item = {
            let mut iterator = ChildIterator::new(self.children.iter(), usize::MAX);
            iterator.nth(self.focus).unwrap()
        };

        // Make a vector offset from the scalar value
        let offset = Vec2::new(0, item.offset);

        // And ask the child its own area.
        let rect = item.child.view.important_area(item.child.last_size);

        // Add `offset` to the rect.
        rect + offset
    }
}

/*
#[crate::blueprint(LinearLayout::new(orientation))]
struct Blueprint {
    orientation: direction::Orientation,

    #[blueprint(foreach=add_child)]
    children: Vec<crate::views::BoxedView>,

    #[blueprint(
        set_focus_index,
        on_err="LinearLayout.focus cannot be larger than the number of views.",
    )]
    focus: Option<usize>,
}
*/

cursive::manual_blueprint!(LinearLayout, |config, context| {
    let orientation = match config.get("orientation") {
        Some(orientation) => context.resolve(orientation)?,
        None => direction::Orientation::Vertical,
    };

    let mut layout = LinearLayout::new(orientation);

    let children: Option<Vec<crate::views::BoxedView>> =
        context.resolve_as_config(&config["children"])?;
    if let Some(children) = children {
        for child in children {
            layout.add_child(child);
        }
    }

    if let Some(focus) = config.get("focus") {
        let focus = context.resolve(focus)?;
        layout
            .set_focus_index(focus)
            .map_err(|_| crate::builder::Error::InvalidConfig {
                message: "LinearLayout.focus cannot be larger than the number of views.".into(),
                config: config.clone(),
            })?;
    }

    Ok(layout)
});
