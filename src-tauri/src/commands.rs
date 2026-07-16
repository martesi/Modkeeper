pub mod global;
pub mod library;

use crate::models::error::SError;

/// Logs the English error line at the point a command returns Err - the one
/// place Display crosses into the developer trail; the frontend receives only
/// code/data (consolidated-spec.md §7g/§13).
pub(crate) fn log_err<T>(result: Result<T, SError>) -> Result<T, SError> {
    result.inspect_err(|e| tracing::error!("{e}"))
}
