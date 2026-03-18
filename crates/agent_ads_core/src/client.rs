use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::time::sleep;
use tracing::debug;

use crate::config::ResolvedConfig;
use crate::error::{
    is_retryable_status, GraphApiError, GraphApiErrorEnvelope, MetaAdsError, Result,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PagingCursors {
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Paging {
    #[serde(default)]
    pub cursors: Option<PagingCursors>,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub previous: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paging: Option<Paging>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl GraphResponse {
    pub fn new(data: Value) -> Self {
        Self {
            data,
            paging: None,
            summary: None,
            request_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphClient {
    http: reqwest::Client,
    api_base_url: String,
    api_version: String,
    access_token: String,
    max_retries: usize,
}

impl GraphClient {
    pub fn from_config(config: &ResolvedConfig) -> Result<Self> {
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

    pub async fn get_node(
        &self,
        node_id: &str,
        params: &BTreeMap<String, String>,
        fields: &[String],
    ) -> Result<GraphResponse> {
        let path = normalize_path(node_id);
        self.request_json(Method::GET, Some(&path), None, params, fields)
            .await
    }

    pub async fn get_edge(
        &self,
        node_id: &str,
        edge: &str,
        params: &BTreeMap<String, String>,
        fields: &[String],
    ) -> Result<GraphResponse> {
        let path = format!("{}/{}", normalize_path(node_id), edge.trim_matches('/'));
        self.request_json(Method::GET, Some(&path), None, params, fields)
            .await
    }

    pub async fn get_edge_all(
        &self,
        node_id: &str,
        edge: &str,
        params: &BTreeMap<String, String>,
        fields: &[String],
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        let mut working_params = params.clone();
        let mut collected = Vec::new();
        let (last_paging, last_summary, last_request_id) = loop {
            let response = self
                .get_edge(node_id, edge, &working_params, fields)
                .await?;

            match response.data {
                Value::Array(values) => {
                    for value in values {
                        if let Some(max_items) = max_items {
                            if collected.len() >= max_items {
                                break;
                            }
                        }
                        collected.push(value);
                    }
                }
                other => {
                    return Err(MetaAdsError::InvalidArgument(format!(
                        "expected array data when paginating, received {other}"
                    )))
                }
            }

            if let Some(max_items) = max_items {
                if collected.len() >= max_items {
                    break (response.paging, response.summary, response.request_id);
                }
            }

            let next_after = response
                .paging
                .as_ref()
                .and_then(|paging| paging.next.as_ref())
                .and_then(|_| response.paging.as_ref().and_then(paging_after));

            let Some(after) = next_after else {
                break (response.paging, response.summary, response.request_id);
            };
            working_params.insert("after".to_string(), after);
        };

        Ok(GraphResponse {
            data: Value::Array(collected),
            paging: last_paging,
            summary: last_summary,
            request_id: last_request_id,
        })
    }

    pub async fn post_edge(
        &self,
        node_id: &str,
        edge: &str,
        params: &BTreeMap<String, String>,
        fields: &[String],
    ) -> Result<GraphResponse> {
        let path = format!("{}/{}", normalize_path(node_id), edge.trim_matches('/'));
        self.request_json(Method::POST, Some(&path), None, params, fields)
            .await
    }

    pub async fn request_next_page(&self, next_url: &str) -> Result<GraphResponse> {
        self.request_json(Method::GET, None, Some(next_url), &BTreeMap::new(), &[])
            .await
    }

    async fn request_json(
        &self,
        method: Method,
        path: Option<&str>,
        absolute_url: Option<&str>,
        params: &BTreeMap<String, String>,
        fields: &[String],
    ) -> Result<GraphResponse> {
        let url = absolute_url.map(str::to_string).unwrap_or_else(|| {
            format!(
                "{}/{}/{}",
                self.api_base_url,
                self.api_version,
                path.unwrap()
            )
        });
        let mut query = params.clone();
        query.insert("access_token".to_string(), self.access_token.clone());
        if !fields.is_empty() {
            query.insert("fields".to_string(), fields.join(","));
        }

        let mut last_error = None;
        for attempt in 0..=self.max_retries {
            let request = self.http.request(method.clone(), &url).query(&query);
            let response = request.send().await;
            match response {
                Ok(response) => {
                    let status = response.status();
                    let request_id = response
                        .headers()
                        .get("x-fb-request-id")
                        .or_else(|| response.headers().get("x-fb-trace-id"))
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    let body = response.text().await?;

                    if status.is_success() {
                        let value = serde_json::from_str::<Value>(&body)?;
                        return Ok(parse_graph_response(value, request_id));
                    }

                    let graph_error = parse_graph_error(status, &body);
                    let retryable = is_retryable_status(status)
                        || graph_error
                            .as_ref()
                            .map(GraphApiError::retryable)
                            .unwrap_or(false);
                    if retryable && attempt < self.max_retries {
                        let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                        debug!(attempt, delay_ms, "retrying graph api request");
                        sleep(Duration::from_millis(delay_ms)).await;
                        last_error = graph_error.map(MetaAdsError::Api);
                        continue;
                    }

                    return Err(graph_error.map(MetaAdsError::Api).unwrap_or_else(|| {
                        MetaAdsError::Config(format!("unexpected response status {status}"))
                    }));
                }
                Err(error) => {
                    if error.is_timeout() || error.is_connect() {
                        if attempt < self.max_retries {
                            let delay_ms = 250_u64 * 2_u64.pow(attempt as u32);
                            debug!(attempt, delay_ms, "retrying after transport failure");
                            sleep(Duration::from_millis(delay_ms)).await;
                            last_error = Some(MetaAdsError::Http(error));
                            continue;
                        }
                    }
                    return Err(MetaAdsError::Http(error));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            MetaAdsError::Config("request retries exhausted without a captured error".to_string())
        }))
    }
}

fn parse_graph_error(status: StatusCode, body: &str) -> Option<GraphApiError> {
    if let Ok(mut envelope) =
        serde_json::from_str::<GraphApiErrorEnvelope>(body).map(GraphApiError::from)
    {
        envelope.status_code = Some(status.as_u16());
        return Some(envelope);
    }

    Some(GraphApiError {
        message: body.to_string(),
        error_type: None,
        code: None,
        error_subcode: None,
        fbtrace_id: None,
        is_transient: Some(is_retryable_status(status)),
        status_code: Some(status.as_u16()),
    })
}

fn parse_graph_response(value: Value, request_id: Option<String>) -> GraphResponse {
    match value {
        Value::Object(mut map) => {
            let paging = take_paging(&mut map);
            let summary = map.remove("summary");
            if let Some(data) = map.remove("data") {
                GraphResponse {
                    data,
                    paging,
                    summary,
                    request_id,
                }
            } else {
                GraphResponse {
                    data: Value::Object(map),
                    paging,
                    summary,
                    request_id,
                }
            }
        }
        other => GraphResponse {
            data: other,
            paging: None,
            summary: None,
            request_id,
        },
    }
}

fn take_paging(map: &mut Map<String, Value>) -> Option<Paging> {
    map.remove("paging")
        .and_then(|value| serde_json::from_value::<Paging>(value).ok())
}

fn paging_after(paging: &Paging) -> Option<String> {
    paging
        .cursors
        .as_ref()
        .and_then(|cursors| cursors.after.clone())
}

fn normalize_path(path: &str) -> String {
    path.trim_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use wiremock::matchers::{method, path, query_param, query_param_is_missing};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::GraphClient;
    use crate::config::ResolvedConfig;
    use crate::output::OutputFormat;

    fn test_config(base_url: &str) -> ResolvedConfig {
        ResolvedConfig {
            access_token: "token".to_string(),
            api_base_url: base_url.to_string(),
            api_version: "v25.0".to_string(),
            timeout_seconds: 10,
            default_business_id: None,
            default_account_id: None,
            output_format: OutputFormat::Json,
            config_path: "agent-ads.config.json".into(),
        }
    }

    #[tokio::test]
    async fn parses_data_and_paging() {
        let server = MockServer::start().await;
        let body = json!({
            "data": [{ "id": "1" }],
            "paging": {
                "cursors": { "after": "cursor-1" },
                "next": "https://example.com/next"
            }
        });

        Mock::given(method("GET"))
            .and(path("/v25.0/me/businesses"))
            .and(query_param("access_token", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let client = GraphClient::from_config(&test_config(&server.uri())).unwrap();
        let response = client
            .get_edge("me", "businesses", &BTreeMap::new(), &[])
            .await
            .unwrap();

        assert_eq!(response.data, json!([{ "id": "1" }]));
        assert_eq!(
            response.paging.unwrap().cursors.unwrap().after.unwrap(),
            "cursor-1"
        );
    }

    #[tokio::test]
    async fn paginates_until_last_page() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v25.0/act_1/campaigns"))
            .and(query_param_is_missing("after"))
            .and(query_param("access_token", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{ "id": "1" }],
                "paging": {
                    "cursors": { "after": "next-cursor" },
                    "next": "https://example.com/next"
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v25.0/act_1/campaigns"))
            .and(query_param("after", "next-cursor"))
            .and(query_param("access_token", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{ "id": "2" }]
            })))
            .mount(&server)
            .await;

        let client = GraphClient::from_config(&test_config(&server.uri())).unwrap();
        let response = client
            .get_edge_all("act_1", "campaigns", &BTreeMap::new(), &[], None)
            .await
            .unwrap();

        assert_eq!(response.data, json!([{ "id": "1" }, { "id": "2" }]));
    }

    #[tokio::test]
    async fn paginates_until_max_items() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v25.0/act_1/campaigns"))
            .and(query_param_is_missing("after"))
            .and(query_param("access_token", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{ "id": "1" }, { "id": "2" }],
                "paging": {
                    "cursors": { "after": "next-cursor" },
                    "next": "https://example.com/next"
                }
            })))
            .mount(&server)
            .await;

        let client = GraphClient::from_config(&test_config(&server.uri())).unwrap();
        let response = client
            .get_edge_all("act_1", "campaigns", &BTreeMap::new(), &[], Some(1))
            .await
            .unwrap();

        assert_eq!(response.data, json!([{ "id": "1" }]));
    }
}
