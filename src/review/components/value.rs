use cursive::{
    theme::{BaseColor, ColorStyle, Effect, Style},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::TextView,
    wrap_impl,
};

pub struct ValueView {
    label: String,
    inner: TextView,
}

impl ValueView {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            inner: TextView::empty(),
        }
    }

    pub fn set_value(&mut self, value: &str) {
        self.set_styled(StyledString::styled(
            value,
            Style::from(ColorStyle::back(BaseColor::Black.dark())),
        ));
    }

    pub fn set_styled<S: Into<StyledString>>(&mut self, value: S) {
        let value = value.into();
        if !value.is_empty() {
            self.inner.set_content(StyledString::concatenate([
                StyledString::styled(
                    format!(" {} ", self.label),
                    Style::from(ColorStyle::back(BaseColor::Black.dark())).combine(Effect::Dim),
                ),
                value,
                StyledString::styled(
                    " ",
                    Style::from(ColorStyle::back(BaseColor::Black.dark())).combine(Effect::Dim),
                ),
            ]));
        } else {
            self.inner.set_content("");
        }
    }
}

impl ViewWrapper for ValueView {
    wrap_impl!(self.inner: TextView);
}
