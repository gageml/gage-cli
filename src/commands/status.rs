use std::{env, io, path::PathBuf};

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
    config::Config, error::Error, inspect::log::resolve_log_dir, py::py_call, result::Result,
    util::TableExt,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Show more detail
    #[arg(short, long)]
    verbose: bool,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    Python::initialize();

    // Special row types - used for styling
    let mut errors = Vec::new();
    let mut not_found = Vec::new();

    let mut table = Builder::new();

    // Package versions
    table.push_record(["gage version", VERSION]);
    Python::attach(|py| {
        table.push_record(["gage_inspect version", &pkg_version(py, "gage_inspect")]);
        if args.verbose {
            table.push_record(["gage_inspect path", &pkg_path(py, "gage_inspect")]);
        }
        table.push_record(["inspect_ai version", &pkg_version(py, "inspect_ai")]);
        if args.verbose {
            table.push_record(["inspect_ai path", &pkg_path(py, "inspect_ai")]);
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
            table.push_record([
                "Log dir",
                &path.strip_prefix(&cwd).unwrap_or(&path).to_string_lossy(),
            ]);
        }
        Err(Error::IO(e)) if e.kind() == io::ErrorKind::NotFound => {
            let path = PathBuf::from(e.to_string());
            table.push_record([
                "Log dir",
                &path.strip_prefix(&cwd).unwrap_or(&path).to_string_lossy(),
            ]);
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

    // match if let Some(path) = config.as_ref() {
    //     Config::from_file(path)
    // } else {
    //     Config::from_default()
    // } {
    //     Ok(config) => {
    //         let path = PathBuf::from(&config.path);
    //         table.push_record([
    //             "Config",
    //             &path.strip_prefix(&cwd).unwrap_or(&path).to_string_lossy(),
    //         ]);
    //     }
    //     Err(Error::IO(e)) if e.kind() == io::ErrorKind::NotFound => {
    //         table.push_record(["Config", &e.to_string()]);
    //         not_found.push(table.count_records() - 1);
    //     }
    //     Err(e) => {
    //         table.push_record(["Config", &e.to_string()]);
    //         errors.push(table.count_records() - 1);
    //     }
    // };

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

    println!("{table}");

    Ok(())
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
