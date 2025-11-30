use cursive::{
    direction::Direction,
    event::EventResult,
    theme::{BaseColor, Effect},
    utils::markup::StyledString,
    view::{CannotFocus, IntoBoxedView, ViewWrapper},
    views::TextView,
    wrap_impl,
};

use crate::{
    cursive::{
        view::{Padding, Scrollable},
        views::{PageLayout, ScrollView},
    },
    inspect::log::{EvalLog, EvalStatus},
    review::{
        components::{banner::Banner, toggle::ToggleView},
        screens::{
            common::{IntoAttrsView, attr},
            log::advanced::{
                error::error_view, eval_plan::eval_plan_view, eval_spec::eval_spec_view,
                results::results_view, sample::eval_sample_view,
            },
        },
    },
};

pub struct AdvancedView {
    inner: ScrollView<PageLayout>,
}

impl AdvancedView {
    pub fn new() -> Self {
        let inner = PageLayout::new().scrollable();
        Self { inner }
    }

    pub fn set_log_sample(&mut self, log: &EvalLog, active_sample: Option<usize>) {
        self.clear();

        // If log status isn't success, show banner qualifying results
        if log.status != EvalStatus::Success {
            self.add_child(Banner::new(StyledString::concatenate([
                StyledString::plain("Log status is "),
                StyledString::styled(log.status.to_string(), Effect::Italic),
                StyledString::plain(". Results may not be valid."),
            ])));
        }

        self.add_child(
            [
                attr("location", &log.location),
                attr("status", log.status),
                attr("version", log.version),
            ]
            .into_attrs_view()
            .pad_lrtb(1, 1, 1, 0),
        );

        self.add_child(
            ToggleView::new("Eval spec", eval_spec_view(&log.eval).pad_lrtb(3, 1, 0, 1)).pad_t(1),
        );

        self.add_child(ToggleView::new(
            "Eval plan",
            eval_plan_view(&log.plan).pad_lrtb(3, 1, 0, 1),
        ));

        self.add_child(ToggleView::new(
            "Results",
            results_view(log.results.as_ref()).pad_lrtb(3, 1, 0, 0),
        ));

        // TODO /// Eval stats (runtime, model usage)
        // pub stats: EvalStats,

        if let Some(error) = log.error.as_ref() {
            self.add_child(ToggleView::new(
                "Halt error",
                error_view(error).pad_lrtb(3, 1, 0, 0),
            ));
        }

        if let Some(active_sample) = active_sample {
            let sample = log.samples.as_ref().unwrap().get(active_sample).unwrap();
            self.add_child(
                TextView::new(StyledString::concatenate([
                    StyledString::plain("Active sample "),
                    StyledString::styled(sample.id.to_string(), BaseColor::Yellow.dark()),
                ]))
                .pad_lrtb(1, 1, 1, 0),
            );
            self.add_child(eval_sample_view(sample).pad_l(3));
        }

        // /// Reduced sample values
        // pub reductions: Option<Vec<EvalSampleReductions>>,
    }

    fn add_child<V: IntoBoxedView + 'static>(&mut self, view: V) {
        self.inner.get_inner_mut().add_child(view);
    }

    pub fn clear(&mut self) {
        self.inner.get_inner_mut().clear();
    }
}

impl ViewWrapper for AdvancedView {
    wrap_impl!(self.inner: ScrollView<PageLayout>);

    fn wrap_take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::consumed())
    }
}
