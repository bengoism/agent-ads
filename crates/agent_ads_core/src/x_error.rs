use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};

use thiserror::Error;

use crate::error::MetaAdsError;

pub type XResult<T> = std::result::Result<T, XError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XApiError {
    pub message: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub parameter: Option<String>,
    #[serde(default)]
    pub details: Option<Value>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(skip)]
    pub http_status: Option<u16>,
}

impl XApiError {
    pub fn retryable(&self) -> bool {
        matches!(self.http_status, Some(429 | 500 | 502 | 503 | 504))
    }
}

impl Display for XApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (&self.code, &self.parameter) {
            (Some(code), Some(parameter)) => {
                write!(
                    f,
                    "{} (code {}, parameter {})",
                    self.message, code, parameter
                )
            }
            (Some(code), None) => write!(f, "{} (code {})", self.message, code),
            (None, Some(parameter)) => write!(f, "{} (parameter {})", self.message, parameter),
            (None, None) => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for XApiError {}

#[derive(Debug, Error)]
pub enum XError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("x ads api error: {0}")]
    Api(#[from] XApiError),
}

impl XError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) | Self::InvalidArgument(_) => 2,
            Self::Api(_) => 8,
            Self::Http(_) | Self::Io(_) | Self::Json(_) | Self::Csv(_) => 1,
        }
    }
}

impl From<MetaAdsError> for XError {
    fn from(value: MetaAdsError) -> Self {
        match value {
            MetaAdsError::Config(message) => Self::Config(message),
            MetaAdsError::InvalidArgument(message) => Self::InvalidArgument(message),
            MetaAdsError::Http(error) => Self::Http(error),
            MetaAdsError::Io(error) => Self::Io(error),
            MetaAdsError::Json(error) => Self::Json(error),
            MetaAdsError::Csv(error) => Self::Csv(error),
            MetaAdsError::Api(error) => Self::Config(error.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct XApiErrorEnvelope {
    #[serde(default)]
    errors: Vec<XApiErrorBody>,
}

#[derive(Debug, Deserialize)]
struct XApiErrorBody {
    #[serde(default)]
    code: Option<Value>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    parameter: Option<String>,
    #[serde(flatten)]
    extra: Value,
}

pub fn parse_x_api_error(status_code: u16, request_id: Option<String>, body: &str) -> XApiError {
    match serde_json::from_str::<XApiErrorEnvelope>(body) {
        Ok(envelope) if !envelope.errors.is_empty() => {
            let first = &envelope.errors[0];
            XApiError {
                message: first
                    .message
                    .clone()
                    .unwrap_or_else(|| body.trim().to_string()),
                code: first.code.as_ref().map(value_to_string),
                parameter: first.parameter.clone(),
                details: Some(first.extra.clone()),
                request_id,
                http_status: Some(status_code),
            }
        }
        _ => XApiError {
            message: body.trim().to_string(),
            code: Some(status_code.to_string()),
            parameter: None,
            details: None,
            request_id,
            http_status: Some(status_code),
        },
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(string) => string.clone(),
        other => other.to_string(),
    }
}
