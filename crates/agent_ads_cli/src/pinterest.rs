use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use agent_ads_core::output::{OutputEnvelope, OutputMeta};
use agent_ads_core::pinterest_config::{
    pinterest_inspect_auth, PinterestAuthSnapshot, PinterestConfigOverrides,
    PinterestConfigSnapshot, PinterestResolvedConfig, PinterestSecretSource, PinterestSecretStatus,
    PINTEREST_DEFAULT_API_VERSION,
};
use agent_ads_core::pinterest_endpoints::{
    accounts, adgroups, ads, analytics as pinterest_analytics, audiences, campaigns, reports,
    targeting as pinterest_targeting,
};
use agent_ads_core::secret_store::SecretStore;
use agent_ads_core::{
    pinterest_refresh_access_token, PinterestClient, PinterestError, PinterestResponse,
    PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT, PINTEREST_ADS_ACCESS_TOKEN_SERVICE,
    PINTEREST_ADS_APP_ID_ACCOUNT, PINTEREST_ADS_APP_ID_SERVICE, PINTEREST_ADS_APP_SECRET_ACCOUNT,
    PINTEREST_ADS_APP_SECRET_SERVICE, PINTEREST_ADS_REFRESH_TOKEN_ACCOUNT,
    PINTEREST_ADS_REFRESH_TOKEN_SERVICE,
};
use clap::{Args, Subcommand, ValueEnum};
use rpassword::prompt_password;
use serde_json::{json, Map, Value};
use tokio::time::sleep;

use crate::{command_result, read_input, CommandResult};

const PINTEREST_ADS_APP_ID_ENV_VAR: &str = "PINTEREST_ADS_APP_ID";
const PINTEREST_ADS_APP_SECRET_ENV_VAR: &str = "PINTEREST_ADS_APP_SECRET";
const PINTEREST_ADS_ACCESS_TOKEN_ENV_VAR: &str = "PINTEREST_ADS_ACCESS_TOKEN";
const PINTEREST_ADS_REFRESH_TOKEN_ENV_VAR: &str = "PINTEREST_ADS_REFRESH_TOKEN";

// ---------------------------------------------------------------------------
// Clap subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum PinterestCommand {
    #[command(about = "List and inspect Pinterest ad accounts")]
    AdAccounts {
        #[command(subcommand)]
        command: AdAccountsCommand,
    },
    #[command(about = "List campaigns")]
    Campaigns {
        #[command(subcommand)]
        command: CampaignsCommand,
    },
    #[command(about = "List ad groups")]
    Adgroups {
        #[command(subcommand)]
        command: AdgroupsCommand,
    },
    #[command(about = "List ads")]
    Ads {
        #[command(subcommand)]
        command: AdsCommand,
    },
    #[command(about = "Query Pinterest analytics synchronously")]
    Analytics {
        #[command(subcommand)]
        command: AnalyticsCommand,
    },
    #[command(about = "Manage async Pinterest report runs")]
    ReportRuns {
        #[command(subcommand)]
        command: ReportRunsCommand,
    },
    #[command(about = "List and inspect audiences")]
    Audiences {
        #[command(subcommand)]
        command: AudiencesCommand,
    },
    #[command(about = "Query targeting analytics")]
    TargetingAnalytics {
        #[command(subcommand)]
        command: TargetingAnalyticsCommand,
    },
    #[command(about = "Manage stored Pinterest auth credentials")]
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    #[command(about = "Verify auth, config, and API connectivity")]
    Doctor(DoctorArgs),
    #[command(about = "Inspect and validate configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum AdAccountsCommand {
    #[command(about = "List ad accounts", visible_alias = "ls")]
    List(AdAccountListArgs),
    #[command(about = "Get a single ad account", visible_alias = "cat")]
    Get(AdAccountGetArgs),
}

#[derive(Subcommand, Debug)]
pub enum CampaignsCommand {
    #[command(about = "List campaigns for an ad account", visible_alias = "ls")]
    List(CampaignListArgs),
}

#[derive(Subcommand, Debug)]
pub enum AdgroupsCommand {
    #[command(about = "List ad groups for an ad account", visible_alias = "ls")]
    List(AdgroupListArgs),
}

#[derive(Subcommand, Debug)]
pub enum AdsCommand {
    #[command(about = "List ads for an ad account", visible_alias = "ls")]
    List(AdsListArgs),
}

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommand {
    #[command(about = "Run a synchronous analytics query")]
    Query(AnalyticsQueryArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReportRunsCommand {
    #[command(about = "Submit an async report request")]
    Submit(ReportSubmitArgs),
    #[command(about = "Check async report status")]
    Status(ReportStatusArgs),
    #[command(about = "Poll until the report reaches a terminal state")]
    Wait(ReportWaitArgs),
}

#[derive(Subcommand, Debug)]
pub enum AudiencesCommand {
    #[command(about = "List audiences for an ad account", visible_alias = "ls")]
    List(AudienceListArgs),
    #[command(about = "Get a single audience", visible_alias = "cat")]
    Get(AudienceGetArgs),
}

#[derive(Subcommand, Debug)]
pub enum TargetingAnalyticsCommand {
    #[command(about = "Run a targeting analytics query")]
    Query(TargetingAnalyticsQueryArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    #[command(about = "Store Pinterest app credentials and tokens in the OS credential store")]
    Set(AuthSetArgs),
    #[command(about = "Show auth source and secure storage status")]
    Status,
    #[command(about = "Delete stored Pinterest credentials")]
    Delete,
    #[command(about = "Refresh the access token using the stored refresh token")]
    Refresh,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    #[command(about = "Show resolved config file path")]
    Path,
    #[command(about = "Show full resolved configuration")]
    Show,
    #[command(about = "Validate config file")]
    Validate,
}

// ---------------------------------------------------------------------------
// Arg structs
// ---------------------------------------------------------------------------

#[derive(Args, Debug, Clone, Default)]
pub struct PinterestPaginationArgs {
    #[arg(long, help = "Resume from a Pinterest bookmark")]
    pub bookmark: Option<String>,
    #[arg(long = "page-size", help = "Items per API request")]
    pub page_size: Option<u32>,
    #[arg(long, help = "Auto-paginate through all results")]
    pub all: bool,
    #[arg(long = "max-items", help = "Stop after collecting N total items")]
    pub max_items: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct AdAccountSelectorArgs {
    #[arg(long = "ad-account-id", help = "Pinterest ad account ID")]
    pub ad_account_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AdAccountListArgs {
    #[arg(
        long = "include-shared-accounts",
        help = "Include ad accounts shared with the authorized user"
    )]
    pub include_shared_accounts: bool,
    #[command(flatten)]
    pub pagination: PinterestPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AdAccountGetArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CampaignListArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long, help = "Sort order")]
    pub order: Option<String>,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter to one or more campaign IDs"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "entity-status",
        value_delimiter = ',',
        help = "Filter by entity status"
    )]
    pub entity_statuses: Vec<String>,
    #[command(flatten)]
    pub pagination: PinterestPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AdgroupListArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long, help = "Sort order")]
    pub order: Option<String>,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter to one or more campaign IDs"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "ad-group-id",
        value_delimiter = ',',
        help = "Filter to one or more ad group IDs"
    )]
    pub ad_group_ids: Vec<String>,
    #[arg(
        long = "entity-status",
        value_delimiter = ',',
        help = "Filter by entity status"
    )]
    pub entity_statuses: Vec<String>,
    #[arg(
        long = "translate-interests-to-names",
        help = "Resolve interest IDs to human-readable names"
    )]
    pub translate_interests_to_names: bool,
    #[command(flatten)]
    pub pagination: PinterestPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AdsListArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long, help = "Sort order")]
    pub order: Option<String>,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter to one or more campaign IDs"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "ad-group-id",
        value_delimiter = ',',
        help = "Filter to one or more ad group IDs"
    )]
    pub ad_group_ids: Vec<String>,
    #[arg(
        long = "ad-id",
        value_delimiter = ',',
        help = "Filter to one or more ad IDs"
    )]
    pub ad_ids: Vec<String>,
    #[arg(
        long = "entity-status",
        value_delimiter = ',',
        help = "Filter by entity status"
    )]
    pub entity_statuses: Vec<String>,
    #[command(flatten)]
    pub pagination: PinterestPaginationArgs,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AnalyticsLevelArg {
    #[value(name = "ad_account")]
    AdAccount,
    #[value(name = "campaign")]
    Campaign,
    #[value(name = "ad_group")]
    AdGroup,
    #[value(name = "ad")]
    Ad,
    #[value(name = "ad_pin")]
    AdPin,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyticsQueryArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long, value_enum, help = "Analytics level")]
    pub level: AnalyticsLevelArg,
    #[arg(long = "start-date", help = "Start date (YYYY-MM-DD)")]
    pub start_date: String,
    #[arg(long = "end-date", help = "End date (YYYY-MM-DD)")]
    pub end_date: String,
    #[arg(long, value_delimiter = ',', help = "Columns to return")]
    pub columns: Vec<String>,
    #[arg(long, help = "Granularity (e.g. TOTAL, DAY, HOUR)")]
    pub granularity: String,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Campaign IDs for campaign- or ad-level analytics"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "ad-group-id",
        value_delimiter = ',',
        help = "Ad group IDs for ad-group analytics"
    )]
    pub ad_group_ids: Vec<String>,
    #[arg(
        long = "ad-id",
        value_delimiter = ',',
        help = "Ad IDs for ad-level analytics"
    )]
    pub ad_ids: Vec<String>,
    #[arg(
        long = "pin-id",
        value_delimiter = ',',
        help = "Pin IDs for ad or ad-pin analytics"
    )]
    pub pin_ids: Vec<String>,
    #[arg(
        long = "campaign",
        help = "Single campaign ID required for ad-pin analytics"
    )]
    pub campaign_id: Option<String>,
    #[arg(long = "click-window-days", help = "Click attribution window in days")]
    pub click_window_days: Option<u32>,
    #[arg(
        long = "engagement-window-days",
        help = "Engagement attribution window in days"
    )]
    pub engagement_window_days: Option<u32>,
    #[arg(long = "view-window-days", help = "View attribution window in days")]
    pub view_window_days: Option<u32>,
    #[arg(long = "conversion-report-time", help = "Conversion report time mode")]
    pub conversion_report_time: Option<String>,
    #[arg(long = "reporting-timezone", help = "Reporting timezone")]
    pub reporting_timezone: Option<String>,
    #[arg(
        long = "aggregate-report-rows",
        help = "Aggregate report rows when the API supports it"
    )]
    pub aggregate_report_rows: bool,
}

#[derive(Args, Debug, Clone, Default)]
#[group(id = "pinterest_report_input", multiple = false)]
pub struct ReportRequestInputArgs {
    #[arg(long = "request-json", help = "Inline JSON request body")]
    pub request_json: Option<String>,
    #[arg(
        long = "request-file",
        help = "Read JSON request body from file (- for stdin)"
    )]
    pub request_file: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReportSubmitArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[command(flatten)]
    pub request_input: ReportRequestInputArgs,
    #[arg(long, help = "Report level (e.g. CAMPAIGN, AD_GROUP, PIN_PROMOTION)")]
    pub level: Option<String>,
    #[arg(long = "start-date", help = "Start date (YYYY-MM-DD)")]
    pub start_date: Option<String>,
    #[arg(long = "end-date", help = "End date (YYYY-MM-DD)")]
    pub end_date: Option<String>,
    #[arg(long, help = "Granularity (e.g. TOTAL, DAY, HOUR)")]
    pub granularity: Option<String>,
    #[arg(long, value_delimiter = ',', help = "Columns to include in the report")]
    pub columns: Vec<String>,
    #[arg(long = "report-format", help = "Report format (JSON or CSV)")]
    pub report_format: Option<String>,
    #[arg(long = "campaign-id", value_delimiter = ',', help = "Campaign IDs")]
    pub campaign_ids: Vec<String>,
    #[arg(long = "ad-group-id", value_delimiter = ',', help = "Ad group IDs")]
    pub ad_group_ids: Vec<String>,
    #[arg(long = "ad-id", value_delimiter = ',', help = "Ad IDs")]
    pub ad_ids: Vec<String>,
    #[arg(
        long = "targeting-type",
        value_delimiter = ',',
        help = "Targeting types"
    )]
    pub targeting_types: Vec<String>,
    #[arg(long = "reporting-timezone", help = "Reporting timezone")]
    pub reporting_timezone: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ReportStatusArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long, help = "Pinterest report token")]
    pub token: String,
}

#[derive(Args, Debug, Clone)]
pub struct ReportWaitArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long, help = "Pinterest report token")]
    pub token: String,
    #[arg(long, default_value_t = 5, help = "Seconds between status polls")]
    pub poll_interval_seconds: u64,
    #[arg(
        long,
        default_value_t = 600,
        help = "Max seconds to wait before timing out"
    )]
    pub wait_timeout_seconds: u64,
}

#[derive(Args, Debug, Clone)]
pub struct AudienceListArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long, help = "Sort order")]
    pub order: Option<String>,
    #[arg(long = "ownership-type", help = "Audience ownership type")]
    pub ownership_type: Option<String>,
    #[command(flatten)]
    pub pagination: PinterestPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AudienceGetArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long = "audience-id", help = "Pinterest audience ID")]
    pub audience_id: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TargetingLevelArg {
    #[value(name = "ad_account")]
    AdAccount,
    #[value(name = "ad_group")]
    AdGroup,
    #[value(name = "ad")]
    Ad,
}

#[derive(Args, Debug, Clone)]
pub struct TargetingAnalyticsQueryArgs {
    #[command(flatten)]
    pub selector: AdAccountSelectorArgs,
    #[arg(long, value_enum, help = "Targeting analytics level")]
    pub level: TargetingLevelArg,
    #[arg(long = "start-date", help = "Start date (YYYY-MM-DD)")]
    pub start_date: String,
    #[arg(long = "end-date", help = "End date (YYYY-MM-DD)")]
    pub end_date: String,
    #[arg(
        long = "targeting-type",
        value_delimiter = ',',
        help = "Targeting types"
    )]
    pub targeting_types: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Columns to return")]
    pub columns: Vec<String>,
    #[arg(long, help = "Granularity (e.g. TOTAL, DAY, HOUR)")]
    pub granularity: String,
    #[arg(long = "ad-group-id", value_delimiter = ',', help = "Ad group IDs")]
    pub ad_group_ids: Vec<String>,
    #[arg(long = "ad-id", value_delimiter = ',', help = "Ad IDs")]
    pub ad_ids: Vec<String>,
    #[arg(long = "click-window-days", help = "Click attribution window in days")]
    pub click_window_days: Option<u32>,
    #[arg(
        long = "engagement-window-days",
        help = "Engagement attribution window in days"
    )]
    pub engagement_window_days: Option<u32>,
    #[arg(long = "view-window-days", help = "View attribution window in days")]
    pub view_window_days: Option<u32>,
    #[arg(long = "conversion-report-time", help = "Conversion report time mode")]
    pub conversion_report_time: Option<String>,
    #[arg(
        long = "attribution-type",
        value_delimiter = ',',
        help = "Attribution types"
    )]
    pub attribution_types: Vec<String>,
    #[arg(long = "reporting-timezone", help = "Reporting timezone")]
    pub reporting_timezone: Option<String>,
    #[arg(long = "sort-column", value_delimiter = ',', help = "Sort columns")]
    pub sort_columns: Vec<String>,
    #[arg(long = "sort-ascending", help = "Sort ascending")]
    pub sort_ascending: bool,
}

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    #[arg(
        long,
        help = "Also refresh credentials and make a lightweight Pinterest Ads API request"
    )]
    pub api: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AuthSetArgs {
    #[arg(
        long,
        conflicts_with_all = ["app_id", "app_secret", "access_token", "refresh_token"],
        help = "Read app ID, app secret, access token, and refresh token from stdin"
    )]
    pub stdin: bool,
    #[arg(long = "app-id", help = "Pinterest app ID")]
    pub app_id: Option<String>,
    #[arg(long = "app-secret", help = "Pinterest app secret")]
    pub app_secret: Option<String>,
    #[arg(long = "access-token", help = "Pinterest access token")]
    pub access_token: Option<String>,
    #[arg(long = "refresh-token", help = "Pinterest refresh token")]
    pub refresh_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Dispatch: auth, config, doctor
// ---------------------------------------------------------------------------

pub fn handle_auth(
    command: AuthCommand,
    secret_store: &dyn SecretStore,
) -> Result<CommandResult, PinterestError> {
    match command {
        AuthCommand::Set(args) => {
            let inputs = resolve_pinterest_auth_inputs(&args)?;
            secret_store
                .set_secret(
                    PINTEREST_ADS_APP_ID_SERVICE,
                    PINTEREST_ADS_APP_ID_ACCOUNT,
                    &inputs.app_id,
                )
                .map_err(|error| pinterest_auth_storage_error("store app ID", &error))?;
            secret_store
                .set_secret(
                    PINTEREST_ADS_APP_SECRET_SERVICE,
                    PINTEREST_ADS_APP_SECRET_ACCOUNT,
                    &inputs.app_secret,
                )
                .map_err(|error| pinterest_auth_storage_error("store app secret", &error))?;
            secret_store
                .set_secret(
                    PINTEREST_ADS_ACCESS_TOKEN_SERVICE,
                    PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT,
                    &inputs.access_token,
                )
                .map_err(|error| pinterest_auth_storage_error("store access token", &error))?;
            secret_store
                .set_secret(
                    PINTEREST_ADS_REFRESH_TOKEN_SERVICE,
                    PINTEREST_ADS_REFRESH_TOKEN_ACCOUNT,
                    &inputs.refresh_token,
                )
                .map_err(|error| pinterest_auth_storage_error("store refresh token", &error))?;

            Ok(pinterest_command_result(
                json!({
                    "provider": "pinterest",
                    "stored": true,
                    "credentials_stored": [
                        "app_id",
                        "app_secret",
                        "access_token",
                        "refresh_token"
                    ],
                }),
                "/pinterest/auth/set",
                0,
            ))
        }
        AuthCommand::Status => Ok(pinterest_command_result(
            pinterest_auth_status_payload(pinterest_inspect_auth(secret_store)),
            "/pinterest/auth/status",
            0,
        )),
        AuthCommand::Delete => {
            let deleted_app_id = secret_store
                .delete_secret(PINTEREST_ADS_APP_ID_SERVICE, PINTEREST_ADS_APP_ID_ACCOUNT)
                .map_err(|error| pinterest_auth_storage_error("delete app ID", &error))?;
            let deleted_app_secret = secret_store
                .delete_secret(
                    PINTEREST_ADS_APP_SECRET_SERVICE,
                    PINTEREST_ADS_APP_SECRET_ACCOUNT,
                )
                .map_err(|error| pinterest_auth_storage_error("delete app secret", &error))?;
            let deleted_access_token = secret_store
                .delete_secret(
                    PINTEREST_ADS_ACCESS_TOKEN_SERVICE,
                    PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT,
                )
                .map_err(|error| pinterest_auth_storage_error("delete access token", &error))?;
            let deleted_refresh_token = secret_store
                .delete_secret(
                    PINTEREST_ADS_REFRESH_TOKEN_SERVICE,
                    PINTEREST_ADS_REFRESH_TOKEN_ACCOUNT,
                )
                .map_err(|error| pinterest_auth_storage_error("delete refresh token", &error))?;

            Ok(pinterest_command_result(
                json!({
                    "provider": "pinterest",
                    "app_id_deleted": deleted_app_id,
                    "app_secret_deleted": deleted_app_secret,
                    "access_token_deleted": deleted_access_token,
                    "refresh_token_deleted": deleted_refresh_token,
                }),
                "/pinterest/auth/delete",
                0,
            ))
        }
        AuthCommand::Refresh => unreachable!("auth refresh is dispatched separately"),
    }
}

pub async fn handle_auth_refresh(
    secret_store: &dyn SecretStore,
    snapshot: &PinterestConfigSnapshot,
) -> Result<CommandResult, PinterestError> {
    let auth = resolve_pinterest_refresh_auth(secret_store)?;
    let refresh = pinterest_refresh_access_token(
        snapshot.timeout_seconds,
        &auth.app_id,
        &auth.app_secret,
        &auth.refresh_token,
    )
    .await?;

    secret_store
        .set_secret(
            PINTEREST_ADS_ACCESS_TOKEN_SERVICE,
            PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT,
            &refresh.access_token,
        )
        .map_err(|error| pinterest_auth_storage_error("store refreshed access token", &error))?;

    if let Some(new_refresh_token) = &refresh.refresh_token {
        secret_store
            .set_secret(
                PINTEREST_ADS_REFRESH_TOKEN_SERVICE,
                PINTEREST_ADS_REFRESH_TOKEN_ACCOUNT,
                new_refresh_token,
            )
            .map_err(|error| pinterest_auth_storage_error("store new refresh token", &error))?;
    }

    Ok(command_result(
        json!({
            "provider": "pinterest",
            "refreshed": true,
            "expires_in": refresh.expires_in,
            "scope": refresh.scope,
            "token_type": refresh.token_type,
            "refresh_token_expires_in": refresh.refresh_token_expires_in,
            "refresh_token_expires_at": refresh.refresh_token_expires_at,
            "new_refresh_token_stored": refresh.refresh_token.is_some(),
        }),
        "/pinterest/auth/refresh",
        0,
        Some(&snapshot.api_version),
    ))
}

pub fn handle_config(
    command: ConfigCommand,
    snapshot: PinterestConfigSnapshot,
) -> Result<CommandResult, PinterestError> {
    match command {
        ConfigCommand::Path => Ok(pinterest_command_result(
            json!({
                "path": snapshot.config_path,
                "exists": snapshot.config_file_exists,
            }),
            "/pinterest/config/path",
            0,
        )),
        ConfigCommand::Show => Ok(pinterest_command_result(
            json!(snapshot),
            "/pinterest/config/show",
            0,
        )),
        ConfigCommand::Validate => Ok(pinterest_command_result(
            json!({
                "valid": true,
                "config": snapshot,
            }),
            "/pinterest/config/validate",
            0,
        )),
    }
}

pub async fn handle_doctor(
    args: DoctorArgs,
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &PinterestConfigOverrides,
    snapshot: PinterestConfigSnapshot,
) -> Result<CommandResult, PinterestError> {
    let mut checks = vec![
        json!({
            "name": "credential_store",
            "ok": pinterest_credential_store_check_ok(&snapshot),
            "detail": pinterest_credential_store_detail(&snapshot),
        }),
        json!({
            "name": "config_file",
            "ok": snapshot.config_file_exists,
            "detail": if snapshot.config_file_exists {
                format!("using {}", snapshot.config_path.display())
            } else {
                format!("config file not found at {}", snapshot.config_path.display())
            }
        }),
        json!({
            "name": "app_id",
            "ok": snapshot.auth.app_id.present,
            "detail": pinterest_secret_detail(PINTEREST_ADS_APP_ID_ENV_VAR, "app ID", &snapshot.auth.app_id),
        }),
        json!({
            "name": "app_secret",
            "ok": snapshot.auth.app_secret.present,
            "detail": pinterest_secret_detail(PINTEREST_ADS_APP_SECRET_ENV_VAR, "app secret", &snapshot.auth.app_secret),
        }),
        json!({
            "name": "access_token",
            "ok": snapshot.auth.access_token.present,
            "detail": pinterest_secret_detail(PINTEREST_ADS_ACCESS_TOKEN_ENV_VAR, "access token", &snapshot.auth.access_token),
        }),
        json!({
            "name": "refresh_token",
            "ok": snapshot.auth.refresh_token.present,
            "detail": pinterest_secret_detail(PINTEREST_ADS_REFRESH_TOKEN_ENV_VAR, "refresh token", &snapshot.auth.refresh_token),
        }),
    ];

    let mut ok = required_pinterest_credentials_present(&snapshot.auth);
    if args.api {
        if ok {
            match PinterestResolvedConfig::load(config_path, secret_store, overrides) {
                Ok(config) => match PinterestClient::from_config(&config).await {
                    Ok(client) => match accounts::list_ad_accounts(
                        &client,
                        None,
                        None,
                        Some(1),
                        false,
                        Some(1),
                    )
                    .await
                    {
                        Ok(response) => {
                            let count = response
                                .data
                                .as_array()
                                .map(|items| items.len())
                                .unwrap_or(0);
                            checks.push(json!({
                                "name": "api_ping",
                                "ok": true,
                                "detail": format!(
                                    "credentials accepted by Pinterest Ads API; sampled {} ad account record(s)",
                                    count
                                )
                            }));
                        }
                        Err(error) => {
                            ok = false;
                            checks.push(json!({
                                "name": "api_ping",
                                "ok": false,
                                "detail": error.to_string()
                            }));
                        }
                    },
                    Err(error) => {
                        ok = false;
                        checks.push(json!({
                            "name": "api_ping",
                            "ok": false,
                            "detail": error.to_string()
                        }));
                    }
                },
                Err(error) => {
                    ok = false;
                    checks.push(json!({
                        "name": "api_ping",
                        "ok": false,
                        "detail": error.to_string()
                    }));
                }
            }
        } else {
            ok = false;
            checks.push(json!({
                "name": "api_ping",
                "ok": false,
                "detail": "skipped because required Pinterest credentials are missing"
            }));
        }
    }

    Ok(pinterest_command_result(
        json!({
            "ok": ok,
            "checks": checks,
            "config": snapshot,
        }),
        "/pinterest/doctor",
        if ok { 0 } else { 1 },
    ))
}

// ---------------------------------------------------------------------------
// Dispatch: authenticated commands
// ---------------------------------------------------------------------------

pub async fn dispatch_pinterest_with_client(
    client: &PinterestClient,
    config: &PinterestResolvedConfig,
    command: PinterestCommand,
) -> Result<CommandResult, PinterestError> {
    match command {
        PinterestCommand::AdAccounts { command } => match command {
            AdAccountsCommand::List(args) => {
                let response = accounts::list_ad_accounts(
                    client,
                    bool_flag(args.include_shared_accounts),
                    args.pagination.bookmark.as_deref(),
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(pinterest_result(
                    client,
                    response,
                    "/ad_accounts",
                    None,
                    None,
                    Vec::new(),
                ))
            }
            AdAccountsCommand::Get(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let response = accounts::get_ad_account(client, &ad_account_id).await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!("/ad_accounts/{ad_account_id}"),
                    Some(ad_account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        PinterestCommand::Campaigns { command } => match command {
            CampaignsCommand::List(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let response = campaigns::list_campaigns(
                    client,
                    &ad_account_id,
                    args.pagination.bookmark.as_deref(),
                    args.pagination.page_size,
                    args.order.as_deref(),
                    &args.campaign_ids,
                    &args.entity_statuses,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!("/ad_accounts/{ad_account_id}/campaigns"),
                    Some(ad_account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        PinterestCommand::Adgroups { command } => match command {
            AdgroupsCommand::List(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let response = adgroups::list_adgroups(
                    client,
                    &ad_account_id,
                    args.pagination.bookmark.as_deref(),
                    args.pagination.page_size,
                    args.order.as_deref(),
                    &args.campaign_ids,
                    &args.ad_group_ids,
                    &args.entity_statuses,
                    bool_flag(args.translate_interests_to_names),
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!("/ad_accounts/{ad_account_id}/ad_groups"),
                    Some(ad_account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        PinterestCommand::Ads { command } => match command {
            AdsCommand::List(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let response = ads::list_ads(
                    client,
                    &ad_account_id,
                    args.pagination.bookmark.as_deref(),
                    args.pagination.page_size,
                    args.order.as_deref(),
                    &args.campaign_ids,
                    &args.ad_group_ids,
                    &args.ad_ids,
                    &args.entity_statuses,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!("/ad_accounts/{ad_account_id}/ads"),
                    Some(ad_account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        PinterestCommand::Analytics { command } => match command {
            AnalyticsCommand::Query(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                validate_analytics_query(&args)?;
                let response = pinterest_analytics::query_analytics(
                    client,
                    pinterest_analytics::AnalyticsQuery {
                        ad_account_id: &ad_account_id,
                        level: to_analytics_level(args.level),
                        start_date: &args.start_date,
                        end_date: &args.end_date,
                        columns: &args.columns,
                        granularity: &args.granularity,
                        campaign_ids: &args.campaign_ids,
                        ad_group_ids: &args.ad_group_ids,
                        ad_ids: &args.ad_ids,
                        pin_ids: &args.pin_ids,
                        campaign_id: args.campaign_id.as_deref(),
                        click_window_days: args.click_window_days,
                        engagement_window_days: args.engagement_window_days,
                        view_window_days: args.view_window_days,
                        conversion_report_time: args.conversion_report_time.as_deref(),
                        reporting_timezone: args.reporting_timezone.as_deref(),
                        aggregate_report_rows: bool_flag(args.aggregate_report_rows),
                    },
                )
                .await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!(
                        "/ad_accounts/{ad_account_id}/{}",
                        analytics_endpoint_path(args.level)
                    ),
                    Some(ad_account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        PinterestCommand::ReportRuns { command } => match command {
            ReportRunsCommand::Submit(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let request = resolve_report_request(&args)?;
                let response = reports::create_report(client, &ad_account_id, &request).await?;
                let report_run_id = extract_report_token(&response.data);
                Ok(pinterest_result(
                    client,
                    response,
                    &format!("/ad_accounts/{ad_account_id}/reports"),
                    Some(ad_account_id),
                    report_run_id,
                    Vec::new(),
                ))
            }
            ReportRunsCommand::Status(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let response = reports::get_report(client, &ad_account_id, &args.token).await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!("/ad_accounts/{ad_account_id}/reports"),
                    Some(ad_account_id),
                    Some(args.token),
                    Vec::new(),
                ))
            }
            ReportRunsCommand::Wait(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let response = wait_for_report(
                    client,
                    &ad_account_id,
                    &args.token,
                    Duration::from_secs(args.poll_interval_seconds),
                    Duration::from_secs(args.wait_timeout_seconds),
                )
                .await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!("/ad_accounts/{ad_account_id}/reports"),
                    Some(ad_account_id),
                    Some(args.token),
                    Vec::new(),
                ))
            }
        },
        PinterestCommand::Audiences { command } => match command {
            AudiencesCommand::List(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let response = audiences::list_audiences(
                    client,
                    &ad_account_id,
                    args.pagination.bookmark.as_deref(),
                    args.pagination.page_size,
                    args.order.as_deref(),
                    args.ownership_type.as_deref(),
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!("/ad_accounts/{ad_account_id}/audiences"),
                    Some(ad_account_id),
                    None,
                    Vec::new(),
                ))
            }
            AudiencesCommand::Get(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                let response =
                    audiences::get_audience(client, &ad_account_id, &args.audience_id).await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!(
                        "/ad_accounts/{ad_account_id}/audiences/{}",
                        args.audience_id
                    ),
                    Some(ad_account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        PinterestCommand::TargetingAnalytics { command } => match command {
            TargetingAnalyticsCommand::Query(args) => {
                let ad_account_id =
                    resolve_ad_account_id(config, args.selector.ad_account_id.as_deref())?;
                validate_targeting_query(&args)?;
                let response = pinterest_targeting::query_targeting_analytics(
                    client,
                    pinterest_targeting::TargetingAnalyticsQuery {
                        ad_account_id: &ad_account_id,
                        level: to_targeting_level(args.level),
                        start_date: &args.start_date,
                        end_date: &args.end_date,
                        targeting_types: &args.targeting_types,
                        columns: &args.columns,
                        granularity: &args.granularity,
                        ad_group_ids: &args.ad_group_ids,
                        ad_ids: &args.ad_ids,
                        click_window_days: args.click_window_days,
                        engagement_window_days: args.engagement_window_days,
                        view_window_days: args.view_window_days,
                        conversion_report_time: args.conversion_report_time.as_deref(),
                        attribution_types: &args.attribution_types,
                        reporting_timezone: args.reporting_timezone.as_deref(),
                        sort_columns: &args.sort_columns,
                        sort_ascending: bool_flag(args.sort_ascending),
                    },
                )
                .await?;
                Ok(pinterest_result(
                    client,
                    response,
                    &format!(
                        "/ad_accounts/{ad_account_id}/{}",
                        targeting_endpoint_path(args.level)
                    ),
                    Some(ad_account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        PinterestCommand::Auth { .. }
        | PinterestCommand::Doctor(_)
        | PinterestCommand::Config { .. } => {
            unreachable!("auth/config/doctor are dispatched before loading Pinterest credentials")
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn pinterest_result(
    client: &PinterestClient,
    response: PinterestResponse,
    endpoint: &str,
    ad_account_id: Option<String>,
    report_run_id: Option<String>,
    warnings: Vec<String>,
) -> CommandResult {
    let mut envelope = OutputEnvelope::new(
        response.data,
        OutputMeta {
            api_version: client.api_version().to_string(),
            endpoint: endpoint.to_string(),
            object_id: ad_account_id,
            request_id: response.request_id,
            report_run_id,
        },
    );
    envelope.paging = response.paging;
    if !warnings.is_empty() {
        envelope.warnings = Some(warnings);
    }
    CommandResult {
        envelope,
        exit_code: 0,
    }
}

fn pinterest_command_result(data: Value, endpoint: &str, exit_code: u8) -> CommandResult {
    command_result(
        data,
        endpoint,
        exit_code,
        Some(PINTEREST_DEFAULT_API_VERSION),
    )
}

fn resolve_ad_account_id(
    config: &PinterestResolvedConfig,
    value: Option<&str>,
) -> Result<String, PinterestError> {
    value
        .map(str::to_string)
        .or_else(|| config.default_ad_account_id.clone())
        .ok_or_else(|| {
            PinterestError::InvalidArgument(
                "Pinterest ad account ID is required. Pass --ad-account-id or set providers.pinterest.default_ad_account_id / PINTEREST_ADS_DEFAULT_AD_ACCOUNT_ID.".to_string(),
            )
        })
}

fn validate_analytics_query(args: &AnalyticsQueryArgs) -> Result<(), PinterestError> {
    if args.columns.is_empty() {
        return Err(PinterestError::InvalidArgument(
            "--columns requires at least one analytics column".to_string(),
        ));
    }

    match args.level {
        AnalyticsLevelArg::Campaign if args.campaign_ids.is_empty() => {
            Err(PinterestError::InvalidArgument(
                "campaign analytics requires at least one --campaign-id".to_string(),
            ))
        }
        AnalyticsLevelArg::AdGroup if args.ad_group_ids.is_empty() => {
            Err(PinterestError::InvalidArgument(
                "ad-group analytics requires at least one --ad-group-id".to_string(),
            ))
        }
        AnalyticsLevelArg::AdPin if args.campaign_id.is_none() || args.pin_ids.is_empty() => {
            Err(PinterestError::InvalidArgument(
                "ad-pin analytics requires --campaign and at least one --pin-id".to_string(),
            ))
        }
        _ => Ok(()),
    }
}

fn validate_targeting_query(args: &TargetingAnalyticsQueryArgs) -> Result<(), PinterestError> {
    if args.targeting_types.is_empty() {
        return Err(PinterestError::InvalidArgument(
            "--targeting-type requires at least one value".to_string(),
        ));
    }
    if args.columns.is_empty() {
        return Err(PinterestError::InvalidArgument(
            "--columns requires at least one targeting analytics column".to_string(),
        ));
    }

    match args.level {
        TargetingLevelArg::AdGroup if args.ad_group_ids.is_empty() => {
            Err(PinterestError::InvalidArgument(
                "ad-group targeting analytics requires at least one --ad-group-id".to_string(),
            ))
        }
        TargetingLevelArg::Ad if args.ad_ids.is_empty() => Err(PinterestError::InvalidArgument(
            "ad targeting analytics requires at least one --ad-id".to_string(),
        )),
        _ => Ok(()),
    }
}

fn analytics_endpoint_path(level: AnalyticsLevelArg) -> &'static str {
    match level {
        AnalyticsLevelArg::AdAccount => "analytics",
        AnalyticsLevelArg::Campaign => "campaigns/analytics",
        AnalyticsLevelArg::AdGroup => "ad_groups/analytics",
        AnalyticsLevelArg::Ad => "ads/analytics",
        AnalyticsLevelArg::AdPin => "pins/analytics",
    }
}

fn targeting_endpoint_path(level: TargetingLevelArg) -> &'static str {
    match level {
        TargetingLevelArg::AdAccount => "targeting_analytics",
        TargetingLevelArg::AdGroup => "ad_groups/targeting_analytics",
        TargetingLevelArg::Ad => "ads/targeting_analytics",
    }
}

fn to_analytics_level(level: AnalyticsLevelArg) -> pinterest_analytics::AnalyticsLevel {
    match level {
        AnalyticsLevelArg::AdAccount => pinterest_analytics::AnalyticsLevel::AdAccount,
        AnalyticsLevelArg::Campaign => pinterest_analytics::AnalyticsLevel::Campaign,
        AnalyticsLevelArg::AdGroup => pinterest_analytics::AnalyticsLevel::AdGroup,
        AnalyticsLevelArg::Ad => pinterest_analytics::AnalyticsLevel::Ad,
        AnalyticsLevelArg::AdPin => pinterest_analytics::AnalyticsLevel::AdPin,
    }
}

fn to_targeting_level(level: TargetingLevelArg) -> pinterest_targeting::TargetingLevel {
    match level {
        TargetingLevelArg::AdAccount => pinterest_targeting::TargetingLevel::AdAccount,
        TargetingLevelArg::AdGroup => pinterest_targeting::TargetingLevel::AdGroup,
        TargetingLevelArg::Ad => pinterest_targeting::TargetingLevel::Ad,
    }
}

fn bool_flag(value: bool) -> Option<bool> {
    value.then_some(true)
}

async fn wait_for_report(
    client: &PinterestClient,
    ad_account_id: &str,
    token: &str,
    poll_interval: Duration,
    timeout: Duration,
) -> Result<PinterestResponse, PinterestError> {
    let started = Instant::now();
    loop {
        let response = reports::get_report(client, ad_account_id, token).await?;
        let status = response
            .data
            .get("report_status")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_uppercase();

        match status.as_str() {
            "FINISHED" => return Ok(response),
            "FAILED" | "CANCELLED" | "EXPIRED" | "DOES_NOT_EXIST" => {
                return Err(PinterestError::Config(format!(
                    "report run {token} ended with report_status `{status}`"
                )));
            }
            _ => {}
        }

        if started.elapsed() >= timeout {
            return Err(PinterestError::Config(format!(
                "timed out waiting for report run {token}"
            )));
        }

        sleep(poll_interval).await;
    }
}

fn resolve_report_request(args: &ReportSubmitArgs) -> Result<Value, PinterestError> {
    let mut body = match read_optional_json_input(
        args.request_input.request_json.as_deref(),
        args.request_input.request_file.as_deref(),
        "report request body",
    )? {
        Some(value) => value,
        None => Value::Object(Map::new()),
    };

    let object = body.as_object_mut().ok_or_else(|| {
        PinterestError::InvalidArgument(
            "Pinterest report request body must be a JSON object".to_string(),
        )
    })?;

    insert_string(object, "level", args.level.as_deref());
    insert_string(object, "start_date", args.start_date.as_deref());
    insert_string(object, "end_date", args.end_date.as_deref());
    insert_string(object, "granularity", args.granularity.as_deref());
    insert_string(object, "report_format", args.report_format.as_deref());
    insert_string(
        object,
        "reporting_timezone",
        args.reporting_timezone.as_deref(),
    );
    insert_array(object, "columns", &args.columns);
    insert_array(object, "campaign_ids", &args.campaign_ids);
    insert_array(object, "ad_group_ids", &args.ad_group_ids);
    insert_array(object, "ad_ids", &args.ad_ids);
    insert_array(object, "targeting_types", &args.targeting_types);

    if !has_nonempty_string(object, "level")
        || !has_nonempty_string(object, "start_date")
        || !has_nonempty_string(object, "end_date")
        || !has_nonempty_string(object, "granularity")
        || !has_nonempty_array(object, "columns")
    {
        return Err(PinterestError::InvalidArgument(
            "report-runs submit requires level, start_date, end_date, granularity, and a non-empty columns array. Pass them with flags or supply --request-json/--request-file.".to_string(),
        ));
    }

    Ok(body)
}

fn read_optional_json_input(
    inline: Option<&str>,
    file: Option<&Path>,
    label: &str,
) -> Result<Option<Value>, PinterestError> {
    if let Some(inline) = inline {
        let value = serde_json::from_str::<Value>(inline).map_err(|error| {
            PinterestError::InvalidArgument(format!("invalid {label} JSON: {error}"))
        })?;
        return Ok(Some(value));
    }

    if let Some(path) = file {
        let content = read_input(path).map_err(PinterestError::from)?;
        let value = serde_json::from_str::<Value>(&content).map_err(|error| {
            PinterestError::InvalidArgument(format!("invalid {label} JSON in file: {error}"))
        })?;
        return Ok(Some(value));
    }

    Ok(None)
}

fn insert_string(object: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        object.insert(key.to_string(), json!(value));
    }
}

fn insert_array(object: &mut Map<String, Value>, key: &str, values: &[String]) {
    if !values.is_empty() {
        object.insert(key.to_string(), json!(values));
    }
}

fn has_nonempty_string(object: &Map<String, Value>, key: &str) -> bool {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn has_nonempty_array(object: &Map<String, Value>, key: &str) -> bool {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(|values| !values.is_empty())
        .unwrap_or(false)
}

fn extract_report_token(value: &Value) -> Option<String> {
    [
        value.get("token"),
        value.get("report_token"),
        value.get("report_id"),
        value.get("report_run_id"),
    ]
    .into_iter()
    .flatten()
    .find_map(Value::as_str)
    .map(str::to_string)
}

struct PinterestAuthInputs {
    app_id: String,
    app_secret: String,
    access_token: String,
    refresh_token: String,
}

struct PinterestRefreshAuth {
    app_id: String,
    app_secret: String,
    refresh_token: String,
}

fn resolve_pinterest_auth_inputs(
    args: &AuthSetArgs,
) -> Result<PinterestAuthInputs, PinterestError> {
    if args.stdin {
        let input = read_input(Path::new("-")).map_err(PinterestError::from)?;
        return parse_pinterest_auth_inputs_from_stdin(&input);
    }

    Ok(PinterestAuthInputs {
        app_id: normalize_secret(
            args.app_id
                .as_deref()
                .unwrap_or(&prompt_password("Pinterest app ID: ").map_err(PinterestError::Io)?),
            "app ID",
        )?,
        app_secret: normalize_secret(
            args.app_secret
                .as_deref()
                .unwrap_or(&prompt_password("Pinterest app secret: ").map_err(PinterestError::Io)?),
            "app secret",
        )?,
        access_token: normalize_secret(
            args.access_token.as_deref().unwrap_or(
                &prompt_password("Pinterest access token: ").map_err(PinterestError::Io)?,
            ),
            "access token",
        )?,
        refresh_token: normalize_secret(
            args.refresh_token.as_deref().unwrap_or(
                &prompt_password("Pinterest refresh token: ").map_err(PinterestError::Io)?,
            ),
            "refresh token",
        )?,
    })
}

fn parse_pinterest_auth_inputs_from_stdin(
    input: &str,
) -> Result<PinterestAuthInputs, PinterestError> {
    let values = input
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    match values.as_slice() {
        [app_id, app_secret, access_token, refresh_token] => Ok(PinterestAuthInputs {
            app_id: normalize_secret(app_id, "app ID")?,
            app_secret: normalize_secret(app_secret, "app secret")?,
            access_token: normalize_secret(access_token, "access token")?,
            refresh_token: normalize_secret(refresh_token, "refresh token")?,
        }),
        [] => Err(PinterestError::InvalidArgument(
            "stdin did not contain Pinterest credentials".to_string(),
        )),
        _ => Err(PinterestError::InvalidArgument(
            "stdin must contain exactly four non-empty lines: app ID, app secret, access token, refresh token".to_string(),
        )),
    }
}

fn normalize_secret(value: &str, label: &str) -> Result<String, PinterestError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(PinterestError::InvalidArgument(format!(
            "Pinterest {label} cannot be empty"
        )));
    }
    Ok(trimmed.to_string())
}

fn resolve_pinterest_refresh_auth(
    secret_store: &dyn SecretStore,
) -> Result<PinterestRefreshAuth, PinterestError> {
    Ok(PinterestRefreshAuth {
        app_id: resolve_secret_value(
            secret_store,
            PINTEREST_ADS_APP_ID_ENV_VAR,
            PINTEREST_ADS_APP_ID_SERVICE,
            PINTEREST_ADS_APP_ID_ACCOUNT,
            "app ID",
        )?,
        app_secret: resolve_secret_value(
            secret_store,
            PINTEREST_ADS_APP_SECRET_ENV_VAR,
            PINTEREST_ADS_APP_SECRET_SERVICE,
            PINTEREST_ADS_APP_SECRET_ACCOUNT,
            "app secret",
        )?,
        refresh_token: resolve_secret_value(
            secret_store,
            PINTEREST_ADS_REFRESH_TOKEN_ENV_VAR,
            PINTEREST_ADS_REFRESH_TOKEN_SERVICE,
            PINTEREST_ADS_REFRESH_TOKEN_ACCOUNT,
            "refresh token",
        )?,
    })
}

fn resolve_secret_value(
    secret_store: &dyn SecretStore,
    env_var: &str,
    service: &str,
    account: &str,
    label: &str,
) -> Result<String, PinterestError> {
    if let Some(value) = env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok(value);
    }

    match secret_store.get_secret(service, account) {
        Ok(Some(value)) if !value.trim().is_empty() => Ok(value),
        Ok(Some(_)) | Ok(None) => Err(PinterestError::Config(format!(
            "{env_var} is missing and no Pinterest {label} was found in the OS credential store. Export {env_var} or run `agent-ads pinterest auth set` first."
        ))),
        Err(error) => Err(PinterestError::Config(format!(
            "{env_var} is missing and the OS credential store could not be read: {error}. Export {env_var} or run `agent-ads pinterest auth set` first."
        ))),
    }
}

fn required_pinterest_credentials_present(auth: &PinterestAuthSnapshot) -> bool {
    auth.app_id.present
        && auth.app_secret.present
        && auth.access_token.present
        && auth.refresh_token.present
}

fn pinterest_credential_store_check_ok(snapshot: &PinterestConfigSnapshot) -> bool {
    snapshot.auth.credential_store_available || pinterest_shell_override_active(&snapshot.auth)
}

fn pinterest_shell_override_active(auth: &PinterestAuthSnapshot) -> bool {
    auth.app_id.source == PinterestSecretSource::ShellEnv
        && auth.app_secret.source == PinterestSecretSource::ShellEnv
        && auth.access_token.source == PinterestSecretSource::ShellEnv
        && auth.refresh_token.source == PinterestSecretSource::ShellEnv
}

fn pinterest_credential_store_detail(snapshot: &PinterestConfigSnapshot) -> String {
    match snapshot.auth.credential_store_error.as_deref() {
        Some(error) if pinterest_shell_override_active(&snapshot.auth) => {
            format!("shell env overrides active; OS credential store unavailable: {error}")
        }
        Some(error) => format!("OS credential store unavailable: {error}"),
        None if snapshot.auth.app_id.keychain_present
            || snapshot.auth.app_secret.keychain_present
            || snapshot.auth.access_token.keychain_present
            || snapshot.auth.refresh_token.keychain_present =>
        {
            "stored Pinterest credentials found in the OS credential store".to_string()
        }
        None if snapshot.auth.credential_store_available => {
            "OS credential store is available; no stored Pinterest credentials found".to_string()
        }
        None => "OS credential store is unavailable".to_string(),
    }
}

fn pinterest_secret_detail(env_var: &str, label: &str, status: &PinterestSecretStatus) -> String {
    match status.source {
        PinterestSecretSource::ShellEnv if status.keychain_present => {
            format!("{env_var} is set in shell env and overrides the stored {label}")
        }
        PinterestSecretSource::ShellEnv => format!("{env_var} is set in shell env"),
        PinterestSecretSource::Keychain => {
            format!("using stored Pinterest {label} from the OS credential store")
        }
        PinterestSecretSource::Missing => format!("{env_var} is missing"),
    }
}

fn pinterest_auth_status_payload(auth: PinterestAuthSnapshot) -> Value {
    json!({
        "provider": "pinterest",
        "credential_store_available": auth.credential_store_available,
        "credential_store_error": auth.credential_store_error,
        "credentials": {
            "app_id": {
                "env_var": PINTEREST_ADS_APP_ID_ENV_VAR,
                "credential_store_service": PINTEREST_ADS_APP_ID_SERVICE,
                "credential_store_account": PINTEREST_ADS_APP_ID_ACCOUNT,
                "present": auth.app_id.present,
                "source": auth.app_id.source,
                "keychain_present": auth.app_id.keychain_present,
            },
            "app_secret": {
                "env_var": PINTEREST_ADS_APP_SECRET_ENV_VAR,
                "credential_store_service": PINTEREST_ADS_APP_SECRET_SERVICE,
                "credential_store_account": PINTEREST_ADS_APP_SECRET_ACCOUNT,
                "present": auth.app_secret.present,
                "source": auth.app_secret.source,
                "keychain_present": auth.app_secret.keychain_present,
            },
            "access_token": {
                "env_var": PINTEREST_ADS_ACCESS_TOKEN_ENV_VAR,
                "credential_store_service": PINTEREST_ADS_ACCESS_TOKEN_SERVICE,
                "credential_store_account": PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT,
                "present": auth.access_token.present,
                "source": auth.access_token.source,
                "keychain_present": auth.access_token.keychain_present,
            },
            "refresh_token": {
                "env_var": PINTEREST_ADS_REFRESH_TOKEN_ENV_VAR,
                "credential_store_service": PINTEREST_ADS_REFRESH_TOKEN_SERVICE,
                "credential_store_account": PINTEREST_ADS_REFRESH_TOKEN_ACCOUNT,
                "present": auth.refresh_token.present,
                "source": auth.refresh_token.source,
                "keychain_present": auth.refresh_token.keychain_present,
            }
        }
    })
}

fn pinterest_auth_storage_error(action: &str, error: &impl std::fmt::Display) -> PinterestError {
    PinterestError::Config(format!(
        "failed to {action} in the OS credential store: {error}{}",
        pinterest_linux_secure_storage_hint()
    ))
}

fn pinterest_linux_secure_storage_hint() -> &'static str {
    if cfg!(target_os = "linux") {
        " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet."
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{parse_pinterest_auth_inputs_from_stdin, pinterest_auth_status_payload};
    use agent_ads_core::pinterest_config::{
        PinterestAuthSnapshot, PinterestSecretSource, PinterestSecretStatus,
    };

    #[test]
    fn parses_pinterest_auth_inputs_from_stdin() {
        let inputs = parse_pinterest_auth_inputs_from_stdin(
            " app-id \n app-secret \n access-token \n refresh-token \n",
        )
        .unwrap();

        assert_eq!(inputs.app_id, "app-id");
        assert_eq!(inputs.app_secret, "app-secret");
        assert_eq!(inputs.access_token, "access-token");
        assert_eq!(inputs.refresh_token, "refresh-token");
    }

    #[test]
    fn pinterest_auth_status_payload_includes_credentials() {
        let payload = pinterest_auth_status_payload(PinterestAuthSnapshot {
            app_id: PinterestSecretStatus {
                present: true,
                source: PinterestSecretSource::Keychain,
                keychain_present: true,
            },
            app_secret: PinterestSecretStatus {
                present: true,
                source: PinterestSecretSource::ShellEnv,
                keychain_present: false,
            },
            access_token: PinterestSecretStatus {
                present: true,
                source: PinterestSecretSource::ShellEnv,
                keychain_present: true,
            },
            refresh_token: PinterestSecretStatus {
                present: false,
                source: PinterestSecretSource::Missing,
                keychain_present: false,
            },
            credential_store_available: true,
            credential_store_error: None,
        });

        assert_eq!(payload["provider"], json!("pinterest"));
        assert_eq!(
            payload["credentials"]["app_id"]["source"],
            json!("keychain")
        );
        assert_eq!(
            payload["credentials"]["refresh_token"]["present"],
            json!(false)
        );
    }
}
