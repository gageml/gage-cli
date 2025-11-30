use cursive::{View, theme::BaseColor, utils::markup::StyledString, views::TextView};

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::{
        log::Attachments,
        model::{ChatCompletionChoice, ModelOutput},
    },
    review::{
        components::toggle::ToggleView,
        screens::{
            common::{
                IntoAttrsView, attr, attr_option, caption, nested_attrs, nested_attrs_option,
            },
            log::advanced::chat_message::chat_message_content_view,
        },
    },
};

pub fn output_view(output: &ModelOutput, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();

    view.add_child(
        [
            attr("model", &output.model),
            if let Some(usage) = output.usage.as_ref() {
                let usage_attrs = [
                    (&"input_tokens".to_string(), usage.input_tokens.to_string()),
                    (
                        &"output_tokens".to_string(),
                        usage.output_tokens.to_string(),
                    ),
                    (&"total_tokens".to_string(), usage.total_tokens.to_string()),
                    (
                        &"input_tokens_cache_write".to_string(),
                        usage
                            .input_tokens_cache_write
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        &"input_tokens_cache_read".to_string(),
                        usage
                            .input_tokens_cache_read
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        &"reasoning_tokens".to_string(),
                        usage
                            .reasoning_tokens
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    ),
                ];
                nested_attrs("usage", usage_attrs)
            } else {
                attr("usage", "")
            },
            attr(
                "time",
                if let Some(t) = output.time {
                    format!("{t:.2}")
                } else {
                    "".to_string()
                },
            ),
            nested_attrs_option("metadata", output.metadata.as_ref()),
        ]
        .into_attrs_view(),
    );

    // Show errror in red
    if let Some(error) = output.error.as_ref() {
        view.add_child(TextView::new(StyledString::styled(error, BaseColor::Red.dark())).pad_t(1));
    }

    if output.choices.len() > 1 {
        // If more than one choice show toggles
        for (i, choice) in output.choices.iter().enumerate() {
            view.add_child(
                ToggleView::new(format!("choices[{i}]"), choice_view(choice, attachments))
                    .pad_t(if i == 0 { 1 } else { 0 }),
            );
        }
    } else if output.choices.len() == 1 {
        // Otherwise show last choice directly
        view.add_child(caption("choices[0]").pad_t(1));
        view.add_child(choice_view(&output.choices[0], attachments).pad_l(2));
    }

    view
}

fn choice_view(choice: &ChatCompletionChoice, attachments: &Attachments) -> impl View {
    let mut view = PageLayout::new();

    view.add_child(
        [
            attr_option("id", choice.message.base.id.as_ref()),
            attr_option("source", choice.message.base.source.as_ref()),
            attr_option("model", choice.message.model.as_ref()),
            attr("stop_reason", &choice.stop_reason),
            nested_attrs_option("metadata", choice.message.base.metadata.as_ref()),
        ]
        .into_attrs_view(),
    );

    view.add_child(chat_message_content_view(&choice.message.base.content, attachments).pad_t(1));

    view
}
