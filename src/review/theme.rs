use cursive::{
    With,
    style::BaseColor,
    theme::{Color, ColorStyle, ColorType, Effect, Theme},
};

pub fn default() -> Theme {
    Theme::terminal_default().with(|t| {
        t.shadow = false;
        // Colors
        {
            use cursive::style::PaletteColor::Highlight;
            t.palette[Highlight] = BaseColor::Blue.light();
        }

        // Styles
        {
            // use cursive::style::PaletteStyle::TitlePrimary;
            // t.palette[TitlePrimary] = Style::from(Blue.light()).combine(Effect::Reverse);
        }
    })
}

pub struct Style;

impl Style {
    pub fn panel() -> ColorStyle {
        ColorStyle::new(BaseColor::White.dark(), BaseColor::Black.dark())
    }

    pub fn panel_border() -> cursive::theme::Style {
        cursive::theme::Style::from(ColorStyle::new(
            BaseColor::Black.dark(), // Set to bg for no border
            BaseColor::Black.dark(),
        ))
    }

    pub fn panel_focus_border() -> cursive::theme::Style {
        cursive::theme::Style::from(ColorStyle::new(
            BaseColor::Yellow.dark(),
            BaseColor::Black.dark(),
        ))
        .combine(Effect::Dim)
    }

    // pub fn panel_caption() -> Effect {
    //     Effect::Dim
    // }

    // pub fn caption() -> Effect {
    //     Effect::Dim
    // }

    // pub fn value_highlight() -> ColorStyle {
    //     ColorStyle::front(BaseColor::Cyan.light())
    // }

    pub fn header_title() -> ColorStyle {
        ColorStyle::new(BaseColor::Black.dark(), BaseColor::White.dark())
    }

    pub fn footer_key() -> ColorType {
        ColorType::highlight()
    }

    pub fn footer_highlight() -> ColorStyle {
        ColorStyle::front(BaseColor::Yellow.dark())
    }

    pub fn help_highlight() -> ColorStyle {
        ColorStyle::front(BaseColor::Yellow.light())
    }

    pub fn footer_caption() -> Effect {
        Effect::Dim
    }

    pub fn footer_sep() -> Color {
        BaseColor::Black.light()
    }
}
