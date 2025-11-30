use cursive::View;

use crate::{
    cursive::{view::Padding, views::PageLayout},
    inspect::tool::ToolInfo,
    review::screens::common::{IntoAttrsView, attr, caption, nested_attrs, nested_attrs_option},
};

pub fn tool_info_view(tool: &ToolInfo) -> impl View {
    let mut view = PageLayout::new();

    view.add_child(
        [
            // Name handled by caller/parent
            attr("description", &tool.description),
            nested_attrs_option("options", tool.options.as_ref()),
        ]
        .into_attrs_view(),
    );

    view.add_child(caption("Parameters").pad_t(1));
    for (name, prop) in tool.parameters.properties.iter() {
        view.add_child(
            [nested_attrs(
                name,
                [
                    (
                        &String::from("description"),
                        prop.description.as_deref().unwrap_or_default(),
                    ),
                    (
                        &String::from("type"),
                        prop.json_type.as_deref().unwrap_or_default(),
                    ),
                    (
                        &String::from("format"),
                        prop.format.as_deref().unwrap_or_default(),
                    ),
                    (&String::from("default"), &prop.default.to_string()),
                    (
                        &String::from("required"),
                        &prop
                            .required
                            .as_ref()
                            .map(|v| v.join(", "))
                            .unwrap_or_default(),
                    ),
                ],
            )]
            .into_attrs_view(),
        );
    }

    view
}
