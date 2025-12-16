#![allow(dead_code)]

use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{error::Error, result::Result, util::find_try_parents};

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    #[serde(skip)]
    pub path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profiles: Default::default(),
            path: "gage.toml".into(),
        }
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct Profile {
    pub help: Option<String>,
    pub secrets: Option<String>,
    pub log_dir: Option<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl Config {
    pub fn from_arg(arg: Option<&PathBuf>) -> Result<Self> {
        arg.map(|path| Self::from_file(path))
            .unwrap_or_else(Self::from_default_file)
    }

    pub fn try_from_arg(arg: Option<&PathBuf>) -> Result<Option<Self>> {
        match Self::from_arg(arg) {
            Ok(config) => Ok(Some(config)),
            Err(Error::IO(e)) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn from_default_file() -> Result<Self> {
        if let Some(path) = find_try_parents("gage.toml")? {
            Self::from_file(path.as_path())
        } else {
            Err(Error::IO(io::Error::new(
                io::ErrorKind::NotFound,
                "gage.toml",
            )))
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|e| Error::custom(format!("Cannot read {}: {}", path.to_string_lossy(), e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut config: Self =
            toml::from_str(&contents).map_err(|e| Error::custom(e.to_string()))?;
        config.path = path.into();
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn test_config_toml() {
        let config: Config = toml::from_str(
            r#"
            [profiles.foo]
            help = "A sample profile"
            secrets = "secrets.json"
            log_dir = "logs"
            env.bar = "123"
            env.baz = "321"
            "#,
        )
        .unwrap();

        assert_eq!(1, config.profiles.len());

        let foo = &config.profiles["foo"];
        assert_eq!("A sample profile", foo.help.as_ref().unwrap());
        assert_eq!("logs", foo.log_dir.as_ref().unwrap());
        assert_eq!("secrets.json", foo.secrets.as_ref().unwrap());
        assert_eq!("123", foo.env["bar"]);
        assert_eq!("321", foo.env["baz"]);
    }
}
