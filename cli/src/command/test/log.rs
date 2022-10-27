use gag::BufferRedirect;
use std::io::Read;

use crate::Result;

/// Captures stdout of a function and returns it with the result of the function.
pub fn capture<F, T>(log: &mut String, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    // REVIEW: what will happen if the function panics or crashes

    // TODO: do not capture logs if verbose logging is enabled
    let mut buffer = BufferRedirect::stdout()?;
    let result = f()?;
    buffer.read_to_string(log)?;

    Ok(result)
}
