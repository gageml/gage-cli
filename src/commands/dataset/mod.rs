use clap::{Args as ArgsTrait, Subcommand};
use cliclack as cli;
use pyo3::Python;

use crate::{
    inspect::dataset::{DatasetInfo, list_datasets},
    result::Result,
    util::split_path_or_env,
};

mod list;

#[derive(ArgsTrait, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List datasets
    List(list::Args),
}

pub fn main(args: Args) -> Result<()> {
    match args.cmd {
        Cmd::List(args) => list::main(args),
    }
}

pub fn list_datasets_dialog<'py>(
    py: Python<'py>,
    path: Option<&str>,
) -> Result<Vec<DatasetInfo<'py>>> {
    let pb = cli::spinner();
    pb.start("Finding datasets");
    let path = split_path_or_env(path, "TASKPATH");
    let datasets = list_datasets(py, &path)?;
    pb.clear();
    Ok(datasets)
}
