use std::path::{Path, PathBuf};
use std::time::Duration;

use agent_ads_core::client::GraphResponse;
use agent_ads_core::config::{
    inspect_access_token, AccessTokenSource, AccessTokenStatus, ConfigOverrides, ConfigSnapshot,
    ResolvedConfig,
};
use agent_ads_core::endpoints::{accounts, changes, creative, objects, reports, tracking};
use agent_ads_core::error::MetaAdsError;
use agent_ads_core::output::{OutputEnvelope, OutputMeta};
use agent_ads_core::{
    GraphClient, SecretStore, META_ACCESS_TOKEN_ACCOUNT, META_ACCESS_TOKEN_SERVICE,
};
use clap::{Args, Subcommand, ValueEnum};
use rpassword::prompt_password;
use serde_json::{json, Value};
use tokio::time::{sleep, Instant};

use crate::{command_result, read_input, resolve_fields, CommandResult, FieldInputArgs};

// ---------------------------------------------------------------------------
// Clap subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum MetaCommand {
    #[command(about = "List businesses accessible to your token")]
    Businesses {
        #[command(subcommand)]
        command: BusinessesCommand,
    },
    #[command(about = "List ad accounts under a business")]
    AdAccounts {
        #[command(subcommand)]
        command: AdAccountsCommand,
    },
    #[command(about = "List campaigns in an ad account")]
    Campaigns {
        #[command(subcommand)]
        command: ObjectListCommand,
    },
    #[command(about = "List ad sets in an ad account")]
    Adsets {
        #[command(subcommand)]
        command: ObjectListCommand,
    },
    #[command(about = "List ads in an ad account")]
    Ads {
        #[command(subcommand)]
        command: ObjectListCommand,
    },
    #[command(about = "Query performance insights (sync and async)")]
    Insights {
        #[command(subcommand)]
        command: InsightsCommand,
    },
    #[command(about = "Manage async report run lifecycle")]
    ReportRuns {
        #[command(subcommand)]
        command: ReportRunsCommand,
    },
    #[command(about = "Inspect ad creatives and previews")]
    Creatives {
        #[command(subcommand)]
        command: CreativesCommand,
    },
    #[command(about = "List account activity and change history")]
    Activities {
        #[command(subcommand)]
        command: ActivitiesCommand,
    },
    #[command(about = "List custom conversion rules")]
    CustomConversions {
        #[command(subcommand)]
        command: TrackingListCommand,
    },
    #[command(about = "List tracking pixels")]
    Pixels {
        #[command(subcommand)]
        command: TrackingListCommand,
    },
    #[command(about = "Get dataset quality metrics")]
    Datasets {
        #[command(subcommand)]
        command: DatasetsCommand,
    },
    #[command(about = "Combined pixel health diagnostics")]
    PixelHealth {
        #[command(subcommand)]
        command: PixelHealthCommand,
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
pub enum BusinessesCommand {
    #[command(
        about = "List businesses accessible to your token",
        visible_alias = "ls"
    )]
    List(BusinessListArgs),
}

#[derive(Subcommand, Debug)]
pub enum AdAccountsCommand {
    #[command(about = "List ad accounts by scope", visible_alias = "ls")]
    List(AdAccountListArgs),
}

#[derive(Subcommand, Debug)]
pub enum ObjectListCommand {
    #[command(about = "List objects in an ad account", visible_alias = "ls")]
    List(AccountListArgs),
}

#[derive(Subcommand, Debug)]
pub enum InsightsCommand {
    #[command(about = "Run a synchronous insights query")]
    Query(InsightsQueryArgs),
    #[command(about = "Query insights with optional async mode")]
    Export(InsightsExportArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReportRunsCommand {
    #[command(about = "Submit an async report run")]
    Submit(InsightsRequestArgs),
    #[command(about = "Check async report run status")]
    Status(ReportRunStatusArgs),
    #[command(about = "Fetch completed report run results")]
    Results(ReportRunResultsArgs),
    #[command(about = "Poll until a report run completes")]
    Wait(ReportRunWaitArgs),
}

#[derive(Subcommand, Debug)]
pub enum CreativesCommand {
    #[command(about = "Fetch a creative by ID", visible_alias = "cat")]
    Get(CreativeGetArgs),
    #[command(about = "Get rendered ad preview")]
    Preview(CreativePreviewArgs),
}

#[derive(Subcommand, Debug)]
pub enum ActivitiesCommand {
    #[command(
        about = "List account activity and change history",
        visible_alias = "ls"
    )]
    List(ActivitiesArgs),
}

#[derive(Subcommand, Debug)]
pub enum TrackingListCommand {
    #[command(about = "List tracking objects in an ad account", visible_alias = "ls")]
    List(AccountListArgs),
}

#[derive(Subcommand, Debug)]
pub enum DatasetsCommand {
    #[command(about = "Get dataset quality metrics", visible_alias = "cat")]
    Get(DatasetGetArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    #[command(about = "Store the Meta token in the OS credential store")]
    Set(AuthSetArgs),
    #[command(about = "Show auth source and secure storage status")]
    Status,
    #[command(about = "Delete the stored Meta token")]
    Delete,
}

#[derive(Subcommand, Debug)]
pub enum PixelHealthCommand {
    #[command(about = "Get combined pixel health diagnostics", visible_alias = "cat")]
    Get(PixelHealthArgs),
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
pub struct PaginationArgs {
    #[arg(long = "page-size", alias = "limit", help = "Items per API request")]
    pub page_size: Option<u32>,
    #[arg(
        long = "cursor",
        alias = "after",
        help = "Resume from a pagination cursor"
    )]
    pub cursor: Option<String>,
    #[arg(long, help = "Auto-paginate through all results")]
    pub all: bool,
    #[arg(long = "max-items", help = "Stop after collecting N total items")]
    pub max_items: Option<usize>,
}

#[derive(Args, Debug, Clone, Default)]
#[group(id = "meta_selector", multiple = false)]
pub struct SelectorArgs {
    #[arg(long, help = "Ad account ID (e.g. act_1234567890)")]
    pub account: Option<String>,
    #[arg(
        long = "object",
        alias = "object-id",
        help = "Arbitrary Graph API object ID"
    )]
    pub object: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct TimeInputArgs {
    #[arg(long, requires = "until", conflicts_with_all = ["date_preset", "time_range_file"], help = "Start date (YYYY-MM-DD)")]
    pub since: Option<String>,
    #[arg(long, requires = "since", conflicts_with_all = ["date_preset", "time_range_file"], help = "End date (YYYY-MM-DD)")]
    pub until: Option<String>,
    #[arg(long, conflicts_with_all = ["since", "until", "time_range_file"], help = "Named date preset (e.g. last_7d, last_30d)")]
    pub date_preset: Option<String>,
    #[arg(long, conflicts_with_all = ["since", "until", "date_preset"], help = "JSON file with since/until (- for stdin)")]
    pub time_range_file: Option<PathBuf>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct BusinessListArgs {
    #[command(flatten)]
    pub pagination: PaginationArgs,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ScopeArg {
    Accessible,
    Owned,
    PendingClient,
}

impl From<ScopeArg> for accounts::AdAccountScope {
    fn from(value: ScopeArg) -> Self {
        match value {
            ScopeArg::Accessible => Self::Accessible,
            ScopeArg::Owned => Self::Owned,
            ScopeArg::PendingClient => Self::PendingClient,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct AdAccountListArgs {
    #[arg(long)]
    pub business_id: Option<String>,
    #[arg(long, value_enum, default_value_t = ScopeArg::Accessible)]
    pub scope: ScopeArg,
    #[command(flatten)]
    pub pagination: PaginationArgs,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AccountListArgs {
    #[arg(long)]
    pub account: Option<String>,
    #[command(flatten)]
    pub pagination: PaginationArgs,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct InsightsRequestArgs {
    #[command(flatten)]
    pub selector: SelectorArgs,
    #[arg(long, help = "Aggregation level: account, campaign, adset, ad")]
    pub level: Option<String>,
    #[arg(long, help = "Time bucketing: 1 (daily), 7, 14, monthly, all_days")]
    pub time_increment: Option<String>,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
    #[command(flatten)]
    pub time_input: TimeInputArgs,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Dimension breakdowns (e.g. age,gender,country)"
    )]
    pub breakdowns: Vec<String>,
    #[arg(
        long = "action-breakdowns",
        value_delimiter = ',',
        help = "Action breakdowns (requires actions in --fields)"
    )]
    pub action_breakdowns: Vec<String>,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Sort order (e.g. spend_descending)"
    )]
    pub sort: Vec<String>,
    #[arg(
        long = "filter",
        alias = "filtering",
        help = "Inline filter JSON (repeatable)"
    )]
    pub filters: Vec<String>,
    #[arg(long, help = "JSON file with filter array (- for stdin)")]
    pub filter_file: Option<PathBuf>,
    #[arg(
        long = "attribution-windows",
        alias = "action-attribution-windows",
        value_delimiter = ',',
        help = "Attribution windows (e.g. 1d_click,7d_click,1d_view)"
    )]
    pub attribution_windows: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub struct InsightsQueryArgs {
    #[command(flatten)]
    pub request: InsightsRequestArgs,
    #[command(flatten)]
    pub pagination: PaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct InsightsExportArgs {
    #[command(flatten)]
    pub request: InsightsRequestArgs,
    #[command(flatten)]
    pub pagination: PaginationArgs,
    #[arg(long = "async", help = "Use async report run instead of inline query")]
    pub async_mode: bool,
    #[arg(
        long,
        requires = "async_mode",
        help = "Poll until complete, then return results"
    )]
    pub wait: bool,
    #[arg(long, default_value_t = 5, help = "Seconds between status polls")]
    pub poll_interval_seconds: u64,
    #[arg(
        long,
        default_value_t = 3600,
        help = "Max seconds to wait before timeout"
    )]
    pub wait_timeout_seconds: u64,
}

#[derive(Args, Debug, Clone)]
pub struct ReportRunStatusArgs {
    #[arg(long = "id", alias = "report-run-id")]
    pub id: String,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ReportRunResultsArgs {
    #[arg(long = "id", alias = "report-run-id")]
    pub id: String,
    #[command(flatten)]
    pub pagination: PaginationArgs,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ReportRunWaitArgs {
    #[arg(long = "id", alias = "report-run-id")]
    pub id: String,
    #[arg(long, default_value_t = 5)]
    pub poll_interval_seconds: u64,
    #[arg(long, default_value_t = 3600)]
    pub wait_timeout_seconds: u64,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CreativeGetArgs {
    #[arg(long = "id", alias = "creative-id")]
    pub id: String,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
#[command(group(clap::ArgGroup::new("preview_target").required(true).multiple(false).args(["creative", "ad"])))]
pub struct CreativePreviewArgs {
    #[arg(long = "creative", alias = "creative-id")]
    pub creative: Option<String>,
    #[arg(long = "ad", alias = "ad-id")]
    pub ad: Option<String>,
    #[arg(long)]
    pub ad_format: Option<String>,
    #[arg(long)]
    pub render_type: Option<String>,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ActivitiesArgs {
    #[arg(long)]
    pub account: Option<String>,
    #[command(flatten)]
    pub time_input: TimeInputArgs,
    #[arg(long)]
    pub category: Option<String>,
    #[arg(long)]
    pub data_source: Option<String>,
    #[arg(long)]
    pub oid: Option<String>,
    #[arg(long)]
    pub business_id: Option<String>,
    #[arg(long)]
    pub add_children: bool,
    #[command(flatten)]
    pub pagination: PaginationArgs,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct DatasetGetArgs {
    #[arg(long = "id", alias = "dataset-id")]
    pub id: String,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct PixelHealthArgs {
    #[arg(long = "pixel", alias = "pixel-id")]
    pub pixel: String,
    #[arg(long)]
    pub aggregation: Option<String>,
    #[arg(long)]
    pub event: Option<String>,
    #[arg(long)]
    pub event_source: Option<String>,
    #[arg(long)]
    pub start_time: Option<String>,
    #[arg(long)]
    pub end_time: Option<String>,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    #[arg(long, help = "Also ping the Meta API to verify the token")]
    pub api: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AuthSetArgs {
    #[arg(long, help = "Read the token from stdin instead of prompting")]
    pub stdin: bool,
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn handle_auth(
    command: AuthCommand,
    secret_store: &dyn SecretStore,
) -> Result<CommandResult, MetaAdsError> {
    match command {
        AuthCommand::Set(args) => {
            let token = resolve_auth_token_input(&args)?;
            secret_store
                .set_secret(META_ACCESS_TOKEN_SERVICE, META_ACCESS_TOKEN_ACCOUNT, &token)
                .map_err(|error| auth_storage_error("store", &error))?;

            Ok(meta_command_result(
                json!({
                    "provider": "meta",
                    "stored": true,
                    "credential_store_service": META_ACCESS_TOKEN_SERVICE,
                    "credential_store_account": META_ACCESS_TOKEN_ACCOUNT,
                }),
                "/meta/auth/set",
                0,
            ))
        }
        AuthCommand::Status => Ok(meta_command_result(
            auth_status_payload(inspect_access_token(secret_store)),
            "/meta/auth/status",
            0,
        )),
        AuthCommand::Delete => {
            let deleted = secret_store
                .delete_secret(META_ACCESS_TOKEN_SERVICE, META_ACCESS_TOKEN_ACCOUNT)
                .map_err(|error| auth_storage_error("delete", &error))?;

            Ok(meta_command_result(
                json!({
                    "provider": "meta",
                    "deleted": deleted,
                    "credential_store_service": META_ACCESS_TOKEN_SERVICE,
                    "credential_store_account": META_ACCESS_TOKEN_ACCOUNT,
                }),
                "/meta/auth/delete",
                0,
            ))
        }
    }
}

pub fn handle_config(
    command: ConfigCommand,
    snapshot: ConfigSnapshot,
) -> Result<CommandResult, MetaAdsError> {
    match command {
        ConfigCommand::Path => Ok(meta_command_result(
            json!({
                "path": snapshot.config_path,
                "exists": snapshot.config_file_exists,
            }),
            "/meta/config/path",
            0,
        )),
        ConfigCommand::Show => Ok(meta_command_result(json!(snapshot), "/meta/config/show", 0)),
        ConfigCommand::Validate => Ok(meta_command_result(
            json!({
                "valid": true,
                "config": snapshot,
            }),
            "/meta/config/validate",
            0,
        )),
    }
}

pub async fn handle_doctor(
    args: DoctorArgs,
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &ConfigOverrides,
    snapshot: ConfigSnapshot,
) -> Result<CommandResult, MetaAdsError> {
    let mut checks = vec![
        json!({
            "name": "credential_store",
            "ok": credential_store_check_ok(&snapshot),
            "detail": credential_store_detail(&snapshot),
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
            "detail": access_token_detail(&snapshot),
        }),
    ];

    let mut ok = snapshot.access_token_present;
    if args.api {
        if snapshot.access_token_present {
            match ResolvedConfig::load(config_path, secret_store, overrides)
                .and_then(|config| GraphClient::from_config(&config).map(|client| (config, client)))
            {
                Ok((_, client)) => {
                    match accounts::list_businesses(&client, &[], Some(1), None, false, Some(1))
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
                                "detail": format!("token accepted by Meta API; sampled {} business record(s)", count)
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
                "detail": "skipped because META_ADS_ACCESS_TOKEN is missing"
            }));
        }
    }

    Ok(meta_command_result(
        json!({
            "ok": ok,
            "checks": checks,
            "config": snapshot,
        }),
        "/meta/doctor",
        if ok { 0 } else { 1 },
    ))
}

pub async fn dispatch_meta_with_client(
    client: &GraphClient,
    config: &ResolvedConfig,
    command: MetaCommand,
) -> Result<CommandResult, MetaAdsError> {
    match command {
        MetaCommand::Businesses { command } => match command {
            BusinessesCommand::List(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let response = accounts::list_businesses(
                    client,
                    &fields,
                    args.pagination.page_size,
                    args.pagination.cursor.as_deref(),
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(graph_result(
                    client,
                    response,
                    "/me/businesses",
                    Some("me".to_string()),
                    None,
                    Vec::new(),
                ))
            }
        },
        MetaCommand::AdAccounts { command } => match command {
            AdAccountsCommand::List(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let business_id = resolve_business_id(config, args.business_id.as_deref())?;
                let edge = match args.scope {
                    ScopeArg::Accessible => "ad_accounts",
                    ScopeArg::Owned => "owned_ad_accounts",
                    ScopeArg::PendingClient => "pending_client_ad_accounts",
                };
                let response = accounts::list_ad_accounts(
                    client,
                    &business_id,
                    args.scope.into(),
                    &fields,
                    args.pagination.page_size,
                    args.pagination.cursor.as_deref(),
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{business_id}/{edge}"),
                    Some(business_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        MetaCommand::Campaigns { command } => match command {
            ObjectListCommand::List(args) => {
                list_object_edge(client, config, args, ObjectEdge::Campaigns).await
            }
        },
        MetaCommand::Adsets { command } => match command {
            ObjectListCommand::List(args) => {
                list_object_edge(client, config, args, ObjectEdge::Adsets).await
            }
        },
        MetaCommand::Ads { command } => match command {
            ObjectListCommand::List(args) => {
                list_object_edge(client, config, args, ObjectEdge::Ads).await
            }
        },
        MetaCommand::Insights { command } => match command {
            InsightsCommand::Query(args) => run_insights_query(client, config, args).await,
            InsightsCommand::Export(args) => run_insights_export(client, config, args).await,
        },
        MetaCommand::ReportRuns { command } => match command {
            ReportRunsCommand::Submit(args) => submit_report_run(client, config, args).await,
            ReportRunsCommand::Status(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let response = reports::get_report_run(client, &args.id, &fields).await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{}", args.id),
                    Some(args.id.clone()),
                    Some(args.id),
                    Vec::new(),
                ))
            }
            ReportRunsCommand::Results(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let response = reports::get_report_run_results(
                    client,
                    &args.id,
                    &fields,
                    args.pagination.page_size,
                    args.pagination.cursor.as_deref(),
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{}/insights", args.id),
                    Some(args.id.clone()),
                    Some(args.id),
                    Vec::new(),
                ))
            }
            ReportRunsCommand::Wait(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let response = wait_for_report_run(
                    client,
                    &args.id,
                    Duration::from_secs(args.poll_interval_seconds),
                    Duration::from_secs(args.wait_timeout_seconds),
                    &fields,
                )
                .await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{}", args.id),
                    Some(args.id.clone()),
                    Some(args.id),
                    Vec::new(),
                ))
            }
        },
        MetaCommand::Creatives { command } => match command {
            CreativesCommand::Get(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let response = creative::get_creative(client, &args.id, &fields).await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{}", args.id),
                    Some(args.id),
                    None,
                    Vec::new(),
                ))
            }
            CreativesCommand::Preview(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let creative_id = resolve_preview_creative_id(client, &args).await?;
                let response = creative::get_creative_preview(
                    client,
                    &creative_id,
                    args.ad_format.as_deref(),
                    args.render_type.as_deref(),
                    &fields,
                )
                .await?;
                let warnings = if args.ad.is_some() {
                    vec![
                        "preview resolved the ad to its creative before calling the preview edge"
                            .to_string(),
                    ]
                } else {
                    Vec::new()
                };
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{creative_id}/previews"),
                    Some(creative_id),
                    None,
                    warnings,
                ))
            }
        },
        MetaCommand::Activities { command } => match command {
            ActivitiesCommand::List(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let account_id = resolve_account_id(config, args.account.as_deref())?;
                let (date_preset, since, until) = resolve_time_input(&args.time_input)?;
                if date_preset.is_some() {
                    return Err(MetaAdsError::InvalidArgument(
                        "`activities list` does not support --date-preset or --time-range-file without since/until"
                            .to_string(),
                    ));
                }
                let response = changes::list_activities(
                    client,
                    &account_id,
                    &fields,
                    args.pagination.page_size,
                    args.pagination.cursor.as_deref(),
                    since.as_deref(),
                    until.as_deref(),
                    args.category.as_deref(),
                    args.data_source.as_deref(),
                    args.oid.as_deref(),
                    args.business_id.as_deref(),
                    args.add_children,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{account_id}/activities"),
                    Some(account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        MetaCommand::CustomConversions { command } => match command {
            TrackingListCommand::List(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let account_id = resolve_account_id(config, args.account.as_deref())?;
                let response = tracking::list_custom_conversions(
                    client,
                    &account_id,
                    &fields,
                    args.pagination.page_size,
                    args.pagination.cursor.as_deref(),
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{account_id}/customconversions"),
                    Some(account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        MetaCommand::Pixels { command } => match command {
            TrackingListCommand::List(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let account_id = resolve_account_id(config, args.account.as_deref())?;
                let response = tracking::list_pixels(
                    client,
                    &account_id,
                    &fields,
                    args.pagination.page_size,
                    args.pagination.cursor.as_deref(),
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{account_id}/adspixels"),
                    Some(account_id),
                    None,
                    Vec::new(),
                ))
            }
        },
        MetaCommand::Datasets { command } => match command {
            DatasetsCommand::Get(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let response = tracking::get_dataset_quality(client, &args.id, &fields).await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{}", args.id),
                    Some(args.id),
                    None,
                    Vec::new(),
                ))
            }
        },
        MetaCommand::PixelHealth { command } => match command {
            PixelHealthCommand::Get(args) => {
                let fields = resolve_fields(&args.field_input)?;
                let response = tracking::get_emq_diagnostics(
                    client,
                    &args.pixel,
                    &fields,
                    args.aggregation.as_deref(),
                    args.event.as_deref(),
                    args.event_source.as_deref(),
                    args.start_time.as_deref(),
                    args.end_time.as_deref(),
                )
                .await?;
                Ok(graph_result(
                    client,
                    response,
                    &format!("/{}/stats", args.pixel),
                    Some(args.pixel),
                    None,
                    vec![
                        "pixel-health is a practical diagnostics view built from pixel metadata and the documented /stats edge."
                            .to_string(),
                    ],
                ))
            }
        },
        MetaCommand::Doctor(_) | MetaCommand::Config { .. } | MetaCommand::Auth { .. } => {
            unreachable!("handled before auth setup")
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum ObjectEdge {
    Campaigns,
    Adsets,
    Ads,
}

impl ObjectEdge {
    fn as_str(self) -> &'static str {
        match self {
            Self::Campaigns => "campaigns",
            Self::Adsets => "adsets",
            Self::Ads => "ads",
        }
    }
}

async fn list_object_edge(
    client: &GraphClient,
    config: &ResolvedConfig,
    args: AccountListArgs,
    edge: ObjectEdge,
) -> Result<CommandResult, MetaAdsError> {
    let fields = resolve_fields(&args.field_input)?;
    let account_id = resolve_account_id(config, args.account.as_deref())?;
    let response = match edge {
        ObjectEdge::Campaigns => {
            objects::list_campaigns(
                client,
                &account_id,
                &fields,
                args.pagination.page_size,
                args.pagination.cursor.as_deref(),
                args.pagination.all,
                args.pagination.max_items,
            )
            .await?
        }
        ObjectEdge::Adsets => {
            objects::list_adsets(
                client,
                &account_id,
                &fields,
                args.pagination.page_size,
                args.pagination.cursor.as_deref(),
                args.pagination.all,
                args.pagination.max_items,
            )
            .await?
        }
        ObjectEdge::Ads => {
            objects::list_ads(
                client,
                &account_id,
                &fields,
                args.pagination.page_size,
                args.pagination.cursor.as_deref(),
                args.pagination.all,
                args.pagination.max_items,
            )
            .await?
        }
    };
    Ok(graph_result(
        client,
        response,
        &format!("/{account_id}/{}", edge.as_str()),
        Some(account_id),
        None,
        Vec::new(),
    ))
}

async fn run_insights_query(
    client: &GraphClient,
    config: &ResolvedConfig,
    args: InsightsQueryArgs,
) -> Result<CommandResult, MetaAdsError> {
    let resolved = resolve_insights_request(config, &args.request, &args.pagination)?;
    let response = reports::query_insights(client, to_query(&resolved)).await?;
    Ok(graph_result(
        client,
        response,
        &format!("/{}/insights", resolved.object_id),
        Some(resolved.object_id.clone()),
        None,
        Vec::new(),
    ))
}

async fn run_insights_export(
    client: &GraphClient,
    config: &ResolvedConfig,
    args: InsightsExportArgs,
) -> Result<CommandResult, MetaAdsError> {
    let resolved = resolve_insights_request(config, &args.request, &args.pagination)?;
    if !args.async_mode {
        let response = reports::query_insights(client, to_query(&resolved)).await?;
        return Ok(graph_result(
            client,
            response,
            &format!("/{}/insights", resolved.object_id),
            Some(resolved.object_id),
            None,
            Vec::new(),
        ));
    }

    let submit = reports::submit_report_run(client, to_query(&resolved)).await?;
    let report_run_id = extract_report_run_id(&submit.data).ok_or_else(|| {
        MetaAdsError::Config("Meta did not return a report_run_id for the async export".to_string())
    })?;

    if !args.wait {
        return Ok(graph_result(
            client,
            submit,
            &format!("/{}/insights", resolved.object_id),
            Some(resolved.object_id),
            Some(report_run_id),
            Vec::new(),
        ));
    }

    wait_for_report_run(
        client,
        &report_run_id,
        Duration::from_secs(args.poll_interval_seconds),
        Duration::from_secs(args.wait_timeout_seconds),
        &[],
    )
    .await?;

    let results = reports::get_report_run_results(
        client,
        &report_run_id,
        &resolved.fields,
        resolved.page_size,
        resolved.cursor.as_deref(),
        true,
        resolved.max_items,
    )
    .await?;

    Ok(graph_result(
        client,
        results,
        &format!("/{report_run_id}/insights"),
        Some(report_run_id.clone()),
        Some(report_run_id),
        vec![
            "insights export waited for the async report run and returned the final result set."
                .to_string(),
        ],
    ))
}

async fn submit_report_run(
    client: &GraphClient,
    config: &ResolvedConfig,
    args: InsightsRequestArgs,
) -> Result<CommandResult, MetaAdsError> {
    let resolved = resolve_insights_request(config, &args, &PaginationArgs::default())?;
    let response = reports::submit_report_run(client, to_query(&resolved)).await?;
    let report_run_id = extract_report_run_id(&response.data);
    Ok(graph_result(
        client,
        response,
        &format!("/{}/insights", resolved.object_id),
        Some(resolved.object_id),
        report_run_id,
        Vec::new(),
    ))
}

async fn wait_for_report_run(
    client: &GraphClient,
    report_run_id: &str,
    poll_interval: Duration,
    timeout: Duration,
    fields: &[String],
) -> Result<GraphResponse, MetaAdsError> {
    let started = Instant::now();
    loop {
        let response = reports::get_report_run(client, report_run_id, fields).await?;
        let status = response
            .data
            .get("async_status")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();
        let percent = response
            .data
            .get("async_percent_completion")
            .and_then(Value::as_i64)
            .unwrap_or_default();

        if status.contains("complete") || percent >= 100 {
            return Ok(response);
        }

        if status.contains("fail") || status.contains("error") || status.contains("skip") {
            return Err(MetaAdsError::Config(format!(
                "report run {report_run_id} ended with async_status `{status}`"
            )));
        }

        if started.elapsed() >= timeout {
            return Err(MetaAdsError::Config(format!(
                "timed out waiting for report run {report_run_id}"
            )));
        }

        sleep(poll_interval).await;
    }
}

fn graph_result(
    client: &GraphClient,
    response: GraphResponse,
    endpoint: &str,
    object_id: Option<String>,
    report_run_id: Option<String>,
    warnings: Vec<String>,
) -> CommandResult {
    let mut envelope = OutputEnvelope::new(
        response.data,
        OutputMeta {
            api_version: client.api_version().to_string(),
            endpoint: endpoint.to_string(),
            object_id,
            request_id: response.request_id,
            report_run_id,
        },
    );
    envelope.paging = response.paging.map(|paging| json!(paging));
    if !warnings.is_empty() {
        envelope.warnings = Some(warnings);
    }
    CommandResult {
        envelope,
        exit_code: 0,
    }
}

fn meta_command_result(data: Value, endpoint: &str, exit_code: u8) -> CommandResult {
    command_result(
        data,
        endpoint,
        exit_code,
        Some(agent_ads_core::DEFAULT_API_VERSION),
    )
}

fn resolve_business_id(
    config: &ResolvedConfig,
    value: Option<&str>,
) -> Result<String, MetaAdsError> {
    value
        .map(str::to_string)
        .or_else(|| config.default_business_id.clone())
        .ok_or_else(|| {
            MetaAdsError::InvalidArgument(
                "business id is required; pass --business-id or set default_business_id"
                    .to_string(),
            )
        })
}

fn resolve_account_id(
    config: &ResolvedConfig,
    value: Option<&str>,
) -> Result<String, MetaAdsError> {
    value
        .map(str::to_string)
        .or_else(|| config.default_account_id.clone())
        .ok_or_else(|| {
            MetaAdsError::InvalidArgument(
                "account id is required; pass --account or set default_account_id".to_string(),
            )
        })
}

fn resolve_object_id(
    config: &ResolvedConfig,
    selector: &SelectorArgs,
) -> Result<String, MetaAdsError> {
    match (&selector.account, &selector.object) {
        (Some(account), None) => resolve_account_id(config, Some(account)),
        (None, Some(object)) => Ok(object.clone()),
        (None, None) => resolve_account_id(config, None),
        (Some(_), Some(_)) => Err(MetaAdsError::InvalidArgument(
            "use either --account or --object, not both".to_string(),
        )),
    }
}

async fn resolve_preview_creative_id(
    client: &GraphClient,
    args: &CreativePreviewArgs,
) -> Result<String, MetaAdsError> {
    match (&args.creative, &args.ad) {
        (Some(creative_id), None) => Ok(creative_id.clone()),
        (None, Some(ad_id)) => creative::resolve_creative_id_from_ad(client, ad_id).await,
        _ => Err(MetaAdsError::InvalidArgument(
            "preview requires exactly one of --creative or --ad".to_string(),
        )),
    }
}

#[derive(Debug, Clone)]
struct ResolvedInsightsRequest {
    object_id: String,
    level: Option<String>,
    time_increment: Option<String>,
    fields: Vec<String>,
    date_preset: Option<String>,
    since: Option<String>,
    until: Option<String>,
    breakdowns: Vec<String>,
    action_breakdowns: Vec<String>,
    sort: Vec<String>,
    filters: Vec<String>,
    attribution_windows: Vec<String>,
    page_size: Option<u32>,
    cursor: Option<String>,
    fetch_all: bool,
    max_items: Option<usize>,
}

fn resolve_insights_request(
    config: &ResolvedConfig,
    request: &InsightsRequestArgs,
    pagination: &PaginationArgs,
) -> Result<ResolvedInsightsRequest, MetaAdsError> {
    let fields = resolve_fields(&request.field_input)?;
    let (date_preset, since, until) = resolve_time_input(&request.time_input)?;
    let filters = resolve_filters(&request.filters, request.filter_file.as_deref())?;
    Ok(ResolvedInsightsRequest {
        object_id: resolve_object_id(config, &request.selector)?,
        level: request.level.clone(),
        time_increment: request.time_increment.clone(),
        fields,
        date_preset,
        since,
        until,
        breakdowns: request.breakdowns.clone(),
        action_breakdowns: request.action_breakdowns.clone(),
        sort: request.sort.clone(),
        filters,
        attribution_windows: request.attribution_windows.clone(),
        page_size: pagination.page_size,
        cursor: pagination.cursor.clone(),
        fetch_all: pagination.all,
        max_items: pagination.max_items,
    })
}

fn to_query(resolved: &ResolvedInsightsRequest) -> reports::InsightsQuery<'_> {
    reports::InsightsQuery {
        object_id: &resolved.object_id,
        level: resolved.level.as_deref(),
        fields: &resolved.fields,
        date_preset: resolved.date_preset.as_deref(),
        since: resolved.since.as_deref(),
        until: resolved.until.as_deref(),
        time_increment: resolved.time_increment.as_deref(),
        breakdowns: &resolved.breakdowns,
        action_breakdowns: &resolved.action_breakdowns,
        sort: &resolved.sort,
        filtering: &resolved.filters,
        action_attribution_windows: &resolved.attribution_windows,
        limit: resolved.page_size,
        after: resolved.cursor.as_deref(),
        fetch_all: resolved.fetch_all,
        max_items: resolved.max_items,
    }
}

fn resolve_filters(
    filters: &[String],
    filter_file: Option<&Path>,
) -> Result<Vec<String>, MetaAdsError> {
    let mut resolved = filters.to_vec();
    if let Some(path) = filter_file {
        let content = read_input(path)?;
        let value: Value = serde_json::from_str(&content)?;
        match value {
            Value::Array(items) => {
                resolved.extend(items.into_iter().map(|item| item.to_string()));
            }
            other => resolved.push(other.to_string()),
        }
    }
    Ok(resolved)
}

fn resolve_time_input(
    input: &TimeInputArgs,
) -> Result<(Option<String>, Option<String>, Option<String>), MetaAdsError> {
    if let Some(path) = &input.time_range_file {
        let content = read_input(path)?;
        let value: Value = serde_json::from_str(&content)?;
        let since = value
            .get("since")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                MetaAdsError::InvalidArgument(
                    "time range files must contain a `since` string".to_string(),
                )
            })?;
        let until = value
            .get("until")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                MetaAdsError::InvalidArgument(
                    "time range files must contain an `until` string".to_string(),
                )
            })?;
        return Ok((None, Some(since), Some(until)));
    }

    Ok((
        input.date_preset.clone(),
        input.since.clone(),
        input.until.clone(),
    ))
}

fn resolve_auth_token_input(args: &AuthSetArgs) -> Result<String, MetaAdsError> {
    let token = if args.stdin {
        read_input(Path::new("-"))?
    } else {
        prompt_for_auth_token()?
    };

    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(MetaAdsError::InvalidArgument(
            "token input was empty".to_string(),
        ));
    }

    Ok(token)
}

fn prompt_for_auth_token() -> Result<String, MetaAdsError> {
    prompt_password("Meta access token: ").map_err(MetaAdsError::Io)
}

fn auth_storage_error(action: &str, error: &impl std::fmt::Display) -> MetaAdsError {
    MetaAdsError::Config(format!(
        "failed to {action} the Meta token in the OS credential store: {error}{}",
        linux_secure_storage_hint()
    ))
}

fn linux_secure_storage_hint() -> &'static str {
    if cfg!(target_os = "linux") {
        " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet."
    } else {
        ""
    }
}

pub fn credential_store_check_ok(snapshot: &ConfigSnapshot) -> bool {
    snapshot.credential_store_available
        || snapshot.access_token_source == AccessTokenSource::ShellEnv
}

pub fn credential_store_detail(snapshot: &ConfigSnapshot) -> String {
    match snapshot.credential_store_error.as_deref() {
        Some(error) if snapshot.access_token_source == AccessTokenSource::ShellEnv => {
            format!("shell env override active; OS credential store unavailable: {error}")
        }
        Some(error) => format!("OS credential store unavailable: {error}"),
        None if snapshot.keychain_token_present => {
            "stored Meta token found in OS credential store".to_string()
        }
        None if snapshot.credential_store_available => {
            "OS credential store is available; no stored Meta token found".to_string()
        }
        None => "OS credential store is unavailable".to_string(),
    }
}

fn access_token_detail(snapshot: &ConfigSnapshot) -> String {
    match snapshot.access_token_source {
        AccessTokenSource::ShellEnv if snapshot.keychain_token_present => {
            "META_ADS_ACCESS_TOKEN is set in shell env and overrides the stored token".to_string()
        }
        AccessTokenSource::ShellEnv => "META_ADS_ACCESS_TOKEN is set in shell env".to_string(),
        AccessTokenSource::Keychain => {
            "using stored Meta token from the OS credential store".to_string()
        }
        AccessTokenSource::Missing => match snapshot.credential_store_error.as_deref() {
            Some(error) => format!("META_ADS_ACCESS_TOKEN is missing; {error}"),
            None => "META_ADS_ACCESS_TOKEN is missing".to_string(),
        },
    }
}

fn auth_status_payload(status: AccessTokenStatus) -> Value {
    json!({
        "provider": "meta",
        "credential_store_service": META_ACCESS_TOKEN_SERVICE,
        "credential_store_account": META_ACCESS_TOKEN_ACCOUNT,
        "access_token_present": status.access_token_present,
        "access_token_source": status.access_token_source,
        "credential_store_available": status.credential_store_available,
        "keychain_token_present": status.keychain_token_present,
        "credential_store_error": status.credential_store_error,
    })
}

fn extract_report_run_id(data: &Value) -> Option<String> {
    data.get("report_run_id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| data.get("id").and_then(Value::as_str).map(str::to_string))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use agent_ads_core::config::{AccessTokenSource, ConfigSnapshot};
    use agent_ads_core::output::OutputFormat;
    use std::path::PathBuf;

    use super::{credential_store_check_ok, credential_store_detail};

    pub fn snapshot_with_auth(
        access_token_source: AccessTokenSource,
        credential_store_available: bool,
        credential_store_error: Option<&str>,
    ) -> ConfigSnapshot {
        ConfigSnapshot {
            config_path: PathBuf::from("agent-ads.config.json"),
            config_file_exists: true,
            access_token_present: access_token_source != AccessTokenSource::Missing,
            access_token_source,
            credential_store_available,
            keychain_token_present: false,
            credential_store_error: credential_store_error.map(str::to_string),
            api_base_url: "https://graph.facebook.com".to_string(),
            api_version: "v25.0".to_string(),
            timeout_seconds: 60,
            default_business_id: None,
            default_account_id: None,
            output_format: OutputFormat::Json,
        }
    }

    #[test]
    fn doctor_treats_unavailable_credential_store_as_ok_when_shell_env_is_active() {
        let snapshot = snapshot_with_auth(
            AccessTokenSource::ShellEnv,
            false,
            Some("secure storage backend is unavailable"),
        );

        assert!(credential_store_check_ok(&snapshot));
        assert!(credential_store_detail(&snapshot).contains("shell env override active"));
    }

    #[test]
    fn doctor_fails_credential_store_check_when_store_is_unavailable_and_token_is_missing() {
        let snapshot = snapshot_with_auth(
            AccessTokenSource::Missing,
            false,
            Some("secure storage backend is unavailable"),
        );

        assert!(!credential_store_check_ok(&snapshot));
        assert!(credential_store_detail(&snapshot).contains("OS credential store unavailable"));
    }
}
