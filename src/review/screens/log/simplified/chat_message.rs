use std::cmp::min;

use cursive::{view::ViewWrapper, wrap_impl};

use crate::{
    cursive::{
        view::Padding,
        views::{PageLayout, PlainTextView},
    },
    inspect::{
        log::Attachments,
        model::{ChatMessage, ChatMessageContent, Content},
    },
    review::screens::common::{IntoAttrsView, attr, attr_option, caption, nested_attrs},
};

pub struct ChatMessageView {
    inner: PageLayout,
}

impl ChatMessageView {
    pub fn new(msg: &ChatMessage, attachments: &Attachments) -> Self {
        let mut inner = PageLayout::new();

        // Base attrs
        let mut attrs = Vec::new();
        if let Some(source) = msg.base().source.as_ref() {
            attrs.push(attr("source", source));
        }
        if let Some(metadata) = msg.base().metadata.as_ref() {
            attrs.push(nested_attrs("metadata", metadata))
        }

        // Message type specific attrs
        match msg {
            ChatMessage::Assistant(assistant) => {
                if let Some(model) = assistant.model.as_ref() {
                    attrs.extend([attr("model", model)]);
                }
            }
            ChatMessage::Tool(tool) => {
                if let Some(id) = tool.tool_call_id.as_ref() {
                    attrs.extend([attr("tool_call_id", id)]);
                }
                if let Some(function) = tool.function.as_ref() {
                    attrs.extend([attr("function", function)]);
                }
            }
            _ => {}
        }

        if !attrs.is_empty() {
            inner.add_child(attrs.into_attrs_view());
        }

        // Assistant tool calls
        if let ChatMessage::Assistant(assistant) = msg
            && let Some(tool_calls) = assistant.tool_calls.as_ref()
        {
            inner.add_child(caption("Tool calls").pad_t(if inner.is_empty() { 0 } else { 1 }));
            for (i, call) in tool_calls.iter().enumerate() {
                let attrs = [
                    attr("id", &call.id),
                    attr("function", &call.function),
                    nested_attrs("arguments", &call.arguments), // TODO
                    attr_option("parse_error", call.parse_error.as_ref()),
                    attr_option("view", call.view.as_ref().map(|v| format!("{v:?}"))), // TODO
                    attr("call_type", &call.call_type),
                ];
                inner.add_child(attrs.into_attrs_view().pad_t(min(i, 1)));
            }
        }

        // Content
        if !msg.base().content.is_empty() {
            inner.add_child(
                ChatMessageContentView::new(&msg.base().content, attachments)
                    .pad_t(if inner.is_empty() { 0 } else { 1 }),
            );
        }
        Self { inner }
    }
}

impl ViewWrapper for ChatMessageView {
    wrap_impl!(self.inner: PageLayout);
}

pub struct ChatMessageContentView {
    inner: PageLayout,
}

impl ChatMessageContentView {
    pub fn new(msg: &ChatMessageContent, _attachments: &Attachments) -> Self {
        let mut inner = PageLayout::new();
        match msg {
            ChatMessageContent::String(s) => {
                inner.add_child(PlainTextView::wrap(s));
            }
            ChatMessageContent::ContentList(l) => {
                // TODO - This is a quick pass at showing complex chat
                // message content - this should be formatted with
                // attrs, etc. and colors as needed to make sense of a
                // list of messages.
                for (i, content) in l.iter().enumerate() {
                    let s = match content {
                        Content::Text(t) => &t.text,
                        Content::Reasoning(r) => &format!(
                            "Reasoning: {}",
                            if r.redacted {
                                "[REDACTED]"
                            } else {
                                &r.reasoning
                            }
                        ),
                        // TODO - handle other content types as they come online
                        _ => &format!("{content:#?}"),
                    };
                    inner.add_child(PlainTextView::wrap(s).pad_t(min(i, 1)));
                }
            }
        };
        Self { inner }
    }
}

impl ViewWrapper for ChatMessageContentView {
    wrap_impl!(self.inner: PageLayout);
}
