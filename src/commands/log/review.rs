use std::path::PathBuf;

use clap::Args as ArgsTrait;

use crate::{
    config::Config, inspect::log::resolve_log_dir, profile::apply_profile, result::Result, review,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    dev: bool,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    apply_profile(config)?;
    review::run(&resolve_log_dir(args.log_dir.as_ref()), args.dev)
}
