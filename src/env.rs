use std::io;

pub fn init() {
    // Intentionally panicing on unexpected/unhandled errors - we don't
    // have a good path to logging for this function as we don't yet
    // know what are use facade is (e.g. terminal vs tui, etc.)
    match dotenvy::dotenv() {
        Ok(path) => {
            log::debug!("Using env from {}", path.to_string_lossy());
        }
        Err(dotenvy::Error::Io(e)) => match e.kind() {
            io::ErrorKind::NotFound => {}
            _ => panic!("{e:?}"),
        },
        Err(e) => panic!("{e:?}"),
    }
}

pub fn get(name: &str) -> Option<String> {
    std::env::var(name).ok()
}
