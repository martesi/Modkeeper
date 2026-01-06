use crate::models::error::SError;
use crate::models::task_status::TaskStatus;
use tauri::async_runtime::spawn_blocking;
use tauri::ipc::Channel;
use tokio::task_local;

task_local! {
     static CHANNEL: Channel<TaskStatus>;
}

pub struct TaskContext;

impl TaskContext {
    pub async fn provide<F, R>(channel: Channel<TaskStatus>, f: F) -> Result<R, SError>
    where
        F: FnOnce() -> R + Send + 'static, // F is a standard closure, not a Future
        R: Send + 'static,
    {
        spawn_blocking(move || {
            // USE sync_scope HERE
            // This runs 'f' immediately and returns 'R'
            CHANNEL.sync_scope(channel, f)
        })
        .await
        .map_err(|e| SError::AsyncRuntimeError(e.to_string()))
    }
    pub fn emit(status: TaskStatus) -> Result<(), SError> {
        CHANNEL
            .try_with(|c| {
                c.send(status)
                    .map_err(|e| SError::UpdateStatusError(e.to_string()))
            })
            .unwrap_or_else(|_| Err(SError::ContextUnprovided))
    }
}
