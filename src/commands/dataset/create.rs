use clap::Args as ArgsTrait;
use pyo3::Python;

use crate::{commands::task::select_task_dialog, py, result::Result};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Task to create dataset for
    #[arg(short, long)]
    task: Option<String>,

    /// Path to find tasks
    #[arg(short, long)]
    path: Option<String>,

    /// Don't prompt when creating dataset
    #[arg(short, long)]
    yes: bool,
}

pub fn main(args: Args) -> Result<()> {
    py::init();
    Python::attach(|py| {
        let task = select_task_dialog(py, args.task.as_deref(), args.path.as_deref(), args.yes)?;
        println!("TODO something with {task:?}");
        Ok(())
    })
}
