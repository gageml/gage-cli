use std::fmt::Display;

use chrono::{DateTime, FixedOffset, Local, ParseResult, Utc};
use chrono_humanize::HumanTime;
use pyo3::{
    Borrowed, Bound, FromPyObject, PyAny, PyErr, PyResult, Python,
    call::PyCallArgs,
    exceptions::{PyTypeError, PyValueError},
    types::{PyAnyMethods, PyModule},
};

#[derive(Debug)]
pub enum Any {
    Str(String),
    Int(i64),
    Float(f64),
    Other(String),
}

impl<'a, 'py> FromPyObject<'a, 'py> for Any {
    type Error = PyErr;
    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(s) = ob.extract::<String>() {
            Ok(Self::Str(s))
        } else if let Ok(i) = ob.extract::<i64>() {
            Ok(Self::Int(i))
        } else if let Ok(n) = ob.extract::<f64>() {
            Ok(Self::Float(n))
        } else {
            Ok(Self::Other(ob.to_string()))
        }
    }
}

impl Display for Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => s.fmt(f),
            Self::Int(i) => i.fmt(f),
            Self::Float(n) => n.fmt(f),
            Self::Other(o) => o.fmt(f),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
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
}

impl FromPyObject<'_, '_> for EpochMillis {
    type Error = PyErr;
    fn extract(ob: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        if let Ok(mtime) = ob.extract::<f64>() {
            // Float encoding - treat as epoch millis
            Ok(Self::from_epoch_millis(mtime as i64))
        } else if let Ok(iso) = ob.extract::<String>() {
            // ISO encoded string
            Ok(Self::from_python_iso(&iso).map_err(|_| {
                PyErr::new::<PyValueError, _>(format!("invalid datetime ISO string '{iso}'"))
            })?)
        } else if let Ok(isoformat) = ob.getattr("isoformat") {
            // Object with isoformat function - treat as `datetime.datetime`
            let iso: String = isoformat.call0().unwrap().extract().unwrap();
            Ok(Self::from_python_iso(&iso).unwrap())
        } else {
            Err(PyErr::new::<PyTypeError, _>(format!(
                "{} cannot be converted to EpochMillis",
                *ob
            )))
        }
    }
}

pub fn py_call<'py, A>(
    py: Python<'py>,
    mod_name: &str,
    func_name: &str,
    args: A,
) -> PyResult<Bound<'py, PyAny>>
where
    A: PyCallArgs<'py>,
{
    let module = PyModule::import(py, mod_name)?;
    let func = module.getattr(func_name)?;
    func.call(args, None)
}

#[derive(FromPyObject, Debug)]
pub struct Docstring {
    pub description: Option<String>,
    pub short_description: Option<String>,
    // pub long_description: Option<String>,
    pub params: Vec<DocstringParam>,
    pub returns: Option<DocstringReturns>,
}

#[derive(FromPyObject, Debug)]
pub struct DocstringParam {
    pub arg_name: String,
    pub description: Option<String>,
    pub type_name: Option<String>,
}

#[derive(FromPyObject, Debug)]
pub struct DocstringReturns {
    // pub args: Vec<String>,
    pub description: Option<String>,
    // pub type_name: Option<String>,
}

pub fn init() {
    Python::initialize();
}

#[cfg(test)]
mod tests {
    use pyo3::{Python, ffi::c_str, types::PyAnyMethods};

    use crate::py::EpochMillis;

    #[test]
    fn test_millis_extract() {
        Python::initialize();
        Python::attach(|py| {
            // Parse from UTC millis float
            let f = py.eval(c_str!("1759254091325.5388"), None, None).unwrap();
            let ms = f.extract::<EpochMillis>().unwrap();
            assert_eq!("2025-09-30T17:41:31.325+00:00", ms.0.to_rfc3339());

            // Parse from UTC millis int
            let i = py.eval(c_str!("1759254091456"), None, None).unwrap();
            let ms = i.extract::<EpochMillis>().unwrap();
            assert_eq!("2025-09-30T17:41:31.456+00:00", ms.0.to_rfc3339());

            // Parse from Python datetime ISO (8601)
            let iso = py
                .eval(c_str!("'2025-09-30T12:34:56-05:00'"), None, None)
                .unwrap();
            let ms = iso.extract::<EpochMillis>().unwrap();
            assert_eq!("2025-09-30T17:34:56+00:00", ms.0.to_rfc3339());

            let iso = py
                .eval(c_str!("'2025-11-04T18:03:02.088960-06:00'"), None, None)
                .unwrap();
            let ms = iso.extract::<EpochMillis>().unwrap();
            assert_eq!("2025-11-05T00:03:02.088960+00:00", ms.0.to_rfc3339());

            // Try to parse invalid ISO string
            let invalid_date = py.eval(c_str!("'not-a-date'"), None, None).unwrap();
            let err = invalid_date.extract::<EpochMillis>().unwrap_err();
            assert_eq!(
                "ValueError: invalid datetime ISO string 'not-a-date'",
                err.to_string()
            );

            // Try to parse from unsupported type
            let unsupported = py.eval(c_str!("[]"), None, None).unwrap();
            let err = unsupported.extract::<EpochMillis>().unwrap_err();
            assert_eq!(
                "TypeError: [] cannot be converted to EpochMillis",
                err.to_string()
            );
        });
    }
}
