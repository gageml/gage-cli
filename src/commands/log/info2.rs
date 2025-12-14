use std::path::PathBuf;

use clap::Args as ArgsTrait;
use itertools::Itertools;
use tabled::{builder::Builder, settings::Color};

use crate::{
    config::Config,
    error::Error,
    inspect::log::resolve_log_dir,
    inspect2::log::{LogHeader, list_logs_filter},
    profile::apply_profile,
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

pub fn main(args: Args, config: &Config) -> Result<()> {
    apply_profile(config)?;
    let log_dir = resolve_log_dir(args.log_dir.as_ref());

    let logs = list_logs_filter(&log_dir, |log| {
        !log.is_deleted && log.log_id.starts_with(&args.id)
    })?;

    if logs.is_empty() {
        return Err(Error::general(format!(
            "No logs matching '{}'\n\
                \n\
                Try 'gage logs list' for a list of logs.",
            args.id
        )));
    }

    if logs.len() > 1 {
        let ids = logs
            .into_iter()
            .map(|log| log.log_id)
            .sorted()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(Error::general(format!(
            "{}\n\
                \n\
                Use the full log ID instead.",
            wrap(
                &format!("More than one log matches '{}': {}", args.id, ids),
                term_width() - 4
            ),
        )));
    }

    let log = &logs[0];

    let mut table = Builder::new();

    // Log Id
    table.push_record(["Log", &log.log_id]);
    let log_id_row = table.count_records() - 1;

    // Task
    table.push_record(["Task", &log.task]);
    let task_row = table.count_records() - 1;
    table.push_record([
        "Created",
        &log.mtime.as_ref().map(|t| t.to_human()).unwrap_or_default(),
    ]);

    // Header attrs
    match LogHeader::try_from(log) {
        Ok(header) => {
            // Status
            table.push_record(["Status", &header.status.to_string()]);

            // Dataset
            table.push_record([
                "Dataset",
                header.eval.dataset.name.as_deref().unwrap_or_default(),
            ]);

            // Samples
            table.push_record(["Samples", &{
                let samples = &header.results.total_samples;
                let completed = &header.results.completed_samples;
                if samples == completed {
                    format!("{samples}")
                } else {
                    format!("{samples} ({completed} completed)")
                }
            }]);
        }
        Err(err) => table.push_record(["Error", &err.to_string()]),
    }

    println!(
        "{}",
        table
            .build()
            .with_term_fit()
            .with_row_labels()
            .with_col_labels()
            .with_rounded()
            .with_cell_color(1, log_id_row, Color::FG_BRIGHT_CYAN)
            .with_cell_color(1, task_row, Color::FG_BRIGHT_YELLOW)
    );
    Ok(())
}
