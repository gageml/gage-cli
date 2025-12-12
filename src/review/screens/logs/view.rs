use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use cursive::{
    ScreenId,
    event::EventResult,
    theme::{BaseColor, Effect, Style},
    utils::markup::StyledString,
    view::ViewWrapper,
    views::{BoxedView, TextView},
    wrap_impl,
};
use itertools::Itertools;
use pyo3::Python;

use crate::{
    cursive::views::ScreensView,
    inspect::log::{EvalLog, list_logs, read_log_header},
    py,
    result::Result,
    review::{
        App, AppScreen,
        components::table::{DefaultSortDir, Sort, SortDir, TableColExt, TableView},
        screens::{common::StyledEvalLog, log::LogScreen},
    },
    util::fit_path_name,
};

#[derive(Clone, PartialEq, Debug)]
pub enum Col {
    Id,
    Task,
    Type,
    Status,
    Model,
    Dataset,
    Score,
    Samples,
    Created,
}

#[derive(Clone, Debug, Default)]
pub struct Filter {
    pub task: Option<String>,
    pub run_type: Option<String>,
    pub status: Option<String>,
    pub model: Option<String>,
    pub dataset: Option<String>,
}

impl Filter {
    pub fn from_col(value: &Self, col: Col) -> Self {
        let mut filter = Filter::default();
        match col {
            Col::Id => {}
            Col::Task => filter.task = value.task.clone(),
            Col::Type => filter.run_type = value.run_type.clone(),
            Col::Status => filter.status = value.status.clone(),
            Col::Model => filter.model = value.model.clone(),
            Col::Dataset => filter.dataset = value.dataset.clone(),
            Col::Score => {}
            Col::Samples => {}
            Col::Created => {}
        };
        filter
    }

    pub fn filter(&self, item: &EvalLog) -> bool {
        // AND the specified field criteria
        if let Some(task) = self.task.as_ref()
            && item.eval.task != *task
        {
            return false;
        }
        if let Some(run_type) = self.run_type.as_ref() {
            let eval_run_type = item
                .eval
                .run_type()
                .as_deref()
                .unwrap_or_default()
                .to_string();
            if eval_run_type != *run_type {
                return false;
            }
        }
        if let Some(status) = self.status.as_ref()
            && item.status.to_string() != *status
        {
            return false;
        }
        if let Some(model) = self.model.as_ref()
            && item.eval.model != *model
        {
            return false;
        }
        if let Some(dataset) = self.dataset.as_ref() {
            let dataset_name = item
                .eval
                .dataset
                .name
                .as_deref()
                .unwrap_or_default()
                .to_string();
            if dataset_name != *dataset {
                return false;
            }
        }
        true
    }

    pub fn merge(&mut self, other: &Self) {
        macro_rules! merge {
            ($attr:ident) => {
                if let Some(val) = other.$attr.as_ref() {
                    self.$attr = Some(val.clone());
                }
            };
        }
        merge!(task);
        merge!(status);
        merge!(model);
        merge!(dataset);
    }
}

impl From<&EvalLog> for Filter {
    fn from(value: &EvalLog) -> Self {
        Self {
            task: Some(value.eval.task.clone()),
            run_type: Some(
                value
                    .eval
                    .run_type()
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
            ),
            status: Some(value.status.to_string()),
            model: Some(value.eval.model.clone()),
            dataset: Some(
                value
                    .eval
                    .dataset
                    .name
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
            ),
        }
    }
}

type LogsTable = TableView<EvalLog, Col>;

impl TableColExt<EvalLog> for Col {
    fn fmt(&self, log: &EvalLog) -> impl Into<StyledString> {
        match self {
            Self::Id => StyledString::styled(fmt_log_id(log), Effect::Dim),
            Self::Task => log.styled_task(),
            Self::Type => log.styled_run_type(),
            Self::Status => log.styled_status(),
            Self::Model => fit_path_name(log.eval.model.as_str(), 20).into(),
            Self::Dataset => {
                fit_path_name(log.eval.dataset.name.as_deref().unwrap_or_default(), 20).into()
            }
            Self::Score => log.styled_score(),
            Self::Samples => log
                .eval
                .dataset
                .evaluated_count()
                .map(|n| n.to_string())
                .unwrap_or_default()
                .into(),
            Self::Created => log.styled_created(),
        }
    }

    fn cmp(&self, lhs: &EvalLog, rhs: &EvalLog) -> Ordering {
        match self {
            Self::Id => lhs.eval.task_id.cmp(&rhs.eval.task_id),
            Self::Task => lhs.eval.task.cmp(&rhs.eval.task),
            Self::Type => lhs.eval.run_type().into_iter().cmp(rhs.eval.run_type()),
            Self::Status => lhs.status.cmp(&rhs.status),
            Self::Model => lhs.eval.model.cmp(&rhs.eval.model),
            Self::Dataset => lhs.eval.dataset.name.cmp(&rhs.eval.dataset.name),
            Self::Score => lhs
                .results
                .as_ref()
                .and_then(|r| r.first_accuracy().map(|f| (f * 1000.) as i64))
                .into_iter()
                .cmp(
                    rhs.results
                        .as_ref()
                        .and_then(|r| r.first_accuracy().map(|f| (f * 1000.) as i64)),
                ),
            Self::Samples => lhs
                .eval
                .dataset
                .evaluated_count()
                .cmp(&rhs.eval.dataset.evaluated_count()),
            Self::Created => lhs.eval.created.cmp(&rhs.eval.created),
        }
    }
}

impl DefaultSortDir for Col {
    fn default_sort(&self) -> SortDir {
        match self {
            Self::Score => SortDir::Desc,
            Self::Samples => SortDir::Desc,
            Self::Created => SortDir::Desc,
            _ => SortDir::Asc,
        }
    }
}

fn fmt_log_id(log: &EvalLog) -> &str {
    let id = &log.eval.task_id;
    match id.split_at_checked(4) {
        Some((s, _)) => s,
        None => id,
    }
}

pub struct LogsView {
    log_dir: PathBuf,
    filter: Option<Filter>,
    table_screen: ScreenId,
    error_screen: ScreenId,
    inner: ScreensView,
}

impl LogsView {
    pub fn new(log_dir: &Path) -> Self {
        let mut inner = ScreensView::new();
        let table_screen = inner.add_screen(BoxedView::new(Box::new(
            LogsTable::new()
                .col(Col::Id, "Id")
                .col(Col::Task, "Task")
                .col(Col::Type, "Type")
                .col(Col::Status, "Status")
                .col(Col::Model, "Model")
                .col(Col::Dataset, "Dataset")
                .col(Col::Score, "Score")
                .col(Col::Samples, "Samples")
                .col(Col::Created, "Created")
                .sort(Sort::desc(Col::Created))
                .empty_msg(Self::default_empty_msg())
                .on_select(|log: &EvalLog| {
                    let location = log.location.clone();
                    EventResult::with_cb(move |siv| {
                        let location = location.clone();
                        App::with_screen(siv, AppScreen::Log, move |screen: &mut LogScreen| {
                            screen.set_log_location(&location);
                        });
                        App::push_screen(siv, AppScreen::Log);
                    })
                }),
        )));
        let error_screen = inner.add_screen(BoxedView::new(Box::new(TextView::empty())));
        let mut view = Self {
            log_dir: log_dir.into(),
            filter: None,
            table_screen,
            error_screen,
            inner,
        };
        view.refresh_items();
        view
    }

    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    fn default_empty_msg() -> StyledString {
        StyledString::concatenate([
            StyledString::styled(
                "Nothing to review (",
                Style::from(Effect::Italic).combine(Effect::Dim),
            ),
            StyledString::styled(
                "r",
                Style::from(BaseColor::Cyan.light()).combine(Effect::Italic),
            ),
            StyledString::styled(
                " to refresh)",
                Style::from(Effect::Italic).combine(Effect::Dim),
            ),
        ])
    }

    pub fn refresh_items(&mut self) {
        match Self::items(&self.log_dir) {
            Ok(items) => {
                let items = if let Some(filter) = self.filter.as_ref() {
                    items
                        .into_iter()
                        .filter(|item| filter.filter(item))
                        .collect_vec()
                } else {
                    items
                };
                let table = self.table_mut();
                if let Some(active_id) = table.get_active().map(|item| item.eval.task_id.clone()) {
                    table.set_items_with_active(items, |item| item.eval.task_id == active_id);
                } else {
                    table.set_items(items);
                }
            }
            Err(e) => self.set_error(&format!("Error loading logs: {e:?}")),
        }
    }

    fn items(log_dir: &Path) -> Result<Vec<EvalLog>> {
        py::init();
        Python::attach(|py| {
            Ok(list_logs(py, log_dir)?
                .into_iter()
                .filter_map(|log_info| match read_log_header(py, &log_info.name) {
                    Ok(log) => Some(log),
                    Err(err) => {
                        log::error!("Reading {}: {:?}", log_info.name, err);
                        None
                    }
                })
                .collect_vec())
        })
    }

    fn set_error(&mut self, msg: &str) {
        self.inner.set_active_screen(self.error_screen);
        self.inner
            .screen_mut()
            .unwrap()
            .downcast_mut::<TextView>()
            .unwrap()
            .set_content(msg);
    }

    pub fn set_sort(&mut self, sort: Sort<Col>) {
        self.table_mut().set_sort(sort);
    }

    pub fn get_sort(&self) -> Option<&Sort<Col>> {
        self.table().get_sort()
    }

    pub fn get_active(&self) -> Option<&EvalLog> {
        self.table().get_active()
    }

    fn table(&self) -> &TableView<EvalLog, Col> {
        self.inner
            .get_screen(self.table_screen)
            .unwrap()
            .downcast_ref::<LogsTable>()
            .unwrap()
    }

    fn table_mut(&mut self) -> &mut LogsTable {
        self.inner
            .get_screen_mut(self.table_screen)
            .unwrap()
            .downcast_mut::<LogsTable>()
            .unwrap()
    }

    pub fn get_filter(&self) -> Option<&Filter> {
        self.filter.as_ref()
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = Some(filter);
        self.refresh_items();
    }

    pub fn clear_filter(&mut self) {
        self.filter = None;
        self.refresh_items();
    }
}

impl ViewWrapper for LogsView {
    wrap_impl!(self.inner: ScreensView);
}
