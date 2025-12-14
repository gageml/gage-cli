use clap::{Args as ArgsTrait, Subcommand};

use crate::{config::Config, result::Result};

mod common;
pub mod delete;
mod info;
pub mod list;
mod purge;
mod restore;
pub mod review;

#[derive(ArgsTrait, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Show avaliable logs
    List(list::Args),

    /// Review logs
    Review(review::Args),

    /// Show log info
    Info(info::Args),

    /// Delete logs
    Delete(delete::Args),

    /// Purge deleted logs
    Purge(purge::Args),

    /// Restore deleted logs
    Restore(restore::Args),
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    match args.cmd {
        Cmd::List(args) => list::main(args, config),
        Cmd::Review(args) => review::main(args, config),
        Cmd::Info(args) => info::main(args, config),
        Cmd::Delete(args) => delete::main(args, config),
        Cmd::Purge(args) => purge::main(args, config),
        Cmd::Restore(args) => restore::main(args, config),
    }
}
