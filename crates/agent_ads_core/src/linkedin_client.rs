use std::time::Duration;

use reqwest::Method;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::time::sleep;
use tracing::debug;

use crate::linkedin_config::LinkedInResolvedConfig;
use crate::linkedin_error::{parse_linkedin_api_error, LinkedInError, LinkedInResult};

const MAX_RAW_URL_BYTES: usize = 8 * 1024;
const MAX_QUERY_STRING_BYTES: usize = 4 * 1024;

#[derive(Debug, Clone)]
pub struct LinkedInResponse {
    pub data: Value,
    pub paging: Option<Value>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LinkedInClient {
    http: reqwest::Client,
    api_base_url: String,
    api_version: String,
    access_token: String,
    max_retries: usize,
}

#[derive(Debug, Deserialize)]
struct ListResponse {
    #[serde(default)]
    metadata: Option<ListMetadata>,
    #[serde(default)]
    paging: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ListMetadata {
    #[serde(default, rename = "nextPageToken")]
    next_page_token: Option<String>,
}

impl LinkedInClient {
    pub fn from_config(config: &LinkedInResolvedConfig) -> LinkedInResult<Self> {
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

    pub async fn get_object(
        &self,
        path: &str,
        params: &[(String, String)],
        extra_headers: &[(&str, &str)],
    ) -> LinkedInResult<LinkedInResponse> {
        let (data, request_id) = self
            .request_json(Method::GET, path, params, extra_headers)
            .await?;
        Ok(LinkedInResponse {
            paging: extract_paging(&data),
            data,
            request_id,
        })
    }

    pub async fn get_list(
        &self,
        path: &str,
        params: &[(String, String)],
        extra_headers: &[(&str, &str)],
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> LinkedInResult<LinkedInResponse> {
        if !fetch_all {
            let (data, request_id) = self
                .request_json(Method::GET, path, params, extra_headers)
                .await?;
            let elements = extract_elements(&data);
            return Ok(LinkedInResponse {
                data: Value::Array(truncate_items(elements, max_items)),
                paging: extract_paging(&data),
                request_id,
            });
        }

        let mut collected = Vec::new();
        let mut current_params = params.to_vec();
        let (mut data, mut last_request_id) = self
            .request_json(Method::GET, path, &current_params, extra_headers)
            .await?;
        let mut last_paging = extract_paging(&data);

        loop {
            for element in extract_elements(&data) {
                if let Some(max_items) = max_items {
                    if collected.len() >= max_items {
                        break;
                    }
                }
                collected.push(element);
            }

            if let Some(max_items) = max_items {
                if collected.len() >= max_items {
                    break;
                }
            }

            let Some(next_page_token) = extract_next_page_token(&data) else {
                break;
            };
            current_params = replace_query_param(&current_params, "pageToken", &next_page_token);
            let (next_data, next_request_id) = self
                .request_json(Method::GET, path, &current_params, extra_headers)
                .await?;
            data = next_data;
            last_request_id = next_request_id;
            last_paging = extract_paging(&data);
        }

        Ok(LinkedInResponse {
            data: Value::Array(collected),
            paging: last_paging,
            request_id: last_request_id,
        })
    }

    async fn request_json(
        &self,
        method: Method,
        path: &str,
        params: &[(String, String)],
        extra_headers: &[(&str, &str)],
    ) -> LinkedInResult<(Value, Option<String>)> {
        let url = self.build_url(path);
        let query_string = encode_query(params);
        let use_query_tunneling = method == Method::GET && should_query_tunnel(&url, &query_string);
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let mut request = if use_query_tunneling {
                let request = self
                    .http
                    .post(&url)
                    .header("X-HTTP-Method-Override", "GET")
                    .header("Content-Type", "application/x-www-form-urlencoded");
                if query_string.is_empty() {
                    request
                } else {
                    request.body(query_string.clone())
                }
            } else {
                let full_url = if query_string.is_empty() {
                    url.clone()
                } else {
                    format!("{url}?{query_string}")
                };
                self.http.request(method.clone(), &full_url)
            };

            request = request
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("Linkedin-Version", &self.api_version)
                .header("X-Restli-Protocol-Version", "2.0.0");

            for (name, value) in extra_headers {
                request = request.header(*name, *value);
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    let request_id = response
                        .headers()
                        .get("x-li-request-id")
                        .or_else(|| response.headers().get("x-li-uuid"))
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    let response_body = response.text().await?;

                    if status.is_success() {
                        return Ok((serde_json::from_str(&response_body)?, request_id));
                    }

                    let api_error =
                        parse_linkedin_api_error(status.as_u16(), request_id, &response_body);
                    if api_error.retryable() && attempt < self.max_retries {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(
                            attempt,
                            delay_ms,
                            status = status.as_u16(),
                            "retrying linkedin request"
                        );
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = Some(LinkedInError::Api(api_error));
                        continue;
                    }

                    return Err(LinkedInError::Api(api_error));
                }
                Err(error) => {
                    if (error.is_timeout() || error.is_connect()) && attempt < self.max_retries {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(attempt, delay_ms, "retrying linkedin transport failure");
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = Some(LinkedInError::Http(error));
                        continue;
                    }
                    return Err(LinkedInError::Http(error));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            LinkedInError::Config(
                "request retries exhausted without a captured LinkedIn error".to_string(),
            )
        }))
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}/{}", self.api_base_url, path.trim_start_matches('/'))
    }
}

pub fn encode_path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn encode_query(params: &[(String, String)]) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in params {
        serializer.append_pair(key, value);
    }
    serializer.finish()
}

fn should_query_tunnel(url: &str, query_string: &str) -> bool {
    if query_string.is_empty() {
        return false;
    }

    query_string.len() > MAX_QUERY_STRING_BYTES
        || url.len() + 1 + query_string.len() > MAX_RAW_URL_BYTES
}

fn extract_elements(data: &Value) -> Vec<Value> {
    data.get("elements")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn truncate_items(mut items: Vec<Value>, max_items: Option<usize>) -> Vec<Value> {
    if let Some(max_items) = max_items {
        items.truncate(max_items);
    }
    items
}

fn extract_next_page_token(data: &Value) -> Option<String> {
    serde_json::from_value::<ListResponse>(data.clone())
        .ok()
        .and_then(|parsed| {
            parsed
                .metadata
                .and_then(|metadata| metadata.next_page_token)
        })
}

fn extract_paging(data: &Value) -> Option<Value> {
    if let Some(next_page_token) = extract_next_page_token(data) {
        return Some(json!({ "next_page_token": next_page_token }));
    }

    serde_json::from_value::<ListResponse>(data.clone())
        .ok()
        .and_then(|parsed| parsed.paging)
}

fn replace_query_param(
    params: &[(String, String)],
    key: &str,
    value: &str,
) -> Vec<(String, String)> {
    let mut next = params
        .iter()
        .filter(|(existing_key, _)| existing_key != key)
        .cloned()
        .collect::<Vec<_>>();
    next.push((key.to_string(), value.to_string()));
    next
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{body_string_contains, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::LinkedInClient;
    use crate::linkedin_config::LinkedInResolvedConfig;
    use crate::output::OutputFormat;

    fn test_config(base_url: &str) -> LinkedInResolvedConfig {
        LinkedInResolvedConfig {
            access_token: "access-token".to_string(),
            api_base_url: base_url.to_string(),
            api_version: "202603".to_string(),
            timeout_seconds: 10,
            default_account_id: None,
            output_format: OutputFormat::Json,
            config_path: "agent-ads.config.json".into(),
        }
    }

    #[tokio::test]
    async fn uses_required_linkedin_headers() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/adAccounts/123"))
            .and(header("Authorization", "Bearer access-token"))
            .and(header("Linkedin-Version", "202603"))
            .and(header("X-Restli-Protocol-Version", "2.0.0"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("x-li-request-id", "req-1")
                    .set_body_json(json!({ "id": 123 })),
            )
            .mount(&server)
            .await;

        let client = LinkedInClient::from_config(&test_config(&server.uri())).unwrap();
        let response = client.get_object("adAccounts/123", &[], &[]).await.unwrap();

        assert_eq!(response.data, json!({ "id": 123 }));
        assert_eq!(response.request_id.as_deref(), Some("req-1"));
    }

    #[tokio::test]
    async fn auto_paginates_with_next_page_token() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/adAccounts/123/adCampaigns"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "elements": [{ "id": 1 }],
                "metadata": { "nextPageToken": "next-token" }
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/adAccounts/123/adCampaigns"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "elements": [{ "id": 2 }]
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        let client = LinkedInClient::from_config(&test_config(&server.uri())).unwrap();
        let response = client
            .get_list(
                "adAccounts/123/adCampaigns",
                &[("q".to_string(), "search".to_string())],
                &[],
                true,
                None,
            )
            .await
            .unwrap();

        assert_eq!(response.data, json!([{ "id": 1 }, { "id": 2 }]));
    }

    #[tokio::test]
    async fn uses_query_tunneling_for_long_requests() {
        let server = MockServer::start().await;
        let long_value = "x".repeat(5000);

        Mock::given(method("POST"))
            .and(path("/adAnalytics"))
            .and(header("X-HTTP-Method-Override", "GET"))
            .and(header("Content-Type", "application/x-www-form-urlencoded"))
            .and(body_string_contains("q=analytics"))
            .and(body_string_contains("fields="))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "elements": [{ "impressions": 1 }]
            })))
            .mount(&server)
            .await;

        let client = LinkedInClient::from_config(&test_config(&server.uri())).unwrap();
        let response = client
            .get_object(
                "adAnalytics",
                &[
                    ("q".to_string(), "analytics".to_string()),
                    ("fields".to_string(), long_value),
                ],
                &[],
            )
            .await
            .unwrap();

        assert_eq!(response.data, json!({ "elements": [{ "impressions": 1 }] }));
    }

    #[tokio::test]
    async fn returns_linkedin_api_error_exit_code() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/adAccounts/123"))
            .respond_with(ResponseTemplate::new(429).set_body_json(json!({
                "message": "rate limited",
                "serviceErrorCode": 429,
                "status": 429
            })))
            .expect(5)
            .mount(&server)
            .await;

        let client = LinkedInClient::from_config(&test_config(&server.uri())).unwrap();
        let error = client
            .get_object("adAccounts/123", &[], &[])
            .await
            .unwrap_err();

        assert_eq!(error.exit_code(), 7);
    }
}
