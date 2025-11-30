#![allow(dead_code)]

use pyo3::{
    Bound, FromPyObject, Python,
    types::{PyAnyMethods, PyDict, PyDictMethods},
};

use crate::{
    inspect::{
        Metadata,
        log::{SampleId, SampleInput, Target},
    },
    py::py_call,
    result::Result,
};

#[derive(FromPyObject, Debug)]
pub struct DatasetInfo<'py> {
    pub file: String,
    pub name: String,
    attribs: Bound<'py, PyDict>,
}

impl<'py> DatasetInfo<'py> {
    pub fn get_description(&self) -> Option<String> {
        self.attribs
            .get_item("description")
            .unwrap()
            .map(|value| value.to_string())
    }
}

pub fn list_datasets<'py>(py: Python<'py>, path: &Vec<String>) -> Result<Vec<DatasetInfo<'py>>> {
    Ok(
        py_call(py, "gage_inspect.dataset", "list_datasets", (path,))?
            .extract()
            .unwrap(),
    )
}

/// Sample for an evaluation task.
#[derive(FromPyObject, Debug)]
pub struct Sample {
    /// The input to be submitted to the model.
    pub input: SampleInput,

    /// List of available answer choices (used only for multiple-choice evals).
    pub choices: Option<Vec<String>>,

    /// Ideal target output. May be a literal value or narrative text to be used by a model grader.
    pub target: Target,

    /// Unique identifier for sample.
    pub id: Option<SampleId>,

    /// Arbitrary metadata associated with the sample.
    pub metadata: Option<Metadata>,
    //
    // sandbox: SandboxEnvironmentSpec | None = Field(default=None)
    // """Sandbox environment type and optional config file."""

    // files: dict[str, str] | None = Field(default=None)
    // """Files that go along with the sample (copied to SandboxEnvironment)"""

    // setup: str | None = Field(default=None)
    // """Setup script to run for sample (run within default SandboxEnvironment)."""
}
