use std::fmt::{Debug, Display};

use pyo3::Python;

pub enum Error {
    Py(pyo3::PyErr),
    IO(std::io::Error),
    Custom(String),
    Quiet,
    Canceled,
}

impl std::error::Error for Error {}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Py(py_err) => {
                Python::attach(|py| py_err.display(py));
                Ok(())
            }
            Self::IO(io_error) => Display::fmt(io_error, f),
            Self::Custom(msg) => f.write_str(msg),
            Self::Quiet => Ok(()),
            Self::Canceled => Ok(()),
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::general(value)
    }
}

impl From<pyo3::PyErr> for Error {
    fn from(value: pyo3::PyErr) -> Self {
        Self::Py(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl Error {
    pub fn general(msg: impl Display) -> Self {
        Self::Custom(msg.to_string())
    }

    pub fn no_tasks() -> Self {
        // TODO - review error msg - suggest -p/--path or TASTPATH env?
        Self::Custom("Cannot find tasks to run".into())
    }

    pub fn no_matching_tasks(tasks_arg: &[String]) -> Self {
        Self::Custom(format!(
            "Cannot find tasks matching '{}'\n\
            \n\
            Try 'gage task list' for a list of available tasks.",
            tasks_arg.join(" ")
        ))
    }

    pub fn no_such_task(name: &str) -> Self {
        Self::Custom(format!(
            "Cannot find task '{name}'\n\
            \n\
            Try 'gage task list' for a list of available tasks.",
        ))
    }

    pub fn missing_required_model() -> Self {
        Self::Custom(
            "Missing required model\n\
            \n\
            Specify --model or set GAGE_MODEL or INSPECT_EVAL_MODEL."
                .into(),
        )
    }
}
