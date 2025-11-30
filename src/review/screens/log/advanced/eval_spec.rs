use std::fmt::Display;

use cursive::{View, utils::markup::StyledString};
use itertools::Itertools;

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::log::{EvalDataset, EvalMetricDefinition, EvalScorer, EvalSpec},
    review::{
        components::toggle::ToggleView,
        screens::{
            common::{IntoAttrsView, attr, attr_option, nested_attrs, nested_attrs_option},
            log::advanced::util::view_or_none,
        },
    },
};

pub fn eval_spec_view(spec: &EvalSpec) -> impl View {
    let attrs = [
        attr_option("eval_set_id", spec.eval_set_id.as_ref()),
        attr("eval_id", &spec.eval_id),
        attr("run_id", &spec.run_id),
        attr("created", spec.created.to_iso_8601_local()),
        attr("task", &spec.task),
        attr("task_id", &spec.task_id),
        attr("task_version", &spec.task_version),
        attr_option("task_file", spec.task_file.as_ref()),
        attr_option("task_display_name", spec.task_display_name.as_ref()),
        attr_option("task_registry_name", spec.task_registry_name.as_ref()),
        nested_attrs("task_attribs", &spec.task_attribs),
        nested_attrs("task_args", &spec.task_args),
        nested_attrs("task_args_passed", &spec.task_args_passed),
        attr_option("solver", spec.solver.as_ref()),
        nested_attrs_option("solver_args", spec.solver_args.as_ref()),
        attr_option("tags", spec.tags.as_ref().map(|v| v.join(", "))),
        attr("model", &spec.model),
        attr_option("model_base_url", spec.model_base_url.as_ref()),
        nested_attrs("model_args", &spec.model_args),
        attr_option("revision", spec.revision.as_ref().map(|v| v.to_string())),
        nested_attrs("packages", &spec.packages),
        nested_attrs_option("metadata", spec.metadata.as_ref()),
        nested_attrs_option(
            "metrics",
            spec.metrics.as_ref().map(|metrics| {
                metrics
                    .into_iter()
                    .map(|(name, metric)| (name.unwrap_or(&metric.name), metric))
            }),
        ),
    ];

    PageLayout::new()
        .child(attrs.into_attrs_view())
        .child(
            ToggleView::new("Dataset", dataset_view(&spec.dataset).pad_lrtb(3, 0, 0, 1)).pad_t(1),
        )
        .child(ToggleView::new(
            "Scorers",
            scorers_view(spec.scorers.as_ref()).pad_l(3),
        ))
}

impl Display for EvalMetricDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(options) = self.options.as_ref()
            && !options.is_empty()
        {
            write!(
                f,
                "{}",
                options
                    .iter()
                    .map(|(name, val)| format!("{name}={val}"))
                    .join(", ")
            )
        } else {
            write!(f, "")
        }
    }
}

fn dataset_view(dataset: &EvalDataset) -> impl View {
    [
        attr_option("name", dataset.name.as_ref()),
        attr_option("location", dataset.location.as_ref()),
        attr(
            "samples",
            dataset.samples.map(|v| v.to_string()).unwrap_or_default(),
        ),
        attr(
            "shuffled",
            dataset.shuffled.map(|v| v.to_string()).unwrap_or_default(),
        ),
        attr(
            "sample_ids",
            dataset
                .sample_ids
                .as_ref()
                .map(|v| v.iter().map(|v| v.to_string()).join(", "))
                .unwrap_or_default(),
        ),
    ]
    .into_attrs_view()
}

fn scorers_view(scorers: Option<&Vec<EvalScorer>>) -> impl View {
    view_or_none(scorers, |scorers| {
        let mut attrs = Vec::new();
        for (i, scorer) in scorers.iter().enumerate() {
            if i > 0 {
                attrs.push(StyledString::plain(""));
            }
            attrs.extend([
                attr("name", &scorer.name),
                nested_attrs_option("options", scorer.options.as_ref()),
                nested_attrs_option(
                    "metrics",
                    scorer.metrics.as_ref().map(|metrics| {
                        metrics
                            .into_iter()
                            .map(|(name, metric)| (name.unwrap_or(&metric.name), metric))
                    }),
                ),
                nested_attrs_option("metadata", scorer.metadata.as_ref()),
            ]);
        }
        attrs.into_attrs_view()
    })
}
