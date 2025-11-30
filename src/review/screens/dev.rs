use std::time::Instant;

use cursive::{
    Cursive, Printer, Vec2, View,
    event::{Event, EventResult},
    view::{Resizable, ViewWrapper},
    views::ResizedView,
    wrap_impl,
};

use crate::{
    cursive::{
        view::{Padding, Scrollable},
        views::{PageLayout, PlainTextView, ScrollView},
    },
    review::{App, AppScreen, components::toggle::ToggleView, dialogs::help::HelpDialog},
};

// Timings
//
// 1. LinearLayout + TextView
//
//    10x10
//      Outer required_size: 363
//      Outer wrap_layout: 411
//      Toggle draw: 777
//      Outer draw: 1
//
//    20x20
//      Outer required_size: 1391
//      Outer wrap_layout: 1635
//      Toggle draw: 3029
//      Outer draw: 2
//
//    This is a clear pathological case. With only 400 items (20x20) a
//    toggle takes 3 seconds.
//
// 2. LinearLayout + PlainTextView
//
//    10x10
//      Outer required_size: 2
//      Outer wrap_layout: 5
//      Toggle draw: 11
//      Outer draw: 1
//
//    20x20
//      Outer required_size: 5
//      Outer wrap_layout: 17
//      Toggle draw: 25
//      Outer draw: 2
//
//    40x40
//      Outer required_size: 20
//      Outer wrap_layout: 73
//      Toggle draw: 96
//      Outer draw: 5
//
//    100x100
//      Outer required_size: 130
//      Outer wrap_layout: 442
//      Toggle draw: 576
//      Outer draw: 26
//
//    This confirms that PlainTextView, which doesn't wrap text, is a
//    big win for performance. The toggle still takes half a second at
//    10,000 items.
//
// 3. LinearLayout + PlainTextView - ScrollView
//
//    100x100
//      Outer required_size: 70
//      Outer wrap_layout: 161
//      Toggle draw: 236
//      Outer draw: 25
//
//    Interestingly, the scroll view contributes substantially to the
//    timings. I wonder if there are other containers that introduce
//    high latency and we're not testing them here. This could have a
//    multiplicative affect, which would explain the sudden horrendous
//    times we see with the detailed log view.
//
// 4. LinearLayout + TextView - ScrollView
//
//    10x10
//      Outer required_size: 185
//      Outer wrap_layout: 349
//      Toggle draw: 537
//      Outer draw: 1
//
//    20x20
//      Outer required_size: 703
//      Outer wrap_layout: 1507
//      Toggle draw: 2213
//      Outer draw: 2
//
//    Reintroducing TextView beings us back to our initial pathological
//    case, even with the ScrollView removed.
//
// 5. LinearLayout + TextView - ScrollView - ResizedView
//
//    20x20
//      Outer required_size: 708
//      Outer wrap_layout: 1448
//      Toggle draw: 2158
//      Outer draw: 2
//
//    Removing ResizedView cut toggle draw time by roughly a third.
//
// 6. LinearLayout + PlainView - ScrollView - ResizedView
//
//    100x100
//      Outer required_size: 78
//      Outer wrap_layout: 169
//      Toggle draw: 250
//      Outer draw: 27
//
//    Removing ResizedView while using PlainTextView does not appear to
//    impact performance. Nonetheless, we have the finding above that
//    shows that removing resize view does improve performance when
//    using TextView.
//
//    It's worth looking into where we use containers that we don't
//    need. Everything we use contributes to these latencies.
//
//    At this stage we'll start looking into performance optimizations
//    for linear layout. This is the one remaining candidate for
//    improvement.
//
// 7. PageLayout + PlainView
//
//    100x100
//      Outer required_size: 49
//      Outer wrap_layout: 141
//      Toggle draw: 192
//      Outer draw: 24
//

const SAMPLE: &str = "Mauris congue orci sed quam pulvinar, ut luctus sapien venenatis. Nulla vel fermentum libero. Donec ac tincidunt tellus. Mauris semper justo leo, eget eleifend eros convallis eu. Donec volutpat, sapien eget convallis tempus, mi felis suscipit nunc, sed viverra turpis mi dapibus leo. Proin arcu lectus, tristique id rhoncus ut, molestie posuere justo. Vivamus at dictum eros. Nulla porta, odio nec cursus condimentum, lectus est bibendum lacus, molestie consequat lectus mauris ut mauris. Proin efficitur mollis leo at pulvinar. Quisque condimentum sapien id mi pulvinar ultrices. Nunc ultricies erat at quam pulvinar, ut dictum dui placerat. Duis eu dapibus lectus, quis semper orci.\nDonec quis tempor velit, in blandit felis. Nam pellentesque, enim eu ullamcorper consectetur, libero felis sollicitudin ex, vel commodo velit ligula sit amet ante. Nullam vitae leo non nisl porta feugiat in et lectus. Vestibulum ut massa massa. Fusce tincidunt, ligula in ornare finibus, metus libero viverra leo, a gravida mi lectus finibus ipsum. Maecenas ullamcorper, nunc vel ullamcorper pretium, massa mauris tincidunt sem, eu gravida sem felis at quam. Phasellus varius mi sit amet odio pellentesque venenatis. Aliquam erat volutpat. Suspendisse potenti. Morbi sed felis elit. Nulla at pharetra quam, et consectetur diam. Nunc viverra mattis ornare. Donec sollicitudin vulputate augue, ac vehicula dui posuere non. In nisi est, finibus ut mattis nec, bibendum vel ipsum. Suspendisse gravida augue at nunc pretium commodo. Quisque aliquam molestie placerat.";

pub struct DevScreen {
    inner: ScrollView<ResizedView<PageLayout>>,
}

impl DevScreen {
    pub fn new() -> Self {
        let mut layout = PageLayout::new();

        let expand_every = 1;
        let i_count = 100;
        let j_count = 100;

        for i in 1..i_count + 1 {
            let mut child = PageLayout::new();
            for j in 1..j_count + 1 {
                child.add_child(
                    ToggleView::new(
                        format!("Child {j} of {j_count}"),
                        PlainTextView::new(SAMPLE).pad_lr(3, 1),
                    )
                    .expanded(j % expand_every == 0),
                );
            }
            layout.add_child(
                ToggleView::new(format!("Child {i} of {i_count}"), child.pad_lr(3, 0))
                    .expanded(i % expand_every == 0),
            );
        }

        Self {
            inner: layout.full_width().scrollable(),
        }
    }
}

impl ViewWrapper for DevScreen {
    wrap_impl!(self.inner: ScrollView<ResizedView<PageLayout>>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('`') => {
                EventResult::with_cb_once(|siv| App::push_screen(siv, AppScreen::Console))
            }
            Event::Char('?') => EventResult::with_cb_once(Self::on_help),
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb_once(App::quit),
            _ => self.inner.on_event(event),
        }
    }

    fn wrap_required_size(&mut self, req: Vec2) -> Vec2 {
        let start = Instant::now();
        let result = self.inner.required_size(req);
        log::info!(
            "Outer required_size: {}",
            Instant::now().duration_since(start).as_millis()
        );
        result
    }

    fn wrap_layout(&mut self, size: Vec2) {
        let start = Instant::now();
        self.inner.layout(size);
        log::info!(
            "Outer wrap_layout: {}",
            Instant::now().duration_since(start).as_millis()
        );
    }

    fn wrap_draw(&self, printer: &Printer) {
        let start = Instant::now();
        self.inner.draw(printer);
        log::info!(
            "Outer draw: {}",
            Instant::now().duration_since(start).as_millis()
        );
    }
}

// Event handlers
impl DevScreen {
    fn on_help(siv: &mut Cursive) {
        let help = vec![(
            None,
            vec![("~", "Debug console".into()), ("q", "Exit".into())],
        )];
        siv.add_layer(HelpDialog::new(help).title("Help - Dev"));
    }
}
