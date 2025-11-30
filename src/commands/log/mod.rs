use clap::{Args as ArgsTrait, Subcommand};

use crate::result::Result;

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

pub fn main(args: Args) -> Result<()> {
    match args.cmd {
        Cmd::List(args) => list::main(args),
        Cmd::Review(args) => review::main(args),
        Cmd::Info(args) => info::main(args),
        Cmd::Delete(args) => delete::main(args),
        Cmd::Purge(args) => purge::main(args),
        Cmd::Restore(args) => restore::main(args),
    }
}
