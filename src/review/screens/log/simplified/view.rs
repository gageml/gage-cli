use cursive::{
    direction::Direction,
    event::EventResult,
    theme::Effect,
    utils::markup::StyledString,
    view::{CannotFocus, Finder, Nameable, Resizable, ViewWrapper},
    views::{HideableView, LinearLayout, NamedView, PaddedView, ResizedView, ViewRef},
    wrap_impl,
};

use crate::{
    cursive::view::Padding,
    inspect::log::{EvalLog, EvalStatus},
    review::{
        components::banner::Banner,
        screens::log::simplified::{Body, LogHeader, header::SampleHeader},
    },
};

pub struct SampleView {
    banner: usize,
    body: usize,
    inner: ResizedView<LinearLayout>,
}

impl SampleView {
    pub fn new() -> Self {
        let mut inner = LinearLayout::vertical()
            .child(Banner::empty().full_width())
            .child(
                LogHeader::new()
                    .with_name("log_header")
                    .full_width()
                    .pad_lrtb(1, 1, 1, 0),
            )
            .child(
                HideableView::new(
                    SampleHeader::new()
                        .with_name("sample_header")
                        .full_width()
                        .pad(1),
                )
                .with_name("sample_header_container"),
            )
            .child(Body::new().full_screen())
            .full_width();

        // Children
        let banner = 0;
        let body = 3;

        // Initial focus on body
        inner.get_inner_mut().set_focus_index(body).unwrap();

        Self {
            banner,
            body,
            inner,
        }
    }

    pub fn set_log_sample(
        &mut self,
        log: &EvalLog,
        active_sample: Option<usize>,
        reset_active_panel: bool,
    ) {
        // If log status isn't success, show banner qualifying results
        if log.status != EvalStatus::Success {
            self.banner_mut().set_content(StyledString::concatenate([
                StyledString::plain("Log status is "),
                StyledString::styled(log.status.to_string(), Effect::Italic),
                StyledString::plain(". Results may not be valid."),
            ]));
        } else {
            self.banner_mut().clear();
        }

        // Log header
        self.log_header().set_log(log);

        // Sample header
        if let Some(sample) = log
            .samples
            .as_ref()
            .and_then(|samples| active_sample.and_then(|i| samples.get(i)))
        {
            self.sample_header().set_sample(sample);
            self.sample_header_container().set_visible(true);
        } else {
            self.sample_header_container().set_visible(false);
        }

        // Sample body
        self.body_mut()
            .set_sample(log, active_sample, reset_active_panel);
    }

    fn banner_mut(&mut self) -> &mut Banner {
        self.child_mut(self.banner)
    }

    fn log_header(&mut self) -> ViewRef<LogHeader> {
        self.find_name("log_header").unwrap()
    }

    fn sample_header(&mut self) -> ViewRef<SampleHeader> {
        self.find_name("sample_header").unwrap()
    }

    fn sample_header_container(
        &mut self,
    ) -> ViewRef<HideableView<PaddedView<ResizedView<NamedView<SampleHeader>>>>> {
        self.find_name("sample_header_container").unwrap()
    }

    fn body_mut(&mut self) -> &mut Body {
        self.child_mut(self.body)
    }

    fn child_mut<V>(&mut self, pos: usize) -> &mut V
    where
        V: 'static,
    {
        self.inner
            .get_inner_mut()
            .get_child_mut(pos)
            .unwrap()
            .downcast_mut::<ResizedView<V>>()
            .unwrap()
            .get_inner_mut()
    }

    pub fn clear(&mut self) {
        self.log_header().clear();
        self.body_mut().clear();
    }
}

impl ViewWrapper for SampleView {
    wrap_impl!(self.inner: ResizedView<LinearLayout>);

    fn wrap_take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::consumed())
    }
}
