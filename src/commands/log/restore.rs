use clap::Args as ArgsTrait;
use std::path::PathBuf;

use crate::{
    commands::log::common::{LogOpDialog, LogOpSuccessMap, LogSelect, cmd_resolve_log_dir},
    dialog::handle_dialog_result,
    error::Error,
    inspect::log::LogFilter,
    plural,
    result::Result,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// The target log(s) to restore
    #[arg(value_name = "LOG")]
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

pub fn main(args: Args) -> Result<()> {
    validate_args(&args)?;
    let log_specs = LogSelect::parse_specs(&args.specs)?;
    handle_dialog_result(
        LogOpDialog::new("Restore logs")
            .log_dir(cmd_resolve_log_dir(args.log_dir.as_ref())?)
            .filter(LogFilter::Deleted)
            .log_specs(log_specs)
            .show_prompt(!args.yes)
            .confirm_prompt(move |selected| {
                format!(
                    "You are about to restore {} {}. Continue?",
                    selected.len(),
                    plural!("log", selected.len())
                )
            })
            .run(|log| log.restore())
            .on_success(|count| format!("{count} {} restored", plural!("log", count))),
    )
}

fn validate_args(args: &Args) -> Result<()> {
    if args.specs.is_empty() && !args.all {
        return Err(Error::general(
            "--all flag must be provided if there is no target",
        ));
    }
    Ok(())
}
