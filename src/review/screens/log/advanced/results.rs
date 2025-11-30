use std::fmt::Display;

use cursive::{View, utils::markup::StyledString};

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::log::{EvalMetric, EvalResults, EvalScore},
    review::{
        components::toggle::ToggleView,
        screens::{
            common::{IntoAttrsView, attr, attr_option, nested_attrs, nested_attrs_option},
            log::advanced::util::view_or_none,
        },
    },
};

pub fn results_view(results: Option<&EvalResults>) -> impl View {
    view_or_none(results, |results| {
        let mut view = PageLayout::new();
        let attrs = [
            attr("total_samples", results.total_samples),
            attr("completed_samples", results.completed_samples),
            nested_attrs_option("metadata", results.metadata.as_ref()),
        ];
        view.add_child(attrs.into_attrs_view());
        view.add_child(
            ToggleView::new("Scores", eval_scores_view(&results.scores).pad_l(3)).pad_t(1),
        );
        view
    })
}

fn eval_scores_view(scores: &Vec<EvalScore>) -> impl View {
    view_or_none((!scores.is_empty()).then_some(scores), |scores| {
        let mut view = PageLayout::new();
        let mut attrs = Vec::new();
        for (i, score) in scores.iter().enumerate() {
            if i > 0 {
                attrs.push(StyledString::plain(""));
            }
            attrs.extend([
                attr("name", &score.name),
                attr("scorer", &score.scorer),
                attr_option("reducer", score.reducer.as_ref()),
                attr(
                    "scored_samples",
                    score
                        .scored_samples
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
                attr(
                    "unscored_samples",
                    score
                        .unscored_samples
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
                nested_attrs("params", &score.params),
                nested_attrs("metrics", &score.metrics),
                nested_attrs_option("metadata", score.metadata.as_ref()),
            ]);
        }
        view.add_child(attrs.into_attrs_view());
        view
    })
}

impl Display for EvalMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut extra = Vec::new();
        for (name, val) in self.params.iter() {
            extra.push(format!("{name}: {val}"));
        }
        if extra.is_empty() {
            self.value.fmt(f)
        } else {
            write!(f, "{} ({})", self.value, extra.join(", "))
        }
    }
}
