mod google;
mod meta;
mod tiktok;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use agent_ads_core::config::{inspect, ConfigOverrides, ResolvedConfig};
use agent_ads_core::error::{GraphApiError, MetaAdsError};
use agent_ads_core::google_config::GoogleConfigOverrides;
use agent_ads_core::google_error::{GoogleApiError, GoogleError};
use agent_ads_core::output::{
    render_output, OutputEnvelope, OutputFormat, OutputMeta, RenderOptions,
};
use agent_ads_core::tiktok_config::TikTokConfigOverrides;
use agent_ads_core::tiktok_error::{TikTokApiError, TikTokError};
use agent_ads_core::{
    google_inspect, GoogleClient, GoogleResolvedConfig, GraphClient, OsKeyringStore, TikTokClient,
    TikTokResolvedConfig,
};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use tracing_subscriber::EnvFilter;

use google::GoogleCommand;
use meta::MetaCommand;
use tiktok::TikTokCommand;

const TIKTOK_REFRESH_TOKEN_ENV_VAR: &str = "TIKTOK_ADS_REFRESH_TOKEN";

// ---------------------------------------------------------------------------
// Shared arg structs (reused across providers)
// ---------------------------------------------------------------------------

#[derive(Args, Debug, Clone, Default)]
pub struct FieldInputArgs {
    #[arg(long, value_delimiter = ',', help = "Comma-separated field names")]
    pub fields: Vec<String>,
    #[arg(long, help = "Read field names from file (- for stdin)")]
    pub fields_file: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------

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
    #[arg(long, global = true, help = "Override API base URL")]
    api_base_url: Option<String>,
    #[arg(
        long,
        global = true,
        help = "Override API version (e.g. Meta v25.0 or Google v23)"
    )]
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
    #[command(about = "Google Ads commands")]
    Google {
        #[command(subcommand)]
        command: GoogleCommand,
    },
    #[command(about = "TikTok Business API commands")]
    Tiktok {
        #[command(subcommand)]
        command: TikTokCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ProvidersCommand {
    #[command(about = "List available and planned providers", visible_alias = "ls")]
    List,
}

// ---------------------------------------------------------------------------
// Shared output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub envelope: OutputEnvelope,
    pub exit_code: u8,
}

#[derive(Debug, Clone, Copy)]
struct OutputOptions {
    format: OutputFormat,
    pretty: bool,
    envelope: bool,
    include_meta: bool,
    quiet: bool,
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

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
    let secret_store = OsKeyringStore;

    init_tracing(cli.verbose, cli.quiet);

    let tiktok_overrides = TikTokConfigOverrides {
        api_base_url: cli.api_base_url.clone(),
        api_version: cli.api_version.clone(),
        timeout_seconds: cli.timeout_seconds,
        output_format: cli.format.map(Into::into),
        ..TikTokConfigOverrides::default()
    };
    let google_overrides = GoogleConfigOverrides {
        api_base_url: cli.api_base_url.clone(),
        api_version: cli.api_version.clone(),
        timeout_seconds: cli.timeout_seconds,
        output_format: cli.format.map(Into::into),
        ..GoogleConfigOverrides::default()
    };

    let result = match cli.command {
        Command::Providers { command } => Ok(handle_providers(command)),
        Command::Google { command } => {
            let google_result = dispatch_google(
                command,
                &secret_store,
                cli.config.as_deref(),
                &google_overrides,
            )
            .await;
            match google_result {
                Ok(result) => Ok(result),
                Err(google_err) => {
                    let payload = google_error_payload(&google_err);
                    let rendered = if output_options.pretty {
                        serde_json::to_string_pretty(&payload)
                    } else {
                        serde_json::to_string(&payload)
                    }
                    .unwrap_or_else(|_| {
                        "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                    });
                    eprintln!("{rendered}");
                    return ExitCode::from(google_err.exit_code() as u8);
                }
            }
        }
        Command::Tiktok { command } => {
            let tiktok_result = dispatch_tiktok(
                command,
                &secret_store,
                cli.config.as_deref(),
                &tiktok_overrides,
            )
            .await;
            match tiktok_result {
                Ok(result) => Ok(result),
                Err(tiktok_err) => {
                    // Convert TikTokError to exit on stderr
                    let payload = tiktok_error_payload(&tiktok_err);
                    let rendered = if output_options.pretty {
                        serde_json::to_string_pretty(&payload)
                    } else {
                        serde_json::to_string(&payload)
                    }
                    .unwrap_or_else(|_| {
                        "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                    });
                    eprintln!("{rendered}");
                    return ExitCode::from(tiktok_err.exit_code() as u8);
                }
            }
        }
        Command::Meta { command } => match command {
            MetaCommand::Auth { command } => meta::handle_auth(command, &secret_store),
            MetaCommand::Config { command } => {
                let snapshot = inspect(cli.config.as_deref(), &secret_store, &overrides);
                match snapshot {
                    Ok(snapshot) => meta::handle_config(command, snapshot),
                    Err(error) => Err(error),
                }
            }
            MetaCommand::Doctor(args) => {
                let snapshot = inspect(cli.config.as_deref(), &secret_store, &overrides);
                match snapshot {
                    Ok(snapshot) => {
                        meta::handle_doctor(
                            args,
                            cli.config.as_deref(),
                            &secret_store,
                            &overrides,
                            snapshot,
                        )
                        .await
                    }
                    Err(error) => Err(error),
                }
            }
            command => {
                let config =
                    match ResolvedConfig::load(cli.config.as_deref(), &secret_store, &overrides) {
                        Ok(config) => config,
                        Err(error) => return exit_with_error(&error, &output_options),
                    };
                let client = match GraphClient::from_config(&config) {
                    Ok(client) => client,
                    Err(error) => return exit_with_error(&error, &output_options),
                };
                meta::dispatch_meta_with_client(&client, &config, command).await
            }
        },
    };

    match result {
        Ok(result) => emit_result(result, &output_options, cli.output.as_deref()),
        Err(error) => exit_with_error(&error, &output_options),
    }
}

// ---------------------------------------------------------------------------
// TikTok dispatch
// ---------------------------------------------------------------------------

async fn dispatch_tiktok(
    command: TikTokCommand,
    secret_store: &dyn agent_ads_core::SecretStore,
    config_path: Option<&Path>,
    overrides: &TikTokConfigOverrides,
) -> Result<CommandResult, TikTokError> {
    match command {
        TikTokCommand::Auth { command } => match command {
            tiktok::AuthCommand::Refresh(ref args) => {
                let snapshot =
                    agent_ads_core::tiktok_inspect(config_path, secret_store, overrides)?;
                let refresh_token = resolve_tiktok_refresh_token(secret_store)?;
                tiktok::handle_auth_refresh(
                    &args.app_id,
                    &args.app_secret,
                    &refresh_token,
                    secret_store,
                    &snapshot,
                )
                .await
            }
            _ => tiktok::handle_auth(command, secret_store),
        },
        TikTokCommand::Config { command } => {
            let snapshot = agent_ads_core::tiktok_inspect(config_path, secret_store, overrides)?;
            tiktok::handle_config(command, snapshot)
        }
        TikTokCommand::Doctor(args) => {
            let snapshot = agent_ads_core::tiktok_inspect(config_path, secret_store, overrides)?;
            tiktok::handle_doctor(args, config_path, secret_store, overrides, snapshot).await
        }
        command => {
            let config = TikTokResolvedConfig::load(config_path, secret_store, overrides)?;
            let client = TikTokClient::from_config(&config)?;
            tiktok::dispatch_tiktok_with_client(&client, &config, command).await
        }
    }
}

async fn dispatch_google(
    command: GoogleCommand,
    secret_store: &dyn agent_ads_core::SecretStore,
    config_path: Option<&Path>,
    overrides: &GoogleConfigOverrides,
) -> Result<CommandResult, GoogleError> {
    match command {
        GoogleCommand::Auth { command } => google::handle_auth(command, secret_store),
        GoogleCommand::Config { command } => {
            let snapshot = google_inspect(config_path, secret_store, overrides)?;
            google::handle_config(command, snapshot)
        }
        GoogleCommand::Doctor(args) => {
            let snapshot = google_inspect(config_path, secret_store, overrides)?;
            google::handle_doctor(args, config_path, secret_store, overrides, snapshot).await
        }
        command => {
            let config = GoogleResolvedConfig::load(config_path, secret_store, overrides)?;
            let client = GoogleClient::from_config(&config).await?;
            google::dispatch_google_with_client(&client, &config, command).await
        }
    }
}

fn tiktok_error_payload(error: &TikTokError) -> Value {
    match error {
        TikTokError::Api(TikTokApiError {
            code,
            message,
            request_id,
        }) => json!({
            "error": {
                "kind": "api",
                "provider": "tiktok",
                "message": message,
                "code": code,
                "request_id": request_id,
            }
        }),
        _ => json!({
            "error": {
                "kind": "internal",
                "provider": "tiktok",
                "message": error.to_string()
            }
        }),
    }
}

fn google_error_payload(error: &GoogleError) -> Value {
    match error {
        GoogleError::Api(GoogleApiError {
            message,
            status,
            code,
            request_id,
            ..
        }) => json!({
            "error": {
                "kind": "api",
                "provider": "google",
                "message": message,
                "status": status,
                "code": code,
                "request_id": request_id,
            }
        }),
        _ => json!({
            "error": {
                "kind": "internal",
                "provider": "google",
                "message": error.to_string()
            }
        }),
    }
}

// ---------------------------------------------------------------------------
// Providers
// ---------------------------------------------------------------------------

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
                    "implemented": true,
                    "status": "available",
                    "summary": "Read-only Google Ads support with native GAQL."
                },
                {
                    "provider": "tiktok",
                    "implemented": true,
                    "status": "available",
                    "summary": "Read-only TikTok Business API support."
                }
            ]),
            "/providers",
            0,
        ),
    }
}

// ---------------------------------------------------------------------------
// Shared helpers (used by provider modules)
// ---------------------------------------------------------------------------

fn static_command_result(data: Value, endpoint: &str, exit_code: u8) -> CommandResult {
    command_result(data, endpoint, exit_code, None)
}

pub fn command_result(
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

pub fn resolve_fields(args: &FieldInputArgs) -> Result<Vec<String>, MetaAdsError> {
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

pub fn read_input(path: &Path) -> Result<String, MetaAdsError> {
    if path == Path::new("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

fn resolve_tiktok_refresh_token(
    secret_store: &dyn agent_ads_core::SecretStore,
) -> Result<String, TikTokError> {
    if let Some(refresh_token) = env::var(TIKTOK_REFRESH_TOKEN_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok(refresh_token);
    }

    match secret_store.get_secret(
        agent_ads_core::TIKTOK_REFRESH_TOKEN_SERVICE,
        agent_ads_core::TIKTOK_REFRESH_TOKEN_ACCOUNT,
    ) {
        Ok(Some(refresh_token)) => Ok(refresh_token),
        Ok(None) => Err(TikTokError::Config(format!(
            "{TIKTOK_REFRESH_TOKEN_ENV_VAR} is missing and no refresh token was found in the OS credential store. Export {TIKTOK_REFRESH_TOKEN_ENV_VAR} or run `agent-ads tiktok auth set --refresh-token` first."
        ))),
        Err(error) => Err(TikTokError::Config(format!(
            "{TIKTOK_REFRESH_TOKEN_ENV_VAR} is missing and the OS credential store could not be read: {error}. Export {TIKTOK_REFRESH_TOKEN_ENV_VAR} or run `agent-ads tiktok auth set --refresh-token` first."
        ))),
    }
}

// ---------------------------------------------------------------------------
// Output and error rendering
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tracing
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::sync::{LazyLock, Mutex};

    use agent_ads_core::secret_store::{SecretStore, SecretStoreError, SecretStoreErrorKind};
    use clap::{Command, CommandFactory, Parser};

    use super::{resolve_tiktok_refresh_token, Cli, TIKTOK_REFRESH_TOKEN_ENV_VAR};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
        get_error: Mutex<Option<SecretStoreError>>,
    }

    impl FakeSecretStore {
        fn with_tiktok_refresh_token(refresh_token: &str) -> Self {
            let store = Self::default();
            store.secrets.lock().unwrap().insert(
                (
                    agent_ads_core::TIKTOK_REFRESH_TOKEN_SERVICE.to_string(),
                    agent_ads_core::TIKTOK_REFRESH_TOKEN_ACCOUNT.to_string(),
                ),
                refresh_token.to_string(),
            );
            store
        }

        fn set_get_error(&self, error: SecretStoreError) {
            *self.get_error.lock().unwrap() = Some(error);
        }
    }

    impl SecretStore for FakeSecretStore {
        fn get_secret(
            &self,
            service: &str,
            account: &str,
        ) -> std::result::Result<Option<String>, SecretStoreError> {
            if let Some(error) = self.get_error.lock().unwrap().clone() {
                return Err(error);
            }

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
        assert!(!help.contains("--env-file"));
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
        assert!(help.contains("auth"));
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
    fn parses_auth_set_command() {
        let cli = Cli::try_parse_from(["agent-ads", "meta", "auth", "set", "--stdin"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Auth"));
        assert!(debug.contains("Set"));
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

    #[test]
    fn doctor_treats_unavailable_credential_store_as_ok_when_shell_env_is_active() {
        use crate::meta::tests::snapshot_with_auth;
        use crate::meta::{credential_store_check_ok, credential_store_detail};
        use agent_ads_core::config::AccessTokenSource;

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
        use crate::meta::tests::snapshot_with_auth;
        use crate::meta::{credential_store_check_ok, credential_store_detail};
        use agent_ads_core::config::AccessTokenSource;

        let snapshot = snapshot_with_auth(
            AccessTokenSource::Missing,
            false,
            Some("secure storage backend is unavailable"),
        );

        assert!(!credential_store_check_ok(&snapshot));
        assert!(credential_store_detail(&snapshot).contains("OS credential store unavailable"));
    }

    // -----------------------------------------------------------------------
    // Google CLI tests
    // -----------------------------------------------------------------------

    #[test]
    fn google_help_lists_command_topics() {
        let help = nested_help(&["google"]);
        assert!(help.contains("customers"));
        assert!(help.contains("campaigns"));
        assert!(help.contains("adgroups"));
        assert!(help.contains("ads"));
        assert!(help.contains("gaql"));
        assert!(help.contains("auth"));
        assert!(help.contains("doctor"));
        assert!(help.contains("config"));
    }

    #[test]
    fn google_gaql_search_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "google",
            "gaql",
            "search",
            "--customer-id",
            "123-456-7890",
            "--query",
            "SELECT campaign.id FROM campaign",
            "--page-size",
            "100",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Google"));
        assert!(debug.contains("Gaql"));
    }

    #[test]
    fn google_auth_set_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "google",
            "auth",
            "set",
            "--developer-token",
            "dev-token",
            "--client-id",
            "client-id",
            "--client-secret",
            "client-secret",
            "--refresh-token",
            "refresh-token",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Google"));
        assert!(debug.contains("Auth"));
        assert!(debug.contains("Set"));
    }

    // -----------------------------------------------------------------------
    // TikTok CLI tests
    // -----------------------------------------------------------------------

    #[test]
    fn tiktok_help_lists_command_topics() {
        let help = nested_help(&["tiktok"]);
        assert!(help.contains("advertisers"));
        assert!(help.contains("campaigns"));
        assert!(help.contains("adgroups"));
        assert!(help.contains("ads"));
        assert!(help.contains("insights"));
        assert!(help.contains("report-runs"));
        assert!(help.contains("creatives"));
        assert!(help.contains("pixels"));
        assert!(help.contains("audiences"));
        assert!(help.contains("auth"));
        assert!(help.contains("doctor"));
        assert!(help.contains("config"));
    }

    #[test]
    fn tiktok_campaigns_list_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "campaigns",
            "list",
            "--advertiser-id",
            "123",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Tiktok"));
        assert!(debug.contains("Campaigns"));
    }

    #[test]
    fn tiktok_campaigns_list_alias() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "campaigns",
            "ls",
            "--advertiser-id",
            "123",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Campaigns"));
    }

    #[test]
    fn tiktok_insights_query_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "insights",
            "query",
            "--advertiser-id",
            "123",
            "--report-type",
            "BASIC",
            "--dimensions",
            "stat_time_day",
            "--metrics",
            "spend,impressions",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Insights"));
    }

    #[test]
    fn tiktok_auth_set_parses() {
        let cli = Cli::try_parse_from(["agent-ads", "tiktok", "auth", "set", "--stdin"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Auth"));
        assert!(debug.contains("Set"));
    }

    #[test]
    fn tiktok_auth_refresh_requires_app_credentials() {
        let result = Cli::try_parse_from(["agent-ads", "tiktok", "auth", "refresh"]);
        assert!(result.is_err());
    }

    #[test]
    fn tiktok_report_runs_submit_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "report-runs",
            "submit",
            "--advertiser-id",
            "123",
            "--report-type",
            "BASIC",
            "--dimensions",
            "stat_time_day",
            "--metrics",
            "spend",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("ReportRuns"));
        assert!(debug.contains("Submit"));
    }

    #[test]
    fn tiktok_report_runs_status_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "report-runs",
            "status",
            "--advertiser-id",
            "123",
            "--task-id",
            "task-456",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Status"));
    }

    #[test]
    fn tiktok_creatives_videos_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "creatives",
            "videos",
            "--advertiser-id",
            "123",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Creatives"));
        assert!(debug.contains("Videos"));
    }

    #[test]
    fn tiktok_pixels_list_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "pixels",
            "list",
            "--advertiser-id",
            "123",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Pixels"));
    }

    #[test]
    fn tiktok_audiences_list_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "audiences",
            "list",
            "--advertiser-id",
            "123",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Audiences"));
    }

    #[test]
    fn tiktok_doctor_parses() {
        let cli = Cli::try_parse_from(["agent-ads", "tiktok", "doctor", "--api"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Doctor"));
    }

    #[test]
    fn tiktok_pagination_args_parse() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "tiktok",
            "campaigns",
            "list",
            "--advertiser-id",
            "123",
            "--page",
            "2",
            "--page-size",
            "50",
            "--all",
            "--max-items",
            "100",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Campaigns"));
    }

    #[test]
    fn tiktok_refresh_token_prefers_shell_env_over_secret_store() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var(TIKTOK_REFRESH_TOKEN_ENV_VAR, "env-refresh-token");
        let store = FakeSecretStore::with_tiktok_refresh_token("stored-refresh-token");

        let refresh_token = resolve_tiktok_refresh_token(&store).unwrap();

        assert_eq!(refresh_token, "env-refresh-token");
        env::remove_var(TIKTOK_REFRESH_TOKEN_ENV_VAR);
    }

    #[test]
    fn tiktok_refresh_token_falls_back_to_secret_store() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var(TIKTOK_REFRESH_TOKEN_ENV_VAR);
        let store = FakeSecretStore::with_tiktok_refresh_token("stored-refresh-token");

        let refresh_token = resolve_tiktok_refresh_token(&store).unwrap();

        assert_eq!(refresh_token, "stored-refresh-token");
    }

    #[test]
    fn tiktok_refresh_token_reports_store_errors_when_env_missing() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var(TIKTOK_REFRESH_TOKEN_ENV_VAR);
        let store = FakeSecretStore::default();
        store.set_get_error(SecretStoreError::new(
            SecretStoreErrorKind::Unavailable,
            "secure storage backend is unavailable".to_string(),
        ));

        let error = resolve_tiktok_refresh_token(&store).unwrap_err();

        assert!(error
            .to_string()
            .contains("secure storage backend is unavailable"));
    }
}
