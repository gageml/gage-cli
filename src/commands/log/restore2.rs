use clap::Args as ArgsTrait;
use std::path::PathBuf;

use crate::{
    commands::log::common2::{log_op_dialog, parse_log_specs, restore_log, select_logs},
    config::Config,
    dialog::{DialogResult, handle_dialog_result},
    error::Error,
    inspect::log::resolve_log_dir,
    inspect2::log::{LogInfo, list_logs_filter},
    profile::apply_profile,
    result::Result,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Log(s) to restsore.
    ///
    /// SPEC may be a log ID, index position, or index position range in
    /// the form START:END.
    #[arg(value_name = "SPEC")]
    specs: Vec<String>,

    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Restore all deleted logs
    #[arg(short, long)]
    all: bool,

    /// Don't prompt for dialog
    #[arg(short, long)]
    yes: bool,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    if args.specs.is_empty() && !args.all {
        return Err(Error::custom(
            "--all flag must be provided if there is no target",
        ));
    }

    let specs = parse_log_specs(args.specs.as_ref())?;

    // Select logs
    apply_profile(config)?;
    let log_dir = resolve_log_dir(args.log_dir.as_ref());
    let logs = list_logs_filter(&log_dir, |log| log.is_deleted)?;
    let selected = select_logs(&logs, &specs);

    // Log op dialog
    let prompt = (!args.yes).then_some(|logs: &Vec<&LogInfo>| prompt(logs.len()));
    let op = |log: &LogInfo| restore_log(log);
    let finish = |restored: Vec<_>| finish(restored.len());
    handle_dialog_result(log_op_dialog("Restore logs", selected, prompt, op, finish))
}

fn prompt(count: usize) -> (String, bool) {
    (
        format!(
            "You are about to restore {} {}. Continue?",
            count,
            if count == 1 { "log" } else { "logs" },
        ),
        true,
    )
}

fn finish(count: usize) -> DialogResult {
    DialogResult::message(format!(
        "{} {} restored",
        count,
        if count == 1 { "log" } else { "logs" },
    ))
}
