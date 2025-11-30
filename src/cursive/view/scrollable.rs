use crate::cursive::views::ScrollView;
use cursive::view::View;

/// Makes a view wrappable in a [`ScrollView`].
///
/// [`ScrollView`]: crate::views::ScrollView
pub trait Scrollable: View + Sized {
    /// Wraps `self` in a `ScrollView`.
    fn scrollable(self) -> ScrollView<Self> {
        ScrollView::new(self)
    }
}

impl<T: View> Scrollable for T {}
