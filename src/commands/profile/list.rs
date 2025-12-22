use itertools::Itertools;
use tabled::{
    builder::Builder,
    settings::{
        Color,
        object::{Columns, Object, Rows},
        themes::Colorization,
    },
};

use crate::{config::Config, result::Result, util::TableExt};

pub fn main(config: &Config) -> Result<()> {
    if config.profiles.is_empty() {
        println!("No profiles defined in {}", config.path.to_string_lossy());
        return Ok(());
    }

    let mut table = Builder::default();
    table.push_record(["Name", "Description"]);
    for (name, profile) in config
        .profiles
        .iter()
        .sorted_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs))
    {
        table.push_record([name, profile.description.as_deref().unwrap_or_default()]);
    }
    println!(
        "{}",
        table
            .build()
            .with_col_labels()
            .with_rounded()
            .with_term_fit()
            .with(Colorization::exact(
                [Color::FG_BRIGHT_YELLOW],
                Columns::one(0).intersect(Rows::new(1..)),
            ))
    );
    Ok(())
}
