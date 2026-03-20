use std::path::{Path, PathBuf};

use agent_ads_core::google_config::{
    google_inspect_auth, normalize_google_customer_id, GoogleAuthSnapshot, GoogleConfigOverrides,
    GoogleConfigSnapshot, GoogleResolvedConfig, GoogleSecretSource, GoogleSecretStatus,
    GOOGLE_DEFAULT_API_VERSION,
};
use agent_ads_core::output::{OutputEnvelope, OutputMeta};
use agent_ads_core::secret_store::SecretStore;
use agent_ads_core::{
    mutate_auth_bundle, GoogleAuthBundle, GoogleClient, GoogleError, GoogleResponse,
    AUTH_BUNDLE_ACCOUNT, AUTH_BUNDLE_SERVICE,
};
use clap::{Args, Subcommand};
use rpassword::prompt_password;
use serde_json::{json, Value};

use crate::{command_result, read_input, resolve_fields, CommandResult, FieldInputArgs};

const GOOGLE_ADS_DEVELOPER_TOKEN_ENV_VAR: &str = "GOOGLE_ADS_DEVELOPER_TOKEN";
const GOOGLE_ADS_CLIENT_ID_ENV_VAR: &str = "GOOGLE_ADS_CLIENT_ID";
const GOOGLE_ADS_CLIENT_SECRET_ENV_VAR: &str = "GOOGLE_ADS_CLIENT_SECRET";
const GOOGLE_ADS_REFRESH_TOKEN_ENV_VAR: &str = "GOOGLE_ADS_REFRESH_TOKEN";

const DEFAULT_CUSTOMER_HIERARCHY_FIELDS: &[&str] = &[
    "customer_client.client_customer",
    "customer_client.id",
    "customer_client.descriptive_name",
    "customer_client.level",
    "customer_client.manager",
    "customer_client.status",
    "customer_client.currency_code",
    "customer_client.time_zone",
    "customer_client.test_account",
];
const DEFAULT_CAMPAIGN_FIELDS: &[&str] = &[
    "campaign.id",
    "campaign.name",
    "campaign.status",
    "campaign.advertising_channel_type",
];
const DEFAULT_ADGROUP_FIELDS: &[&str] = &[
    "ad_group.id",
    "ad_group.name",
    "ad_group.status",
    "campaign.id",
    "campaign.name",
];
const DEFAULT_AD_FIELDS: &[&str] = &[
    "ad_group_ad.ad.id",
    "ad_group_ad.status",
    "ad_group.id",
    "ad_group.name",
    "campaign.id",
    "campaign.name",
];

// ---------------------------------------------------------------------------
// Clap subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum GoogleCommand {
    #[command(about = "List accessible customers and customer hierarchies")]
    Customers {
        #[command(subcommand)]
        command: CustomersCommand,
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
    #[command(about = "Run Google Ads Query Language (GAQL) requests")]
    Gaql {
        #[command(subcommand)]
        command: GaqlCommand,
    },
    #[command(about = "Manage stored Google Ads credentials")]
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
pub enum CustomersCommand {
    #[command(
        about = "List customers accessible to your Google Ads credentials",
        visible_alias = "ls"
    )]
    List,
    #[command(about = "List a customer hierarchy via customer_client GAQL")]
    Hierarchy(CustomerHierarchyArgs),
}

#[derive(Subcommand, Debug)]
pub enum CampaignsCommand {
    #[command(about = "List campaigns for a customer", visible_alias = "ls")]
    List(CampaignListArgs),
}

#[derive(Subcommand, Debug)]
pub enum AdgroupsCommand {
    #[command(about = "List ad groups for a customer", visible_alias = "ls")]
    List(AdGroupListArgs),
}

#[derive(Subcommand, Debug)]
pub enum AdsCommand {
    #[command(about = "List ads for a customer", visible_alias = "ls")]
    List(AdsListArgs),
}

#[derive(Subcommand, Debug)]
pub enum GaqlCommand {
    #[command(about = "Run a paged GAQL search request")]
    Search(GaqlSearchArgs),
    #[command(about = "Run a streamed GAQL search request")]
    SearchStream(GaqlSearchStreamArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    #[command(about = "Store Google Ads credentials in the OS credential store")]
    Set(AuthSetArgs),
    #[command(about = "Show auth source and secure storage status")]
    Status,
    #[command(about = "Delete stored Google Ads credentials")]
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
pub struct GooglePaginationArgs {
    #[arg(long = "page-token", help = "Resume from a Google nextPageToken")]
    pub page_token: Option<String>,
    #[arg(long, help = "Auto-follow all available pages")]
    pub all: bool,
    #[arg(long = "max-items", help = "Stop after collecting N total rows")]
    pub max_items: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct CustomerSelectorArgs {
    #[arg(long = "customer-id", help = "Google customer ID")]
    pub customer_id: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
#[group(id = "google_query_input", required = true, multiple = false)]
pub struct QueryInputArgs {
    #[arg(long, help = "Inline GAQL query")]
    pub query: Option<String>,
    #[arg(long, help = "Read GAQL query from file (- for stdin)")]
    pub query_file: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct CustomerHierarchyArgs {
    #[command(flatten)]
    pub customer: CustomerSelectorArgs,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
    #[command(flatten)]
    pub pagination: GooglePaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CampaignListArgs {
    #[command(flatten)]
    pub customer: CustomerSelectorArgs,
    #[arg(long, help = "Filter campaign status (e.g. ENABLED, PAUSED)")]
    pub status: Option<String>,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
    #[command(flatten)]
    pub pagination: GooglePaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AdGroupListArgs {
    #[command(flatten)]
    pub customer: CustomerSelectorArgs,
    #[arg(long = "campaign-id", help = "Filter to a campaign ID")]
    pub campaign_id: Option<String>,
    #[arg(long, help = "Filter ad group status (e.g. ENABLED, PAUSED)")]
    pub status: Option<String>,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
    #[command(flatten)]
    pub pagination: GooglePaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct AdsListArgs {
    #[command(flatten)]
    pub customer: CustomerSelectorArgs,
    #[arg(long = "campaign-id", help = "Filter to a campaign ID")]
    pub campaign_id: Option<String>,
    #[arg(long = "ad-group-id", help = "Filter to an ad group ID")]
    pub ad_group_id: Option<String>,
    #[arg(long, help = "Filter ad status (e.g. ENABLED, PAUSED)")]
    pub status: Option<String>,
    #[command(flatten)]
    pub field_input: FieldInputArgs,
    #[command(flatten)]
    pub pagination: GooglePaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GaqlSearchArgs {
    #[command(flatten)]
    pub customer: CustomerSelectorArgs,
    #[command(flatten)]
    pub query_input: QueryInputArgs,
    #[command(flatten)]
    pub pagination: GooglePaginationArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GaqlSearchStreamArgs {
    #[command(flatten)]
    pub customer: CustomerSelectorArgs,
    #[command(flatten)]
    pub query_input: QueryInputArgs,
    #[arg(long = "max-items", help = "Stop after collecting N total rows")]
    pub max_items: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct AuthSetArgs {
    #[arg(
        long,
        conflicts_with_all = [
            "developer_token",
            "client_id",
            "client_secret",
            "refresh_token"
        ],
        help = "Read developer token, client ID, client secret, and refresh token from stdin"
    )]
    pub stdin: bool,
    #[arg(long, help = "Google Ads developer token")]
    pub developer_token: Option<String>,
    #[arg(long, help = "OAuth client ID")]
    pub client_id: Option<String>,
    #[arg(long, help = "OAuth client secret")]
    pub client_secret: Option<String>,
    #[arg(long, help = "OAuth refresh token")]
    pub refresh_token: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    #[arg(
        long,
        help = "Also exchange the refresh token and ping the Google Ads API"
    )]
    pub api: bool,
}

// ---------------------------------------------------------------------------
// Dispatch: auth, config, doctor (before token resolution)
// ---------------------------------------------------------------------------

pub fn handle_auth(
    command: AuthCommand,
    secret_store: &dyn SecretStore,
) -> Result<CommandResult, GoogleError> {
    match command {
        AuthCommand::Set(args) => {
            let inputs = resolve_google_auth_inputs(&args)?;
            let outcome = mutate_auth_bundle(secret_store, move |bundle| {
                bundle.google = Some(GoogleAuthBundle {
                    developer_token: Some(inputs.developer_token),
                    client_id: Some(inputs.client_id),
                    client_secret: Some(inputs.client_secret),
                    refresh_token: Some(inputs.refresh_token),
                });
            })
            .map_err(|error| google_auth_storage_error("store Google Ads credentials", &error))?;

            Ok(google_command_result(
                json!({
                    "provider": "google",
                    "stored": true,
                    "recovered_invalid_bundle": outcome.recovered_invalid_bundle,
                    "credentials_stored": [
                        "developer_token",
                        "client_id",
                        "client_secret",
                        "refresh_token"
                    ],
                }),
                "/google/auth/set",
                0,
            ))
        }
        AuthCommand::Status => Ok(google_command_result(
            google_auth_status_payload(google_inspect_auth(secret_store)),
            "/google/auth/status",
            0,
        )),
        AuthCommand::Delete => {
            let mut deleted_developer_token = false;
            let mut deleted_client_id = false;
            let mut deleted_client_secret = false;
            let mut deleted_refresh_token = false;
            let outcome = mutate_auth_bundle(secret_store, |bundle| {
                let deleted_google = bundle.google.take();
                deleted_developer_token = deleted_google
                    .as_ref()
                    .and_then(|google| google.developer_token.as_ref())
                    .is_some();
                deleted_client_id = deleted_google
                    .as_ref()
                    .and_then(|google| google.client_id.as_ref())
                    .is_some();
                deleted_client_secret = deleted_google
                    .as_ref()
                    .and_then(|google| google.client_secret.as_ref())
                    .is_some();
                deleted_refresh_token = deleted_google
                    .as_ref()
                    .and_then(|google| google.refresh_token.as_ref())
                    .is_some();
            })
            .map_err(|error| google_auth_storage_error("delete Google Ads credentials", &error))?;

            Ok(google_command_result(
                json!({
                    "provider": "google",
                    "developer_token_deleted": deleted_developer_token,
                    "client_id_deleted": deleted_client_id,
                    "client_secret_deleted": deleted_client_secret,
                    "refresh_token_deleted": deleted_refresh_token,
                    "recovered_invalid_bundle": outcome.recovered_invalid_bundle,
                }),
                "/google/auth/delete",
                0,
            ))
        }
    }
}

pub fn handle_config(
    command: ConfigCommand,
    snapshot: GoogleConfigSnapshot,
) -> Result<CommandResult, GoogleError> {
    match command {
        ConfigCommand::Path => Ok(google_command_result(
            json!({
                "path": snapshot.config_path,
                "exists": snapshot.config_file_exists,
            }),
            "/google/config/path",
            0,
        )),
        ConfigCommand::Show => Ok(google_command_result(
            json!(snapshot),
            "/google/config/show",
            0,
        )),
        ConfigCommand::Validate => Ok(google_command_result(
            json!({
                "valid": true,
                "config": snapshot,
            }),
            "/google/config/validate",
            0,
        )),
    }
}

pub async fn handle_doctor(
    args: DoctorArgs,
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &GoogleConfigOverrides,
    snapshot: GoogleConfigSnapshot,
) -> Result<CommandResult, GoogleError> {
    let mut checks = vec![
        json!({
            "name": "credential_store",
            "ok": google_credential_store_check_ok(&snapshot),
            "detail": google_credential_store_detail(&snapshot),
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
            "name": "developer_token",
            "ok": snapshot.auth.developer_token.present,
            "detail": google_secret_detail(
                GOOGLE_ADS_DEVELOPER_TOKEN_ENV_VAR,
                "developer token",
                &snapshot.auth.developer_token,
            ),
        }),
        json!({
            "name": "client_id",
            "ok": snapshot.auth.client_id.present,
            "detail": google_secret_detail(
                GOOGLE_ADS_CLIENT_ID_ENV_VAR,
                "client ID",
                &snapshot.auth.client_id,
            ),
        }),
        json!({
            "name": "client_secret",
            "ok": snapshot.auth.client_secret.present,
            "detail": google_secret_detail(
                GOOGLE_ADS_CLIENT_SECRET_ENV_VAR,
                "client secret",
                &snapshot.auth.client_secret,
            ),
        }),
        json!({
            "name": "refresh_token",
            "ok": snapshot.auth.refresh_token.present,
            "detail": google_secret_detail(
                GOOGLE_ADS_REFRESH_TOKEN_ENV_VAR,
                "refresh token",
                &snapshot.auth.refresh_token,
            ),
        }),
    ];

    let mut ok = required_google_credentials_present(&snapshot.auth);
    if args.api {
        if ok {
            match GoogleResolvedConfig::load(config_path, secret_store, overrides) {
                Ok(config) => match GoogleClient::from_config(&config).await {
                    Ok(client) => match client.list_accessible_customers().await {
                        Ok(response) => {
                            let count =
                                response.data.as_array().map(|rows| rows.len()).unwrap_or(0);
                            checks.push(json!({
                                "name": "api_ping",
                                "ok": true,
                                "detail": format!("credentials accepted by Google Ads API; sampled {} accessible customer record(s)", count)
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
                "detail": "skipped because required Google Ads credentials are missing"
            }));
        }
    }

    Ok(google_command_result(
        json!({
            "ok": ok,
            "checks": checks,
            "config": snapshot,
        }),
        "/google/doctor",
        if ok { 0 } else { 1 },
    ))
}

// ---------------------------------------------------------------------------
// Dispatch: authenticated commands
// ---------------------------------------------------------------------------

pub async fn dispatch_google_with_client(
    client: &GoogleClient,
    config: &GoogleResolvedConfig,
    command: GoogleCommand,
) -> Result<CommandResult, GoogleError> {
    match command {
        GoogleCommand::Customers { command } => match command {
            CustomersCommand::List => {
                let response = client.list_accessible_customers().await?;
                Ok(google_result(
                    client,
                    response,
                    "/google/customers/list",
                    None,
                    vec![],
                ))
            }
            CustomersCommand::Hierarchy(args) => {
                let customer_id =
                    resolve_customer_id(config, args.customer.customer_id.as_deref())?;
                let fields =
                    resolve_google_fields(&args.field_input, DEFAULT_CUSTOMER_HIERARCHY_FIELDS)?;
                let query = build_select_query(
                    "customer_client",
                    &fields,
                    &[],
                    order_by_clause(&fields, &["customer_client.level", "customer_client.id"])
                        .as_deref(),
                );
                execute_paged_google_query(
                    client,
                    &customer_id,
                    &query,
                    "/google/customers/hierarchy",
                    &args.pagination,
                )
                .await
            }
        },
        GoogleCommand::Campaigns { command } => match command {
            CampaignsCommand::List(args) => {
                let customer_id =
                    resolve_customer_id(config, args.customer.customer_id.as_deref())?;
                let fields = resolve_google_fields(&args.field_input, DEFAULT_CAMPAIGN_FIELDS)?;
                let mut filters = Vec::new();
                if let Some(status) = args.status.as_deref() {
                    filters.push(format!(
                        "campaign.status = '{}'",
                        escape_gaql_string(&status.trim().to_uppercase())
                    ));
                }
                let query = build_select_query(
                    "campaign",
                    &fields,
                    &filters,
                    order_by_clause(&fields, &["campaign.id"]).as_deref(),
                );
                execute_paged_google_query(
                    client,
                    &customer_id,
                    &query,
                    "/google/campaigns/list",
                    &args.pagination,
                )
                .await
            }
        },
        GoogleCommand::Adgroups { command } => match command {
            AdgroupsCommand::List(args) => {
                let customer_id =
                    resolve_customer_id(config, args.customer.customer_id.as_deref())?;
                let fields = resolve_google_fields(&args.field_input, DEFAULT_ADGROUP_FIELDS)?;
                let mut filters = Vec::new();
                if let Some(campaign_id) = args.campaign_id.as_deref() {
                    filters.push(format!(
                        "campaign.id = {}",
                        normalize_numeric_filter(campaign_id, "campaign ID")?
                    ));
                }
                if let Some(status) = args.status.as_deref() {
                    filters.push(format!(
                        "ad_group.status = '{}'",
                        escape_gaql_string(&status.trim().to_uppercase())
                    ));
                }
                let query = build_select_query(
                    "ad_group",
                    &fields,
                    &filters,
                    order_by_clause(&fields, &["ad_group.id"]).as_deref(),
                );
                execute_paged_google_query(
                    client,
                    &customer_id,
                    &query,
                    "/google/adgroups/list",
                    &args.pagination,
                )
                .await
            }
        },
        GoogleCommand::Ads { command } => match command {
            AdsCommand::List(args) => {
                let customer_id =
                    resolve_customer_id(config, args.customer.customer_id.as_deref())?;
                let fields = resolve_google_fields(&args.field_input, DEFAULT_AD_FIELDS)?;
                let mut filters = Vec::new();
                if let Some(campaign_id) = args.campaign_id.as_deref() {
                    filters.push(format!(
                        "campaign.id = {}",
                        normalize_numeric_filter(campaign_id, "campaign ID")?
                    ));
                }
                if let Some(ad_group_id) = args.ad_group_id.as_deref() {
                    filters.push(format!(
                        "ad_group.id = {}",
                        normalize_numeric_filter(ad_group_id, "ad group ID")?
                    ));
                }
                if let Some(status) = args.status.as_deref() {
                    filters.push(format!(
                        "ad_group_ad.status = '{}'",
                        escape_gaql_string(&status.trim().to_uppercase())
                    ));
                }
                let query = build_select_query(
                    "ad_group_ad",
                    &fields,
                    &filters,
                    order_by_clause(&fields, &["ad_group_ad.ad.id"]).as_deref(),
                );
                execute_paged_google_query(
                    client,
                    &customer_id,
                    &query,
                    "/google/ads/list",
                    &args.pagination,
                )
                .await
            }
        },
        GoogleCommand::Gaql { command } => match command {
            GaqlCommand::Search(args) => {
                let customer_id =
                    resolve_customer_id(config, args.customer.customer_id.as_deref())?;
                let query = resolve_query_input(&args.query_input)?;
                execute_paged_google_query(
                    client,
                    &customer_id,
                    &query,
                    "/google/gaql/search",
                    &args.pagination,
                )
                .await
            }
            GaqlCommand::SearchStream(args) => {
                let customer_id =
                    resolve_customer_id(config, args.customer.customer_id.as_deref())?;
                let query = resolve_query_input(&args.query_input)?;
                let response = client
                    .search_stream(&customer_id, &query, args.max_items)
                    .await?;
                Ok(google_result(
                    client,
                    response,
                    "/google/gaql/search-stream",
                    Some(customer_id),
                    vec![],
                ))
            }
        },
        GoogleCommand::Auth { .. } | GoogleCommand::Doctor(_) | GoogleCommand::Config { .. } => {
            unreachable!("auth/config/doctor are dispatched before loading Google credentials")
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn google_result(
    client: &GoogleClient,
    response: GoogleResponse,
    endpoint: &str,
    customer_id: Option<String>,
    warnings: Vec<String>,
) -> CommandResult {
    let mut envelope = OutputEnvelope::new(
        response.data,
        OutputMeta {
            api_version: client.api_version().to_string(),
            endpoint: endpoint.to_string(),
            object_id: customer_id,
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

fn google_command_result(data: Value, endpoint: &str, exit_code: u8) -> CommandResult {
    command_result(data, endpoint, exit_code, Some(GOOGLE_DEFAULT_API_VERSION))
}

async fn execute_paged_google_query(
    client: &GoogleClient,
    customer_id: &str,
    query: &str,
    endpoint: &str,
    pagination: &GooglePaginationArgs,
) -> Result<CommandResult, GoogleError> {
    let response = if pagination.all {
        client
            .search_all(
                customer_id,
                query,
                pagination.page_token.as_deref(),
                pagination.max_items,
            )
            .await?
    } else {
        client
            .search(
                customer_id,
                query,
                pagination.page_token.as_deref(),
                pagination.max_items,
            )
            .await?
    };
    Ok(google_result(
        client,
        response,
        endpoint,
        Some(customer_id.to_string()),
        vec![],
    ))
}

fn resolve_customer_id(
    config: &GoogleResolvedConfig,
    explicit_customer_id: Option<&str>,
) -> Result<String, GoogleError> {
    match explicit_customer_id {
        Some(customer_id) => normalize_google_customer_id(customer_id),
        None => config.default_customer_id.clone().ok_or_else(|| {
            GoogleError::InvalidArgument(
                "Google customer ID is required. Pass --customer-id or set providers.google.default_customer_id / GOOGLE_ADS_DEFAULT_CUSTOMER_ID.".to_string(),
            )
        }),
    }
}

fn resolve_google_fields(
    args: &FieldInputArgs,
    defaults: &[&str],
) -> Result<Vec<String>, GoogleError> {
    let fields = resolve_fields(args).map_err(GoogleError::from)?;
    if fields.is_empty() {
        Ok(defaults.iter().map(|field| (*field).to_string()).collect())
    } else {
        Ok(fields)
    }
}

fn resolve_query_input(args: &QueryInputArgs) -> Result<String, GoogleError> {
    if let Some(query) = args.query.as_deref() {
        let query = query.trim();
        if query.is_empty() {
            return Err(GoogleError::InvalidArgument(
                "query input was empty".to_string(),
            ));
        }
        return Ok(query.to_string());
    }

    let path = args.query_file.as_deref().ok_or_else(|| {
        GoogleError::InvalidArgument("one of --query or --query-file is required".to_string())
    })?;
    let query = read_input(path).map_err(GoogleError::from)?;
    let query = query.trim();
    if query.is_empty() {
        return Err(GoogleError::InvalidArgument(
            "query file was empty".to_string(),
        ));
    }
    Ok(query.to_string())
}

fn build_select_query(
    resource: &str,
    fields: &[String],
    filters: &[String],
    order_by: Option<&str>,
) -> String {
    let mut query = format!("SELECT {} FROM {}", fields.join(", "), resource);
    if !filters.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&filters.join(" AND "));
    }
    if let Some(order_by) = order_by {
        query.push_str(" ORDER BY ");
        query.push_str(order_by);
    }
    query
}

fn order_by_clause(fields: &[String], order_fields: &[&str]) -> Option<String> {
    let supports_ordering = order_fields.iter().all(|order_field| {
        fields
            .iter()
            .any(|field| field.trim().eq_ignore_ascii_case(order_field))
    });

    supports_ordering.then(|| order_fields.join(", "))
}

fn escape_gaql_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn normalize_numeric_filter(value: &str, label: &str) -> Result<String, GoogleError> {
    let normalized = value.trim();
    if normalized.is_empty()
        || !normalized
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(GoogleError::InvalidArgument(format!(
            "{label} must be numeric"
        )));
    }
    Ok(normalized.to_string())
}

#[derive(Debug)]
pub(crate) struct GoogleAuthInputs {
    pub developer_token: String,
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
}

pub(crate) fn resolve_google_auth_inputs(
    args: &AuthSetArgs,
) -> Result<GoogleAuthInputs, GoogleError> {
    if args.stdin {
        let input = read_input(Path::new("-")).map_err(GoogleError::from)?;
        return parse_google_auth_inputs_from_stdin(&input);
    }

    Ok(GoogleAuthInputs {
        developer_token: match args.developer_token.as_deref() {
            Some(value) => normalize_google_secret(value, "developer token")?,
            None => normalize_google_secret(
                &prompt_password("Google Ads developer token: ").map_err(GoogleError::Io)?,
                "developer token",
            )?,
        },
        client_id: match args.client_id.as_deref() {
            Some(value) => normalize_google_secret(value, "client ID")?,
            None => normalize_google_secret(
                &prompt_password("Google OAuth client ID: ").map_err(GoogleError::Io)?,
                "client ID",
            )?,
        },
        client_secret: match args.client_secret.as_deref() {
            Some(value) => normalize_google_secret(value, "client secret")?,
            None => normalize_google_secret(
                &prompt_password("Google OAuth client secret: ").map_err(GoogleError::Io)?,
                "client secret",
            )?,
        },
        refresh_token: match args.refresh_token.as_deref() {
            Some(value) => normalize_google_secret(value, "refresh token")?,
            None => normalize_google_secret(
                &prompt_password("Google OAuth refresh token: ").map_err(GoogleError::Io)?,
                "refresh token",
            )?,
        },
    })
}

fn parse_google_auth_inputs_from_stdin(input: &str) -> Result<GoogleAuthInputs, GoogleError> {
    let values = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    match values.as_slice() {
        [developer_token, client_id, client_secret, refresh_token] => Ok(GoogleAuthInputs {
            developer_token: developer_token.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            refresh_token: refresh_token.to_string(),
        }),
        [] => Err(GoogleError::InvalidArgument(
            "stdin did not contain Google Ads credentials".to_string(),
        )),
        _ => Err(GoogleError::InvalidArgument(
            "stdin must contain developer token, client ID, client secret, and refresh token on separate lines".to_string(),
        )),
    }
}

fn normalize_google_secret(value: &str, label: &str) -> Result<String, GoogleError> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(GoogleError::InvalidArgument(format!(
            "{label} input was empty"
        )));
    }
    Ok(normalized)
}

fn required_google_credentials_present(auth: &GoogleAuthSnapshot) -> bool {
    auth.developer_token.present
        && auth.client_id.present
        && auth.client_secret.present
        && auth.refresh_token.present
}

fn google_credential_store_check_ok(snapshot: &GoogleConfigSnapshot) -> bool {
    snapshot.auth.credential_store_available || google_shell_override_active(&snapshot.auth)
}

fn google_shell_override_active(auth: &GoogleAuthSnapshot) -> bool {
    auth.developer_token.source == GoogleSecretSource::ShellEnv
        && auth.client_id.source == GoogleSecretSource::ShellEnv
        && auth.client_secret.source == GoogleSecretSource::ShellEnv
        && auth.refresh_token.source == GoogleSecretSource::ShellEnv
}

fn google_credential_store_detail(snapshot: &GoogleConfigSnapshot) -> String {
    match snapshot.auth.credential_store_error.as_deref() {
        Some(error) if google_shell_override_active(&snapshot.auth) => {
            format!("shell env overrides active; OS credential store unavailable: {error}")
        }
        Some(error) => format!("OS credential store unavailable: {error}"),
        None if snapshot.auth.developer_token.keychain_present
            || snapshot.auth.client_id.keychain_present
            || snapshot.auth.client_secret.keychain_present
            || snapshot.auth.refresh_token.keychain_present =>
        {
            "stored Google Ads credentials found in the OS credential store".to_string()
        }
        None if snapshot.auth.credential_store_available => {
            "OS credential store is available; no stored Google Ads credentials found".to_string()
        }
        None => "OS credential store is unavailable".to_string(),
    }
}

fn google_secret_detail(env_var: &str, label: &str, status: &GoogleSecretStatus) -> String {
    match status.source {
        GoogleSecretSource::ShellEnv if status.keychain_present => {
            format!("{env_var} is set in shell env and overrides the stored {label}")
        }
        GoogleSecretSource::ShellEnv => format!("{env_var} is set in shell env"),
        GoogleSecretSource::Keychain => {
            format!("using stored Google Ads {label} from the OS credential store")
        }
        GoogleSecretSource::Missing => format!("{env_var} is missing"),
    }
}

fn google_auth_status_payload(auth: GoogleAuthSnapshot) -> Value {
    json!({
        "provider": "google",
        "credential_store_available": auth.credential_store_available,
        "credential_store_error": auth.credential_store_error,
        "credentials": {
            "developer_token": {
                "env_var": GOOGLE_ADS_DEVELOPER_TOKEN_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.developer_token.present,
                "source": auth.developer_token.source,
                "keychain_present": auth.developer_token.keychain_present,
            },
            "client_id": {
                "env_var": GOOGLE_ADS_CLIENT_ID_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.client_id.present,
                "source": auth.client_id.source,
                "keychain_present": auth.client_id.keychain_present,
            },
            "client_secret": {
                "env_var": GOOGLE_ADS_CLIENT_SECRET_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.client_secret.present,
                "source": auth.client_secret.source,
                "keychain_present": auth.client_secret.keychain_present,
            },
            "refresh_token": {
                "env_var": GOOGLE_ADS_REFRESH_TOKEN_ENV_VAR,
                "credential_store_service": AUTH_BUNDLE_SERVICE,
                "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                "present": auth.refresh_token.present,
                "source": auth.refresh_token.source,
                "keychain_present": auth.refresh_token.keychain_present,
            }
        }
    })
}

pub(crate) fn google_auth_storage_error(
    action: &str,
    error: &impl std::fmt::Display,
) -> GoogleError {
    GoogleError::Config(format!(
        "failed to {action} in the OS credential store: {error}{}",
        google_linux_secure_storage_hint()
    ))
}

fn google_linux_secure_storage_hint() -> &'static str {
    if cfg!(target_os = "linux") {
        " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet."
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{google_auth_status_payload, order_by_clause, parse_google_auth_inputs_from_stdin};
    use agent_ads_core::google_config::{
        GoogleAuthSnapshot, GoogleSecretSource, GoogleSecretStatus,
    };

    #[test]
    fn parses_stdin_auth_inputs() {
        let inputs = parse_google_auth_inputs_from_stdin(
            " developer-token \n client-id \n client-secret \n refresh-token \n",
        )
        .unwrap();

        assert_eq!(inputs.developer_token, "developer-token");
        assert_eq!(inputs.client_id, "client-id");
        assert_eq!(inputs.client_secret, "client-secret");
        assert_eq!(inputs.refresh_token, "refresh-token");
    }

    #[test]
    fn rejects_short_stdin_auth_inputs() {
        let error =
            parse_google_auth_inputs_from_stdin("developer-token\nclient-id\n").unwrap_err();
        assert!(error
            .to_string()
            .contains("developer token, client ID, client secret, and refresh token"));
    }

    #[test]
    fn auth_status_payload_includes_all_google_credentials() {
        let payload = google_auth_status_payload(GoogleAuthSnapshot {
            developer_token: GoogleSecretStatus {
                present: true,
                source: GoogleSecretSource::Keychain,
                keychain_present: true,
            },
            client_id: GoogleSecretStatus {
                present: true,
                source: GoogleSecretSource::ShellEnv,
                keychain_present: false,
            },
            client_secret: GoogleSecretStatus {
                present: true,
                source: GoogleSecretSource::ShellEnv,
                keychain_present: false,
            },
            refresh_token: GoogleSecretStatus {
                present: true,
                source: GoogleSecretSource::Missing,
                keychain_present: false,
            },
            credential_store_available: true,
            credential_store_error: None,
        });

        assert_eq!(payload["provider"], json!("google"));
        assert_eq!(
            payload["credentials"]["developer_token"]["present"],
            json!(true)
        );
        assert_eq!(
            payload["credentials"]["refresh_token"]["source"],
            json!("missing")
        );
    }

    #[test]
    fn order_by_clause_requires_selected_fields() {
        let fields = vec!["campaign.name".to_string()];
        assert_eq!(order_by_clause(&fields, &["campaign.id"]), None);
    }

    #[test]
    fn order_by_clause_keeps_supported_fields() {
        let fields = vec!["campaign.id".to_string(), "campaign.name".to_string()];
        assert_eq!(
            order_by_clause(&fields, &["campaign.id"]).as_deref(),
            Some("campaign.id")
        );
    }
}
