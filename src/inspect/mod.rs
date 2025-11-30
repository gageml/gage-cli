use std::collections::HashMap;

use crate::py::Any;

pub mod dataset;
pub mod event;
pub mod json;
pub mod log;
pub mod model;
pub mod scorer;
pub mod task;
pub mod tool;
pub mod error;

pub type Metadata = HashMap<String, Any>;
pub type Attributes = HashMap<String, Any>;
pub type Args = HashMap<String, Any>;
