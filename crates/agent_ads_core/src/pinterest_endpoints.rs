use serde_json::Value;

use crate::pinterest_client::{PinterestClient, PinterestResponse};
use crate::pinterest_error::PinterestResult;

fn with_optional_bool(params: &mut Vec<(String, String)>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        params.push((key.to_string(), value.to_string()));
    }
}

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

fn with_required_string(params: &mut Vec<(String, String)>, key: &str, value: &str) {
    params.push((key.to_string(), value.to_string()));
}

fn with_repeated_values(params: &mut Vec<(String, String)>, key: &str, values: &[String]) {
    for value in values {
        params.push((key.to_string(), value.to_string()));
    }
}

pub mod accounts {
    use super::*;

    pub async fn list_ad_accounts(
        client: &PinterestClient,
        include_shared_accounts: Option<bool>,
        bookmark: Option<&str>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> PinterestResult<PinterestResponse> {
        let mut params = Vec::new();
        with_optional_bool(
            &mut params,
            "include_shared_accounts",
            include_shared_accounts,
        );
        with_optional_string(&mut params, "bookmark", bookmark);
        with_optional_number(&mut params, "page_size", page_size);

        client
            .get_list("ad_accounts", &params, fetch_all, max_items)
            .await
    }

    pub async fn get_ad_account(
        client: &PinterestClient,
        ad_account_id: &str,
    ) -> PinterestResult<PinterestResponse> {
        client
            .get_object(&format!("ad_accounts/{ad_account_id}"), &[])
            .await
    }
}

pub mod campaigns {
    use super::*;

    pub async fn list_campaigns(
        client: &PinterestClient,
        ad_account_id: &str,
        bookmark: Option<&str>,
        page_size: Option<u32>,
        order: Option<&str>,
        campaign_ids: &[String],
        entity_statuses: &[String],
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> PinterestResult<PinterestResponse> {
        let mut params = Vec::new();
        with_optional_string(&mut params, "bookmark", bookmark);
        with_optional_number(&mut params, "page_size", page_size);
        with_optional_string(&mut params, "order", order);
        with_repeated_values(&mut params, "campaign_ids", campaign_ids);
        with_repeated_values(&mut params, "entity_statuses", entity_statuses);

        client
            .get_list(
                &format!("ad_accounts/{ad_account_id}/campaigns"),
                &params,
                fetch_all,
                max_items,
            )
            .await
    }
}

pub mod adgroups {
    use super::*;

    pub async fn list_adgroups(
        client: &PinterestClient,
        ad_account_id: &str,
        bookmark: Option<&str>,
        page_size: Option<u32>,
        order: Option<&str>,
        campaign_ids: &[String],
        ad_group_ids: &[String],
        entity_statuses: &[String],
        translate_interests_to_names: Option<bool>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> PinterestResult<PinterestResponse> {
        let mut params = Vec::new();
        with_optional_string(&mut params, "bookmark", bookmark);
        with_optional_number(&mut params, "page_size", page_size);
        with_optional_string(&mut params, "order", order);
        with_repeated_values(&mut params, "campaign_ids", campaign_ids);
        with_repeated_values(&mut params, "ad_group_ids", ad_group_ids);
        with_repeated_values(&mut params, "entity_statuses", entity_statuses);
        with_optional_bool(
            &mut params,
            "translate_interests_to_names",
            translate_interests_to_names,
        );

        client
            .get_list(
                &format!("ad_accounts/{ad_account_id}/ad_groups"),
                &params,
                fetch_all,
                max_items,
            )
            .await
    }
}

pub mod ads {
    use super::*;

    pub async fn list_ads(
        client: &PinterestClient,
        ad_account_id: &str,
        bookmark: Option<&str>,
        page_size: Option<u32>,
        order: Option<&str>,
        campaign_ids: &[String],
        ad_group_ids: &[String],
        ad_ids: &[String],
        entity_statuses: &[String],
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> PinterestResult<PinterestResponse> {
        let mut params = Vec::new();
        with_optional_string(&mut params, "bookmark", bookmark);
        with_optional_number(&mut params, "page_size", page_size);
        with_optional_string(&mut params, "order", order);
        with_repeated_values(&mut params, "campaign_ids", campaign_ids);
        with_repeated_values(&mut params, "ad_group_ids", ad_group_ids);
        with_repeated_values(&mut params, "ad_ids", ad_ids);
        with_repeated_values(&mut params, "entity_statuses", entity_statuses);

        client
            .get_list(
                &format!("ad_accounts/{ad_account_id}/ads"),
                &params,
                fetch_all,
                max_items,
            )
            .await
    }
}

pub mod audiences {
    use super::*;

    pub async fn list_audiences(
        client: &PinterestClient,
        ad_account_id: &str,
        bookmark: Option<&str>,
        page_size: Option<u32>,
        order: Option<&str>,
        ownership_type: Option<&str>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> PinterestResult<PinterestResponse> {
        let mut params = Vec::new();
        with_optional_string(&mut params, "bookmark", bookmark);
        with_optional_number(&mut params, "page_size", page_size);
        with_optional_string(&mut params, "order", order);
        with_optional_string(&mut params, "ownership_type", ownership_type);

        client
            .get_list(
                &format!("ad_accounts/{ad_account_id}/audiences"),
                &params,
                fetch_all,
                max_items,
            )
            .await
    }

    pub async fn get_audience(
        client: &PinterestClient,
        ad_account_id: &str,
        audience_id: &str,
    ) -> PinterestResult<PinterestResponse> {
        client
            .get_object(
                &format!("ad_accounts/{ad_account_id}/audiences/{audience_id}"),
                &[],
            )
            .await
    }
}

pub mod analytics {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub enum AnalyticsLevel {
        AdAccount,
        Campaign,
        AdGroup,
        Ad,
        AdPin,
    }

    impl AnalyticsLevel {
        fn path(self) -> &'static str {
            match self {
                Self::AdAccount => "analytics",
                Self::Campaign => "campaigns/analytics",
                Self::AdGroup => "ad_groups/analytics",
                Self::Ad => "ads/analytics",
                Self::AdPin => "pins/analytics",
            }
        }
    }

    pub struct AnalyticsQuery<'a> {
        pub ad_account_id: &'a str,
        pub level: AnalyticsLevel,
        pub start_date: &'a str,
        pub end_date: &'a str,
        pub columns: &'a [String],
        pub granularity: &'a str,
        pub campaign_ids: &'a [String],
        pub ad_group_ids: &'a [String],
        pub ad_ids: &'a [String],
        pub pin_ids: &'a [String],
        pub campaign_id: Option<&'a str>,
        pub click_window_days: Option<u32>,
        pub engagement_window_days: Option<u32>,
        pub view_window_days: Option<u32>,
        pub conversion_report_time: Option<&'a str>,
        pub reporting_timezone: Option<&'a str>,
        pub aggregate_report_rows: Option<bool>,
    }

    pub async fn query_analytics(
        client: &PinterestClient,
        query: AnalyticsQuery<'_>,
    ) -> PinterestResult<PinterestResponse> {
        let mut params = Vec::new();
        with_required_string(&mut params, "start_date", query.start_date);
        with_required_string(&mut params, "end_date", query.end_date);
        with_repeated_values(&mut params, "columns", query.columns);
        with_required_string(&mut params, "granularity", query.granularity);
        with_repeated_values(&mut params, "campaign_ids", query.campaign_ids);
        with_repeated_values(&mut params, "ad_group_ids", query.ad_group_ids);
        with_repeated_values(&mut params, "ad_ids", query.ad_ids);
        with_repeated_values(&mut params, "pin_ids", query.pin_ids);
        with_optional_string(&mut params, "campaign_id", query.campaign_id);
        with_optional_number(&mut params, "click_window_days", query.click_window_days);
        with_optional_number(
            &mut params,
            "engagement_window_days",
            query.engagement_window_days,
        );
        with_optional_number(&mut params, "view_window_days", query.view_window_days);
        with_optional_string(
            &mut params,
            "conversion_report_time",
            query.conversion_report_time,
        );
        with_optional_string(&mut params, "reporting_timezone", query.reporting_timezone);
        with_optional_bool(
            &mut params,
            "aggregate_report_rows",
            query.aggregate_report_rows,
        );

        client
            .get_object(
                &format!("ad_accounts/{}/{}", query.ad_account_id, query.level.path()),
                &params,
            )
            .await
    }
}

pub mod targeting {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub enum TargetingLevel {
        AdAccount,
        AdGroup,
        Ad,
    }

    impl TargetingLevel {
        fn path(self) -> &'static str {
            match self {
                Self::AdAccount => "targeting_analytics",
                Self::AdGroup => "ad_groups/targeting_analytics",
                Self::Ad => "ads/targeting_analytics",
            }
        }
    }

    pub struct TargetingAnalyticsQuery<'a> {
        pub ad_account_id: &'a str,
        pub level: TargetingLevel,
        pub start_date: &'a str,
        pub end_date: &'a str,
        pub targeting_types: &'a [String],
        pub columns: &'a [String],
        pub granularity: &'a str,
        pub ad_group_ids: &'a [String],
        pub ad_ids: &'a [String],
        pub click_window_days: Option<u32>,
        pub engagement_window_days: Option<u32>,
        pub view_window_days: Option<u32>,
        pub conversion_report_time: Option<&'a str>,
        pub attribution_types: &'a [String],
        pub reporting_timezone: Option<&'a str>,
        pub sort_columns: &'a [String],
        pub sort_ascending: Option<bool>,
    }

    pub async fn query_targeting_analytics(
        client: &PinterestClient,
        query: TargetingAnalyticsQuery<'_>,
    ) -> PinterestResult<PinterestResponse> {
        let mut params = Vec::new();
        with_required_string(&mut params, "start_date", query.start_date);
        with_required_string(&mut params, "end_date", query.end_date);
        with_repeated_values(&mut params, "targeting_types", query.targeting_types);
        with_repeated_values(&mut params, "columns", query.columns);
        with_required_string(&mut params, "granularity", query.granularity);
        with_repeated_values(&mut params, "ad_group_ids", query.ad_group_ids);
        with_repeated_values(&mut params, "ad_ids", query.ad_ids);
        with_optional_number(&mut params, "click_window_days", query.click_window_days);
        with_optional_number(
            &mut params,
            "engagement_window_days",
            query.engagement_window_days,
        );
        with_optional_number(&mut params, "view_window_days", query.view_window_days);
        with_optional_string(
            &mut params,
            "conversion_report_time",
            query.conversion_report_time,
        );
        with_repeated_values(&mut params, "attribution_types", query.attribution_types);
        with_optional_string(&mut params, "reporting_timezone", query.reporting_timezone);
        with_repeated_values(&mut params, "sort_columns", query.sort_columns);
        with_optional_bool(&mut params, "sort_ascending", query.sort_ascending);

        client
            .get_object(
                &format!("ad_accounts/{}/{}", query.ad_account_id, query.level.path()),
                &params,
            )
            .await
    }
}

pub mod reports {
    use super::*;

    pub async fn create_report(
        client: &PinterestClient,
        ad_account_id: &str,
        body: &Value,
    ) -> PinterestResult<PinterestResponse> {
        client
            .post_json(&format!("ad_accounts/{ad_account_id}/reports"), body)
            .await
    }

    pub async fn get_report(
        client: &PinterestClient,
        ad_account_id: &str,
        token: &str,
    ) -> PinterestResult<PinterestResponse> {
        let params = vec![("token".to_string(), token.to_string())];
        client
            .get_object(&format!("ad_accounts/{ad_account_id}/reports"), &params)
            .await
    }
}
