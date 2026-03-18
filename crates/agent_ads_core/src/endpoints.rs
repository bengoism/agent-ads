use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::client::{GraphClient, GraphResponse};
use crate::error::{MetaAdsError, Result};
use crate::ids::normalize_account_id;

fn with_limit(params: &mut BTreeMap<String, String>, limit: Option<u32>) {
    if let Some(limit) = limit {
        params.insert("limit".to_string(), limit.to_string());
    }
}

fn with_after(params: &mut BTreeMap<String, String>, after: Option<&str>) {
    if let Some(after) = after {
        params.insert("after".to_string(), after.to_string());
    }
}

fn with_if_present(params: &mut BTreeMap<String, String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        params.insert(key.to_string(), value.to_string());
    }
}

fn with_csv(params: &mut BTreeMap<String, String>, key: &str, values: &[String]) {
    if !values.is_empty() {
        params.insert(key.to_string(), values.join(","));
    }
}

fn list_fields_or_default(fields: &[String], defaults: &[&str]) -> Vec<String> {
    if fields.is_empty() {
        defaults.iter().map(|value| value.to_string()).collect()
    } else {
        fields.to_vec()
    }
}

pub mod accounts {
    use super::*;

    pub async fn list_businesses(
        client: &GraphClient,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        let mut params = BTreeMap::new();
        with_limit(&mut params, limit);
        with_after(&mut params, after);
        let fields = list_fields_or_default(fields, &["id", "name", "verification_status"]);
        if fetch_all {
            client
                .get_edge_all("me", "businesses", &params, &fields, max_items)
                .await
        } else {
            client.get_edge("me", "businesses", &params, &fields).await
        }
    }

    pub async fn list_ad_accounts(
        client: &GraphClient,
        business_id: &str,
        scope: AdAccountScope,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        let mut params = BTreeMap::new();
        with_limit(&mut params, limit);
        with_after(&mut params, after);
        let fields = list_fields_or_default(
            fields,
            &[
                "id",
                "account_id",
                "name",
                "account_status",
                "currency",
                "timezone_name",
            ],
        );
        let edge = match scope {
            AdAccountScope::Accessible => "client_ad_accounts",
            AdAccountScope::Owned => "owned_ad_accounts",
            AdAccountScope::PendingClient => "pending_client_ad_accounts",
        };
        if fetch_all {
            client
                .get_edge_all(business_id, edge, &params, &fields, max_items)
                .await
        } else {
            client.get_edge(business_id, edge, &params, &fields).await
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum AdAccountScope {
        Accessible,
        Owned,
        PendingClient,
    }
}

pub mod objects {
    use super::*;

    pub async fn list_campaigns(
        client: &GraphClient,
        account_id: &str,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        list_account_edge(
            client,
            account_id,
            "campaigns",
            fields,
            &[
                "id",
                "name",
                "status",
                "effective_status",
                "objective",
                "created_time",
                "updated_time",
            ],
            limit,
            after,
            fetch_all,
            max_items,
        )
        .await
    }

    pub async fn list_adsets(
        client: &GraphClient,
        account_id: &str,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        list_account_edge(
            client,
            account_id,
            "adsets",
            fields,
            &[
                "id",
                "name",
                "campaign_id",
                "status",
                "effective_status",
                "daily_budget",
                "lifetime_budget",
                "billing_event",
            ],
            limit,
            after,
            fetch_all,
            max_items,
        )
        .await
    }

    pub async fn list_ads(
        client: &GraphClient,
        account_id: &str,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        list_account_edge(
            client,
            account_id,
            "ads",
            fields,
            &[
                "id",
                "name",
                "adset_id",
                "campaign_id",
                "status",
                "effective_status",
                "creative{id,name}",
            ],
            limit,
            after,
            fetch_all,
            max_items,
        )
        .await
    }

    async fn list_account_edge(
        client: &GraphClient,
        account_id: &str,
        edge: &str,
        fields: &[String],
        defaults: &[&str],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        let mut params = BTreeMap::new();
        with_limit(&mut params, limit);
        with_after(&mut params, after);
        let fields = list_fields_or_default(fields, defaults);
        let account_id = normalize_account_id(account_id)?;
        if fetch_all {
            client
                .get_edge_all(&account_id, edge, &params, &fields, max_items)
                .await
        } else {
            client.get_edge(&account_id, edge, &params, &fields).await
        }
    }
}

pub mod reports {
    use super::*;

    pub struct InsightsQuery<'a> {
        pub object_id: &'a str,
        pub level: Option<&'a str>,
        pub fields: &'a [String],
        pub date_preset: Option<&'a str>,
        pub since: Option<&'a str>,
        pub until: Option<&'a str>,
        pub time_increment: Option<&'a str>,
        pub breakdowns: &'a [String],
        pub action_breakdowns: &'a [String],
        pub sort: &'a [String],
        pub filtering: &'a [String],
        pub action_attribution_windows: &'a [String],
        pub limit: Option<u32>,
        pub after: Option<&'a str>,
        pub fetch_all: bool,
        pub max_items: Option<usize>,
    }

    pub async fn query_insights(
        client: &GraphClient,
        query: InsightsQuery<'_>,
    ) -> Result<GraphResponse> {
        if !query.action_breakdowns.is_empty()
            && !query.fields.iter().any(|field| field == "actions")
        {
            return Err(MetaAdsError::InvalidArgument(
                "the `actions` field is required when using action_breakdowns".to_string(),
            ));
        }

        let mut params = BTreeMap::new();
        with_if_present(&mut params, "level", query.level);
        with_if_present(&mut params, "date_preset", query.date_preset);
        with_if_present(&mut params, "time_increment", query.time_increment);
        with_csv(&mut params, "breakdowns", query.breakdowns);
        with_csv(&mut params, "action_breakdowns", query.action_breakdowns);
        with_csv(&mut params, "sort", query.sort);
        with_csv(
            &mut params,
            "action_attribution_windows",
            query.action_attribution_windows,
        );
        if !query.filtering.is_empty() {
            params.insert(
                "filtering".to_string(),
                format!("[{}]", query.filtering.join(",")),
            );
        }
        if let (Some(since), Some(until)) = (query.since, query.until) {
            params.insert(
                "time_range".to_string(),
                json!({ "since": since, "until": until }).to_string(),
            );
        }
        with_limit(&mut params, query.limit);
        with_after(&mut params, query.after);
        let fields = list_fields_or_default(
            query.fields,
            &[
                "account_id",
                "account_name",
                "campaign_id",
                "campaign_name",
                "impressions",
                "clicks",
                "spend",
            ],
        );
        if query.fetch_all {
            client
                .get_edge_all(
                    query.object_id,
                    "insights",
                    &params,
                    &fields,
                    query.max_items,
                )
                .await
        } else {
            client
                .get_edge(query.object_id, "insights", &params, &fields)
                .await
        }
    }

    pub async fn submit_report_run(
        client: &GraphClient,
        query: InsightsQuery<'_>,
    ) -> Result<GraphResponse> {
        if !query.action_breakdowns.is_empty()
            && !query.fields.iter().any(|field| field == "actions")
        {
            return Err(MetaAdsError::InvalidArgument(
                "the `actions` field is required when using action_breakdowns".to_string(),
            ));
        }

        let mut params = BTreeMap::new();
        with_if_present(&mut params, "level", query.level);
        with_if_present(&mut params, "date_preset", query.date_preset);
        with_if_present(&mut params, "time_increment", query.time_increment);
        with_csv(&mut params, "breakdowns", query.breakdowns);
        with_csv(&mut params, "action_breakdowns", query.action_breakdowns);
        with_csv(&mut params, "sort", query.sort);
        with_csv(
            &mut params,
            "action_attribution_windows",
            query.action_attribution_windows,
        );
        if !query.filtering.is_empty() {
            params.insert(
                "filtering".to_string(),
                format!("[{}]", query.filtering.join(",")),
            );
        }
        if let (Some(since), Some(until)) = (query.since, query.until) {
            params.insert(
                "time_range".to_string(),
                json!({ "since": since, "until": until }).to_string(),
            );
        }
        if !query.fields.is_empty() {
            params.insert("fields".to_string(), query.fields.join(","));
        }

        client
            .post_edge(query.object_id, "insights", &params, &[])
            .await
    }

    pub async fn get_report_run(
        client: &GraphClient,
        report_run_id: &str,
        fields: &[String],
    ) -> Result<GraphResponse> {
        let fields = list_fields_or_default(
            fields,
            &[
                "id",
                "async_status",
                "async_percent_completion",
                "date_start",
                "date_stop",
            ],
        );
        client
            .get_node(report_run_id, &BTreeMap::new(), &fields)
            .await
    }

    pub async fn get_report_run_results(
        client: &GraphClient,
        report_run_id: &str,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        let mut params = BTreeMap::new();
        with_limit(&mut params, limit);
        with_after(&mut params, after);
        let fields = list_fields_or_default(
            fields,
            &[
                "account_id",
                "campaign_id",
                "campaign_name",
                "impressions",
                "clicks",
                "spend",
            ],
        );
        if fetch_all {
            client
                .get_edge_all(report_run_id, "insights", &params, &fields, max_items)
                .await
        } else {
            client
                .get_edge(report_run_id, "insights", &params, &fields)
                .await
        }
    }
}

pub mod creative {
    use super::*;

    pub async fn get_creative(
        client: &GraphClient,
        creative_id: &str,
        fields: &[String],
    ) -> Result<GraphResponse> {
        let fields = list_fields_or_default(
            fields,
            &[
                "id",
                "name",
                "object_story_spec",
                "asset_feed_spec",
                "thumbnail_url",
            ],
        );
        client
            .get_node(creative_id, &BTreeMap::new(), &fields)
            .await
    }

    pub async fn get_creative_preview(
        client: &GraphClient,
        creative_id: &str,
        ad_format: Option<&str>,
        render_type: Option<&str>,
        fields: &[String],
    ) -> Result<GraphResponse> {
        let mut params = BTreeMap::new();
        with_if_present(&mut params, "ad_format", ad_format);
        with_if_present(&mut params, "render_type", render_type);
        let fields = list_fields_or_default(fields, &["body"]);
        client
            .get_edge(creative_id, "previews", &params, &fields)
            .await
    }

    pub async fn resolve_creative_id_from_ad(client: &GraphClient, ad_id: &str) -> Result<String> {
        let response = client
            .get_node(ad_id, &BTreeMap::new(), &[String::from("creative{id}")])
            .await?;
        response
            .data
            .get("creative")
            .and_then(|creative| creative.get("id"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                MetaAdsError::InvalidArgument(format!("ad `{ad_id}` did not return a creative id"))
            })
    }
}

pub mod changes {
    use super::*;

    pub async fn list_activities(
        client: &GraphClient,
        account_id: &str,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        since: Option<&str>,
        until: Option<&str>,
        category: Option<&str>,
        data_source: Option<&str>,
        oid: Option<&str>,
        business_id: Option<&str>,
        add_children: bool,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        let mut params = BTreeMap::new();
        with_limit(&mut params, limit);
        with_after(&mut params, after);
        with_if_present(&mut params, "since", since);
        with_if_present(&mut params, "until", until);
        with_if_present(&mut params, "category", category);
        with_if_present(&mut params, "data_source", data_source);
        with_if_present(&mut params, "oid", oid);
        with_if_present(&mut params, "business_id", business_id);
        if add_children {
            params.insert("add_children".to_string(), "true".to_string());
        }
        let fields = list_fields_or_default(
            fields,
            &[
                "id",
                "event_time",
                "event_type",
                "category",
                "object_type",
                "translated_event_type",
            ],
        );
        let account_id = normalize_account_id(account_id)?;
        if fetch_all {
            client
                .get_edge_all(&account_id, "activities", &params, &fields, max_items)
                .await
        } else {
            client
                .get_edge(&account_id, "activities", &params, &fields)
                .await
        }
    }
}

pub mod tracking {
    use super::*;

    pub async fn list_custom_conversions(
        client: &GraphClient,
        account_id: &str,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        let mut params = BTreeMap::new();
        with_limit(&mut params, limit);
        with_after(&mut params, after);
        let fields = list_fields_or_default(
            fields,
            &["id", "name", "custom_event_type", "rule", "creation_time"],
        );
        let account_id = normalize_account_id(account_id)?;
        if fetch_all {
            client
                .get_edge_all(
                    &account_id,
                    "customconversions",
                    &params,
                    &fields,
                    max_items,
                )
                .await
        } else {
            client
                .get_edge(&account_id, "customconversions", &params, &fields)
                .await
        }
    }

    pub async fn list_pixels(
        client: &GraphClient,
        account_id: &str,
        fields: &[String],
        limit: Option<u32>,
        after: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> Result<GraphResponse> {
        let mut params = BTreeMap::new();
        with_limit(&mut params, limit);
        with_after(&mut params, after);
        let fields = list_fields_or_default(
            fields,
            &[
                "id",
                "name",
                "owner_ad_account",
                "last_fired_time",
                "match_rate_approx",
                "event_stats",
            ],
        );
        let account_id = normalize_account_id(account_id)?;
        if fetch_all {
            client
                .get_edge_all(&account_id, "adspixels", &params, &fields, max_items)
                .await
        } else {
            client
                .get_edge(&account_id, "adspixels", &params, &fields)
                .await
        }
    }

    pub async fn get_dataset_quality(
        client: &GraphClient,
        dataset_id: &str,
        fields: &[String],
    ) -> Result<GraphResponse> {
        let fields = list_fields_or_default(
            fields,
            &[
                "id",
                "name",
                "event_stats",
                "last_fired_time",
                "match_rate_approx",
            ],
        );
        client.get_node(dataset_id, &BTreeMap::new(), &fields).await
    }

    pub async fn get_emq_diagnostics(
        client: &GraphClient,
        pixel_id: &str,
        pixel_fields: &[String],
        aggregation: Option<&str>,
        event: Option<&str>,
        event_source: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<GraphResponse> {
        let pixel_fields = list_fields_or_default(
            pixel_fields,
            &[
                "id",
                "name",
                "match_rate_approx",
                "event_stats",
                "last_fired_time",
            ],
        );
        let pixel = client
            .get_node(pixel_id, &BTreeMap::new(), &pixel_fields)
            .await?;
        let mut stats_params = BTreeMap::new();
        with_if_present(&mut stats_params, "aggregation", aggregation);
        with_if_present(&mut stats_params, "event", event);
        with_if_present(&mut stats_params, "event_source", event_source);
        with_if_present(&mut stats_params, "start_time", start_time);
        with_if_present(&mut stats_params, "end_time", end_time);
        let stats = client
            .get_edge(pixel_id, "stats", &stats_params, &[])
            .await?;

        Ok(GraphResponse {
            data: json!({
                "pixel": pixel.data,
                "stats": stats.data,
            }),
            paging: stats.paging,
            summary: stats.summary,
            request_id: stats.request_id.or(pixel.request_id),
        })
    }
}
