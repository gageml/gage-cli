use cursive::View;

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::error::EvalError,
    review::screens::{
        common::{IntoAttrsView, attr},
        log::advanced::util::text_content_view,
    },
};

pub fn error_view(error: &EvalError) -> impl View {
    let mut view = PageLayout::new();

    view.add_child([attr("message", &error.message)].into_attrs_view());
    view.add_child(text_content_view(error.traceback.trim_end()).pad_t(1));

    view
}
