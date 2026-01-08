use crate::models::error::SError;
use camino::Utf8PathBuf;

pub struct Toml;

impl Toml {
    pub fn write<T: serde::Serialize>(path: &Utf8PathBuf, data: &T) -> Result<(), SError> {
        toml::to_string(data)
            .map_err(|e| SError::ParseError(e.to_string()))
            .and_then(|t| std::fs::write(path, t).map_err(|e| SError::IOError(e.to_string())))
    }

    pub fn read<T: serde::de::DeserializeOwned>(path: &Utf8PathBuf) -> Result<T, SError> {
        let s = std::fs::read_to_string(path).map_err(|e| SError::IOError(e.to_string()))?;
        toml::from_str::<T>(&s).map_err(|e| SError::ParseError(e.to_string()))
    }
}
