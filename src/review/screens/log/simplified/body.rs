use std::sync::{Arc, Mutex};

use cursive::{
    View,
    direction::Direction,
    event::EventResult,
    theme::{BaseColor, ColorStyle, Effect, Style},
    view::{CannotFocus, Finder, IntoBoxedView, Nameable, ViewWrapper},
    views::TextView,
    wrap_impl,
};

use crate::{
    cursive::{
        view::{Padding, Scrollable},
        views::{PageLayout, ScrollView},
    },
    inspect::log::EvalLog,
    review::{
        components::{panel::Panel, toggle::ToggleView},
        screens::{
            common::dim_ital,
            log::simplified::{
                ChatMessageView, ErrorsView, InputView, OutputView, SampleScores, TargetView,
            },
        },
    },
};

pub struct Body {
    active_panel: Arc<Mutex<BodyPanel>>,
    inner: ScrollView<PageLayout>,
}

#[derive(Clone, Debug)]
enum BodyPanel {
    Input,
    Output,
    Scores,
}

impl Body {
    pub fn new() -> Self {
        Self {
            active_panel: Arc::new(Mutex::new(BodyPanel::Input)),
            inner: PageLayout::new().scrollable(),
        }
    }

    pub fn set_sample(
        &mut self,
        log: &EvalLog,
        active_sample: Option<usize>,
        reset_active_panel: bool,
    ) {
        if reset_active_panel {
            *self.active_panel.lock().unwrap() = BodyPanel::Input;
        }
        self.clear();

        // Early exit for no samples
        let samples = if let Some(val) = log.samples.as_ref() {
            val
        } else {
            self.add_child(TextView::new("No samples for this log."));
            return;
        };

        // Early exit for no active sample
        let active = if let Some(val) = active_sample {
            val
        } else {
            self.add_child(TextView::new("No active samples."));
            return;
        };

        let sample = samples.get(active).unwrap();

        // Input
        let active_panel = self.active_panel.clone();
        self.add_child(
            ToggleView::new(
                "Input",
                Panel::new(InputView::new(&sample.input, &sample.attachments))
                    .on_focus(move |_| {
                        *active_panel.lock().unwrap() = BodyPanel::Input;
                        EventResult::Ignored
                    })
                    .with_name("input")
                    .pad_b(1),
            )
            .expanded(true)
            .pad_x(1),
        );

        // Output
        let active_panel = self.active_panel.clone();
        self.add_child(
            ToggleView::new(
                "Output",
                Panel::new(OutputView::new(&sample.output, &sample.attachments))
                    .on_focus(move |_| {
                        *active_panel.lock().unwrap() = BodyPanel::Output;
                        EventResult::Ignored
                    })
                    .with_name("output")
                    .pad_b(1),
            )
            .expanded(true)
            .pad_x(1),
        );

        // Errors
        if !sample.errors().is_empty() {
            self.add_child(
                ToggleView::new(
                    "Errors",
                    Panel::new(ErrorsView::new(sample.errors())).pad_y(1),
                )
                .expanded(true)
                .pad_x(1),
            );
        }

        // Score
        if let Some(scores) = sample.scores.as_ref()
            && !scores.is_empty()
        {
            let scores = PageLayout::new().child(SampleScores::new(scores)).child(
                ToggleView::new(
                    "Target",
                    TargetView::new(&sample.target).pad_lrtb(3, 0, 1, 0),
                )
                .expanded(
                    !sample
                        .is_correct()
                        .expect("value as there's a least one score"),
                )
                .pad_t(
                    // For the common case where a sample has one score
                    // with an explanation, don't pad target toggle to
                    // keep the answer and target toggles together,
                    // otherwise add a space. There's a corollary rule
                    // in SampleScores view to pad the toggle content of
                    // the answer.
                    if scores.len() == 1 && scores.values().next().unwrap().answer.is_some() {
                        0
                    } else {
                        1
                    },
                ),
            );
            let active_panel = self.active_panel.clone();
            self.add_child(
                ToggleView::new(
                    "Score",
                    Panel::new(scores)
                        .on_focus(move |_| {
                            *active_panel.lock().unwrap() = BodyPanel::Scores;
                            EventResult::Ignored
                        })
                        .with_name("score")
                        .pad_b(1),
                )
                .expanded(true)
                .pad_x(1),
            );
        }

        // Messages
        let mut messages_layout = PageLayout::new();
        for msg in sample.messages.iter() {
            messages_layout.add_child(ToggleView::new(
                msg.role(),
                Panel::new(ChatMessageView::new(msg, &sample.attachments))
                    .color(ColorStyle::back(BaseColor::Black.light()))
                    .border(ColorStyle::new(
                        BaseColor::Black.light(),
                        BaseColor::Black.light(),
                    ))
                    .focus_border(
                        Style::from(ColorStyle::new(
                            BaseColor::Yellow.dark(),
                            BaseColor::Black.light(),
                        ))
                        .combine(Effect::Dim),
                    )
                    .pad_b(1),
            ))
        }
        if messages_layout.is_empty() {
            messages_layout.add_child(TextView::new(dim_ital("Empty")));
        }
        self.add_child(ToggleView::new("Messages", messages_layout.pad_lrtb(2, 0, 0, 0)).pad_x(1));

        // Set focus on active panel (TODO - use of take_focus here isn't working)
        let active_panel = (*self.active_panel.lock().unwrap()).clone();
        match active_panel {
            BodyPanel::Input => {
                self.call_on_name("input", |view: &mut Panel<InputView>| {
                    view.take_focus(Direction::none()).unwrap();
                });
            }
            BodyPanel::Output => {
                self.call_on_name("output", |view: &mut Panel<OutputView>| {
                    view.take_focus(Direction::none()).unwrap();
                });
            }
            BodyPanel::Scores => {
                self.call_on_name("score", |view: &mut Panel<PageLayout>| {
                    view.take_focus(Direction::none()).unwrap();
                });
            }
        }
    }

    fn add_child<V: IntoBoxedView + 'static>(&mut self, view: V) -> usize {
        let inner = self.layout_mut();
        inner.add_child(view);
        inner.len() - 1
    }

    // pub fn set_focus_index(
    //     &mut self,
    //     index: usize,
    // ) -> std::result::Result<EventResult, ViewNotFound> {
    //     self.layout_mut().set_focus_index(index)
    // }

    pub fn clear(&mut self) {
        self.layout_mut().clear();
    }

    fn layout_mut(&mut self) -> &mut PageLayout {
        self.inner.get_inner_mut()
    }

    fn call_on_name<V, F, R>(&mut self, name: &str, callback: F) -> Option<R>
    where
        V: View,
        F: FnOnce(&mut V) -> R,
    {
        self.inner.call_on_name(name, callback)
    }
}

impl ViewWrapper for Body {
    wrap_impl!(self.inner: ScrollView<PageLayout>);

    fn wrap_take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::consumed())
    }
}
