use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use agent_ads_core::client::GraphResponse;
use agent_ads_core::config::{
    inspect, load_env, ConfigOverrides, ConfigSnapshot, EnvFileSource, EnvFileState, ResolvedConfig,
};
use agent_ads_core::endpoints::{accounts, changes, creative, objects, reports, tracking};
use agent_ads_core::error::{GraphApiError, MetaAdsError};
use agent_ads_core::output::{
    render_output, OutputEnvelope, OutputFormat, OutputMeta, RenderOptions,
};
use agent_ads_core::GraphClient;
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use tokio::time::{sleep, Instant};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "agent-ads",
    version,
    about = "Unix-first multi-provider ads CLI"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Config file path [default: agent-ads.config.json]"
    )]
    config: Option<PathBuf>,
    #[arg(
        long = "env-file",
        global = true,
        help = "Env file for secrets [default: ./.env]"
    )]
    env_file: Option<PathBuf>,
    #[arg(long, global = true, help = "Override API base URL")]
    api_base_url: Option<String>,
    #[arg(long, global = true, help = "Override API version (e.g. v25.0)")]
    api_version: Option<String>,
    #[arg(long, global = true, help = "HTTP request timeout in seconds")]
    timeout_seconds: Option<u64>,
    #[arg(long, global = true, help = "Output format")]
    format: Option<FormatArg>,
    #[arg(long, global = true, help = "Write output to file (- for stdout)")]
    output: Option<PathBuf>,
    #[arg(long, global = true, help = "Pretty-print JSON output")]
    pretty: bool,
    #[arg(
        long,
        global = true,
        help = "Include response metadata, paging, and warnings"
    )]
    envelope: bool,
    #[arg(long, global = true, help = "Add metadata columns to CSV output")]
    include_meta: bool,
    #[arg(
        short,
        long,
        global = true,
        help = "Suppress warnings and non-data output"
    )]
    quiet: bool,
    #[arg(short, long, global = true, action = ArgAction::Count, conflicts_with = "quiet", help = "Increase log verbosity (-v info, -vv debug)")]
    verbose: u8,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FormatArg {
    Json,
    Jsonl,
    Csv,
}

impl From<FormatArg> for OutputFormat {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Json => Self::Json,
            FormatArg::Jsonl => Self::Jsonl,
            FormatArg::Csv => Self::Csv,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Inspect available and planned ad providers")]
    Providers {
        #[command(subcommand)]
        command: ProvidersCommand,
    },
    #[command(about = "Meta (Facebook/Instagram) Marketing API commands")]
    Meta {
        #[command(subcommand)]
        command: MetaCommand,
    },
    #[command(about = "Google Ads provider namespace (not implemented yet)")]
    Google,
    #[command(about = "TikTok Ads provider namespace (not implemented yet)")]
    Tiktok,
}

#[derive(Subcommand, Debug)]
enum ProvidersCommand {
    #[command(about = "List available and planned providers", visible_alias = "ls")]
    List,
}

#[derive(Subcommand, Debug)]
enum MetaCommand {
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
    #[command(about = "Verify auth, config, and API connectivity")]
    Doctor(DoctorArgs),
    #[command(about = "Inspect and validate configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Subcommand, Debug)]
enum BusinessesCommand {
    #[command(
        about = "List businesses accessible to your token",
        visible_alias = "ls"
    )]
    List(BusinessListArgs),
}

#[derive(Subcommand, Debug)]
enum AdAccountsCommand {
    #[command(about = "List ad accounts by scope", visible_alias = "ls")]
    List(AdAccountListArgs),
}

#[derive(Subcommand, Debug)]
enum ObjectListCommand {
    #[command(about = "List objects in an ad account", visible_alias = "ls")]
    List(AccountListArgs),
}

#[derive(Subcommand, Debug)]
enum InsightsCommand {
    #[command(about = "Run a synchronous insights query")]
    Query(InsightsQueryArgs),
    #[command(about = "Query insights with optional async mode")]
    Export(InsightsExportArgs),
}

#[derive(Subcommand, Debug)]
enum ReportRunsCommand {
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
enum CreativesCommand {
    #[command(about = "Fetch a creative by ID", visible_alias = "cat")]
    Get(CreativeGetArgs),
    #[command(about = "Get rendered ad preview")]
    Preview(CreativePreviewArgs),
}

#[derive(Subcommand, Debug)]
enum ActivitiesCommand {
    #[command(
        about = "List account activity and change history",
        visible_alias = "ls"
    )]
    List(ActivitiesArgs),
}

#[derive(Subcommand, Debug)]
enum TrackingListCommand {
    #[command(about = "List tracking objects in an ad account", visible_alias = "ls")]
    List(AccountListArgs),
}

#[derive(Subcommand, Debug)]
enum DatasetsCommand {
    #[command(about = "Get dataset quality metrics", visible_alias = "cat")]
    Get(DatasetGetArgs),
}

#[derive(Subcommand, Debug)]
enum PixelHealthCommand {
    #[command(about = "Get combined pixel health diagnostics", visible_alias = "cat")]
    Get(PixelHealthArgs),
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    #[command(about = "Show resolved config file path")]
    Path,
    #[command(about = "Show full resolved configuration")]
    Show,
    #[command(about = "Validate config file")]
    Validate,
}

#[derive(Args, Debug, Clone, Default)]
struct PaginationArgs {
    #[arg(long = "page-size", alias = "limit", help = "Items per API request")]
    page_size: Option<u32>,
    #[arg(
        long = "cursor",
        alias = "after",
        help = "Resume from a pagination cursor"
    )]
    cursor: Option<String>,
    #[arg(long, help = "Auto-paginate through all results")]
    all: bool,
    #[arg(long = "max-items", help = "Stop after collecting N total items")]
    max_items: Option<usize>,
}

#[derive(Args, Debug, Clone, Default)]
struct FieldInputArgs {
    #[arg(long, value_delimiter = ',', help = "Comma-separated field names")]
    fields: Vec<String>,
    #[arg(long, help = "Read field names from file (- for stdin)")]
    fields_file: Option<PathBuf>,
}

#[derive(Args, Debug, Clone, Default)]
#[group(id = "selector", multiple = false)]
struct SelectorArgs {
    #[arg(long, help = "Ad account ID (e.g. act_1234567890)")]
    account: Option<String>,
    #[arg(
        long = "object",
        alias = "object-id",
        help = "Arbitrary Graph API object ID"
    )]
    object: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
struct TimeInputArgs {
    #[arg(long, requires = "until", conflicts_with_all = ["date_preset", "time_range_file"], help = "Start date (YYYY-MM-DD)")]
    since: Option<String>,
    #[arg(long, requires = "since", conflicts_with_all = ["date_preset", "time_range_file"], help = "End date (YYYY-MM-DD)")]
    until: Option<String>,
    #[arg(long, conflicts_with_all = ["since", "until", "time_range_file"], help = "Named date preset (e.g. last_7d, last_30d)")]
    date_preset: Option<String>,
    #[arg(long, conflicts_with_all = ["since", "until", "date_preset"], help = "JSON file with since/until (- for stdin)")]
    time_range_file: Option<PathBuf>,
}

#[derive(Args, Debug, Clone, Default)]
struct BusinessListArgs {
    #[command(flatten)]
    pagination: PaginationArgs,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ScopeArg {
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
struct AdAccountListArgs {
    #[arg(long)]
    business_id: Option<String>,
    #[arg(long, value_enum, default_value_t = ScopeArg::Accessible)]
    scope: ScopeArg,
    #[command(flatten)]
    pagination: PaginationArgs,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct AccountListArgs {
    #[arg(long)]
    account: Option<String>,
    #[command(flatten)]
    pagination: PaginationArgs,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct InsightsRequestArgs {
    #[command(flatten)]
    selector: SelectorArgs,
    #[arg(long, help = "Aggregation level: account, campaign, adset, ad")]
    level: Option<String>,
    #[arg(long, help = "Time bucketing: 1 (daily), 7, 14, monthly, all_days")]
    time_increment: Option<String>,
    #[command(flatten)]
    field_input: FieldInputArgs,
    #[command(flatten)]
    time_input: TimeInputArgs,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Dimension breakdowns (e.g. age,gender,country)"
    )]
    breakdowns: Vec<String>,
    #[arg(
        long = "action-breakdowns",
        value_delimiter = ',',
        help = "Action breakdowns (requires actions in --fields)"
    )]
    action_breakdowns: Vec<String>,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Sort order (e.g. spend_descending)"
    )]
    sort: Vec<String>,
    #[arg(
        long = "filter",
        alias = "filtering",
        help = "Inline filter JSON (repeatable)"
    )]
    filters: Vec<String>,
    #[arg(long, help = "JSON file with filter array (- for stdin)")]
    filter_file: Option<PathBuf>,
    #[arg(
        long = "attribution-windows",
        alias = "action-attribution-windows",
        value_delimiter = ',',
        help = "Attribution windows (e.g. 1d_click,7d_click,1d_view)"
    )]
    attribution_windows: Vec<String>,
}

#[derive(Args, Debug, Clone)]
struct InsightsQueryArgs {
    #[command(flatten)]
    request: InsightsRequestArgs,
    #[command(flatten)]
    pagination: PaginationArgs,
}

#[derive(Args, Debug, Clone)]
struct InsightsExportArgs {
    #[command(flatten)]
    request: InsightsRequestArgs,
    #[command(flatten)]
    pagination: PaginationArgs,
    #[arg(long = "async", help = "Use async report run instead of inline query")]
    async_mode: bool,
    #[arg(
        long,
        requires = "async_mode",
        help = "Poll until complete, then return results"
    )]
    wait: bool,
    #[arg(long, default_value_t = 5, help = "Seconds between status polls")]
    poll_interval_seconds: u64,
    #[arg(
        long,
        default_value_t = 3600,
        help = "Max seconds to wait before timeout"
    )]
    wait_timeout_seconds: u64,
}

#[derive(Args, Debug, Clone)]
struct ReportRunStatusArgs {
    #[arg(long = "id", alias = "report-run-id")]
    id: String,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct ReportRunResultsArgs {
    #[arg(long = "id", alias = "report-run-id")]
    id: String,
    #[command(flatten)]
    pagination: PaginationArgs,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct ReportRunWaitArgs {
    #[arg(long = "id", alias = "report-run-id")]
    id: String,
    #[arg(long, default_value_t = 5)]
    poll_interval_seconds: u64,
    #[arg(long, default_value_t = 3600)]
    wait_timeout_seconds: u64,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct CreativeGetArgs {
    #[arg(long = "id", alias = "creative-id")]
    id: String,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
#[command(group(clap::ArgGroup::new("preview_target").required(true).multiple(false).args(["creative", "ad"])))]
struct CreativePreviewArgs {
    #[arg(long = "creative", alias = "creative-id")]
    creative: Option<String>,
    #[arg(long = "ad", alias = "ad-id")]
    ad: Option<String>,
    #[arg(long)]
    ad_format: Option<String>,
    #[arg(long)]
    render_type: Option<String>,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct ActivitiesArgs {
    #[arg(long)]
    account: Option<String>,
    #[command(flatten)]
    time_input: TimeInputArgs,
    #[arg(long)]
    category: Option<String>,
    #[arg(long)]
    data_source: Option<String>,
    #[arg(long)]
    oid: Option<String>,
    #[arg(long)]
    business_id: Option<String>,
    #[arg(long)]
    add_children: bool,
    #[command(flatten)]
    pagination: PaginationArgs,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct DatasetGetArgs {
    #[arg(long = "id", alias = "dataset-id")]
    id: String,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct PixelHealthArgs {
    #[arg(long = "pixel", alias = "pixel-id")]
    pixel: String,
    #[arg(long)]
    aggregation: Option<String>,
    #[arg(long)]
    event: Option<String>,
    #[arg(long)]
    event_source: Option<String>,
    #[arg(long)]
    start_time: Option<String>,
    #[arg(long)]
    end_time: Option<String>,
    #[command(flatten)]
    field_input: FieldInputArgs,
}

#[derive(Args, Debug, Clone)]
struct DoctorArgs {
    #[arg(long, help = "Also ping the Meta API to verify the token")]
    api: bool,
}

#[derive(Debug, Clone)]
struct CommandResult {
    envelope: OutputEnvelope,
    exit_code: u8,
}

#[derive(Debug, Clone, Copy)]
struct OutputOptions {
    format: OutputFormat,
    pretty: bool,
    envelope: bool,
    include_meta: bool,
    quiet: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let overrides = ConfigOverrides {
        api_base_url: cli.api_base_url.clone(),
        api_version: cli.api_version.clone(),
        timeout_seconds: cli.timeout_seconds,
        output_format: cli.format.map(Into::into),
        ..ConfigOverrides::default()
    };
    let output_options = OutputOptions {
        format: cli.format.map(Into::into).unwrap_or(OutputFormat::Json),
        pretty: cli.pretty,
        envelope: cli.envelope,
        include_meta: cli.include_meta,
        quiet: cli.quiet,
    };
    let env_file_state = match load_env(cli.env_file.as_deref()) {
        Ok(state) => state,
        Err(error) => return exit_with_error(&error, &output_options),
    };

    init_tracing(cli.verbose, cli.quiet);

    let result = match cli.command {
        Command::Providers { command } => Ok(handle_providers(command)),
        Command::Google => Ok(handle_placeholder_provider("google")),
        Command::Tiktok => Ok(handle_placeholder_provider("tiktok")),
        Command::Meta { command } => match command {
            MetaCommand::Config { command } => {
                let snapshot = inspect(cli.config.as_deref(), &env_file_state, &overrides);
                match snapshot {
                    Ok(snapshot) => handle_config(command, snapshot),
                    Err(error) => Err(error),
                }
            }
            MetaCommand::Doctor(args) => {
                let snapshot = inspect(cli.config.as_deref(), &env_file_state, &overrides);
                match snapshot {
                    Ok(snapshot) => {
                        handle_doctor(
                            args,
                            cli.config.as_deref(),
                            &env_file_state,
                            &overrides,
                            snapshot,
                        )
                        .await
                    }
                    Err(error) => Err(error),
                }
            }
            command => {
                let config = match ResolvedConfig::load(
                    cli.config.as_deref(),
                    &env_file_state,
                    &overrides,
                ) {
                    Ok(config) => config,
                    Err(error) => return exit_with_error(&error, &output_options),
                };
                let client = match GraphClient::from_config(&config) {
                    Ok(client) => client,
                    Err(error) => return exit_with_error(&error, &output_options),
                };
                dispatch_meta_with_client(&client, &config, command).await
            }
        },
    };

    match result {
        Ok(result) => emit_result(result, &output_options, cli.output.as_deref()),
        Err(error) => exit_with_error(&error, &output_options),
    }
}

fn handle_providers(command: ProvidersCommand) -> CommandResult {
    match command {
        ProvidersCommand::List => static_command_result(
            json!([
                {
                    "provider": "meta",
                    "implemented": true,
                    "status": "available",
                    "summary": "Read-only Meta Marketing API support."
                },
                {
                    "provider": "google",
                    "implemented": false,
                    "status": "planned",
                    "summary": "Google Ads namespace is reserved but not implemented yet."
                },
                {
                    "provider": "tiktok",
                    "implemented": false,
                    "status": "planned",
                    "summary": "TikTok Ads namespace is reserved but not implemented yet."
                }
            ]),
            "/providers",
            0,
        ),
    }
}

fn handle_placeholder_provider(provider: &str) -> CommandResult {
    static_command_result(
        json!({
            "provider": provider,
            "implemented": false,
            "message": format!(
                "{provider} commands are not implemented yet. Use `agent-ads providers list` to inspect available providers or `agent-ads meta ...` for the current implementation."
            )
        }),
        &format!("/providers/{provider}"),
        0,
    )
}

fn handle_config(
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

async fn handle_doctor(
    args: DoctorArgs,
    config_path: Option<&Path>,
    env_file_state: &EnvFileState,
    overrides: &ConfigOverrides,
    snapshot: ConfigSnapshot,
) -> Result<CommandResult, MetaAdsError> {
    let mut checks = vec![
        json!({
            "name": "env_file",
            "ok": true,
            "detail": match (
                snapshot.env_file_loaded,
                snapshot.env_file_exists,
                snapshot.env_file_source.as_ref(),
                snapshot.env_file_path.as_ref(),
            ) {
                (true, _, Some(source), Some(path)) => format!(
                    "loaded {} env file from {}",
                    match source {
                        EnvFileSource::Auto => "auto-discovered",
                        EnvFileSource::Explicit => "explicit",
                    },
                    path.display()
                ),
                (false, false, Some(EnvFileSource::Auto), Some(path)) => format!(
                    "no optional .env file found at {}",
                    path.display()
                ),
                (_, _, _, Some(path)) => format!("env file resolved to {}", path.display()),
                _ => "env file discovery not configured".to_string(),
            }
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
            "detail": if snapshot.access_token_present {
                "META_ADS_ACCESS_TOKEN is set"
            } else {
                "META_ADS_ACCESS_TOKEN is missing"
            }
        }),
        json!({
            "name": "app_secret",
            "ok": snapshot.app_secret_present,
            "detail": if snapshot.app_secret_present {
                "META_ADS_APP_SECRET is set"
            } else {
                "META_ADS_APP_SECRET is not set"
            }
        }),
    ];

    let mut ok = snapshot.access_token_present;
    if args.api {
        if snapshot.access_token_present {
            match ResolvedConfig::load(config_path, env_file_state, overrides)
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

async fn dispatch_meta_with_client(
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
        MetaCommand::Doctor(_) | MetaCommand::Config { .. } => {
            unreachable!("handled before auth setup")
        }
    }
}

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
    envelope.paging = response.paging;
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

fn static_command_result(data: Value, endpoint: &str, exit_code: u8) -> CommandResult {
    command_result(data, endpoint, exit_code, None)
}

fn command_result(
    data: Value,
    endpoint: &str,
    exit_code: u8,
    api_version: Option<&str>,
) -> CommandResult {
    CommandResult {
        envelope: OutputEnvelope::new(
            data,
            OutputMeta {
                api_version: api_version.unwrap_or("n/a").to_string(),
                endpoint: endpoint.to_string(),
                object_id: None,
                request_id: None,
                report_run_id: None,
            },
        ),
        exit_code,
    }
}

fn emit_result(
    result: CommandResult,
    options: &OutputOptions,
    output_path: Option<&Path>,
) -> ExitCode {
    if !options.quiet && !options.envelope {
        if let Some(warnings) = &result.envelope.warnings {
            for warning in warnings {
                eprintln!("warning: {warning}");
            }
        }
    }

    let rendered = match render_output(
        &result.envelope,
        options.format,
        RenderOptions {
            pretty: options.pretty,
            envelope: options.envelope,
            include_meta: options.include_meta,
        },
    ) {
        Ok(rendered) => rendered,
        Err(error) => return exit_with_error(&error, options),
    };

    if let Some(path) = output_path {
        if path == Path::new("-") {
            println!("{rendered}");
        } else if let Err(error) = fs::write(path, rendered) {
            return exit_with_error(&MetaAdsError::Io(error), options);
        }
    } else {
        println!("{rendered}");
    }

    ExitCode::from(result.exit_code)
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

fn resolve_fields(args: &FieldInputArgs) -> Result<Vec<String>, MetaAdsError> {
    let mut fields = args.fields.clone();
    if let Some(path) = &args.fields_file {
        let content = read_input(path)?;
        fields.extend(
            content
                .split(|ch| ch == ',' || ch == '\n')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        );
    }
    Ok(fields)
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

fn read_input(path: &Path) -> Result<String, MetaAdsError> {
    if path == Path::new("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

fn extract_report_run_id(data: &Value) -> Option<String> {
    data.get("report_run_id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| data.get("id").and_then(Value::as_str).map(str::to_string))
}

fn init_tracing(verbose: u8, quiet: bool) {
    let default_level = if quiet {
        "error"
    } else {
        match verbose {
            0 => "warn",
            1 => "info",
            _ => "debug",
        }
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{default_level},agent_ads_core=debug")));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .init();
}

fn exit_with_error(error: &MetaAdsError, options: &OutputOptions) -> ExitCode {
    let payload = error_payload(error);
    let rendered = if options.pretty {
        serde_json::to_string_pretty(&payload)
    } else {
        serde_json::to_string(&payload)
    }
    .unwrap_or_else(|_| "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string());
    eprintln!("{rendered}");
    ExitCode::from(error.exit_code() as u8)
}

fn error_payload(error: &MetaAdsError) -> Value {
    match error {
        MetaAdsError::Api(GraphApiError {
            message,
            error_type,
            code,
            error_subcode,
            fbtrace_id,
            status_code,
            ..
        }) => json!({
            "error": {
                "kind": "api",
                "message": message,
                "type": error_type,
                "code": code,
                "error_subcode": error_subcode,
                "fbtrace_id": fbtrace_id,
                "status_code": status_code
            }
        }),
        _ => json!({
            "error": {
                "kind": "internal",
                "message": error.to_string()
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use clap::{Command, CommandFactory, Parser};

    use super::Cli;

    fn render_help(command: &mut Command) -> String {
        let mut buffer = Vec::new();
        command.write_long_help(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    fn nested_help(path: &[&str]) -> String {
        let mut command = Cli::command();
        let mut current = &mut command;
        for segment in path {
            current = current.find_subcommand_mut(segment).unwrap();
        }
        render_help(current)
    }

    #[test]
    fn parses_provider_business_list_command() {
        let cli =
            Cli::try_parse_from(["agent-ads", "meta", "businesses", "list", "--all"]).unwrap();
        let help = format!("{cli:?}");
        assert!(help.contains("Meta"));
        assert!(help.contains("Businesses"));
    }

    #[test]
    fn parses_provider_list_command() {
        let cli = Cli::try_parse_from(["agent-ads", "providers", "list"]).unwrap();
        let help = format!("{cli:?}");
        assert!(help.contains("Providers"));
    }

    #[test]
    fn root_help_lists_provider_topics() {
        let help = render_help(&mut Cli::command());
        assert!(help.contains("--env-file"));
        assert!(help.contains("providers"));
        assert!(help.contains("meta"));
        assert!(help.contains("google"));
        assert!(help.contains("tiktok"));
    }

    #[test]
    fn meta_help_lists_command_topics() {
        let help = nested_help(&["meta"]);
        assert!(help.contains("businesses"));
        assert!(help.contains("insights"));
        assert!(help.contains("report-runs"));
        assert!(help.contains("config"));
    }

    #[test]
    fn meta_insights_help_lists_subcommands() {
        let help = nested_help(&["meta", "insights"]);
        assert!(help.contains("query"));
        assert!(help.contains("export"));
    }

    #[test]
    fn rejects_conflicting_insight_selectors() {
        let result = Cli::try_parse_from([
            "agent-ads",
            "meta",
            "insights",
            "query",
            "--account",
            "123",
            "--object",
            "456",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_colon_delimited_provider_syntax() {
        let result = Cli::try_parse_from(["agent-ads", "meta:insights:query"]);
        assert!(result.is_err());
    }

    #[test]
    fn parses_list_alias() {
        let cli = Cli::try_parse_from(["agent-ads", "meta", "businesses", "ls"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Businesses"));
    }

    #[test]
    fn parses_get_alias() {
        let cli =
            Cli::try_parse_from(["agent-ads", "meta", "creatives", "cat", "--id", "123"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Creatives"));
        assert!(debug.contains("Get"));
    }

    #[test]
    fn rejects_conflicting_time_inputs() {
        let result = Cli::try_parse_from([
            "agent-ads",
            "meta",
            "insights",
            "query",
            "--date-preset",
            "last_7d",
            "--since",
            "2026-03-01",
            "--until",
            "2026-03-10",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn requires_a_preview_target() {
        let result = Cli::try_parse_from(["agent-ads", "meta", "creatives", "preview"]);
        assert!(result.is_err());
    }
}
