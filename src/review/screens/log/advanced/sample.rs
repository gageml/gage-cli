use std::collections::HashMap;

use cursive::{View, utils::markup::StyledString, views::TextView};

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::{
        event::{self, ToNodeIter},
        log::{Attachments, EvalSample},
        model::{ChatMessage, ChatMessageContent},
        scorer::Score,
    },
    review::{
        components::toggle::ToggleView,
        screens::{
            common::{IntoAttrsView, attr, attr_option, attrs, dim_ital, nested_attrs_option},
            log::advanced::{
                chat_message::chat_message_view,
                event::event_view,
                input::input_view,
                output::output_view,
                target::target_view,
                util::{text_content_view, view_or_none},
            },
        },
    },
};

pub fn eval_sample_view(sample: &EvalSample) -> impl View {
    let mut view = PageLayout::new();

    view.add_child(
        [
            attr("epoch", sample.epoch),
            attr_option("setup", sample.setup.as_ref()),
        ]
        .into_attrs_view(),
    );

    view.add_child(
        ToggleView::new(
            "Metadata",
            TextView::new(attrs(&sample.metadata)).pad_lrtb(3, 1, 0, 1),
        )
        .pad_t(1),
    );

    view.add_child(ToggleView::new(
        "Input",
        input_view(&sample.input, &sample.attachments).pad_lrtb(3, 1, 0, 1),
    ));

    view.add_child(ToggleView::new(
        "Target",
        target_view(&sample.target).pad_lrtb(3, 1, 0, 1),
    ));

    view.add_child(ToggleView::new(
        "Files",
        files_view(sample.files.as_ref()).pad_lrtb(3, 1, 0, 1),
    ));

    view.add_child(ToggleView::new(
        "Messages",
        messages_view(&sample.messages, &sample.attachments).pad_lrtb(3, 1, 0, 1),
    ));

    view.add_child(ToggleView::new(
        "Output",
        output_view(&sample.output, &sample.attachments).pad_lrtb(3, 1, 0, 1),
    ));

    view.add_child(ToggleView::new(
        "Scores",
        scores_view(sample.scores.as_ref()).pad_lrtb(3, 1, 0, 1),
    ));

    view.add_child(ToggleView::new(
        "Events",
        events_view(sample.events.iter_nodes(), &sample.attachments).pad_lrtb(3, 1, 0, 1),
    ));

    view
}

fn files_view(files: Option<&Vec<String>>) -> impl View {
    view_or_none(files, |files| text_content_view(&files.join("\n\n")))
}

pub fn messages_view(messages: &Vec<ChatMessage>, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    for msg in messages {
        view.add_child(
            ToggleView::new(
                StyledString::concatenate([
                    StyledString::plain(format!("{} ", msg.role())),
                    dim_ital(&chat_message_content_preview(&msg.base().content)),
                ]),
                chat_message_view(msg, attachments).pad_lrtb(3, 0, 0, 1),
            )
            .expanded_label(msg.role()),
        )
    }
    view
}

fn chat_message_content_preview(msg: &ChatMessageContent) -> String {
    match msg {
        ChatMessageContent::String(s) => s
            .split_at_checked(40)
            .map(|(s, _)| format!("{s}..."))
            .unwrap_or(s.into()),
        ChatMessageContent::ContentList(l) => format!(
            "{} {}",
            l.len(),
            if l.len() == 1 { "message" } else { "messages" }
        ),
    }
}

fn scores_view(scores: Option<&HashMap<String, Score>>) -> impl View {
    let mut view = PageLayout::new();

    if let Some(scores) = scores
        && !scores.is_empty()
    {
        for (name, score) in scores {
            view.add_child(
                ToggleView::new(
                    StyledString::concatenate([
                        StyledString::plain(format!("{name} ")),
                        dim_ital(&score.value.to_string()),
                    ]),
                    [
                        attr("value", &score.value),
                        attr_option("answer", score.answer.as_ref()),
                        attr_option("explanation", score.explanation.as_ref()),
                        nested_attrs_option("metadata", score.metadata.as_ref()),
                    ]
                    .into_attrs_view()
                    .pad_lrtb(3, 0, 0, 1),
                )
                .expanded_label(name),
            )
        }
    } else {
        view.add_child(TextView::new(dim_ital("None")))
    }

    view
}

fn events_view<'a, I: Iterator<Item = event::Node<'a>>>(
    nodes: I,
    attachments: &Attachments,
) -> impl View {
    let mut view = PageLayout::new();
    for node in nodes {
        match node {
            event::Node::Span(span) => {
                view.add_child(ToggleView::new(
                    span.name,
                    events_view(span.events, attachments).pad_l(3),
                ));
            }
            event::Node::Event(event) => {
                view.add_child(ToggleView::new(
                    &event.base().event_name,
                    event_view(event, attachments).pad_lrtb(3, 0, 0, 1),
                ));
            }
        }
    }
    if view.is_empty() {
        view.add_child(TextView::new(dim_ital("None")));
    }
    view
}
