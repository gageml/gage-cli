use clap::Args as ArgsTrait;
use std::path::PathBuf;

use crate::{
    commands::log::common::{LogOpDialog, LogOpSuccessMap, LogSelect},
    config::Config,
    dialog::handle_dialog_result,
    error::Error,
    inspect::log::{LogFilter, resolve_log_dir},
    plural,
    profile::apply_profile,
    result::Result,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// The target logs to permanently delete
    #[arg(value_name = "LOG")]
    specs: Vec<String>,

    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Permanently clean all deleted logs
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
    let log_specs = LogSelect::parse_specs(&args.specs)?;
    apply_profile(config)?;

    handle_dialog_result(
        LogOpDialog::new("Purge deleted logs")
            .log_dir(resolve_log_dir(args.log_dir.as_ref()))
            .filter(LogFilter::Deleted)
            .log_specs(log_specs)
            .show_prompt(!args.yes)
            .confirm_prompt(move |selected| {
                format!(
                    "You are about to PERMANENTLY delete {} {}. \
                        This cannot be undone. Continue?",
                    selected.len(),
                    plural!("log", selected.len())
                )
            })
            .run(|log| log.delete(true))
            .on_success(|count| format!("{count} {} permanently deleted", plural!("log", count))),
    )
}
