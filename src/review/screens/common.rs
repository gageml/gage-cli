use std::fmt::Display;

use cursive::{
    View,
    theme::{BaseColor, Effect, Effects, Style},
    utils::markup::StyledString,
    views::TextView,
};
use itertools::{Itertools, intersperse};

use crate::inspect::log::{EvalLog, EvalStatus};

pub trait StyledEvalLog {
    fn styled_status(&self) -> StyledString;
    fn styled_task(&self) -> StyledString;
    fn styled_created(&self) -> StyledString;
    fn styled_run_type(&self) -> StyledString;
    fn styled_score(&self) -> StyledString;
}

impl StyledEvalLog for EvalLog {
    fn styled_status(&self) -> StyledString {
        StyledString::styled(
            self.status.to_string(),
            match self.status {
                EvalStatus::Success => Style::from(Effect::Simple),
                EvalStatus::Error => Style::from(BaseColor::Red.light()).combine(Effect::Dim),
                _ => Style::from(Effect::Dim),
            },
        )
    }

    fn styled_task(&self) -> StyledString {
        StyledString::styled(&self.eval.task, BaseColor::Yellow.light())
    }

    fn styled_created(&self) -> StyledString {
        StyledString::styled(self.eval.created.to_human(), Effect::Dim)
    }

    fn styled_run_type(&self) -> StyledString {
        StyledString::styled(self.eval.run_type().unwrap_or_default(), Effect::Dim)
    }

    fn styled_score(&self) -> StyledString {
        self.results
            .as_ref()
            .and_then(|r| r.first_accuracy())
            .map(|acc| format!("{acc:0.4}"))
            .unwrap_or_default()
            .into()
    }
}

pub fn dim_ital(s: &str) -> StyledString {
    StyledString::styled(
        s,
        Effects::merge(Effects::only(Effect::Dim), Effects::only(Effect::Italic)),
    )
}

pub fn attr<S: Display>(label: &str, value: S) -> StyledString {
    attr_option(label, Some(value))
}

pub fn attr_option<S: Display>(label: &str, value: Option<S>) -> StyledString {
    StyledString::concatenate([
        StyledString::styled(label, BaseColor::Cyan.light()),
        StyledString::plain(" "),
        if let Some(value) = value {
            StyledString::plain(value.to_string())
        } else {
            dim_ital("empty")
        },
    ])
}

pub fn attr_label(label: &str) -> impl View {
    TextView::new(StyledString::styled(label, BaseColor::Cyan.light()))
}

pub trait IntoAttrsView {
    fn into_attrs_view(self) -> impl View;
}

impl<I> IntoAttrsView for I
where
    I: IntoIterator<Item = StyledString>,
{
    fn into_attrs_view(self) -> impl View {
        let s = StyledString::concatenate(intersperse(self, "\n".into()));
        if s.is_empty() {
            TextView::new(dim_ital("empty"))
        } else {
            TextView::new(s)
        }
    }
}

pub fn nested_attrs<'a, Attrs: IntoIterator<Item = (&'a String, S)>, S: Display>(
    label: &str,
    attrs: Attrs,
) -> StyledString {
    nested_attrs_option(label, Some(attrs))
}

pub fn nested_attrs_option<'a, Attrs: IntoIterator<Item = (&'a String, S)>, S: Display>(
    label: &str,
    attrs: Option<Attrs>,
) -> StyledString {
    if let Some(attrs) = attrs {
        let attrs = attrs
            .into_iter()
            .map(|(label, value)| StyledString::concatenate(["  ".into(), attr(label, value)]))
            .collect_vec();
        if attrs.is_empty() {
            attr_option::<S>(label, None)
        } else {
            StyledString::concatenate([
                StyledString::styled(label, BaseColor::Cyan.light()),
                StyledString::plain("\n"),
                StyledString::concatenate(intersperse(attrs, "\n".into())),
            ])
        }
    } else {
        attr_option::<S>(label, None)
    }
}

pub fn attrs<'a, Attrs: IntoIterator<Item = (&'a String, S)>, S: Display>(
    attrs: Attrs,
) -> StyledString {
    attrs_option(Some(attrs))
}

pub fn attrs_option<'a, Attrs: IntoIterator<Item = (&'a String, S)>, S: Display>(
    attrs: Option<Attrs>,
) -> StyledString {
    if let Some(attrs) = attrs {
        let styled = attrs
            .into_iter()
            .map(|(name, val)| attr(name, val))
            .collect_vec();
        if styled.is_empty() {
            dim_ital("None")
        } else {
            StyledString::concatenate(styled)
        }
    } else {
        dim_ital("None")
    }
}

pub fn caption(s: &str) -> TextView {
    TextView::new(StyledString::styled(s, Effect::Dim))
}
