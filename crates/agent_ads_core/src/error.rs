use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, MetaAdsError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphApiError {
    pub message: String,
    #[serde(default)]
    pub error_type: Option<String>,
    #[serde(default)]
    pub code: Option<i64>,
    #[serde(default)]
    pub error_subcode: Option<i64>,
    #[serde(default)]
    pub fbtrace_id: Option<String>,
    #[serde(default)]
    pub is_transient: Option<bool>,
    #[serde(skip)]
    pub status_code: Option<u16>,
}

impl GraphApiError {
    pub fn retryable(&self) -> bool {
        if self.is_transient.unwrap_or(false) {
            return true;
        }

        matches!(self.code, Some(1 | 2 | 4 | 17 | 32 | 341 | 613))
    }
}

impl Display for GraphApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.code {
            write!(f, "{} (code {code})", self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for GraphApiError {}

#[derive(Debug, Deserialize)]
pub(crate) struct GraphApiErrorEnvelope {
    pub error: GraphApiErrorBody,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GraphApiErrorBody {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<i64>,
    pub error_subcode: Option<i64>,
    pub fbtrace_id: Option<String>,
    pub is_transient: Option<bool>,
}

impl From<GraphApiErrorEnvelope> for GraphApiError {
    fn from(value: GraphApiErrorEnvelope) -> Self {
        Self {
            message: value.error.message,
            error_type: value.error.error_type,
            code: value.error.code,
            error_subcode: value.error.error_subcode,
            fbtrace_id: value.error.fbtrace_id,
            is_transient: value.error.is_transient,
            status_code: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum MetaAdsError {
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
    #[error("meta api error: {0}")]
    Api(#[from] GraphApiError),
}

impl MetaAdsError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) | Self::InvalidArgument(_) => 2,
            Self::Api(_) => 3,
            Self::Http(_) | Self::Io(_) | Self::Json(_) | Self::Csv(_) => 1,
        }
    }
}

pub(crate) fn is_retryable_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}
