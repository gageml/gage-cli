use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
};

use clap::Args as ArgsTrait;

use crate::{
    commands::profile::status::profile_status, config::Config, error::Error, result::Result,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Profile to apply
    profile: String,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    let profile_name = &args.profile;
    let profile = config
        .profiles
        .get(profile_name)
        .ok_or_else(|| no_such_profile(profile_name, &config.path))?;
    log::debug!("Using {profile_name}: {profile:?}");

    // Read existing .env in config dir
    let config_parent = &config.path.parent().unwrap();
    let dotenv = config_parent.join(".env");
    let lines = if dotenv.exists() {
        let mut buf = io::BufReader::new(File::open(&dotenv)?);
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            let read = buf.read_line(&mut line)?;
            if read == 0 {
                break;
            }
            lines.push(line);
        }
        lines
    } else {
        Vec::new()
    };

    // Write new .env with GAGE_PROFILE set to new profile
    let mut file = File::create(&dotenv)?;
    let mut wrote_profile = false;
    let eol = detect_eol(&lines).unwrap_or_else(|| "\n".into());
    let gage_profile_line = format!("GAGE_PROFILE={profile_name}{eol}");
    for line in lines {
        if line.starts_with("GAGE_PROFILE=") {
            file.write_all(gage_profile_line.as_bytes())?;
            wrote_profile = true;
        } else {
            file.write_all(line.as_bytes())?;
        }
    }
    if !wrote_profile {
        file.write_all(gage_profile_line.as_bytes())?;
    }
    file.flush()?;

    // Show profile details
    let status = profile_status(profile_name, &config.path, Some(&dotenv), false);
    println!("{status}");

    Ok(())
}

fn detect_eol(lines: &[String]) -> Option<String> {
    lines.iter().next().and_then(|s| {
        if s.ends_with("\r\n") {
            Some("\r\n".into())
        } else if s.ends_with("\n") {
            Some("\n".into())
        } else {
            None
        }
    })
}

fn no_such_profile(profile: &str, config: &Path) -> Error {
    Error::Custom(format!(
        "Profile '{}' not defined in {}\n\n\
         Try 'gage profile list' for a list of profiles.",
        profile,
        config.to_string_lossy()
    ))
}
