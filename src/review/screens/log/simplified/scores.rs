use std::{collections::HashMap, fmt::Display};

use cursive::{
    theme::{BaseColor, Effect},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::TextView,
    wrap_impl,
};
use itertools::Itertools;

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::scorer::{Score, Value},
    review::components::toggle::ToggleView,
};

pub struct SampleScores {
    inner: PageLayout,
}

impl SampleScores {
    pub fn new(scores: &HashMap<String, Score>) -> Self {
        let mut inner = PageLayout::new();
        if scores.len() == 1 {
            let (_name, score) = scores.iter().next().unwrap();
            // For the common case where a sample has one score with an
            // explanation, the parent is assumed to place the target
            // directly below this view to maintain a tie between answer
            // and target. In this case we want to pad the interior
            // content of the answer to provide spacing between the
            // answer content and the target toggle. See the layout for
            // Scores in the `body` module.
            inner.add_child(ScoreView::new(score, score.answer.is_some()));
        } else {
            for (i, name) in scores.keys().sorted().enumerate() {
                let score = &scores[name];
                let value = score.value.to_string();
                let value_label = match value.as_str() {
                    "I" => StyledString::styled("Incorrect", BaseColor::Red.dark()),
                    "C" => StyledString::styled("Correct", BaseColor::Cyan.light()),
                    _ => StyledString::styled(&value, Effect::Dim),
                };
                inner.add_child(
                    ToggleView::new(
                        StyledString::concatenate([
                            StyledString::plain(format!("{name} ")),
                            value_label,
                        ]),
                        ScoreView::new(score, false).pad_lrtb(
                            3,
                            0,
                            1,
                            if i < scores.len() - 1 { 1 } else { 0 },
                        ),
                    )
                    .expanded_label(name)
                    .expanded(value == "I"),
                );
            }
        }

        Self { inner }
    }
}

impl ViewWrapper for SampleScores {
    wrap_impl!(self.inner: PageLayout);
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(s) => s.fmt(f),
            _ => write!(f, "{self:?}"),
        }
    }
}

struct ScoreView {
    inner: PageLayout,
}

impl ScoreView {
    fn new(score: &Score, pad_answer_inner: bool) -> Self {
        let mut inner = PageLayout::new();

        // Score value
        let value = score.value.to_string();
        inner.add_child(TextView::new(match value.as_str() {
            "I" => StyledString::styled("Incorrect", BaseColor::Red.dark()),
            "C" => StyledString::styled("Correct", BaseColor::Cyan.light()),
            _ => StyledString::plain(&value),
        }));

        // Explanation
        if let Some(explanation) = score.explanation.as_ref()
            && !explanation.is_empty()
        {
            inner.add_child(TextView::new(StyledString::styled(explanation, Effect::Dim)).pad_t(1));
        }

        // Answer (toggle)
        if let Some(answer) = score.answer.as_ref() {
            inner.add_child(
                ToggleView::new(
                    "Answer",
                    TextView::new(StyledString::styled(
                        answer,
                        match value.as_str() {
                            "I" => BaseColor::Red.dark(),
                            "C" => BaseColor::Cyan.light(),
                            _ => BaseColor::White.dark(),
                        },
                    ))
                    .pad_lrtb(3, 0, 1, if pad_answer_inner { 1 } else { 0 }),
                )
                .expanded(value.as_str() == "I")
                .pad_t(1),
            );
        }

        Self { inner }
    }
}

impl ViewWrapper for ScoreView {
    wrap_impl!(self.inner: PageLayout);
}
