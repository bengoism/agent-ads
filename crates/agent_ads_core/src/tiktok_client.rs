use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;
use tracing::debug;

use crate::tiktok_config::TikTokResolvedConfig;
use crate::tiktok_error::{TikTokApiError, TikTokError, TikTokResult};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokPageInfo {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub total_number: Option<u64>,
    #[serde(default)]
    pub total_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TikTokResponse {
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<TikTokPageInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl TikTokResponse {
    pub fn new(data: Value) -> Self {
        Self {
            data,
            page_info: None,
            request_id: None,
        }
    }
}

/// Raw envelope from the TikTok Business API.
#[derive(Debug, Deserialize)]
struct ApiEnvelope {
    code: i64,
    message: String,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    data: Option<Value>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TikTokClient {
    http: reqwest::Client,
    api_base_url: String,
    api_version: String,
    access_token: String,
    max_retries: usize,
}

impl TikTokClient {
    pub fn from_config(config: &TikTokResolvedConfig) -> TikTokResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            http,
            api_base_url: config.api_base_url.trim_end_matches('/').to_string(),
            api_version: config.api_version.clone(),
            access_token: config.access_token.clone(),
            max_retries: 4,
        })
    }

    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    /// Build the full URL for a TikTok API endpoint path.
    /// Example path: `campaign/get` → `https://business-api.tiktok.com/open_api/v1.3/campaign/get/`
    fn build_url(&self, path: &str) -> String {
        let trimmed = path.trim_matches('/');
        format!(
            "{}/open_api/{}/{}/",
            self.api_base_url, self.api_version, trimmed
        )
    }

    /// Make a GET request to a TikTok API endpoint.
    pub async fn get(
        &self,
        path: &str,
        params: &BTreeMap<String, String>,
    ) -> TikTokResult<TikTokResponse> {
        self.request(reqwest::Method::GET, path, params, None).await
    }

    /// Make a POST request (JSON body) to a TikTok API endpoint.
    pub async fn post(&self, path: &str, body: &Value) -> TikTokResult<TikTokResponse> {
        self.request(reqwest::Method::POST, path, &BTreeMap::new(), Some(body))
            .await
    }

    /// Auto-paginate a GET request that returns `data.list` + `data.page_info`.
    pub async fn get_all(
        &self,
        path: &str,
        params: &BTreeMap<String, String>,
        max_items: Option<usize>,
    ) -> TikTokResult<TikTokResponse> {
        let mut collected: Vec<Value> = Vec::new();
        let mut current_page: u32 = params
            .get("page")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        let mut last_page_info: Option<TikTokPageInfo>;
        let mut last_request_id: Option<String>;

        loop {
            let mut page_params = params.clone();
            page_params.insert("page".to_string(), current_page.to_string());

            let response = self.get(path, &page_params).await?;
            last_request_id = response.request_id.clone();

            // Extract list items from the data
            let items = extract_list_items(&response.data);
            let page_info = extract_page_info(&response.data);

            for item in items {
                if let Some(max) = max_items {
                    if collected.len() >= max {
                        break;
                    }
                }
                collected.push(item);
            }

            let total_page = page_info.as_ref().and_then(|pi| pi.total_page).unwrap_or(1);

            last_page_info = page_info;

            if let Some(max) = max_items {
                if collected.len() >= max {
                    break;
                }
            }

            if current_page >= total_page {
                break;
            }

            current_page += 1;
        }

        Ok(TikTokResponse {
            data: Value::Array(collected),
            page_info: last_page_info,
            request_id: last_request_id,
        })
    }

    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        params: &BTreeMap<String, String>,
        body: Option<&Value>,
    ) -> TikTokResult<TikTokResponse> {
        let url = self.build_url(path);
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let mut request = self
                .http
                .request(method.clone(), &url)
                .header("Access-Token", &self.access_token)
                .query(params);

            if let Some(body) = body {
                request = request.json(body);
            }

            let response = request.send().await;
            match response {
                Ok(response) => {
                    let response_body = response.text().await?;
                    let envelope: ApiEnvelope = serde_json::from_str(&response_body)?;

                    if envelope.code == 0 {
                        // Success
                        let data = envelope.data.unwrap_or(Value::Null);
                        let page_info = extract_page_info(&data);
                        return Ok(TikTokResponse {
                            data,
                            page_info,
                            request_id: envelope.request_id,
                        });
                    }

                    // Partial success
                    if envelope.code == 20001 {
                        let data = envelope.data.unwrap_or(Value::Null);
                        let page_info = extract_page_info(&data);
                        return Ok(TikTokResponse {
                            data,
                            page_info,
                            request_id: envelope.request_id,
                        });
                    }

                    let api_error = TikTokApiError {
                        code: envelope.code,
                        message: envelope.message,
                        request_id: envelope.request_id,
                    };

                    if api_error.retryable() && attempt < self.max_retries {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(
                            attempt,
                            delay_ms,
                            code = api_error.code,
                            "retrying tiktok api request"
                        );
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = Some(TikTokError::Api(api_error));
                        continue;
                    }

                    return Err(TikTokError::Api(api_error));
                }
                Err(error) => {
                    if error.is_timeout() || error.is_connect() {
                        if attempt < self.max_retries {
                            let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                            debug!(attempt, delay_ms, "retrying after transport failure");
                            sleep(Duration::from_millis(delay_ms)).await;
                            last_error = Some(TikTokError::Http(error));
                            continue;
                        }
                    }
                    return Err(TikTokError::Http(error));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TikTokError::Config("request retries exhausted without a captured error".to_string())
        }))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_list_items(data: &Value) -> Vec<Value> {
    // TikTok list endpoints return { "list": [...], "page_info": {...} }
    if let Some(list) = data.get("list").and_then(Value::as_array) {
        list.clone()
    } else if let Some(arr) = data.as_array() {
        arr.clone()
    } else {
        Vec::new()
    }
}

fn extract_page_info(data: &Value) -> Option<TikTokPageInfo> {
    data.get("page_info")
        .and_then(|v| serde_json::from_value::<TikTokPageInfo>(v.clone()).ok())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::TikTokClient;
    use crate::output::OutputFormat;
    use crate::tiktok_config::TikTokResolvedConfig;

    fn test_config(base_url: &str) -> TikTokResolvedConfig {
        TikTokResolvedConfig {
            access_token: "test-token".to_string(),
            api_base_url: base_url.to_string(),
            api_version: "v1.3".to_string(),
            timeout_seconds: 10,
            default_advertiser_id: None,
            output_format: OutputFormat::Json,
            config_path: "agent-ads.config.json".into(),
        }
    }

    #[tokio::test]
    async fn parses_success_response() {
        let server = MockServer::start().await;
        let body = json!({
            "code": 0,
            "message": "OK",
            "request_id": "req-123",
            "data": {
                "list": [{ "campaign_id": "1" }],
                "page_info": { "page": 1, "page_size": 20, "total_number": 1, "total_page": 1 }
            }
        });

        Mock::given(method("GET"))
            .and(path("/open_api/v1.3/campaign/get/"))
            .and(header("Access-Token", "test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let client = TikTokClient::from_config(&test_config(&server.uri())).unwrap();
        let mut params = BTreeMap::new();
        params.insert("advertiser_id".to_string(), "123".to_string());
        let response = client.get("campaign/get", &params).await.unwrap();

        assert_eq!(response.request_id.as_deref(), Some("req-123"));
        assert!(response.data.get("list").is_some());
    }

    #[tokio::test]
    async fn returns_api_error_on_non_zero_code() {
        let server = MockServer::start().await;
        let body = json!({
            "code": 40105,
            "message": "Invalid access token",
            "request_id": "req-err"
        });

        Mock::given(method("GET"))
            .and(path("/open_api/v1.3/advertiser/info/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let client = TikTokClient::from_config(&test_config(&server.uri())).unwrap();
        let result = client.get("advertiser/info", &BTreeMap::new()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 4);
    }

    #[tokio::test]
    async fn auto_paginates() {
        let server = MockServer::start().await;

        // Page 1
        Mock::given(method("GET"))
            .and(path("/open_api/v1.3/campaign/get/"))
            .and(wiremock::matchers::query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0,
                "message": "OK",
                "data": {
                    "list": [{ "campaign_id": "1" }],
                    "page_info": { "page": 1, "page_size": 1, "total_number": 2, "total_page": 2 }
                }
            })))
            .mount(&server)
            .await;

        // Page 2
        Mock::given(method("GET"))
            .and(path("/open_api/v1.3/campaign/get/"))
            .and(wiremock::matchers::query_param("page", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0,
                "message": "OK",
                "data": {
                    "list": [{ "campaign_id": "2" }],
                    "page_info": { "page": 2, "page_size": 1, "total_number": 2, "total_page": 2 }
                }
            })))
            .mount(&server)
            .await;

        let client = TikTokClient::from_config(&test_config(&server.uri())).unwrap();
        let response = client
            .get_all("campaign/get", &BTreeMap::new(), None)
            .await
            .unwrap();

        assert_eq!(
            response.data,
            json!([{ "campaign_id": "1" }, { "campaign_id": "2" }])
        );
    }

    #[tokio::test]
    async fn auto_paginate_respects_max_items() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/open_api/v1.3/campaign/get/"))
            .and(wiremock::matchers::query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0,
                "message": "OK",
                "data": {
                    "list": [{ "campaign_id": "1" }, { "campaign_id": "2" }],
                    "page_info": { "page": 1, "page_size": 2, "total_number": 10, "total_page": 5 }
                }
            })))
            .mount(&server)
            .await;

        let client = TikTokClient::from_config(&test_config(&server.uri())).unwrap();
        let response = client
            .get_all("campaign/get", &BTreeMap::new(), Some(1))
            .await
            .unwrap();

        assert_eq!(response.data, json!([{ "campaign_id": "1" }]));
    }
}
