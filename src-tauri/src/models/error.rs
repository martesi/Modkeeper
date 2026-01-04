use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Type, Serialize, Deserialize, Debug)]
pub enum SError {
    UnsupportedSPTVersion(String),
    ParseError(String),
    IOError(String),
    GameOrServerRunning,
    UnableToDetermineModId,
    FileOrDirectoryNotFound(String),
    FileCollision(Vec<String>),
    Unexpected(Option<String>),
    Link,
}
