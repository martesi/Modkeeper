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