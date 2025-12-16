use clap::{Args as ArgsTrait, Subcommand};

use crate::{config::Config, result::Result};

mod common;
mod common2;
pub mod delete;
mod delete2;
mod info;
pub mod list;
mod purge;
mod purge2;
mod restore;
mod restore2;
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

    /// Delete logs
    #[command(hide = true)]
    Delete2(delete2::Args),

    /// Purge deleted logs
    Purge(purge::Args),

    /// Purge deleted logs
    #[command(hide = true)]
    Purge2(purge2::Args),

    /// Restore deleted logs
    Restore(restore::Args),

    /// Restore deleted logs
    #[command(hide = true)]
    Restore2(restore2::Args),
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    match args.cmd {
        Cmd::List(args) => list::main(args, config),
        Cmd::Review(args) => review::main(args, config),
        Cmd::Info(args) => info::main(args, config),
        Cmd::Delete(args) => delete::main(args, config),
        Cmd::Delete2(args) => delete2::main(args, config),
        Cmd::Purge(args) => purge::main(args, config),
        Cmd::Purge2(args) => purge2::main(args, config),
        Cmd::Restore(args) => restore::main(args, config),
        Cmd::Restore2(args) => restore2::main(args, config),
    }
}
