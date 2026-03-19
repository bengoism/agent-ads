use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};

use thiserror::Error;

use crate::error::MetaAdsError;

pub type GoogleResult<T> = std::result::Result<T, GoogleError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleApiError {
    pub message: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub code: Option<i64>,
    #[serde(default)]
    pub details: Option<Value>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(skip)]
    pub http_status: Option<u16>,
}

impl GoogleApiError {
    pub fn retryable(&self) -> bool {
        matches!(self.http_status, Some(429 | 500 | 502 | 503 | 504))
            || matches!(
                self.status.as_deref(),
                Some("RESOURCE_EXHAUSTED" | "INTERNAL" | "UNAVAILABLE")
            )
    }
}

impl Display for GoogleApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.status.as_deref(), self.code) {
            (Some(status), Some(code)) => write!(f, "{} ({status}, code {code})", self.message),
            (Some(status), None) => write!(f, "{} ({status})", self.message),
            (None, Some(code)) => write!(f, "{} (code {code})", self.message),
            (None, None) => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for GoogleApiError {}

#[derive(Debug, Error)]
pub enum GoogleError {
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
    #[error("google api error: {0}")]
    Api(#[from] GoogleApiError),
}

impl GoogleError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) | Self::InvalidArgument(_) => 2,
            Self::Api(_) => 5,
            Self::Http(_) | Self::Io(_) | Self::Json(_) | Self::Csv(_) => 1,
        }
    }
}

impl From<MetaAdsError> for GoogleError {
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
struct GoogleApiErrorEnvelope {
    error: GoogleApiErrorBody,
}

#[derive(Debug, Deserialize)]
struct GoogleApiErrorBody {
    #[serde(default)]
    code: Option<i64>,
    message: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    details: Option<Value>,
}

pub(crate) fn parse_google_api_error(
    status_code: u16,
    request_id: Option<String>,
    body: &str,
) -> GoogleApiError {
    match serde_json::from_str::<GoogleApiErrorEnvelope>(body) {
        Ok(envelope) => GoogleApiError {
            message: envelope.error.message,
            status: envelope.error.status,
            code: envelope.error.code,
            details: envelope.error.details,
            request_id,
            http_status: Some(status_code),
        },
        Err(_) => GoogleApiError {
            message: body.trim().to_string(),
            status: None,
            code: Some(status_code as i64),
            details: None,
            request_id,
            http_status: Some(status_code),
        },
    }
}
