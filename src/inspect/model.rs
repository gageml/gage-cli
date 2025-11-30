#![allow(dead_code)]

use std::{collections::HashMap, fmt::Display};

use pyo3::{
    Borrowed, FromPyObject, PyAny, PyErr, PyResult, exceptions::PyTypeError, types::PyAnyMethods,
};

use crate::{
    inspect::{Metadata, json::JsonValue},
    py::Any,
};

#[derive(Debug)]
pub enum ChatMessage {
    System(ChatMessageBase),
    User(ChatMessageUser),
    Assistant(ChatMessageAssistant),
    Tool(ChatMessageTool),
}

impl<'a, 'py> FromPyObject<'a, 'py> for ChatMessage {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let role = ob.getattr("role")?.extract::<String>()?;
        match role.as_str() {
            "system" => Ok(Self::System(ob.extract()?)),
            "user" => Ok(Self::User(ob.extract()?)),
            "assistant" => Ok(Self::Assistant(ob.extract()?)),
            "tool" => Ok(Self::Tool(ob.extract()?)),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "unknown chat message role '{role}'"
            ))),
        }
    }
}

impl ChatMessage {
    pub fn role(&self) -> &str {
        match self {
            Self::System(_) => "system",
            Self::User(_) => "user",
            Self::Assistant(_) => "assistant",
            Self::Tool(_) => "tool",
        }
    }

    pub fn base(&self) -> &ChatMessageBase {
        match self {
            Self::System(base) => base,
            Self::User(user) => &user.base,
            Self::Assistant(assistant) => &assistant.base,
            Self::Tool(tool) => &tool.base,
        }
    }
}

#[derive(FromPyObject, Debug)]
pub struct ChatMessageBase {
    /// Unique identifer for message.
    pub id: Option<String>,

    /// Content (simple string or list of content objects)
    pub content: ChatMessageContent,

    /// Source of message.
    pub source: Option<String>,

    /// Additional message metadata.
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
pub struct ChatMessageUser {
    pub base: ChatMessageBase,

    /// ID(s) of tool call(s) this message has the content payload for.
    pub tool_call_id: Option<Vec<String>>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ChatMessageUser {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            tool_call_id: ob.getattr("tool_call_id")?.extract()?,
        })
    }
}

/// Assistant chat message.
#[derive(Debug)]
pub struct ChatMessageAssistant {
    pub base: ChatMessageBase,

    /// Tool calls made by the model.
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Model used to generate assistant message.
    pub model: Option<String>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ChatMessageAssistant {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            tool_calls: ob.getattr("tool_calls")?.extract()?,
            model: ob.getattr("model")?.extract()?,
        })
    }
}

#[derive(FromPyObject, Debug)]
pub struct ToolCall {
    /// Unique identifier for tool call.
    pub id: String,

    /// Function called.
    pub function: String,

    /// Arguments to function.
    pub arguments: HashMap<String, Any>,

    /// Error which occurred parsing tool call.
    pub parse_error: Option<String>,

    /// Custom view of tool call input.
    pub view: Option<ToolCallContent>,

    /// Type of tool call.
    #[pyo3(attribute("type"))]
    pub call_type: String,
}

/// Content to include in tool call view.
#[derive(FromPyObject, Debug)]
pub struct ToolCallContent {
    /// Optional (plain text) title for tool call content.
    pub title: Option<String>,

    /// Format (text or markdown).
    ///
    /// Literal["text", "markdown"]
    pub format: String,

    /// Text or markdown content.
    pub content: String,
}

#[derive(Debug)]
pub struct ChatMessageTool {
    pub base: ChatMessageBase,

    /// ID of tool call.
    pub tool_call_id: Option<String>,

    /// Name of function called.
    pub function: Option<String>,

    /// Error which occurred during tool call.
    pub error: Option<ToolCallError>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ChatMessageTool {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(Self {
            base: ob.extract()?,
            tool_call_id: ob.getattr("tool_call_id")?.extract()?,
            function: ob.getattr("function")?.extract()?,
            error: ob.getattr("error")?.extract()?,
        })
    }
}

/// Error raised by a tool call.
#[derive(FromPyObject, Debug)]
pub struct ToolCallError {
    /// Error type.
    ///
    /// type: Literal[
    ///     "parsing",
    ///     "timeout",
    ///     "unicode_decode",
    ///     "permission",
    ///     "file_not_found",
    ///     "is_a_directory",
    ///     "limit",
    ///     "approval",
    ///     "unknown",
    ///     # Retained for backward compatibility when loading logs created with an older
    ///     # version of inspect.
    ///     "output_limit",
    /// ]
    #[pyo3(attribute("type"))]
    pub error_type: String,

    /// Error message.
    pub message: String,
}

#[derive(FromPyObject, Debug)]
pub enum ChatMessageContent {
    String(String),
    ContentList(Vec<Content>),
}

impl ChatMessageContent {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::String(s) => s.is_empty(),
            Self::ContentList(l) => l.is_empty(),
        }
    }
}

#[derive(Debug)]
pub enum Content {
    Text(ContentText),
    Reasoning(ContentReasoning),
    Image(ContentImage),
    Audio(ContentAudio),
    Video(ContentVideo),
    Data(ContentData),
    ToolUse(ContentToolUse),
    Document(ContentDocument),
}

impl<'a, 'py> FromPyObject<'a, 'py> for Content {
    type Error = PyErr;
    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        match obj.getattr("type")?.extract::<String>()?.as_str() {
            "text" => ContentText::extract(obj).map(Self::Text),
            "reasoning" => ContentReasoning::extract(obj).map(Self::Reasoning),
            "image" => ContentImage::extract(obj).map(Self::Image),
            "audio" => ContentAudio::extract(obj).map(Self::Audio),
            "video" => ContentVideo::extract(obj).map(Self::Video),
            "data" => ContentData::extract(obj).map(Self::Data),
            "tool_use" => ContentToolUse::extract(obj).map(Self::ToolUse),
            "document" => ContentDocument::extract(obj).map(Self::Document),
            other => Err(PyErr::new::<PyTypeError, _>(format!(
                "unknown content type '{other}'"
            ))),
        }
    }
}

impl Content {
    pub fn type_name(&self) -> &str {
        match self {
            Self::Text(_) => "text",
            Self::Reasoning(_) => "reasoning",
            Self::Image(_) => "image",
            Self::Audio(_) => "audio",
            Self::Video(_) => "video",
            Self::Data(_) => "data",
            Self::ToolUse(_) => "tool_use",
            Self::Document(_) => "document",
        }
    }
}

#[derive(FromPyObject, Debug)]
pub struct ContentText {
    /// Text content.
    pub text: String,

    /// Was this a refusal message?
    pub refusal: Option<bool>,
    //
    // citations: Sequence[Citation] | None = Field(default=None)
    // """Citations supporting the text block."""
}

#[derive(FromPyObject, Debug)]
pub struct ContentReasoning {
    /// Reasoning content.
    pub reasoning: String,

    /// Reasoning summary.
    pub summary: Option<String>,

    /// Signature for reasoning content (used by some models to ensure
    /// that reasoning content is not modified for replay)
    pub signature: Option<String>,

    /// Indicates that the explicit content of this reasoning block has
    /// been redacted.
    pub redacted: bool,
}

#[derive(FromPyObject, Debug)]
pub struct ContentImage {
    /// Either a URL of the image or the base64 encoded image data.
    pub image: String,

    /// Specifies the detail level of the image.
    pub detail: String,
}

#[derive(FromPyObject, Debug)]
pub struct ContentAudio {
    /// Audio file path or base64 encoded data URL.
    pub audio: String,

    /// Format of audio data ('mp3' or 'wav')
    pub format: String,
}

#[derive(FromPyObject, Debug)]
pub struct ContentVideo {
    /// Video file path or base64 encoded data URL.
    pub video: String,

    /// Format of video data ('mp4', 'mpeg', or 'mov')
    pub format: String,
}

#[derive(FromPyObject, Debug)]
pub struct ContentData {
    /// Model provider specific payload - required for internal content.
    pub data: HashMap<String, JsonValue>,
}

#[derive(FromPyObject, Debug)]
pub struct ContentToolUse {
    /// The type of the tool call.
    pub tool_type: String,

    /// The unique ID of the tool call.
    pub id: String,

    /// Name of the tool.
    pub name: String,

    /// Tool context (e.g. MCP Server)
    pub context: Option<String>,

    /// Arguments passed to the tool.
    pub arguments: String,

    /// Result from the tool call.
    pub result: String,

    /// The error from the tool call (if any).
    pub error: Option<String>,
}

#[derive(FromPyObject, Debug)]
pub struct ContentDocument {
    /// Document file path or base64 encoded data URL.
    pub document: String,

    /// Document filename (automatically determined from 'document' if not specified).
    pub filename: String,

    /// Document mime type (automatically determined from 'document' if not specified).
    pub mime_type: String,
}

#[derive(FromPyObject, Debug)]
pub struct ModelOutput {
    /// Model used for generation.
    pub model: String,

    /// Completion choices.
    pub choices: Vec<ChatCompletionChoice>,

    /// Model completion.
    pub completion: String,

    /// Model token usage
    pub usage: Option<ModelUsage>,

    /// Time elapsed (in seconds) for call to generate.
    pub time: Option<f64>,

    /// Additional metadata associated with model output.
    pub metadata: Option<Metadata>,

    /// Error message in the case of content moderation refusals.
    pub error: Option<String>,
}

#[derive(FromPyObject, Debug)]
pub struct ChatCompletionChoice {
    /// Assistant message.
    pub message: ChatMessageAssistant,

    /// Reason that the model stopped generating.
    pub stop_reason: StopReason,
    //
    // logprobs: Logprobs | None = Field(default=None)
    // """Logprobs."""
}

#[derive(Debug)]
pub enum StopReason {
    Stop,
    MaxTokens,
    ModelLength,
    ToolCalls,
    ContentFilter,
    Unknown,
}

impl<'a, 'py> FromPyObject<'a, 'py> for StopReason {
    type Error = PyErr;
    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        match obj.extract::<String>()?.as_str() {
            "stop" => Ok(Self::Stop),
            "max_tokens" => Ok(Self::MaxTokens),
            "model_length" => Ok(Self::ModelLength),
            "tool_calls" => Ok(Self::ToolCalls),
            "ContentFilter" => Ok(Self::ContentFilter),
            "unknown" => Ok(Self::Unknown),
            other => Err(PyErr::new::<PyTypeError, _>(format!(
                "unknown stop reason '{other}'"
            ))),
        }
    }
}

impl Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop => "stop",
            Self::MaxTokens => "max_tokens",
            Self::ModelLength => "model_length",
            Self::ToolCalls => "tool_calls",
            Self::ContentFilter => "content_filter",
            Self::Unknown => "unknown",
        }
        .fmt(f)
    }
}

#[derive(FromPyObject, Debug)]
pub struct ModelUsage {
    // Total input tokens used.
    pub input_tokens: usize,

    /// Total output tokens used.
    pub output_tokens: usize,

    /// Total tokens used.
    pub total_tokens: usize,

    /// Number of tokens written to the cache.
    pub input_tokens_cache_write: Option<usize>,

    /// Number of tokens retrieved from the cache.
    pub input_tokens_cache_read: Option<usize>,

    /// Number of tokens used for reasoning.
    pub reasoning_tokens: Option<usize>,
}

#[cfg(test)]
mod tests {
    use pyo3::{
        Python,
        ffi::c_str,
        types::{PyAnyMethods, PyModule},
    };

    use crate::inspect::model::ChatMessage;

    #[test]
    fn test_chat_message_extract() {
        Python::initialize();
        Python::attach(|py| {
            let test_mod = PyModule::from_code(
                py,
                c_str!(
                    r#"
from inspect_ai.model import *
system = ChatMessageSystem(content="")
user = ChatMessageUser(content="")
assistant = ChatMessageAssistant(content="")
tool = ChatMessageTool(content="")
"#
                ),
                c_str!("test.py"),
                c_str!("test"),
            )
            .unwrap();

            // System message
            let msg: ChatMessage = test_mod.getattr("system").unwrap().extract().unwrap();
            assert_eq!("System(ChatMessageBase", format!("{msg:?}").split_at(22).0);

            // User message
            let msg: ChatMessage = test_mod.getattr("user").unwrap().extract().unwrap();
            assert_eq!("User(ChatMessageUser", format!("{msg:?}").split_at(20).0);

            // Assistant message
            let msg: ChatMessage = test_mod.getattr("assistant").unwrap().extract().unwrap();
            assert_eq!(
                "Assistant(ChatMessageAssistant",
                format!("{msg:?}").split_at(30).0
            );

            // Tool message
            let msg: ChatMessage = test_mod.getattr("tool").unwrap().extract().unwrap();
            assert_eq!("Tool(ChatMessageTool", format!("{msg:?}").split_at(20).0);
        });
    }
}
