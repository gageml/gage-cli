#![allow(dead_code)]

use std::{collections::HashMap, fmt::Display};

use pyo3::{FromPyObject, PyAny, PyErr, exceptions::PyTypeError, types::PyAnyMethods};

use crate::{
    inspect::{
        Metadata,
        dataset::Sample,
        error::EvalError,
        json::{JsonChange, JsonValue},
        log::Target,
        model::{ChatMessage, ContentAudio, ContentImage, ContentText, ContentVideo, ModelOutput},
        scorer::Score,
        tool::ToolInfo,
    },
    util::EpochMillis,
};

pub enum Node<'a> {
    Span(Span<'a>),
    Event(&'a Event),
}

pub struct Span<'a> {
    pub name: &'a str,
    pub events: NodeIter<'a>,
}

pub trait ToNodeIter<'a> {
    fn iter_nodes(&'a self) -> impl Iterator<Item = Node<'a>>;
}

impl<'a> ToNodeIter<'a> for Vec<Event> {
    fn iter_nodes(&'a self) -> impl Iterator<Item = Node<'a>> {
        NodeIter::Iter(self.iter())
    }
}

pub enum NodeIter<'a> {
    Iter(std::slice::Iter<'a, Event>),
    IntoIter(std::vec::IntoIter<&'a Event>),
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Helper to advance inner iter
        let mut inner_next = || match self {
            Self::Iter(inner) => inner.next(),
            Self::IntoIter(inner) => inner.next(),
        };

        let next = inner_next()?;
        if let Event::SpanBegin(span_begin) = next {
            // Collect events up to corresponding span end (consumed)
            let mut span_events = Vec::new();
            while let Some(event) = inner_next() {
                if let Event::SpanEnd(span_end) = event
                    && let Some(span_end_id) = span_end.base.span_id.as_ref()
                    && span_end_id == &span_begin.id
                {
                    break;
                }
                span_events.push(event);
            }
            Some(Node::Span(Span {
                name: &span_begin.name,
                events: NodeIter::IntoIter(span_events.into_iter()),
            }))
        } else {
            Some(Node::Event(next))
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Event {
    SampleInit(SampleInitEvent),
    SampleLimit(SampleLimitEvent),
    Sandbox(SandboxEvent),
    State(StateEvent),
    Store(StoreEvent),
    Model(ModelEvent),
    Tool(ToolEvent),
    Approval(ApprovalEvent),
    Input(InputEvent),
    Score(ScoreEvent),
    Error(ErrorEvent),
    Logger(LoggerEvent),
    Info(InfoEvent),
    SpanBegin(SpanBeginEvent),
    SpanEnd(SpanEndEvent),
    Step(StepEvent),
    Subtask(SubtaskEvent),
}

impl<'a, 'py> FromPyObject<'a, 'py> for Event {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        match ob.getattr("event")?.extract::<String>()?.as_str() {
            "sample_init" => Ok(Self::SampleInit(ob.extract()?)),
            "sample_limit" => Ok(Self::SampleLimit(ob.extract()?)),
            "sandbox" => Ok(Self::Sandbox(ob.extract()?)),
            "state" => Ok(Self::State(ob.extract()?)),
            "store" => Ok(Self::Store(ob.extract()?)),
            "model" => Ok(Self::Model(ob.extract()?)),
            "tool" => Ok(Self::Tool(ob.extract()?)),
            "approval" => Ok(Self::Approval(ob.extract()?)),
            "input" => Ok(Self::Input(ob.extract()?)),
            "score" => Ok(Self::Score(ob.extract()?)),
            "error" => Ok(Self::Error(ob.extract()?)),
            "logger" => Ok(Self::Logger(ob.extract()?)),
            "info" => Ok(Self::Info(ob.extract()?)),
            "span_begin" => Ok(Self::SpanBegin(ob.extract()?)),
            "span_end" => Ok(Self::SpanEnd(ob.extract()?)),
            "step" => Ok(Self::Step(ob.extract()?)),
            "subtask" => Ok(Self::Subtask(ob.extract()?)),
            // other => Err(PyErr::new::<PyTypeError, _>(format!(
            //     "unknown event type '{other}'"
            // ))),
            other => panic!("{other}"), // TEMP until we resolve error propogation
        }
    }
}

impl Event {
    pub fn base(&self) -> &BaseEvent {
        match self {
            Self::SampleInit(e) => &e.base,
            Self::SampleLimit(e) => &e.base,
            Self::Sandbox(e) => &e.base,
            Self::State(e) => &e.base,
            Self::Store(e) => &e.base,
            Self::Model(e) => &e.base,
            Self::Tool(e) => &e.base,
            Self::Approval(e) => &e.base,
            Self::Input(e) => &e.base,
            Self::Score(e) => &e.base,
            Self::Error(e) => &e.base,
            Self::Logger(e) => &e.base,
            Self::Info(e) => &e.base,
            Self::SpanBegin(e) => &e.base,
            Self::SpanEnd(e) => &e.base,
            Self::Step(e) => &e.base,
            Self::Subtask(e) => &e.base,
        }
    }
}

#[derive(FromPyObject, Debug)]
pub struct BaseEvent {
    #[pyo3(attribute("event"))]
    pub event_name: String,

    /// Unique identifer for event.
    pub uuid: Option<String>,

    /// Span the event occurred within.
    pub span_id: Option<String>,

    /// Clock time at which event occurred.
    pub timestamp: EpochMillis,

    /// Working time (within sample) at which the event occurred.
    pub working_start: f64,

    /// Additional event metadata.
    pub metadata: Option<Metadata>,

    /// Is this event pending?
    pub pending: Option<bool>,
}

#[derive(Debug)]
pub struct SampleInitEvent {
    pub base: BaseEvent,

    /// Sample.
    pub sample: Sample,

    /// Initial state.
    pub state: JsonValue,
}

impl<'a, 'py> FromPyObject<'a, 'py> for SampleInitEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            sample: ob.getattr("sample").unwrap().extract().unwrap(),
            state: ob.getattr("state").unwrap().extract().unwrap(),
        })
    }
}

#[derive(Debug)]
pub struct SampleLimitEvent {
    pub base: BaseEvent,
}

impl<'a, 'py> FromPyObject<'a, 'py> for SampleLimitEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
        })
    }
}

/// Sandbox execution or I/O
#[derive(Debug)]
pub struct SandboxEvent {
    pub base: BaseEvent,

    /// Sandbox action
    pub action: SandboxAction,

    /// Command (for exec)
    pub cmd: Option<String>,

    /// Options (for exec)
    pub options: Option<HashMap<String, JsonValue>>,

    /// File (for read_file and write_file)
    pub file: Option<String>,

    // Input (for cmd and write_file). Truncated to 100 lines.
    pub input: Option<String>,

    /// Result (for exec)
    pub result: Option<u16>,

    /// Output (for exec and read_file). Truncated to 100 lines.
    pub output: Option<String>,
    // """Time that sandbox action completed (see `timestamp` for started)"""
    // completed: datetime | None = Field(default=None)
}

#[derive(Debug)]
pub enum SandboxAction {
    Exec,
    ReadFile,
    WriteFile,
}

impl Display for SandboxAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exec => "exec",
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
        }
        .fmt(f)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SandboxEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            action: match ob.getattr("action")?.extract::<String>()?.as_str() {
                "exec" => Ok(SandboxAction::Exec),
                "read_file" => Ok(SandboxAction::ReadFile),
                "write_file" => Ok(SandboxAction::WriteFile),
                other => Err(PyErr::new::<PyTypeError, _>(format!(
                    "unknown sandbox action type '{other}'"
                ))),
            }?,
            cmd: ob.getattr("cmd")?.extract()?,
            options: ob.getattr("options")?.extract()?,
            file: ob.getattr("file")?.extract()?,
            input: ob.getattr("input")?.extract()?,
            result: ob.getattr("result")?.extract()?,
            output: ob.getattr("output")?.extract()?,
        })
    }
}

/// Change to the current `TaskState`
#[derive(Debug)]
pub struct StateEvent {
    pub base: BaseEvent,

    /// List of changes to the `TaskState`
    pub changes: Vec<JsonChange>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for StateEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            changes: ob.getattr("changes")?.extract()?,
        })
    }
}

/// Change to data within the current `Store`.
#[derive(Debug)]
pub struct StoreEvent {
    pub base: BaseEvent,

    /// List of changes to the `Store`.
    pub changes: Vec<JsonChange>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for StoreEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            changes: ob.getattr("changes")?.extract()?,
        })
    }
}

/// Call to a language model.
#[derive(Debug)]
pub struct ModelEvent {
    pub base: BaseEvent,

    /// Model name.
    pub model: String,

    /// Model role.
    pub role: Option<String>,

    /// Model input (list of messages).
    pub input: Vec<ChatMessage>,

    /// Tools available to the model.
    pub tools: Vec<ToolInfo>,

    /// Directive to the model which tools to prefer.
    pub tool_choice: ToolChoice,

    // config: GenerateConfig
    /// Generate config used for call to model.

    /// Output from model.
    pub output: ModelOutput,

    /// Retries for the model API request.
    pub retries: Option<usize>,

    /// Error which occurred during model call.
    pub error: Option<String>,

    /// Was this a cache read or write.
    pub cache: Option<CacheType>,

    /// Raw call made to model API.
    pub call: Option<ModelCall>,

    /// Time that model call completed (see `timestamp` for started)
    pub completed: Option<EpochMillis>,

    /// working time for model call that succeeded (i.e. was not retried)."""
    pub working_time: Option<f64>,
}

#[derive(Debug)]
pub enum ToolChoice {
    Auto,
    Any,
    None,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ToolChoice {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        match ob.extract::<String>()?.as_str() {
            "auto" => Ok(Self::Auto),
            "any" => Ok(Self::Any),
            "none" => Ok(Self::None),
            // other => Err(PyErr::new::<PyTypeError, _>(format!(
            //     "unknown tool choice '{other}'"
            // ))),
            other => panic!("{other}"), // TEMP
        }
    }
}

impl Display for ToolChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => "auto".fmt(f),
            Self::Any => "any".fmt(f),
            Self::None => "none".fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum CacheType {
    Read,
    Write,
}

impl<'a, 'py> FromPyObject<'a, 'py> for CacheType {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        match ob.extract::<String>()?.as_str() {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            // other => Err(PyErr::new::<PyTypeError, _>(format!(
            //     "unknown cache type '{other}'"
            // ))),
            other => panic!("{other}"), // TEMP
        }
    }
}

impl Display for CacheType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => "read".fmt(f),
            Self::Write => "write".fmt(f),
        }
    }
}

/// Model call (raw request/response data).
#[derive(FromPyObject, Debug)]
pub struct ModelCall {
    /// Raw data posted to model.
    pub request: HashMap<String, JsonValue>,

    /// Raw response data from model.
    pub response: HashMap<String, pyo3::Py<PyAny>>,

    /// Time taken for underlying model call.
    pub time: Option<f64>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ModelEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        // let call = ob.getattr("call").unwrap();
        // panic!("{}", call.as_any());
        Ok(Self {
            base: ob.extract()?,
            model: ob.getattr("model")?.extract()?,
            role: ob.getattr("role")?.extract()?,
            input: ob.getattr("input")?.extract()?,
            tools: ob.getattr("tools")?.extract()?,
            tool_choice: ob.getattr("tool_choice")?.extract()?,
            output: ob.getattr("output")?.extract()?,
            retries: ob.getattr("retries")?.extract()?,
            error: ob.getattr("error")?.extract()?,
            cache: ob.getattr("cache")?.extract()?,
            call: ob.getattr("call")?.extract()?,
            completed: ob.getattr("completed")?.extract()?,
            working_time: ob.getattr("working_time")?.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct ToolEvent {
    pub base: BaseEvent,

    /// Type of tool call (currently only 'function')
    pub call_type: String,

    /// Unique identifier for tool call.
    pub id: String,

    /// Function called.
    pub function: String,

    /// Arguments to function.
    pub arguments: HashMap<String, JsonValue>,

    /// Custom view of tool call input.
    pub view: Option<ToolCallContent>,

    /// Function return value.
    pub result: ToolResult,

    /// Bytes truncated (from,to) if truncation occurred
    pub truncated: Option<(i64, i64)>,

    /// Error that occurred during tool call.
    pub error: Option<ToolCallError>,

    /// Time that tool call completed (see `timestamp` for started)
    pub completed: Option<EpochMillis>,

    /// Working time for tool call (i.e. time not spent waiting on semaphores).
    pub working_time: Option<f64>,

    /// Name of agent if the tool call was an agent handoff.
    pub agent: Option<String>,

    /// Did the tool call fail with a hard error?.
    pub failed: Option<bool>,

    /// Id of ChatMessageTool associated with this event.
    pub message_id: Option<String>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ToolEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            call_type: ob.getattr("type")?.extract()?,
            id: ob.getattr("id")?.extract()?,
            function: ob.getattr("function")?.extract()?,
            arguments: ob.getattr("arguments")?.extract()?,
            view: ob.getattr("view")?.extract()?,
            result: ob.getattr("result")?.extract()?,
            truncated: ob.getattr("truncated")?.extract()?,
            error: ob.getattr("error")?.extract()?,
            completed: ob.getattr("completed")?.extract()?,
            working_time: ob.getattr("working_time")?.extract()?,
            agent: ob.getattr("agent")?.extract()?,
            failed: ob.getattr("failed")?.extract()?,
            message_id: ob.getattr("message_id")?.extract()?,
        })
    }
}

/// Content to include in tool call view.
#[derive(FromPyObject, Debug)]
pub struct ToolCallContent {
    /// Optional (plain text) title for tool call content.
    pub title: Option<String>,

    /// Format (text or markdown).
    pub format: String,

    /// Text or markdown content.
    pub content: String,
}

#[derive(FromPyObject, Debug)]
pub enum ToolResult {
    String(String),
    Int(i64),
    Float(f64),
    Text(ContentText),
    Image(ContentImage),
    Audio(ContentAudio),
    Video(ContentVideo),
    List(Vec<ToolResultItem>),
}

#[derive(FromPyObject, Debug)]
pub enum ToolResultItem {
    Text(ContentText),
    Image(ContentImage),
    Audio(ContentAudio),
    Video(ContentVideo),
}

/// Error raised by a tool call.
#[derive(FromPyObject, Debug)]
pub struct ToolCallError {
    /// Error type.
    #[pyo3(attribute("type"))]
    pub error_type: String,

    /// Error message.
    pub message: String,
}

#[derive(Debug)]
pub struct ApprovalEvent {
    pub base: BaseEvent,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ApprovalEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct InputEvent {
    pub base: BaseEvent,
}

impl<'a, 'py> FromPyObject<'a, 'py> for InputEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct ScoreEvent {
    pub base: BaseEvent,

    /// Score value.
    pub score: Score,

    /// Sample target.
    pub target: Option<Target>,

    /// Was this an intermediate scoring?
    pub intermediate: bool,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ScoreEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            score: ob.getattr("score")?.extract()?,
            target: ob.getattr("target")?.extract()?,
            intermediate: ob.getattr("intermediate")?.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct ErrorEvent {
    pub base: BaseEvent,

    /// Sample error
    pub error: EvalError,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ErrorEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            error: ob.getattr("error")?.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct LoggerEvent {
    pub base: BaseEvent,
}

impl<'a, 'py> FromPyObject<'a, 'py> for LoggerEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct InfoEvent {
    pub base: BaseEvent,
}

impl<'a, 'py> FromPyObject<'a, 'py> for InfoEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
        })
    }
}

/// Mark the beginning of a transcript span.
#[derive(Debug)]
pub struct SpanBeginEvent {
    pub base: BaseEvent,

    /// Unique identifier for span.
    pub id: String,

    /// Identifier for parent span.
    pub parent_id: Option<String>,

    /// Optional 'type' field for span.
    pub span_type: Option<String>,

    /// Span name.
    pub name: String,
}

impl<'a, 'py> FromPyObject<'a, 'py> for SpanBeginEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            id: ob.getattr("id")?.extract()?,
            parent_id: ob.getattr("parent_id")?.extract()?,
            span_type: ob.getattr("type")?.extract()?,
            name: ob.getattr("name")?.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct SpanEndEvent {
    pub base: BaseEvent,
}

impl<'a, 'py> FromPyObject<'a, 'py> for SpanEndEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct StepEvent {
    pub base: BaseEvent,
}

impl<'a, 'py> FromPyObject<'a, 'py> for StepEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
        })
    }
}

#[derive(Debug)]
pub struct SubtaskEvent {
    pub base: BaseEvent,
}

impl<'a, 'py> FromPyObject<'a, 'py> for SubtaskEvent {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
        })
    }
}
