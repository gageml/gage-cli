use clap::Args as ArgsTrait;
use std::path::PathBuf;

use crate::{
    commands::log::common::{LogOpDialog, LogOpSuccessMap, LogSelect, cmd_resolve_log_dir},
    dialog::handle_dialog_result,
    error::Error,
    plural,
    result::Result,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// The target logs to delete
    ///
    /// LOGS may be specified using their # or Id. To delete a range, use
    /// one of 'START:', ':END', or 'START:END'.
    #[arg(value_name = "LOG")]
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

pub fn main(args: Args) -> Result<()> {
    validate_args(&args)?;
    let log_specs = LogSelect::parse_specs(&args.specs)?;
    handle_dialog_result(
        LogOpDialog::new("Delete logs")
            .log_dir(cmd_resolve_log_dir(args.log_dir.as_ref())?)
            .log_specs(log_specs)
            .show_prompt(!args.yes)
            .confirm_prompt(move |selected| {
                if args.permanent {
                    format!(
                        "You are about to PERMANENTLY delete {} {}. \
                         This cannot be undone. Continue?",
                        selected.len(),
                        plural!("log", selected.len()),
                    )
                } else {
                    format!(
                        "You are about to delete {} {}. Continue?",
                        selected.len(),
                        plural!("log", selected.len())
                    )
                }
            })
            .run(|log| log.delete(args.permanent))
            .on_success(|count| format!("{count} {} deleted", plural!("log", count))),
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
