use std::{io, path::PathBuf};

use clap::{Parser, Subcommand, command};

mod commands;
mod config;
mod cursive;
mod dialog;
mod env;
mod error;
mod inspect;
mod inspect2;
mod logger;
mod profile;
mod py;
mod result;
mod review;
mod secrets;
mod theme;
mod util;

use commands as cmd;

use crate::{
    config::Config,
    error::Error,
    result::{Exit, Result},
};

#[derive(Parser)]
#[command(name = "gage", version, about)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,

    /// Gage config file (defaults to gage.toml)
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Manage datasets
    Dataset(cmd::dataset::Args),

    /// Delete one or more logs
    Delete(cmd::log::delete::Args),

    /// Run an evaluation
    Eval(cmd::task::eval::Args),

    /// List eval logs
    List(cmd::log::list::Args),

    /// Manage logs
    Log(cmd::log::Args),

    /// Manage profiles
    Profile(cmd::profile::Args),

    /// Review logs
    Review(cmd::log::review::Args),

    /// Run a task
    Run(cmd::task::run::Args),

    /// Show project status
    Status(cmd::status::Args),

    /// Manage tasks
    Task(cmd::task::Args),
}

fn main() -> Exit {
    let args = Args::parse();

    logger::init(args.debug);
    theme::init();
    env::init();

    let config = match init_config(&args) {
        Ok(config) => config,
        Err(e) => {
            return Exit::Err(e);
        }
    };

    // Dispatch command
    handle_result(match args.cmd {
        Cmd::Dataset(args) => cmd::dataset::main(args),
        Cmd::Eval(args) => cmd::task::eval::main(args, &config),
        Cmd::List(args) => cmd::log::list::main(args, &config),
        Cmd::Log(args) => cmd::log::main(args, &config),
        Cmd::Profile(args) => cmd::profile::main(args, &config),
        Cmd::Review(args) => cmd::log::review::main(args, &config),
        Cmd::Delete(args) => cmd::log::delete::main(args, &config),
        Cmd::Status(args) => cmd::status::main(args, &config),
        Cmd::Run(args) => cmd::task::run::main(args, &config),
        Cmd::Task(args) => cmd::task::main(args, &config),
    })
}

fn init_config(args: &Args) -> Result<Config> {
    let config = Config::try_from_arg(args.config.as_ref())
        .map_err(not_found_msg)?
        .unwrap_or_default();
    Ok(config)
}

fn not_found_msg(e: Error) -> Error {
    match e {
        Error::IO(e) if e.kind() == io::ErrorKind::NotFound => Error::general(format!(
            "Config file '{e}' not found\n\nCreate this file to define profiles."
        )),
        _ => e,
    }
}

fn handle_result(result: Result<()>) -> Exit {
    match result {
        Ok(()) => Exit::Ok,
        Err(e) => Exit::Err(e),
    }
}
