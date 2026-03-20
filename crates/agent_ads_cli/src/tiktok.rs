use std::env;
use std::path::{Path, PathBuf};

use agent_ads_core::output::{OutputEnvelope, OutputMeta};
use agent_ads_core::secret_store::SecretStore;
use agent_ads_core::tiktok_auth::refresh_access_token;
use agent_ads_core::tiktok_client::{TikTokClient, TikTokResponse};
use agent_ads_core::tiktok_config::{
    tiktok_inspect_auth, TikTokAccessTokenSource, TikTokAuthSnapshot, TikTokConfigOverrides,
    TikTokConfigSnapshot, TikTokResolvedConfig, TikTokSecretStatus,
    TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR, TIKTOK_ADS_APP_ID_ENV_VAR, TIKTOK_ADS_APP_SECRET_ENV_VAR,
    TIKTOK_ADS_REFRESH_TOKEN_ENV_VAR, TIKTOK_DEFAULT_API_VERSION,
};
use agent_ads_core::tiktok_endpoints::{
    accounts, adgroups, ads, audiences, campaigns, creative, pixels, reports,
};
use agent_ads_core::tiktok_error::TikTokError;
use agent_ads_core::{
    load_auth_bundle, store_auth_bundle, AUTH_BUNDLE_ACCOUNT, AUTH_BUNDLE_SERVICE,
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
    #[command(about = "Manage stored auth credentials")]
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
    #[command(about = "Store TikTok auth credentials in the OS credential store")]
    Set(AuthSetArgs),
    #[command(about = "Show auth source and secure storage status")]
    Status,
    #[command(about = "Delete the stored TikTok credentials")]
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
    #[arg(long, help = "Read auth input from stdin instead of prompting")]
    pub stdin: bool,
    #[arg(
        long = "refresh-token",
        conflicts_with = "full",
        help = "Also store a refresh token"
    )]
    pub refresh_token: bool,
    #[arg(
        long,
        help = "Store app ID, app secret, access token, and optionally refresh token"
    )]
    pub full: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AuthRefreshArgs {
    #[arg(long, env = "TIKTOK_ADS_APP_ID", help = "TikTok app ID")]
    pub app_id: Option<String>,
    #[arg(long, env = "TIKTOK_ADS_APP_SECRET", help = "TikTok app secret")]
    pub app_secret: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedAuthRefreshInputs {
    pub app_id: String,
    pub app_secret: String,
    pub refresh_token: String,
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
            let inputs = resolve_tiktok_auth_inputs(&args)?;
            let mut bundle = load_auth_bundle(secret_store)
                .map_err(|error| tiktok_auth_storage_error("store TikTok credentials", &error))?;
            let mut tiktok_bundle = bundle.tiktok.take().unwrap_or_default();

            let mut credentials_stored = vec!["access_token"];
            tiktok_bundle.access_token = Some(inputs.access_token);
            if let Some(app_id) = inputs.app_id {
                tiktok_bundle.app_id = Some(app_id);
                credentials_stored.push("app_id");
            }
            if let Some(app_secret) = inputs.app_secret {
                tiktok_bundle.app_secret = Some(app_secret);
                credentials_stored.push("app_secret");
            }
            if let Some(refresh_token) = inputs.refresh_token {
                tiktok_bundle.refresh_token = Some(refresh_token);
                credentials_stored.push("refresh_token");
            }
            bundle.tiktok = Some(tiktok_bundle);
            store_auth_bundle(secret_store, &bundle)
                .map_err(|error| tiktok_auth_storage_error("store TikTok credentials", &error))?;

            Ok(tiktok_command_result(
                json!({
                    "provider": "tiktok",
                    "stored": true,
                    "credentials_stored": credentials_stored,
                }),
                "/tiktok/auth/set",
                0,
            ))
        }
        AuthCommand::Status => Ok(tiktok_command_result(
            tiktok_auth_status_payload(tiktok_inspect_auth(secret_store)),
            "/tiktok/auth/status",
            0,
        )),
        AuthCommand::Delete => {
            let mut bundle = load_auth_bundle(secret_store)
                .map_err(|error| tiktok_auth_storage_error("delete TikTok credentials", &error))?;
            let deleted_tiktok = bundle.tiktok.take();
            let deleted_app_id = deleted_tiktok
                .as_ref()
                .and_then(|tiktok| tiktok.app_id.as_ref())
                .is_some();
            let deleted_app_secret = deleted_tiktok
                .as_ref()
                .and_then(|tiktok| tiktok.app_secret.as_ref())
                .is_some();
            let deleted_access = deleted_tiktok
                .as_ref()
                .and_then(|tiktok| tiktok.access_token.as_ref())
                .is_some();
            let deleted_refresh = deleted_tiktok
                .as_ref()
                .and_then(|tiktok| tiktok.refresh_token.as_ref())
                .is_some();
            store_auth_bundle(secret_store, &bundle)
                .map_err(|error| tiktok_auth_storage_error("delete TikTok credentials", &error))?;

            Ok(tiktok_command_result(
                json!({
                    "provider": "tiktok",
                    "app_id_deleted": deleted_app_id,
                    "app_secret_deleted": deleted_app_secret,
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
    snapshot: &TikTokConfigSnapshot,
) -> Result<CommandResult, TikTokError> {
    let result = refresh_access_token(
        &snapshot.api_base_url,
        &snapshot.api_version,
        snapshot.timeout_seconds,
        app_id,
        app_secret,
        refresh_token,
    )
    .await?;

    let mut bundle = load_auth_bundle(secret_store)
        .map_err(|error| tiktok_auth_storage_error("store refreshed access token", &error))?;
    let mut tiktok_bundle = bundle.tiktok.take().unwrap_or_default();
    tiktok_bundle.access_token = Some(result.access_token.clone());

    if let Some(new_refresh) = &result.refresh_token {
        tiktok_bundle.refresh_token = Some(new_refresh.clone());
    }
    bundle.tiktok = Some(tiktok_bundle);
    store_auth_bundle(secret_store, &bundle)
        .map_err(|error| tiktok_auth_storage_error("store refreshed access token", &error))?;

    Ok(command_result(
        json!({
            "provider": "tiktok",
            "refreshed": true,
            "access_token_expire_in": result.access_token_expire_in,
            "refresh_token_expire_in": result.refresh_token_expire_in,
            "new_refresh_token_stored": result.refresh_token.is_some(),
        }),
        "/tiktok/auth/refresh",
        0,
        Some(&snapshot.api_version),
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
    envelope.paging = response.page_info.map(|page_info| json!(page_info));
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

#[derive(Debug, Clone)]
struct TikTokAuthInputs {
    app_id: Option<String>,
    app_secret: Option<String>,
    access_token: String,
    refresh_token: Option<String>,
}

fn resolve_tiktok_auth_inputs(args: &AuthSetArgs) -> Result<TikTokAuthInputs, TikTokError> {
    if args.full {
        if args.stdin {
            let input =
                read_input(Path::new("-")).map_err(|e| TikTokError::Config(e.to_string()))?;
            return parse_tiktok_full_auth_inputs_from_stdin(&input);
        }

        return Ok(TikTokAuthInputs {
            app_id: Some(normalize_tiktok_value(
                &prompt_password("TikTok app ID: ").map_err(TikTokError::Io)?,
                "app ID",
            )?),
            app_secret: Some(normalize_tiktok_value(
                &prompt_password("TikTok app secret: ").map_err(TikTokError::Io)?,
                "app secret",
            )?),
            access_token: normalize_tiktok_value(
                &prompt_password("TikTok access token: ").map_err(TikTokError::Io)?,
                "access token",
            )?,
            refresh_token: normalize_optional_tiktok_value(
                &prompt_password("TikTok refresh token (optional): ").map_err(TikTokError::Io)?,
                "refresh token",
            )?,
        });
    }

    if args.stdin {
        let input = read_input(Path::new("-")).map_err(|e| TikTokError::Config(e.to_string()))?;
        return parse_tiktok_auth_inputs_from_stdin(&input, args.refresh_token);
    }

    let access_token = normalize_tiktok_value(
        &prompt_password("TikTok access token: ").map_err(TikTokError::Io)?,
        "access token",
    )?;
    let refresh_token = if args.refresh_token {
        Some(normalize_tiktok_value(
            &prompt_password("TikTok refresh token: ").map_err(TikTokError::Io)?,
            "refresh token",
        )?)
    } else {
        None
    };

    Ok(TikTokAuthInputs {
        app_id: None,
        app_secret: None,
        access_token,
        refresh_token,
    })
}

fn parse_tiktok_auth_inputs_from_stdin(
    input: &str,
    expect_refresh_token: bool,
) -> Result<TikTokAuthInputs, TikTokError> {
    if !expect_refresh_token {
        return Ok(TikTokAuthInputs {
            app_id: None,
            app_secret: None,
            access_token: normalize_tiktok_value(input, "access token")?,
            refresh_token: None,
        });
    }

    let tokens: Vec<&str> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    match tokens.as_slice() {
        [access_token, refresh_token] => Ok(TikTokAuthInputs {
            app_id: None,
            app_secret: None,
            access_token: normalize_tiktok_value(access_token, "access token")?,
            refresh_token: Some(normalize_tiktok_value(refresh_token, "refresh token")?),
        }),
        [] => Err(TikTokError::InvalidArgument(
            "stdin did not contain an access token".to_string(),
        )),
        [_] => Err(TikTokError::InvalidArgument(
            "stdin did not contain a refresh token; provide the access token on the first line and the refresh token on the second".to_string(),
        )),
        _ => Err(TikTokError::InvalidArgument(
            "stdin contained too many non-empty lines; expected access token on the first line and refresh token on the second".to_string(),
        )),
    }
}

fn parse_tiktok_full_auth_inputs_from_stdin(input: &str) -> Result<TikTokAuthInputs, TikTokError> {
    let tokens: Vec<&str> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    match tokens.as_slice() {
        [app_id, app_secret, access_token] => Ok(TikTokAuthInputs {
            app_id: Some(normalize_tiktok_value(app_id, "app ID")?),
            app_secret: Some(normalize_tiktok_value(app_secret, "app secret")?),
            access_token: normalize_tiktok_value(access_token, "access token")?,
            refresh_token: None,
        }),
        [app_id, app_secret, access_token, refresh_token] => Ok(TikTokAuthInputs {
            app_id: Some(normalize_tiktok_value(app_id, "app ID")?),
            app_secret: Some(normalize_tiktok_value(app_secret, "app secret")?),
            access_token: normalize_tiktok_value(access_token, "access token")?,
            refresh_token: Some(normalize_tiktok_value(refresh_token, "refresh token")?),
        }),
        _ => Err(TikTokError::InvalidArgument(
            "stdin must contain three or four non-empty lines: app ID, app secret, access token, and optional refresh token".to_string(),
        )),
    }
}

pub fn resolve_auth_refresh_inputs(
    args: &AuthRefreshArgs,
    secret_store: &dyn SecretStore,
) -> Result<ResolvedAuthRefreshInputs, TikTokError> {
    let bundle_result = load_auth_bundle(secret_store);
    let bundle = bundle_result.as_ref().ok();
    let store_error = bundle_result.as_ref().err().cloned();

    Ok(ResolvedAuthRefreshInputs {
        app_id: resolve_refresh_secret(
            args.app_id.as_deref(),
            TIKTOK_ADS_APP_ID_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.tiktok.as_ref())
                .and_then(|tiktok| tiktok.app_id.clone()),
            "app ID",
            "Pass --app-id, export TIKTOK_ADS_APP_ID, or run `agent-ads tiktok auth set --full` first.",
            store_error.clone(),
        )?,
        app_secret: resolve_refresh_secret(
            args.app_secret.as_deref(),
            TIKTOK_ADS_APP_SECRET_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.tiktok.as_ref())
                .and_then(|tiktok| tiktok.app_secret.clone()),
            "app secret",
            "Pass --app-secret, export TIKTOK_ADS_APP_SECRET, or run `agent-ads tiktok auth set --full` first.",
            store_error.clone(),
        )?,
        refresh_token: resolve_refresh_secret(
            None,
            TIKTOK_ADS_REFRESH_TOKEN_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.tiktok.as_ref())
                .and_then(|tiktok| tiktok.refresh_token.clone()),
            "refresh token",
            "Export TIKTOK_ADS_REFRESH_TOKEN or run `agent-ads tiktok auth set --full` or `agent-ads tiktok auth set --refresh-token` first.",
            store_error,
        )?,
    })
}

fn resolve_refresh_secret(
    direct_value: Option<&str>,
    env_var: &str,
    keychain_value: Option<String>,
    label: &str,
    missing_guidance: &str,
    store_error: Option<agent_ads_core::SecretStoreError>,
) -> Result<String, TikTokError> {
    if let Some(value) = direct_value {
        return normalize_tiktok_value(value, label);
    }

    if let Some(value) = env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok(value);
    }

    match (keychain_value, store_error) {
        (Some(value), None) => Ok(value),
        (None, None) => Err(TikTokError::Config(format!(
            "{env_var} is missing and no TikTok {label} was found in the OS credential store. {missing_guidance}"
        ))),
        (None, Some(error)) => Err(TikTokError::Config(format!(
            "{env_var} is missing and the OS credential store could not be read: {error}. {missing_guidance}"
        ))),
        (Some(value), Some(_)) => Ok(value),
    }
}

fn normalize_tiktok_value(value: &str, token_kind: &str) -> Result<String, TikTokError> {
    let token = value.trim().to_string();
    if token.is_empty() {
        return Err(TikTokError::InvalidArgument(format!(
            "{token_kind} input was empty"
        )));
    }

    Ok(token)
}

fn normalize_optional_tiktok_value(
    value: &str,
    token_kind: &str,
) -> Result<Option<String>, TikTokError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }

    Ok(Some(normalize_tiktok_value(value, token_kind)?))
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
            format!(
                "{TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR} is set in shell env and overrides the stored token"
            )
        }
        TikTokAccessTokenSource::ShellEnv => {
            format!("{TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR} is set in shell env")
        }
        TikTokAccessTokenSource::Keychain => {
            "using stored TikTok token from the OS credential store".to_string()
        }
        TikTokAccessTokenSource::Missing => match snapshot.credential_store_error.as_deref() {
            Some(error) => format!("{TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR} is missing; {error}"),
            None => format!("{TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR} is missing"),
        },
    }
}

fn tiktok_secret_detail(env_var: &str, label: &str, status: &TikTokSecretStatus) -> String {
    match status.source {
        agent_ads_core::TikTokSecretSource::ShellEnv if status.keychain_present => {
            format!("{env_var} is set in shell env and overrides the stored {label}")
        }
        agent_ads_core::TikTokSecretSource::ShellEnv => format!("{env_var} is set in shell env"),
        agent_ads_core::TikTokSecretSource::Keychain => {
            format!("using stored TikTok {label} from the OS credential store")
        }
        agent_ads_core::TikTokSecretSource::Missing => format!("{env_var} is missing"),
    }
}

fn tiktok_auth_status_payload(auth: TikTokAuthSnapshot) -> Value {
    json!({
        "provider": "tiktok",
        "credential_store_available": auth.credential_store_available,
        "credential_store_error": auth.credential_store_error,
        "credentials": {
            "app_id": {
                "env_var": TIKTOK_ADS_APP_ID_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.app_id.present,
                "source": auth.app_id.source,
                "keychain_present": auth.app_id.keychain_present,
                "detail": tiktok_secret_detail(TIKTOK_ADS_APP_ID_ENV_VAR, "app ID", &auth.app_id),
            },
            "app_secret": {
                "env_var": TIKTOK_ADS_APP_SECRET_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.app_secret.present,
                "source": auth.app_secret.source,
                "keychain_present": auth.app_secret.keychain_present,
                "detail": tiktok_secret_detail(TIKTOK_ADS_APP_SECRET_ENV_VAR, "app secret", &auth.app_secret),
            },
            "access_token": {
                "env_var": TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.access_token.present,
                "source": auth.access_token.source,
                "keychain_present": auth.access_token.keychain_present,
                "detail": tiktok_secret_detail(TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR, "access token", &auth.access_token),
            },
            "refresh_token": {
                "env_var": TIKTOK_ADS_REFRESH_TOKEN_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.refresh_token.present,
                "source": auth.refresh_token.source,
                "keychain_present": auth.refresh_token.keychain_present,
                "detail": tiktok_secret_detail(TIKTOK_ADS_REFRESH_TOKEN_ENV_VAR, "refresh token", &auth.refresh_token),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use agent_ads_core::secret_store::{SecretStore, SecretStoreError};
    use agent_ads_core::{load_auth_bundle, store_auth_bundle, AuthBundle, TikTokAuthBundle};
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
    }

    impl SecretStore for FakeSecretStore {
        fn get_secret(
            &self,
            service: &str,
            account: &str,
        ) -> std::result::Result<Option<String>, SecretStoreError> {
            Ok(self
                .secrets
                .lock()
                .unwrap()
                .get(&(service.to_string(), account.to_string()))
                .cloned())
        }

        fn set_secret(
            &self,
            service: &str,
            account: &str,
            secret: &str,
        ) -> std::result::Result<(), SecretStoreError> {
            self.secrets.lock().unwrap().insert(
                (service.to_string(), account.to_string()),
                secret.to_string(),
            );
            Ok(())
        }

        fn delete_secret(
            &self,
            service: &str,
            account: &str,
        ) -> std::result::Result<bool, SecretStoreError> {
            Ok(self
                .secrets
                .lock()
                .unwrap()
                .remove(&(service.to_string(), account.to_string()))
                .is_some())
        }
    }

    fn test_snapshot(
        api_base_url: &str,
        api_version: &str,
        timeout_seconds: u64,
    ) -> TikTokConfigSnapshot {
        TikTokConfigSnapshot {
            config_path: PathBuf::from("agent-ads.config.json"),
            config_file_exists: true,
            access_token_present: false,
            access_token_source: TikTokAccessTokenSource::Missing,
            credential_store_available: true,
            keychain_token_present: false,
            credential_store_error: None,
            api_base_url: api_base_url.to_string(),
            api_version: api_version.to_string(),
            timeout_seconds,
            default_advertiser_id: None,
            output_format: agent_ads_core::output::OutputFormat::Json,
        }
    }

    #[test]
    fn parse_stdin_auth_inputs_supports_access_token_only() {
        let inputs = parse_tiktok_auth_inputs_from_stdin("  access-token  \n", false).unwrap();

        assert_eq!(inputs.access_token, "access-token");
        assert_eq!(inputs.refresh_token, None);
        assert_eq!(inputs.app_id, None);
    }

    #[test]
    fn parse_stdin_auth_inputs_requires_refresh_token_when_requested() {
        let error = parse_tiktok_auth_inputs_from_stdin("access-token\n", true).unwrap_err();

        assert!(error
            .to_string()
            .contains("stdin did not contain a refresh token"));
    }

    #[test]
    fn parse_stdin_auth_inputs_reads_access_and_refresh_tokens() {
        let inputs =
            parse_tiktok_auth_inputs_from_stdin(" access-token \n refresh-token \n", true).unwrap();

        assert_eq!(inputs.access_token, "access-token");
        assert_eq!(inputs.refresh_token.as_deref(), Some("refresh-token"));
    }

    #[test]
    fn parse_full_stdin_auth_inputs_reads_all_credentials() {
        let inputs = parse_tiktok_full_auth_inputs_from_stdin(
            " app-id \n app-secret \n access-token \n refresh-token \n",
        )
        .unwrap();

        assert_eq!(inputs.app_id.as_deref(), Some("app-id"));
        assert_eq!(inputs.app_secret.as_deref(), Some("app-secret"));
        assert_eq!(inputs.access_token, "access-token");
        assert_eq!(inputs.refresh_token.as_deref(), Some("refresh-token"));
    }

    #[test]
    fn parse_full_stdin_auth_inputs_allows_missing_refresh_token() {
        let inputs =
            parse_tiktok_full_auth_inputs_from_stdin(" app-id \n app-secret \n access-token \n")
                .unwrap();

        assert_eq!(inputs.app_id.as_deref(), Some("app-id"));
        assert_eq!(inputs.app_secret.as_deref(), Some("app-secret"));
        assert_eq!(inputs.access_token, "access-token");
        assert_eq!(inputs.refresh_token, None);
    }

    #[test]
    fn resolve_auth_refresh_inputs_uses_stored_credentials() {
        let store = FakeSecretStore::default();
        store_auth_bundle(
            &store,
            &AuthBundle {
                tiktok: Some(TikTokAuthBundle {
                    app_id: Some("stored-app-id".to_string()),
                    app_secret: Some("stored-app-secret".to_string()),
                    refresh_token: Some("stored-refresh-token".to_string()),
                    ..TikTokAuthBundle::default()
                }),
                ..AuthBundle::default()
            },
        )
        .unwrap();

        let inputs = resolve_auth_refresh_inputs(
            &AuthRefreshArgs {
                app_id: None,
                app_secret: None,
            },
            &store,
        )
        .unwrap();

        assert_eq!(inputs.app_id, "stored-app-id");
        assert_eq!(inputs.app_secret, "stored-app-secret");
        assert_eq!(inputs.refresh_token, "stored-refresh-token");
    }

    #[test]
    fn tiktok_auth_status_payload_includes_credentials() {
        let payload = tiktok_auth_status_payload(TikTokAuthSnapshot {
            app_id: agent_ads_core::TikTokSecretStatus {
                present: true,
                source: agent_ads_core::TikTokSecretSource::Keychain,
                keychain_present: true,
            },
            app_secret: agent_ads_core::TikTokSecretStatus {
                present: true,
                source: agent_ads_core::TikTokSecretSource::ShellEnv,
                keychain_present: false,
            },
            access_token: agent_ads_core::TikTokSecretStatus {
                present: true,
                source: agent_ads_core::TikTokSecretSource::ShellEnv,
                keychain_present: true,
            },
            refresh_token: agent_ads_core::TikTokSecretStatus {
                present: false,
                source: agent_ads_core::TikTokSecretSource::Missing,
                keychain_present: false,
            },
            credential_store_available: true,
            credential_store_error: None,
        });

        assert_eq!(payload["provider"], json!("tiktok"));
        assert_eq!(
            payload["credentials"]["app_id"]["source"],
            json!("keychain")
        );
        assert_eq!(
            payload["credentials"]["refresh_token"]["present"],
            json!(false)
        );
    }

    #[tokio::test]
    async fn handle_auth_refresh_uses_snapshot_api_settings() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/open_api/v9.9/oauth2/access_token/"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0,
                "message": "OK",
                "request_id": "req-123",
                "data": {
                    "access_token": "new-access-token",
                    "refresh_token": "new-refresh-token",
                    "access_token_expire_in": 86400,
                    "refresh_token_expire_in": 31536000
                }
            })))
            .mount(&server)
            .await;

        let store = FakeSecretStore::default();
        let snapshot = test_snapshot(&server.uri(), "v9.9", 5);

        let result = handle_auth_refresh(
            "app-id",
            "app-secret",
            "stored-refresh-token",
            &store,
            &snapshot,
        )
        .await
        .unwrap();

        assert_eq!(result.exit_code, 0);
        let bundle = load_auth_bundle(&store).unwrap();
        let tiktok = bundle.tiktok.unwrap();
        assert_eq!(tiktok.access_token.as_deref(), Some("new-access-token"));
        assert_eq!(tiktok.refresh_token.as_deref(), Some("new-refresh-token"));
        assert_eq!(result.envelope.meta.api_version, "v9.9");
    }
}
