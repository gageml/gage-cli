use std::{fs, path::PathBuf};

use cliclack as cli;
use console::style;
use tabled::{Table, builder::Builder, settings::Color};

use crate::{
    dialog::DialogResult,
    error::Error,
    inspect2::log::{EvalStatus, LogHeader, LogInfo},
    result::Result,
    theme::Colors,
    util::{TableExt, fit_path_name, relpath, term_width, wrap},
};

#[derive(Debug, PartialEq)]
pub enum SelectSpec {
    Index(usize),
    Range((usize, usize)),
    From(usize),
    Id(String),
}

impl SelectSpec {
    fn apply(&self, log: &LogInfo, table_index: usize) -> bool {
        match self {
            Self::Index(i) => table_index == *i,
            Self::Range((start, end)) => table_index >= *start && table_index <= *end,
            Self::From(start) => table_index >= *start,
            Self::Id(prefix) => log.log_id.starts_with(prefix),
        }
    }
}

pub enum SelectSpecParseError {
    Start(String),
    End(String),
    Order(usize, usize),
}

impl TryFrom<&String> for SelectSpec {
    type Error = SelectSpecParseError;

    fn try_from(value: &String) -> std::result::Result<Self, Self::Error> {
        // Number -> table index
        if let Ok(pos) = value.parse::<usize>() {
            Ok(Self::Index(pos))

        // START:END -> range
        } else if let Some((p1, p2)) = value.split_once(':') {
            let start = if p1.is_empty() {
                1
            } else {
                p1.parse::<usize>()
                    .map_err(|_| SelectSpecParseError::Start(p1.into()))?
            };
            if p2.is_empty() {
                Ok(Self::From(start))
            } else {
                let end = p2
                    .parse::<usize>()
                    .map_err(|_| SelectSpecParseError::End(p2.into()))?;
                if start > end {
                    return Err(SelectSpecParseError::Order(start, end));
                }
                Ok(Self::Range((start, end)))
            }

        // Everything else -> Id
        } else {
            Ok(Self::Id(value.into()))
        }
    }
}

pub fn parse_log_specs(args: &[String]) -> Result<Vec<SelectSpec>> {
    args.iter()
        .map(SelectSpec::try_from)
        .collect::<std::result::Result<Vec<SelectSpec>, SelectSpecParseError>>()
        .map_err(|e| match e {
            SelectSpecParseError::Start(s) => Error::custom(format!(
                "invalid range start '{s}': expected a positive number"
            )),
            SelectSpecParseError::End(e) => Error::custom(format!(
                "invalid range end '{e}': expected a positive number"
            )),
            SelectSpecParseError::Order(s, e) => {
                Error::custom(format!("invalid range '{s}:{e}': expected START:END"))
            }
        })
}

pub fn print_log_table(logs: &[LogInfo]) {
    let table = logs_table(&logs.iter().collect(), true);
    if table.count_rows() == 1 {
        println!("No logs found");
    } else {
        println!("{table}");
    }
}

/// Returns a tuple of table and read-error log IDs
pub fn logs_table(logs: &Vec<&LogInfo>, index: bool) -> Table {
    let now = chrono::Utc::now();
    let index_offset = if index { 0 } else { 1 };
    let mut colored_cells: Vec<(usize, usize, Color)> = Vec::new();
    let mut table = Builder::new();
    table.push_record([
        "#", "Id", "Task", "Type", "Status", "Model", "Dataset", "Started",
    ]);
    for (i, log) in logs.iter().enumerate() {
        match LogHeader::try_from(*log) {
            Ok(header) => {
                colored_cells.push((i + 1, 4 - index_offset, status_color(&header.status)));
                table.push_record([
                    (i + 1).to_string(),
                    log.short_log_id().into(),
                    log.task.clone(),
                    header.eval.run_type().unwrap_or_default().into(),
                    header.status.to_string(),
                    fit_path_name(&header.eval.model, 20),
                    fit_path_name(&header.eval.dataset.name.unwrap_or_default(), 20),
                    log.mtime.to_human_since(&now),
                ]);
            }
            Err(err) => {
                log::error!("Error reading {}: {}", log.expect_local_path(), err);
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
        table.with_col_color(1, Color::FG_BRIGHT_CYAN);
    } else {
        table.remove_col(0);
    }

    // Id is dim
    table.with_col_color(1 - index_offset, Colors::dim());

    // Task is yellow
    table.with_col_color(2 - index_offset, Color::FG_BRIGHT_YELLOW);

    // Type is dim
    table.with_col_color(3 - index_offset, Colors::dim());

    // Date is dim
    table.with_col_color(7 - index_offset, Colors::dim());

    // Apply colored cells (e.g. status)
    for (row, col, color) in colored_cells.into_iter() {
        table.with_cell_color(col, row, color);
    }

    table
}

fn status_color(status: &EvalStatus) -> Color {
    match status {
        EvalStatus::Success => Color::empty(),
        EvalStatus::Error => Color::FG_RED,
        _ => Colors::dim(),
    }
}

pub fn select_logs<'a>(
    logs: &'a [LogInfo],
    specs: &[SelectSpec],
) -> impl Iterator<Item = &'a LogInfo> {
    logs.iter().enumerate().filter_map(|(i, log)| {
        if specs.is_empty() {
            Some(log)
        } else {
            select_log(log, i + 1, specs).then_some(log)
        }
    })
}

fn select_log(log: &LogInfo, table_index: usize, specs: &[SelectSpec]) -> bool {
    specs.iter().any(|spec| spec.apply(log, table_index))
}

pub fn log_op_dialog<'a, FnOp, FnConfirm, R, FnFinish>(
    title: &str,
    logs: impl Iterator<Item = &'a LogInfo>,
    confirm_prompt: Option<FnConfirm>,
    op: FnOp,
    finish: FnFinish,
) -> Result<DialogResult>
where
    FnOp: Fn(&LogInfo) -> Result<R>,
    FnConfirm: Fn(&Vec<&LogInfo>) -> (String, bool),
    FnFinish: Fn(Vec<R>) -> DialogResult,
{
    cli::intro(style(title).bold())?;

    // Show table of selected logs
    let logs = logs.collect::<Vec<_>>();
    let table = logs_table(&logs, false);
    cli::log::remark(table.to_string())?;

    // Prompt user
    if let Some(confirm_fn) = confirm_prompt {
        let (prompt, initial_value) = confirm_fn(&logs);
        if !cli::confirm(wrap(&prompt, term_width() - 4))
            .initial_value(initial_value)
            .interact()?
        {
            return Err(Error::Canceled);
        }
    }

    // Apply the op
    let mut result = Vec::new();
    for log in logs {
        result.push(op(log)?);
    }

    Ok(finish(result))
}

pub fn delete_log(log: &LogInfo, permanent: bool) -> Result<()> {
    let path = PathBuf::from(log.expect_local_path());
    if permanent {
        fs::remove_file(path)?;
    } else {
        let deleted_path = path.with_added_extension("deleted");
        if deleted_path.exists() {
            return Err(Error::custom(format!(
                "CONFLICT: log {} already deleted (see {})",
                log.log_id,
                relpath(&deleted_path).to_string_lossy()
            )));
        }
        fs::rename(path, deleted_path)?;
    }
    Ok(())
}

pub fn restore_log(log: &LogInfo) -> Result<()> {
    let path = PathBuf::from(log.expect_local_path());
    if path.extension().unwrap_or_default().to_string_lossy() != "deleted" {
        return Err(Error::custom(format!(
            "unexpected file extension for delete log {}",
            relpath(&path).to_string_lossy()
        )));
    }
    let restore_path = path.parent().unwrap().join(path.file_stem().unwrap());
    if restore_path.exists() {
        return Err(Error::custom(format!(
            "CONFLICT: log {} exists and would be overwritten (see {})",
            log.log_id,
            relpath(&restore_path).to_string_lossy()
        )));
    }
    fs::rename(path, restore_path)?;
    Ok(())
}

pub fn purge_log(log: &LogInfo) -> Result<()> {
    let path = PathBuf::from(log.expect_local_path());
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_log_specs() {
        use crate::commands::log::common::{SelectSpec, parse_log_specs};
        assert!(parse_log_specs(&[]).unwrap().is_empty());

        let parsed = parse_log_specs(&[
            "1".into(),
            "3:12".into(),
            ":5".into(),
            "5:".into(),
            "abc".into(),
            "3:3".into(),
        ])
        .unwrap();
        assert_eq!(parsed.len(), 6);
        assert_eq!(parsed[0], SelectSpec::Index(1));
        assert_eq!(parsed[1], SelectSpec::Range((3, 12)));
        assert_eq!(parsed[2], SelectSpec::Range((1, 5)));
        assert_eq!(parsed[3], SelectSpec::From(5));
        assert_eq!(parsed[4], SelectSpec::Id("abc".into()));
        assert_eq!(parsed[5], SelectSpec::Range((3, 3)));

        let err = parse_log_specs(&["-1:2".into()]).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid range start '-1': expected a positive number"
        );

        let err = parse_log_specs(&["1:-2".into()]).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid range end '-2': expected a positive number"
        );

        let err = parse_log_specs(&["2:1".into()]).unwrap_err();
        assert_eq!(err.to_string(), "invalid range '2:1': expected START:END");
    }

    #[test]
    fn test_select_spec_apply() {
        use crate::commands::log::common::SelectSpec;
        use crate::inspect2::log::LogInfo;
        use crate::util::EpochMillis;

        let log = LogInfo {
            name: "".into(),
            mtime: EpochMillis::default(),
            task: "".into(),
            log_id: "abcd".into(),
            is_deleted: false,
        };

        assert_eq!(SelectSpec::Index(1).apply(&log, 1), true);
        assert_eq!(SelectSpec::Index(1).apply(&log, 2), false);
        assert_eq!(SelectSpec::Range((1, 2)).apply(&log, 1), true);
        assert_eq!(SelectSpec::Range((1, 2)).apply(&log, 2), true);
        assert_eq!(SelectSpec::Range((1, 2)).apply(&log, 3), false);
        assert_eq!(SelectSpec::From(2).apply(&log, 3), true);
        assert_eq!(SelectSpec::From(2).apply(&log, 2), true);
        assert_eq!(SelectSpec::From(2).apply(&log, 1), false);
        assert_eq!(SelectSpec::Id("a".into()).apply(&log, 1), true);
        assert_eq!(SelectSpec::Id("abc".into()).apply(&log, 1), true);
        assert_eq!(SelectSpec::Id("abcd".into()).apply(&log, 1), true);
        assert_eq!(SelectSpec::Id("abce".into()).apply(&log, 1), false);
        assert_eq!(SelectSpec::Id("abcde".into()).apply(&log, 1), false);
    }
}
