use crate::core::library::Library;
use crate::models::error::SError;
use parking_lot::Mutex;
use std::sync::Arc;

pub fn with_lib_arc_mut<F, R>(handle: Arc<Mutex<Option<Library>>>, f: F) -> Result<R, SError>
where
    F: FnOnce(&mut Library) -> R,
{
    let mut guard = handle.lock();
    let lib = guard.as_mut().ok_or(SError::NoActiveLibrary)?;
    Ok(f(lib))
}

pub fn with_lib_arc<F, R>(handle: Arc<Mutex<Option<Library>>>, f: F) -> Result<R, SError>
where
    F: FnOnce(&Library) -> R,
{
    let guard = handle.lock();
    let lib = guard.as_ref().ok_or(SError::NoActiveLibrary)?;
    Ok(f(lib))
}
