use std::{collections::HashMap, env, io, path::PathBuf};

use clap::Args as ArgsTrait;
use pyo3::{Python, types::PyAnyMethods};
use tabled::{
    builder::Builder,
    settings::{
        Color,
        object::{Columns, Object, Rows},
        themes::Colorization,
    },
};

use crate::{
    config::Config,
    error::Error,
    inspect::log::resolve_log_dir,
    py::{self, py_call},
    result::Result,
    util::{TableExt, relpath_str},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Show a status value
    #[arg(long, hide = true)]
    attr: Option<String>,

    /// Show more detail
    #[arg(short, long)]
    verbose: bool,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    // Special row types - used for styling
    let mut errors = Vec::new();
    let mut not_found = Vec::new();

    let mut table = Builder::new();
    let mut attrs = HashMap::<String, String>::new();

    // Package versions
    table.push_record(["gage version", VERSION]);

    py::init();
    Python::attach(|py| {
        // gage_inspect
        table.push_record(["gage_inspect version", &pkg_version(py, "gage_inspect")]);
        if args.verbose {
            table.push_record([
                "gage_inspect path",
                relpath_str(&pkg_path(py, "gage_inspect")),
            ]);
        }

        // inspect_ai
        table.push_record(["inspect_ai version", &pkg_version(py, "inspect_ai")]);
        if args.verbose {
            table.push_record(["inspect_ai path", relpath_str(&pkg_path(py, "inspect_ai"))]);
        }

        // Python version
        let sys = py.import("sys").unwrap();
        table.push_record([
            "Python version",
            sys.getattr("version")
                .unwrap()
                .extract::<String>()
                .unwrap()
                .split(' ')
                .next()
                .unwrap(),
        ]);

        if args.verbose {
            // Python exe
            table.push_record([
                "Python executable",
                relpath_str(
                    &sys.getattr("executable")
                        .unwrap()
                        .extract::<String>()
                        .unwrap(),
                ),
            ]);

            // Python system path
            table.push_record([
                "Python sys path",
                &sys.getattr("path")
                    .unwrap()
                    .extract::<Vec<String>>()
                    .unwrap()
                    .iter()
                    .map(|path| relpath_str(path))
                    .collect::<Vec<&str>>()
                    .join("\n"),
            ]);
        }
    });

    // .env
    match dotenv_path() {
        Ok(Some(path)) => {
            table.push_record([".env", &path]);
        }
        Ok(None) => {
            table.push_record([".env", ""]);
        }
        Err(e) => {
            table.push_record([".env", &e.to_string()]);
            errors.push(table.count_records() - 1);
        }
    }

    // Current dir used to format paths
    let cwd = env::current_dir().unwrap();

    // log dir
    match resolve_log_dir(args.log_dir.as_ref()) {
        Ok(path) => {
            let path = PathBuf::from(path);
            let relpath = path.strip_prefix(&cwd).unwrap_or(&path).to_string_lossy();
            attrs.insert("log_dir".into(), relpath.to_string());
            table.push_record(["Log dir", &relpath]);
        }
        Err(Error::IO(e)) if e.kind() == io::ErrorKind::NotFound => {
            let path = PathBuf::from(e.to_string());
            let relpath = path.strip_prefix(&cwd).unwrap_or(&path).to_string_lossy();
            attrs.insert("log_dir".into(), relpath.to_string());
            table.push_record(["Log dir", &relpath]);
            not_found.push(table.count_records() - 1);
        }
        Err(e) => {
            table.push_record(["Log dir", &e.to_string()]);
            errors.push(table.count_records() - 1);
        }
    };

    // Config
    let path = PathBuf::from(&config.path);
    table.push_record([
        "Config",
        &path.strip_prefix(&cwd).unwrap_or(&path).to_string_lossy(),
    ]);
    if !path.exists() {
        not_found.push(table.count_records() - 1);
    }

    // Active profile
    table.push_record([
        "Active profile",
        &env::var("GAGE_PROFILE").unwrap_or_default(),
    ]);

    let mut table = table.build();
    table
        .with_term_fit()
        .with_row_labels()
        .with_rounded_no_header();

    // Error colors
    for row in errors {
        table.with(Colorization::exact(
            [Color::FG_BRIGHT_RED],
            Columns::one(1).intersect(Rows::one(row)),
        ));
    }

    // Not found colors
    for row in not_found {
        table.with(Colorization::exact(
            [Color::FG_BRIGHT_BLACK],
            Columns::one(1).intersect(Rows::one(row)),
        ));
    }

    if let Some(attr) = args.attr {
        if let Some(val) = attrs.get(&attr) {
            println!("{val}");
            Ok(())
        } else {
            Err(Error::general(format!("Unknown attr '{attr}'")))
        }
    } else {
        println!("{table}");
        Ok(())
    }
}

pub fn pkg_version<'py>(py: Python<'py>, pkg: &str) -> String {
    py_call(py, "gage_inspect._util", "pkg_version", (pkg,))
        .map(|bound| bound.extract::<String>().unwrap())
        .unwrap_or_else(|e| e.to_string())
}

pub fn pkg_path<'py>(py: Python<'py>, pkg: &str) -> String {
    py_call(py, "gage_inspect._util", "pkg_path", (pkg,))
        .map(|bound| bound.extract::<String>().unwrap())
        .unwrap_or_else(|e| e.to_string())
}

fn dotenv_path() -> Result<Option<String>> {
    match dotenvy::dotenv() {
        Ok(path) => {
            let cwd = std::env::current_dir().unwrap();
            Ok(Some(
                path.strip_prefix(cwd)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string(),
            ))
        }
        Err(dotenvy::Error::Io(e)) => match e.kind() {
            io::ErrorKind::NotFound => Ok(None),
            _ => Err(e.to_string().into()),
        },
        Err(e) => Err(e.to_string().into()),
    }
}
