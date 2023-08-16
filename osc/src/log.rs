use std::io::Write;

use env_logger::{Env, Builder};

const FILTER_ENV: &str = "OSC_LOG";
const WRITE_STYLE_ENV: &str = "OSC_LOG";

/// Initializes the logger to provide standard formatting for the [`log`] crate using [`env_logger`].
///
/// Format:
// [<timestamp>] [<short file>:<line number>] [<log level>] <message>
///
/// TODO: examples
///
/// WARNING: this function should only be called *once* as early in the program execution as possible.
pub fn init_logger() {
    let env = Env::default()
        .filter(FILTER_ENV)
        .write_style(WRITE_STYLE_ENV);

    Builder::from_env(env)
        .format(|buf, record| {
            let timestamp = buf.timestamp();
            let file = record.module_path().unwrap_or("unknown");
            let line_number = record.line().unwrap_or(0);
            let level = record.level();

            writeln!(
                buf,
                "[{}] [{}:{}] [{}] {}",
                timestamp,
                file,
                line_number,
                level,
                record.args()
            )
        })
        .init();
}
