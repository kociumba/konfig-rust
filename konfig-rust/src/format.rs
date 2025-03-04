use std::any::Any;
use crate::KonfigError;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::str;

pub enum Format {
    JSON,
    YAML,
    TOML,
}

impl Format {
    pub fn create_handler(&self) -> FormatHandler {
        match self {
            Format::JSON => FormatHandler::Builtin(BuiltinFormat::JSON),
            Format::YAML => FormatHandler::Builtin(BuiltinFormat::YAML),
            Format::TOML => FormatHandler::Builtin(BuiltinFormat::TOML),
        }
    }
}

/// A generic trait for format handlers, implement to create a custom format
pub trait ConfigFormat {
    /// Uses the serde `Serialize` trait to serialize data to bytes in the specified format
    fn marshal<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, KonfigError>;

    /// Uses the serde `DeserializeOwned` trait to deserialize data from bytes in the specified format
    fn unmarshal<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, KonfigError>;
}

// I love how this just duplicates the Format enum xd
pub enum BuiltinFormat {
    JSON,
    YAML,
    TOML,
}

/// Where all formats go to marshal
pub enum FormatHandler {
    Builtin(BuiltinFormat),
    Custom(Box<dyn ConfigFormat>),
}

impl FormatHandler {
    pub fn marshal<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, KonfigError> {
        match self {
            FormatHandler::Builtin(BuiltinFormat::JSON) => {
                serde_json::to_vec(data).map_err(|err| KonfigError::MarshalError(err.to_string()))
            }

            FormatHandler::Builtin(BuiltinFormat::YAML) => serde_yaml::to_string(data)
                .map_err(|err| KonfigError::MarshalError(err.to_string()))
                .map(|s| s.into_bytes()),

            FormatHandler::Builtin(BuiltinFormat::TOML) => toml::to_string(data)
                .map_err(|err| KonfigError::MarshalError(err.to_string()))
                .map(|s| s.into_bytes()),

            FormatHandler::Custom(custom) => custom.marshal(data),
        }
    }

    pub fn unmarshal<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, KonfigError> {
        match self {
            FormatHandler::Builtin(BuiltinFormat::JSON) => serde_json::from_slice(data)
                .map_err(|err| KonfigError::UnmarshalError(err.to_string())),

            FormatHandler::Builtin(BuiltinFormat::YAML) => serde_yaml::from_slice(data)
                .map_err(|err| KonfigError::UnmarshalError(err.to_string())),

            FormatHandler::Builtin(BuiltinFormat::TOML) => toml::from_str(
                str::from_utf8(data)
                    .map_err(|err| KonfigError::UnmarshalError(err.to_string()))?,
            )
            .map_err(|err| KonfigError::UnmarshalError(err.to_string())),

            FormatHandler::Custom(custom) => custom.marshal(data).and_then(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|err| KonfigError::UnmarshalError(err.to_string()))
            }),
        }
    }
}
