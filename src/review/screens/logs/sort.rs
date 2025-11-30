use cursive::{
    Cursive, View,
    align::HAlign,
    event::{Event, EventResult, Key},
    theme::Effect,
    utils::markup::StyledString,
    view::ViewWrapper,
    views::PaddedView,
    wrap_impl,
};

use crate::{
    cursive::{view::Padding, views::Dialog},
    handle_wrapped_dialog_event,
    review::{
        App, AppScreen,
        components::table::Sort,
        dialogs::help::HelpView,
        screens::logs::{LogsScreen, view::Col},
    },
};

pub struct SortDialog {
    inner: PaddedView<Dialog>,
}

impl SortDialog {
    pub fn new(active_sort: Option<Sort<Col>>) -> Self {
        macro_rules! col {
            ($col:expr, $label:literal) => {
                StyledString::concatenate([
                    $label.into(),
                    if let Some(sort) = active_sort.as_ref()
                        && sort.col() == &$col
                    {
                        StyledString::styled(" (toggle)", Effect::Dim)
                    } else {
                        "".into()
                    },
                ])
            };
        }
        let commands = vec![
            (
                None,
                vec![
                    ("t", col!(Col::Task, "Task")),
                    ("c", col!(Col::Created, "Created")),
                    ("s", col!(Col::Status, "Status")),
                    ("m", col!(Col::Model, "Model")),
                    ("d", col!(Col::Dataset, "Dataset")),
                    ("r", col!(Col::Score, "Score")),
                    ("n", col!(Col::Dataset, "Number of samples")),
                ],
            ),
            (None, vec![("Esc", "Cancel".into())]),
        ];
        let inner = Dialog::around(HelpView::new(commands))
            .padding_lrtb(4, 4, 1, 1)
            .title("Sort")
            .h_align(HAlign::Center)
            .pad_x(1);
        Self { inner }
    }

    fn on_sort(col: Col) -> EventResult {
        EventResult::with_cb(move |siv| Self::toggle_sort(siv, col.clone()))
    }

    fn toggle_sort(siv: &mut Cursive, col: Col) {
        App::with_screen(siv, AppScreen::Logs, move |screen: &mut LogsScreen| {
            let col = col.clone();
            if let Some(sort) = screen.get_sort()
                && sort.col() == &col
            {
                screen.set_sort(sort.reversed());
            } else {
                screen.set_sort(Sort::default(col));
            }
            // screen.refresh_footer();  // TODO make screen xxx_mut() private and wrap with explicit API
        });
        siv.pop_layer();
    }

    fn close(siv: &mut Cursive) {
        siv.pop_layer();
    }
}

impl ViewWrapper for SortDialog {
    wrap_impl!(self.inner: PaddedView<Dialog>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('t') => Self::on_sort(Col::Task),
            Event::Char('c') => Self::on_sort(Col::Created),
            Event::Char('s') => Self::on_sort(Col::Status),
            Event::Char('m') => Self::on_sort(Col::Model),
            Event::Char('d') => Self::on_sort(Col::Dataset),
            Event::Char('n') => Self::on_sort(Col::Samples),
            Event::Char('r') => Self::on_sort(Col::Score),
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb(Self::close),
            Event::Key(Key::Esc) => EventResult::with_cb(Self::close),
            _ => handle_wrapped_dialog_event!(self, event),
        }
    }
}
