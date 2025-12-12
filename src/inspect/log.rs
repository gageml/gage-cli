#![allow(dead_code)]

use std::{
    collections::HashMap,
    fmt::Display,
    io,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use pyo3::{
    Borrowed, Bound, FromPyObject, PyAny, PyErr, PyResult, Python, exceptions::PyValueError,
    types::PyAnyMethods,
};

use crate::{
    env,
    inspect::{
        Args, Attributes, Metadata,
        error::EvalError,
        event::{Event, SpanBeginEvent, StateEvent},
        model::{ChatMessage, ModelOutput, ModelUsage},
        scorer::Score,
    },
    py::{Any, EpochMillis, py_call},
    result::Result,
    util::{PathExt, find_try_parents},
};

pub const LOG_VERSION: u16 = 2;

#[derive(FromPyObject, Debug, Clone)]
pub struct EvalLogInfo {
    pub name: String,
    pub mtime: Option<EpochMillis>,
    pub task: String,

    /// Log Id, derived from log name
    #[pyo3(attribute("name"), from_py_with = EvalLogInfo::log_id_from_py_name)]
    pub log_id: String,
}

impl EvalLogInfo {
    fn log_id_from_py_name<'py>(py_name: &Bound<'py, PyAny>) -> PyResult<String> {
        let name = py_name.extract::<String>()?;
        name.rsplit_once("_")
            .and_then(|(_, suffix)| suffix.split_once("."))
            .map(|(id, _)| id.into())
            .ok_or(PyErr::new::<PyValueError, _>(format!(
                "cannot get log ID from name '{name}'"
            )))
    }

    pub fn short_log_id(&self) -> &str {
        self.log_id
            .split_at_checked(6)
            .map(|(short_id, _)| short_id)
            .unwrap_or(&self.log_id)
    }
}
#[derive(FromPyObject, Debug)]
pub struct EvalLog {
    /// Eval log file format version.
    pub version: u16,

    /// Status of evaluation (did it succeed or fail).
    pub status: EvalStatus,

    /// Eval identity and configuration.
    pub eval: EvalSpec,

    /// Eval plan (solvers and config)
    pub plan: EvalPlan,

    /// Eval results (scores and metrics).
    pub results: Option<EvalResults>,

    /// Eval stats (runtime, model usage)
    pub stats: EvalStats,

    /// Error that halted eval (if status=="error")
    pub error: Option<EvalError>,

    /// Samples processed by eval.
    pub samples: Option<Vec<EvalSample>>,

    /// Reduced sample values
    pub reductions: Option<Vec<EvalSampleReductions>>,

    /// Location that the log file was read from.
    pub location: String,
    //
    // /// ETag from S3 for conditional writes.
    // pub etag: Option<String>,
}

impl EvalLog {
    pub fn short_log_id(&self) -> &str {
        let id = &self.eval.task_id;
        match id.split_at_checked(6) {
            Some((s, _)) => s,
            None => id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum EvalStatus {
    Started,
    Success,
    Cancelled,
    Error,
}

impl<'a, 'py> FromPyObject<'a, 'py> for EvalStatus {
    type Error = PyErr;
    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        match ob.extract::<String>()?.as_str() {
            "started" => Ok(Self::Started),
            "success" => Ok(Self::Success),
            "cancelled" => Ok(Self::Cancelled),
            "error" => Ok(Self::Error),
            other => Err(PyErr::new::<PyValueError, _>(format!(
                "unknown eval status '{other}'"
            ))),
        }
    }
}

impl Display for EvalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Started => f.write_str("started"),
            Self::Success => f.write_str("success"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Error => f.write_str("error"),
        }
    }
}

#[derive(FromPyObject, Debug)]
pub struct EvalSpec {
    /// Globally unique id for eval set (if any).
    pub eval_set_id: Option<String>,

    /// Globally unique id for eval.
    pub eval_id: String,

    /// Unique run id
    pub run_id: String,

    /// Time created.
    pub created: EpochMillis,

    /// Task name.
    pub task: String,

    /// Unique task id.
    pub task_id: String,

    /// Task version.
    pub task_version: TaskVersion,

    /// Task source file.
    pub task_file: Option<String>,

    /// Task display name.
    pub task_display_name: Option<String>,

    /// Task registry name.
    pub task_registry_name: Option<String>,

    /// Attributes of the @task decorator.
    pub task_attribs: Attributes,

    /// Arguments used for invoking the task (including defaults).
    pub task_args: Args,

    /// Arguments explicitly passed by caller for invoking the task.
    pub task_args_passed: Args,

    /// Solver name.
    pub solver: Option<String>,

    /// Arguments used for invoking the solver.
    pub solver_args: Option<Args>,

    /// Tags associated with evaluation run.
    pub tags: Option<Vec<String>>,

    // Dataset used for eval.
    pub dataset: EvalDataset,

    // sandbox: SandboxEnvironmentSpec | None = Field(default=None)
    // """Sandbox environment type and optional config file."""
    //
    /// Model used for eval.
    pub model: String,

    // model_generate_config: GenerateConfig = Field(default_factory=GenerateConfig)
    // """Generate config specified for model instance."""
    //
    /// Optional override of model base url.
    pub model_base_url: Option<String>,

    /// Model specific arguments.
    pub model_args: Args,

    // model_roles: dict[str, ModelConfig] | None = Field(default=None)
    // """Model roles."""

    // config: EvalConfig
    // """Configuration values for eval."""
    //
    /// Source revision of eval.
    pub revision: Option<EvalRevision>,

    /// Package versions for eval.
    pub packages: HashMap<String, String>,

    /// Additional eval metadata.
    pub metadata: Option<Metadata>,

    /// Scorers and args for this eval.
    pub scorers: Option<Vec<EvalScorer>>,

    /// Metrics and args for this eval.
    pub metrics: Option<Metrics>,
    //
    // # allow field model_args
    // model_config = ConfigDict(protected_namespaces=())
}

impl EvalSpec {
    pub fn task_description(&self) -> Option<String> {
        self.task_attribs
            .get("description")
            .map(|val| val.to_string())
    }

    pub fn run_type(&self) -> Option<String> {
        Some(
            self.tags
                .as_ref()?
                .iter()
                .find_map(|val| val.strip_prefix("type:"))?
                .to_string(),
        )
    }
}

#[derive(FromPyObject, Debug)]
pub enum TaskVersion {
    Str(String),
    Int(u32),
}

impl Display for TaskVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => s.fmt(f),
            Self::Int(i) => i.fmt(f),
        }
    }
}

#[derive(FromPyObject, Debug)]
pub struct EvalRevision {
    // Type of revision (currently only "git")
    // type: String,
    /// Revision origin server.
    pub origin: String,

    /// Revision commit.
    pub commit: String,
}

impl Display for EvalRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.origin, self.commit)
    }
}

#[derive(FromPyObject, Debug)]
pub struct EvalDataset {
    /// Dataset name.
    pub name: Option<String>,

    /// Dataset location (file path or remote URL)
    pub location: Option<String>,

    /// Number of samples in the dataset.
    pub samples: Option<usize>,

    /// """IDs of samples in the dataset."""
    pub sample_ids: Option<Vec<SampleId>>,

    /// Was the dataset shuffled after reading.
    pub shuffled: Option<bool>,
}

impl EvalDataset {
    pub fn evaluated_count(&self) -> Option<usize> {
        self.sample_ids.as_ref().map(|ids| ids.len())
    }
}

#[derive(FromPyObject, Debug)]
pub struct EvalScorer {
    /// Scorer name
    pub name: String,

    // Scorer arguments
    pub options: Option<HashMap<String, Any>>,

    /// Scorer metrics.
    pub metrics: Option<Metrics>,

    /// Scorer metadata.
    pub metadata: Option<Metadata>,
}

#[derive(FromPyObject, Debug)]
pub enum Metrics {
    List(Vec<MetricsOneOrMap>),
    Map(HashMap<String, Vec<EvalMetricDefinition>>),
}

#[derive(FromPyObject, Debug)]
pub enum MetricsOneOrMap {
    One(EvalMetricDefinition),
    Map(HashMap<String, Vec<EvalMetricDefinition>>),
}

#[derive(FromPyObject, Debug)]
pub struct EvalMetricDefinition {
    /// Metric name
    pub name: String,

    /// Metric options
    pub options: Option<HashMap<String, Any>>,
}

impl<'a> IntoIterator for &'a Metrics {
    type Item = (Option<&'a String>, &'a EvalMetricDefinition);

    type IntoIter = MetricsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Metrics::List(l) => MetricsIter::from_list(l),
            Metrics::Map(m) => MetricsIter::from_map(m),
        }
    }
}

type MetricsMapIter<'a> = std::collections::hash_map::Iter<'a, String, Vec<EvalMetricDefinition>>;

pub enum MetricsIter<'a> {
    List {
        inner: std::slice::Iter<'a, MetricsOneOrMap>,
        map_iter: Option<MetricsMapIter<'a>>,
        list_iter: Option<(&'a String, std::slice::Iter<'a, EvalMetricDefinition>)>,
    },
    Map {
        inner: MetricsMapIter<'a>,
        list_iter: Option<(&'a String, std::slice::Iter<'a, EvalMetricDefinition>)>,
    },
}

impl<'a> MetricsIter<'a> {
    fn from_list(l: &'a [MetricsOneOrMap]) -> Self {
        Self::List {
            inner: l.iter(),
            map_iter: None,
            list_iter: None,
        }
    }

    fn from_map(m: &'a HashMap<String, Vec<EvalMetricDefinition>>) -> Self {
        Self::Map {
            inner: m.iter(),
            list_iter: None,
        }
    }
}

impl<'a> Iterator for MetricsIter<'a> {
    type Item = (Option<&'a String>, &'a EvalMetricDefinition);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::List {
                inner,
                map_iter,
                list_iter: None,
            } if map_iter.is_none() => {
                // At top level of iter
                match inner.next()? {
                    MetricsOneOrMap::One(metric) => {
                        // Yield single metric without name
                        Some((None, metric))
                    }
                    MetricsOneOrMap::Map(m) => {
                        // Move to next level (map)
                        *map_iter = Some(m.iter());
                        self.next()
                    }
                }
            }
            Self::List {
                inner: _,
                map_iter,
                list_iter,
            } if list_iter.is_none() => {
                // At map level of iter
                let unwrapped_map_iter = match map_iter {
                    Some(v) => v,
                    None => panic!("invalid state - see prev arm"),
                };
                if let Some((name, l)) = unwrapped_map_iter.next() {
                    // Move to next level (list)
                    *list_iter = Some((name, l.iter()));
                } else {
                    // Move up to top level
                    *map_iter = None;
                }
                self.next()
            }
            Self::List {
                inner: _,
                map_iter: _,
                list_iter,
            } => {
                // At list level of iter
                let (name, unwrapped_list_iter) = match list_iter {
                    Some(v) => v,
                    None => panic!("invalid state - see prev arms"),
                };
                if let Some(metric) = unwrapped_list_iter.next() {
                    // Yield with name from parent map
                    Some((Some(name), metric))
                } else {
                    // Move up to map level
                    *list_iter = None;
                    self.next()
                }
            }
            Self::Map { inner, list_iter } if list_iter.is_none() => {
                // At top level of iter
                let (name, l) = inner.next()?;

                // Move to next level (list)
                *list_iter = Some((name, l.iter()));
                self.next()
            }
            Self::Map {
                inner: _,
                list_iter,
            } => {
                let (name, unwrapped_list_iter) = match list_iter {
                    Some(v) => v,
                    None => panic!("invalid state - see prev arms"),
                };
                if let Some(metric) = unwrapped_list_iter.next() {
                    // Yield with name from parent map
                    Some((Some(name), metric))
                } else {
                    // Move up to map level
                    *list_iter = None;
                    self.next()
                }
            }
        }
    }
}

#[derive(FromPyObject, Debug)]
pub struct EvalPlan {
    /// Plan name.
    pub name: String,

    /// Steps in plan.
    pub steps: Vec<EvalPlanStep>,

    /// Step to always run at the end.
    pub finish: Option<EvalPlanStep>,
    //
    // config: GenerateConfig = Field(default=GenerateConfig())
    // """Generation config."""
}

#[derive(FromPyObject, Debug)]
pub struct EvalPlanStep {
    /// Name of solver.
    pub solver: String,

    /// Parameters used to instantiate solver.
    pub params: HashMap<String, Any>,
}

#[derive(FromPyObject, Debug)]
pub struct EvalResults {
    /// Total samples in eval (dataset samples * epochs)
    pub total_samples: usize,

    /// Samples completed without error.
    pub completed_samples: usize,

    /// Scorers used to compute results.
    pub scores: Vec<EvalScore>,

    /// Additional results metadata.
    pub metadata: Option<Metadata>,
}

impl EvalResults {
    pub fn first_accuracy(&self) -> Option<f64> {
        Some(match self.scores.first()?.metrics.get("accuracy")?.value {
            MetricVal::Int(i) => i as f64,
            MetricVal::Float(f) => f,
        })
    }

    pub fn first_stderr(&self) -> Option<f64> {
        Some(match self.scores.first()?.metrics.get("stderr")?.value {
            MetricVal::Int(i) => i as f64,
            MetricVal::Float(f) => f,
        })
    }
}

#[derive(FromPyObject, Debug)]
pub struct EvalScore {
    /// Score name.
    pub name: String,

    /// Scorer name.
    pub scorer: String,

    /// Reducer name.
    pub reducer: Option<String>,

    /// Number of samples scored by this scorer.
    pub scored_samples: Option<usize>,

    /// Number of samples not scored by this scorer.
    pub unscored_samples: Option<usize>,

    /// Parameters specified when creating scorer.
    pub params: HashMap<String, Any>,

    /// Metrics computed for this scorer.
    pub metrics: HashMap<String, EvalMetric>,

    /// Additional scorer metadata.
    pub metadata: Option<Metadata>,
}

#[derive(FromPyObject, Debug)]
pub struct EvalMetric {
    /// Metric name.
    pub name: String,

    /// Metric value.
    pub value: MetricVal,

    /// Params specified when creating metric.
    pub params: HashMap<String, Any>,

    /// Additional metadata associated with metric.
    pub metadata: Option<Metadata>,
}

#[derive(FromPyObject, Debug)]
pub enum MetricVal {
    Int(i64),
    Float(f64),
}

impl Display for MetricVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => i.fmt(f),
            Self::Float(float) => float.fmt(f),
        }
    }
}

#[derive(FromPyObject, Debug)]
pub struct EvalStats {
    /// Evaluation start time.
    pub started_at: EpochMillis,

    /// Evaluation completion time.
    pub completed_at: EpochMillis,

    /// Model token usage for evaluation.
    pub model_usage: HashMap<String, ModelUsage>,
    //
    // # allow field model_usage
    // model_config = ConfigDict(protected_namespaces=())
}

#[derive(FromPyObject, Debug)]
pub struct EvalSample {
    /// Unique id for sample.
    pub id: SampleId,

    /// Epoch number for sample.
    pub epoch: i64,

    /// Sample input.
    pub input: SampleInput,

    /// Sample target value(s)
    pub target: Target,

    // sandbox: SandboxEnvironmentSpec | None = Field(default=None)
    // """Sandbox environment type and optional config file."""
    //
    /// Files that go along with the sample (copied to SandboxEnvironment)
    pub files: Option<Vec<String>>,

    /// Setup script to run for sample (run within default SandboxEnvironment).
    pub setup: Option<String>,

    /// Chat conversation history for sample.
    pub messages: Vec<ChatMessage>,

    /// Model output from sample.
    pub output: ModelOutput,

    /// Scores for sample.
    pub scores: Option<HashMap<String, Score>>,

    /// Additional sample metadata.
    pub metadata: Metadata,

    // store: dict[str, Any] = Field(default_factory=dict)
    // """State at end of sample execution."""
    //
    /// Events that occurred during sample execution.
    pub events: Vec<Event>,
    //
    // model_usage: dict[str, ModelUsage] = Field(default_factory=dict)
    // """Model token usage for sample."""

    // total_time: float | None = Field(default=None)
    // """Total time that the sample was running."""

    // working_time: float | None = Field(default=None)
    // """Time spent working (model generation, sandbox calls, etc.)"""

    // uuid: str | None = Field(default=None)
    // """Globally unique identifier for sample run (exists for samples created in Inspect >= 0.3.70)"""

    // error: EvalError | None = Field(default=None)
    // """Error that halted sample."""

    // error_retries: list[EvalError] | None = Field(default=None)
    // """Errors that were retried for this sample."""
    //
    /// Attachments referenced from messages and events.
    pub attachments: Attachments,
    //
    // limit: EvalSampleLimit | None = Field(default=None)
    // """The limit that halted the sample"""

    // # allow field model_usage
    // model_config = ConfigDict(protected_namespaces=())
}

pub type Attachments = HashMap<String, String>;

impl EvalSample {
    pub fn steps<'a>(&'a self) -> StepsIterator<'a> {
        StepsIterator::new(&self.events)
    }

    pub fn errors(&self) -> Vec<&EvalError> {
        self.events
            .iter()
            .filter_map(|e| {
                if let Event::Error(e) = e {
                    Some(&e.error)
                } else {
                    None
                }
            })
            .collect_vec()
    }

    /// Returns an option of bool, where bool is true if at least one
    /// score value is "C" and there are no "I" scores. Returns None if
    /// there are no scores.
    pub fn is_correct(&self) -> Option<bool> {
        if let Some(scores) = self.scores.as_ref()
            && !scores.is_empty()
        {
            let mut seen_c = false;
            for score in scores.values() {
                let score_as_str = score.value.to_string();
                if score_as_str == "I" {
                    // Any one "I" -> incorrect
                    return Some(false);
                }
                seen_c = seen_c || score_as_str == "C";
            }
            // Any one "C" at this point -> correct
            Some(seen_c)
        } else {
            // Nothing to consider -> None
            None
        }
    }

    /// Returns the default score for a sample.
    ///
    /// The default score is one who's value is either "I" or "C". If
    /// there is more than one such score, returns the first score using
    /// name sort order.
    pub fn default_score(&self) -> Option<(&String, &Score)> {
        if let Some(scores) = self.scores.as_ref() {
            for name in scores.keys().sorted() {
                let score = &scores[name];
                match score.value.to_string().as_str() {
                    "I" | "C" => return Some((name, score)),
                    _ => {}
                }
            }
        }
        None
    }
}

pub struct StepsIterator<'a> {
    state_events: HashMap<String, &'a StateEvent>,
    inner: std::slice::Iter<'a, Event>,
}

impl<'a> StepsIterator<'a> {
    fn new(events: &'a [Event]) -> Self {
        let state_events = events
            .iter()
            .filter_map(|event| {
                if let Event::State(state) = event
                    && let Some(span_id) = state.base.span_id.as_ref()
                {
                    Some((span_id.clone(), state))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        Self {
            state_events,
            inner: events.iter(),
        }
    }
}

#[derive(Debug)]
pub struct Step<'a> {
    pub span: &'a SpanBeginEvent,
    pub state: Option<&'a StateEvent>,
}

impl<'a> Iterator for StepsIterator<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for event in self.inner.by_ref() {
            if let Event::SpanBegin(span) = event
                && span.span_type.as_deref() == Some("solver")
            {
                return Some(Step {
                    span,
                    state: self.state_events.get(&span.id).copied(),
                });
            }
        }
        None
    }
}

#[derive(FromPyObject, Debug)]
pub enum SampleId {
    Int(i32),
    Str(String),
}

impl Display for SampleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => i.fmt(f),
            Self::Str(s) => s.fmt(f),
        }
    }
}

#[derive(FromPyObject, Debug)]
pub enum SampleInput {
    String(String),
    ChatMessageList(Vec<ChatMessage>),
}

#[derive(FromPyObject, Debug)]
pub enum Target {
    String(String),
    List(Vec<String>),
}

impl Target {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::String(s) => s.is_empty(),
            Self::List(l) => l.is_empty(),
        }
    }

    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::String(s) => vec![s.clone()],
            Self::List(l) => l.clone(),
        }
    }
}

#[derive(Debug, FromPyObject)]
pub struct EvalSampleReductions {
    /// Name the of scorer
    pub scorer: String,

    /// Name the of reducer
    pub reducer: Option<String>,

    /// List of reduced scores
    pub samples: Vec<EvalSampleScore>,
}

#[derive(Debug, FromPyObject)]
pub struct EvalSampleScore {
    /// Sample ID.
    pub sample_id: Option<SampleId>,
}

pub fn resolve_log_dir(log_dir: Option<&PathBuf>) -> PathBuf {
    log_dir
        .cloned()
        .or_else(|| env::get("INSPECT_LOG_DIR").map(PathBuf::from))
        .or_else(|| {
            find_try_parents("logs")
                .or_else(|e| {
                    log::warn!("Error finding logs dir: {e}");
                    Ok::<Option<PathBuf>, io::Error>(None)
                })
                .unwrap()
        })
        .unwrap_or(PathBuf::from("logs"))
}

pub fn list_logs<'py>(py: Python<'py>, log_dir: &Path) -> Result<Vec<EvalLogInfo>> {
    list_logs_filter(py, log_dir, LogFilter::None)
}

pub enum LogFilter {
    Deleted,
    None,
}

pub fn list_logs_filter<'py, F>(
    py: Python<'py>,
    log_dir: &Path,
    filter: F,
) -> Result<Vec<EvalLogInfo>>
where
    F: Into<LogFilter>,
{
    if let Ok(false) = std::fs::exists(log_dir) {
        return Ok(Vec::new());
    }
    let deleted = match filter.into() {
        LogFilter::Deleted => true,
        LogFilter::None => false,
    };
    let mut logs: Vec<EvalLogInfo> = py_call(
        py,
        "gage_inspect.log",
        "list_logs",
        (log_dir.expect_string(), deleted),
    )?
    .extract()
    .unwrap();
    // Sort using name, which contains a leading create timestamp,
    // showing most recently created logs first
    logs.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name).reverse());
    Ok(logs)
}

pub fn read_log<'py>(py: Python<'py>, path: &str) -> Result<EvalLog> {
    Ok(py_call(py, "gage_inspect.log", "read_eval_log", (path,))?.extract()?)
}

pub fn read_log_header<'py>(py: Python<'py>, path: &str) -> Result<EvalLog> {
    Ok(py_call(py, "gage_inspect.log", "read_eval_log", (path, true))?.extract()?)
}
