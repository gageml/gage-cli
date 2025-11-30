pub mod start;
mod status;
mod stop;

use clap::{Args as ArgsTrait, Subcommand};

use crate::result::Result;

#[derive(ArgsTrait, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Start a task endpoint
    Start(start::Args),

    /// Show endpoint status
    Status(status::Args),

    /// Stop an endpoint
    Stop(stop::Args),
}

pub fn main(args: Args) -> Result<()> {
    match args.cmd {
        Cmd::Start(args) => start::main(args),
        Cmd::Status(args) => status::main(args),
        Cmd::Stop(args) => stop::main(args),
    }
}
