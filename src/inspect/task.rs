use pyo3::{FromPyObject, Python, types::PyAnyMethods};

use crate::{
    error::Error,
    inspect::{
        Attributes,
        log::{EvalLog, EvalSample},
    },
    py::{Any, Docstring, py_call},
    result::Result,
};

#[derive(FromPyObject, Debug)]
pub struct TaskInfo {
    pub file: String,
    pub name: String,
    pub attribs: Attributes,
}

impl TaskInfo {
    pub fn get_description(&self) -> Option<String> {
        self.attribs
            .get("description")
            .map(|value| value.to_string())
    }

    pub fn get_full_name(&self) -> String {
        format!("{}@{}", self.file, self.name)
    }
}

pub fn list_tasks<'py>(py: Python<'py>, path: &Vec<String>) -> Result<Vec<TaskInfo>> {
    Ok(py_call(py, "gage_inspect.task", "list_tasks", (path,))?
        .extract()
        .unwrap())
}

#[allow(clippy::too_many_arguments)]
pub fn eval_tasks<'py>(
    py: Python<'py>,
    tasks: Vec<String>,
    models: Vec<String>,
    task_args: Vec<String>,
    dataset: Option<String>,
    limit: Option<usize>,
    samples: Vec<String>,
    shuffle: bool,
    sandbox: Option<String>,
    epochs: Option<usize>,
    max_tasks: usize,
    log_dir: String,
) -> Result<()> {
    py_call(
        py,
        "gage_inspect.task",
        "eval_tasks",
        (
            tasks, models, task_args, dataset, limit, samples, shuffle, sandbox, epochs, max_tasks,
            log_dir,
        ),
    )?;
    Ok(())
}

mod gage_inspect {
    pyo3::import_exception!(gage_inspect.task, NoModel);
}

#[derive(FromPyObject, Debug)]
pub struct TaskResult {
    log: EvalLog,
}

impl TaskResult {
    pub fn log(&self) -> &EvalLog {
        &self.log
    }

    pub fn sample(&self) -> &EvalSample {
        self.log
            .samples
            .as_ref()
            .expect("task result has samples")
            .first()
            .expect("at least one sample")
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_task<'py>(
    py: Python<'py>,
    task: String,
    input: String,
    task_args: Vec<String>,
    model: Option<String>,
    target: Option<String>,
    log_dir: Option<String>,
    tags: Vec<String>,
) -> Result<TaskResult> {
    let result = py_call(
        py,
        "gage_inspect.task",
        "run_task",
        (task, input, task_args, model, target, log_dir, tags),
    );
    log::debug!("{result:?}");
    match result {
        Ok(result) => Ok(result.extract()?),
        Err(err) => {
            if err.is_instance_of::<gage_inspect::NoModel>(py) {
                Err(Error::missing_required_model())
            } else {
                Err(Error::Py(err))
            }
        }
    }
}

pub fn get_task_doc<'py>(py: Python<'py>, task: &TaskInfo) -> Result<Option<Docstring>> {
    if let Some(Any::Str(doc)) = task.attribs.get("__doc__") {
        let result = py_call(py, "gage_inspect.task", "parse_task_doc", (doc,))?;
        Ok(result.extract()?)
    } else {
        Ok(None)
    }
}
