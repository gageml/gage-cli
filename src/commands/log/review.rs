use std::path::PathBuf;

use clap::Args as ArgsTrait;

use crate::{commands::log::common::cmd_resolve_log_dir, result::Result, review};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    dev: bool,
}

pub fn main(args: Args) -> Result<()> {
    review::run(&cmd_resolve_log_dir(args.log_dir.as_ref())?, args.dev)
}
