use derive_more::Display;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Type, Serialize, Deserialize, Debug, Display)]
pub enum SError {
    UnsupportedSPTVersion(String),
    ParseError(String),
    IOError(String),
    GameOrServerRunning,
    ProcessRunning,
    UnableToDetermineModId,
    #[display("Mod not found: {}", _0)]
    ModNotFound(String),
    FileOrDirectoryNotFound(String),
    #[display("File collisions detected: {}", "_0.join(\", \")")]
    FileCollision(Vec<String>),
    Unexpected,
    UnhandledCompression(String),
    AsyncRuntimeError(String),
    ContextUnprovided,
    UpdateStatusError(String),
    NoActiveLibrary,
}

macro_rules! impl_from {
    ($from_type:ty, $variant:ident) => {
        impl From<$from_type> for SError {
            fn from(err: $from_type) -> Self {
                // We convert the foreign error to a String immediately
                // so we can serialize it later.
                SError::$variant(err.to_string())
            }
        }
    };
}

impl_from!(semver::Error, ParseError);
impl_from!(std::io::Error, IOError);
impl_from!(serde_json::Error, ParseError);
impl_from!(std::path::StripPrefixError, ParseError);
impl_from!(zip::result::ZipError, UnhandledCompression);
