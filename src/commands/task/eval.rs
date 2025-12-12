use std::path::PathBuf;

use clap::Args as ArgsTrait;
use cliclack as cli;
use console::style;
use itertools::Itertools;
use pyo3::Python;

use crate::{
    commands::task::{list_value, select_model_dialog, select_tasks_dialog},
    dialog::{DialogResult, handle_dialog_result},
    error::Error,
    inspect::{log::resolve_log_dir, task::eval_tasks},
    py,
    result::Result,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Tasks to evaluate
    #[arg(name = "TASK")]
    tasks: Vec<String>,

    /// Task model (may be used more than once)
    #[arg(short, long = "model", value_name = "MODEL")]
    models: Vec<String>,

    /// Task argument NAME=VALUE (may use more than once)
    #[arg(short = 'T', value_name = "TASK_ARG")]
    task_args: Vec<String>,

    /// Evaluation dataset
    #[arg(short, long, value_name = "NAME")]
    dataset: Option<String>,

    /// Limit the number of samples to evaluate
    #[arg(short, long, value_name = "N")]
    limit: Option<usize>,

    /// Task model (may be used more than once)
    #[arg(short, long = "sample", value_name = "ID")]
    samples: Vec<String>,

    /// Sandbox environment type
    #[arg(long, value_name = "TYPE")]
    sandbox: Option<String>,

    /// Suffle sample order
    #[arg(long)]
    shuffle: bool,

    /// Number of times to evaluate dataset
    #[arg(short, long, value_name = "N")]
    epochs: Option<usize>,

    /// Path to find tasks
    #[arg(short, long)]
    path: Option<String>,

    /// Don't prompt to for input
    #[arg(short, long)]
    yes: bool,

    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,
}

pub fn main(args: Args) -> Result<()> {
    if args.limit.is_some() && !args.samples.is_empty() {
        return Err(Error::general("--limit cannot be used with samples"));
    }

    handle_dialog_result(eval_dialog(args))
}

fn eval_dialog(args: Args) -> Result<DialogResult> {
    cli::intro(style("Evaluate tasks").bold())?;

    py::init();
    Python::attach(|py| {
        let tasks = select_tasks_dialog(py, args.path.as_deref(), &args.tasks, args.yes)?;

        // Models
        let models = if args.models.is_empty() {
            if let Some(model) = select_model_dialog(None, args.yes)? {
                vec![model]
            } else {
                return Err(Error::missing_required_model());
            }
        } else {
            cli::log::step(format!(
                "{}:\n{}",
                if args.models.len() == 1 {
                    "Model"
                } else {
                    "Models"
                },
                list_value(args.models.iter())
            ))?;
            args.models
        };

        // Additional options
        let mut options = Vec::new();
        if let Some(dataset) = args.dataset.as_ref() {
            options.push(format!("Dataset: {dataset}"));
        }
        if let Some(val) = args.limit.as_ref() {
            options.push(format!(
                "Sample limit: {}{}",
                val,
                if args.shuffle && *val > 1 {
                    " (shuffled)"
                } else {
                    ""
                }
            ));
        } else if !args.samples.is_empty() {
            // Samples are exclusive of limit
            options.push(format!(
                "Samples: {}{}",
                args.samples.iter().join(", "),
                if args.shuffle && args.samples.len() > 1 {
                    " (shuffled)"
                } else {
                    ""
                }
            ));
        } else if args.shuffle {
            options.push("Shuffle samples: yes".into())
        }
        if let Some(val) = args.epochs.as_ref()
            && *val > 1
        {
            options.push(format!("Epochs: {val}"));
        }
        if !args.task_args.is_empty() {
            options.push(format!("Task args: {}", args.task_args.iter().join(", ")));
        }
        if !options.is_empty() {
            cli::log::step(format!(
                "Additional options:\n{}",
                options.iter().map(|s| style(s).dim()).join("\n")
            ))?;
        }

        // Confirm before running unless --yes
        if !args.yes
            && !cli::confirm(if tasks.len() == 1 {
                format!(
                    "You are about to evaluate {}. Continue?",
                    style(&tasks[0].name).cyan().bright()
                )
            } else {
                format!("You are about to evaluate {} tasks. Continue?", tasks.len())
            })
            .initial_value(true)
            .interact()?
        {
            return Err(Error::Canceled);
        }

        // Format tasks arg for Inspect
        let tasks = tasks
            .into_iter()
            .map(|task| task.get_full_name())
            .collect::<Vec<_>>();

        // Log dir
        let log_dir = match resolve_log_dir(args.log_dir.as_ref()) {
            Ok(path) => path,
            Err(_) => "logs".into(),
        };

        // Run eval
        eval_tasks(
            py,
            tasks,
            models,
            args.task_args,
            args.dataset,
            args.limit,
            args.samples,
            args.shuffle,
            args.sandbox,
            args.epochs,
            4,
            log_dir,
        )?;

        Ok(DialogResult::Done)
    })
}
