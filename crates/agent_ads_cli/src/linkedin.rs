use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_ads_core::linkedin_config::{
    linkedin_inspect_auth, LinkedInAccessTokenSource, LinkedInAccessTokenStatus,
    LinkedInAuthSnapshot, LinkedInConfigOverrides, LinkedInConfigSnapshot, LinkedInResolvedConfig,
    LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR, LINKEDIN_DEFAULT_API_VERSION,
};
use agent_ads_core::linkedin_endpoints::{
    accounts, campaign_groups, campaigns, creatives, reports as linkedin_reports,
};
use agent_ads_core::output::{OutputEnvelope, OutputMeta};
use agent_ads_core::secret_store::SecretStore;
use agent_ads_core::{
    mutate_auth_bundle, LinkedInAuthBundle, LinkedInClient, LinkedInError, LinkedInResponse,
    AUTH_BUNDLE_ACCOUNT, AUTH_BUNDLE_SERVICE,
};
use clap::{Args, Subcommand, ValueEnum};
use rpassword::prompt_password;
use serde_json::{json, Value};

use crate::{command_result, read_input, CommandResult};

const CAMPAIGN_GROUP_STATUS_VALUES: &[&str] = &[
    "ACTIVE",
    "ARCHIVED",
    "CANCELED",
    "DRAFT",
    "PAUSED",
    "PENDING_DELETION",
    "REMOVED",
];
const CAMPAIGN_STATUS_VALUES: &[&str] = &[
    "ACTIVE",
    "PAUSED",
    "ARCHIVED",
    "COMPLETED",
    "CANCELED",
    "DRAFT",
    "PENDING_DELETION",
    "REMOVED",
];

// ---------------------------------------------------------------------------
// Clap subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum LinkedInCommand {
    #[command(about = "List and inspect LinkedIn ad accounts")]
    AdAccounts {
        #[command(subcommand)]
        command: AdAccountsCommand,
    },
    #[command(about = "List campaign groups for an ad account")]
    CampaignGroups {
        #[command(subcommand)]
        command: CampaignGroupsCommand,
    },
    #[command(about = "List and inspect campaigns")]
    Campaigns {
        #[command(subcommand)]
        command: CampaignsCommand,
    },
    #[command(about = "List and inspect creatives")]
    Creatives {
        #[command(subcommand)]
        command: CreativesCommand,
    },
    #[command(about = "Query LinkedIn reporting finders")]
    Analytics {
        #[command(subcommand)]
        command: AnalyticsCommand,
    },
    #[command(about = "Manage stored LinkedIn access tokens")]
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
    #[command(
        about = "List accessible ad accounts joined with the authenticated user's role",
        visible_alias = "ls"
    )]
    List,
    #[command(about = "Get a single ad account", visible_alias = "cat")]
    Get(AdAccountGetArgs),
    #[command(about = "Search ad accounts", visible_alias = "find")]
    Search(AdAccountSearchArgs),
}

#[derive(Subcommand, Debug)]
pub enum CampaignGroupsCommand {
    #[command(about = "List campaign groups for an ad account", visible_alias = "ls")]
    List(CampaignGroupListArgs),
}

#[derive(Subcommand, Debug)]
pub enum CampaignsCommand {
    #[command(about = "List campaigns for an ad account", visible_alias = "ls")]
    List(CampaignListArgs),
    #[command(about = "Get a single campaign", visible_alias = "cat")]
    Get(CampaignGetArgs),
}

#[derive(Subcommand, Debug)]
pub enum CreativesCommand {
    #[command(about = "List creatives for an ad account", visible_alias = "ls")]
    List(CreativeListArgs),
    #[command(about = "Get a single creative", visible_alias = "cat")]
    Get(CreativeGetArgs),
}

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommand {
    #[command(about = "Run one of the LinkedIn adAnalytics finders")]
    Query(AnalyticsQueryArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    #[command(about = "Store the LinkedIn access token in the OS credential store")]
    Set(AuthSetArgs),
    #[command(about = "Show auth source and secure storage status")]
    Status,
    #[command(about = "Delete the stored LinkedIn access token")]
    Delete,
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
pub struct LinkedInPaginationArgs {
    #[arg(long = "page-token", help = "Resume from a LinkedIn nextPageToken")]
    pub page_token: Option<String>,
    #[arg(long = "page-size", help = "Items per API request")]
    pub page_size: Option<u32>,
    #[arg(long, help = "Auto-follow all available pages")]
    pub all: bool,
    #[arg(long = "max-items", help = "Stop after collecting N total items")]
    pub max_items: Option<usize>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct LinkedInReportPaginationArgs {
    #[arg(long, help = "Starting offset for the first page")]
    pub start: Option<u32>,
    #[arg(long = "page-size", alias = "count", help = "Rows per API request")]
    pub page_size: Option<u32>,
    #[arg(long, help = "Auto-follow all available pages")]
    pub all: bool,
    #[arg(long = "max-items", help = "Stop after collecting N total rows")]
    pub max_items: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct AccountSelectorArgs {
    #[arg(
        long = "account-id",
        alias = "ad-account-id",
        help = "LinkedIn ad account ID or sponsored account URN"
    )]
    pub account_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AdAccountGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SortOrderArg {
    Ascending,
    Descending,
}

#[derive(Args, Debug, Clone)]
pub struct AdAccountSearchArgs {
    #[arg(
        long = "account-id",
        alias = "ad-account-id",
        value_delimiter = ',',
        help = "Filter to one or more account IDs or URNs"
    )]
    pub account_ids: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Filter by account name")]
    pub names: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Filter by reference URN")]
    pub references: Vec<String>,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Filter by account type (BUSINESS, ENTERPRISE)"
    )]
    pub account_types: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Filter by account status")]
    pub statuses: Vec<String>,
    #[arg(long, help = "Filter test accounts (`true` or `false`)")]
    pub test: Option<bool>,
    #[arg(long = "sort-order", value_enum, help = "Sort by account ID")]
    pub sort_order: Option<SortOrderArg>,
    #[command(flatten)]
    pub pagination: LinkedInPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CampaignGroupListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "campaign-group-id",
        value_delimiter = ',',
        help = "Filter to one or more campaign group IDs or URNs"
    )]
    pub campaign_group_ids: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Filter by campaign group name")]
    pub names: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Filter by campaign group status")]
    pub statuses: Vec<String>,
    #[arg(long, help = "Filter test campaign groups (`true` or `false`)")]
    pub test: Option<bool>,
    #[arg(long = "sort-order", value_enum, help = "Sort by campaign group ID")]
    pub sort_order: Option<SortOrderArg>,
    #[command(flatten)]
    pub pagination: LinkedInPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CampaignListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter to one or more campaign IDs or URNs"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "campaign-group-id",
        value_delimiter = ',',
        help = "Filter to one or more campaign group IDs or URNs"
    )]
    pub campaign_group_ids: Vec<String>,
    #[arg(
        long = "associated-entity",
        value_delimiter = ',',
        help = "Filter by associated entity URN"
    )]
    pub associated_entities: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Filter by campaign name")]
    pub names: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Filter by campaign status")]
    pub statuses: Vec<String>,
    #[arg(long, value_delimiter = ',', help = "Filter by campaign type")]
    pub campaign_types: Vec<String>,
    #[arg(long, help = "Filter test campaigns (`true` or `false`)")]
    pub test: Option<bool>,
    #[arg(long = "sort-order", value_enum, help = "Sort by campaign ID")]
    pub sort_order: Option<SortOrderArg>,
    #[command(flatten)]
    pub pagination: LinkedInPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CampaignGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "campaign-id", help = "Campaign ID or sponsored campaign URN")]
    pub campaign_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct CreativeListArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(
        long = "creative-id",
        value_delimiter = ',',
        help = "Filter to one or more creative IDs or URNs"
    )]
    pub creative_ids: Vec<String>,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter by campaign IDs or URNs"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "content-reference",
        value_delimiter = ',',
        help = "Filter by content reference URNs"
    )]
    pub content_references: Vec<String>,
    #[arg(
        long = "intended-status",
        value_delimiter = ',',
        help = "Filter by intended creative status"
    )]
    pub intended_statuses: Vec<String>,
    #[arg(
        long = "test-account",
        help = "Filter creatives by whether the parent account is a test account"
    )]
    pub test_account: Option<bool>,
    #[arg(long = "sort-order", value_enum, help = "Sort by creative ID")]
    pub sort_order: Option<SortOrderArg>,
    #[command(flatten)]
    pub pagination: LinkedInPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CreativeGetArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long = "creative-id", help = "Creative ID or sponsored creative URN")]
    pub creative_id: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AnalyticsFinderArg {
    Analytics,
    Statistics,
    #[value(name = "attributed-revenue-metrics")]
    AttributedRevenueMetrics,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyticsQueryArgs {
    #[command(flatten)]
    pub selector: AccountSelectorArgs,
    #[arg(long, value_enum, help = "LinkedIn adAnalytics finder")]
    pub finder: AnalyticsFinderArg,
    #[arg(
        long = "pivot",
        value_delimiter = ',',
        help = "Pivot value(s) for the selected finder"
    )]
    pub pivots: Vec<String>,
    #[arg(
        long = "time-granularity",
        help = "Time granularity (required for analytics/statistics)"
    )]
    pub time_granularity: Option<String>,
    #[arg(long = "since", help = "Start date (YYYY-MM-DD)")]
    pub since: String,
    #[arg(long = "until", help = "End date (YYYY-MM-DD)")]
    pub until: Option<String>,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Comma-separated metrics/fields projection"
    )]
    pub fields: Vec<String>,
    #[arg(
        long = "campaign-id",
        value_delimiter = ',',
        help = "Filter by campaign IDs or URNs"
    )]
    pub campaign_ids: Vec<String>,
    #[arg(
        long = "campaign-group-id",
        value_delimiter = ',',
        help = "Filter by campaign group IDs or URNs"
    )]
    pub campaign_group_ids: Vec<String>,
    #[arg(
        long = "creative-id",
        value_delimiter = ',',
        help = "Filter by creative IDs or URNs"
    )]
    pub creative_ids: Vec<String>,
    #[command(flatten)]
    pub pagination: LinkedInReportPaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    #[arg(long, help = "Also make a lightweight LinkedIn Marketing API request")]
    pub api: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct AuthSetArgs {
    #[arg(
        long,
        help = "Read the LinkedIn access token from stdin instead of prompting",
        conflicts_with = "access_token"
    )]
    pub stdin: bool,
    #[arg(long = "access-token", help = "LinkedIn access token")]
    pub access_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Dispatch: auth, config, doctor
// ---------------------------------------------------------------------------

pub fn handle_auth(
    command: AuthCommand,
    secret_store: &dyn SecretStore,
) -> Result<CommandResult, LinkedInError> {
    match command {
        AuthCommand::Set(args) => {
            let access_token = resolve_auth_token_input(&args)?;
            let outcome = mutate_auth_bundle(secret_store, move |bundle| {
                bundle.linkedin = Some(LinkedInAuthBundle {
                    access_token: Some(access_token),
                });
            })
            .map_err(|error| auth_storage_error("store LinkedIn credentials", &error))?;

            Ok(linkedin_command_result(
                json!({
                    "provider": "linkedin",
                    "stored": true,
                    "recovered_invalid_bundle": outcome.recovered_invalid_bundle,
                    "credentials_stored": ["access_token"],
                }),
                "/linkedin/auth/set",
                0,
            ))
        }
        AuthCommand::Status => Ok(linkedin_command_result(
            linkedin_auth_status_payload(linkedin_inspect_auth(secret_store)),
            "/linkedin/auth/status",
            0,
        )),
        AuthCommand::Delete => {
            let mut deleted = false;
            let outcome = mutate_auth_bundle(secret_store, |bundle| {
                deleted = bundle
                    .linkedin
                    .take()
                    .and_then(|linkedin| linkedin.access_token)
                    .is_some();
            })
            .map_err(|error| auth_storage_error("delete LinkedIn credentials", &error))?;

            Ok(linkedin_command_result(
                json!({
                    "provider": "linkedin",
                    "access_token_deleted": deleted,
                    "recovered_invalid_bundle": outcome.recovered_invalid_bundle,
                }),
                "/linkedin/auth/delete",
                0,
            ))
        }
    }
}

pub fn handle_config(
    command: ConfigCommand,
    snapshot: LinkedInConfigSnapshot,
) -> Result<CommandResult, LinkedInError> {
    match command {
        ConfigCommand::Path => Ok(linkedin_command_result(
            json!({
                "path": snapshot.config_path,
                "exists": snapshot.config_file_exists,
            }),
            "/linkedin/config/path",
            0,
        )),
        ConfigCommand::Show => Ok(linkedin_command_result(
            json!(snapshot),
            "/linkedin/config/show",
            0,
        )),
        ConfigCommand::Validate => Ok(linkedin_command_result(
            json!({
                "valid": true,
                "config": snapshot,
            }),
            "/linkedin/config/validate",
            0,
        )),
    }
}

pub async fn handle_doctor(
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &LinkedInConfigOverrides,
    args: DoctorArgs,
    snapshot: LinkedInConfigSnapshot,
) -> Result<CommandResult, LinkedInError> {
    let mut ok = snapshot.auth.access_token.present;
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
            "ok": snapshot.auth.access_token.present,
            "detail": access_token_detail(LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR, &snapshot.auth.access_token),
        }),
    ];

    if args.api {
        match LinkedInResolvedConfig::load(config_path, secret_store, overrides) {
            Ok(config) => match LinkedInClient::from_config(&config) {
                Ok(client) => match accounts::list_accessible_account_users(&client).await {
                    Ok(response) => {
                        let count = response
                            .data
                            .get("elements")
                            .and_then(Value::as_array)
                            .map(|items| items.len())
                            .unwrap_or(0);
                        checks.push(json!({
                            "name": "api_ping",
                            "ok": true,
                            "detail": format!("LinkedIn Marketing API accepted the token; sampled {} account-access record(s)", count),
                        }));
                    }
                    Err(error) => {
                        ok = false;
                        checks.push(json!({
                            "name": "api_ping",
                            "ok": false,
                            "detail": error.to_string(),
                        }));
                    }
                },
                Err(error) => {
                    ok = false;
                    checks.push(json!({
                        "name": "api_ping",
                        "ok": false,
                        "detail": error.to_string(),
                    }));
                }
            },
            Err(error) => {
                ok = false;
                checks.push(json!({
                    "name": "api_ping",
                    "ok": false,
                    "detail": error.to_string(),
                }));
            }
        }
    }

    Ok(linkedin_command_result(
        json!({
            "ok": ok,
            "checks": checks,
            "config": snapshot,
        }),
        "/linkedin/doctor",
        if ok { 0 } else { 1 },
    ))
}

// ---------------------------------------------------------------------------
// Dispatch: authenticated commands
// ---------------------------------------------------------------------------

pub async fn dispatch_linkedin_with_client(
    client: &LinkedInClient,
    config: &LinkedInResolvedConfig,
    command: LinkedInCommand,
) -> Result<CommandResult, LinkedInError> {
    match command {
        LinkedInCommand::AdAccounts { command } => match command {
            AdAccountsCommand::List => list_accessible_accounts_with_roles(client).await,
            AdAccountsCommand::Get(args) => {
                let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
                let response = accounts::get_account(client, &account_id).await?;
                Ok(linkedin_result(
                    client,
                    response,
                    &format!("/adAccounts/{account_id}"),
                    Some(account_id),
                    vec![],
                ))
            }
            AdAccountsCommand::Search(args) => {
                let search = build_account_search_expression(&args)?;
                let sort = args.sort_order.map(account_sort_clause);
                let response = accounts::search_accounts(
                    client,
                    search.as_deref(),
                    sort.as_deref(),
                    args.pagination.page_token.as_deref(),
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(linkedin_result(
                    client,
                    response,
                    "/adAccounts",
                    None,
                    vec![],
                ))
            }
        },
        LinkedInCommand::CampaignGroups { command } => match command {
            CampaignGroupsCommand::List(args) => {
                let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
                let search = build_campaign_group_search_expression(&args)?;
                let response = campaign_groups::list_campaign_groups(
                    client,
                    &account_id,
                    &search,
                    args.sort_order.map(sort_order_value),
                    args.pagination.page_token.as_deref(),
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(linkedin_result(
                    client,
                    response,
                    &format!("/adAccounts/{account_id}/adCampaignGroups"),
                    Some(account_id),
                    vec![],
                ))
            }
        },
        LinkedInCommand::Campaigns { command } => match command {
            CampaignsCommand::List(args) => {
                let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
                let search = build_campaign_search_expression(&args)?;
                let response = campaigns::list_campaigns(
                    client,
                    &account_id,
                    &search,
                    args.sort_order.map(sort_order_value),
                    args.pagination.page_token.as_deref(),
                    args.pagination.page_size,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(linkedin_result(
                    client,
                    response,
                    &format!("/adAccounts/{account_id}/adCampaigns"),
                    Some(account_id),
                    vec![],
                ))
            }
            CampaignsCommand::Get(args) => {
                let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
                let campaign_id = normalize_campaign_id(&args.campaign_id)?;
                let response = campaigns::get_campaign(client, &account_id, &campaign_id).await?;
                Ok(linkedin_result(
                    client,
                    response,
                    &format!("/adAccounts/{account_id}/adCampaigns/{campaign_id}"),
                    Some(account_id),
                    vec![],
                ))
            }
        },
        LinkedInCommand::Creatives { command } => match command {
            CreativesCommand::List(args) => {
                let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
                let params = build_creative_search_params(&args)?;
                let response = creatives::list_creatives(
                    client,
                    &account_id,
                    &params,
                    args.pagination.all,
                    args.pagination.max_items,
                )
                .await?;
                Ok(linkedin_result(
                    client,
                    response,
                    &format!("/adAccounts/{account_id}/creatives"),
                    Some(account_id),
                    vec![],
                ))
            }
            CreativesCommand::Get(args) => {
                let account_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
                let creative_urn = normalize_creative_urn(&args.creative_id)?;
                let response = creatives::get_creative(client, &account_id, &creative_urn).await?;
                Ok(linkedin_result(
                    client,
                    response,
                    &format!("/adAccounts/{account_id}/creatives"),
                    Some(account_id),
                    vec![],
                ))
            }
        },
        LinkedInCommand::Analytics { command } => match command {
            AnalyticsCommand::Query(args) => {
                let query = validate_analytics_query(config, &args)?;
                let account_path_id = query.account_path_id.clone();
                let response =
                    linkedin_reports::query_analytics(client, query.as_endpoint_query()).await?;
                Ok(linkedin_result(
                    client,
                    response,
                    "/adAnalytics",
                    Some(account_path_id),
                    vec![],
                ))
            }
        },
        LinkedInCommand::Auth { .. }
        | LinkedInCommand::Doctor(_)
        | LinkedInCommand::Config { .. } => {
            unreachable!("auth/config/doctor are dispatched before loading LinkedIn credentials")
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn linkedin_result(
    client: &LinkedInClient,
    response: LinkedInResponse,
    endpoint: &str,
    object_id: Option<String>,
    warnings: Vec<String>,
) -> CommandResult {
    let mut envelope = OutputEnvelope::new(
        response.data,
        OutputMeta {
            api_version: client.api_version().to_string(),
            endpoint: endpoint.to_string(),
            object_id,
            request_id: response.request_id,
            report_run_id: None,
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

fn linkedin_command_result(data: Value, endpoint: &str, exit_code: u8) -> CommandResult {
    command_result(
        data,
        endpoint,
        exit_code,
        Some(LINKEDIN_DEFAULT_API_VERSION),
    )
}

async fn list_accessible_accounts_with_roles(
    client: &LinkedInClient,
) -> Result<CommandResult, LinkedInError> {
    let access_response = accounts::list_accessible_account_users(client).await?;
    let access_rows = access_response
        .data
        .get("elements")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut hydrated_accounts = Vec::with_capacity(access_rows.len());
    for access_row in access_rows {
        let account_urn = access_row
            .get("account")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                LinkedInError::Config(
                    "LinkedIn adAccountUsers response did not include an account URN".to_string(),
                )
            })?;
        let role = access_row
            .get("role")
            .and_then(Value::as_str)
            .map(str::to_string);
        let account_id = normalize_account_id(account_urn)?;
        let mut account = accounts::get_account(client, &account_id).await?.data;
        if let Some(role) = role {
            if let Some(object) = account.as_object_mut() {
                object.insert("authenticated_user_role".to_string(), json!(role));
            }
        }
        hydrated_accounts.push(account);
    }

    let mut envelope = OutputEnvelope::new(
        Value::Array(hydrated_accounts),
        OutputMeta {
            api_version: client.api_version().to_string(),
            endpoint: "/adAccounts".to_string(),
            object_id: None,
            request_id: access_response.request_id,
            report_run_id: None,
        },
    );
    envelope.paging = access_response.paging;
    Ok(CommandResult {
        envelope,
        exit_code: 0,
    })
}

fn resolve_account_id(
    config: &LinkedInResolvedConfig,
    explicit_account_id: Option<&str>,
) -> Result<String, LinkedInError> {
    match explicit_account_id {
        Some(account_id) => normalize_account_id(account_id),
        None => config
            .default_account_id
            .as_deref()
            .ok_or_else(|| {
                LinkedInError::InvalidArgument(
                    "LinkedIn account ID is required. Pass --account-id or set providers.linkedin.default_account_id / LINKEDIN_ADS_DEFAULT_ACCOUNT_ID.".to_string(),
                )
            })
            .and_then(normalize_account_id),
    }
}

pub(crate) fn resolve_auth_token_input(args: &AuthSetArgs) -> Result<String, LinkedInError> {
    if args.stdin {
        let input = read_input(Path::new("-")).map_err(LinkedInError::from)?;
        let token = input
            .lines()
            .map(str::trim)
            .find(|value| !value.is_empty())
            .ok_or_else(|| {
                LinkedInError::InvalidArgument(
                    "stdin did not contain a LinkedIn access token".to_string(),
                )
            })?;
        return normalize_nonempty(token, "access token");
    }

    match args.access_token.as_deref() {
        Some(value) => normalize_nonempty(value, "access token"),
        None => normalize_nonempty(
            &prompt_password("LinkedIn access token: ").map_err(LinkedInError::Io)?,
            "access token",
        ),
    }
}

fn normalize_nonempty(value: &str, label: &str) -> Result<String, LinkedInError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(LinkedInError::InvalidArgument(format!(
            "LinkedIn {label} cannot be empty"
        )));
    }
    Ok(trimmed.to_string())
}

fn normalize_account_id(value: &str) -> Result<String, LinkedInError> {
    normalize_numeric_or_urn(value, "urn:li:sponsoredAccount:", "account ID")
}

fn normalize_account_urn(value: &str) -> Result<String, LinkedInError> {
    Ok(format!(
        "urn:li:sponsoredAccount:{}",
        normalize_account_id(value)?
    ))
}

fn normalize_campaign_group_id(value: &str) -> Result<String, LinkedInError> {
    normalize_numeric_or_urn(value, "urn:li:sponsoredCampaignGroup:", "campaign group ID")
}

fn normalize_campaign_group_urn(value: &str) -> Result<String, LinkedInError> {
    Ok(format!(
        "urn:li:sponsoredCampaignGroup:{}",
        normalize_campaign_group_id(value)?
    ))
}

fn normalize_campaign_id(value: &str) -> Result<String, LinkedInError> {
    normalize_numeric_or_urn(value, "urn:li:sponsoredCampaign:", "campaign ID")
}

fn normalize_campaign_urn(value: &str) -> Result<String, LinkedInError> {
    Ok(format!(
        "urn:li:sponsoredCampaign:{}",
        normalize_campaign_id(value)?
    ))
}

fn normalize_creative_urn(value: &str) -> Result<String, LinkedInError> {
    Ok(format!(
        "urn:li:sponsoredCreative:{}",
        normalize_numeric_or_urn(value, "urn:li:sponsoredCreative:", "creative ID")?
    ))
}

fn normalize_numeric_or_urn(
    value: &str,
    urn_prefix: &str,
    label: &str,
) -> Result<String, LinkedInError> {
    let normalized = value.trim();
    let id = normalized.strip_prefix(urn_prefix).unwrap_or(normalized);
    if id.is_empty() || !id.chars().all(|character| character.is_ascii_digit()) {
        return Err(LinkedInError::InvalidArgument(format!(
            "{label} must be numeric or start with `{urn_prefix}`"
        )));
    }
    Ok(id.to_string())
}

fn account_sort_clause(sort_order: SortOrderArg) -> String {
    format!("(field:ID,order:{})", sort_order_value(sort_order))
}

fn sort_order_value(sort_order: SortOrderArg) -> &'static str {
    match sort_order {
        SortOrderArg::Ascending => "ASCENDING",
        SortOrderArg::Descending => "DESCENDING",
    }
}

fn build_account_search_expression(
    args: &AdAccountSearchArgs,
) -> Result<Option<String>, LinkedInError> {
    let mut clauses = Vec::new();

    push_list_clause(
        &mut clauses,
        "id",
        normalize_many(&args.account_ids, normalize_account_urn)?,
    );
    push_list_clause(
        &mut clauses,
        "name",
        normalize_many(&args.names, passthrough_value)?,
    );
    push_list_clause(
        &mut clauses,
        "reference",
        normalize_many(&args.references, passthrough_value)?,
    );
    push_list_clause(
        &mut clauses,
        "type",
        normalize_many(&args.account_types, uppercase_value)?,
    );
    push_list_clause(
        &mut clauses,
        "status",
        normalize_many(&args.statuses, uppercase_value)?,
    );
    push_bool_clause(&mut clauses, "test", args.test);

    if clauses.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!("({})", clauses.join(","))))
    }
}

fn build_campaign_group_search_expression(
    args: &CampaignGroupListArgs,
) -> Result<String, LinkedInError> {
    let mut clauses = Vec::new();

    push_list_clause(
        &mut clauses,
        "id",
        normalize_many(&args.campaign_group_ids, normalize_campaign_group_urn)?,
    );
    push_list_clause(
        &mut clauses,
        "name",
        normalize_many(&args.names, passthrough_value)?,
    );
    push_list_clause(
        &mut clauses,
        "status",
        if args.statuses.is_empty() {
            CAMPAIGN_GROUP_STATUS_VALUES
                .iter()
                .map(|value| (*value).to_string())
                .collect()
        } else {
            normalize_many(&args.statuses, uppercase_value)?
        },
    );
    push_bool_clause(&mut clauses, "test", args.test);

    Ok(format!("({})", clauses.join(",")))
}

fn build_campaign_search_expression(args: &CampaignListArgs) -> Result<String, LinkedInError> {
    let mut clauses = Vec::new();

    push_list_clause(
        &mut clauses,
        "id",
        normalize_many(&args.campaign_ids, normalize_campaign_urn)?,
    );
    push_list_clause(
        &mut clauses,
        "campaignGroup",
        normalize_many(&args.campaign_group_ids, normalize_campaign_group_urn)?,
    );
    push_list_clause(
        &mut clauses,
        "associatedEntity",
        normalize_many(&args.associated_entities, passthrough_value)?,
    );
    push_list_clause(
        &mut clauses,
        "name",
        normalize_many(&args.names, passthrough_value)?,
    );
    push_list_clause(
        &mut clauses,
        "status",
        if args.statuses.is_empty() {
            CAMPAIGN_STATUS_VALUES
                .iter()
                .map(|value| (*value).to_string())
                .collect()
        } else {
            normalize_many(&args.statuses, uppercase_value)?
        },
    );
    push_list_clause(
        &mut clauses,
        "type",
        normalize_many(&args.campaign_types, uppercase_value)?,
    );
    push_bool_clause(&mut clauses, "test", args.test);

    Ok(format!("({})", clauses.join(",")))
}

fn build_creative_search_params(
    args: &CreativeListArgs,
) -> Result<Vec<(String, String)>, LinkedInError> {
    let mut params = vec![("q".to_string(), "criteria".to_string())];

    push_list_param(
        &mut params,
        "creatives",
        normalize_many(&args.creative_ids, normalize_creative_urn)?,
    );
    push_list_param(
        &mut params,
        "campaigns",
        normalize_many(&args.campaign_ids, normalize_campaign_urn)?,
    );
    push_list_param(
        &mut params,
        "contentReferences",
        normalize_many(&args.content_references, passthrough_value)?,
    );
    push_list_param(
        &mut params,
        "intendedStatuses",
        normalize_many(&args.intended_statuses, uppercase_value)?,
    );
    if let Some(test_account) = args.test_account {
        params.push(("isTestAccount".to_string(), test_account.to_string()));
    }
    if let Some(sort_order) = args.sort_order {
        params.push((
            "sortOrder".to_string(),
            sort_order_value(sort_order).to_string(),
        ));
    }
    if let Some(page_size) = args.pagination.page_size {
        params.push(("pageSize".to_string(), page_size.to_string()));
    }
    if let Some(page_token) = args.pagination.page_token.as_deref() {
        params.push(("pageToken".to_string(), page_token.to_string()));
    }

    Ok(params)
}

#[derive(Clone)]
struct ValidatedAnalyticsQuery {
    finder: &'static str,
    pivots: Vec<String>,
    time_granularity: Option<String>,
    date_range: String,
    account_urn: String,
    account_path_id: String,
    campaign_ids: Vec<String>,
    campaign_group_ids: Vec<String>,
    creative_ids: Vec<String>,
    fields: Vec<String>,
    start: Option<u32>,
    page_size: Option<u32>,
    fetch_all: bool,
    max_items: Option<usize>,
}

impl ValidatedAnalyticsQuery {
    fn as_endpoint_query(&self) -> linkedin_reports::AnalyticsQuery<'_> {
        linkedin_reports::AnalyticsQuery {
            finder: self.finder,
            pivots: &self.pivots,
            time_granularity: self.time_granularity.as_deref(),
            date_range: &self.date_range,
            account: &self.account_urn,
            campaign_ids: &self.campaign_ids,
            campaign_group_ids: &self.campaign_group_ids,
            creative_ids: &self.creative_ids,
            fields: &self.fields,
            start: self.start,
            page_size: self.page_size,
            fetch_all: self.fetch_all,
            max_items: self.max_items,
        }
    }
}

fn validate_analytics_query(
    config: &LinkedInResolvedConfig,
    args: &AnalyticsQueryArgs,
) -> Result<ValidatedAnalyticsQuery, LinkedInError> {
    let account_path_id = resolve_account_id(config, args.selector.account_id.as_deref())?;
    let account_urn = format!("urn:li:sponsoredAccount:{account_path_id}");
    let pivots = normalize_many(&args.pivots, uppercase_value)?;
    if pivots.is_empty() {
        return Err(LinkedInError::InvalidArgument(
            "--pivot requires at least one value".to_string(),
        ));
    }

    let date_range = build_date_range(&args.since, args.until.as_deref())?;
    let campaign_ids = normalize_many(&args.campaign_ids, normalize_campaign_urn)?;
    let campaign_group_ids =
        normalize_many(&args.campaign_group_ids, normalize_campaign_group_urn)?;
    let creative_ids = normalize_many(&args.creative_ids, normalize_creative_urn)?;
    let fields = normalize_many(&args.fields, passthrough_value)?;

    let finder = match args.finder {
        AnalyticsFinderArg::Analytics => {
            if pivots.len() != 1 {
                return Err(LinkedInError::InvalidArgument(
                    "analytics finder requires exactly one --pivot".to_string(),
                ));
            }
            if args.time_granularity.is_none() {
                return Err(LinkedInError::InvalidArgument(
                    "analytics finder requires --time-granularity".to_string(),
                ));
            }
            "analytics"
        }
        AnalyticsFinderArg::Statistics => {
            if !(1..=3).contains(&pivots.len()) {
                return Err(LinkedInError::InvalidArgument(
                    "statistics finder requires between one and three --pivot values".to_string(),
                ));
            }
            if args.time_granularity.is_none() {
                return Err(LinkedInError::InvalidArgument(
                    "statistics finder requires --time-granularity".to_string(),
                ));
            }
            "statistics"
        }
        AnalyticsFinderArg::AttributedRevenueMetrics => {
            if !(1..=3).contains(&pivots.len()) {
                return Err(LinkedInError::InvalidArgument(
                    "attributed-revenue-metrics finder requires between one and three --pivot values".to_string(),
                ));
            }
            if !creative_ids.is_empty() {
                return Err(LinkedInError::InvalidArgument(
                    "attributed-revenue-metrics does not support --creative-id".to_string(),
                ));
            }
            if pivots
                .iter()
                .any(|pivot| !matches!(pivot.as_str(), "ACCOUNT" | "CAMPAIGN" | "CAMPAIGN_GROUP"))
            {
                return Err(LinkedInError::InvalidArgument(
                    "attributed-revenue-metrics only supports ACCOUNT, CAMPAIGN_GROUP, and CAMPAIGN pivots".to_string(),
                ));
            }
            validate_revenue_date_range(&args.since, args.until.as_deref())?;
            "attributedRevenueMetrics"
        }
    };

    Ok(ValidatedAnalyticsQuery {
        finder,
        pivots,
        time_granularity: args.time_granularity.clone(),
        date_range,
        account_urn,
        account_path_id,
        campaign_ids,
        campaign_group_ids,
        creative_ids,
        fields,
        start: args.pagination.start,
        page_size: args.pagination.page_size,
        fetch_all: args.pagination.all,
        max_items: args.pagination.max_items,
    })
}

fn build_date_range(since: &str, until: Option<&str>) -> Result<String, LinkedInError> {
    let start = parse_date(since, "--since")?;
    match until {
        Some(until) => {
            let end = parse_date(until, "--until")?;
            if end.days_since_epoch() < start.days_since_epoch() {
                return Err(LinkedInError::InvalidArgument(
                    "--until must be on or after --since".to_string(),
                ));
            }
            Ok(format!(
                "(start:(year:{},month:{},day:{}),end:(year:{},month:{},day:{}))",
                start.year, start.month, start.day, end.year, end.month, end.day
            ))
        }
        None => Ok(format!(
            "(start:(year:{},month:{},day:{}))",
            start.year, start.month, start.day
        )),
    }
}

fn validate_revenue_date_range(since: &str, until: Option<&str>) -> Result<(), LinkedInError> {
    let end = until.ok_or_else(|| {
        LinkedInError::InvalidArgument("attributed-revenue-metrics requires --until".to_string())
    })?;

    let start = parse_date(since, "--since")?;
    let end = parse_date(end, "--until")?;
    let current_date = current_utc_date();

    if end.days_since_epoch() < start.days_since_epoch() {
        return Err(LinkedInError::InvalidArgument(
            "--until must be on or after --since".to_string(),
        ));
    }

    let inclusive_days = end.days_since_epoch() - start.days_since_epoch() + 1;
    if !(30..=366).contains(&inclusive_days) {
        return Err(LinkedInError::InvalidArgument(
            "attributed-revenue-metrics requires a date range between 30 and 366 days inclusive"
                .to_string(),
        ));
    }

    if end.days_since_epoch() > current_date.days_since_epoch() {
        return Err(LinkedInError::InvalidArgument(
            "attributed-revenue-metrics cannot request dates in the future".to_string(),
        ));
    }

    if start.days_since_epoch() < current_date.days_since_epoch() - 366 {
        return Err(LinkedInError::InvalidArgument(
            "attributed-revenue-metrics requires --since to be within the last year".to_string(),
        ));
    }

    Ok(())
}

fn normalize_many<F>(values: &[String], normalizer: F) -> Result<Vec<String>, LinkedInError>
where
    F: Fn(&str) -> Result<String, LinkedInError>,
{
    values.iter().map(|value| normalizer(value)).collect()
}

fn passthrough_value(value: &str) -> Result<String, LinkedInError> {
    normalize_nonempty(value, "value")
}

fn uppercase_value(value: &str) -> Result<String, LinkedInError> {
    Ok(normalize_nonempty(value, "value")?.to_ascii_uppercase())
}

fn push_list_clause(clauses: &mut Vec<String>, field: &str, values: Vec<String>) {
    if !values.is_empty() {
        clauses.push(format!("{field}:(values:List({}))", values.join(",")));
    }
}

fn push_bool_clause(clauses: &mut Vec<String>, field: &str, value: Option<bool>) {
    if let Some(value) = value {
        clauses.push(format!("{field}:{value}"));
    }
}

fn push_list_param(params: &mut Vec<(String, String)>, key: &str, values: Vec<String>) {
    if !values.is_empty() {
        params.push((key.to_string(), format!("List({})", values.join(","))));
    }
}

fn credential_store_check_ok(snapshot: &LinkedInConfigSnapshot) -> bool {
    snapshot.auth.credential_store_available
        || snapshot.auth.access_token.source == LinkedInAccessTokenSource::ShellEnv
}

fn credential_store_detail(snapshot: &LinkedInConfigSnapshot) -> String {
    match snapshot.auth.credential_store_error.as_deref() {
        Some(error) if snapshot.auth.access_token.source == LinkedInAccessTokenSource::ShellEnv => {
            format!("shell env override active; OS credential store unavailable: {error}")
        }
        Some(error) => format!("OS credential store unavailable: {error}"),
        None if snapshot.auth.access_token.keychain_present => {
            "stored LinkedIn access token found in the OS credential store".to_string()
        }
        None if snapshot.auth.credential_store_available => {
            "OS credential store is available; no stored LinkedIn access token found".to_string()
        }
        None => "OS credential store is unavailable".to_string(),
    }
}

fn access_token_detail(env_var: &str, status: &LinkedInAccessTokenStatus) -> String {
    match status.source {
        LinkedInAccessTokenSource::ShellEnv if status.keychain_present => {
            format!("{env_var} is set in shell env and overrides the stored LinkedIn access token")
        }
        LinkedInAccessTokenSource::ShellEnv => format!("{env_var} is set in shell env"),
        LinkedInAccessTokenSource::Keychain => {
            "using stored LinkedIn access token from the OS credential store".to_string()
        }
        LinkedInAccessTokenSource::Missing => format!("{env_var} is missing"),
    }
}

fn linkedin_auth_status_payload(auth: LinkedInAuthSnapshot) -> Value {
    json!({
        "provider": "linkedin",
        "credential_store_available": auth.credential_store_available,
        "credential_store_error": auth.credential_store_error,
        "credentials": {
            "access_token": {
                "env_var": LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.access_token.present,
                "source": auth.access_token.source,
                "keychain_present": auth.access_token.keychain_present,
            }
        }
    })
}

pub(crate) fn auth_storage_error(action: &str, error: &impl std::fmt::Display) -> LinkedInError {
    LinkedInError::Config(format!(
        "failed to {action} in the OS credential store: {error}{}",
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

#[derive(Debug, Clone, Copy)]
struct SimpleDate {
    year: i32,
    month: u32,
    day: u32,
}

impl SimpleDate {
    fn days_since_epoch(self) -> i64 {
        days_from_civil(self.year, self.month as i32, self.day as i32)
    }
}

fn parse_date(value: &str, label: &str) -> Result<SimpleDate, LinkedInError> {
    let trimmed = value.trim();
    let parts = trimmed.split('-').collect::<Vec<_>>();
    let [year, month, day] = parts.as_slice() else {
        return Err(LinkedInError::InvalidArgument(format!(
            "{label} must be in YYYY-MM-DD format"
        )));
    };

    let year = year.parse::<i32>().map_err(|_| {
        LinkedInError::InvalidArgument(format!("{label} must be in YYYY-MM-DD format"))
    })?;
    let month = month.parse::<u32>().map_err(|_| {
        LinkedInError::InvalidArgument(format!("{label} must be in YYYY-MM-DD format"))
    })?;
    let day = day.parse::<u32>().map_err(|_| {
        LinkedInError::InvalidArgument(format!("{label} must be in YYYY-MM-DD format"))
    })?;

    if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
        return Err(LinkedInError::InvalidArgument(format!(
            "{label} must be a valid calendar date"
        )));
    }

    Ok(SimpleDate { year, month, day })
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn current_utc_date() -> SimpleDate {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    civil_from_days(seconds / 86_400)
}

fn days_from_civil(year: i32, month: i32, day: i32) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146097 + doe - 719468) as i64
}

fn civil_from_days(days: i64) -> SimpleDate {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    SimpleDate {
        year: year as i32,
        month: month as u32,
        day: day as u32,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        build_account_search_expression, normalize_creative_urn, parse_date,
        resolve_auth_token_input, validate_revenue_date_range, AdAccountSearchArgs, AuthSetArgs,
        LinkedInAccessTokenSource, LinkedInReportPaginationArgs, SortOrderArg,
    };
    use agent_ads_core::linkedin_config::{LinkedInAccessTokenStatus, LinkedInAuthSnapshot};

    #[test]
    fn resolves_access_token_from_flag() {
        let token = resolve_auth_token_input(&AuthSetArgs {
            stdin: false,
            access_token: Some(" token ".to_string()),
        })
        .unwrap();

        assert_eq!(token, "token");
    }

    #[test]
    fn account_search_expression_includes_filters() {
        let expression = build_account_search_expression(&AdAccountSearchArgs {
            account_ids: vec!["123".to_string()],
            names: vec!["Acme".to_string()],
            references: vec![],
            account_types: vec!["business".to_string()],
            statuses: vec!["active".to_string()],
            test: Some(false),
            sort_order: Some(SortOrderArg::Descending),
            pagination: Default::default(),
        })
        .unwrap()
        .unwrap();

        assert!(expression.contains("urn:li:sponsoredAccount:123"));
        assert!(expression.contains("type:(values:List(BUSINESS))"));
        assert!(expression.contains("test:false"));
    }

    #[test]
    fn parses_dates() {
        let date = parse_date("2026-03-23", "--since").unwrap();
        assert_eq!(date.year, 2026);
        assert_eq!(date.month, 3);
        assert_eq!(date.day, 23);
    }

    #[test]
    fn rejects_impossible_calendar_dates() {
        let error = parse_date("2026-02-31", "--since").unwrap_err();
        assert!(error
            .to_string()
            .contains("--since must be a valid calendar date"));
    }

    #[test]
    fn rejects_non_leap_day_on_non_leap_year() {
        let error = parse_date("2025-02-29", "--since").unwrap_err();
        assert!(error
            .to_string()
            .contains("--since must be a valid calendar date"));
    }

    #[test]
    fn accepts_leap_day_on_leap_year() {
        let date = parse_date("2024-02-29", "--since").unwrap();
        assert_eq!(date.year, 2024);
        assert_eq!(date.month, 2);
        assert_eq!(date.day, 29);
    }

    #[test]
    fn normalizes_numeric_creative_id_to_urn() {
        let creative_urn = normalize_creative_urn("123").unwrap();
        assert_eq!(creative_urn, "urn:li:sponsoredCreative:123");
    }

    #[test]
    fn preserves_valid_numeric_creative_urn() {
        let creative_urn = normalize_creative_urn("urn:li:sponsoredCreative:123").unwrap();
        assert_eq!(creative_urn, "urn:li:sponsoredCreative:123");
    }

    #[test]
    fn report_pagination_args_default_to_empty() {
        let pagination = LinkedInReportPaginationArgs::default();
        assert_eq!(pagination.start, None);
        assert_eq!(pagination.page_size, None);
        assert!(!pagination.all);
        assert_eq!(pagination.max_items, None);
    }

    #[test]
    fn rejects_creative_urn_with_non_numeric_id() {
        let error = normalize_creative_urn("urn:li:sponsoredCreative:not-a-number").unwrap_err();
        assert!(error
            .to_string()
            .contains("creative ID must be numeric or start with"));
    }

    #[test]
    fn rejects_creative_urn_with_empty_suffix() {
        let error = normalize_creative_urn("urn:li:sponsoredCreative:").unwrap_err();
        assert!(error
            .to_string()
            .contains("creative ID must be numeric or start with"));
    }

    #[test]
    fn auth_status_payload_includes_access_token() {
        let payload = super::linkedin_auth_status_payload(LinkedInAuthSnapshot {
            access_token: LinkedInAccessTokenStatus {
                present: true,
                source: LinkedInAccessTokenSource::Keychain,
                keychain_present: true,
            },
            credential_store_available: true,
            credential_store_error: None,
        });

        assert_eq!(payload["provider"], json!("linkedin"));
        assert_eq!(
            payload["credentials"]["access_token"]["source"],
            json!("keychain")
        );
    }

    #[test]
    fn revenue_validation_rejects_short_ranges() {
        let error = validate_revenue_date_range("2026-03-01", Some("2026-03-15")).unwrap_err();
        assert!(error
            .to_string()
            .contains("between 30 and 366 days inclusive"));
    }
}
