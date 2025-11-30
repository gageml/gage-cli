use cursive::{
    theme::{BaseColor, Effect},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::TextView,
    wrap_impl,
};

use crate::inspect::log::Target;

pub struct TargetView {
    inner: TextView,
}

impl TargetView {
    pub fn new(target: &Target) -> Self {
        let target = match target {
            Target::String(s) => s.clone(),
            Target::List(l) => l.join("\n\n"),
        };
        let inner = TextView::new(if target.is_empty() {
            StyledString::styled("<not specified>", Effect::Dim)
        } else {
            StyledString::styled(target, BaseColor::Cyan.light())
        });
        Self { inner }
    }
}

impl ViewWrapper for TargetView {
    wrap_impl!(self.inner: TextView);
}
