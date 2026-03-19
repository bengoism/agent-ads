use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use thiserror::Error;

pub type PinterestResult<T> = std::result::Result<T, PinterestError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinterestApiError {
    pub code: i64,
    pub message: String,
    #[serde(skip)]
    pub http_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl PinterestApiError {
    pub fn retryable(&self) -> bool {
        matches!(self.http_status, Some(429 | 500 | 502 | 503 | 504))
    }
}

impl Display for PinterestApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.http_status {
            Some(status) => write!(f, "{} (code {}, http {})", self.message, self.code, status),
            None => write!(f, "{} (code {})", self.message, self.code),
        }
    }
}

impl std::error::Error for PinterestApiError {}

#[derive(Debug, Error)]
pub enum PinterestError {
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
    #[error("pinterest api error: {0}")]
    Api(#[from] PinterestApiError),
}

impl PinterestError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) | Self::InvalidArgument(_) => 2,
            Self::Api(_) => 6,
            Self::Http(_) | Self::Io(_) | Self::Json(_) | Self::Csv(_) => 1,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PinterestApiErrorBody {
    code: Option<i64>,
    message: Option<String>,
}

pub fn parse_pinterest_api_error(
    status_code: u16,
    request_id: Option<String>,
    body: &str,
) -> PinterestApiError {
    match serde_json::from_str::<PinterestApiErrorBody>(body) {
        Ok(parsed) => PinterestApiError {
            code: parsed.code.unwrap_or(status_code as i64),
            message: parsed.message.unwrap_or_else(|| body.trim().to_string()),
            http_status: Some(status_code),
            request_id,
        },
        Err(_) => PinterestApiError {
            code: status_code as i64,
            message: body.trim().to_string(),
            http_status: Some(status_code),
            request_id,
        },
    }
}
