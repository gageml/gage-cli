use std::io::ErrorKind;

use cliclack as cli;
use console::style;

use crate::{error::Error, result::Result};

pub enum DialogResult {
    Message(String),
    Done,
}

pub fn handle_dialog_result(result: Result<DialogResult>) -> Result<()> {
    match &result {
        Ok(DialogResult::Done) => cli::outro(style("Done").green().bright())?,
        Ok(DialogResult::Message(msg)) => cli::outro(style(msg).green().bright())?,
        Err(Error::Py(err)) => {
            let msg = err.to_string();
            if msg == "SystemExit: 0" {
                // System exit with 0 is success
                return Ok(());
            }
            cli::outro_cancel("Error\n")?;
        }
        Err(Error::IO(err)) => match err.kind() {
            ErrorKind::Interrupted => {
                // Canceled outro already shown, exit quietly
                return Err(Error::Quiet);
            }
            _ => cli::outro_cancel("Error\n")?,
        },
        Err(Error::Custom(_)) => cli::outro_cancel("Error\n")?,
        Err(Error::Quiet) => cli::outro_cancel("Error")?,
        Err(Error::Canceled) => cli::outro_cancel("Canceled")?,
    };
    result.map(|_| ())
}

pub trait DialogInfo {
    fn to_dialog_info(&self) -> String;
}
