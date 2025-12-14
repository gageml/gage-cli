use std::env;

use clap::{Args as ArgsTrait, Subcommand};
use cliclack as cli;
use console::style;
use pyo3::Python;

use crate::{
    commands::endpoint,
    config::Config,
    error::Error,
    inspect::{self, task::TaskInfo},
    result::Result,
    util::split_path_or_env,
};

pub mod eval;
mod info;
mod list;
pub mod run;

#[derive(ArgsTrait, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Evaluate tasks
    Eval(eval::Args),

    /// Show task info
    Info(info::Args),

    /// Show available tasks
    List(list::Args),

    /// Run a task
    Run(run::Args),

    /// Start a task endpoint
    #[command(hide = true)] // TODO
    Serve(endpoint::start::Args),
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    match args.cmd {
        Cmd::Serve(args) => endpoint::start::main(args),
        Cmd::Eval(args) => eval::main(args, config),
        Cmd::Info(args) => info::main(args, config),
        Cmd::List(args) => list::main(args, config),
        Cmd::Run(args) => run::main(args, config),
    }
}

pub fn list_tasks_dialog<'py>(py: Python<'py>, path: Option<&str>) -> Result<Vec<TaskInfo>> {
    list_tasks_impl(py, path, true)
}

pub fn list_tasks<'py>(py: Python<'py>, path: Option<&str>) -> Result<Vec<TaskInfo>> {
    list_tasks_impl(py, path, false)
}

fn list_tasks_impl<'py>(
    py: Python<'py>,
    path: Option<&str>,
    dialog: bool,
) -> Result<Vec<TaskInfo>> {
    // If dialog, use a spinner to show indeterminate progress
    let pb = if dialog {
        let pb = cli::spinner();
        pb.start("Finding tasks");
        Some(pb)
    } else {
        None
    };
    let path = split_path_or_env(path, "TASKPATH");
    let tasks = inspect::task::list_tasks(py, &path)?;
    if let Some(pb) = pb {
        pb.clear();
    }
    Ok(tasks)
}

pub fn select_task<'py>(
    py: Python<'py>,
    task_arg: Option<&str>,
    path_arg: Option<&str>,
) -> Result<TaskInfo> {
    select_task_impl(py, task_arg, path_arg, false, false)
}

pub fn select_task_dialog<'py>(
    py: Python<'py>,
    task_arg: Option<&str>,
    path_arg: Option<&str>,
    yes_arg: bool,
) -> Result<TaskInfo> {
    select_task_impl(py, task_arg, path_arg, yes_arg, true)
}

fn select_task_impl<'py>(
    py: Python<'py>,
    task_arg: Option<&str>,
    path_arg: Option<&str>,
    yes_arg: bool,
    dialog: bool,
) -> Result<TaskInfo> {
    // Get available tasks
    let tasks = if dialog {
        list_tasks_dialog(py, path_arg)?
    } else {
        list_tasks(py, path_arg)?
    };
    if tasks.is_empty() {
        return Err(Error::no_tasks());
    }

    // If a task arg is specified, use it to find a task
    if let Some(name) = task_arg {
        for task in tasks {
            if task.name == name {
                if dialog {
                    cli::log::step(format!("Task:\n{}", style(&task.name).dim()))?;
                }
                return Ok(task);
            }
        }
        return Err(Error::no_such_task(name));
    }

    // If there's only one task, select it without prompting
    if tasks.len() == 1 {
        let task = tasks.into_iter().next().unwrap();
        if dialog {
            cli::log::step(format!("Task:\n{}", style(&task.name).dim()))?;
        }
        return Ok(task);
    }

    // Have multiple tasks - error if yes arg
    if yes_arg {
        return Err(Error::general(format!(
            "Found multiple tasks\n\nSpecify one of: {}",
            tasks
                .into_iter()
                .map(|t| t.name)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    // Prompt user to select a task
    let mut select = cli::select("Task:");
    for (i, task) in tasks.iter().enumerate() {
        select = select.item(i, task.name.clone(), task.file.clone());
    }
    let selected = select.interact()?;
    Ok(tasks.into_iter().nth(selected).unwrap())
}

fn select_tasks_dialog<'py>(
    py: Python<'py>,
    path_arg: Option<&str>,
    tasks_arg: &[String],
    yes_arg: bool,
) -> Result<Vec<TaskInfo>> {
    let tasks = list_tasks_dialog(py, path_arg)?;
    if tasks.is_empty() {
        return Err(Error::no_tasks());
    }

    // If one task matched or tasks arg (explicit list) or yes arg, use
    // matching tasks
    if tasks.len() == 1 || !tasks_arg.is_empty() || yes_arg {
        let matching = if tasks_arg.is_empty() {
            tasks
        } else {
            tasks
                .into_iter()
                .filter(|task| tasks_arg.contains(&task.name))
                .collect::<Vec<_>>()
        };
        if matching.is_empty() {
            return Err(Error::no_matching_tasks(tasks_arg));
        }
        // Show matching tasks as a step
        cli::log::step(format!(
            "Tasks:\n{}",
            list_value(matching.iter().map(|task| &task.name))
        ))?;
        return Ok(matching);
    }

    // Let user preview selected tasks and modify list if needed
    let mut select = cli::multiselect("Tasks:");
    let mut selected = Vec::<usize>::new();
    for (i, task) in tasks.iter().enumerate() {
        select = select.item(i, task.name.clone(), task.file.clone());
        if tasks_arg.is_empty() || tasks_arg.contains(&task.name) {
            selected.push(i);
        }
    }
    select = select.initial_values(selected);
    let selected = select.interact()?;
    if selected.is_empty() {
        return Err(Error::Canceled);
    }
    Ok(tasks
        .into_iter()
        .enumerate()
        .filter_map(|(i, task)| {
            if selected.contains(&i) {
                Some(task)
            } else {
                None
            }
        })
        .collect::<Vec<_>>())
}

fn list_value<T, I>(iter: T) -> String
where
    T: Iterator<Item = I>,
    I: ToString,
{
    iter.map(|s| style(s.to_string()).dim().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn select_model_dialog(model_arg: Option<&str>, yes_arg: bool) -> Result<Option<String>> {
    // If model arg specified, use it without prompting
    if let Some(model) = model_arg {
        cli::log::step(format!("Model:\n{}", style(model).dim()))?;
        return Ok(Some(model.to_string()));
    }

    // If env var specified, use it as default
    if let Ok(model) = env::var("GAGE_MODEL").or_else(|_| env::var("INSPECT_EVAL_MODEL")) {
        if yes_arg {
            cli::log::step(format!("Model:\n{}", style(&model).dim()))?;
            Ok(Some(model))
        } else {
            Ok(Some(
                cli::Input::new("Model:").default_input(&model).interact()?,
            ))
        }
    } else if yes_arg {
        Ok(None)
    } else {
        Ok(Some(cli::Input::new("Model:").interact()?))
    }
}
