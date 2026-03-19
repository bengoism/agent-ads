use std::time::Duration;

use reqwest::Method;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokio::time::sleep;
use tracing::debug;

use crate::google_auth::refresh_access_token;
use crate::google_config::GoogleResolvedConfig;
use crate::google_error::{parse_google_api_error, GoogleError, GoogleResult};

#[derive(Debug, Clone)]
pub struct GoogleResponse {
    pub data: Value,
    pub paging: Option<Value>,
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AccessibleCustomersResponse {
    #[serde(default, rename = "resourceNames")]
    resource_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: Vec<Value>,
    #[serde(default, rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchStreamBatch {
    #[serde(default)]
    results: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct GoogleClient {
    http: reqwest::Client,
    api_base_url: String,
    api_version: String,
    access_token: String,
    developer_token: String,
    login_customer_id: Option<String>,
    max_retries: usize,
}

impl GoogleClient {
    pub async fn from_config(config: &GoogleResolvedConfig) -> GoogleResult<Self> {
        let refresh = refresh_access_token(
            config.timeout_seconds,
            &config.client_id,
            &config.client_secret,
            &config.refresh_token,
        )
        .await?;

        Self::from_access_token(
            &config.api_base_url,
            &config.api_version,
            config.timeout_seconds,
            &config.developer_token,
            config.login_customer_id.as_deref(),
            &refresh.access_token,
        )
    }

    pub fn from_access_token(
        api_base_url: &str,
        api_version: &str,
        timeout_seconds: u64,
        developer_token: &str,
        login_customer_id: Option<&str>,
        access_token: &str,
    ) -> GoogleResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()?;

        Ok(Self {
            http,
            api_base_url: api_base_url.trim_end_matches('/').to_string(),
            api_version: api_version.to_string(),
            access_token: access_token.to_string(),
            developer_token: developer_token.to_string(),
            login_customer_id: login_customer_id.map(str::to_string),
            max_retries: 4,
        })
    }

    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    pub async fn list_accessible_customers(&self) -> GoogleResult<GoogleResponse> {
        let (value, request_id) = self
            .request_json(Method::GET, "customers:listAccessibleCustomers", None)
            .await?;
        let payload: AccessibleCustomersResponse = serde_json::from_value(value)?;
        let rows = payload
            .resource_names
            .into_iter()
            .map(|resource_name| {
                let customer_id = resource_name
                    .strip_prefix("customers/")
                    .unwrap_or(&resource_name)
                    .to_string();
                json!({
                    "customer_id": customer_id,
                    "resource_name": resource_name,
                })
            })
            .collect::<Vec<_>>();

        Ok(GoogleResponse {
            data: Value::Array(rows),
            paging: None,
            request_id,
        })
    }

    pub async fn search(
        &self,
        customer_id: &str,
        query: &str,
        page_size: Option<u32>,
        page_token: Option<&str>,
        max_items: Option<usize>,
    ) -> GoogleResult<GoogleResponse> {
        let page = self
            .search_page(
                customer_id,
                query,
                page_size,
                page_token.filter(|value| !value.is_empty()),
            )
            .await?;
        let mut rows = page.results;
        if let Some(max_items) = max_items {
            rows.truncate(max_items);
        }

        Ok(GoogleResponse {
            data: Value::Array(rows),
            paging: page
                .next_page_token
                .map(|token| json!({ "next_page_token": token })),
            request_id: page.request_id,
        })
    }

    pub async fn search_all(
        &self,
        customer_id: &str,
        query: &str,
        page_size: Option<u32>,
        page_token: Option<&str>,
        max_items: Option<usize>,
    ) -> GoogleResult<GoogleResponse> {
        let mut collected = Vec::new();
        let mut next_page_token = page_token
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let (last_request_id, paging) = loop {
            let page = self
                .search_page(customer_id, query, page_size, next_page_token.as_deref())
                .await?;
            let request_id = page.request_id.clone();
            let page_paging = page
                .next_page_token
                .as_ref()
                .map(|token| json!({ "next_page_token": token }));

            for row in page.results {
                if let Some(max_items) = max_items {
                    if collected.len() >= max_items {
                        break;
                    }
                }
                collected.push(row);
            }

            if let Some(max_items) = max_items {
                if collected.len() >= max_items {
                    break (request_id, page_paging);
                }
            }

            let Some(token) = page.next_page_token else {
                break (request_id, page_paging);
            };
            next_page_token = Some(token);
        };

        Ok(GoogleResponse {
            data: Value::Array(collected),
            paging,
            request_id: last_request_id,
        })
    }

    pub async fn search_stream(
        &self,
        customer_id: &str,
        query: &str,
        max_items: Option<usize>,
    ) -> GoogleResult<GoogleResponse> {
        let path = format!("customers/{customer_id}/googleAds:searchStream");
        let mut body = Map::new();
        body.insert("query".to_string(), Value::String(query.to_string()));

        let (value, request_id) = self
            .request_json(Method::POST, &path, Some(&Value::Object(body)))
            .await?;
        let batches: Vec<SearchStreamBatch> = serde_json::from_value(value)?;

        let mut rows = Vec::new();
        for batch in batches {
            for row in batch.results {
                if let Some(max_items) = max_items {
                    if rows.len() >= max_items {
                        break;
                    }
                }
                rows.push(row);
            }
            if let Some(max_items) = max_items {
                if rows.len() >= max_items {
                    break;
                }
            }
        }

        Ok(GoogleResponse {
            data: Value::Array(rows),
            paging: None,
            request_id,
        })
    }

    async fn search_page(
        &self,
        customer_id: &str,
        query: &str,
        page_size: Option<u32>,
        page_token: Option<&str>,
    ) -> GoogleResult<SearchPage> {
        let path = format!("customers/{customer_id}/googleAds:search");
        let mut body = Map::new();
        body.insert("query".to_string(), Value::String(query.to_string()));
        if let Some(page_size) = page_size {
            body.insert("pageSize".to_string(), json!(page_size));
        }
        if let Some(page_token) = page_token {
            body.insert(
                "pageToken".to_string(),
                Value::String(page_token.to_string()),
            );
        }

        let (value, request_id) = self
            .request_json(Method::POST, &path, Some(&Value::Object(body)))
            .await?;
        let payload: SearchResponse = serde_json::from_value(value)?;

        Ok(SearchPage {
            results: payload.results,
            next_page_token: payload.next_page_token,
            request_id,
        })
    }

    async fn request_json(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> GoogleResult<(Value, Option<String>)> {
        let url = format!(
            "{}/{}/{}",
            self.api_base_url,
            self.api_version,
            path.trim_start_matches('/')
        );

        let mut last_error = None;
        for attempt in 0..=self.max_retries {
            let mut request = self
                .http
                .request(method.clone(), &url)
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("developer-token", &self.developer_token);

            if let Some(login_customer_id) = &self.login_customer_id {
                request = request.header("login-customer-id", login_customer_id);
            }
            if let Some(body) = body {
                request = request.json(body);
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    let request_id = response
                        .headers()
                        .get("request-id")
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    let response_body = response.text().await?;

                    if status.is_success() {
                        return Ok((serde_json::from_str(&response_body)?, request_id));
                    }

                    let api_error =
                        parse_google_api_error(status.as_u16(), request_id, &response_body);
                    if api_error.retryable() && attempt < self.max_retries {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(
                            attempt,
                            delay_ms,
                            status = status.as_u16(),
                            "retrying google ads request"
                        );
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = Some(GoogleError::Api(api_error));
                        continue;
                    }

                    return Err(GoogleError::Api(api_error));
                }
                Err(error) => {
                    if (error.is_timeout() || error.is_connect()) && attempt < self.max_retries {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(attempt, delay_ms, "retrying after transport failure");
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = Some(GoogleError::Http(error));
                        continue;
                    }

                    return Err(GoogleError::Http(error));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            GoogleError::Config("request retries exhausted without a captured error".to_string())
        }))
    }
}

#[derive(Debug)]
struct SearchPage {
    results: Vec<Value>,
    next_page_token: Option<String>,
    request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::GoogleClient;

    #[tokio::test]
    async fn lists_accessible_customers() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v23/customers:listAccessibleCustomers"))
            .and(header("developer-token", "dev-token"))
            .and(header("Authorization", "Bearer access-token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("request-id", "req-1")
                    .set_body_json(json!({
                        "resourceNames": ["customers/1234567890"]
                    })),
            )
            .mount(&server)
            .await;

        let client = GoogleClient::from_access_token(
            &server.uri(),
            "v23",
            10,
            "dev-token",
            None,
            "access-token",
        )
        .unwrap();
        let response = client.list_accessible_customers().await.unwrap();

        assert_eq!(
            response.data,
            json!([{ "customer_id": "1234567890", "resource_name": "customers/1234567890" }])
        );
        assert_eq!(response.request_id.as_deref(), Some("req-1"));
    }

    #[tokio::test]
    async fn search_uses_google_headers_and_paging() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v23/customers/1234567890/googleAds:search"))
            .and(header("developer-token", "dev-token"))
            .and(header("Authorization", "Bearer access-token"))
            .and(header("login-customer-id", "1112223333"))
            .and(body_partial_json(json!({
                "query": "SELECT campaign.id FROM campaign",
                "pageSize": 1
            })))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("request-id", "req-2")
                    .set_body_json(json!({
                        "results": [{ "campaign": { "id": "1" } }],
                        "nextPageToken": "next-token"
                    })),
            )
            .mount(&server)
            .await;

        let client = GoogleClient::from_access_token(
            &server.uri(),
            "v23",
            10,
            "dev-token",
            Some("1112223333"),
            "access-token",
        )
        .unwrap();
        let response = client
            .search(
                "1234567890",
                "SELECT campaign.id FROM campaign",
                Some(1),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(response.data, json!([{ "campaign": { "id": "1" } }]));
        assert_eq!(
            response.paging,
            Some(json!({ "next_page_token": "next-token" }))
        );
        assert_eq!(response.request_id.as_deref(), Some("req-2"));
    }

    #[tokio::test]
    async fn search_all_follows_next_page_token() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v23/customers/1234567890/googleAds:search"))
            .and(body_partial_json(json!({
                "query": "SELECT campaign.id FROM campaign"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{ "campaign": { "id": "1" } }],
                "nextPageToken": "next-token"
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v23/customers/1234567890/googleAds:search"))
            .and(body_partial_json(json!({
                "query": "SELECT campaign.id FROM campaign",
                "pageToken": "next-token"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{ "campaign": { "id": "2" } }]
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        let client = GoogleClient::from_access_token(
            &server.uri(),
            "v23",
            10,
            "dev-token",
            None,
            "access-token",
        )
        .unwrap();
        let response = client
            .search_all(
                "1234567890",
                "SELECT campaign.id FROM campaign",
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            response.data,
            json!([{ "campaign": { "id": "1" } }, { "campaign": { "id": "2" } }])
        );
    }

    #[tokio::test]
    async fn search_stream_flattens_batches() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v23/customers/1234567890/googleAds:searchStream"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                { "results": [{ "campaign": { "id": "1" } }] },
                { "results": [{ "campaign": { "id": "2" } }] }
            ])))
            .mount(&server)
            .await;

        let client = GoogleClient::from_access_token(
            &server.uri(),
            "v23",
            10,
            "dev-token",
            None,
            "access-token",
        )
        .unwrap();
        let response = client
            .search_stream("1234567890", "SELECT campaign.id FROM campaign", None)
            .await
            .unwrap();

        assert_eq!(
            response.data,
            json!([{ "campaign": { "id": "1" } }, { "campaign": { "id": "2" } }])
        );
    }

    #[tokio::test]
    async fn returns_google_api_error_exit_code() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v23/customers/1234567890/googleAds:search"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("request-id", "req-err")
                    .set_body_json(json!({
                        "error": {
                            "code": 429,
                            "message": "rate limited",
                            "status": "RESOURCE_EXHAUSTED"
                        }
                    })),
            )
            .expect(5)
            .mount(&server)
            .await;

        let client = GoogleClient::from_access_token(
            &server.uri(),
            "v23",
            10,
            "dev-token",
            None,
            "access-token",
        )
        .unwrap();
        let error = client
            .search(
                "1234567890",
                "SELECT campaign.id FROM campaign",
                None,
                None,
                None,
            )
            .await
            .unwrap_err();

        assert_eq!(error.exit_code(), 5);
    }
}
