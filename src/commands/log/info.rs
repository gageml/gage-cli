use std::path::PathBuf;

use clap::Args as ArgsTrait;

use crate::{config::Config, inspect2::log::open_log, profile::apply_profile, result::Result};

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
    let log = open_log(&args.id, args.log_dir.as_ref());
    println!("TODO show log info for {log:?}");
    Ok(())
}
