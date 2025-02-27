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
    pub fn create_handler(&self) -> FormatHandlerEnum {
        match self {
            Format::JSON => FormatHandlerEnum::JSON(JSONFormat {}),
            Format::YAML => FormatHandlerEnum::YAML(YAMLFormat {}),
            Format::TOML => FormatHandlerEnum::TOML(TOMLFormat {}),
        }
    }
}

// Needed couse rust is a crybaby and won't let me use generic traits
pub enum FormatHandlerEnum {
    JSON(JSONFormat),
    YAML(YAMLFormat),
    TOML(TOMLFormat),
}

impl FormatHandlerEnum {
    pub fn marshal<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, KonfigError> {
        match self {
            FormatHandlerEnum::JSON(handler) => handler.marshal(data),
            FormatHandlerEnum::YAML(handler) => handler.marshal(data),
            FormatHandlerEnum::TOML(handler) => handler.marshal(data),
        }
    }

    pub fn unmarshal<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, KonfigError> {
        match self {
            FormatHandlerEnum::JSON(handler) => handler.unmarshal(data),
            FormatHandlerEnum::YAML(handler) => handler.unmarshal(data),
            FormatHandlerEnum::TOML(handler) => handler.unmarshal(data),
        }
    }
}

pub trait FormatHandler {
    fn marshal<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, KonfigError>;
    fn unmarshal<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, KonfigError>;
}

pub struct JSONFormat {}
impl FormatHandler for JSONFormat {
    fn marshal<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, KonfigError> {
        serde_json::to_vec(data).map_err(|err| KonfigError::MarshalError(err.to_string()))
    }

    fn unmarshal<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, KonfigError> {
        serde_json::from_slice(data).map_err(|err| KonfigError::UnmarshalError(err.to_string()))
    }
}

pub struct YAMLFormat {}
impl FormatHandler for YAMLFormat {
    fn marshal<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, KonfigError> {
        serde_yaml::to_string(data)
            .map_err(|err| KonfigError::MarshalError(err.to_string()))
            .map(|s| s.into_bytes())
    }

    fn unmarshal<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, KonfigError> {
        serde_yaml::from_slice(data).map_err(|err| KonfigError::UnmarshalError(err.to_string()))
    }
}

pub struct TOMLFormat {}
impl FormatHandler for TOMLFormat {
    fn marshal<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, KonfigError> {
        toml::to_string(data)
            .map_err(|err| KonfigError::MarshalError(err.to_string()))
            .map(|s| s.into_bytes())
    }

    fn unmarshal<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, KonfigError> {
        str::from_utf8(data)
            .map_err(|err| KonfigError::UnmarshalError(err.to_string()))
            .and_then(|s| {
                toml::from_str(s).map_err(|err| KonfigError::UnmarshalError(err.to_string()))
            })
    }
}
