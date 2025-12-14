use std::{
    fs::{self, DirEntry},
    path::Path,
};

use crate::{error::Error, result::Result, util::EpochMillis};

#[derive(Debug)]
pub struct EvalLogInfo {
    #[allow(dead_code)]
    pub name: String,
    pub mtime: Option<EpochMillis>,
    pub task: String,
    pub log_id: String,
    pub is_deleted: bool,
}

#[allow(dead_code)]
pub fn list_logs(log_dir: &Path) -> Result<Vec<EvalLogInfo>> {
    list_logs_filter(log_dir, |log| !log.is_deleted)
}

pub fn list_logs_filter<F>(log_dir: &Path, filter: F) -> Result<Vec<EvalLogInfo>>
where
    F: Fn(&EvalLogInfo) -> bool,
{
    if let Ok(false) = std::fs::exists(log_dir) {
        return Ok(Vec::new());
    }

    // Get filtered logs
    let mut logs = fs::read_dir(log_dir)?
        .filter_map(|f| f.ok().and_then(|f| EvalLogInfo::try_from(f).ok()))
        .filter(filter)
        .collect::<Vec<_>>();

    // Sort by mtime descending (latest logs first)
    logs.sort_by(|lhs, rhs| lhs.mtime.cmp(&rhs.mtime).reverse());

    Ok(logs)
}

impl TryFrom<DirEntry> for EvalLogInfo {
    type Error = Error;

    fn try_from(value: DirEntry) -> std::result::Result<Self, Self::Error> {
        let file_name = value.file_name();
        let path = value.path();
        let LogNameParts {
            timestamp,
            task,
            log_id,
        } = split_log_file_name(file_name.to_str().unwrap())
            .ok_or_else(|| Error::general(format!("not a log: {}", path.to_string_lossy())))?;
        let name = format!("file://{}", path.to_string_lossy());
        let mtime = EpochMillis::from_file_name_timestamp(timestamp);
        let is_deleted = name.ends_with(".deleted");
        Ok(Self {
            name,
            mtime,
            task: task.into(),
            log_id: log_id.into(),
            is_deleted,
        })
    }
}

struct LogNameParts<'a> {
    timestamp: &'a str,
    task: &'a str,
    log_id: &'a str,
}

impl EpochMillis {
    /// Converts Inspect log file timestamp to ISO.
    ///
    /// Inspect saves logs using Python's ISO format but with colon ":"
    /// replaced with "-".
    ///
    /// Inspect file format:
    ///
    /// 2025-12-12T23-26-11+00-00
    ///
    /// Python ISO format:
    ///
    /// 2025-12-12T23:26:11+00:00
    fn from_file_name_timestamp(timestamp: &str) -> Option<Self> {
        let (date, rest) = timestamp.split_at_checked(10)?;
        let time = rest.replace("-", ":");
        Self::from_python_iso(&format!("{date}{time}")).ok()
    }
}

/// Splits an eval log filename.
///
/// Sample file name:
///
/// 2025-12-12T23-26-11+00-00_runnable_EKC6UcVinoTinVFWxrie7N.eval
///
fn split_log_file_name<'a>(file_name: &'a str) -> Option<LogNameParts<'a>> {
    // Timestamp is ISO 8601 (Python generated) at pos 1
    let (timestamp, rest) = file_name.split_at_checked(25)?;

    // Task is delimited with "_" and goes through last "_"
    let (delim, rest) = rest.split_at_checked(1)?;
    if delim != "_" {
        return None;
    }
    let (task, rest) = rest.split_at(rest.rfind("_")?);

    // Log ID is deliminted by "_" and is 22 chars
    let (delim, rest) = rest.split_at_checked(1)?;
    assert!(delim == "_");
    let (log_id, rest) = rest.split_at_checked(22)?;

    // Only valid remaining file extension (e.g. ".eval") or empty ("")
    if !rest.starts_with(".") && !rest.is_empty() {
        return None;
    }

    Some(LogNameParts {
        timestamp,
        task,
        log_id,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_split_log_file_name() {
        use crate::inspect2::log::split_log_file_name as split;

        // Typical case
        let parts =
            split("2025-12-12T23-26-11+00-00_runnable_EKC6UcVinoTinVFWxrie7N.eval").unwrap();
        assert_eq!(parts.timestamp, "2025-12-12T23-26-11+00-00");
        assert_eq!(parts.task, "runnable");
        assert_eq!(parts.log_id, "EKC6UcVinoTinVFWxrie7N");

        // Task contains "_"
        let parts = split("2025-12-12T23-26-11+00-00_foo_bar_EKC6UcVinoTinVFWxrie7N.eval").unwrap();
        assert_eq!(parts.timestamp, "2025-12-12T23-26-11+00-00");
        assert_eq!(parts.task, "foo_bar");
        assert_eq!(parts.log_id, "EKC6UcVinoTinVFWxrie7N");

        // Different file extension
        let parts = split("2025-12-12T23-26-11+00-00_foo_bar_EKC6UcVinoTinVFWxrie7N.xxx").unwrap();
        assert_eq!(parts.timestamp, "2025-12-12T23-26-11+00-00");
        assert_eq!(parts.task, "foo_bar");
        assert_eq!(parts.log_id, "EKC6UcVinoTinVFWxrie7N");

        // Missing file extension
        let parts = split("2025-12-12T23-26-11+00-00_foo_bar_EKC6UcVinoTinVFWxrie7N").unwrap();
        assert_eq!(parts.timestamp, "2025-12-12T23-26-11+00-00");
        assert_eq!(parts.task, "foo_bar");
        assert_eq!(parts.log_id, "EKC6UcVinoTinVFWxrie7N");

        // Timestamp too short
        assert!(split("").is_none());
        assert!(split("asd").is_none());

        // Timestamp only
        assert!(split("2025-12-12T23-26-11+00-00").is_none());

        // Timestamp and task, no ID
        assert!(split("2025-12-12T23-26-11+00-00_foo").is_none());

        // ID too short
        assert!(split("2025-12-12T23-26-11+00-00_foo_123").is_none());

        // ID too long
        assert!(split("2025-12-12T23-26-11+00-00_foo_12345678901234567890123").is_none());

        // No delim after timestamp
        assert!(split("2025-12-12T23-26-11+00-00foo_EKC6UcVinoTinVFWxrie7N.eval").is_none());

        // No delim after task
        assert!(split("2025-12-12T23-26-11+00-00_fooEKC6UcVinoTinVFWxrie7N.eval").is_none());
    }

    #[test]
    fn test_epoch_millis_from_file_name() {
        use crate::util::EpochMillis;

        // Standard case
        assert_eq!(
            EpochMillis::from_file_name_timestamp("2025-12-12T23-26-11+00-00").unwrap(),
            EpochMillis::from_python_iso("2025-12-12T23:26:11+00:00").unwrap()
        );

        // Also parses already-correct ISO formats
        assert_eq!(
            EpochMillis::from_file_name_timestamp("2025-12-12T23:26:11+00-00").unwrap(),
            EpochMillis::from_python_iso("2025-12-12T23:26:11+00:00").unwrap()
        );

        // Any other format returns None
        assert!(EpochMillis::from_file_name_timestamp("").is_none());
        assert!(EpochMillis::from_file_name_timestamp("foo").is_none());
        assert!(EpochMillis::from_file_name_timestamp("2025-12-12T23_26_11+00-00").is_none());
    }
}
