use std::env;
use std::path::{Path, PathBuf};

use agent_ads_core::output::{OutputEnvelope, OutputMeta};
use agent_ads_core::secret_store::SecretStore;
use agent_ads_core::tiktok_auth::refresh_access_token;
use agent_ads_core::tiktok_client::{TikTokClient, TikTokResponse};
use agent_ads_core::tiktok_config::{
    tiktok_inspect_access_token, TikTokAccessTokenSource, TikTokAccessTokenStatus,
    TikTokConfigOverrides, TikTokConfigSnapshot, TikTokResolvedConfig, TIKTOK_DEFAULT_API_BASE_URL,
    TIKTOK_DEFAULT_API_VERSION,
};
use agent_ads_core::tiktok_endpoints::{
    accounts, adgroups, ads, audiences, campaigns, creative, pixels, reports,
};
use agent_ads_core::tiktok_error::TikTokError;
use agent_ads_core::{
    TIKTOK_ACCESS_TOKEN_ACCOUNT, TIKTOK_ACCESS_TOKEN_SERVICE, TIKTOK_REFRESH_TOKEN_ACCOUNT,
    TIKTOK_REFRESH_TOKEN_SERVICE,
};
use clap::{Args, Subcommand};
use rpassword::prompt_password;
use serde_json::{json, Value};

use crate::{command_result, read_input, resolve_fields, CommandResult, FieldInputArgs};

// ---------------------------------------------------------------------------
// Clap subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum TikTokCommand {
    #[command(about = "List and inspect advertiser accounts")]
    Advertisers {
        #[command(subcommand)]
        command: AdvertisersCommand,
    },
    #[command(about = "List campaigns")]
    Campaigns {
        #[command(subcommand)]
        command: TikTokListCommand,
    },
    #[command(about = "List ad groups")]
    Adgroups {
        #[command(subcommand)]
        command: TikTokListCommand,
    },
    #[command(about = "List ads")]
    Ads {
        #[command(subcommand)]
        command: TikTokListCommand,
    },
    #[command(about = "Query performance insights (synchronous)")]
    Insights {
        #[command(subcommand)]
        command: InsightsCommand,
    },
    #[command(about = "Manage async report tasks")]
    ReportRuns {
        #[command(subcommand)]
        command: ReportRunsCommand,
    },
    #[command(about = "Search video and image creative assets")]
    Creatives {
        #[command(subcommand)]
        command: CreativesCommand,
    },
    #[command(about = "List tracking pixels")]
    Pixels {
        #[command(subcommand)]
        command: TikTokSimpleListCommand,
    },
    #[command(about = "List custom audiences")]
    Audiences {
        #[command(subcommand)]
        command: TikTokSimpleListCommand,
    },
    #[command(about = "Manage stored auth token")]
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
pub enum AdvertisersCommand {
    #[command(about = "List authorized advertisers", visible_alias = "ls")]
    List(AdvertiserListArgs),
    #[command(about = "Get advertiser account details", visible_alias = "cat")]
    Info(AdvertiserInfoArgs),
}

#[derive(Subcommand, Debug)]
pub enum TikTokListCommand {
    #[command(about = "List objects for an advertiser", visible_alias = "ls")]
    List(TikTokObjectListArgs),
}

#[derive(Subcommand, Debug)]
pub enum TikTokSimpleListCommand {
    #[command(about = "List objects for an advertiser", visible_alias = "ls")]
    List(TikTokSimpleListArgs),
}

#[derive(Subcommand, Debug)]
pub enum InsightsCommand {
    #[command(about = "Run a synchronous insights query")]
    Query(InsightsQueryArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReportRunsCommand {
    #[command(about = "Create an async report task")]
    Submit(ReportSubmitArgs),
    #[command(about = "Check async report task status")]
    Status(ReportStatusArgs),
    #[command(about = "Cancel an async report task")]
    Cancel(ReportCancelArgs),
}

#[derive(Subcommand, Debug)]
pub enum CreativesCommand {
    #[command(about = "Search video assets")]
    Videos(CreativeSearchArgs),
    #[command(about = "Get image info by IDs")]
    Images(CreativeImageArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    #[command(about = "Store the TikTok access token in the OS credential store")]
    Set(AuthSetArgs),
    #[command(about = "Show auth source and secure storage status")]
    Status,
    #[command(about = "Delete the stored TikTok tokens")]
    Delete,
    #[command(about = "Refresh the access token using a stored refresh token")]
    Refresh(AuthRefreshArgs),
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
pub struct TikTokPaginationArgs {
    #[arg(long, help = "Page number (1-indexed)")]
    pub page: Option<u32>,
    #[arg(long = "page-size", help = "Items per page")]
    pub page_size: Option<u32>,
    #[arg(long, help = "Auto-paginate through all results")]
    pub all: bool,
    #[arg(long = "max-items", help = "Stop after collecting N total items")]
    pub max_items: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct AdvertiserListArgs {
    #[arg(long, env = "TIKTOK_ADS_APP_ID", help = "TikTok app ID")]
    pub app_id: String,
    #[arg(long, env = "TIKTOK_ADS_APP_SECRET", help = "TikTok app secret")]
    pub app_secret: String,
    #[command(flatten)]
    pub pagination: TikTokPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AdvertiserInfoArgs {
    #[arg(
        long = "advertiser-id",
        value_delimiter = ',',
        help = "One or more advertiser IDs"
    )]
    pub advertiser_ids: Vec<String>,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct TikTokObjectListArgs {
    #[arg(long = "advertiser-id", help = "Advertiser ID")]
    pub advertiser_id: Option<String>,
    #[arg(long, help = "Inline filter JSON")]
    pub filter: Option<String>,
    #[arg(long, help = "JSON file with filtering object (- for stdin)")]
    pub filter_file: Option<PathBuf>,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
    #[command(flatten)]
    pub pagination: TikTokPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct TikTokSimpleListArgs {
    #[arg(long = "advertiser-id", help = "Advertiser ID")]
    pub advertiser_id: Option<String>,
    #[command(flatten)]
    pub pagination: TikTokPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct InsightsQueryArgs {
    #[arg(long = "advertiser-id", help = "Advertiser ID")]
    pub advertiser_id: Option<String>,
    #[arg(
        long = "report-type",
        help = "Report type (e.g. BASIC, AUDIENCE, PLAYABLE_MATERIAL, CATALOG)"
    )]
    pub report_type: String,
    #[arg(
        long = "data-level",
        help = "Data level (e.g. AUCTION_AD, AUCTION_ADGROUP, AUCTION_CAMPAIGN, AUCTION_ADVERTISER)"
    )]
    pub data_level: Option<String>,
    #[arg(long, value_delimiter = ',', help = "Dimension columns")]
    pub dimensions: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Metric columns")]
    pub metrics: Vec<String>,
    #[arg(long = "start-date", help = "Start date (YYYY-MM-DD)")]
    pub start_date: Option<String>,
    #[arg(long = "end-date", help = "End date (YYYY-MM-DD)")]
    pub end_date: Option<String>,
    #[arg(long, help = "Inline filter JSON")]
    pub filter: Option<String>,
    #[arg(long, help = "JSON file with filtering object (- for stdin)")]
    pub filter_file: Option<PathBuf>,
    #[arg(long = "order-field", help = "Field to sort by")]
    pub order_field: Option<String>,
    #[arg(long = "order-type", help = "Sort direction (ASC or DESC)")]
    pub order_type: Option<String>,
    #[arg(long = "query-lifetime", help = "Query lifetime metrics")]
    pub query_lifetime: bool,
    #[command(flatten)]
    pub pagination: TikTokPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ReportSubmitArgs {
    #[arg(long = "advertiser-id", help = "Advertiser ID")]
    pub advertiser_id: Option<String>,
    #[arg(long = "report-type", help = "Report type")]
    pub report_type: String,
    #[arg(long = "data-level", help = "Data level")]
    pub data_level: Option<String>,
    #[arg(long, value_delimiter = ',', help = "Dimension columns")]
    pub dimensions: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Metric columns")]
    pub metrics: Vec<String>,
    #[arg(long = "start-date", help = "Start date (YYYY-MM-DD)")]
    pub start_date: Option<String>,
    #[arg(long = "end-date", help = "End date (YYYY-MM-DD)")]
    pub end_date: Option<String>,
    #[arg(long, help = "Inline filter JSON")]
    pub filter: Option<String>,
    #[arg(long, help = "JSON file with filtering object (- for stdin)")]
    pub filter_file: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReportStatusArgs {
    #[arg(long = "advertiser-id", help = "Advertiser ID")]
    pub advertiser_id: Option<String>,
    #[arg(long = "task-id", help = "Report task ID")]
    pub task_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct ReportCancelArgs {
    #[arg(long = "advertiser-id", help = "Advertiser ID")]
    pub advertiser_id: Option<String>,
    #[arg(long = "task-id", help = "Report task ID")]
    pub task_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct CreativeSearchArgs {
    #[arg(long = "advertiser-id", help = "Advertiser ID")]
    pub advertiser_id: Option<String>,
    #[arg(long, help = "Inline filter JSON")]
    pub filter: Option<String>,
    #[command(flatten)]
    pub pagination: TikTokPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CreativeImageArgs {
    #[arg(long = "advertiser-id", help = "Advertiser ID")]
    pub advertiser_id: Option<String>,
    #[arg(
        long = "image-id",
        value_delimiter = ',',
        help = "One or more image IDs"
    )]
    pub image_ids: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    #[arg(long, help = "Also ping the TikTok API to verify the token")]
    pub api: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AuthSetArgs {
    #[arg(long, help = "Read the token from stdin instead of prompting")]
    pub stdin: bool,
    #[arg(long = "refresh-token", help = "Also store a refresh token")]
    pub refresh_token: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AuthRefreshArgs {
    #[arg(long, env = "TIKTOK_ADS_APP_ID", help = "TikTok app ID")]
    pub app_id: String,
    #[arg(long, env = "TIKTOK_ADS_APP_SECRET", help = "TikTok app secret")]
    pub app_secret: String,
}

// ---------------------------------------------------------------------------
// Dispatch: auth, config, doctor (before token resolution)
// ---------------------------------------------------------------------------

pub fn handle_auth(
    command: AuthCommand,
    secret_store: &dyn SecretStore,
) -> Result<CommandResult, TikTokError> {
    match command {
        AuthCommand::Set(args) => {
            let token = resolve_tiktok_auth_token_input(&args)?;
            secret_store
                .set_secret(
                    TIKTOK_ACCESS_TOKEN_SERVICE,
                    TIKTOK_ACCESS_TOKEN_ACCOUNT,
                    &token,
                )
                .map_err(|error| tiktok_auth_storage_error("store access token", &error))?;

            let mut result = json!({
                "provider": "tiktok",
                "stored": true,
                "credential_store_service": TIKTOK_ACCESS_TOKEN_SERVICE,
                "credential_store_account": TIKTOK_ACCESS_TOKEN_ACCOUNT,
            });

            if args.refresh_token {
                let refresh = if args.stdin {
                    read_input(Path::new("-")).map_err(|e| TikTokError::Config(e.to_string()))?
                } else {
                    prompt_password("TikTok refresh token: ").map_err(TikTokError::Io)?
                };
                let refresh = refresh.trim().to_string();
                if !refresh.is_empty() {
                    secret_store
                        .set_secret(
                            TIKTOK_REFRESH_TOKEN_SERVICE,
                            TIKTOK_REFRESH_TOKEN_ACCOUNT,
                            &refresh,
                        )
                        .map_err(|error| {
                            tiktok_auth_storage_error("store refresh token", &error)
                        })?;
                    result["refresh_token_stored"] = json!(true);
                }
            }

            Ok(tiktok_command_result(result, "/tiktok/auth/set", 0))
        }
        AuthCommand::Status => Ok(tiktok_command_result(
            tiktok_auth_status_payload(tiktok_inspect_access_token(secret_store)),
            "/tiktok/auth/status",
            0,
        )),
        AuthCommand::Delete => {
            let deleted_access = secret_store
                .delete_secret(TIKTOK_ACCESS_TOKEN_SERVICE, TIKTOK_ACCESS_TOKEN_ACCOUNT)
                .map_err(|error| tiktok_auth_storage_error("delete access token", &error))?;
            let deleted_refresh = secret_store
                .delete_secret(TIKTOK_REFRESH_TOKEN_SERVICE, TIKTOK_REFRESH_TOKEN_ACCOUNT)
                .map_err(|error| tiktok_auth_storage_error("delete refresh token", &error))?;

            Ok(tiktok_command_result(
                json!({
                    "provider": "tiktok",
                    "access_token_deleted": deleted_access,
                    "refresh_token_deleted": deleted_refresh,
                }),
                "/tiktok/auth/delete",
                0,
            ))
        }
        AuthCommand::Refresh(_) => {
            unreachable!("auth refresh is dispatched separately as an async operation")
        }
    }
}

/// Async handler for auth refresh, called from main.
pub async fn handle_auth_refresh(
    app_id: &str,
    app_secret: &str,
    refresh_token: &str,
    secret_store: &dyn SecretStore,
    overrides: &TikTokConfigOverrides,
) -> Result<CommandResult, TikTokError> {
    let env_base = env::var("TIKTOK_ADS_API_BASE_URL").ok();
    let api_base_url = overrides
        .api_base_url
        .as_deref()
        .or(env_base.as_deref())
        .unwrap_or(TIKTOK_DEFAULT_API_BASE_URL);
    let env_ver = env::var("TIKTOK_ADS_API_VERSION").ok();
    let api_version = overrides
        .api_version
        .as_deref()
        .or(env_ver.as_deref())
        .unwrap_or(TIKTOK_DEFAULT_API_VERSION);

    let result =
        refresh_access_token(api_base_url, api_version, app_id, app_secret, refresh_token).await?;

    // Store the new access token
    secret_store
        .set_secret(
            TIKTOK_ACCESS_TOKEN_SERVICE,
            TIKTOK_ACCESS_TOKEN_ACCOUNT,
            &result.access_token,
        )
        .map_err(|error| tiktok_auth_storage_error("store refreshed access token", &error))?;

    // Store the new refresh token if returned
    if let Some(new_refresh) = &result.refresh_token {
        secret_store
            .set_secret(
                TIKTOK_REFRESH_TOKEN_SERVICE,
                TIKTOK_REFRESH_TOKEN_ACCOUNT,
                new_refresh,
            )
            .map_err(|error| tiktok_auth_storage_error("store new refresh token", &error))?;
    }

    Ok(tiktok_command_result(
        json!({
            "provider": "tiktok",
            "refreshed": true,
            "access_token_expire_in": result.access_token_expire_in,
            "refresh_token_expire_in": result.refresh_token_expire_in,
            "new_refresh_token_stored": result.refresh_token.is_some(),
        }),
        "/tiktok/auth/refresh",
        0,
    ))
}

pub fn handle_config(
    command: ConfigCommand,
    snapshot: TikTokConfigSnapshot,
) -> Result<CommandResult, TikTokError> {
    match command {
        ConfigCommand::Path => Ok(tiktok_command_result(
            json!({
                "path": snapshot.config_path,
                "exists": snapshot.config_file_exists,
            }),
            "/tiktok/config/path",
            0,
        )),
        ConfigCommand::Show => Ok(tiktok_command_result(
            json!(snapshot),
            "/tiktok/config/show",
            0,
        )),
        ConfigCommand::Validate => Ok(tiktok_command_result(
            json!({
                "valid": true,
                "config": snapshot,
            }),
            "/tiktok/config/validate",
            0,
        )),
    }
}

pub async fn handle_doctor(
    args: DoctorArgs,
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &TikTokConfigOverrides,
    snapshot: TikTokConfigSnapshot,
) -> Result<CommandResult, TikTokError> {
    let mut checks = vec![
        json!({
            "name": "credential_store",
            "ok": tiktok_credential_store_check_ok(&snapshot),
            "detail": tiktok_credential_store_detail(&snapshot),
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
            "name": "access_token",
            "ok": snapshot.access_token_present,
            "detail": tiktok_access_token_detail(&snapshot),
        }),
    ];

    let mut ok = snapshot.access_token_present;
    if args.api {
        if snapshot.access_token_present {
            match TikTokResolvedConfig::load(config_path, secret_store, overrides)
                .and_then(|config| TikTokClient::from_config(&config))
            {
                Ok(client) => {
                    // Try a simple API call — get advertiser info requires IDs,
                    // so we'll just verify the token by making any authenticated request.
                    // Use a simple pixel list with page_size=1 as a health check.
                    let mut params = std::collections::BTreeMap::new();
                    // We need an advertiser_id for most endpoints, but we might not have one.
                    // If default_advertiser_id is set, use it.
                    if let Some(adv_id) = &snapshot.default_advertiser_id {
                        params.insert("advertiser_id".to_string(), adv_id.clone());
                        params.insert("page_size".to_string(), "1".to_string());
                        match client.get("campaign/get", &params).await {
                            Ok(_response) => {
                                checks.push(json!({
                                    "name": "api_ping",
                                    "ok": true,
                                    "detail": "token accepted by TikTok API"
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
                        }
                    } else {
                        checks.push(json!({
                            "name": "api_ping",
                            "ok": true,
                            "detail": "token present but no default_advertiser_id to test; skipped live check"
                        }));
                    }
                }
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
                "detail": "skipped because TIKTOK_ADS_ACCESS_TOKEN is missing"
            }));
        }
    }

    Ok(tiktok_command_result(
        json!({
            "ok": ok,
            "checks": checks,
            "config": snapshot,
        }),
        "/tiktok/doctor",
        if ok { 0 } else { 1 },
    ))
}

// ---------------------------------------------------------------------------
// Dispatch: data commands (require client)
// ---------------------------------------------------------------------------

pub async fn dispatch_tiktok_with_client(
    client: &TikTokClient,
    config: &TikTokResolvedConfig,
    command: TikTokCommand,
) -> Result<CommandResult, TikTokError> {
    match command {
        TikTokCommand::Advertisers { command } => match command {
            AdvertisersCommand::List(args) => {
                let response = accounts::list_advertisers(
                    client,
                    &args.app_id,
                    &args.app_secret,
                    args.pagination.page,
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/oauth2/advertiser/get",
                    None,
                    Vec::new(),
                ))
            }
            AdvertisersCommand::Info(args) => {
                let ids = if args.advertiser_ids.is_empty() {
                    vec![resolve_advertiser_id(config, None)?]
                } else {
                    args.advertiser_ids.clone()
                };
                let fields = resolve_fields(&args.field_input)
                    .map_err(|e| TikTokError::Config(e.to_string()))?;
                let response = accounts::get_advertiser_info(client, &ids, &fields).await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/advertiser/info",
                    config.default_advertiser_id.clone(),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::Campaigns { command } => match command {
            TikTokListCommand::List(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let fields = resolve_fields(&args.field_input)
                    .map_err(|e| TikTokError::Config(e.to_string()))?;
                let filtering =
                    resolve_tiktok_filter(args.filter.as_deref(), args.filter_file.as_deref())?;
                let response = campaigns::list_campaigns(
                    client,
                    &advertiser_id,
                    &fields,
                    filtering.as_ref(),
                    args.pagination.page,
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/campaign/get",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::Adgroups { command } => match command {
            TikTokListCommand::List(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let fields = resolve_fields(&args.field_input)
                    .map_err(|e| TikTokError::Config(e.to_string()))?;
                let filtering =
                    resolve_tiktok_filter(args.filter.as_deref(), args.filter_file.as_deref())?;
                let response = adgroups::list_adgroups(
                    client,
                    &advertiser_id,
                    &fields,
                    filtering.as_ref(),
                    args.pagination.page,
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/adgroup/get",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::Ads { command } => match command {
            TikTokListCommand::List(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let fields = resolve_fields(&args.field_input)
                    .map_err(|e| TikTokError::Config(e.to_string()))?;
                let filtering =
                    resolve_tiktok_filter(args.filter.as_deref(), args.filter_file.as_deref())?;
                let response = ads::list_ads(
                    client,
                    &advertiser_id,
                    &fields,
                    filtering.as_ref(),
                    args.pagination.page,
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/ad/get",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::Insights { command } => match command {
            InsightsCommand::Query(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let filtering =
                    resolve_tiktok_filter(args.filter.as_deref(), args.filter_file.as_deref())?;
                let response = reports::query_insights(
                    client,
                    reports::TikTokInsightsQuery {
                        advertiser_id: &advertiser_id,
                        report_type: &args.report_type,
                        data_level: args.data_level.as_deref(),
                        dimensions: &args.dimensions,
                        metrics: &args.metrics,
                        start_date: args.start_date.as_deref(),
                        end_date: args.end_date.as_deref(),
                        filtering: filtering.as_ref(),
                        order_field: args.order_field.as_deref(),
                        order_type: args.order_type.as_deref(),
                        query_lifetime: Some(args.query_lifetime),
                        page: args.pagination.page,
                        page_size: args.pagination.page_size,
                        fetch_all: args.pagination.all,
                        max_items: args.pagination.max_items,
                    },
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/report/integrated/get",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::ReportRuns { command } => match command {
            ReportRunsCommand::Submit(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let filtering =
                    resolve_tiktok_filter(args.filter.as_deref(), args.filter_file.as_deref())?;
                let response = reports::create_report_task(
                    client,
                    &advertiser_id,
                    &args.report_type,
                    args.data_level.as_deref(),
                    &args.dimensions,
                    &args.metrics,
                    args.start_date.as_deref(),
                    args.end_date.as_deref(),
                    filtering.as_ref(),
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/report/task/create",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
            ReportRunsCommand::Status(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let response =
                    reports::check_report_task(client, &advertiser_id, &args.task_id).await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/report/task/check",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
            ReportRunsCommand::Cancel(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let response =
                    reports::cancel_report_task(client, &advertiser_id, &args.task_id).await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/report/task/cancel",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::Creatives { command } => match command {
            CreativesCommand::Videos(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let filtering = resolve_tiktok_filter(args.filter.as_deref(), None)?;
                let response = creative::search_videos(
                    client,
                    &advertiser_id,
                    filtering.as_ref(),
                    args.pagination.page,
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/file/video/ad/search",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
            CreativesCommand::Images(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let response =
                    creative::get_images(client, &advertiser_id, &args.image_ids).await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/file/image/ad/info",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::Pixels { command } => match command {
            TikTokSimpleListCommand::List(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let response = pixels::list_pixels(
                    client,
                    &advertiser_id,
                    args.pagination.page,
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/pixel/list",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::Audiences { command } => match command {
            TikTokSimpleListCommand::List(args) => {
                let advertiser_id = resolve_advertiser_id(config, args.advertiser_id.as_deref())?;
                let response = audiences::list_audiences(
                    client,
                    &advertiser_id,
                    args.pagination.page,
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(tiktok_result(
                    client,
                    response,
                    "/dmp/custom_audience/list",
                    Some(advertiser_id),
                    Vec::new(),
                ))
            }
        },
        TikTokCommand::Auth { .. } | TikTokCommand::Config { .. } | TikTokCommand::Doctor(_) => {
            unreachable!("handled before token resolution")
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn tiktok_result(
    client: &TikTokClient,
    response: TikTokResponse,
    endpoint: &str,
    advertiser_id: Option<String>,
    warnings: Vec<String>,
) -> CommandResult {
    let mut envelope = OutputEnvelope::new(
        response.data,
        OutputMeta {
            api_version: client.api_version().to_string(),
            endpoint: endpoint.to_string(),
            object_id: advertiser_id,
            request_id: response.request_id,
            report_run_id: None,
        },
    );
    if !warnings.is_empty() {
        envelope.warnings = Some(warnings);
    }
    CommandResult {
        envelope,
        exit_code: 0,
    }
}

fn tiktok_command_result(data: Value, endpoint: &str, exit_code: u8) -> CommandResult {
    command_result(data, endpoint, exit_code, Some(TIKTOK_DEFAULT_API_VERSION))
}

fn resolve_advertiser_id(
    config: &TikTokResolvedConfig,
    value: Option<&str>,
) -> Result<String, TikTokError> {
    value
        .map(str::to_string)
        .or_else(|| config.default_advertiser_id.clone())
        .ok_or_else(|| {
            TikTokError::InvalidArgument(
                "advertiser id is required; pass --advertiser-id or set TIKTOK_ADS_DEFAULT_ADVERTISER_ID"
                    .to_string(),
            )
        })
}

fn resolve_tiktok_filter(
    inline: Option<&str>,
    file: Option<&Path>,
) -> Result<Option<Value>, TikTokError> {
    if let Some(filter) = inline {
        let value: Value = serde_json::from_str(filter)
            .map_err(|e| TikTokError::InvalidArgument(format!("invalid filter JSON: {e}")))?;
        return Ok(Some(value));
    }
    if let Some(path) = file {
        let content = read_input(path).map_err(|e| TikTokError::Config(e.to_string()))?;
        let value: Value = serde_json::from_str(&content).map_err(|e| {
            TikTokError::InvalidArgument(format!("invalid filter JSON in file: {e}"))
        })?;
        return Ok(Some(value));
    }
    Ok(None)
}

fn resolve_tiktok_auth_token_input(args: &AuthSetArgs) -> Result<String, TikTokError> {
    let token = if args.stdin {
        read_input(Path::new("-")).map_err(|e| TikTokError::Config(e.to_string()))?
    } else {
        prompt_password("TikTok access token: ").map_err(TikTokError::Io)?
    };

    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(TikTokError::InvalidArgument(
            "token input was empty".to_string(),
        ));
    }

    Ok(token)
}

fn tiktok_auth_storage_error(action: &str, error: &impl std::fmt::Display) -> TikTokError {
    TikTokError::Config(format!(
        "failed to {action} in the OS credential store: {error}{}",
        tiktok_linux_secure_storage_hint()
    ))
}

fn tiktok_linux_secure_storage_hint() -> &'static str {
    if cfg!(target_os = "linux") {
        " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet."
    } else {
        ""
    }
}

fn tiktok_credential_store_check_ok(snapshot: &TikTokConfigSnapshot) -> bool {
    snapshot.credential_store_available
        || snapshot.access_token_source == TikTokAccessTokenSource::ShellEnv
}

fn tiktok_credential_store_detail(snapshot: &TikTokConfigSnapshot) -> String {
    match snapshot.credential_store_error.as_deref() {
        Some(error) if snapshot.access_token_source == TikTokAccessTokenSource::ShellEnv => {
            format!("shell env override active; OS credential store unavailable: {error}")
        }
        Some(error) => format!("OS credential store unavailable: {error}"),
        None if snapshot.keychain_token_present => {
            "stored TikTok token found in OS credential store".to_string()
        }
        None if snapshot.credential_store_available => {
            "OS credential store is available; no stored TikTok token found".to_string()
        }
        None => "OS credential store is unavailable".to_string(),
    }
}

fn tiktok_access_token_detail(snapshot: &TikTokConfigSnapshot) -> String {
    match snapshot.access_token_source {
        TikTokAccessTokenSource::ShellEnv if snapshot.keychain_token_present => {
            "TIKTOK_ADS_ACCESS_TOKEN is set in shell env and overrides the stored token".to_string()
        }
        TikTokAccessTokenSource::ShellEnv => {
            "TIKTOK_ADS_ACCESS_TOKEN is set in shell env".to_string()
        }
        TikTokAccessTokenSource::Keychain => {
            "using stored TikTok token from the OS credential store".to_string()
        }
        TikTokAccessTokenSource::Missing => match snapshot.credential_store_error.as_deref() {
            Some(error) => format!("TIKTOK_ADS_ACCESS_TOKEN is missing; {error}"),
            None => "TIKTOK_ADS_ACCESS_TOKEN is missing".to_string(),
        },
    }
}

fn tiktok_auth_status_payload(status: TikTokAccessTokenStatus) -> Value {
    json!({
        "provider": "tiktok",
        "credential_store_service": TIKTOK_ACCESS_TOKEN_SERVICE,
        "credential_store_account": TIKTOK_ACCESS_TOKEN_ACCOUNT,
        "access_token_present": status.access_token_present,
        "access_token_source": status.access_token_source,
        "credential_store_available": status.credential_store_available,
        "keychain_token_present": status.keychain_token_present,
        "credential_store_error": status.credential_store_error,
    })
}
