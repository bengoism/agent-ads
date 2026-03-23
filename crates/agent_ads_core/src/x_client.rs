use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{io::Cursor, io::Read};

use flate2::read::GzDecoder;
use reqwest::header::{HeaderMap, RETRY_AFTER};
use reqwest::{Method, StatusCode};
use reqwest_oauth1::{Error as OAuthRequestError, OAuthClientProvider, Secrets};
use serde_json::{json, Value};
use tokio::time::sleep;
use tracing::debug;

use crate::x_config::XResolvedConfig;
use crate::x_error::{parse_x_api_error, XError, XResult};

#[derive(Debug, Clone)]
pub struct XResponse {
    pub data: Value,
    pub paging: Option<Value>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct XClient {
    http: reqwest::Client,
    api_base_url: String,
    api_version: String,
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_token_secret: String,
    max_retries: usize,
}

impl XClient {
    pub fn from_config(config: &XResolvedConfig) -> XResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            http,
            api_base_url: config.api_base_url.trim_end_matches('/').to_string(),
            api_version: config.api_version.clone(),
            consumer_key: config.consumer_key.clone(),
            consumer_secret: config.consumer_secret.clone(),
            access_token: config.access_token.clone(),
            access_token_secret: config.access_token_secret.clone(),
            max_retries: 4,
        })
    }

    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    pub async fn get_object(&self, path: &str, params: &[(String, String)]) -> XResult<XResponse> {
        let (value, request_id) = self.request_json(Method::GET, path, params).await?;
        Ok(XResponse {
            paging: extract_next_cursor(&value).map(|cursor| json!({ "cursor": cursor })),
            data: extract_data(value),
            request_id,
        })
    }

    pub async fn post_form(&self, path: &str, params: &[(String, String)]) -> XResult<XResponse> {
        let (value, request_id) = self.request_json(Method::POST, path, params).await?;
        Ok(XResponse {
            paging: extract_next_cursor(&value).map(|cursor| json!({ "cursor": cursor })),
            data: extract_data(value),
            request_id,
        })
    }

    pub async fn download_json_url(&self, url: &str) -> XResult<Value> {
        let response = self.http.get(url).send().await?;
        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let bytes = response.bytes().await?;

        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes).into_owned();
            return Err(XError::Api(parse_x_api_error(
                status.as_u16(),
                request_id,
                &body,
            )));
        }

        let payload = if bytes.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = GzDecoder::new(Cursor::new(bytes));
            let mut decoded = String::new();
            decoder.read_to_string(&mut decoded)?;
            decoded
        } else {
            String::from_utf8_lossy(&bytes).into_owned()
        };

        Ok(serde_json::from_str(&payload)?)
    }

    pub async fn get_list(
        &self,
        path: &str,
        params: &[(String, String)],
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> XResult<XResponse> {
        if !fetch_all {
            let (value, request_id) = self.request_json(Method::GET, path, params).await?;
            return Ok(XResponse {
                data: Value::Array(truncate_items(extract_items(&value), max_items)),
                paging: extract_next_cursor(&value).map(|cursor| json!({ "cursor": cursor })),
                request_id,
            });
        }

        let mut current_params = params.to_vec();
        let (first_value, mut last_request_id) = self
            .request_json(Method::GET, path, &current_params)
            .await?;
        let mut collected = truncate_items(extract_items(&first_value), max_items);
        let mut next_cursor = extract_next_cursor(&first_value);

        while let Some(cursor) = next_cursor.clone() {
            if let Some(max_items) = max_items {
                if collected.len() >= max_items {
                    break;
                }
            }

            current_params = replace_query_param(&current_params, "cursor", &cursor);
            let (value, request_id) = self
                .request_json(Method::GET, path, &current_params)
                .await?;
            last_request_id = request_id;
            next_cursor = extract_next_cursor(&value);

            for item in extract_items(&value) {
                if let Some(max_items) = max_items {
                    if collected.len() >= max_items {
                        break;
                    }
                }
                collected.push(item);
            }
        }

        Ok(XResponse {
            data: Value::Array(collected),
            paging: next_cursor.map(|cursor| json!({ "cursor": cursor })),
            request_id: last_request_id,
        })
    }

    async fn request_json(
        &self,
        method: Method,
        path: &str,
        params: &[(String, String)],
    ) -> XResult<(Value, Option<String>)> {
        let url = self.build_url(path);
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let form_params = params.to_vec();
            let secrets = Secrets::new_with_token(
                self.consumer_key.clone(),
                self.consumer_secret.clone(),
                self.access_token.clone(),
                self.access_token_secret.clone(),
            );
            let signed = self.http.clone().oauth1(secrets);
            let request = match method {
                Method::GET => signed.get(&url).query(params),
                Method::POST => signed.post(&url).form(&form_params),
                Method::DELETE => signed.delete(&url).query(params),
                Method::PUT => signed.put(&url).form(&form_params),
                Method::PATCH => signed.patch(&url).form(&form_params),
                _ => signed.request(method.clone(), &url).query(params),
            };

            match request.send().await {
                Ok(response) => {
                    let request_id = response
                        .headers()
                        .get("x-transaction-id")
                        .or_else(|| response.headers().get("x-request-id"))
                        .or_else(|| response.headers().get("x-correlation-id"))
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    let headers = response.headers().clone();
                    let status = response.status();
                    let response_body = response.text().await?;

                    if status.is_success() {
                        let value: Value = serde_json::from_str(&response_body)?;
                        return Ok((value, request_id));
                    }

                    let api_error = parse_x_api_error(status.as_u16(), request_id, &response_body);
                    if api_error.retryable() && attempt < self.max_retries {
                        let delay = retry_delay_for_api_response(
                            &headers,
                            status,
                            attempt,
                            SystemTime::now(),
                        );
                        let delay_ms = delay.as_millis();
                        debug!(attempt, delay_ms, "retrying x ads api request");
                        sleep(delay).await;
                        last_error = Some(XError::Api(api_error));
                        continue;
                    }

                    return Err(XError::Api(api_error));
                }
                Err(error) => match map_request_error(error) {
                    RetryDecision::Retry(retry_error) if attempt < self.max_retries => {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(attempt, delay_ms, "retrying x ads transport failure");
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = Some(retry_error);
                        continue;
                    }
                    RetryDecision::Retry(retry_error) | RetryDecision::Fail(retry_error) => {
                        return Err(retry_error);
                    }
                },
            }
        }

        Err(last_error.unwrap_or_else(|| {
            XError::Config(
                "request retries exhausted without a captured X Ads API error".to_string(),
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
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn extract_data(value: Value) -> Value {
    value.get("data").cloned().unwrap_or(value)
}

fn extract_next_cursor(value: &Value) -> Option<String> {
    value
        .get("next_cursor")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn truncate_items(mut items: Vec<Value>, max_items: Option<usize>) -> Vec<Value> {
    if let Some(max_items) = max_items {
        items.truncate(max_items);
    }
    items
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

fn retry_delay_for_api_response(
    headers: &HeaderMap,
    status: StatusCode,
    attempt: usize,
    now: SystemTime,
) -> Duration {
    match status {
        StatusCode::TOO_MANY_REQUESTS => parse_rate_limit_reset(headers, now),
        StatusCode::SERVICE_UNAVAILABLE => parse_retry_after(headers, now),
        _ => None,
    }
    .unwrap_or_else(|| default_retry_delay(attempt))
}

fn default_retry_delay(attempt: usize) -> Duration {
    Duration::from_millis(250_u64 * 2_u64.pow(attempt as u32))
}

fn parse_rate_limit_reset(headers: &HeaderMap, now: SystemTime) -> Option<Duration> {
    let now_epoch = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    ["x-account-rate-limit-reset", "x-rate-limit-reset"]
        .iter()
        .filter_map(|header_name| {
            headers
                .get(*header_name)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.trim().parse::<u64>().ok())
        })
        .next()
        .map(|reset_at| Duration::from_secs(reset_at.saturating_sub(now_epoch)))
}

fn parse_retry_after(headers: &HeaderMap, now: SystemTime) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();

    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    let retry_at = httpdate::parse_http_date(value).ok()?;
    Some(match retry_at.duration_since(now) {
        Ok(delay) => delay,
        Err(_) => Duration::ZERO,
    })
}

enum RetryDecision {
    Retry(XError),
    Fail(XError),
}

fn map_request_error(error: OAuthRequestError) -> RetryDecision {
    match error {
        OAuthRequestError::Reqwest(error) => {
            if error.is_timeout() || error.is_connect() {
                RetryDecision::Retry(XError::Http(error))
            } else {
                RetryDecision::Fail(XError::Http(error))
            }
        }
        other => RetryDecision::Fail(XError::Config(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_retry_delay, parse_rate_limit_reset, parse_retry_after,
        retry_delay_for_api_response, XClient,
    };
    use crate::output::OutputFormat;
    use crate::x_config::XResolvedConfig;
    use httpdate::fmt_http_date;
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
    use reqwest::StatusCode;
    use serde_json::json;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(base_url: &str) -> XResolvedConfig {
        XResolvedConfig {
            consumer_key: "consumer-key".to_string(),
            consumer_secret: "consumer-secret".to_string(),
            access_token: "access-token".to_string(),
            access_token_secret: "access-token-secret".to_string(),
            api_base_url: base_url.to_string(),
            api_version: "12".to_string(),
            timeout_seconds: 10,
            default_account_id: None,
            output_format: OutputFormat::Json,
            config_path: "agent-ads.config.json".into(),
        }
    }

    #[test]
    fn prefers_account_rate_limit_reset_for_429() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-account-rate-limit-reset",
            HeaderValue::from_static("1005"),
        );
        headers.insert("x-rate-limit-reset", HeaderValue::from_static("1010"));

        let now = UNIX_EPOCH + Duration::from_secs(1000);
        assert_eq!(
            parse_rate_limit_reset(&headers, now),
            Some(Duration::from_secs(5))
        );
    }

    #[test]
    fn falls_back_to_global_rate_limit_reset_for_429() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-account-rate-limit-reset",
            HeaderValue::from_static("invalid"),
        );
        headers.insert("x-rate-limit-reset", HeaderValue::from_static("1007"));

        let now = UNIX_EPOCH + Duration::from_secs(1000);
        assert_eq!(
            parse_rate_limit_reset(&headers, now),
            Some(Duration::from_secs(7))
        );
    }

    #[test]
    fn uses_retry_after_seconds_for_503() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("12"));

        let now = UNIX_EPOCH + Duration::from_secs(1000);
        assert_eq!(
            parse_retry_after(&headers, now),
            Some(Duration::from_secs(12))
        );
    }

    #[test]
    fn uses_retry_after_http_date_for_503() {
        let now = UNIX_EPOCH + Duration::from_secs(1000);
        let retry_at = now + Duration::from_secs(30);
        let mut headers = HeaderMap::new();
        headers.insert(
            RETRY_AFTER,
            HeaderValue::from_str(&fmt_http_date(retry_at)).unwrap(),
        );

        assert_eq!(
            parse_retry_after(&headers, now),
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn falls_back_to_exponential_backoff_when_headers_are_missing_or_invalid() {
        let mut invalid_headers = HeaderMap::new();
        invalid_headers.insert("x-rate-limit-reset", HeaderValue::from_static("invalid"));
        invalid_headers.insert(RETRY_AFTER, HeaderValue::from_static("nope"));

        let now = UNIX_EPOCH + Duration::from_secs(1000);
        assert_eq!(
            retry_delay_for_api_response(&HeaderMap::new(), StatusCode::TOO_MANY_REQUESTS, 2, now,),
            default_retry_delay(2)
        );
        assert_eq!(
            retry_delay_for_api_response(&invalid_headers, StatusCode::SERVICE_UNAVAILABLE, 1, now,),
            default_retry_delay(1)
        );
    }

    #[tokio::test]
    async fn retries_rate_limited_requests_until_budget_exhausted() {
        let server = MockServer::start().await;
        let reset_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(1);

        Mock::given(method("GET"))
            .and(path("/12/accounts"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("x-rate-limit-reset", reset_at.to_string())
                    .set_body_json(json!({
                        "errors": [{
                            "message": "rate limited"
                        }]
                    })),
            )
            .expect(5)
            .mount(&server)
            .await;

        let client = XClient::from_config(&test_config(&server.uri())).unwrap();
        let error = client.get_object("accounts", &[]).await.unwrap_err();

        assert_eq!(error.exit_code(), 8);
    }
}
