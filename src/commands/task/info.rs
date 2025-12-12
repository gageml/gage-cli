use clap::Args as ArgsTrait;
use pyo3::Python;
use tabled::{
    builder::Builder,
    settings::{Padding, Style, Width, object::Columns, peaker::Priority, themes::Colorization},
};

use crate::{
    commands::task::select_task, inspect::task::get_task_doc, py, result::Result, theme::Colors,
    util::term_width,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Task name
    task: String,

    /// Path to find tasks
    #[arg(short, long)]
    path: Option<String>,
}

pub fn main(args: Args) -> Result<()> {
    py::init();
    Python::attach(|py| {
        let task = select_task(py, Some(&args.task), args.path.as_deref())?;

        let mut table = Builder::new();

        // Task name and source
        table.push_record(["Name", &task.name]);
        table.push_record(["Source", &task.file]);

        // Try to get task doc
        let doc = get_task_doc(py, &task)?;

        // Task description
        table.push_record([
            "Description",
            &doc.as_ref()
                // Use full doc description if available
                .and_then(|doc| doc.description.clone())
                // Otherwise use description from task info
                .or_else(|| task.get_description())
                .unwrap_or_default(),
        ]);

        // Task doc attributes
        if let Some(doc) = doc {
            // Input params
            for (i, param) in doc.params.iter().enumerate() {
                if i == 0 {
                    table.push_record(["", ""]);
                }
                let param_fmt = format!(
                    "{} {}",
                    param.arg_name,
                    param.description.as_deref().unwrap_or_default()
                );
                table.push_record([if i == 0 { "Input" } else { "" }, &param_fmt]);
            }

            // Output
            if let Some(output) = doc.returns {
                table.push_record(["", ""]);
                table.push_record(["Output", &output.description.unwrap_or_default()]);
            }
        }

        println!(
            "{}",
            table
                .build()
                .with(Style::empty())
                .with(Padding::zero())
                .modify(Columns::first(), Padding::new(0, 2, 0, 0))
                .with(
                    Width::wrap(term_width())
                        .keep_words(true)
                        .priority(Priority::max(true)),
                )
                .with(Colorization::exact([Colors::dim()], Columns::first()))
        );
        Ok(())
    })
}
