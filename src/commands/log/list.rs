use std::path::PathBuf;

use clap::Args as ArgsTrait;
use console::style;

use crate::{
    commands::log::common2::print_log_table, config::Config, error::Error,
    inspect::log::resolve_log_dir, inspect2::log::list_logs_filter, profile::apply_profile,
    result::Result, util::term_height,
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

pub fn main(args: Args, config: &Config) -> Result<()> {
    // Check incompatible options
    if args.more > 0 && args.limit.is_some() {
        return Err(Error::custom("--more and --limit cannot both be used"));
    }
    if args.more > 0 && args.all {
        return Err(Error::custom("--more and --all cannot both be used"));
    }
    if args.all && args.limit.is_some() {
        return Err(Error::custom("--all and --limit cannot both be used"));
    }

    apply_profile(config)?;
    let log_dir = resolve_log_dir(args.log_dir.as_ref());
    let logs = list_logs_filter(&log_dir, |l| l.is_deleted == args.deleted)?;

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
    print_log_table(logs[..count].iter());

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
}
