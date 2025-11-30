use cursive::{
    Cursive, View,
    align::HAlign,
    event::{Event, EventResult, Key},
    theme::{Effect, Style},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::PaddedView,
    wrap_impl,
};
use itertools::Itertools;

use crate::{
    cursive::{view::Padding, views::Dialog},
    handle_wrapped_dialog_event,
    review::{
        App, AppScreen,
        dialogs::help::HelpView,
        screens::logs::{
            LogsScreen,
            view::{Col, Filter},
        },
        theme,
    },
};

pub struct FilterDialog {
    filter: Filter,
    inner: PaddedView<Dialog>,
}

impl FilterDialog {
    pub fn new(filter: Filter) -> Self {
        let keys = vec![
            (
                None,
                [Col::Task, Col::Type, Col::Status, Col::Model, Col::Dataset]
                    .to_vec()
                    .iter()
                    .filter_map(|col| Self::col_key(&filter, col))
                    .collect_vec(),
            ),
            (None, vec![("c", "Clear filter".into())]),
            (None, vec![("Esc", "Cancel".into())]),
        ];
        let inner = Dialog::around(HelpView::new(keys))
            .padding_lrtb(4, 4, 1, 1)
            .title("Filter")
            .h_align(HAlign::Center)
            .pad_x(1);
        Self { filter, inner }
    }

    fn col_key(filter: &Filter, col: &Col) -> Option<(&'static str, StyledString)> {
        macro_rules! key {
            ($key:literal, $label:literal, $attr:ident) => {
                filter
                    .$attr
                    .as_ref()
                    .map(|s| ($key, Self::fmt_col_key_desc($label, s)))
            };
        }
        match col {
            Col::Task => key!("t", "Task", task),
            Col::Type => key!("y", "Type", run_type),
            Col::Status => key!("s", "Status", status),
            Col::Model => key!("m", "Model", model),
            Col::Dataset => key!("d", "Dataset", dataset),
            _ => panic!("{col:?}"),
        }
    }

    fn fmt_col_key_desc(label: &str, val: &str) -> StyledString {
        StyledString::concatenate([
            format!("{label:9}").into(),
            StyledString::styled(
                if val.is_empty() { "<empty>" } else { val },
                Style::from_color_style(theme::Style::help_highlight()).combine(Effect::Dim),
            ),
        ])
    }

    fn on_clear(&self) -> EventResult {
        EventResult::with_cb(|siv| {
            App::with_screen(siv, AppScreen::Logs, |screen: &mut LogsScreen| {
                screen.clear_filter();
            });
            siv.pop_layer();
        })
    }

    fn on_apply(&self, col: Col) -> EventResult {
        let filter = Filter::from_col(&self.filter, col);
        EventResult::with_cb(move |siv| {
            let filter = filter.clone();
            App::with_screen(siv, AppScreen::Logs, move |screen: &mut LogsScreen| {
                let mut filter = filter.clone();
                if let Some(cur_filter) = screen.get_filter() {
                    filter.merge(cur_filter);
                }
                screen.set_filter(filter);
            });
            siv.pop_layer();
        })
    }

    fn close(siv: &mut Cursive) {
        siv.pop_layer();
    }
}

impl ViewWrapper for FilterDialog {
    wrap_impl!(self.inner: PaddedView<Dialog>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('t') => self.on_apply(Col::Task),
            Event::Char('y') => self.on_apply(Col::Type),
            Event::Char('s') => self.on_apply(Col::Status),
            Event::Char('m') => self.on_apply(Col::Model),
            Event::Char('d') => self.on_apply(Col::Dataset),
            Event::Char('c') => self.on_clear(),
            Event::Char('q') | Event::Char('Q') => EventResult::with_cb(Self::close),
            Event::Key(Key::Esc) => EventResult::with_cb(Self::close),
            _ => handle_wrapped_dialog_event!(self, event),
        }
    }
}
