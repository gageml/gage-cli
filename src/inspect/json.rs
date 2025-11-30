#![allow(dead_code)]

use std::{collections::HashMap, fmt::Display};

use pyo3::{
    Borrowed, FromPyObject, PyAny, PyErr, PyResult, exceptions::PyTypeError, types::PyAnyMethods,
};

use crate::py::Any;

/// Describes a change to data using JSON Patch format.
#[derive(FromPyObject, Debug)]
pub struct JsonChange {
    // """Change operation."""
    pub op: JsonChangeOp,

    /// Path within object that was changed (uses / to delimit levels).
    pub path: String,

    /// Location from which data was moved or copied.
    #[pyo3(attribute("from_"))]
    pub from_location: Option<String>,

    // Changed value.
    pub value: JsonValue,

    /// Replaced value.
    pub replaced: JsonValue,
    //
    // model_config = {"populate_by_name": True}
}

#[derive(Debug)]
pub enum JsonChangeOp {
    Remove,
    Add,
    Replace,
    Move,
    Test,
    Copy,
}

impl<'a, 'py> FromPyObject<'a, 'py> for JsonChangeOp {
    type Error = PyErr;
    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        match obj.extract::<String>()?.as_str() {
            "remove" => Ok(Self::Remove),
            "add" => Ok(Self::Add),
            "replace" => Ok(Self::Replace),
            "move" => Ok(Self::Move),
            "test" => Ok(Self::Test),
            "copy" => Ok(Self::Copy),
            other => Err(PyErr::new::<PyTypeError, _>(format!(
                "unknown JSON change op '{other}'"
            ))),
        }
    }
}

impl Display for JsonChangeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Remove => "remove",
            Self::Add => "add",
            Self::Replace => "replace",
            Self::Move => "move",
            Self::Test => "test",
            Self::Copy => "copy",
        }
        .fmt(f)
    }
}

#[derive(Debug)]
pub enum JsonValue {
    None,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Map(HashMap<String, JsonValue>),
    List(Vec<JsonValue>),
}

impl JsonValue {
    pub fn some(&self) -> Option<&Self> {
        match self {
            Self::None => None,
            other => Some(other),
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for JsonValue {
    type Error = PyErr;
    /// Manual implementation to support Null
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        if ob.is_none() {
            Ok(Self::None)
        } else if let Ok(s) = ob.extract::<String>() {
            Ok(Self::String(s))
        } else if let Ok(i) = ob.extract::<i64>() {
            Ok(Self::Int(i))
        } else if let Ok(f) = ob.extract::<f64>() {
            Ok(Self::Float(f))
        } else if let Ok(b) = ob.extract::<bool>() {
            Ok(Self::Bool(b))
        } else if let Ok(m) = ob.extract::<HashMap<String, JsonValue>>() {
            Ok(Self::Map(m))
        } else if let Ok(l) = ob.extract::<Vec<JsonValue>>() {
            Ok(Self::List(l))
        } else {
            Err(PyErr::new::<PyTypeError, _>(format!(
                "invalid JSON value '{}'",
                ob.as_any()
            )))
        }
    }
}

impl Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(l) => write!(f, "{l:?}"),
            Self::Map(m) => write!(f, "{m:?}"),
            Self::String(s) => s.fmt(f),
            Self::Bool(b) => b.fmt(f),
            Self::Int(i) => i.fmt(f),
            Self::Float(n) => n.fmt(f),
            Self::None => "null".fmt(f),
        }
    }
}

/// JSON Schema for type.
#[derive(FromPyObject, Debug)]
pub struct JSONSchema {
    /// JSON type of tool parameter.
    #[pyo3(attribute("type"))]
    pub json_type: Option<String>,

    /// Format of the parameter (e.g. date-time).
    pub format: Option<String>,

    // Parameter description.
    pub description: Option<String>,

    /// Default value for parameter.
    pub default: Any,

    /// Valid values for enum parameters.
    #[pyo3(attribute("enum"))]
    pub enum_vals: Option<Vec<Any>>,
    //
    // """Valid type for array parameters."""
    // items: Option<JSONSchema> // TODO handle nesting

    // """Valid fields for object parametrs."""
    // properties: dict[str, "JSONSchema"] | None = Field(default=None)

    // additionalProperties: Optional["JSONSchema"] | bool | None = Field(default=None)
    // """Are additional properties allowed?"""

    // anyOf: list["JSONSchema"] | None = Field(default=None)
    // """Valid types for union parameters."""
    //
    /// Required fields for object parameters.
    pub required: Option<Vec<String>>,
}
