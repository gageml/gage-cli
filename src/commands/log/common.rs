use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs,
    ops::{Range, RangeFrom},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

use cliclack as cli;
use console::style;
use pyo3::Python;
use tabled::{
    Table,
    builder::Builder,
    settings::{
        Color, Remove,
        object::{Cell, Columns, Object, Rows},
        themes::Colorization,
    },
};

use crate::{
    dialog::DialogResult,
    error::Error,
    inspect::log::{EvalLogInfo, LogFilter, list_logs_filter, read_log_header},
    py,
    result::Result,
    theme::Colors,
    util::{TableExt, fit_path_name, term_width, wrap},
};

pub struct LogOpDialog {
    title: String,
    log_dir: PathBuf,
    filter: LogFilter,
    log_specs: Vec<LogSpec>,
    show_prompt: bool,
    confirm_prompt: Option<Box<ConfirmPromptFn>>,
}

type ConfirmPromptFn = dyn Fn(&[SelectedLog]) -> String;

pub struct LogOpResult(usize);

pub trait LogOpSuccessMap {
    fn on_success<F>(self, f: F) -> Result<DialogResult>
    where
        F: Fn(usize) -> String;
}

impl LogOpSuccessMap for Result<LogOpResult> {
    fn on_success<F>(self, f: F) -> Result<DialogResult>
    where
        F: Fn(usize) -> String,
    {
        self.map(|r| DialogResult::Message(f(r.0)))
    }
}

impl LogOpDialog {
    pub fn new<S: Into<String>>(title: S) -> Self {
        Self {
            title: title.into(),
            log_dir: "logs".into(),
            filter: LogFilter::None,
            log_specs: Vec::new(),
            show_prompt: true,
            confirm_prompt: None,
        }
    }

    pub fn log_dir(mut self, log_dir: PathBuf) -> Self {
        self.log_dir = log_dir;
        self
    }

    pub fn filter(mut self, filter: LogFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn log_specs(mut self, log_specs: Vec<LogSpec>) -> Self {
        self.log_specs = log_specs;
        self
    }

    pub fn show_prompt(mut self, flag: bool) -> Self {
        self.show_prompt = flag;
        self
    }

    pub fn confirm_prompt<F>(mut self, f: F) -> Self
    where
        F: Fn(&[SelectedLog]) -> String + 'static,
    {
        self.confirm_prompt = Some(Box::new(f));
        self
    }

    pub fn run<F>(self, cb: F) -> Result<LogOpResult>
    where
        F: Fn(SelectedLog) -> Result<()>,
    {
        cli::intro(style(&self.title).bold())?;

        py::init();
        Python::attach(|py| {
            // Use spinner when reading logs
            let pb = cli::spinner();
            pb.start("Loading log information");
            let logs = list_logs_filter(py, &self.log_dir, self.filter)?;
            pb.clear();

            // Select logs for op
            let selected = LogSelect::select_logs(&logs, &self.log_specs)?
                .into_iter()
                .collect::<Vec<_>>();
            if selected.is_empty() {
                return Err(Error::custom("Log(s) not found"));
            }

            // Prompt user
            if self.show_prompt {
                let (table, _errors) = logs_table(py, selected.iter().map(|s| &s.inner), false);
                // Ignore log table errors as op can proceed regardless
                cli::log::remark(table.to_string())?;
                let msg = if let Some(confirm) = self.confirm_prompt {
                    confirm(&selected)
                } else {
                    "Continue?".into()
                };
                let confirmed = cli::confirm(wrap(&msg, term_width() - 4))
                    .initial_value(false)
                    .interact()?;
                if !confirmed {
                    return Err(Error::Canceled);
                }
            }

            // Perform op
            let count = selected.len();
            for log in selected.into_iter() {
                cb(log)?;
            }

            Ok(LogOpResult(count))
        })
    }
}

type ReadErrorLogIds = Vec<String>;

pub fn logs_table<'a, Logs>(py: Python<'_>, logs: Logs, index: bool) -> (Table, ReadErrorLogIds)
where
    Logs: Iterator<Item = &'a EvalLogInfo>,
{
    let now = chrono::Utc::now();
    let index_offset = if index { 0 } else { 1 };
    let mut colored_cells: Vec<(usize, usize, Color)> = Vec::new();
    let mut table = Builder::new();
    table.push_record([
        "#", "Id", "Task", "Type", "Status", "Model", "Dataset", "Started",
    ]);
    let mut errors = Vec::new();
    for (i, log) in logs.enumerate() {
        match read_log_header(py, &log.name) {
            Ok(header) => {
                let time = log
                    .mtime
                    .as_ref()
                    .map(|mtime| mtime.to_human_since(&now))
                    .unwrap_or_default();
                // status color
                if let Some(color) = status_color(&header.status.to_string()) {
                    colored_cells.push((i + 1, 4 - index_offset, color));
                }
                table.push_record([
                    (i + 1).to_string(),
                    log.short_log_id().into(),
                    log.task.clone(),
                    header.eval.run_type().unwrap_or_default(),
                    header.status.to_string(),
                    fit_path_name(&header.eval.model, 20),
                    fit_path_name(&header.eval.dataset.name.unwrap_or_default(), 20),
                    time,
                ]);
            }
            Err(err) => {
                errors.push(log.log_id.clone());
                log::error!("Error reading {}: {}", log.log_id, err);
                table.push_record([
                    (i + 1).to_string(),
                    log.short_log_id().into(),
                    log.task.clone(),
                    "?".into(),
                    "?".into(),
                    "?".into(),
                    "?".into(),
                    "?".into(),
                ]);
            }
        }
    }
    let mut table = table.build();

    // Standard formatting
    table.with_term_fit().with_rounded().with_col_labels();

    // Index, if included is cyan
    if index {
        table.with(Colorization::exact(
            [Color::FG_BRIGHT_CYAN],
            Columns::one(0).intersect(Rows::new(1..)),
        ));
    } else {
        table.with(Remove::column(Columns::one(0)));
    }

    // Id is dim
    table.with(Colorization::exact(
        [Colors::dim()],
        Columns::one(1 - index_offset).intersect(Rows::new(1..)),
    ));

    // Task is yellow
    table.with(Colorization::exact(
        [Color::FG_BRIGHT_YELLOW],
        Columns::one(2 - index_offset).intersect(Rows::new(1..)),
    ));

    // Type is dim
    table.with(Colorization::exact(
        [Colors::dim()],
        Columns::one(3 - index_offset).intersect(Rows::new(1..)),
    ));

    // Date is dim
    table.with(Colorization::exact(
        [Colors::dim()],
        Columns::one(7 - index_offset).intersect(Rows::new(1..)),
    ));

    // Apply colored cells (e.g. status)
    for (row, col, color) in colored_cells.into_iter() {
        table.with(Colorization::exact([color], Cell::new(row, col)));
    }

    (table, errors)
}

fn status_color(status: &str) -> Option<Color> {
    match status {
        "success" => None,
        "error" => Some(Color::FG_RED),
        _ => Some(Colors::dim()),
    }
}

pub struct LogSelect;

impl LogSelect {
    pub fn parse_specs(specs: &[String]) -> Result<Vec<LogSpec>> {
        specs.iter().map(|item| LogSpec::from_str(item)).collect()
    }

    pub fn select_logs(from: &[EvalLogInfo], specs: &[LogSpec]) -> Result<SelectedLogs> {
        let mut selected = BTreeSet::default();
        // If no specs, assume all
        let specs = if specs.is_empty() {
            &[LogSpec::TablePosRangeFrom(1..)]
        } else {
            specs
        };
        for spec in specs {
            selected.extend(spec.select_logs(from)?)
        }
        Ok(selected)
    }
}

pub struct SelectedLog {
    table_pos: usize,
    inner: EvalLogInfo,
}

impl Ord for SelectedLog {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.table_pos.cmp(&other.table_pos)
    }
}

impl PartialOrd for SelectedLog {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SelectedLog {
    fn eq(&self, other: &Self) -> bool {
        self.table_pos == other.table_pos
    }
}

impl Eq for SelectedLog {}

impl SelectedLog {
    pub fn new(table_pos: usize, log: EvalLogInfo) -> Self {
        Self {
            table_pos,
            inner: log,
        }
    }
}

impl SelectedLog {
    pub fn file_path(&self) -> Result<PathBuf> {
        resolve_file_name(&self.inner.name)
    }

    pub fn delete(self, permanent: bool) -> Result<()> {
        if permanent {
            permanent_deletion(self.file_path()?)
        } else {
            recoverable_deletion(self.file_path()?)
        }
    }

    pub fn restore(self) -> Result<()> {
        restore(self.file_path()?)
    }
}

#[derive(Debug)]
pub enum LogSpec {
    TablePos(usize),
    IdPrefix(String),
    TablePosRange(Range<usize>), // NOTE: Range is NOT inclusive
    TablePosRangeFrom(RangeFrom<usize>),
}

pub type SelectedLogs = BTreeSet<SelectedLog>;

impl LogSpec {
    pub fn select_logs(&self, logs_table: &[EvalLogInfo]) -> Result<SelectedLogs> {
        let mut selected = SelectedLogs::default();
        match self {
            LogSpec::TablePos(table_pos) => {
                let Some(log): Option<&EvalLogInfo> = logs_table.get(*table_pos - 1) else {
                    return Err(Error::custom("log index does not exist"));
                };
                selected.insert(SelectedLog::new(*table_pos, log.clone()));
            }
            LogSpec::TablePosRange(range) => {
                for table_pos in range.clone() {
                    let Some(log): Option<&EvalLogInfo> = logs_table.get(table_pos - 1) else {
                        return Err(Error::custom("log range does not exist"));
                    };
                    selected.insert(SelectedLog::new(table_pos, log.clone()));
                }
            }
            LogSpec::TablePosRangeFrom(range) => {
                for table_pos in range.clone() {
                    let Some(log): Option<&EvalLogInfo> = logs_table.get(table_pos - 1) else {
                        break;
                    };
                    selected.insert(SelectedLog::new(table_pos, log.clone()));
                }
            }
            LogSpec::IdPrefix(prefix) => {
                let matches = logs_table
                    .iter_table()
                    .filter(|(_, log)| log.log_id.starts_with(prefix))
                    .collect::<Vec<_>>();
                if matches.len() > 1 {
                    return Err(Error::custom(format!(
                        "Log spec '{}' matches more than one log: {}\n\
                        \n\
                        Use the full log ID instead.",
                        prefix,
                        matches
                            .iter()
                            .map(|(_, log)| log.log_id.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }
                if let Some((table_pos, log)) = matches.into_iter().next() {
                    selected.insert(SelectedLog::new(table_pos, log.clone()));
                }
            }
        }
        Ok(selected)
    }

    pub fn from_str(target: &str) -> Result<Self> {
        // Attempt to parse a numeric table position
        if let Ok(pos) = target.parse::<usize>() {
            Ok(Self::TablePos(pos))

        // Attempt to parse range in format START:END
        } else if let Some((start, end)) = target.split_once(':') {
            // Check that START is valid
            let start = match start.parse::<usize>() {
                Ok(pos) => pos,
                Err(_) if start.is_empty() => 1,
                Err(_) => {
                    return Err(Error::custom(format!(
                        "Invalid range value '{start}' - expected a number"
                    )));
                }
            };

            // Check that END is valid
            let spec = match end.parse::<usize>() {
                Ok(end) if end < start => Self::TablePosRange(end..start + 1),
                Ok(end) => Self::TablePosRange(start..end + 1),
                Err(_) if end.is_empty() => Self::TablePosRangeFrom(start..),
                Err(_) => {
                    return Err(Error::custom(format!(
                        "Invalid range value '{end}' - expected a number"
                    )));
                }
            };
            Ok(spec)

        // Use everything else as an eval ID part
        } else {
            Ok(Self::IdPrefix(target.to_owned()))
        }
    }
}

trait TableIter {
    fn iter_table(&self) -> impl Iterator<Item = (usize, &EvalLogInfo)>;
}

impl TableIter for &[EvalLogInfo] {
    fn iter_table(&self) -> impl Iterator<Item = (usize, &EvalLogInfo)> {
        self.iter().enumerate().map(|(i, log)| (i + 1, log))
    }
}

fn resolve_file_name(file_name: &str) -> Result<PathBuf> {
    if let Some(path) = file_name.strip_prefix("file://") {
        return Ok(path.into());
    }
    Err(Error::custom("log file type not supported"))
}

fn recoverable_deletion(file_path: PathBuf) -> Result<()> {
    let extension = file_path
        .extension()
        .map(OsStr::as_bytes)
        .unwrap_or("".as_bytes());
    let deleted_extension = if extension.is_empty() {
        String::from("deleted")
    } else {
        format!("{}.deleted", String::from_utf8_lossy(extension))
    };
    if fs::exists(file_path.with_extension(deleted_extension)).unwrap_or(true) {
        return Err(Error::Custom(format!(
            "Deleted file already exists for {:?}",
            file_path.file_name().unwrap_or_default()
        )));
    } else if extension != "deleted".as_bytes() {
        fs::rename(
            file_path.clone(),
            format!("{}.deleted", file_path.to_string_lossy()),
        )?;
    }
    Ok(())
}

fn restore(file_path: PathBuf) -> Result<()> {
    if file_path.extension().map(OsStr::as_bytes) == Some("deleted".as_bytes()) {
        let restored_file_path = file_path.with_extension("");
        if fs::exists(&restored_file_path).unwrap_or(true) {
            return Err(Error::Custom(format!(
                "Restored file already exists for {:?}",
                file_path.file_name().unwrap_or_default()
            )));
        }
        fs::rename(file_path.clone(), restored_file_path)?;
    }
    Ok(())
}

fn permanent_deletion(file_path: PathBuf) -> Result<()> {
    Ok(fs::remove_file(file_path)?)
}

#[macro_export]
macro_rules! plural {
    ($item:literal, $count:expr) => {{
        use icu_locale::locale;
        use icu_plurals::{PluralCategory, PluralRules};

        let pr = PluralRules::try_new(locale!("en").into(), Default::default())
            .expect("locale should be present");
        let mut base = String::from($item);
        let category = pr.category_for($count);
        if !matches!(category, PluralCategory::One) {
            base.push_str("s");
        }
        base
    }};
}
