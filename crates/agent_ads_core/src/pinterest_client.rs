use std::time::Duration;

use reqwest::Method;
use serde_json::{json, Value};
use tokio::time::sleep;
use tracing::debug;

use crate::pinterest_config::PinterestResolvedConfig;
use crate::pinterest_error::{parse_pinterest_api_error, PinterestError, PinterestResult};

#[derive(Debug, Clone)]
pub struct PinterestResponse {
    pub data: Value,
    pub paging: Option<Value>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PinterestClient {
    http: reqwest::Client,
    api_base_url: String,
    api_version: String,
    access_token: String,
    max_retries: usize,
}

impl PinterestClient {
    pub fn from_config(config: &PinterestResolvedConfig) -> PinterestResult<Self> {
        Self::from_access_token(
            &config.api_base_url,
            &config.api_version,
            config.timeout_seconds,
            &config.access_token,
        )
    }

    pub fn from_access_token(
        api_base_url: &str,
        api_version: &str,
        timeout_seconds: u64,
        access_token: &str,
    ) -> PinterestResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()?;

        Ok(Self {
            http,
            api_base_url: api_base_url.trim_end_matches('/').to_string(),
            api_version: api_version.to_string(),
            access_token: access_token.to_string(),
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
    ) -> PinterestResult<PinterestResponse> {
        let (data, request_id) = self.request_json(Method::GET, path, params, None).await?;
        Ok(PinterestResponse {
            paging: extract_bookmark(&data).map(|bookmark| json!({ "bookmark": bookmark })),
            data,
            request_id,
        })
    }

    pub async fn post_json(&self, path: &str, body: &Value) -> PinterestResult<PinterestResponse> {
        let (data, request_id) = self
            .request_json(Method::POST, path, &[], Some(body))
            .await?;
        Ok(PinterestResponse {
            paging: extract_bookmark(&data).map(|bookmark| json!({ "bookmark": bookmark })),
            data,
            request_id,
        })
    }

    pub async fn get_list(
        &self,
        path: &str,
        params: &[(String, String)],
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> PinterestResult<PinterestResponse> {
        if !fetch_all {
            let (data, request_id) = self.request_json(Method::GET, path, params, None).await?;
            let paging = extract_bookmark(&data).map(|bookmark| json!({ "bookmark": bookmark }));
            let items = extract_items(&data);
            return Ok(PinterestResponse {
                data: Value::Array(items),
                paging,
                request_id,
            });
        }

        let mut collected = Vec::new();
        let mut current_params = params.to_vec();
        let mut last_request_id;
        let mut last_bookmark;

        loop {
            let (data, request_id) = self
                .request_json(Method::GET, path, &current_params, None)
                .await?;
            last_request_id = request_id;
            last_bookmark = extract_bookmark(&data);

            for item in extract_items(&data) {
                if let Some(max_items) = max_items {
                    if collected.len() >= max_items {
                        break;
                    }
                }
                collected.push(item);
            }

            if let Some(max_items) = max_items {
                if collected.len() >= max_items {
                    break;
                }
            }

            let Some(bookmark) = last_bookmark.clone() else {
                break;
            };
            current_params = replace_query_param(&current_params, "bookmark", &bookmark);
        }

        Ok(PinterestResponse {
            data: Value::Array(collected),
            paging: last_bookmark.map(|bookmark| json!({ "bookmark": bookmark })),
            request_id: last_request_id,
        })
    }

    async fn request_json(
        &self,
        method: Method,
        path: &str,
        params: &[(String, String)],
        body: Option<&Value>,
    ) -> PinterestResult<(Value, Option<String>)> {
        let url = self.build_url(path);
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let mut request = self
                .http
                .request(method.clone(), &url)
                .bearer_auth(&self.access_token)
                .query(params);

            if let Some(body) = body {
                request = request.json(body);
            }

            match request.send().await {
                Ok(response) => {
                    let request_id = response
                        .headers()
                        .get("x-request-id")
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    let status = response.status();
                    let response_body = response.text().await?;

                    if status.is_success() {
                        let value: Value = serde_json::from_str(&response_body)?;
                        return Ok((value, request_id));
                    }

                    let api_error =
                        parse_pinterest_api_error(status.as_u16(), request_id, &response_body);
                    if api_error.retryable() && attempt < self.max_retries {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(
                            attempt,
                            delay_ms,
                            status = api_error.http_status,
                            "retrying pinterest api request"
                        );
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = Some(PinterestError::Api(api_error));
                        continue;
                    }

                    return Err(PinterestError::Api(api_error));
                }
                Err(error) => {
                    if (error.is_timeout() || error.is_connect()) && attempt < self.max_retries {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(attempt, delay_ms, "retrying pinterest transport failure");
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = Some(PinterestError::Http(error));
                        continue;
                    }
                    return Err(PinterestError::Http(error));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            PinterestError::Config(
                "request retries exhausted without a captured Pinterest error".to_string(),
            )
        }))
    }

    fn build_url(&self, path: &str) -> String {
        let trimmed = path.trim_matches('/');
        format!("{}/{}/{}", self.api_base_url, self.api_version, trimmed)
    }
}

fn extract_items(value: &Value) -> Vec<Value> {
    value
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn extract_bookmark(value: &Value) -> Option<String> {
    value
        .get("bookmark")
        .and_then(Value::as_str)
        .map(str::to_string)
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
