use pyo3::FromPyObject;

/// Eval error details.
#[derive(FromPyObject, Debug)]
pub struct EvalError {
    // Error message.
    pub message: String,

    /// Error traceback.
    pub traceback: String,
    //
    // """Error traceback with ANSI color codes."""
    // traceback_ansi: str
}
