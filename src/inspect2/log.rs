use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::{inspect::log::resolve_log_dir, result::Result};

#[derive(Debug)]
pub struct Log;

pub fn open_log(log_id_prefix: &str, log_dir: Option<&PathBuf>) -> Result<Log> {
    let log_dir = resolve_log_dir(log_dir);
    let matches = fs::read_dir(&log_dir)?
        .filter_map(|f| {
            f.ok()
                .and_then(|f| is_match(&f, log_id_prefix).unwrap_or(false).then_some(f))
        })
        .collect::<Vec<DirEntry>>();
    for f in matches {
        println!("------------ {f:?}");
    }
    Ok(Log)
}

fn is_match(f: &DirEntry, log_id_prefix: &str) -> Result<bool> {
    Ok(f.metadata()?.is_file() && {
        f.file_name()
            .to_str()
            .and_then(|name| {
                (name.ends_with(".eval")
                    && log_id_for_name(name)
                        .map(|log_id| log_id.starts_with(log_id_prefix))
                        .unwrap_or(false))
                .then_some(true)
            })
            .unwrap_or(false)
    })
}

fn log_id_for_name(name: &str) -> Option<&str> {
    (name.len() >= 27).then_some(&name[name.len() - 27..])
}
