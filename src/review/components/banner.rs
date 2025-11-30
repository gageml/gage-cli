use cursive::{
    theme::{BaseColor, ColorStyle},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::{Layer, PaddedView, TextView},
    wrap_impl,
};

use crate::cursive::view::Padding;

pub struct Banner {
    inner: Layer<PaddedView<TextView>>,
}

impl Banner {
    pub fn empty() -> Self {
        Self {
            inner: Layer::with_color(
                TextView::empty().center().pad_x(2),
                ColorStyle::back(BaseColor::Red.dark()),
            ),
        }
    }

    pub fn new<S: Into<StyledString>>(msg: S) -> Self {
        let mut view = Self::empty();
        view.set_content(msg);
        view
    }

    pub fn set_content<S: Into<StyledString>>(&mut self, msg: S) {
        self.text_mut().set_content(msg);
    }

    pub fn clear(&mut self) {
        self.text_mut().set_content("");
    }

    fn text_mut(&mut self) -> &mut TextView {
        self.inner.get_inner_mut().get_inner_mut()
    }
}

impl ViewWrapper for Banner {
    wrap_impl!(self.inner: Layer<PaddedView<TextView>>);
}
