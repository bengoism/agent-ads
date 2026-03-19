use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use crate::pinterest_error::{PinterestError, PinterestResult};

pub const PINTEREST_OAUTH_TOKEN_URL: &str = "https://api.pinterest.com/v5/oauth/token";

#[derive(Debug, Deserialize)]
struct OAuthSuccessResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<u64>,
    scope: Option<String>,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<u64>,
    refresh_token_expires_at: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OAuthErrorResponse {
    code: Option<i64>,
    message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_in: Option<u64>,
    pub refresh_token_expires_at: Option<u64>,
}

pub async fn refresh_access_token(
    timeout_seconds: u64,
    app_id: &str,
    app_secret: &str,
    refresh_token: &str,
) -> PinterestResult<RefreshResult> {
    refresh_access_token_with_url(
        PINTEREST_OAUTH_TOKEN_URL,
        timeout_seconds,
        app_id,
        app_secret,
        refresh_token,
    )
    .await
}

pub(crate) async fn refresh_access_token_with_url(
    token_url: &str,
    timeout_seconds: u64,
    app_id: &str,
    app_secret: &str,
    refresh_token: &str,
) -> PinterestResult<RefreshResult> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build()?;

    let mut form = HashMap::new();
    form.insert("grant_type", "refresh_token");
    form.insert("refresh_token", refresh_token);
    // Pinterest documents this for legacy apps; sending it consistently keeps
    // refresh behavior on the continuous-refresh path.
    form.insert("continuous_refresh", "true");

    let response = http
        .post(token_url)
        .basic_auth(app_id, Some(app_secret))
        .form(&form)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        let oauth_error = serde_json::from_str::<OAuthErrorResponse>(&body)
            .ok()
            .map(|parsed| match (parsed.code, parsed.message) {
                (Some(code), Some(message)) => format!("{message} (code {code})"),
                (Some(code), None) => format!("code {code}"),
                (None, Some(message)) => message,
                (None, None) => body.trim().to_string(),
            })
            .unwrap_or_else(|| body.trim().to_string());
        return Err(PinterestError::Config(format!(
            "failed to refresh Pinterest OAuth access token: {oauth_error}"
        )));
    }

    let payload: OAuthSuccessResponse = serde_json::from_str(&body)?;
    let access_token = payload.access_token.ok_or_else(|| {
        PinterestError::Config(
            "Pinterest OAuth returned success but no access_token in the refresh response"
                .to_string(),
        )
    })?;

    Ok(RefreshResult {
        access_token,
        token_type: payload.token_type,
        expires_in: payload.expires_in,
        scope: payload.scope,
        refresh_token: payload.refresh_token,
        refresh_token_expires_in: payload.refresh_token_expires_in,
        refresh_token_expires_at: payload.refresh_token_expires_at,
    })
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{basic_auth, body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::refresh_access_token_with_url;

    #[tokio::test]
    async fn refreshes_pinterest_access_token() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(basic_auth("test-app-id", "test-app-secret"))
            .and(body_string_contains("grant_type=refresh_token"))
            .and(body_string_contains("refresh_token=test-refresh"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
                    "response_type":"refresh_token",
                    "access_token":"pin.test",
                    "token_type":"bearer",
                    "expires_in":3600,
                    "scope":"ads:read",
                    "refresh_token":"refresh.next",
                    "refresh_token_expires_in":7200,
                    "refresh_token_expires_at":1735689600
                }"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let result = refresh_access_token_with_url(
            &format!("{}/oauth/token", server.uri()),
            10,
            "test-app-id",
            "test-app-secret",
            "test-refresh",
        )
        .await
        .unwrap();

        assert_eq!(result.access_token, "pin.test");
        assert_eq!(result.refresh_token.as_deref(), Some("refresh.next"));
        assert_eq!(result.scope.as_deref(), Some("ads:read"));
    }
}
