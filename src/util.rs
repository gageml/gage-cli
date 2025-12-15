use core::fmt;
use std::{
    borrow::Cow,
    env, io,
    path::{MAIN_SEPARATOR, Path, PathBuf},
};

use chrono::{DateTime, FixedOffset, Local, ParseResult, Utc};
use chrono_humanize::HumanTime;
use tabled::{
    Table,
    settings::{
        Color, Remove, Style, Width,
        object::{Columns, Object, Rows},
        peaker::Priority,
        style::HorizontalLine,
        themes::Colorization,
    },
};
use terminal_size::terminal_size;

use crate::theme::Colors;

pub fn split_path_or_env(path: Option<&str>, env_name: &str) -> Vec<String> {
    if let Some(val) = path {
        env::split_paths(val)
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
    } else if let Ok(env_val) = env::var(env_name) {
        split_path_or_env(Some(&env_val), "")
    } else {
        Vec::new()
    }
}

pub fn term_width() -> usize {
    std::env::var("COLUMNS")
        .and_then(|v| v.parse().map_err(|_| std::env::VarError::NotPresent))
        .unwrap_or_else(|_| terminal_size().map(|(w, _h)| w.0 as usize).unwrap_or(60))
}

pub fn term_height() -> usize {
    terminal_size().map(|(_w, h)| h.0 as usize).unwrap_or(40)
}

pub fn wrap(s: &str, width: usize) -> String {
    wrap_map(s, width, |s| s.to_string())
}

pub fn wrap_map(s: &str, width: usize, f: fn(Cow<'_, str>) -> String) -> String {
    textwrap::wrap(s, width)
        .into_iter()
        .map(f)
        .collect::<Vec<_>>()
        .join("\n")
}

pub trait UnwrapExt<T> {
    fn unwrap_with_msg(self, msg: &str) -> T;
}

impl<T, E> UnwrapExt<T> for Result<T, E>
where
    E: fmt::Debug,
{
    fn unwrap_with_msg(self, msg: &str) -> T {
        self.expect(msg)
    }
}

impl<T> UnwrapExt<T> for Option<T> {
    fn unwrap_with_msg(self, msg: &str) -> T {
        self.expect(msg)
    }
}

pub fn relpath(path: &Path) -> &Path {
    let cwd = env::current_dir().unwrap();
    path.strip_prefix(cwd).unwrap_or(path)
}

pub fn relpath_str(path: &str) -> &str {
    let cwd = env::current_dir().unwrap();
    if let Some(rel) = path.strip_prefix(cwd.to_str().unwrap()) {
        // Stripped prefix leaves leading path sep - strip that as well
        if !rel.is_empty() && rel.chars().next().unwrap() == MAIN_SEPARATOR {
            &rel[1..]
        } else {
            rel
        }
    } else {
        path
    }
}

pub trait TableExt {
    fn with_term_fit(&mut self) -> &mut Self;
    fn with_row_labels(&mut self) -> &mut Self;
    fn with_col_labels(&mut self) -> &mut Self;
    fn with_rounded(&mut self) -> &mut Self;
    fn with_rounded_no_header(&mut self) -> &mut Self;
    fn with_cell_color(&mut self, col: usize, row: usize, color: Color) -> &mut Self;
    fn with_col_color(&mut self, col: usize, color: Color) -> &mut Self;
    fn remove_col(&mut self, col: usize) -> &mut Self;
}

impl TableExt for Table {
    fn with_term_fit(&mut self) -> &mut Self {
        self.with(
            Width::truncate(term_width())
                .suffix("…")
                .priority(Priority::max(true)),
        )
    }

    fn with_col_labels(&mut self) -> &mut Self {
        self.with(Colorization::exact([Colors::dim()], Rows::first()))
    }

    fn with_row_labels(&mut self) -> &mut Self {
        self.with(Colorization::exact([Colors::dim()], Columns::first()))
    }

    fn with_rounded(&mut self) -> &mut Self {
        self.with(Style::rounded())
    }

    fn with_rounded_no_header(&mut self) -> &mut Self {
        self.with(
            Style::empty()
                .left('│')
                .right('│')
                .vertical('│')
                .line_top(HorizontalLine::full('─', '┬', '╭', '╮'))
                .line_bottom(HorizontalLine::full('─', '┴', '╰', '╯')),
        )
    }

    fn with_cell_color(&mut self, col: usize, row: usize, color: Color) -> &mut Self {
        self.with(Colorization::exact(
            [color],
            Columns::one(col).intersect(Rows::one(row)),
        ))
    }

    fn with_col_color(&mut self, col: usize, color: Color) -> &mut Self {
        self.with(Colorization::exact([color], Columns::one(col)))
    }

    fn remove_col(&mut self, col: usize) -> &mut Self {
        self.with(Remove::column(Columns::one(col)))
    }
}

pub fn first_line(s: &str) -> (&str, bool) {
    let mut lines = s.split("\n");
    let line1 = lines.next().unwrap();
    let truncated = lines.next().is_some();
    (line1, truncated)
}

pub fn find_try_parents(name: &str) -> Result<Option<PathBuf>, io::Error> {
    let mut cur_dir = env::current_dir().unwrap();
    loop {
        match cur_dir.join(name).canonicalize() {
            Ok(path) => break Ok(Some(path)),
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => {
                    // Try parent if cur dir isn't a project
                    if !has_project_file(&cur_dir)
                        && let Some(parent) = cur_dir.parent()
                    {
                        cur_dir = parent.to_path_buf();
                    } else {
                        break Ok(None);
                    }
                }
                _ => break Err(e),
            },
        }
    }
}

/// List of project markers
///
/// Used to stop searching for a file in parent dirs.
const PROJECT_FILES: [&str; 3] = ["gage.toml", "pyproject.toml", ".venv"];

fn has_project_file(dir: &Path) -> bool {
    for name in PROJECT_FILES {
        if dir.join(name).exists() {
            return true;
        }
    }
    false
}

pub fn fit_path_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.into()
    } else {
        let (prefix, base) = name.rsplit_once('/').unwrap_or(("", name));
        if base.len() > max_len {
            format!("{}…", base.split_at(max_len - 1).0)
        } else {
            let fit_prefix = max_len.saturating_sub(base.len() + 2);
            if fit_prefix > 0 {
                let trunc_prefix = prefix.split_at(fit_prefix).0;
                format!("{trunc_prefix}…/{base}")
            } else {
                base.into()
            }
        }
    }
}

pub trait PathExt {
    fn expect_string(&self) -> String;
}

impl PathExt for Path {
    fn expect_string(&self) -> String {
        self.to_str().unwrap().to_string()
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct EpochMillis(DateTime<Utc>);

impl EpochMillis {
    pub fn from_epoch_millis(ms: i64) -> Self {
        Self(DateTime::<Utc>::from_timestamp_millis(ms).unwrap())
    }

    pub fn from_python_iso(iso: &str) -> ParseResult<Self> {
        Ok(Self(
            DateTime::<FixedOffset>::parse_from_rfc3339(iso)?.to_utc(),
        ))
    }

    pub fn to_human_since(&self, datetime: &DateTime<Utc>) -> String {
        HumanTime::from(self.0 - datetime).to_string()
    }

    pub fn to_human(&self) -> String {
        self.to_human_since(&Utc::now())
    }

    pub fn to_iso_8601_local(&self) -> String {
        self.0.with_timezone(Local::now().offset()).to_rfc3339()
    }

    #[allow(dead_code)] // Used in py tests
    pub fn as_utc_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fit_dataset_name() {
        use crate::util::fit_path_name;

        assert_eq!(fit_path_name("abc", 3), "abc");
        assert_eq!(fit_path_name("abc", 2), "a…");
        assert_eq!(fit_path_name("abc/def", 1), "…");
        assert_eq!(fit_path_name("abc/def", 2), "d…");
        assert_eq!(fit_path_name("abc/def", 3), "def");
        assert_eq!(fit_path_name("abc/def", 4), "def");
        assert_eq!(fit_path_name("abc/def", 5), "def");
        assert_eq!(fit_path_name("abc/def", 6), "a…/def");
        assert_eq!(fit_path_name("abc/def", 7), "abc/def");
        assert_eq!(fit_path_name("abc/def", 8), "abc/def");
    }

    #[test]
    fn test_epoch_millis_from_python_iso() {
        use crate::util::EpochMillis;

        assert_eq!(
            format!(
                "{:?}",
                EpochMillis::from_python_iso("2025-09-30T12:34:56-05:00")
            ),
            "Ok(EpochMillis(2025-09-30T17:34:56Z))"
        );

        assert_eq!(
            format!(
                "{:?}",
                EpochMillis::from_python_iso("2025-12-12T23:26:11+00:00")
            ),
            "Ok(EpochMillis(2025-12-12T23:26:11Z))"
        );

        // Invalid
        assert_eq!(
            format!(
                "{:?}",
                EpochMillis::from_python_iso("2025-12-12T23-26-11+00-00")
            ),
            "Err(ParseError(Invalid))"
        );
    }
}
