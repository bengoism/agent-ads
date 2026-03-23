use serde_json::Value;

use crate::linkedin_client::{encode_path_segment, LinkedInClient, LinkedInResponse};
use crate::linkedin_error::LinkedInResult;

fn with_optional_number<T: ToString>(
    params: &mut Vec<(String, String)>,
    key: &str,
    value: Option<T>,
) {
    if let Some(value) = value {
        params.push((key.to_string(), value.to_string()));
    }
}

fn with_optional_string(params: &mut Vec<(String, String)>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        params.push((key.to_string(), value.to_string()));
    }
}

pub mod accounts {
    use super::*;

    pub async fn list_accessible_account_users(
        client: &LinkedInClient,
    ) -> LinkedInResult<LinkedInResponse> {
        client
            .get_object(
                "adAccountUsers",
                &[("q".to_string(), "authenticatedUser".to_string())],
                &[],
            )
            .await
    }

    pub async fn get_account(
        client: &LinkedInClient,
        account_id: &str,
    ) -> LinkedInResult<LinkedInResponse> {
        client
            .get_object(&format!("adAccounts/{account_id}"), &[], &[])
            .await
    }

    pub async fn search_accounts(
        client: &LinkedInClient,
        search: Option<&str>,
        sort: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> LinkedInResult<LinkedInResponse> {
        let mut params = vec![("q".to_string(), "search".to_string())];
        with_optional_string(&mut params, "search", search);
        with_optional_string(&mut params, "sort", sort);
        with_optional_number(&mut params, "pageSize", page_size);
        with_optional_string(&mut params, "pageToken", page_token);

        client
            .get_list("adAccounts", &params, &[], fetch_all, max_items)
            .await
    }
}

pub mod campaign_groups {
    use super::*;

    pub async fn list_campaign_groups(
        client: &LinkedInClient,
        account_id: &str,
        search: &str,
        sort_order: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> LinkedInResult<LinkedInResponse> {
        let mut params = vec![("q".to_string(), "search".to_string())];
        params.push(("search".to_string(), search.to_string()));
        with_optional_string(&mut params, "sortOrder", sort_order);
        with_optional_number(&mut params, "pageSize", page_size);
        with_optional_string(&mut params, "pageToken", page_token);

        client
            .get_list(
                &format!("adAccounts/{account_id}/adCampaignGroups"),
                &params,
                &[],
                fetch_all,
                max_items,
            )
            .await
    }
}

pub mod campaigns {
    use super::*;

    pub async fn list_campaigns(
        client: &LinkedInClient,
        account_id: &str,
        search: &str,
        sort_order: Option<&str>,
        page_token: Option<&str>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> LinkedInResult<LinkedInResponse> {
        let mut params = vec![("q".to_string(), "search".to_string())];
        params.push(("search".to_string(), search.to_string()));
        with_optional_string(&mut params, "sortOrder", sort_order);
        with_optional_number(&mut params, "pageSize", page_size);
        with_optional_string(&mut params, "pageToken", page_token);

        client
            .get_list(
                &format!("adAccounts/{account_id}/adCampaigns"),
                &params,
                &[],
                fetch_all,
                max_items,
            )
            .await
    }

    pub async fn get_campaign(
        client: &LinkedInClient,
        account_id: &str,
        campaign_id: &str,
    ) -> LinkedInResult<LinkedInResponse> {
        client
            .get_object(
                &format!("adAccounts/{account_id}/adCampaigns/{campaign_id}"),
                &[],
                &[],
            )
            .await
    }
}

pub mod creatives {
    use super::*;

    pub async fn list_creatives(
        client: &LinkedInClient,
        account_id: &str,
        params: &[(String, String)],
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> LinkedInResult<LinkedInResponse> {
        client
            .get_list(
                &format!("adAccounts/{account_id}/creatives"),
                params,
                &[("X-RestLi-Method", "FINDER")],
                fetch_all,
                max_items,
            )
            .await
    }

    pub async fn get_creative(
        client: &LinkedInClient,
        account_id: &str,
        creative_urn: &str,
    ) -> LinkedInResult<LinkedInResponse> {
        let encoded_creative = encode_path_segment(creative_urn);
        client
            .get_object(
                &format!("adAccounts/{account_id}/creatives/{encoded_creative}"),
                &[],
                &[],
            )
            .await
    }
}

pub mod reports {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct AnalyticsQuery<'a> {
        pub finder: &'a str,
        pub pivots: &'a [String],
        pub time_granularity: Option<&'a str>,
        pub date_range: &'a str,
        pub account: &'a str,
        pub campaign_ids: &'a [String],
        pub campaign_group_ids: &'a [String],
        pub creative_ids: &'a [String],
        pub fields: &'a [String],
    }

    pub async fn query_analytics(
        client: &LinkedInClient,
        query: AnalyticsQuery<'_>,
    ) -> LinkedInResult<LinkedInResponse> {
        let mut params = vec![("q".to_string(), query.finder.to_string())];
        params.push(("dateRange".to_string(), query.date_range.to_string()));

        match query.finder {
            "analytics" => {
                if let Some(pivot) = query.pivots.first() {
                    params.push(("pivot".to_string(), pivot.to_string()));
                }
                with_optional_string(&mut params, "timeGranularity", query.time_granularity);
                params.push(("accounts".to_string(), format!("List({})", query.account)));
            }
            "statistics" => {
                params.push((
                    "pivots".to_string(),
                    format!("List({})", query.pivots.join(",")),
                ));
                with_optional_string(&mut params, "timeGranularity", query.time_granularity);
                params.push(("accounts".to_string(), format!("List({})", query.account)));
            }
            "attributedRevenueMetrics" => {
                params.push((
                    "pivots".to_string(),
                    format!("List({})", query.pivots.join(",")),
                ));
                params.push(("account".to_string(), format!("List({})", query.account)));
            }
            _ => {}
        }

        if !query.campaign_ids.is_empty() {
            params.push((
                "campaigns".to_string(),
                format!("List({})", query.campaign_ids.join(",")),
            ));
        }
        if !query.campaign_group_ids.is_empty() {
            params.push((
                "campaignGroups".to_string(),
                format!("List({})", query.campaign_group_ids.join(",")),
            ));
        }
        if !query.creative_ids.is_empty() {
            params.push((
                "creatives".to_string(),
                format!("List({})", query.creative_ids.join(",")),
            ));
        }
        if !query.fields.is_empty() {
            params.push(("fields".to_string(), query.fields.join(",")));
        }

        client.get_object("adAnalytics", &params, &[]).await
    }

    pub fn extract_elements(value: &Value) -> Vec<Value> {
        value
            .get("elements")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }
}
