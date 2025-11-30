use cursive::View;

use crate::{
    inspect::log::Target,
    review::screens::log::advanced::util::{text_content_view, view_or_none},
};

pub fn target_view(target: &Target) -> impl View {
    let s = match target {
        Target::String(s) => s.to_string(),
        Target::List(l) => l.join("\n\n"),
    };
    view_or_none((!s.is_empty()).then_some(s), |s| text_content_view(&s))
}
