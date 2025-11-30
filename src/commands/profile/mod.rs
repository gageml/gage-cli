use clap::{Args as ArgsTrait, Subcommand};

use crate::{config::Config, result::Result};

mod list;
mod status;
mod use_;

#[derive(ArgsTrait, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Use a profile
    Use(use_::Args),

    /// Show avaliable profiles
    List,

    /// Show current profile status
    Status(status::Args),
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    match args.cmd {
        Cmd::Use(args) => use_::main(args, config),
        Cmd::List => list::main(config),
        Cmd::Status(args) => status::main(args, config),
    }
}
