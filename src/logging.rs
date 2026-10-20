use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

const LOG_FILE_NAME: &str = "yasal_debug.log";
const MAX_LOG_BYTES: u64 = 512 * 1024;

/// Directory for the debug log. `YASAL_LOG_DIR` overrides the default
/// (`%LOCALAPPDATA%\Yasal`), which the idle CPU test uses to isolate its run.
fn log_path() -> PathBuf {
    let dir = std::env::var_os("YASAL_LOG_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::data_local_dir().map(|dir| dir.join("Yasal")))
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir.join(LOG_FILE_NAME)
}

/// Appends a timestamped line to the debug log. The file restarts once it grows
/// past 512 KB so it can never grow without bound. Failures are ignored on purpose.
pub fn log(msg: &str) {
    let path = log_path();
    let too_large = fs::metadata(&path).map_or(false, |meta| meta.len() > MAX_LOG_BYTES);

    let mut options = OpenOptions::new();
    options.create(true);
    if too_large {
        options.write(true).truncate(true);
    } else {
        options.append(true);
    }

    if let Ok(mut file) = options.open(&path) {
        let _ = writeln!(
            file,
            "[{}] {}",
            chrono::Local::now().format("%H:%M:%S%.3f"),
            msg
        );
    }
}
