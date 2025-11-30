use std::path::PathBuf;

use clap::Args as ArgsTrait;
use itertools::Itertools;
use pyo3::Python;
use tabled::{
    builder::Builder,
    settings::{
        Color,
        object::{Columns, Object, Rows},
        themes::Colorization,
    },
};

use crate::{
    commands::log::common::cmd_resolve_log_dir,
    error::Error,
    inspect::log::{EvalLogInfo, list_logs, read_log_header},
    result::Result,
    util::{TableExt, term_width, wrap},
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Eval ID
    #[arg()]
    id: String,

    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Show more detail
    #[arg(short, long)]
    verbose: bool,
}

pub fn main(args: Args) -> Result<()> {
    Python::initialize();
    Python::attach(|py| {
        let log_dir = cmd_resolve_log_dir(args.log_dir.as_ref())?;
        let logs = list_logs(py, &log_dir)?;
        let matches = logs
            .iter()
            .filter(|log| log.log_id.starts_with(&args.id))
            .collect_vec();
        if matches.is_empty() {
            return Err(Error::general(format!(
                "No logs matching '{}'\n\
                \n\
                Try 'gage logs list' for a list of logs.",
                args.id
            )));
        }
        if matches.len() > 1 {
            let matches_list = matches
                .iter()
                .map(|log| log.log_id.clone())
                .sorted()
                .collect::<Vec<_>>()
                .join(", ");
            return Err(Error::general(format!(
                "{}\n\
                \n\
                Use the full log ID instead.",
                wrap(
                    &format!("More than one log matches '{}': {}", args.id, matches_list),
                    term_width() - 4
                ),
            )));
        }
        let log = matches[0];
        let header = read_log_header(py, &log.name)?;

        let mut table = Builder::new();
        table.push_record(["Log", &log.log_id]);
        table.push_record(["Task", &header.eval.task]);
        table.push_record(["Created", &header.eval.created.to_human()]);
        table.push_record(["Status", &header.status.to_string()]);
        if let Some(error) = header.error.as_ref() {
            table.push_record(["Error", &error.message]);
        }
        table.push_record([
            "Dataset",
            header.eval.dataset.name.as_deref().unwrap_or_default(),
        ]);
        table.push_record([
            "Samples",
            &header
                .eval
                .dataset
                .evaluated_count()
                .map(|n| n.to_string())
                .unwrap_or_default(),
        ]);
        table.push_record(["Model", &header.eval.model]);
        if args.verbose {
            table.push_record(["File", &fmt_log_filename(log)]);
            table.push_record(["Eval Id", &header.eval.eval_id]);
            table.push_record(["Run Id", &header.eval.run_id]);
        }
        println!(
            "{}",
            table
                .build()
                .with_term_fit()
                .with_row_labels()
                .with_col_labels()
                .with_rounded()
                // Log Id - used as identifier so highlight with cyan
                .with(Colorization::exact(
                    [Color::FG_BRIGHT_CYAN],
                    Columns::one(1).intersect(Rows::one(0))
                ))
                // Task - highlight with yello
                .with(Colorization::exact(
                    [Color::FG_BRIGHT_YELLOW],
                    Columns::one(1).intersect(Rows::one(1))
                ))
        );
        Ok(())
    })
}

pub fn fmt_log_filename(log: &EvalLogInfo) -> String {
    let file_name = PathBuf::from(log.name.strip_prefix("file://").expect("local files only"));
    let cwd = std::env::current_dir().unwrap();
    file_name
        .strip_prefix(cwd)
        .unwrap_or(&file_name)
        .to_string_lossy()
        .into()
}
