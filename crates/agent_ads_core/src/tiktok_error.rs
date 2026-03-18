use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use thiserror::Error;

pub type TikTokResult<T> = std::result::Result<T, TikTokError>;

/// The TikTok Business API response envelope. Every response has this shape:
/// `{ "code": 0, "message": "OK", "request_id": "...", "data": { ... } }`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokApiError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub request_id: Option<String>,
}

impl TikTokApiError {
    /// Rate-limit or transient error codes that should be retried.
    pub fn retryable(&self) -> bool {
        matches!(
            self.code,
            40100  // request too frequent
            | 40132 // request frequency limited
            | 61000 // too many requests (block-level)
            | 50000 // internal error
            | 50001 // internal error
            | 50002 // service timeout
        )
    }

    /// Whether this is an auth error that cannot be retried.
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self.code,
            40102 // access token expired
            | 40104 // empty access token
            | 40105 // invalid access token
        )
    }
}

impl Display for TikTokApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (code {})", self.message, self.code)
    }
}

impl std::error::Error for TikTokApiError {}

#[derive(Debug, Error)]
pub enum TikTokError {
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
    #[error("tiktok api error: {0}")]
    Api(#[from] TikTokApiError),
}

impl TikTokError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) | Self::InvalidArgument(_) => 2,
            Self::Api(_) => 4,
            Self::Http(_) | Self::Io(_) | Self::Json(_) | Self::Csv(_) => 1,
        }
    }
}
