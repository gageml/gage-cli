use std::sync::{LazyLock, Mutex};

static MODE: LazyLock<Mutex<Mode>> = LazyLock::new(|| Mutex::new(Mode::Term));

use std::io::{self, Write};

use console::{StyledObject, style};
use env_logger::fmt::Formatter;
use log::{Level, LevelFilter, Record};

use crate::util::UnwrapExt;

struct Logger {
    term: env_logger::Logger,
    cursive: cursive::logger::CursiveLogger,
}

enum Mode {
    Term,
    Cursive,
}

pub fn init(debug: bool) {
    // Terminal logger
    let term = env_logger::Builder::from_env(
        env_logger::Env::new().filter_or("RUST_LOG", if debug { "debug" } else { "info" }),
    )
    .format(term_format)
    .build();

    // Use log level from terminal config
    let level = term.filter();

    // Cursive logger
    let cursive = cursive::logger::CursiveLogger;
    cursive::logger::set_external_filter_level(level);
    // Cursive internal logs are chatty - always keep at INTO
    cursive::logger::set_internal_filter_level(LevelFilter::Info);

    // Set log level
    log::set_max_level(level);

    // Set logger (one time only)
    log::set_boxed_logger(Box::new(Logger { term, cursive })).unwrap_with_msg(
        "log already set - cannot be set more than once for the life of the program",
    );
}

pub fn use_cursive() {
    *MODE.lock().unwrap() = Mode::Cursive;
}

pub fn use_default() {
    *MODE.lock().unwrap() = Mode::Term;
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.term.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        match *MODE.lock().unwrap() {
            Mode::Term => self.term.log(record),
            Mode::Cursive => self.cursive.log(record),
        }
    }

    fn flush(&self) {
        match *MODE.lock().unwrap() {
            Mode::Term => self.term.flush(),
            Mode::Cursive => self.cursive.flush(),
        }
    }
}

fn term_format(buf: &mut Formatter, record: &Record<'_>) -> io::Result<()> {
    let ts = buf.timestamp_millis();
    writeln!(
        buf,
        "{} {:5} {} {}",
        style(ts).dim(),
        colorize_level(record.level()),
        style(format!("{}:", record.target())).dim(),
        record.args()
    )
}

fn colorize_level(l: Level) -> StyledObject<Level> {
    match l {
        Level::Warn => style(l).red(),
        Level::Error => style(l).red().bright(),
        l => style(l),
    }
}
