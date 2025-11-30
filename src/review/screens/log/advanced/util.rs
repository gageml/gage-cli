use cursive::{
    View,
    views::{Layer, TextView},
};

use crate::{
    cursive::views::{PageLayout, PlainTextView},
    inspect::log::Attachments,
    review::{screens::common::dim_ital, theme},
};

pub fn view_or_none<T, F: FnOnce(T) -> V, V: View>(predicate: Option<T>, f: F) -> impl View {
    let mut view = PageLayout::new();
    if let Some(v) = predicate {
        view.add_child(f(v))
    } else {
        view.add_child(TextView::new(dim_ital("None")))
    }
    view
}

pub fn text_content_view(s: &str) -> Layer<PlainTextView> {
    Layer::with_color(PlainTextView::wrap(s), theme::Style::panel())
}

pub fn resolve_attachment(value: &String, attachments: &Attachments) -> String {
    if let Some(attachment) = value.strip_prefix("attachment://") {
        attachments.get(attachment).unwrap_or(value).clone()
    } else {
        value.clone()
    }
}
