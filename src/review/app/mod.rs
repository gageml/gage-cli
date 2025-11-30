#[derive(Clone)]
pub struct App;

#[derive(Hash, PartialEq, Eq)]
pub enum AppScreen {
    Logs,
    Log,
    Console,
    Dev,
}

impl App {
    pub fn push_screen(siv: &mut Cursive, screen: AppScreen) {
        siv.screen_mut()
            .get_mut(LayerPosition::FromFront(0))
            .unwrap()
            .downcast_mut::<AppView>()
            .unwrap()
            .push_screen(screen);
    }

    pub fn pop_screen(siv: &mut Cursive) {
        siv.screen_mut()
            .get_mut(LayerPosition::FromFront(0))
            .unwrap()
            .downcast_mut::<AppView>()
            .unwrap()
            .pop_screen();
    }

    pub fn with_screen<F, V: 'static, T>(siv: &mut Cursive, screen: AppScreen, f: F) -> Option<T>
    where
        F: Fn(&mut V) -> T + Send + Sync + 'static,
    {
        siv.screen_mut()
            .get_mut(LayerPosition::FromBack(0))
            .unwrap()
            .downcast_mut::<AppView>()
            .unwrap()
            .with_screen(screen, f)
    }

    pub fn quit(siv: &mut Cursive) {
        siv.quit();
    }
}

use std::{
    panic,
    sync::{Arc, Mutex},
};

use cursive::{Cursive, views::LayerPosition};

use crate::{
    logger,
    result::Result,
    review::{app::view::AppView, theme},
};

mod help;
mod view;

pub use help::Help;

pub fn run(log_dir: &str, dev_mode: bool) -> Result<()> {
    // Set panic hook to capture panic details
    let panic: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    set_panic_hook(Arc::clone(&panic));

    let result = panic::catch_unwind(|| {
        // Switch logs to cursive
        logger::use_cursive();

        // Init cursive
        let mut siv = cursive::default();
        siv.set_theme(theme::default());

        // App view
        siv.add_fullscreen_layer(AppView::new(log_dir, dev_mode));

        // Run app - blocks until cursive quit
        siv.run();
    });

    // Restore default logger
    logger::use_default();

    if result.is_err() {
        let panic = panic
            .lock()
            .unwrap()
            .clone()
            .expect("Set via panic::set_hook above");
        eprintln!("PANIC! {panic}");
    }

    Ok(())
}

fn set_panic_hook(panic: Arc<Mutex<Option<String>>>) {
    panic::set_hook(Box::new(move |info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            Some(s.to_string())
        } else {
            info.payload().downcast_ref::<String>().cloned()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "Unknown".into());
        panic.lock().unwrap().replace(if let Some(val) = payload {
            format!("{val:?} {location}")
        } else {
            location
        });
    }));
}
