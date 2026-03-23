use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};

use thiserror::Error;

use crate::error::MetaAdsError;

pub type LinkedInResult<T> = std::result::Result<T, LinkedInError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedInApiError {
    pub message: String,
    #[serde(default, rename = "serviceErrorCode")]
    pub service_error_code: Option<i64>,
    #[serde(default)]
    pub status: Option<i64>,
    #[serde(default)]
    pub details: Option<Value>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(skip)]
    pub http_status: Option<u16>,
}

impl LinkedInApiError {
    pub fn retryable(&self) -> bool {
        matches!(self.http_status, Some(429 | 500 | 502 | 503 | 504))
    }
}

impl Display for LinkedInApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.service_error_code, self.status) {
            (Some(code), Some(status)) => {
                write!(f, "{} (code {}, status {})", self.message, code, status)
            }
            (Some(code), None) => write!(f, "{} (code {})", self.message, code),
            (None, Some(status)) => write!(f, "{} (status {})", self.message, status),
            (None, None) => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for LinkedInApiError {}

#[derive(Debug, Error)]
pub enum LinkedInError {
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
    #[error("linkedin api error: {0}")]
    Api(#[from] LinkedInApiError),
}

impl LinkedInError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) | Self::InvalidArgument(_) => 2,
            Self::Api(_) => 7,
            Self::Http(_) | Self::Io(_) | Self::Json(_) | Self::Csv(_) => 1,
        }
    }
}

impl From<MetaAdsError> for LinkedInError {
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
struct LinkedInApiErrorBody {
    #[serde(default)]
    message: Option<String>,
    #[serde(default, rename = "serviceErrorCode")]
    service_error_code: Option<i64>,
    #[serde(default)]
    status: Option<i64>,
    #[serde(default)]
    details: Option<Value>,
}

pub fn parse_linkedin_api_error(
    status_code: u16,
    request_id: Option<String>,
    body: &str,
) -> LinkedInApiError {
    match serde_json::from_str::<LinkedInApiErrorBody>(body) {
        Ok(parsed) => LinkedInApiError {
            message: parsed.message.unwrap_or_else(|| body.trim().to_string()),
            service_error_code: parsed.service_error_code,
            status: parsed.status,
            details: parsed.details,
            request_id,
            http_status: Some(status_code),
        },
        Err(_) => LinkedInApiError {
            message: body.trim().to_string(),
            service_error_code: Some(status_code as i64),
            status: Some(status_code as i64),
            details: None,
            request_id,
            http_status: Some(status_code),
        },
    }
}
