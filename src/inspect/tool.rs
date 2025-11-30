#![allow(dead_code)]

use std::collections::HashMap;

use pyo3::FromPyObject;

use crate::{inspect::json::JSONSchema, py::Any};

/// Specification of a tool (JSON Schema compatible)
#[derive(FromPyObject, Debug)]
pub struct ToolInfo {
    /// Name of tool.
    pub name: String,

    /// Short description of tool.
    pub description: String,

    /// JSON Schema of tool parameters object.
    pub parameters: ToolParams,

    /// Optional property bag that can be used by the model provider to customize the implementation of the tool
    pub options: Option<HashMap<String, Any>>,
}

/// Description of tool parameters object in JSON Schema format.
#[derive(FromPyObject, Debug)]
pub struct ToolParams {
    /// Tool function parameters.
    pub properties: HashMap<String, ToolParam>,

    /// List of required fields.
    pub required: Vec<String>,

    /// Are additional object properties allowed? (always `False`)
    #[pyo3(attribute("additionalProperties"))]
    pub additional_properties: bool,
}

pub type ToolParam = JSONSchema;
