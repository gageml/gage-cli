use std::{env, path::Path};

use clap::Args as ArgsTrait;
use itertools::Itertools;
use tabled::{
    Table,
    builder::Builder,
    settings::{
        Color,
        object::{Columns, Object, Rows},
        themes::Colorization,
    },
};

use crate::{
    config::Config,
    error::Error,
    result::Result,
    theme::Colors,
    util::{TableExt, relpath},
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Show more information
    #[arg(short, long)]
    verbose: bool,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    let profile_name = env::var("GAGE_PROFILE")
        .map_err(|_| Error::custom("GAGE_PROFILE not defined - no profile in use"))?;
    let config_path = &config.path;
    let dotenv = dotenvy::dotenv().ok();
    let status = profile_status(&profile_name, config_path, dotenv.as_deref(), args.verbose);
    println!("{status}",);
    Ok(())
}

pub fn profile_status(
    profile_name: &str,
    config_path: &Path,
    dotenv: Option<&Path>,
    verbose: bool,
) -> Table {
    // Use table to show profile info
    let mut table = Builder::new();
    table.push_record(["Active profile", profile_name]);

    // Position of last non-env var label (used for colors below)
    let mut last_non_env: usize = 0;

    // Get profile settings from config
    match Config::from_file(config_path) {
        Ok(config) => {
            let mut empty = true;
            if let Some(profile) = config.profiles.get(profile_name) {
                if let Some(description) = profile.description.as_deref() {
                    table.push_record(["Description", description]);
                    last_non_env += 1;
                    empty = false;
                }
                if let Some(log_dir) = profile.log_dir.as_deref() {
                    table.push_record(["Log dir", log_dir]);
                    last_non_env += 1;
                    empty = false;
                }
                if let Some(secrets) = profile.secrets.as_deref() {
                    table.push_record(["Secrets", secrets]);
                    last_non_env += 1;
                    empty = false;
                }
                for (name, val) in profile.env.iter().sorted() {
                    table.push_record([name, val]);
                    empty = false;
                }
                if empty {
                    table.push_record(["", ""]);
                }
            } else {
                table.push_record([
                    "Error",
                    &format!(
                        "Profile '{}' not defined in {}",
                        profile_name,
                        config_path.to_string_lossy()
                    ),
                ]);
            }
        }
        Err(err) => {
            table.push_record(["Error", &format!("Error reading config: {err:?}")]);
        }
    }

    if verbose {
        if let Some(dotenv) = dotenv {
            table.push_record([".env", &relpath(dotenv).to_string_lossy()]);
        }
        table.push_record(["Config path", &config_path.to_string_lossy()]);
    }

    let mut table = table.build();
    table
        .with_term_fit()
        .with_col_labels()
        .with_rounded()
        // Profile name is yellow
        .with(Colorization::exact(
            [Color::FG_BRIGHT_YELLOW],
            Columns::one(1).intersect(Rows::one(0)),
        ))
        // Non env labels are dim
        .with(Colorization::exact(
            [Colors::dim()],
            Columns::one(0).intersect(Rows::new(..last_non_env + 1)),
        ))
        // Env vars are cyan
        .with(Colorization::exact(
            [Color::FG_BRIGHT_CYAN],
            Columns::one(0).intersect(Rows::new(last_non_env + 1..)),
        ));
    table
}
