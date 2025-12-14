use console::style;
use tabled::{Table, builder::Builder, settings::Color};

use crate::{
    inspect2::log::{LogHeader, LogInfo},
    theme::Colors,
    util::{TableExt, fit_path_name},
};

pub fn print_log_table<'a>(logs: impl Iterator<Item = &'a LogInfo>) {
    let (table, errors) = logs_table(logs, true);
    if table.count_rows() == 1 {
        println!("No logs found");
    } else {
        println!("{table}");
    }
    if !errors.is_empty() {
        println!(
            "{}",
            style("One or more logs could not be loaded. See above for details.").on_red()
        )
    }
}

/// Returns a tuple of table and read-error log IDs
pub fn logs_table<'a>(
    logs: impl Iterator<Item = &'a LogInfo>,
    index: bool,
) -> (Table, Vec<String>) {
    let now = chrono::Utc::now();
    let index_offset = if index { 0 } else { 1 };
    let mut colored_cells: Vec<(usize, usize, Color)> = Vec::new();
    let mut table = Builder::new();
    table.push_record([
        "#", "Id", "Task", "Type", "Status", "Model", "Dataset", "Started",
    ]);
    let mut errors = Vec::new();
    for (i, log) in logs.enumerate() {
        match LogHeader::try_from(log) {
            Ok(header) => {
                if let Some(color) = status_color(&header.status.to_string()) {
                    colored_cells.push((i + 1, 4 - index_offset, color));
                }
                table.push_record([
                    (i + 1).to_string(),
                    log.short_log_id().into(),
                    log.task.clone(),
                    header.eval.run_type().unwrap_or_default().into(),
                    header.status.to_string(),
                    fit_path_name(&header.eval.model, 20),
                    fit_path_name(&header.eval.dataset.name.unwrap_or_default(), 20),
                    log.mtime.to_human_since(&now),
                ]);
            }
            Err(err) => {
                errors.push(log.log_id.clone());
                log::error!("Error reading {}: {}", log.log_id, err);
                table.push_record([
                    (i + 1).to_string(),
                    "TODO - short ID".into(),
                    log.task.clone(),
                    "?".into(),
                    "?".into(),
                    "?".into(),
                    "?".into(),
                    "?".into(),
                ]);
            }
        }
    }
    let mut table = table.build();

    // Standard formatting
    table.with_term_fit().with_rounded().with_col_labels();

    // Index, if included is cyan
    if index {
        table.with_col_color(1, Color::FG_BRIGHT_CYAN);
    } else {
        table.remove_col(0);
    }

    // Id is dim
    table.with_col_color(1 - index_offset, Colors::dim());

    // Task is yellow
    table.with_col_color(2 - index_offset, Color::FG_BRIGHT_YELLOW);

    // Type is dim
    table.with_col_color(3 - index_offset, Colors::dim());

    // Date is dim
    table.with_col_color(7 - index_offset, Colors::dim());

    // Apply colored cells (e.g. status)
    for (row, col, color) in colored_cells.into_iter() {
        table.with_cell_color(col, row, color);
    }

    (table, errors)
}

fn status_color(status: &str) -> Option<Color> {
    match status {
        "success" => None,
        "error" => Some(Color::FG_RED),
        _ => Some(Colors::dim()),
    }
}
