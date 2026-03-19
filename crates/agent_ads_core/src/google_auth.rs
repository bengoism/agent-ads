use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use crate::google_error::{GoogleError, GoogleResult};

const GOOGLE_OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Deserialize)]
struct RefreshTokenResponse {
    access_token: Option<String>,
    expires_in: Option<u64>,
    token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthErrorResponse {
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub access_token: String,
    pub expires_in: Option<u64>,
    pub token_type: Option<String>,
}

pub async fn refresh_access_token(
    timeout_seconds: u64,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> GoogleResult<RefreshResult> {
    refresh_access_token_with_url(
        GOOGLE_OAUTH_TOKEN_URL,
        timeout_seconds,
        client_id,
        client_secret,
        refresh_token,
    )
    .await
}

pub(crate) async fn refresh_access_token_with_url(
    token_url: &str,
    timeout_seconds: u64,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> GoogleResult<RefreshResult> {
    let mut form = HashMap::new();
    form.insert("client_id", client_id);
    form.insert("client_secret", client_secret);
    form.insert("refresh_token", refresh_token);
    form.insert("grant_type", "refresh_token");

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build()?;

    let response = http.post(token_url).form(&form).send().await?;
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        let oauth_error = serde_json::from_str::<OAuthErrorResponse>(&body)
            .ok()
            .and_then(|error| match (error.error, error.error_description) {
                (Some(kind), Some(description)) => Some(format!("{kind}: {description}")),
                (Some(kind), None) => Some(kind),
                (None, Some(description)) => Some(description),
                (None, None) => None,
            })
            .unwrap_or_else(|| body.trim().to_string());
        return Err(GoogleError::Config(format!(
            "failed to refresh Google OAuth access token: {oauth_error}"
        )));
    }

    let payload: RefreshTokenResponse = serde_json::from_str(&body)?;
    let access_token = payload.access_token.ok_or_else(|| {
        GoogleError::Config(
            "Google OAuth returned success but no access_token in the refresh response".to_string(),
        )
    })?;

    Ok(RefreshResult {
        access_token,
        expires_in: payload.expires_in,
        token_type: payload.token_type,
    })
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::refresh_access_token_with_url;

    #[tokio::test]
    async fn refreshes_google_access_token() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .and(body_string_contains("grant_type=refresh_token"))
            .and(body_string_contains("client_id=test-client"))
            .and(body_string_contains("client_secret=test-secret"))
            .and(body_string_contains("refresh_token=test-refresh"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"access_token":"ya29.test","expires_in":3600,"token_type":"Bearer"}"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let result = refresh_access_token_with_url(
            &format!("{}/token", server.uri()),
            10,
            "test-client",
            "test-secret",
            "test-refresh",
        )
        .await
        .unwrap();

        assert_eq!(result.access_token, "ya29.test");
        assert_eq!(result.expires_in, Some(3600));
        assert_eq!(result.token_type.as_deref(), Some("Bearer"));
    }
}
