use std::{collections::HashMap, path::Path};

use crate::{error::Error, result::Result};

pub struct Secrets(HashMap<String, String>);

impl Secrets {
    pub fn from_file(path: &Path) -> Result<Self> {
        let output = std::process::Command::new("sops")
            .args(["-d", &path.to_string_lossy()])
            .output()
            // TODO handle command not found with a helpful message for
            // installing sops
            .map_err(|e| Error::custom(format!("Error running sops command: {e:?}")))?;
        if !output.status.success() {
            return Err(Error::custom(format!(
                "Error decrypting secrets in {}: {:?}",
                path.to_string_lossy(),
                output
            )));
        }
        Ok(Self(serde_json::from_slice(&output.stdout).map_err(
            |e| Error::custom(format!("Error parsing secrets: {e:?}")),
        )?))
    }

    pub fn apply(&self, val: &str) -> String {
        let mut applied = val.to_string();
        for (name, val) in &self.0 {
            applied = applied.replace(&format!("{{{name}}}"), val);
        }
        applied
    }
}
