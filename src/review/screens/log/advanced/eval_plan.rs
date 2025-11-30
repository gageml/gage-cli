use cursive::{View, views::TextView};

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::log::EvalPlan,
    review::screens::common::{IntoAttrsView, attr, caption},
};

pub fn eval_plan_view(plan: &EvalPlan) -> impl View {
    let mut view = PageLayout::new();

    // Base attrs
    if plan.name != "plan" {
        view.add_child([attr("name", &plan.name)].into_attrs_view());
    }

    // Each steps gets toggle
    if (!view.is_empty() && !plan.steps.is_empty()) || plan.finish.is_some() {
        view.add_child(caption("Steps").pad_t(1));
    }
    for step in plan.steps.iter() {
        view.add_child(TextView::new(&step.solver));
        if !step.params.is_empty() {
            view.add_child(
                step.params
                    .iter()
                    .map(|(name, val)| attr(name, val))
                    .into_attrs_view()
                    .pad_l(2),
            );
        }
    }

    // Finish step gets a toggle
    if let Some(finish) = plan.finish.as_ref() {
        view.add_child(TextView::new("Finish"));
        if !finish.params.is_empty() {
            view.add_child(
                finish
                    .params
                    .iter()
                    .map(|(name, val)| attr(name, val))
                    .into_attrs_view()
                    .pad_l(2),
            );
        }
    }

    view
}
