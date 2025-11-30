use std::cmp::min;

use cursive::View;
use itertools::Itertools;

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::{
        dataset::Sample,
        event::{self, ToolEvent, ToolResult},
        json::{JsonChange, JsonValue},
        log::{Attachments, SampleInput, Target},
        scorer::Value,
    },
    review::{
        components::toggle::ToggleView,
        screens::{
            common::{
                IntoAttrsView, attr, attr_label, attr_option, caption, nested_attrs,
                nested_attrs_option,
            },
            log::advanced::{
                chat_message::chat_message_view, error::error_view, input::input_view,
                json::json_value_view, output::output_view, target::target_view,
                tool::tool_info_view, util::text_content_view,
            },
        },
    },
};

pub fn event_view(event: &event::Event, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    match event {
        event::Event::SampleInit(e) => view.add_child(sample_init_event_view(e, attachments)),
        event::Event::SampleLimit(e) => view.add_child(sample_limit_event_view(e)),
        event::Event::Sandbox(e) => view.add_child(sandbox_event_view(e)),
        event::Event::State(e) => view.add_child(state_event_view(e, attachments)),
        event::Event::Store(e) => view.add_child(store_event_view(e)),
        event::Event::Model(e) => view.add_child(model_event_view(e, attachments)),
        event::Event::Tool(e) => view.add_child(tool_event_view(e)),
        event::Event::Approval(e) => view.add_child(approval_event_view(e)),
        event::Event::Input(e) => view.add_child(input_event_view(e)),
        event::Event::Score(e) => view.add_child(score_event_view(e)),
        event::Event::Error(e) => view.add_child(error_event_view(e)),
        event::Event::Logger(e) => view.add_child(logger_event_view(e)),
        event::Event::Info(e) => view.add_child(info_event_view(e)),
        event::Event::Step(e) => view.add_child(step_event_view(e)),
        event::Event::Subtask(e) => view.add_child(subtask_event_view(e)),
        event::Event::SpanBegin(_) => panic!("span_begin should not appear as events"),
        event::Event::SpanEnd(_) => panic!("span_end should not appear as events"),
    }
    view
}

fn base_event_view(event: &event::BaseEvent) -> impl View {
    [
        attr("event_name", &event.event_name),
        attr_option("uuid", event.uuid.as_ref()),
        attr_option("span_id", event.span_id.as_ref()),
        attr("timestamp", event.timestamp.to_iso_8601_local()),
        attr("working_start", event.working_start),
        attr_option("pending", event.pending.as_ref()),
        nested_attrs_option("metadata", event.metadata.as_ref()),
    ]
    .into_attrs_view()
}

fn sample_init_event_view(event: &event::SampleInitEvent, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    view.add_child(base_event_view(&event.base));

    view.add_child(caption("Sample").pad_t(1));
    view.add_child(sample_view(&event.sample, attachments));

    view.add_child(caption("State").pad_t(1));
    view.add_child(json_value_view(&event.state, attachments));

    view
}

fn sample_view(sample: &Sample, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();

    match &sample.input {
        SampleInput::String(s) => {
            view.add_child([attr("input", s)].into_attrs_view());
        }
        SampleInput::ChatMessageList(_) => {
            view.add_child(attr_label("input"));
            view.add_child(input_view(&sample.input, attachments));
        }
    }

    view.add_child(
        [
            attr_option("choices", sample.choices.as_ref().map(|v| v.join(", "))),
            attr_option("target", {
                let target = match &sample.target {
                    Target::String(s) => s.clone(),
                    Target::List(l) => l.join("\n"),
                };
                (!target.is_empty()).then_some(target)
            }),
            attr_option("id", sample.id.as_ref()),
            nested_attrs_option("metadata", sample.metadata.as_ref()),
        ]
        .into_attrs_view(),
    );

    view
}

fn sample_limit_event_view(event: &event::SampleLimitEvent) -> impl View {
    text_content_view(&format!("{event:#?}"))
}

fn sandbox_event_view(event: &event::SandboxEvent) -> impl View {
    let mut view = PageLayout::new();
    view.add_child(base_event_view(&event.base));
    view.add_child(
        [
            attr("action", &event.action),
            attr_option("file", event.file.as_ref()),
            nested_attrs_option("options", event.options.as_ref()),
            attr_option("result", event.result.as_ref()),
        ]
        .into_attrs_view(),
    );

    if let Some(cmd) = event.cmd.as_ref() {
        view.add_child(caption("Command").pad_t(1));
        view.add_child(text_content_view(cmd));
    }

    if let Some(input) = event.input.as_ref() {
        view.add_child(caption("Input").pad_t(1));
        view.add_child(text_content_view(input));
    }

    if let Some(output) = event.output.as_ref() {
        view.add_child(caption("Output").pad_t(1));
        view.add_child(text_content_view(output));
    }

    view
}

fn state_event_view(event: &event::StateEvent, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    view.add_child(base_event_view(&event.base));

    for (i, change) in event.changes.iter().enumerate() {
        view.add_child(
            ToggleView::new(
                format!("{} {}", change.op, change.path),
                state_change_view(change, attachments).pad_lrtb(3, 0, 0, 1),
            )
            .pad_t(if i == 0 { 1 } else { 0 }),
        );
    }

    view
}

fn state_change_view(change: &JsonChange, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();

    view.add_child([attr("op", &change.op), attr("path", &change.path)].into_attrs_view());
    if let Some(location) = change.from_location.as_ref() {
        view.add_child([attr("from_location", location)].into_attrs_view())
    }

    if let JsonValue::None = &change.value {
    } else {
        view.add_child(caption("Value").pad_t(1));
        view.add_child(json_value_view(&change.value, attachments));
    }

    if let JsonValue::None = &change.replaced {
    } else {
        view.add_child(caption("Replaced").pad_t(1));
        view.add_child(json_value_view(&change.replaced, attachments));
    }

    view
}

fn store_event_view(event: &event::StoreEvent) -> impl View {
    text_content_view(&format!("{event:#?}"))
}

fn model_event_view(event: &event::ModelEvent, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();

    view.add_child(base_event_view(&event.base));

    view.add_child(
        [
            attr("model", &event.model),
            attr_option("role", event.role.as_ref()),
            attr_option("retries", event.retries.as_ref()),
            attr_option("error", event.error.as_ref()),
            attr_option(
                "completed",
                event.completed.as_ref().map(|v| v.to_iso_8601_local()),
            ),
            attr_option("working_time", event.working_time.as_ref()),
            attr("tool_choice", &event.tool_choice),
            attr_option("cache", event.cache.as_ref()),
        ]
        .into_attrs_view(),
    );

    view.add_child(
        ToggleView::new("Input", {
            let mut view = PageLayout::new();
            for (i, msg) in event.input.iter().enumerate() {
                view.add_child(chat_message_view(msg, attachments).pad_t(min(i, 1)));
            }
            view.pad_lrtb(3, 0, 0, 1)
        })
        .pad_t(1),
    );

    view.add_child(ToggleView::new(
        "Output",
        output_view(&event.output, attachments).pad_lrtb(3, 0, 0, 1),
    ));

    // TODO - event.call

    if !event.tools.is_empty() {
        view.add_child(ToggleView::new("Tools", {
            let mut view = PageLayout::new();
            for tool in event.tools.iter() {
                view.add_child(ToggleView::new(
                    &tool.name,
                    tool_info_view(tool).pad_lrtb(3, 0, 0, 1),
                ));
            }
            view.pad_l(3)
        }));
    }

    view
}

pub fn tool_event_view(event: &ToolEvent) -> impl View {
    let mut view = PageLayout::new();

    view.add_child(base_event_view(&event.base));
    view.add_child(
        [
            attr("call_type", &event.call_type),
            attr("id", &event.id),
            attr("function", &event.function),
            nested_attrs("arguments", &event.arguments),
            attr_option(
                "completed",
                event.completed.as_ref().map(|v| v.to_iso_8601_local()),
            ),
            attr_option("working_time", event.working_time),
            attr_option("agent", event.agent.as_ref()),
            attr_option("failed", event.failed.as_ref()),
            attr_option("message_id", event.message_id.as_ref()),
            attr_option(
                "truncated",
                event.truncated.as_ref().map(|(a, b)| format!("({a}, {b})")),
            ),
        ]
        .into_attrs_view(),
    );

    if let Some(v) = event.view.as_ref() {
        view.add_child(caption("View").pad_t(1));
        view.add_child(
            [
                attr_option("title", v.title.as_ref()),
                attr("format", &v.format),
                attr("content", &v.content),
            ]
            .into_attrs_view(),
        );
    }

    view.add_child(caption("Result").pad_t(1));
    view.add_child(text_content_view(&match &event.result {
        ToolResult::String(s) => s.clone(),
        ToolResult::Int(i) => i.to_string(),
        ToolResult::Float(f) => f.to_string(),
        ToolResult::Text(t) => t.text.clone(),
        _ => format!("{:#?}", event.result),
    }));

    if let Some(error) = event.error.as_ref() {
        view.add_child(caption("Error").pad_t(1));
        view.add_child(
            [
                attr("error_type", &error.error_type),
                attr("message", &error.message),
            ]
            .into_attrs_view(),
        );
    }

    view
}

fn approval_event_view(event: &event::ApprovalEvent) -> impl View {
    // TODO
    text_content_view(&format!("{event:#?}"))
}

fn input_event_view(event: &event::InputEvent) -> impl View {
    // TODO
    text_content_view(&format!("{event:#?}"))
}

fn score_event_view(event: &event::ScoreEvent) -> impl View {
    let mut view = PageLayout::new();

    view.add_child(base_event_view(&event.base));
    view.add_child([attr("intermediate", event.intermediate)].into_attrs_view());

    // Score
    view.add_child(caption("Score").pad_t(1));
    let score = &event.score;
    let mut score_attrs = Vec::new();
    match &score.value {
        Value::Scalar(v) => score_attrs.push(attr("value", v)),
        Value::Sequence(v) => {
            score_attrs.push(attr("value", v.iter().map(|v| v.to_string()).join(", ")))
        }
        Value::Map(m) => score_attrs.extend(m.iter().map(|(name, v)| attr(name, v.to_string()))),
    }
    score_attrs.extend([
        attr_option("answer", score.answer.as_ref()),
        attr_option("explanation", score.explanation.as_ref()),
        nested_attrs_option("metadata", score.metadata.as_ref()),
    ]);
    view.add_child(score_attrs.into_attrs_view());

    // Target
    if let Some(target) = event.target.as_ref() {
        view.add_child(caption("Target").pad_t(1));
        view.add_child(target_view(target));
    }

    view
}

fn error_event_view(event: &event::ErrorEvent) -> impl View {
    let mut view = PageLayout::new();

    view.add_child(base_event_view(&event.base));
    view.add_child(error_view(&event.error));

    view
}

fn logger_event_view(event: &event::LoggerEvent) -> impl View {
    // TODO
    text_content_view(&format!("{event:#?}"))
}

fn info_event_view(event: &event::InfoEvent) -> impl View {
    // TODO
    text_content_view(&format!("{event:#?}"))
}

fn step_event_view(event: &event::StepEvent) -> impl View {
    // TODO
    text_content_view(&format!("{event:#?}"))
}

fn subtask_event_view(event: &event::SubtaskEvent) -> impl View {
    // TODO
    text_content_view(&format!("{event:#?}"))
}
