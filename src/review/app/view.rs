use std::collections::HashMap;

use cursive::{ScreenId, view::ViewWrapper, views::BoxedView, wrap_impl};

use crate::{
    cursive::views::ScreensView,
    review::{
        AppScreen,
        screens::{console::ConsoleScreen, dev::DevScreen, log::LogScreen, logs::LogsScreen},
    },
};

pub struct AppView {
    screen_stack: Vec<AppScreen>,
    screen_ids: HashMap<AppScreen, ScreenId>,
    inner: ScreensView,
}

impl AppView {
    pub fn new(log_dir: &str, dev_mode: bool) -> Self {
        let mut inner = ScreensView::new();
        let logs = inner.add_screen(BoxedView::new(Box::new(LogsScreen::new(log_dir))));
        let log = inner.add_screen(BoxedView::new(Box::new(LogScreen::new())));
        let console = inner.add_screen(BoxedView::new(Box::new(ConsoleScreen::new())));
        let dev = inner.add_screen(BoxedView::new(Box::new(DevScreen::new())));
        let screen_stack = if dev_mode {
            inner.set_active_screen(dev);
            vec![AppScreen::Dev]
        } else {
            inner.set_active_screen(logs);
            vec![AppScreen::Logs]
        };
        Self {
            screen_ids: HashMap::from([
                (AppScreen::Logs, logs),
                (AppScreen::Log, log),
                (AppScreen::Console, console),
                (AppScreen::Dev, dev),
            ]),
            screen_stack,
            inner,
        }
    }

    pub fn push_screen(&mut self, screen: AppScreen) {
        self.inner.set_active_screen(self.screen_ids[&screen]);
        self.screen_stack.push(screen);
    }

    pub fn pop_screen(&mut self) -> Option<AppScreen> {
        // Always maintain one view on stack
        if self.screen_stack.len() == 1 {
            return None;
        }
        let popped = self.screen_stack.pop().expect("see check above");
        let next = self.screen_stack.last().expect("see check above");
        self.inner.set_active_screen(self.screen_ids[next]);
        Some(popped)
    }

    pub fn with_screen<F, V: 'static, T>(&mut self, screen: AppScreen, f: F) -> Option<T>
    where
        F: Fn(&mut V) -> T + Send + Sync + 'static,
    {
        let screen_id = self.screen_ids.get(&screen)?;
        let screen = self.inner.get_screen_mut(*screen_id)?;
        let view = screen.downcast_mut::<V>()?;
        Some(f(view))
    }
}

impl ViewWrapper for AppView {
    wrap_impl!(self.inner: ScreensView);
}
