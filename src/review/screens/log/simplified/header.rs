use cursive::{
    theme::{BaseColor, Effect},
    utils::markup::StyledString,
    view::{Finder, Nameable, Resizable, ViewWrapper},
    views::{LinearLayout, TextView, ViewRef},
    wrap_impl,
};

use crate::{
    cursive::view::Padding,
    inspect::log::{EvalLog, EvalSample},
    review::{components::value::ValueView, screens::common::StyledEvalLog},
    util::first_line,
};

pub struct LogHeader {
    inner: LinearLayout,
}

impl LogHeader {
    pub fn new() -> Self {
        let inner = LinearLayout::vertical()
            .child(
                LinearLayout::horizontal()
                    .child(TextView::empty().with_name("task").full_width())
                    .child(TextView::empty().with_name("model"))
                    .child(TextView::empty().with_name("status").pad_lr(2, 0))
                    .child(TextView::empty().with_name("created").pad_l(2)),
            )
            .child(
                LinearLayout::horizontal()
                    .child(
                        TextView::empty()
                            .style(Effect::Dim)
                            .with_name("description")
                            .max_height(2)
                            .full_width(),
                    )
                    .child(ValueView::new("Acc").with_name("acc"))
                    .child(ValueView::new("Stderr").with_name("stderr").pad_l(1))
                    .pad_t(1),
            );

        Self { inner }
    }

    pub fn set_log(&mut self, log: &EvalLog) {
        // Task name
        self.task().set_content(StyledString::concatenate([
            StyledString::styled("Task ", Effect::Dim),
            log.styled_task(),
        ]));

        // Model
        self.model().set_content(&log.eval.model);

        // Status
        self.status().set_content(log.styled_status());

        // Created
        self.created().set_content(log.styled_created());

        // Task description
        let description = log.eval.task_description().unwrap_or_default();
        let (line1, truncated) = first_line(&description);
        self.task_description().set_content(format!(
            "{}{}",
            line1,
            if truncated { "..." } else { "" }
        ));

        // Accuracy / stderr
        let (acc, stderr) = log
            .results
            .as_ref()
            .map(|r| (r.first_accuracy(), r.first_stderr()))
            .unwrap_or((None, None));
        self.acc()
            .set_value(&acc.map(|f| format!("{f:.2}")).unwrap_or_default());
        self.stderr()
            .set_value(&stderr.map(|f| format!("{f:.2}")).unwrap_or_default());
    }

    fn task(&mut self) -> ViewRef<TextView> {
        self.find_name("task").unwrap()
    }

    fn model(&mut self) -> ViewRef<TextView> {
        self.find_name("model").unwrap()
    }

    fn status(&mut self) -> ViewRef<TextView> {
        self.find_name("status").unwrap()
    }

    fn created(&mut self) -> ViewRef<TextView> {
        self.find_name("created").unwrap()
    }

    fn task_description(&mut self) -> ViewRef<TextView> {
        self.find_name("description").unwrap()
    }

    fn acc(&mut self) -> ViewRef<ValueView> {
        self.find_name("acc").unwrap()
    }

    fn stderr(&mut self) -> ViewRef<ValueView> {
        self.find_name("stderr").unwrap()
    }

    pub fn clear(&mut self) {
        self.call_on_name("task", |task: &mut TextView| {
            task.set_content("");
        });
        self.status().set_content("");
        self.created().set_content("");
        self.task_description().set_content("");
        self.acc().set_value("");
        self.stderr().set_value("");
    }
}

impl ViewWrapper for LogHeader {
    wrap_impl!(self.inner: LinearLayout);
}

pub struct SampleHeader {
    inner: LinearLayout,
}

impl SampleHeader {
    pub fn new() -> Self {
        let inner = LinearLayout::horizontal()
            .child(
                LinearLayout::horizontal()
                    .child(TextView::new(StyledString::styled("Sample ", Effect::Dim)))
                    .child(TextView::empty().with_name("sample"))
                    .full_width(),
            )
            .child(TextView::empty().with_name("score").pad_r(1));

        Self { inner }
    }

    pub fn set_sample(&mut self, sample: &EvalSample) {
        self.sample().set_content(StyledString::styled(
            sample.id.to_string(),
            BaseColor::Yellow.dark(),
        ));
        let score = match sample
            .default_score()
            .map(|(_name, score)| score.value.to_string())
            .unwrap_or_default()
            .as_str()
        {
            "C" => StyledString::styled("Correct", BaseColor::Cyan.light()),
            "I" => StyledString::styled("Incorrect", BaseColor::Red.dark()),
            other => other.into(),
        };
        self.score().set_content(score);
    }

    fn sample(&mut self) -> ViewRef<TextView> {
        self.find_name("sample").unwrap()
    }

    fn score(&mut self) -> ViewRef<TextView> {
        self.find_name("score").unwrap()
    }
}

impl ViewWrapper for SampleHeader {
    wrap_impl!(self.inner: LinearLayout);
}
