use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::tiktok_client::{TikTokClient, TikTokResponse};
use crate::tiktok_error::TikTokResult;

fn with_required(params: &mut BTreeMap<String, String>, key: &str, value: &str) {
    params.insert(key.to_string(), value.to_string());
}

fn with_optional(params: &mut BTreeMap<String, String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        params.insert(key.to_string(), value.to_string());
    }
}

fn with_page(params: &mut BTreeMap<String, String>, page: Option<u32>) {
    if let Some(page) = page {
        params.insert("page".to_string(), page.to_string());
    }
}

fn with_page_size(params: &mut BTreeMap<String, String>, page_size: Option<u32>) {
    if let Some(page_size) = page_size {
        params.insert("page_size".to_string(), page_size.to_string());
    }
}

fn with_json_array_param(params: &mut BTreeMap<String, String>, key: &str, values: &[String]) {
    if !values.is_empty() {
        let json_arr = serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string());
        params.insert(key.to_string(), json_arr);
    }
}

fn with_json_param(params: &mut BTreeMap<String, String>, key: &str, value: &Value) {
    if !value.is_null() {
        params.insert(key.to_string(), value.to_string());
    }
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

pub mod accounts {
    use super::*;

    /// GET /oauth2/advertiser/get/ — list authorized advertisers
    /// Note: requires app_id and secret as query params, not just access token.
    /// For simplicity, we list advertisers using /advertiser/info/ with known IDs,
    /// or use the OAuth endpoint when app credentials are available.
    pub async fn list_advertisers(
        client: &TikTokClient,
        app_id: &str,
        app_secret: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "app_id", app_id);
        with_required(&mut params, "secret", app_secret);
        with_page(&mut params, page);
        with_page_size(&mut params, page_size);

        if fetch_all {
            client
                .get_all("oauth2/advertiser/get", &params, max_items)
                .await
        } else {
            client.get("oauth2/advertiser/get", &params).await
        }
    }

    /// GET /advertiser/info/ — get advertiser account details
    pub async fn get_advertiser_info(
        client: &TikTokClient,
        advertiser_ids: &[String],
        fields: &[String],
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_json_array_param(&mut params, "advertiser_ids", advertiser_ids);
        with_json_array_param(&mut params, "fields", fields);

        client.get("advertiser/info", &params).await
    }
}

// ---------------------------------------------------------------------------
// Campaigns
// ---------------------------------------------------------------------------

pub mod campaigns {
    use super::*;

    /// GET /campaign/get/ — list campaigns
    pub async fn list_campaigns(
        client: &TikTokClient,
        advertiser_id: &str,
        fields: &[String],
        filtering: Option<&Value>,
        page: Option<u32>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", advertiser_id);
        with_json_array_param(&mut params, "fields", fields);
        if let Some(filtering) = filtering {
            with_json_param(&mut params, "filtering", filtering);
        }
        with_page(&mut params, page);
        with_page_size(&mut params, page_size);

        if fetch_all {
            client.get_all("campaign/get", &params, max_items).await
        } else {
            client.get("campaign/get", &params).await
        }
    }
}

// ---------------------------------------------------------------------------
// Ad Groups
// ---------------------------------------------------------------------------

pub mod adgroups {
    use super::*;

    /// GET /adgroup/get/ — list ad groups
    pub async fn list_adgroups(
        client: &TikTokClient,
        advertiser_id: &str,
        fields: &[String],
        filtering: Option<&Value>,
        page: Option<u32>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", advertiser_id);
        with_json_array_param(&mut params, "fields", fields);
        if let Some(filtering) = filtering {
            with_json_param(&mut params, "filtering", filtering);
        }
        with_page(&mut params, page);
        with_page_size(&mut params, page_size);

        if fetch_all {
            client.get_all("adgroup/get", &params, max_items).await
        } else {
            client.get("adgroup/get", &params).await
        }
    }
}

// ---------------------------------------------------------------------------
// Ads
// ---------------------------------------------------------------------------

pub mod ads {
    use super::*;

    /// GET /ad/get/ — list ads
    pub async fn list_ads(
        client: &TikTokClient,
        advertiser_id: &str,
        fields: &[String],
        filtering: Option<&Value>,
        page: Option<u32>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", advertiser_id);
        with_json_array_param(&mut params, "fields", fields);
        if let Some(filtering) = filtering {
            with_json_param(&mut params, "filtering", filtering);
        }
        with_page(&mut params, page);
        with_page_size(&mut params, page_size);

        if fetch_all {
            client.get_all("ad/get", &params, max_items).await
        } else {
            client.get("ad/get", &params).await
        }
    }
}

// ---------------------------------------------------------------------------
// Reports / Insights
// ---------------------------------------------------------------------------

pub mod reports {
    use super::*;

    /// Synchronous reporting parameters.
    pub struct TikTokInsightsQuery<'a> {
        pub advertiser_id: &'a str,
        pub report_type: &'a str,
        pub data_level: Option<&'a str>,
        pub dimensions: &'a [String],
        pub metrics: &'a [String],
        pub start_date: Option<&'a str>,
        pub end_date: Option<&'a str>,
        pub filtering: Option<&'a Value>,
        pub order_field: Option<&'a str>,
        pub order_type: Option<&'a str>,
        pub query_lifetime: Option<bool>,
        pub page: Option<u32>,
        pub page_size: Option<u32>,
        pub fetch_all: bool,
        pub max_items: Option<usize>,
    }

    /// GET /report/integrated/get/ — synchronous reporting
    pub async fn query_insights(
        client: &TikTokClient,
        query: TikTokInsightsQuery<'_>,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", query.advertiser_id);
        with_required(&mut params, "report_type", query.report_type);
        with_optional(&mut params, "data_level", query.data_level);
        with_json_array_param(&mut params, "dimensions", query.dimensions);
        with_json_array_param(&mut params, "metrics", query.metrics);
        with_optional(&mut params, "start_date", query.start_date);
        with_optional(&mut params, "end_date", query.end_date);
        if let Some(filtering) = query.filtering {
            with_json_param(&mut params, "filtering", filtering);
        }
        with_optional(&mut params, "order_field", query.order_field);
        with_optional(&mut params, "order_type", query.order_type);
        if let Some(true) = query.query_lifetime {
            params.insert("query_lifetime".to_string(), "true".to_string());
        }
        with_page(&mut params, query.page);
        with_page_size(&mut params, query.page_size);

        if query.fetch_all {
            client
                .get_all("report/integrated/get", &params, query.max_items)
                .await
        } else {
            client.get("report/integrated/get", &params).await
        }
    }

    /// POST /report/task/create/ — create an async report task
    pub async fn create_report_task(
        client: &TikTokClient,
        advertiser_id: &str,
        report_type: &str,
        data_level: Option<&str>,
        dimensions: &[String],
        metrics: &[String],
        start_date: Option<&str>,
        end_date: Option<&str>,
        filtering: Option<&Value>,
    ) -> TikTokResult<TikTokResponse> {
        let mut body = json!({
            "advertiser_id": advertiser_id,
            "report_type": report_type,
            "dimensions": dimensions,
            "metrics": metrics,
        });

        if let Some(data_level) = data_level {
            body["data_level"] = json!(data_level);
        }
        if let Some(start_date) = start_date {
            body["start_date"] = json!(start_date);
        }
        if let Some(end_date) = end_date {
            body["end_date"] = json!(end_date);
        }
        if let Some(filtering) = filtering {
            body["filtering"] = filtering.clone();
        }

        client.post("report/task/create", &body).await
    }

    /// GET /report/task/check/ — check async report task status
    pub async fn check_report_task(
        client: &TikTokClient,
        advertiser_id: &str,
        task_id: &str,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", advertiser_id);
        with_required(&mut params, "task_id", task_id);

        client.get("report/task/check", &params).await
    }

    /// POST /report/task/cancel/ — cancel an async report task
    pub async fn cancel_report_task(
        client: &TikTokClient,
        advertiser_id: &str,
        task_id: &str,
    ) -> TikTokResult<TikTokResponse> {
        let body = json!({
            "advertiser_id": advertiser_id,
            "task_id": task_id,
        });

        client.post("report/task/cancel", &body).await
    }
}

// ---------------------------------------------------------------------------
// Creative / Files
// ---------------------------------------------------------------------------

pub mod creative {
    use super::*;

    /// GET /file/video/ad/search/ — search video assets
    pub async fn search_videos(
        client: &TikTokClient,
        advertiser_id: &str,
        filtering: Option<&Value>,
        page: Option<u32>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", advertiser_id);
        if let Some(filtering) = filtering {
            with_json_param(&mut params, "filtering", filtering);
        }
        with_page(&mut params, page);
        with_page_size(&mut params, page_size);

        if fetch_all {
            client
                .get_all("file/video/ad/search", &params, max_items)
                .await
        } else {
            client.get("file/video/ad/search", &params).await
        }
    }

    /// GET /file/image/ad/info/ — get image info
    pub async fn get_images(
        client: &TikTokClient,
        advertiser_id: &str,
        image_ids: &[String],
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", advertiser_id);
        with_json_array_param(&mut params, "image_ids", image_ids);

        client.get("file/image/ad/info", &params).await
    }
}

// ---------------------------------------------------------------------------
// Pixels
// ---------------------------------------------------------------------------

pub mod pixels {
    use super::*;

    /// GET /pixel/list/ — list pixels
    pub async fn list_pixels(
        client: &TikTokClient,
        advertiser_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", advertiser_id);
        with_page(&mut params, page);
        with_page_size(&mut params, page_size);

        if fetch_all {
            client.get_all("pixel/list", &params, max_items).await
        } else {
            client.get("pixel/list", &params).await
        }
    }
}

// ---------------------------------------------------------------------------
// Audiences
// ---------------------------------------------------------------------------

pub mod audiences {
    use super::*;

    /// GET /dmp/custom_audience/list/ — list custom audiences
    pub async fn list_audiences(
        client: &TikTokClient,
        advertiser_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        fetch_all: bool,
        max_items: Option<usize>,
    ) -> TikTokResult<TikTokResponse> {
        let mut params = BTreeMap::new();
        with_required(&mut params, "advertiser_id", advertiser_id);
        with_page(&mut params, page);
        with_page_size(&mut params, page_size);

        if fetch_all {
            client
                .get_all("dmp/custom_audience/list", &params, max_items)
                .await
        } else {
            client.get("dmp/custom_audience/list", &params).await
        }
    }
}
