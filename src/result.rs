use std::process::{ExitCode, Termination};

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Exit {
    Ok,
    Err(Error),
}

impl Termination for Exit {
    fn report(self) -> std::process::ExitCode {
        match self {
            Self::Ok => ().report(),
            Self::Err(err) => {
                eprint!("{err}");
                ExitCode::FAILURE
            }
        }
    }
}
