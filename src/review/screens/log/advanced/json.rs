use std::{cmp::min, collections::HashMap};

use cursive::{View, views::Layer};

use crate::{
    cursive::{
        view::Padding,
        views::{PageLayout, PlainTextView},
    },
    inspect::{json::JsonValue, log::Attachments},
    review::{
        screens::{
            common::{IntoAttrsView, attr, attr_label},
            log::advanced::util::resolve_attachment,
        },
        theme,
    },
};

pub fn json_value_view(value: &JsonValue, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    match value {
        JsonValue::String(s) => {
            view.add_child(PlainTextView::wrap(&resolve_attachment(s, attachments)))
        }
        JsonValue::List(l) => view.add_child(json_list_view(l, attachments)),
        JsonValue::Map(m) => view.add_child(json_map_view(m, attachments)),
        value => view.add_child(PlainTextView::wrap(&value.to_string())),
    }
    Layer::with_color(view, theme::Style::panel())
}

fn json_list_view(l: &[JsonValue], attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    for (i, val) in l.iter().enumerate() {
        view.add_child(json_value_view(val, attachments).pad_t(min(i, 1)));
    }
    view
}

pub fn json_map_view(m: &HashMap<String, JsonValue>, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    for (name, val) in m {
        match val {
            JsonValue::String(s) => view
                .add_child([attr(name, if s.is_empty() { "\"\"" } else { s })].into_attrs_view()),
            JsonValue::Bool(b) => view.add_child([attr(name, b)].into_attrs_view()),
            JsonValue::Int(i) => view.add_child([attr(name, i)].into_attrs_view()),
            JsonValue::Float(n) => view.add_child([attr(name, n)].into_attrs_view()),
            JsonValue::None => view.add_child([attr(name, "null")].into_attrs_view()),
            JsonValue::List(l) => {
                if l.is_empty() {
                    view.add_child([attr(name, "[]")].into_attrs_view());
                } else {
                    view.add_child(attr_label(name));
                    view.add_child(json_list_view(l, attachments).pad_l(2));
                }
            }
            JsonValue::Map(m) => {
                if m.is_empty() {
                    view.add_child([attr(name, "{}")].into_attrs_view());
                } else {
                    view.add_child(attr_label(name));
                    view.add_child(json_map_view(m, attachments).pad_l(2));
                }
            }
        }
    }
    view
}
