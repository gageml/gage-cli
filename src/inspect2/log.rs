use std::{
    fs::{self, DirEntry, File},
    path::Path,
};

use rawzip::{RECOMMENDED_BUFFER_SIZE, ZipArchive};
use serde::{Deserialize, Serialize};

use crate::{error::Error, result::Result, util::EpochMillis};

#[derive(Serialize, Debug)]
pub struct LogInfo {
    #[serde(skip_serializing)]
    pub name: String,
    pub mtime: EpochMillis,
    pub task: String,
    pub log_id: String,
    #[serde(skip_serializing)]
    pub is_deleted: bool,
}

impl LogInfo {
    pub fn expect_local_path(&self) -> &str {
        if !self.name.starts_with("file://") {
            panic!("Non-local log: {}", self.name);
        }
        self.name.split_at(7).1
    }

    pub fn short_log_id(&self) -> &str {
        self.log_id
            .split_at_checked(6)
            .map(|(short_id, _)| short_id)
            .unwrap_or(&self.log_id)
    }
}

#[derive(Deserialize)]
pub struct LogHeader {
    pub status: EvalStatus,
    pub eval: EvalSpec,
    pub results: Option<EvalResults>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvalStatus {
    Started,
    Success,
    Cancelled,
    Error,
}

impl std::fmt::Display for EvalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Started => f.write_str("started"),
            Self::Success => f.write_str("success"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Error => f.write_str("error"),
        }
    }
}

#[derive(Deserialize)]
pub struct EvalSpec {
    // pub eval_set_id: Option<String>,
    // pub eval_id: String,
    // pub run_id: String,
    // pub created: EpochMillis,
    // pub task: String,
    // pub task_id: String,
    // pub task_version: TaskVersion,
    // pub task_file: Option<String>,
    // pub task_display_name: Option<String>,
    // pub task_registry_name: Option<String>,
    // pub task_attribs: Attributes,
    // pub task_args: Args,
    // pub task_args_passed: Args,
    // pub solver: Option<String>,
    // pub solver_args: Option<Args>,
    pub tags: Option<Vec<String>>,
    pub dataset: EvalDataset,
    // sandbox: SandboxEnvironmentSpec | None = Field(default=None)
    pub model: String,
    // model_generate_config: GenerateConfig = Field(default_factory=GenerateConfig)
    // pub model_base_url: Option<String>,
    // pub model_args: Args,
    // model_roles: dict[str, ModelConfig] | None = Field(default=None)
    // config: EvalConfig
    // pub revision: Option<EvalRevision>,
    // pub packages: HashMap<String, String>,
    // pub metadata: Option<Metadata>,
    // pub scorers: Option<Vec<EvalScorer>>,
    // pub metrics: Option<Metrics>,
    // model_config = ConfigDict(protected_namespaces=())
}

impl EvalSpec {
    // pub fn task_description(&self) -> Option<String> {
    //     self.task_attribs
    //         .get("description")
    //         .map(|val| val.to_string())
    // }

    pub fn run_type(&self) -> Option<&str> {
        self.tags
            .as_ref()?
            .iter()
            .find_map(|v| v.strip_prefix("type:"))
    }
}

#[derive(Deserialize)]
pub struct EvalDataset {
    pub name: Option<String>,
    // pub location: Option<String>,
    // pub samples: Option<usize>,
    // pub sample_ids: Option<Vec<SampleId>>,
    // pub shuffled: Option<bool>,
}

#[derive(Deserialize)]
pub struct EvalResults {
    pub total_samples: usize,
    pub completed_samples: usize,
}

impl TryFrom<&LogInfo> for LogHeader {
    type Error = Error;

    fn try_from(value: &LogInfo) -> std::result::Result<Self, Self::Error> {
        if !value.name.starts_with("file://") {
            panic!("{}", value.name);
        }
        let path = &value.name[7..];
        let file = File::open(path)?;

        // TODO - move to general zip facility

        let mut buffer = vec![0u8; RECOMMENDED_BUFFER_SIZE];
        let archive = ZipArchive::from_file(file, &mut buffer)?;

        let header_name_bytes = "header.json".as_bytes();
        let mut entries = archive.entries(&mut buffer);
        while let Some(entry) = entries.next_entry()? {
            if entry.is_dir() {
                continue;
            }
            if entry.file_path().as_bytes() == header_name_bytes {
                let zip_entry = archive.get_entry(entry.wayfinder())?;
                let reader = zip_entry.reader();
                match entry.compression_method() {
                    rawzip::CompressionMethod::Store => {
                        let mut verifier = zip_entry.verifying_reader(reader);
                        return Ok(serde_json::from_reader(&mut verifier)?);
                    }
                    rawzip::CompressionMethod::Deflate => {
                        let inflater = flate2::read::DeflateDecoder::new(reader);
                        let mut verifier = zip_entry.verifying_reader(inflater);
                        return Ok(serde_json::from_reader(&mut verifier)?);
                    }
                    other => {
                        return Err(Error::custom(format!(
                            "unexpected compression method '{other:?}' for header.json in {path}"
                        )));
                    }
                }
            }
        }

        Err(Error::custom(format!("missing header.json in {path}")))
    }
}

#[allow(dead_code)]
pub fn list_logs(log_dir: &Path) -> Result<Vec<LogInfo>> {
    list_logs_filter(log_dir, |log| !log.is_deleted)
}

pub fn list_logs_filter<F>(log_dir: &Path, filter: F) -> Result<Vec<LogInfo>>
where
    F: Fn(&LogInfo) -> bool,
{
    if let Ok(false) = std::fs::exists(log_dir) {
        return Ok(Vec::new());
    }

    // Get filtered logs
    let mut logs = fs::read_dir(log_dir)?
        .filter_map(|f| f.ok().and_then(|f| LogInfo::try_from(f).ok()))
        .filter(filter)
        .collect::<Vec<_>>();

    // Sort using name, which contains a leading create timestamp,
    // showing most recently created logs first
    logs.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name).reverse());

    Ok(logs)
}

impl TryFrom<DirEntry> for LogInfo {
    type Error = Error;

    fn try_from(value: DirEntry) -> std::result::Result<Self, Self::Error> {
        let file_name = value.file_name();
        let path = value.path();
        let LogNameParts {
            timestamp,
            task,
            log_id,
        } = split_log_file_name(file_name.to_str().unwrap())
            .ok_or_else(|| Error::custom(format!("not a log: {}", path.to_string_lossy())))?;
        let name = format!("file://{}", path.to_string_lossy());
        let mtime = match EpochMillis::try_from_file_name_timestamp(timestamp) {
            Ok(v) => v,
            Err(e) => {
                log::debug!("Error getting mtime for log {path:?}: {e}");
                EpochMillis::default()
            }
        };
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
    fn try_from_file_name_timestamp(timestamp: &str) -> Result<Self> {
        // Split at pos 10 for date and rest
        let (date, rest) = timestamp
            .split_at_checked(10)
            .ok_or_else(|| Error::custom("missing date"))?;

        // Expect time part to start with "T"
        let (delim, rest) = rest
            .split_at_checked(1)
            .ok_or_else(|| Error::custom("missing time delimiter at pos 11"))?;
        if delim != "T" {
            return Err(Error::custom("unexpected char at pos 11, expected T"));
        }

        // Split rest at pos 8 for time and rest
        let (time, rest) = rest
            .split_at_checked(8)
            .ok_or_else(|| Error::custom("missing time"))?;

        // Split rest at pos 1 for timezone sign direction and value
        let (tz_sign, tz_offset) = rest
            .split_at_checked(1)
            .ok_or_else(|| Error::custom("missing tz"))?;
        if tz_sign != "-" && tz_sign != "+" {
            return Err(Error::custom("unexpected tz sign at pos 20"));
        }

        // Reconstruct to ISO by relacing "-" in time and tz offset with ":"
        let iso_fmt = format!(
            "{}T{}{}{}",
            date,
            time.replace("-", ":"),
            tz_sign,
            tz_offset.replace("-", ":")
        );
        Self::from_python_iso(&iso_fmt)
            .map_err(|e| Error::custom(format!("invalid ISO date '{iso_fmt}': {e}")))
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
            EpochMillis::try_from_file_name_timestamp("2025-12-12T23-26-11+00-00").unwrap(),
            EpochMillis::from_python_iso("2025-12-12T23:26:11+00:00").unwrap()
        );

        // Also parses already-correct ISO formats
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("2025-12-12T23:26:11-00-00").unwrap(),
            EpochMillis::from_python_iso("2025-12-12T23:26:11-00:00").unwrap()
        );

        // Missing date (too short)
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("")
                .unwrap_err()
                .to_string(),
            "missing date"
        );
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("123456789")
                .unwrap_err()
                .to_string(),
            "missing date"
        );

        // Bad time delimiter
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("1234567890")
                .unwrap_err()
                .to_string(),
            "missing time delimiter at pos 11"
        );
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("12345678901")
                .unwrap_err()
                .to_string(),
            "unexpected char at pos 11, expected T"
        );

        // Missing time
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("1234567890T")
                .unwrap_err()
                .to_string(),
            "missing time"
        );
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("1234567890T1234567")
                .unwrap_err()
                .to_string(),
            "missing time"
        );

        // Missing or invalid timezone
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("1234567890T12345678")
                .unwrap_err()
                .to_string(),
            "missing tz"
        );
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("1234567890T123456789")
                .unwrap_err()
                .to_string(),
            "unexpected tz sign at pos 20"
        );

        // Remaining errors are specific to ISO parse
        assert_eq!(
            EpochMillis::try_from_file_name_timestamp("1234567890T12345678+")
                .unwrap_err()
                .to_string(),
            "invalid ISO date '1234567890T12345678+': input contains invalid characters"
        );
    }
}
