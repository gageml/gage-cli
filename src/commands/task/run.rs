use std::{cmp::min, path::PathBuf};

use clap::Args as ArgsTrait;
use cliclack as cli;
use console::style;
use itertools::Itertools;
use lazy_static::lazy_static;
use pyo3::Python;
use regex::Regex;

use crate::{
    commands::task::{select_model_dialog, select_task_dialog},
    config::Config,
    dialog::{DialogInfo, DialogResult, handle_dialog_result},
    error::Error,
    inspect::{
        log::{EvalStatus, resolve_log_dir},
        model::{ChatMessageAssistant, ChatMessageContent, Content, ModelOutput},
        scorer::Score,
        task::{TaskInfo, get_task_doc, run_task},
    },
    profile::apply_profile_with_secrets,
    py::{self, Docstring},
    result::Result,
    util::{PathExt, term_width, wrap, wrap_map},
};

lazy_static! {
    static ref DEP_ERROR_P: Regex = Regex::new(
        r"(?s)PrerequisiteError: \[bold\]ERROR\[/bold\]: ([\w ]+) requires .*?\[bold\]pip install (\w+)\[/bold\]").unwrap();
    static ref CLIENT_INIT_ERROR_P: Regex = Regex::new(r"(?s)ERROR: Unable to initialise ([\w ]+) client").unwrap();
    static ref CLIENT_ENV_P: Regex = Regex::new(r"(?s)\[bold\]\[blue\](.+?)\[/blue\]\[/bold\]").unwrap();
}

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Task to run
    task: Option<String>,

    /// Task input
    #[arg(short, long)]
    input: Option<String>,

    /// Task argument NAME=VALUE (may use more than once)
    #[arg(short = 'T', value_name = "TASK_ARG")]
    task_args: Vec<String>,

    /// Task model
    ///
    /// If not specified, environent variables GAGE_MODEL and
    /// INSPECT_EVAL_MODEL are used. Otherwise the model defined by the
    /// task itself is used.
    #[arg(short, long)]
    model: Option<String>,

    /// Expected output
    #[arg(long)]
    target: Option<String>,

    /// Score the result (implied when --target is specified)
    #[arg(long)]
    score: bool,

    /// Task tag (may use more than once)
    #[arg(short, long = "tag", value_name = "TAG")]
    tags: Vec<String>,

    /// Path to find tasks
    #[arg(short, long)]
    path: Option<String>,

    /// Inspect log dir
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Don't prompt for input
    ///
    /// --input is required when this option is used.
    #[arg(short, long)]
    yes: bool,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    if args.yes && args.input.is_none() {
        return Err(Error::general("--input required with --yes"));
    }
    apply_profile_with_secrets(config)?;
    handle_dialog_result(run_dialog(args))
}

fn run_dialog(args: Args) -> Result<DialogResult> {
    cli::intro(style("Run task").bold())?;
    py::init();
    Python::attach(|py| {
        let task = select_task_dialog(py, args.task.as_deref(), args.path.as_deref(), args.yes)?;
        let task_doc = get_task_doc(py, &task)?;

        // Task summary
        if let Some(summary) = task_summary(&task, task_doc.as_ref()) {
            cli::log::remark(summary)?;
        }

        // Input
        let mut input = args.input;
        if let Some(val) = input.as_deref() {
            cli::log::step(format!(
                "Input:\n{}",
                val.split("\n").map(|s| style(s).dim()).join("\n")
            ))?;
        } else {
            assert!(
                !args.yes,
                "--yes without --input should be handled by caller"
            );
            input.replace(
                cli::Input::new("Input:")
                    .multiline()
                    .placeholder(&input_placeholder(task_doc.as_ref()))
                    .interact()?,
            );
        }
        let input = input.expect("input from arg or cli::Input");

        // Target
        let mut target = args.target;
        if let Some(val) = target.as_ref() {
            cli::log::step(format!("Target:\n{}", style(val).dim()))?;
        } else if !args.yes {
            let t: String = cli::Input::new("Target (optional):")
                .placeholder(&target_placeholder(task_doc.as_ref()))
                .required(false)
                .default_input("None ") // Hack to show None on empty value
                .interact()?;
            // If user doesn't specify a target, value is "None " (see default_input above)
            if &t != "None " {
                target.replace(t);
            }
        } else {
            cli::log::step(format!("Target:\n{}", style("None").dim()))?;
        }

        // Model
        let model = select_model_dialog(args.model.as_deref(), args.yes)?;

        // Additional options (display only)
        let mut options = Vec::new();
        if args.score && target.is_none() {
            options.push("Output will be scored".into());
        }
        if !args.task_args.is_empty() {
            options.push(format!("Task args: {}", args.task_args.iter().join(", ")));
        }
        if !args.tags.is_empty() {
            options.push(format!("Tags: {}", args.tags.iter().join(", ")));
        }
        if !options.is_empty() {
            cli::log::step(format!(
                "Additional options:\n{}",
                options.iter().map(|s| style(s).dim()).join("\n")
            ))?;
        }

        // Confirm before running unless --yes
        if !args.yes
            && !cli::confirm(format!(
                "You are about to run {}. Continue?",
                style(&task.name).cyan().bright(),
            ))
            .initial_value(true)
            .interact()?
        {
            return Err(Error::Canceled);
        }

        // Log dir
        let log_dir = resolve_log_dir(args.log_dir.as_ref());
        log::debug!("Using log_dir {log_dir:?} for run");

        // Start spinner
        let pb = cli::spinner();
        pb.start(format!("Running {}", task.name));

        // Call run_task function
        let result = run_task(
            py,
            task.get_full_name(),
            input,
            args.task_args,
            model,
            target.or_else(|| args.score.then_some("".into())),
            Some(log_dir.expect_string()),
            args.tags,
        );

        // Ensure spinner is stopped after call
        pb.clear();

        // Show result
        match result {
            Ok(result) => {
                let log = result.log();
                match result.log().status {
                    EvalStatus::Success => {
                        let result = result.sample();

                        // Output
                        if result.output.choices.is_empty() {
                            cli::log::error(
                                "ERROR: model returned an empty response - \
                                 does the solver call generate()?",
                            )?;
                        } else {
                            let output = result.output.to_dialog_info();
                            cli::log::info(format!("Output:\n\n{output}"))?;
                        }

                        // Score
                        if let Some(scores) = result.scores.as_ref()
                            && !scores.is_empty()
                        {
                            let mut scores = scores.values().map(|score| score.to_dialog_info());
                            cli::log::remark(format!("Score:\n\n{}", scores.join("\n\n")))?;
                        }

                        Ok(DialogResult::Done)
                    }
                    EvalStatus::Error => {
                        let error = log.error.as_ref().expect("error for status");
                        cli::log::error(wrap_map(&error.message, term_width() - 4, |s| {
                            style(s).red().bright().to_string()
                        }))?;
                        Err(Error::Quiet)
                    }
                    _ => {
                        cli::log::error(format!("Unexpected log status: {}", log.status))?;
                        Err(Error::Quiet)
                    }
                }
            }
            Err(err) => match &err {
                // Special handling for Inspect formatted error messages
                Error::Py(py_err) => {
                    let msg = py_err.to_string();

                    // Missing Python package
                    if let Some(captures) = DEP_ERROR_P.captures(&msg) {
                        let dep = captures.get(1).unwrap().as_str();
                        let pkg = captures.get(2).unwrap().as_str();
                        let msg = format!(
                            "Missing required package for {dep}\n\
                            \n\
                            Task model requires the {pkg} Python package. Install \
                            it by running 'uv pip install {pkg}'."
                        );
                        Err(Error::general(wrap(&msg, term_width())))

                    // Client init error
                    } else if let Some(captures) = CLIENT_INIT_ERROR_P.captures(&msg) {
                        let client = captures.get(1).unwrap().as_str();
                        let missing_env = CLIENT_ENV_P
                            .captures_iter(&msg)
                            .map(|c| c.get(1).unwrap().as_str())
                            .collect_vec()
                            .join(", ");
                        let msg = format!(
                            "Error initializing {client}\n\
                            \n\
                            Missing one of: {missing_env}"
                        );
                        Err(Error::general(wrap(&msg, term_width())))

                    // Anything else pass through
                    } else {
                        Err(err)
                    }
                }
                _ => Err(err),
            },
        }
    })
}

fn task_summary(task: &TaskInfo, task_doc: Option<&Docstring>) -> Option<String> {
    task_doc
        .and_then(|doc| doc.short_description.clone())
        .or_else(|| task.get_description())
        .map(|val| {
            let width = min(term_width() - 4, 72);
            format!(
                "Description:\n{}",
                style(wrap_map(&val, width, |s| if !s.is_empty() {
                    style(s).dim().to_string()
                } else {
                    "".into()
                }))
            )
        })
}

fn input_placeholder(task_doc: Option<&Docstring>) -> String {
    task_doc
        .map(|doc| {
            if doc.params.len() == 1
                && (doc.params[0]
                    .type_name
                    .as_ref()
                    .map(|s| s == "str")
                    .unwrap_or(true))
            {
                let param = &doc.params[0];
                if let Some(description) = param.description.as_ref() {
                    description.clone()
                } else {
                    param.arg_name.clone()
                }
            } else {
                doc.params
                    .iter()
                    .map(|p| {
                        if let Some(description) = p.description.as_ref() {
                            format!("{}: {}", p.arg_name, description)
                        } else {
                            p.arg_name.clone()
                        }
                    })
                    .join("\n")
            }
        })
        .unwrap_or_default()
}

fn target_placeholder(task_doc: Option<&Docstring>) -> String {
    task_doc
        .map(|doc| {
            doc.returns
                .as_ref()
                .and_then(|v| v.description.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

impl DialogInfo for ModelOutput {
    fn to_dialog_info(&self) -> String {
        let mut blocks = Vec::new();
        for choice in self.choices.iter() {
            blocks.push(choice.message.to_dialog_info())
        }
        blocks.join("\n\n")
    }
}

impl DialogInfo for ChatMessageAssistant {
    fn to_dialog_info(&self) -> String {
        match &self.base.content {
            ChatMessageContent::String(s) => wrapped_output(s),
            ChatMessageContent::ContentList(l) => l.iter().map(|v| v.to_dialog_info()).join("\n\n"),
        }
    }
}

impl DialogInfo for Content {
    fn to_dialog_info(&self) -> String {
        match self {
            Self::Text(v) => wrapped_output(&v.text),
            Self::Reasoning(v) => {
                let mut block = vec![style("Reasoning").dim().to_string()];
                if !v.reasoning.is_empty() {
                    block.push(format!(
                        "reasoning: {}",
                        if v.redacted {
                            "[REDACTED]"
                        } else {
                            &v.reasoning
                        }
                    ));
                }
                if let Some(summary) = v.summary.as_ref()
                    && !summary.is_empty()
                {
                    block.push(format!("summary: {summary}"));
                }
                if let Some(signature) = v.signature.as_ref()
                    && !signature.is_empty()
                {
                    block.push(format!("signature: {signature}"));
                }
                block.join("\n")
            }
            Self::Image(v) => {
                let mut block = vec![style("Image").dim().to_string()];
                block.push(format!("image: {}", v.image));
                block.push(format!("detail: {}", v.detail));
                block.join("\n")
            }
            Self::Audio(v) => {
                let mut block = vec![style("Audio").dim().to_string()];
                block.push(format!("audio: {}", v.audio));
                block.push(format!("format: {}", v.format));
                block.join("\n")
            }
            Self::Video(v) => {
                let mut block = vec![style("Video").dim().to_string()];
                block.push(format!("video: {}", v.video));
                block.push(format!("format: {}", v.format));
                block.join("\n")
            }
            Self::Data(v) => {
                let mut block = vec![style("Data").dim().to_string()];
                for (name, val) in v.data.iter() {
                    block.push(format!("{name}: {val}"));
                }
                block.join("\n")
            }
            Self::ToolUse(v) => {
                let mut block = vec![style("Tool use").dim().to_string()];
                block.push(format!("tool_type: {}", v.tool_type));
                block.push(format!("id: {}", v.id));
                block.push(format!("name: {}", v.name));
                if let Some(context) = v.context.as_ref() {
                    block.push(format!("context: {context}"));
                }
                block.push(format!("arguments: {}", v.arguments));
                block.push(format!("result: {}", v.result));
                if let Some(error) = v.error.as_ref() {
                    block.push(format!("error: {error}"));
                }
                block.join("\n")
            }
            Self::Document(v) => {
                let mut block = vec![style("Document").dim().to_string()];
                block.push(format!("document: {}", v.document));
                block.push(format!("filename: {}", v.filename));
                block.push(format!("mime_type: {}", v.mime_type));
                block.join("\n")
            }
        }
    }
}

fn wrapped_output(s: &str) -> String {
    textwrap::wrap(s, term_width() - 4)
        .iter()
        .map(|s| style(s).yellow())
        .join("\n")
}

impl DialogInfo for Score {
    fn to_dialog_info(&self) -> String {
        let mut parts = Vec::new();

        // Value
        let value = self.value.to_string();
        parts.push(if &value == "C" {
            style("Correct").cyan().to_string()
        } else if &value == "I" {
            style("Incorrect").red().to_string()
        } else {
            style(value).dim().to_string()
        });

        // Explanation
        if let Some(explanation) = self.explanation.as_ref() {
            parts.push(
                textwrap::wrap(explanation, term_width() - 4)
                    .iter()
                    .map(|line| style(line).dim().italic())
                    .join("\n"),
            );
        }

        parts.iter().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::commands::task::run::{CLIENT_ENV_P, CLIENT_INIT_ERROR_P, DEP_ERROR_P};

    #[test]
    fn test_inspect_dep_error() {
        let msg = "inspect_ai._util.error.PrerequisiteError: [bold]ERROR[/bold]: \
                   OpenAI API requires optional dependencies. Install with:\n\
                   \n\
                   [bold]pip install openai[/bold]";
        let captures = DEP_ERROR_P.captures(msg).unwrap();
        assert_eq!("OpenAI API", captures.get(1).unwrap().as_str());
        assert_eq!("openai", captures.get(2).unwrap().as_str());

        let msg = "inspect_ai._util.error.PrerequisiteError: [bold]ERROR[/bold]: \
                   Hugging Face Datasets requires optional dependencies. Install \
                   with:\n\n
                   \n\
                   [bold]pip install datasets[/bold]";
        let captures = DEP_ERROR_P.captures(msg).unwrap();
        assert_eq!("Hugging Face Datasets", captures.get(1).unwrap().as_str());
        assert_eq!("datasets", captures.get(2).unwrap().as_str());
    }

    #[test]
    fn test_inspect_client_init_error() {
        let msg = "inspect_ai._util.error.PrerequisiteError: ERROR: Unable to \
                   initialise OpenAI client\n\
                   \n\
                   No [bold][blue]OPENAI_API_KEY[/blue][/bold], \
                   [bold][blue]AZUREAI_OPENAI_API_KEY[/blue][/bold], or \
                   [bold][blue]or managed identity (Entra ID)[/blue][/bold] \
                   defined in the environment.";
        let captures = CLIENT_INIT_ERROR_P.captures(msg).unwrap();
        assert_eq!("OpenAI", captures.get(1).unwrap().as_str());

        let captures = CLIENT_ENV_P.captures_iter(&msg).collect_vec();
        assert_eq!(3, captures.len());
        assert_eq!(
            "OPENAI_API_KEY",
            captures.get(0).unwrap().get(1).unwrap().as_str()
        );
        assert_eq!(
            "AZUREAI_OPENAI_API_KEY",
            captures.get(1).unwrap().get(1).unwrap().as_str()
        );
        assert_eq!(
            "or managed identity (Entra ID)",
            captures.get(2).unwrap().get(1).unwrap().as_str()
        );
    }
}
