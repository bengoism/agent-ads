use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::client::Paging;
use crate::error::{MetaAdsError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Jsonl,
    Csv,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Jsonl => write!(f, "jsonl"),
            Self::Csv => write!(f, "csv"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = MetaAdsError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "json" => Ok(Self::Json),
            "jsonl" => Ok(Self::Jsonl),
            "csv" => Ok(Self::Csv),
            other => Err(MetaAdsError::InvalidArgument(format!(
                "unsupported output format `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputMeta {
    pub api_version: String,
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputEnvelope {
    pub data: Value,
    pub meta: OutputMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paging: Option<Paging>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl OutputEnvelope {
    pub fn new(data: Value, meta: OutputMeta) -> Self {
        Self {
            data,
            meta,
            paging: None,
            warnings: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RenderOptions {
    pub pretty: bool,
    pub envelope: bool,
    pub include_meta: bool,
}

pub fn render_output(
    envelope: &OutputEnvelope,
    format: OutputFormat,
    options: RenderOptions,
) -> Result<String> {
    if options.envelope {
        return match format {
            OutputFormat::Json => {
                if options.pretty {
                    Ok(serde_json::to_string_pretty(envelope)?)
                } else {
                    Ok(serde_json::to_string(envelope)?)
                }
            }
            OutputFormat::Jsonl => render_jsonl_envelope(envelope),
            OutputFormat::Csv => render_csv(envelope, true),
        };
    }

    match format {
        OutputFormat::Json => {
            let payload = if options.include_meta {
                json!({
                    "data": envelope.data,
                    "meta": envelope.meta,
                })
            } else {
                envelope.data.clone()
            };
            if options.pretty {
                Ok(serde_json::to_string_pretty(&payload)?)
            } else {
                Ok(serde_json::to_string(&payload)?)
            }
        }
        OutputFormat::Jsonl => render_jsonl_data(envelope, options.include_meta),
        OutputFormat::Csv => render_csv(envelope, options.include_meta),
    }
}

fn render_jsonl_envelope(envelope: &OutputEnvelope) -> Result<String> {
    match &envelope.data {
        Value::Array(items) => {
            let lines = items
                .iter()
                .map(|item| {
                    serde_json::to_string(&json!({
                        "data": item,
                        "meta": envelope.meta,
                        "paging": envelope.paging,
                        "warnings": envelope.warnings,
                    }))
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(lines.join("\n"))
        }
        _ => Ok(serde_json::to_string(envelope)?),
    }
}

fn render_jsonl_data(envelope: &OutputEnvelope, include_meta: bool) -> Result<String> {
    match &envelope.data {
        Value::Array(items) => {
            let lines = items
                .iter()
                .map(|item| {
                    if include_meta {
                        serde_json::to_string(&json!({
                            "data": item,
                            "meta": envelope.meta,
                        }))
                    } else {
                        serde_json::to_string(item)
                    }
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(lines.join("\n"))
        }
        value => {
            if include_meta {
                Ok(serde_json::to_string(&json!({
                    "data": value,
                    "meta": envelope.meta,
                }))?)
            } else {
                Ok(serde_json::to_string(value)?)
            }
        }
    }
}

fn render_csv(envelope: &OutputEnvelope, include_meta: bool) -> Result<String> {
    let rows = csv_rows(envelope, include_meta);
    let mut headers = BTreeSet::new();
    for row in &rows {
        headers.extend(row.keys().cloned());
    }

    let mut writer = csv::Writer::from_writer(Vec::new());
    let ordered_headers = headers.into_iter().collect::<Vec<_>>();
    writer.write_record(&ordered_headers)?;
    for row in rows {
        let record = ordered_headers
            .iter()
            .map(|header| row.get(header).cloned().unwrap_or_default())
            .collect::<Vec<_>>();
        writer.write_record(record)?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|error| MetaAdsError::Io(error.into_error()))?;
    String::from_utf8(bytes).map_err(|error| MetaAdsError::Config(error.to_string()))
}

fn csv_rows(envelope: &OutputEnvelope, include_meta: bool) -> Vec<BTreeMap<String, String>> {
    match &envelope.data {
        Value::Array(items) => items
            .iter()
            .map(|item| row_from_value(item, envelope, include_meta))
            .collect::<Vec<_>>(),
        value => vec![row_from_value(value, envelope, include_meta)],
    }
}

fn row_from_value(
    value: &Value,
    envelope: &OutputEnvelope,
    include_meta: bool,
) -> BTreeMap<String, String> {
    let mut row = BTreeMap::new();
    if include_meta {
        row.insert(
            "meta.api_version".to_string(),
            envelope.meta.api_version.clone(),
        );
        row.insert("meta.endpoint".to_string(), envelope.meta.endpoint.clone());
        if let Some(object_id) = &envelope.meta.object_id {
            row.insert("meta.object_id".to_string(), object_id.clone());
        }
        if let Some(request_id) = &envelope.meta.request_id {
            row.insert("meta.request_id".to_string(), request_id.clone());
        }
        if let Some(report_run_id) = &envelope.meta.report_run_id {
            row.insert("meta.report_run_id".to_string(), report_run_id.clone());
        }
    }
    flatten_json("data", value, &mut row);
    row
}

fn flatten_json(prefix: &str, value: &Value, output: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let next_prefix = format!("{prefix}.{key}");
                flatten_json(&next_prefix, value, output);
            }
        }
        Value::Array(_) => {
            output.insert(prefix.to_string(), value.to_string());
        }
        Value::Null => {
            output.insert(prefix.to_string(), String::new());
        }
        Value::Bool(boolean) => {
            output.insert(prefix.to_string(), boolean.to_string());
        }
        Value::Number(number) => {
            output.insert(prefix.to_string(), number.to_string());
        }
        Value::String(string) => {
            output.insert(prefix.to_string(), string.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{render_output, OutputEnvelope, OutputFormat, OutputMeta, RenderOptions};

    fn envelope() -> OutputEnvelope {
        OutputEnvelope::new(
            json!([{ "id": "1", "name": "Campaign" }]),
            OutputMeta {
                api_version: "v25.0".to_string(),
                endpoint: "/act_1/campaigns".to_string(),
                object_id: Some("act_1".to_string()),
                request_id: Some("req-1".to_string()),
                report_run_id: None,
            },
        )
    }

    #[test]
    fn renders_jsonl() {
        let output =
            render_output(&envelope(), OutputFormat::Jsonl, RenderOptions::default()).unwrap();
        assert_eq!(output.trim(), "{\"id\":\"1\",\"name\":\"Campaign\"}");
    }

    #[test]
    fn renders_csv_without_meta_by_default() {
        let output =
            render_output(&envelope(), OutputFormat::Csv, RenderOptions::default()).unwrap();
        assert!(output.contains("data.id"));
        assert!(!output.contains("meta.api_version"));
    }

    #[test]
    fn renders_csv_headers_with_meta() {
        let output = render_output(
            &envelope(),
            OutputFormat::Csv,
            RenderOptions {
                include_meta: true,
                ..RenderOptions::default()
            },
        )
        .unwrap();
        assert!(output.contains("data.id"));
        assert!(output.contains("meta.api_version"));
    }

    #[test]
    fn renders_envelope_when_requested() {
        let output = render_output(
            &envelope(),
            OutputFormat::Json,
            RenderOptions {
                envelope: true,
                ..RenderOptions::default()
            },
        )
        .unwrap();
        assert!(output.contains("\"meta\""));
        assert!(output.contains("\"data\""));
    }
}
