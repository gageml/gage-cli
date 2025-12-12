use clap::Args as ArgsTrait;
use pyo3::Python;
use tabled::{
    builder::Builder,
    settings::{
        Color,
        object::{Columns, Object, Rows},
        themes::Colorization,
    },
};

use crate::{
    commands::dataset::list_datasets_dialog, py, result::Result, theme::Colors, util::TableExt,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Path to find datasets
    #[arg(short, long)]
    path: Option<String>,
}

pub fn main(args: Args) -> Result<()> {
    py::init();
    Python::attach(|py| {
        let datasets = list_datasets_dialog(py, args.path.as_deref())?;
        if datasets.is_empty() {
            println!("No datasets found");
            return Ok(());
        }
        let mut table = Builder::default();
        table.push_record(["Dataset", "Description", "Source"]);
        for task in datasets {
            table.push_record([
                &task.name,
                &task.get_description().unwrap_or_default(),
                &task.file,
            ]);
        }
        println!(
            "{}",
            table
                .build()
                .with_rounded()
                .with_col_labels()
                .with_term_fit()
                .with(Colorization::exact(
                    [Color::FG_BRIGHT_YELLOW],
                    Columns::new(..1).intersect(Rows::new(1..))
                ))
                .with(Colorization::exact(
                    [Colors::dim()],
                    Columns::one(2).intersect(Rows::new(1..)),
                ))
        );
        Ok(())
    })
}
