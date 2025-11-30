use std::cmp::min;

use cursive::View;

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::{
        log::Attachments,
        model::{ChatMessage, ChatMessageContent, Content},
    },
    review::screens::{
        common::{IntoAttrsView, attr, attr_option, caption, nested_attrs, nested_attrs_option},
        log::advanced::{
            json::json_map_view,
            util::{resolve_attachment, text_content_view},
        },
    },
};

pub fn chat_message_view(msg: &ChatMessage, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();

    // Base attrs
    let mut attrs = vec![
        // parent responsible for showing role - don't duplicate here
        attr_option("id", msg.base().id.as_ref()),
        attr_option("source", msg.base().source.as_ref()),
        nested_attrs_option("metadata", msg.base().metadata.as_ref()),
    ];

    // Message type specific attrs
    match msg {
        ChatMessage::User(user) => {
            attrs.extend([attr_option(
                "tool_call_id",
                user.tool_call_id.as_ref().map(|v| v.join(", ")),
            )]);
        }
        ChatMessage::Assistant(assistant) => {
            attrs.extend([attr_option("model", assistant.model.as_ref())]);
        }
        ChatMessage::Tool(tool) => {
            attrs.extend([attr_option("tool_call_id", tool.tool_call_id.as_ref())]);
            attrs.extend([attr_option("function", tool.function.as_ref())]);
        }
        ChatMessage::System(_) => {}
    }
    view.add_child(attrs.into_attrs_view());

    // Assistant tool calls
    if let ChatMessage::Assistant(assistant) = msg
        && let Some(tool_calls) = assistant.tool_calls.as_ref()
    {
        view.add_child(caption("Tool calls").pad_t(1));
        for (i, call) in tool_calls.iter().enumerate() {
            let attrs = [
                attr("id", &call.id),
                attr("function", &call.function),
                nested_attrs("arguments", &call.arguments), // TODO
                attr_option("parse_error", call.parse_error.as_ref()),
                attr_option("view", call.view.as_ref().map(|v| format!("{v:?}"))), // TODO
                attr("call_type", &call.call_type),
            ];
            view.add_child(attrs.into_attrs_view().pad_t(min(i, 1)));
        }
    }

    // Content
    if !msg.base().content.is_empty() {
        view.add_child(chat_message_content_view(&msg.base().content, attachments).pad_t(1));
    }

    view
}

pub fn chat_message_content_view(
    content: &ChatMessageContent,
    attachments: &Attachments,
) -> impl View {
    let mut view = PageLayout::new();
    match content {
        ChatMessageContent::String(s) => {
            view.add_child(text_content_view(&resolve_attachment(s, attachments)))
        }
        ChatMessageContent::ContentList(l) => {
            for (i, content) in l.iter().enumerate() {
                view.add_child(content_view(content, attachments).pad_t(min(i, 1)));
            }
        }
    }
    view
}

fn content_view(content: &Content, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();
    match content {
        Content::Text(text) => {
            view.add_child(caption("Text"));
            view.add_child(text_content_view(&resolve_attachment(
                &text.text,
                attachments,
            )))
        }
        Content::Reasoning(reasoning) => {
            view.add_child(caption("Reasoning"));
            view.add_child(
                [
                    attr("reasoning", &reasoning.reasoning),
                    attr_option("summary", reasoning.summary.as_ref()),
                    attr_option("signature", reasoning.signature.as_ref()),
                    attr("redacted", reasoning.redacted),
                ]
                .into_attrs_view(),
            );
        }
        Content::Image(image) => {
            view.add_child(caption("Image"));
            view.add_child(
                [attr("image", &image.image), attr("detail", &image.detail)].into_attrs_view(),
            );
        }
        Content::Audio(audio) => {
            view.add_child(caption("Audio"));
            view.add_child(
                [attr("audio", &audio.audio), attr("format", &audio.format)].into_attrs_view(),
            );
        }
        Content::Video(video) => {
            view.add_child(caption("View"));
            view.add_child(
                [attr("video", &video.video), attr("format", &video.format)].into_attrs_view(),
            );
        }
        Content::Data(data) => {
            view.add_child(caption("Data"));
            view.add_child(json_map_view(&data.data, attachments));
        }
        Content::ToolUse(tool) => {
            view.add_child(caption("Tool use"));
            view.add_child(
                [
                    attr("tool_type", &tool.tool_type),
                    attr("id", &tool.id),
                    attr("name", &tool.name),
                    attr_option("context", tool.context.as_ref()),
                    attr("arguments", &tool.arguments),
                    attr("result", &tool.result),
                    attr_option("error", tool.error.as_ref()),
                ]
                .into_attrs_view(),
            );
        }
        Content::Document(doc) => {
            view.add_child(caption("Document"));
            view.add_child(
                [
                    attr("document", &doc.document),
                    attr("filename", &doc.filename),
                    attr("mime_type", &doc.mime_type),
                ]
                .into_attrs_view(),
            );
        }
    }
    view
}
