use crate::x_client::{XClient, XResponse};
use crate::x_error::XResult;

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
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        params.push((key.to_string(), value.to_string()));
    }
}

fn with_joined_values(params: &mut Vec<(String, String)>, key: &str, values: &[String]) {
    if !values.is_empty() {
        params.push((key.to_string(), values.join(",")));
    }
}

pub mod accounts {
    use super::*;

    pub async fn list_accounts(
        client: &XClient,
        account_ids: &[String],
        cursor: Option<&str>,
        count: Option<u32>,
        with_deleted: Option<bool>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> XResult<XResponse> {
        let mut params = Vec::new();
        with_joined_values(&mut params, "account_ids", account_ids);
        with_optional_string(&mut params, "cursor", cursor);
        with_optional_number(&mut params, "count", count);
        with_optional_bool(&mut params, "with_deleted", with_deleted);
        client
            .get_list("accounts", &params, fetch_all, max_items)
            .await
    }

    pub async fn get_account(
        client: &XClient,
        account_id: &str,
        with_deleted: Option<bool>,
    ) -> XResult<XResponse> {
        let mut params = Vec::new();
        with_optional_bool(&mut params, "with_deleted", with_deleted);
        client
            .get_object(&format!("accounts/{account_id}"), &params)
            .await
    }

    pub async fn get_authenticated_user_access(
        client: &XClient,
        account_id: &str,
    ) -> XResult<XResponse> {
        client
            .get_object(
                &format!("accounts/{account_id}/authenticated_user_access"),
                &[],
            )
            .await
    }
}

pub mod account_scoped {
    use super::*;

    pub async fn list_resource(
        client: &XClient,
        account_id: &str,
        resource_path: &str,
        params: &[(String, String)],
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> XResult<XResponse> {
        client
            .get_list(
                &format!("accounts/{account_id}/{resource_path}"),
                params,
                fetch_all,
                max_items,
            )
            .await
    }

    pub async fn get_resource(
        client: &XClient,
        account_id: &str,
        resource_path: &str,
        resource_id: &str,
        params: &[(String, String)],
    ) -> XResult<XResponse> {
        client
            .get_object(
                &format!("accounts/{account_id}/{resource_path}/{resource_id}"),
                params,
            )
            .await
    }
}

pub mod analytics {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct SyncAnalyticsQuery<'a> {
        pub account_id: &'a str,
        pub entity: &'a str,
        pub entity_ids: &'a [String],
        pub start_time: &'a str,
        pub end_time: &'a str,
        pub granularity: &'a str,
        pub placement: &'a str,
        pub metric_groups: &'a [String],
        pub country: Option<&'a str>,
        pub platform: Option<&'a str>,
    }

    #[derive(Debug, Clone)]
    pub struct ReachQuery<'a> {
        pub account_id: &'a str,
        pub level: &'a str,
        pub ids: &'a [String],
        pub start_time: &'a str,
        pub end_time: &'a str,
    }

    #[derive(Debug, Clone)]
    pub struct ActiveEntitiesQuery<'a> {
        pub account_id: &'a str,
        pub entity: &'a str,
        pub start_time: &'a str,
        pub end_time: &'a str,
        pub campaign_ids: &'a [String],
        pub funding_instrument_ids: &'a [String],
        pub line_item_ids: &'a [String],
    }

    #[derive(Debug, Clone)]
    pub struct AsyncJobQuery<'a> {
        pub account_id: &'a str,
        pub entity: &'a str,
        pub entity_ids: &'a [String],
        pub start_time: &'a str,
        pub end_time: &'a str,
        pub granularity: &'a str,
        pub placement: &'a str,
        pub metric_groups: &'a [String],
        pub segmentation_type: Option<&'a str>,
        pub country: Option<&'a str>,
        pub platform: Option<&'a str>,
    }

    pub async fn query_sync(client: &XClient, query: SyncAnalyticsQuery<'_>) -> XResult<XResponse> {
        let mut params = Vec::new();
        params.push(("entity".to_string(), query.entity.to_string()));
        params.push(("entity_ids".to_string(), query.entity_ids.join(",")));
        params.push(("start_time".to_string(), query.start_time.to_string()));
        params.push(("end_time".to_string(), query.end_time.to_string()));
        params.push(("granularity".to_string(), query.granularity.to_string()));
        params.push(("placement".to_string(), query.placement.to_string()));
        params.push(("metric_groups".to_string(), query.metric_groups.join(",")));
        with_optional_string(&mut params, "country", query.country);
        with_optional_string(&mut params, "platform", query.platform);

        client
            .get_object(&format!("stats/accounts/{}", query.account_id), &params)
            .await
    }

    pub async fn query_reach(client: &XClient, query: ReachQuery<'_>) -> XResult<XResponse> {
        let mut params = Vec::new();
        let key = match query.level {
            "campaigns" => "campaign_ids",
            "funding_instruments" => "funding_instrument_ids",
            _ => "entity_ids",
        };
        params.push((key.to_string(), query.ids.join(",")));
        params.push(("start_time".to_string(), query.start_time.to_string()));
        params.push(("end_time".to_string(), query.end_time.to_string()));

        client
            .get_object(
                &format!("stats/accounts/{}/reach/{}", query.account_id, query.level),
                &params,
            )
            .await
    }

    pub async fn query_active_entities(
        client: &XClient,
        query: ActiveEntitiesQuery<'_>,
    ) -> XResult<XResponse> {
        let mut params = Vec::new();
        params.push(("entity".to_string(), query.entity.to_string()));
        params.push(("start_time".to_string(), query.start_time.to_string()));
        params.push(("end_time".to_string(), query.end_time.to_string()));
        with_joined_values(&mut params, "campaign_ids", query.campaign_ids);
        with_joined_values(
            &mut params,
            "funding_instrument_ids",
            query.funding_instrument_ids,
        );
        with_joined_values(&mut params, "line_item_ids", query.line_item_ids);

        client
            .get_object(
                &format!("stats/accounts/{}/active_entities", query.account_id),
                &params,
            )
            .await
    }

    pub async fn submit_job(client: &XClient, query: AsyncJobQuery<'_>) -> XResult<XResponse> {
        let mut params = Vec::new();
        params.push(("entity".to_string(), query.entity.to_string()));
        params.push(("entity_ids".to_string(), query.entity_ids.join(",")));
        params.push(("start_time".to_string(), query.start_time.to_string()));
        params.push(("end_time".to_string(), query.end_time.to_string()));
        params.push(("granularity".to_string(), query.granularity.to_string()));
        params.push(("placement".to_string(), query.placement.to_string()));
        params.push(("metric_groups".to_string(), query.metric_groups.join(",")));
        with_optional_string(&mut params, "segmentation_type", query.segmentation_type);
        with_optional_string(&mut params, "country", query.country);
        with_optional_string(&mut params, "platform", query.platform);

        client
            .post_form(
                &format!("stats/jobs/accounts/{}", query.account_id),
                &params,
            )
            .await
    }

    pub async fn get_jobs(
        client: &XClient,
        account_id: &str,
        job_ids: &[String],
        cursor: Option<&str>,
        count: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> XResult<XResponse> {
        let mut params = Vec::new();
        with_joined_values(&mut params, "job_ids", job_ids);
        with_optional_string(&mut params, "cursor", cursor);
        with_optional_number(&mut params, "count", count);

        client
            .get_list(
                &format!("stats/jobs/accounts/{account_id}"),
                &params,
                fetch_all,
                max_items,
            )
            .await
    }
}
