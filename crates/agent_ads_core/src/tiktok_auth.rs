use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use crate::tiktok_error::{TikTokApiError, TikTokError, TikTokResult};

/// Response shape from TikTok's `/oauth2/access_token/` endpoint.
#[derive(Debug, Deserialize)]
struct TokenResponseEnvelope {
    code: i64,
    message: String,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    data: Option<TokenResponseData>,
}

#[derive(Debug, Deserialize)]
struct TokenResponseData {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub access_token_expire_in: Option<u64>,
    #[serde(default)]
    pub refresh_token_expire_in: Option<u64>,
}

/// Result of a successful token refresh.
#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub access_token_expire_in: Option<u64>,
    pub refresh_token_expire_in: Option<u64>,
}

/// Refresh a TikTok access token using the refresh token flow.
///
/// POST `{api_base_url}/open_api/{api_version}/oauth2/access_token/`
/// Body: `{ "app_id": "...", "secret": "...", "grant_type": "refresh_token", "refresh_token": "..." }`
pub async fn refresh_access_token(
    api_base_url: &str,
    api_version: &str,
    app_id: &str,
    app_secret: &str,
    refresh_token: &str,
) -> TikTokResult<RefreshResult> {
    let url = format!(
        "{}/open_api/{}/oauth2/access_token/",
        api_base_url.trim_end_matches('/'),
        api_version
    );

    let mut body = HashMap::new();
    body.insert("app_id", app_id);
    body.insert("secret", app_secret);
    body.insert("grant_type", "refresh_token");
    body.insert("refresh_token", refresh_token);

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = http.post(&url).json(&body).send().await?;
    let response_body = response.text().await?;
    let envelope: TokenResponseEnvelope = serde_json::from_str(&response_body)?;

    if envelope.code != 0 {
        return Err(TikTokError::Api(TikTokApiError {
            code: envelope.code,
            message: envelope.message,
            request_id: envelope.request_id,
        }));
    }

    let data = envelope.data.ok_or_else(|| {
        TikTokError::Config(
            "TikTok returned success but no data in token refresh response".to_string(),
        )
    })?;

    let access_token = data.access_token.ok_or_else(|| {
        TikTokError::Config(
            "TikTok returned success but no access_token in token refresh response".to_string(),
        )
    })?;

    Ok(RefreshResult {
        access_token,
        refresh_token: data.refresh_token,
        access_token_expire_in: data.access_token_expire_in,
        refresh_token_expire_in: data.refresh_token_expire_in,
    })
}
