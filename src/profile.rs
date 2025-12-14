use std::path::PathBuf;

use crate::{
    config::{Config, Profile},
    env,
    result::Result,
    secrets::Secrets,
};

pub fn apply_profile(config: &Config) -> Result<()> {
    apply_profile_impl(config, false)
}

pub fn apply_profile_with_secrets(config: &Config) -> Result<()> {
    apply_profile_impl(config, true)
}

fn apply_profile_impl(config: &Config, with_secrets: bool) -> Result<()> {
    if let Some(profile_name) = env::get("GAGE_PROFILE") {
        // Resolve profile in config
        let empty = Profile::default();
        let profile = config.profiles.get(&profile_name).unwrap_or(&empty);

        // Secrets
        let secrets = if with_secrets && let Some(path) = profile.secrets.as_deref() {
            let secrets_dir = config
                .path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| "".into());
            Some(Secrets::from_file(&secrets_dir.as_path().join(path))?)
        } else {
            None
        };

        // Log dir
        if let Some(log_dir) = profile.log_dir.as_deref()
            && std::env::var("INSPECT_LOG_DIR").is_err()
        {
            let log_dir = config
                .path
                .parent()
                .unwrap()
                .join(log_dir)
                .to_str()
                .unwrap()
                .to_string();
            log::debug!("INSPECT_LOG_DIR={log_dir}");
            unsafe { std::env::set_var("INSPECT_LOG_DIR", log_dir) };
        }

        // Env
        for (name, val) in &profile.env {
            if std::env::var(name).is_err() {
                log::debug!("Setting profile env {name}={val}");
                let val = secrets.as_ref().map(|s| s.apply(val)).unwrap_or(val.into());
                unsafe { std::env::set_var(name, val) };
            } else {
                log::debug!("Skipping profile env {name} (already set)")
            }
        }
    }
    Ok(())
}
