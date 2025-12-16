use clap::Args as ArgsTrait;
use std::path::PathBuf;

use crate::{
    commands::log::common2::{delete_log, log_op_dialog, parse_log_specs, select_logs},
    config::Config,
    dialog::{DialogResult, handle_dialog_result},
    error::Error,
    inspect::log::resolve_log_dir,
    inspect2::log::{LogInfo, list_logs},
    profile::apply_profile,
    result::Result,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Log(s) to delete
    ///
    /// SPEC may be a log ID, index position, or index position range in
    /// the form START:END.
    #[arg(value_name = "SPEC")]
    specs: Vec<String>,

    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Delete all logs
    #[arg(short, long)]
    all: bool,

    /// Don't prompt for dialog
    #[arg(short, long)]
    yes: bool,

    /// Permanently delete logs
    #[arg(short, long)]
    permanent: bool,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    if args.specs.is_empty() && !args.all {
        return Err(Error::custom("--all required if there is no target"));
    }
    let specs = parse_log_specs(args.specs.as_ref())?;

    // Select logs
    apply_profile(config)?;
    let log_dir = resolve_log_dir(args.log_dir.as_ref());
    let logs = list_logs(&log_dir)?;
    let selected = select_logs(&logs, &specs);

    // Log op dialog
    let prompt = (!args.yes).then_some(|logs: &Vec<&LogInfo>| prompt(logs.len(), args.permanent));
    let op = |log: &LogInfo| delete_log(log, args.permanent);
    let finish = |deleted: Vec<_>| finish(deleted.len(), args.permanent);
    handle_dialog_result(log_op_dialog("Delete logs", selected, prompt, op, finish))
}

fn prompt(count: usize, permanent: bool) -> (String, bool) {
    (
        format!(
            "You are about to{} delete {} {}.{} Continue?",
            if permanent { " PERMANENTLY" } else { "" },
            count,
            if count == 1 { "log" } else { "logs" },
            if permanent {
                " This cannot be undone."
            } else {
                ""
            },
        ),
        !permanent,
    )
}

fn finish(count: usize, permanent: bool) -> DialogResult {
    DialogResult::message(format!(
        "{} {} {}",
        count,
        if count == 1 { "log" } else { "logs" },
        if permanent {
            "permanently deleted"
        } else {
            "deleted"
        },
    ))
}
