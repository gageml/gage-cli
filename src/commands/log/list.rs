use std::path::PathBuf;

use clap::Args as ArgsTrait;
use cliclack as cli;
use console::style;
use pyo3::Python;

use crate::{
    commands::log::common::{cmd_resolve_log_dir, print_log_table},
    error::Error,
    inspect::log::{LogFilter, list_logs_filter},
    result::Result,
    util::term_height,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Show more entries
    #[arg(short, long, action = clap::ArgAction::Count)]
    more: u8,

    /// Limit list to N matching entries
    #[arg(short, long, value_name = "N")]
    limit: Option<usize>,

    /// Show all matching entries
    #[arg(short, long)]
    all: bool,

    /// Display deleted logs
    #[arg(short, long)]
    deleted: bool,
}

impl From<&Args> for LogFilter {
    fn from(value: &Args) -> Self {
        if value.deleted {
            Self::Deleted
        } else {
            Self::None
        }
    }
}

pub fn main(args: Args) -> Result<()> {
    // Check incompatible options
    if args.more > 0 && args.limit.is_some() {
        return Err(Error::general("--more and --limit cannot both be used"));
    }
    if args.more > 0 && args.all {
        return Err(Error::general("--more and --all cannot both be used"));
    }
    if args.all && args.limit.is_some() {
        return Err(Error::general("--all and --limit cannot both be used"));
    }

    let log_dir = cmd_resolve_log_dir(args.log_dir.as_ref())?;

    Python::initialize();
    Python::attach(|py| {
        let pb = cli::spinner();
        pb.start("Reading logs");
        let logs = list_logs_filter(py, &log_dir, &args)?;
        pb.clear();

        // Calc number of entries to show based on options
        let count = std::cmp::min(
            if args.all {
                logs.len()
            } else {
                args.limit.unwrap_or_else(|| {
                    let page_size = term_height() - 7;
                    page_size * (args.more as usize + 1)
                })
            },
            logs.len(),
        );

        // Print table
        print_log_table(py, logs[..count].iter());

        // If table truncated show what happened
        if count < logs.len() {
            println!(
                "{}",
                style(format!("Showing {} of {} (-m for more)", count, logs.len()))
                    .dim()
                    .italic()
            );
        }
        Ok(())
    })
}
