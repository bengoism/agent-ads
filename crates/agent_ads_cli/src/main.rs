mod google;
mod linkedin;
mod meta;
mod pinterest;
mod tiktok;
mod x;

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use agent_ads_core::config::{inspect, ConfigOverrides, ResolvedConfig};
use agent_ads_core::error::{GraphApiError, MetaAdsError};
use agent_ads_core::google_config::GoogleConfigOverrides;
use agent_ads_core::google_error::{GoogleApiError, GoogleError};
use agent_ads_core::linkedin_config::LinkedInConfigOverrides;
use agent_ads_core::linkedin_error::{LinkedInApiError, LinkedInError};
use agent_ads_core::output::{
    render_output, OutputEnvelope, OutputFormat, OutputMeta, RenderOptions,
};
use agent_ads_core::pinterest_config::PinterestConfigOverrides;
use agent_ads_core::pinterest_error::{PinterestApiError, PinterestError};
use agent_ads_core::tiktok_config::TikTokConfigOverrides;
use agent_ads_core::tiktok_error::{TikTokApiError, TikTokError};
use agent_ads_core::x_config::XConfigOverrides;
use agent_ads_core::x_error::{XApiError, XError};
use agent_ads_core::{
    google_inspect, linkedin_inspect, load_auth_bundle, lock_auth_bundle, pinterest_inspect,
    prepare_auth_bundle_for_update, store_auth_bundle, tiktok_inspect, x_inspect, AuthBundle,
    GoogleAuthBundle, GoogleClient, GoogleResolvedConfig, GraphClient, LinkedInAuthBundle,
    LinkedInClient, LinkedInResolvedConfig, MetaAuthBundle, OsKeyringStore, PinterestAuthBundle,
    PinterestClient, PinterestResolvedConfig, SecretStoreError, SecretStoreErrorKind, TikTokClient,
    TikTokResolvedConfig, XAuthBundle, XClient, XResolvedConfig, AUTH_BUNDLE_ACCOUNT,
    AUTH_BUNDLE_SERVICE,
};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, Select};
use serde_json::{json, Value};
use tracing_subscriber::EnvFilter;

use google::GoogleCommand;
use linkedin::LinkedInCommand;
use meta::MetaCommand;
use pinterest::PinterestCommand;
use tiktok::TikTokCommand;
use x::XCommand;

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
    #[command(about = "Inspect auth status and route into provider setup or deletion")]
    Auth {
        #[command(subcommand)]
        command: Option<RootAuthCommand>,
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
    #[command(about = "Pinterest Ads API commands")]
    Pinterest {
        #[command(subcommand)]
        command: PinterestCommand,
    },
    #[command(about = "LinkedIn Marketing API commands")]
    Linkedin {
        #[command(subcommand)]
        command: LinkedInCommand,
    },
    #[command(about = "X Ads API commands")]
    X {
        #[command(subcommand)]
        command: XCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ProvidersCommand {
    #[command(about = "List available and planned providers", visible_alias = "ls")]
    List,
}

#[derive(Subcommand, Debug)]
enum RootAuthCommand {
    #[command(about = "Show aggregated auth status across implemented providers")]
    Status,
    #[command(about = "Clear stored provider credentials via interactive picker")]
    Clear,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootAuthProvider {
    Meta,
    Google,
    Tiktok,
    Pinterest,
    Linkedin,
    X,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootAuthFlow {
    Setup,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum RootAuthState {
    Configured,
    Partial,
    Missing,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RootCredentialStatus {
    env_var: &'static str,
    credential_store_service: &'static str,
    credential_store_account: &'static str,
    present: bool,
    source: String,
    keychain_present: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RootProviderAuthStatus {
    provider: &'static str,
    status: RootAuthState,
    usable: bool,
    configured_credentials: usize,
    total_credentials: usize,
    credential_store_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    credential_store_error: Option<String>,
    credentials: BTreeMap<String, RootCredentialStatus>,
}

impl RootAuthProvider {
    fn provider_name(self) -> &'static str {
        match self {
            Self::Meta => "meta",
            Self::Google => "google",
            Self::Tiktok => "tiktok",
            Self::Pinterest => "pinterest",
            Self::Linkedin => "linkedin",
            Self::X => "x",
        }
    }
}

impl RootAuthFlow {
    fn provider_prompt(self) -> &'static str {
        match self {
            Self::Setup => "Select provider to configure",
            Self::Clear => "Select provider to clear",
        }
    }

    fn provider_verb(self) -> &'static str {
        match self {
            Self::Setup => "configure",
            Self::Clear => "clear",
        }
    }
}

const ROOT_AUTH_PROVIDER_ORDER: [RootAuthProvider; 6] = [
    RootAuthProvider::Meta,
    RootAuthProvider::Google,
    RootAuthProvider::Tiktok,
    RootAuthProvider::Pinterest,
    RootAuthProvider::Linkedin,
    RootAuthProvider::X,
];

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let Cli {
        config,
        api_base_url,
        api_version,
        timeout_seconds,
        format,
        output,
        pretty,
        envelope,
        include_meta,
        quiet,
        verbose,
        command,
    } = cli;
    let overrides = ConfigOverrides {
        api_base_url: api_base_url.clone(),
        api_version: api_version.clone(),
        timeout_seconds,
        output_format: format.map(Into::into),
        ..ConfigOverrides::default()
    };
    let mut output_options = OutputOptions {
        format: format.map(Into::into).unwrap_or(OutputFormat::Json),
        pretty,
        envelope,
        include_meta,
        quiet,
    };
    let secret_store = OsKeyringStore;
    let explicit_output_shape =
        format.is_some() || pretty || envelope || include_meta || output.is_some();

    init_tracing(verbose, quiet);

    let tiktok_overrides = TikTokConfigOverrides {
        api_base_url: api_base_url.clone(),
        api_version: api_version.clone(),
        timeout_seconds,
        output_format: format.map(Into::into),
        ..TikTokConfigOverrides::default()
    };
    let google_overrides = GoogleConfigOverrides {
        api_base_url: api_base_url.clone(),
        api_version: api_version.clone(),
        timeout_seconds,
        output_format: format.map(Into::into),
        ..GoogleConfigOverrides::default()
    };
    let pinterest_overrides = PinterestConfigOverrides {
        api_base_url: api_base_url.clone(),
        api_version: api_version.clone(),
        timeout_seconds,
        output_format: format.map(Into::into),
        ..PinterestConfigOverrides::default()
    };
    let linkedin_overrides = LinkedInConfigOverrides {
        api_base_url: api_base_url.clone(),
        api_version: api_version.clone(),
        timeout_seconds,
        output_format: format.map(Into::into),
        ..LinkedInConfigOverrides::default()
    };
    let x_overrides = XConfigOverrides {
        api_base_url: api_base_url.clone(),
        api_version: api_version.clone(),
        timeout_seconds,
        output_format: format.map(Into::into),
        ..XConfigOverrides::default()
    };

    let result = match command {
        Command::Providers { command } => Ok(handle_providers(command)),
        Command::Auth { command } => {
            return handle_root_auth_command(
                command,
                &secret_store,
                &output_options,
                explicit_output_shape,
                output.as_deref(),
            );
        }
        Command::Pinterest { command } => {
            if format.is_none() {
                output_options.format = match resolve_pinterest_output_format(
                    config.as_deref(),
                    &secret_store,
                    &pinterest_overrides,
                ) {
                    Ok(format) => format,
                    Err(error) => {
                        let payload = pinterest_error_payload(&error);
                        let rendered = if output_options.pretty {
                            serde_json::to_string_pretty(&payload)
                        } else {
                            serde_json::to_string(&payload)
                        }
                        .unwrap_or_else(|_| {
                            "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                        });
                        eprintln!("{rendered}");
                        return ExitCode::from(error.exit_code() as u8);
                    }
                };
            }
            let pinterest_result = dispatch_pinterest(
                command,
                &secret_store,
                config.as_deref(),
                &pinterest_overrides,
            )
            .await;
            match pinterest_result {
                Ok(result) => Ok(result),
                Err(pinterest_err) => {
                    let payload = pinterest_error_payload(&pinterest_err);
                    let rendered = if output_options.pretty {
                        serde_json::to_string_pretty(&payload)
                    } else {
                        serde_json::to_string(&payload)
                    }
                    .unwrap_or_else(|_| {
                        "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                    });
                    eprintln!("{rendered}");
                    return ExitCode::from(pinterest_err.exit_code() as u8);
                }
            }
        }
        Command::Linkedin { command } => {
            if format.is_none() {
                output_options.format = match resolve_linkedin_output_format(
                    config.as_deref(),
                    &secret_store,
                    &linkedin_overrides,
                ) {
                    Ok(format) => format,
                    Err(error) => {
                        let payload = linkedin_error_payload(&error);
                        let rendered = if output_options.pretty {
                            serde_json::to_string_pretty(&payload)
                        } else {
                            serde_json::to_string(&payload)
                        }
                        .unwrap_or_else(|_| {
                            "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                        });
                        eprintln!("{rendered}");
                        return ExitCode::from(error.exit_code() as u8);
                    }
                };
            }
            let linkedin_result = dispatch_linkedin(
                command,
                &secret_store,
                config.as_deref(),
                &linkedin_overrides,
            )
            .await;
            match linkedin_result {
                Ok(result) => Ok(result),
                Err(linkedin_err) => {
                    let payload = linkedin_error_payload(&linkedin_err);
                    let rendered = if output_options.pretty {
                        serde_json::to_string_pretty(&payload)
                    } else {
                        serde_json::to_string(&payload)
                    }
                    .unwrap_or_else(|_| {
                        "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                    });
                    eprintln!("{rendered}");
                    return ExitCode::from(linkedin_err.exit_code() as u8);
                }
            }
        }
        Command::X { command } => {
            if format.is_none() {
                output_options.format =
                    match resolve_x_output_format(config.as_deref(), &secret_store, &x_overrides) {
                        Ok(format) => format,
                        Err(error) => {
                            let payload = x_error_payload(&error);
                            let rendered = if output_options.pretty {
                                serde_json::to_string_pretty(&payload)
                            } else {
                                serde_json::to_string(&payload)
                            }
                            .unwrap_or_else(|_| {
                                "{\"error\":{\"message\":\"failed to serialize error\"}}"
                                    .to_string()
                            });
                            eprintln!("{rendered}");
                            return ExitCode::from(error.exit_code() as u8);
                        }
                    };
            }
            let x_result =
                dispatch_x(command, &secret_store, config.as_deref(), &x_overrides).await;
            match x_result {
                Ok(result) => Ok(result),
                Err(x_err) => {
                    let payload = x_error_payload(&x_err);
                    let rendered = if output_options.pretty {
                        serde_json::to_string_pretty(&payload)
                    } else {
                        serde_json::to_string(&payload)
                    }
                    .unwrap_or_else(|_| {
                        "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                    });
                    eprintln!("{rendered}");
                    return ExitCode::from(x_err.exit_code() as u8);
                }
            }
        }
        Command::Google { command } => {
            if format.is_none() {
                output_options.format = match resolve_google_output_format(
                    config.as_deref(),
                    &secret_store,
                    &google_overrides,
                ) {
                    Ok(format) => format,
                    Err(error) => {
                        let payload = google_error_payload(&error);
                        let rendered = if output_options.pretty {
                            serde_json::to_string_pretty(&payload)
                        } else {
                            serde_json::to_string(&payload)
                        }
                        .unwrap_or_else(|_| {
                            "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                        });
                        eprintln!("{rendered}");
                        return ExitCode::from(error.exit_code() as u8);
                    }
                };
            }
            let google_result =
                dispatch_google(command, &secret_store, config.as_deref(), &google_overrides).await;
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
            if format.is_none() {
                output_options.format = match resolve_tiktok_output_format(
                    config.as_deref(),
                    &secret_store,
                    &tiktok_overrides,
                ) {
                    Ok(format) => format,
                    Err(error) => {
                        let payload = tiktok_error_payload(&error);
                        let rendered = if output_options.pretty {
                            serde_json::to_string_pretty(&payload)
                        } else {
                            serde_json::to_string(&payload)
                        }
                        .unwrap_or_else(|_| {
                            "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string()
                        });
                        eprintln!("{rendered}");
                        return ExitCode::from(error.exit_code() as u8);
                    }
                };
            }
            let tiktok_result =
                dispatch_tiktok(command, &secret_store, config.as_deref(), &tiktok_overrides).await;
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
        Command::Meta { command } => {
            if format.is_none() {
                output_options.format = match resolve_meta_output_format(
                    config.as_deref(),
                    &secret_store,
                    &overrides,
                ) {
                    Ok(format) => format,
                    Err(error) => return exit_with_error(&error, &output_options),
                };
            }
            match command {
                MetaCommand::Auth { command } => meta::handle_auth(command, &secret_store),
                MetaCommand::Config { command } => {
                    let snapshot = inspect(config.as_deref(), &secret_store, &overrides);
                    match snapshot {
                        Ok(snapshot) => meta::handle_config(command, snapshot),
                        Err(error) => Err(error),
                    }
                }
                MetaCommand::Doctor(args) => {
                    let snapshot = inspect(config.as_deref(), &secret_store, &overrides);
                    match snapshot {
                        Ok(snapshot) => {
                            meta::handle_doctor(
                                args,
                                config.as_deref(),
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
                        match ResolvedConfig::load(config.as_deref(), &secret_store, &overrides) {
                            Ok(config) => config,
                            Err(error) => return exit_with_error(&error, &output_options),
                        };
                    let client = match GraphClient::from_config(&config) {
                        Ok(client) => client,
                        Err(error) => return exit_with_error(&error, &output_options),
                    };
                    meta::dispatch_meta_with_client(&client, &config, command).await
                }
            }
        }
    };

    match result {
        Ok(result) => emit_result(result, &output_options, output.as_deref()),
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
            tiktok::AuthCommand::Refresh(args) => {
                let snapshot =
                    agent_ads_core::tiktok_inspect(config_path, secret_store, overrides)?;
                let refresh_auth = tiktok::resolve_auth_refresh_inputs(&args, secret_store)?;
                tiktok::handle_auth_refresh(
                    &refresh_auth.app_id,
                    &refresh_auth.app_secret,
                    &refresh_auth.refresh_token,
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

async fn dispatch_linkedin(
    command: LinkedInCommand,
    secret_store: &dyn agent_ads_core::SecretStore,
    config_path: Option<&Path>,
    overrides: &LinkedInConfigOverrides,
) -> Result<CommandResult, LinkedInError> {
    match command {
        LinkedInCommand::Auth { command } => linkedin::handle_auth(command, secret_store),
        LinkedInCommand::Config { command } => {
            let snapshot = linkedin_inspect(config_path, secret_store, overrides)?;
            linkedin::handle_config(command, snapshot)
        }
        LinkedInCommand::Doctor(args) => {
            let snapshot = linkedin_inspect(config_path, secret_store, overrides)?;
            linkedin::handle_doctor(config_path, secret_store, overrides, args, snapshot).await
        }
        command => {
            let config = LinkedInResolvedConfig::load(config_path, secret_store, overrides)?;
            let client = LinkedInClient::from_config(&config)?;
            linkedin::dispatch_linkedin_with_client(&client, &config, command).await
        }
    }
}

async fn dispatch_x(
    command: XCommand,
    secret_store: &dyn agent_ads_core::SecretStore,
    config_path: Option<&Path>,
    overrides: &XConfigOverrides,
) -> Result<CommandResult, XError> {
    match command {
        XCommand::Auth { command } => x::handle_auth(command, secret_store),
        XCommand::Config { command } => {
            let snapshot = x_inspect(config_path, secret_store, overrides)?;
            x::handle_config(command, snapshot)
        }
        XCommand::Doctor(args) => {
            let snapshot = x_inspect(config_path, secret_store, overrides)?;
            x::handle_doctor(args, config_path, secret_store, overrides, snapshot).await
        }
        command => {
            let config = XResolvedConfig::load(config_path, secret_store, overrides)?;
            let client = XClient::from_config(&config)?;
            x::dispatch_x_with_client(&client, &config, command).await
        }
    }
}

async fn dispatch_pinterest(
    command: PinterestCommand,
    secret_store: &dyn agent_ads_core::SecretStore,
    config_path: Option<&Path>,
    overrides: &PinterestConfigOverrides,
) -> Result<CommandResult, PinterestError> {
    match command {
        PinterestCommand::Auth { command } => match command {
            pinterest::AuthCommand::Refresh => {
                let snapshot = pinterest_inspect(config_path, secret_store, overrides)?;
                pinterest::handle_auth_refresh(secret_store, &snapshot).await
            }
            _ => pinterest::handle_auth(command, secret_store),
        },
        PinterestCommand::Config { command } => {
            let snapshot = pinterest_inspect(config_path, secret_store, overrides)?;
            pinterest::handle_config(command, snapshot)
        }
        PinterestCommand::Doctor(args) => {
            let snapshot = pinterest_inspect(config_path, secret_store, overrides)?;
            pinterest::handle_doctor(args, config_path, secret_store, overrides, snapshot).await
        }
        command => {
            let config = PinterestResolvedConfig::load(config_path, secret_store, overrides)?;
            let client = PinterestClient::from_config(&config)?;
            pinterest::dispatch_pinterest_with_client(&client, &config, command).await
        }
    }
}

fn resolve_meta_output_format(
    config_path: Option<&Path>,
    secret_store: &dyn agent_ads_core::SecretStore,
    overrides: &ConfigOverrides,
) -> Result<OutputFormat, MetaAdsError> {
    inspect(config_path, secret_store, overrides).map(|snapshot| snapshot.output_format)
}

fn resolve_google_output_format(
    config_path: Option<&Path>,
    secret_store: &dyn agent_ads_core::SecretStore,
    overrides: &GoogleConfigOverrides,
) -> Result<OutputFormat, GoogleError> {
    google_inspect(config_path, secret_store, overrides).map(|snapshot| snapshot.output_format)
}

fn resolve_linkedin_output_format(
    config_path: Option<&Path>,
    secret_store: &dyn agent_ads_core::SecretStore,
    overrides: &LinkedInConfigOverrides,
) -> Result<OutputFormat, LinkedInError> {
    linkedin_inspect(config_path, secret_store, overrides).map(|snapshot| snapshot.output_format)
}

fn resolve_x_output_format(
    config_path: Option<&Path>,
    secret_store: &dyn agent_ads_core::SecretStore,
    overrides: &XConfigOverrides,
) -> Result<OutputFormat, XError> {
    x_inspect(config_path, secret_store, overrides).map(|snapshot| snapshot.output_format)
}

fn resolve_pinterest_output_format(
    config_path: Option<&Path>,
    secret_store: &dyn agent_ads_core::SecretStore,
    overrides: &PinterestConfigOverrides,
) -> Result<OutputFormat, PinterestError> {
    pinterest_inspect(config_path, secret_store, overrides).map(|snapshot| snapshot.output_format)
}

fn resolve_tiktok_output_format(
    config_path: Option<&Path>,
    secret_store: &dyn agent_ads_core::SecretStore,
    overrides: &TikTokConfigOverrides,
) -> Result<OutputFormat, TikTokError> {
    tiktok_inspect(config_path, secret_store, overrides).map(|snapshot| snapshot.output_format)
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

fn linkedin_error_payload(error: &LinkedInError) -> Value {
    match error {
        LinkedInError::Api(LinkedInApiError {
            message,
            service_error_code,
            status,
            request_id,
            ..
        }) => json!({
            "error": {
                "kind": "api",
                "provider": "linkedin",
                "message": message,
                "code": service_error_code,
                "status": status,
                "request_id": request_id,
            }
        }),
        _ => json!({
            "error": {
                "kind": "internal",
                "provider": "linkedin",
                "message": error.to_string()
            }
        }),
    }
}

fn x_error_payload(error: &XError) -> Value {
    match error {
        XError::Api(XApiError {
            message,
            code,
            parameter,
            request_id,
            http_status,
            ..
        }) => json!({
            "error": {
                "kind": "api",
                "provider": "x",
                "message": message,
                "code": code,
                "parameter": parameter,
                "status": http_status,
                "request_id": request_id,
            }
        }),
        _ => json!({
            "error": {
                "kind": "internal",
                "provider": "x",
                "message": error.to_string()
            }
        }),
    }
}

fn pinterest_error_payload(error: &PinterestError) -> Value {
    match error {
        PinterestError::Api(PinterestApiError {
            code,
            message,
            http_status,
            request_id,
        }) => json!({
            "error": {
                "kind": "api",
                "provider": "pinterest",
                "message": message,
                "code": code,
                "status": http_status,
                "request_id": request_id,
            }
        }),
        _ => json!({
            "error": {
                "kind": "internal",
                "provider": "pinterest",
                "message": error.to_string()
            }
        }),
    }
}

fn handle_root_auth_command(
    command: Option<RootAuthCommand>,
    secret_store: &dyn agent_ads_core::SecretStore,
    output_options: &OutputOptions,
    explicit_output_shape: bool,
    output_path: Option<&Path>,
) -> ExitCode {
    match command {
        Some(RootAuthCommand::Status) => emit_result(
            root_auth_status_result(secret_store),
            output_options,
            output_path,
        ),
        Some(RootAuthCommand::Clear) => {
            match validate_root_auth_clear_mode(explicit_output_shape, output_path) {
                Ok(()) => {
                    run_interactive_root_auth(RootAuthFlow::Clear, secret_store, output_options)
                }
                Err(error) => exit_with_error(&error, output_options),
            }
        }
        None if should_run_interactive_root_auth(explicit_output_shape, output_path) => {
            run_interactive_root_auth(RootAuthFlow::Setup, secret_store, output_options)
        }
        None => emit_result(
            root_auth_status_result(secret_store),
            output_options,
            output_path,
        ),
    }
}

fn should_run_interactive_root_auth(
    explicit_output_shape: bool,
    output_path: Option<&Path>,
) -> bool {
    !explicit_output_shape
        && output_path.is_none()
        && io::stdin().is_terminal()
        && io::stdout().is_terminal()
        && io::stderr().is_terminal()
}

fn validate_root_auth_clear_mode(
    explicit_output_shape: bool,
    output_path: Option<&Path>,
) -> Result<(), MetaAdsError> {
    if should_run_interactive_root_auth(explicit_output_shape, output_path) {
        Ok(())
    } else {
        Err(MetaAdsError::Config(
            "`agent-ads auth clear` requires an interactive terminal without output redirection. Run `agent-ads <provider> auth delete` instead.".to_string(),
        ))
    }
}

fn run_interactive_root_auth(
    flow: RootAuthFlow,
    secret_store: &dyn agent_ads_core::SecretStore,
    output_options: &OutputOptions,
) -> ExitCode {
    let _lock = match lock_auth_bundle() {
        Ok(lock) => lock,
        Err(error) => {
            return exit_with_error(&root_auth_bundle_access_error(&error), output_options);
        }
    };
    let bundle_result = load_auth_bundle_with_timeout();
    let summaries = build_root_auth_status_from_bundle_result(bundle_result.clone());
    match select_root_auth_provider(flow, &summaries) {
        Ok(Some(provider)) => run_root_auth_flow(
            flow,
            provider,
            secret_store,
            output_options,
            bundle_result.clone(),
        ),
        Ok(None) => ExitCode::SUCCESS,
        Err(error) => {
            let mut stderr = io::stderr();
            if writeln!(
                &mut stderr,
                "warning: interactive picker unavailable, falling back to numeric selection: {error}"
            )
            .is_err()
            {
                return exit_with_error(
                    &MetaAdsError::Config(
                        "interactive picker failed and the fallback warning could not be written"
                            .to_string(),
                    ),
                    output_options,
                );
            }

            match run_root_auth_numeric_prompt(flow, &summaries) {
                Ok(Some(provider)) => {
                    run_root_auth_flow(flow, provider, secret_store, output_options, bundle_result)
                }
                Ok(None) => ExitCode::SUCCESS,
                Err(error) => exit_with_error(&MetaAdsError::Io(error), output_options),
            }
        }
    }
}

fn select_root_auth_provider(
    flow: RootAuthFlow,
    summaries: &[RootProviderAuthStatus],
) -> Result<Option<RootAuthProvider>, dialoguer::Error> {
    let items = root_auth_menu_items(summaries);
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(flow.provider_prompt())
        .items(&items)
        .default(0)
        .report(false)
        .interact_opt()?;

    Ok(selection.map(|index| ROOT_AUTH_PROVIDER_ORDER[index]))
}

fn run_root_auth_numeric_prompt(
    flow: RootAuthFlow,
    summaries: &[RootProviderAuthStatus],
) -> io::Result<Option<RootAuthProvider>> {
    let mut stderr = io::stderr();
    render_root_auth_summary(summaries, &mut stderr)?;

    loop {
        write!(
            &mut stderr,
            "Select provider to {} [1-{}, q to cancel]: ",
            flow.provider_verb(),
            summaries.len()
        )?;
        stderr.flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input)? {
            0 => return Ok(None),
            _ => {}
        }

        match parse_root_auth_selection(&input, summaries.len()) {
            Ok(Some(index)) => return Ok(Some(ROOT_AUTH_PROVIDER_ORDER[index])),
            Ok(None) => return Ok(None),
            Err(message) => writeln!(&mut stderr, "{message}")?,
        }
    }
}

fn render_root_auth_summary(
    summaries: &[RootProviderAuthStatus],
    writer: &mut impl Write,
) -> io::Result<()> {
    writeln!(writer, "Auth status:")?;
    for (index, summary) in summaries.iter().enumerate() {
        writeln!(
            writer,
            "  [{}] {}",
            index + 1,
            root_auth_menu_label(summary)
        )?;
    }
    Ok(())
}

fn root_auth_menu_items(summaries: &[RootProviderAuthStatus]) -> Vec<String> {
    summaries.iter().map(root_auth_menu_label).collect()
}

fn root_auth_menu_label(summary: &RootProviderAuthStatus) -> String {
    let usability = if summary.usable {
        "usable"
    } else {
        "not usable"
    };

    format!(
        "{:<10} {:<10} {}/{} credentials {}",
        summary.provider,
        root_auth_state_label(summary.status),
        summary.configured_credentials,
        summary.total_credentials,
        usability,
    )
}

fn parse_root_auth_selection(
    input: &str,
    provider_count: usize,
) -> Result<Option<usize>, &'static str> {
    let trimmed = input.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("q")
        || trimmed.eq_ignore_ascii_case("quit")
        || trimmed.eq_ignore_ascii_case("exit")
    {
        return Ok(None);
    }

    match trimmed.parse::<usize>() {
        Ok(selection) if (1..=provider_count).contains(&selection) => Ok(Some(selection - 1)),
        _ => Err("Invalid selection. Enter a provider number or q to cancel."),
    }
}

fn parse_root_auth_confirmation(input: &str) -> Result<Option<bool>, &'static str> {
    let trimmed = input.trim();
    if trimmed == "1"
        || trimmed.eq_ignore_ascii_case("y")
        || trimmed.eq_ignore_ascii_case("yes")
        || trimmed.eq_ignore_ascii_case("clear")
    {
        return Ok(Some(true));
    }

    if trimmed.is_empty()
        || trimmed == "2"
        || trimmed.eq_ignore_ascii_case("n")
        || trimmed.eq_ignore_ascii_case("no")
        || trimmed.eq_ignore_ascii_case("cancel")
        || trimmed.eq_ignore_ascii_case("q")
        || trimmed.eq_ignore_ascii_case("quit")
        || trimmed.eq_ignore_ascii_case("exit")
    {
        return Ok(None);
    }

    Err("Invalid selection. Enter 1 to clear or 2 to cancel.")
}

fn run_root_auth_flow(
    flow: RootAuthFlow,
    provider: RootAuthProvider,
    secret_store: &dyn agent_ads_core::SecretStore,
    output_options: &OutputOptions,
    bundle_result: Result<AuthBundle, SecretStoreError>,
) -> ExitCode {
    match flow {
        RootAuthFlow::Setup => {
            run_root_auth_setup(provider, secret_store, output_options, bundle_result)
        }
        RootAuthFlow::Clear => {
            run_root_auth_clear(provider, secret_store, output_options, bundle_result)
        }
    }
}

fn run_root_auth_setup(
    provider: RootAuthProvider,
    secret_store: &dyn agent_ads_core::SecretStore,
    output_options: &OutputOptions,
    bundle_result: Result<AuthBundle, SecretStoreError>,
) -> ExitCode {
    let (mut bundle, recovered_invalid_bundle) =
        match prepare_root_auth_bundle_update(bundle_result, output_options) {
            Some(result) => result,
            None => return ExitCode::FAILURE,
        };

    match provider {
        RootAuthProvider::Meta => {
            let token = match meta::resolve_auth_token_input(&meta::AuthSetArgs { stdin: false }) {
                Ok(token) => token,
                Err(error) => return exit_with_error(&error, output_options),
            };
            bundle.meta = Some(MetaAuthBundle {
                access_token: Some(token),
            });

            if let Err(error) = persist_root_auth_bundle(provider, secret_store, &bundle) {
                return exit_with_error(&error, output_options);
            }

            emit_result(
                static_command_result(
                    json!({
                        "provider": "meta",
                        "stored": true,
                        "recovered_invalid_bundle": recovered_invalid_bundle,
                        "credential_store_service": AUTH_BUNDLE_SERVICE,
                        "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                    }),
                    "/meta/auth/set",
                    0,
                ),
                output_options,
                None,
            )
        }
        RootAuthProvider::Google => {
            let inputs = match google::resolve_google_auth_inputs(&google::AuthSetArgs {
                stdin: false,
                developer_token: None,
                client_id: None,
                client_secret: None,
                refresh_token: None,
            }) {
                Ok(inputs) => inputs,
                Err(error) => return exit_with_google_error(&error, output_options),
            };
            bundle.google = Some(GoogleAuthBundle {
                developer_token: Some(inputs.developer_token),
                client_id: Some(inputs.client_id),
                client_secret: Some(inputs.client_secret),
                refresh_token: Some(inputs.refresh_token),
            });

            if let Err(error) = persist_root_auth_bundle(provider, secret_store, &bundle) {
                return exit_with_google_error(
                    &GoogleError::Config(error.to_string()),
                    output_options,
                );
            }

            emit_result(
                static_command_result(
                    json!({
                        "provider": "google",
                        "stored": true,
                        "recovered_invalid_bundle": recovered_invalid_bundle,
                        "credentials_stored": [
                            "developer_token",
                            "client_id",
                            "client_secret",
                            "refresh_token"
                        ],
                    }),
                    "/google/auth/set",
                    0,
                ),
                output_options,
                None,
            )
        }
        RootAuthProvider::Tiktok => {
            let inputs = match tiktok::resolve_tiktok_auth_inputs(&tiktok::AuthSetArgs {
                stdin: false,
                refresh_token: false,
                full: true,
            }) {
                Ok(inputs) => inputs,
                Err(error) => return exit_with_tiktok_error(&error, output_options),
            };
            let credentials_stored = root_tiktok_credentials_stored(&inputs);
            let mut tiktok_bundle = bundle.tiktok.take().unwrap_or_default();
            tiktok_bundle.access_token = Some(inputs.access_token);
            if let Some(app_id) = inputs.app_id {
                tiktok_bundle.app_id = Some(app_id);
            }
            if let Some(app_secret) = inputs.app_secret {
                tiktok_bundle.app_secret = Some(app_secret);
            }
            if let Some(refresh_token) = inputs.refresh_token {
                tiktok_bundle.refresh_token = Some(refresh_token);
            }
            bundle.tiktok = Some(tiktok_bundle);

            if let Err(error) = persist_root_auth_bundle(provider, secret_store, &bundle) {
                return exit_with_tiktok_error(
                    &TikTokError::Config(error.to_string()),
                    output_options,
                );
            }

            emit_result(
                static_command_result(
                    json!({
                        "provider": "tiktok",
                        "stored": true,
                        "recovered_invalid_bundle": recovered_invalid_bundle,
                        "credentials_stored": credentials_stored,
                    }),
                    "/tiktok/auth/set",
                    0,
                ),
                output_options,
                None,
            )
        }
        RootAuthProvider::Pinterest => {
            let inputs = match pinterest::resolve_pinterest_auth_inputs(&pinterest::AuthSetArgs {
                stdin: false,
                app_id: None,
                app_secret: None,
                access_token: None,
                refresh_token: None,
            }) {
                Ok(inputs) => inputs,
                Err(error) => return exit_with_pinterest_error(&error, output_options),
            };
            bundle.pinterest = Some(PinterestAuthBundle {
                app_id: Some(inputs.app_id),
                app_secret: Some(inputs.app_secret),
                access_token: Some(inputs.access_token),
                refresh_token: Some(inputs.refresh_token),
            });

            if let Err(error) = persist_root_auth_bundle(provider, secret_store, &bundle) {
                return exit_with_pinterest_error(
                    &PinterestError::Config(error.to_string()),
                    output_options,
                );
            }

            emit_result(
                static_command_result(
                    json!({
                        "provider": "pinterest",
                        "stored": true,
                        "recovered_invalid_bundle": recovered_invalid_bundle,
                        "credentials_stored": [
                            "app_id",
                            "app_secret",
                            "access_token",
                            "refresh_token"
                        ],
                    }),
                    "/pinterest/auth/set",
                    0,
                ),
                output_options,
                None,
            )
        }
        RootAuthProvider::Linkedin => {
            let token = match linkedin::resolve_auth_token_input(&linkedin::AuthSetArgs::default())
            {
                Ok(token) => token,
                Err(error) => return exit_with_linkedin_error(&error, output_options),
            };
            bundle.linkedin = Some(LinkedInAuthBundle {
                access_token: Some(token),
            });

            if let Err(error) = persist_root_auth_bundle(provider, secret_store, &bundle) {
                return exit_with_linkedin_error(
                    &LinkedInError::Config(error.to_string()),
                    output_options,
                );
            }

            emit_result(
                static_command_result(
                    json!({
                        "provider": "linkedin",
                        "stored": true,
                        "recovered_invalid_bundle": recovered_invalid_bundle,
                        "credentials_stored": ["access_token"],
                    }),
                    "/linkedin/auth/set",
                    0,
                ),
                output_options,
                None,
            )
        }
        RootAuthProvider::X => {
            let inputs = match x::resolve_auth_inputs(&x::AuthSetArgs {
                stdin: false,
                consumer_key: None,
                consumer_secret: None,
                access_token: None,
                access_token_secret: None,
            }) {
                Ok(inputs) => inputs,
                Err(error) => return exit_with_x_error(&error, output_options),
            };
            bundle.x = Some(XAuthBundle {
                consumer_key: Some(inputs.consumer_key),
                consumer_secret: Some(inputs.consumer_secret),
                access_token: Some(inputs.access_token),
                access_token_secret: Some(inputs.access_token_secret),
            });

            if let Err(error) = persist_root_auth_bundle(provider, secret_store, &bundle) {
                return exit_with_x_error(&XError::Config(error.to_string()), output_options);
            }

            emit_result(
                static_command_result(
                    json!({
                        "provider": "x",
                        "stored": true,
                        "recovered_invalid_bundle": recovered_invalid_bundle,
                        "credentials_stored": [
                            "consumer_key",
                            "consumer_secret",
                            "access_token",
                            "access_token_secret"
                        ],
                    }),
                    "/x/auth/set",
                    0,
                ),
                output_options,
                None,
            )
        }
    }
}

fn run_root_auth_clear(
    provider: RootAuthProvider,
    secret_store: &dyn agent_ads_core::SecretStore,
    output_options: &OutputOptions,
    bundle_result: Result<AuthBundle, SecretStoreError>,
) -> ExitCode {
    let mut stderr = io::stderr();
    if writeln!(
        &mut stderr,
        "Clears stored {} credentials from the OS credential store only. Shell environment variables will not be changed.",
        provider.provider_name()
    )
    .is_err()
    {
        return exit_with_error(
            &MetaAdsError::Config(
                "interactive confirmation failed and the scope notice could not be written"
                    .to_string(),
            ),
            output_options,
        );
    }

    let (mut bundle, recovered_invalid_bundle) =
        match prepare_root_auth_bundle_update(bundle_result, output_options) {
            Some(result) => result,
            None => return ExitCode::FAILURE,
        };

    match select_root_auth_clear_confirmation(provider) {
        Ok(Some(true)) => run_root_auth_clear_with_bundle(
            provider,
            secret_store,
            output_options,
            &mut bundle,
            recovered_invalid_bundle,
        ),
        Ok(Some(false) | None) => ExitCode::SUCCESS,
        Err(error) => {
            if writeln!(
                &mut stderr,
                "warning: interactive confirmation unavailable, falling back to typed confirmation: {error}"
            )
            .is_err()
            {
                return exit_with_error(
                    &MetaAdsError::Config(
                        "interactive confirmation failed and the fallback warning could not be written"
                            .to_string(),
                    ),
                    output_options,
                );
            }

            match run_root_auth_clear_confirmation_prompt(provider) {
                Ok(Some(true)) => run_root_auth_clear_with_bundle(
                    provider,
                    secret_store,
                    output_options,
                    &mut bundle,
                    recovered_invalid_bundle,
                ),
                Ok(Some(false) | None) => ExitCode::SUCCESS,
                Err(error) => exit_with_error(&MetaAdsError::Io(error), output_options),
            }
        }
    }
}

fn run_root_auth_clear_with_bundle(
    provider: RootAuthProvider,
    secret_store: &dyn agent_ads_core::SecretStore,
    output_options: &OutputOptions,
    bundle: &mut AuthBundle,
    recovered_invalid_bundle: bool,
) -> ExitCode {
    let result = match provider {
        RootAuthProvider::Meta => {
            let deleted = bundle
                .meta
                .take()
                .and_then(|meta| meta.access_token)
                .is_some();
            static_command_result(
                json!({
                    "provider": "meta",
                    "deleted": deleted,
                    "recovered_invalid_bundle": recovered_invalid_bundle,
                    "credential_store_service": AUTH_BUNDLE_SERVICE,
                    "credential_store_account": AUTH_BUNDLE_ACCOUNT,
                }),
                "/meta/auth/delete",
                0,
            )
        }
        RootAuthProvider::Google => {
            let deleted_google = bundle.google.take();
            static_command_result(
                json!({
                    "provider": "google",
                    "developer_token_deleted": deleted_google.as_ref().and_then(|google| google.developer_token.as_ref()).is_some(),
                    "client_id_deleted": deleted_google.as_ref().and_then(|google| google.client_id.as_ref()).is_some(),
                    "client_secret_deleted": deleted_google.as_ref().and_then(|google| google.client_secret.as_ref()).is_some(),
                    "refresh_token_deleted": deleted_google.as_ref().and_then(|google| google.refresh_token.as_ref()).is_some(),
                    "recovered_invalid_bundle": recovered_invalid_bundle,
                }),
                "/google/auth/delete",
                0,
            )
        }
        RootAuthProvider::Tiktok => {
            let deleted_tiktok = bundle.tiktok.take();
            static_command_result(
                json!({
                    "provider": "tiktok",
                    "app_id_deleted": deleted_tiktok.as_ref().and_then(|tiktok| tiktok.app_id.as_ref()).is_some(),
                    "app_secret_deleted": deleted_tiktok.as_ref().and_then(|tiktok| tiktok.app_secret.as_ref()).is_some(),
                    "access_token_deleted": deleted_tiktok.as_ref().and_then(|tiktok| tiktok.access_token.as_ref()).is_some(),
                    "refresh_token_deleted": deleted_tiktok.as_ref().and_then(|tiktok| tiktok.refresh_token.as_ref()).is_some(),
                    "recovered_invalid_bundle": recovered_invalid_bundle,
                }),
                "/tiktok/auth/delete",
                0,
            )
        }
        RootAuthProvider::Pinterest => {
            let deleted_pinterest = bundle.pinterest.take();
            static_command_result(
                json!({
                    "provider": "pinterest",
                    "app_id_deleted": deleted_pinterest.as_ref().and_then(|pinterest| pinterest.app_id.as_ref()).is_some(),
                    "app_secret_deleted": deleted_pinterest.as_ref().and_then(|pinterest| pinterest.app_secret.as_ref()).is_some(),
                    "access_token_deleted": deleted_pinterest.as_ref().and_then(|pinterest| pinterest.access_token.as_ref()).is_some(),
                    "refresh_token_deleted": deleted_pinterest.as_ref().and_then(|pinterest| pinterest.refresh_token.as_ref()).is_some(),
                    "recovered_invalid_bundle": recovered_invalid_bundle,
                }),
                "/pinterest/auth/delete",
                0,
            )
        }
        RootAuthProvider::Linkedin => {
            let deleted = bundle
                .linkedin
                .take()
                .and_then(|linkedin| linkedin.access_token)
                .is_some();
            static_command_result(
                json!({
                    "provider": "linkedin",
                    "access_token_deleted": deleted,
                    "recovered_invalid_bundle": recovered_invalid_bundle,
                }),
                "/linkedin/auth/delete",
                0,
            )
        }
        RootAuthProvider::X => {
            let deleted_x = bundle.x.take();
            static_command_result(
                json!({
                    "provider": "x",
                    "consumer_key_deleted": deleted_x.as_ref().and_then(|x| x.consumer_key.as_ref()).is_some(),
                    "consumer_secret_deleted": deleted_x.as_ref().and_then(|x| x.consumer_secret.as_ref()).is_some(),
                    "access_token_deleted": deleted_x.as_ref().and_then(|x| x.access_token.as_ref()).is_some(),
                    "access_token_secret_deleted": deleted_x.as_ref().and_then(|x| x.access_token_secret.as_ref()).is_some(),
                    "recovered_invalid_bundle": recovered_invalid_bundle,
                }),
                "/x/auth/delete",
                0,
            )
        }
    };

    if let Err(error) = persist_root_auth_bundle(provider, secret_store, bundle) {
        return match provider {
            RootAuthProvider::Meta => exit_with_error(&error, output_options),
            RootAuthProvider::Google => {
                exit_with_google_error(&GoogleError::Config(error.to_string()), output_options)
            }
            RootAuthProvider::Tiktok => {
                exit_with_tiktok_error(&TikTokError::Config(error.to_string()), output_options)
            }
            RootAuthProvider::Pinterest => exit_with_pinterest_error(
                &PinterestError::Config(error.to_string()),
                output_options,
            ),
            RootAuthProvider::Linkedin => {
                exit_with_linkedin_error(&LinkedInError::Config(error.to_string()), output_options)
            }
            RootAuthProvider::X => {
                exit_with_x_error(&XError::Config(error.to_string()), output_options)
            }
        };
    }

    emit_result(result, output_options, None)
}

fn select_root_auth_clear_confirmation(
    provider: RootAuthProvider,
) -> Result<Option<bool>, dialoguer::Error> {
    let items = root_auth_clear_menu_items(provider);
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose action")
        .items(&items)
        .default(1)
        .report(false)
        .interact_opt()
        .map(|selection| match selection {
            Some(0) => Some(true),
            Some(_) | None => None,
        })
}

fn run_root_auth_clear_confirmation_prompt(provider: RootAuthProvider) -> io::Result<Option<bool>> {
    let mut stderr = io::stderr();

    loop {
        writeln!(
            &mut stderr,
            "[1] Clear stored {} credentials",
            provider.provider_name()
        )?;
        writeln!(&mut stderr, "[2] Cancel")?;
        write!(&mut stderr, "Select action [1-2, q to cancel]: ")?;
        stderr.flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input)? {
            0 => return Ok(None),
            _ => {}
        }

        match parse_root_auth_confirmation(&input) {
            Ok(result) => return Ok(result),
            Err(message) => writeln!(&mut stderr, "{message}")?,
        }
    }
}

fn root_auth_clear_menu_items(provider: RootAuthProvider) -> Vec<String> {
    vec![
        format!("Clear stored {} credentials", provider.provider_name()),
        "Cancel".to_string(),
    ]
}

fn prepare_root_auth_bundle_update(
    bundle_result: Result<AuthBundle, SecretStoreError>,
    output_options: &OutputOptions,
) -> Option<(AuthBundle, bool)> {
    match prepare_auth_bundle_for_update(bundle_result) {
        Ok((bundle, outcome)) => Some((bundle, outcome.recovered_invalid_bundle)),
        Err(error) => {
            let _ = exit_with_error(&root_auth_bundle_access_error(&error), output_options);
            None
        }
    }
}

fn persist_root_auth_bundle(
    provider: RootAuthProvider,
    secret_store: &dyn agent_ads_core::SecretStore,
    bundle: &AuthBundle,
) -> Result<(), MetaAdsError> {
    store_auth_bundle(secret_store, bundle).map_err(|error| match provider {
        RootAuthProvider::Meta => meta::auth_storage_error("store", &error),
        RootAuthProvider::Google => MetaAdsError::Config(
            google::google_auth_storage_error("store Google Ads credentials", &error).to_string(),
        ),
        RootAuthProvider::Tiktok => MetaAdsError::Config(
            tiktok::tiktok_auth_storage_error("store TikTok credentials", &error).to_string(),
        ),
        RootAuthProvider::Pinterest => MetaAdsError::Config(
            pinterest::pinterest_auth_storage_error("store Pinterest credentials", &error)
                .to_string(),
        ),
        RootAuthProvider::Linkedin => MetaAdsError::Config(
            linkedin::auth_storage_error("store LinkedIn credentials", &error).to_string(),
        ),
        RootAuthProvider::X => MetaAdsError::Config(
            x::auth_storage_error("store X Ads credentials", &error).to_string(),
        ),
    })
}

fn root_auth_bundle_access_error(error: &SecretStoreError) -> MetaAdsError {
    MetaAdsError::Config(format!(
        "failed to read the auth bundle from the OS credential store: {error}"
    ))
}

fn root_tiktok_credentials_stored(inputs: &tiktok::TikTokAuthInputs) -> Vec<&'static str> {
    let mut credentials_stored = vec!["access_token"];
    if inputs.app_id.is_some() {
        credentials_stored.push("app_id");
    }
    if inputs.app_secret.is_some() {
        credentials_stored.push("app_secret");
    }
    if inputs.refresh_token.is_some() {
        credentials_stored.push("refresh_token");
    }
    credentials_stored
}

#[cfg(test)]
fn run_root_auth_delete_provider(
    provider: RootAuthProvider,
    secret_store: &dyn agent_ads_core::SecretStore,
    output_options: &OutputOptions,
) -> ExitCode {
    match provider {
        RootAuthProvider::Meta => {
            match meta::handle_auth(meta::AuthCommand::Delete, secret_store) {
                Ok(result) => emit_result(result, output_options, None),
                Err(error) => exit_with_error(&error, output_options),
            }
        }
        RootAuthProvider::Google => {
            match google::handle_auth(google::AuthCommand::Delete, secret_store) {
                Ok(result) => emit_result(result, output_options, None),
                Err(error) => exit_with_google_error(&error, output_options),
            }
        }
        RootAuthProvider::Tiktok => {
            match tiktok::handle_auth(tiktok::AuthCommand::Delete, secret_store) {
                Ok(result) => emit_result(result, output_options, None),
                Err(error) => exit_with_tiktok_error(&error, output_options),
            }
        }
        RootAuthProvider::Pinterest => {
            match pinterest::handle_auth(pinterest::AuthCommand::Delete, secret_store) {
                Ok(result) => emit_result(result, output_options, None),
                Err(error) => exit_with_pinterest_error(&error, output_options),
            }
        }
        RootAuthProvider::Linkedin => {
            match linkedin::handle_auth(linkedin::AuthCommand::Delete, secret_store) {
                Ok(result) => emit_result(result, output_options, None),
                Err(error) => exit_with_linkedin_error(&error, output_options),
            }
        }
        RootAuthProvider::X => match x::handle_auth(x::AuthCommand::Delete, secret_store) {
            Ok(result) => emit_result(result, output_options, None),
            Err(error) => exit_with_x_error(&error, output_options),
        },
    }
}

fn root_auth_status_result(_secret_store: &dyn agent_ads_core::SecretStore) -> CommandResult {
    static_command_result(
        json!({
            "providers": build_runtime_root_auth_status()
        }),
        "/auth/status",
        0,
    )
}

#[cfg(test)]
fn build_root_auth_status(
    secret_store: &dyn agent_ads_core::SecretStore,
) -> Vec<RootProviderAuthStatus> {
    build_root_auth_status_from_bundle_result(load_auth_bundle(secret_store))
}

fn build_runtime_root_auth_status() -> Vec<RootProviderAuthStatus> {
    build_root_auth_status_from_bundle_result(load_auth_bundle_with_timeout())
}

fn build_root_auth_status_from_bundle_result(
    bundle_result: Result<AuthBundle, SecretStoreError>,
) -> Vec<RootProviderAuthStatus> {
    let bundle = bundle_result.as_ref().ok();
    let store_error = bundle_result.as_ref().err().cloned();

    vec![
        build_meta_root_auth_status(bundle, store_error.as_ref()),
        build_google_root_auth_status(bundle, store_error.as_ref()),
        build_tiktok_root_auth_status(bundle, store_error.as_ref()),
        build_pinterest_root_auth_status(bundle, store_error.as_ref()),
        build_linkedin_root_auth_status(bundle, store_error.as_ref()),
        build_x_root_auth_status(bundle, store_error.as_ref()),
    ]
}

fn load_auth_bundle_with_timeout() -> Result<AuthBundle, SecretStoreError> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let bundle = load_auth_bundle(&OsKeyringStore);
        let _ = sender.send(bundle);
    });

    match receiver.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(bundle_result) => bundle_result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(SecretStoreError::new(
            SecretStoreErrorKind::Unavailable,
            "timed out while reading the auth bundle from the credential store".to_string(),
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(SecretStoreError::new(
            SecretStoreErrorKind::Failure,
            "failed to read the auth bundle from the credential store".to_string(),
        )),
    }
}

fn build_meta_root_auth_status(
    bundle: Option<&AuthBundle>,
    store_error: Option<&SecretStoreError>,
) -> RootProviderAuthStatus {
    let access_token = root_bundle_credential(
        "META_ADS_ACCESS_TOKEN",
        bundle
            .and_then(|bundle| bundle.meta.as_ref())
            .and_then(|meta| meta.access_token.as_ref())
            .is_some(),
    );

    provider_auth_summary(
        RootAuthProvider::Meta,
        access_token.present,
        credential_store_available(store_error),
        credential_store_error(store_error),
        [("access_token".to_string(), access_token)],
    )
}

fn build_google_root_auth_status(
    bundle: Option<&AuthBundle>,
    store_error: Option<&SecretStoreError>,
) -> RootProviderAuthStatus {
    let developer_token = root_bundle_credential(
        "GOOGLE_ADS_DEVELOPER_TOKEN",
        bundle
            .and_then(|bundle| bundle.google.as_ref())
            .and_then(|google| google.developer_token.as_ref())
            .is_some(),
    );
    let client_id = root_bundle_credential(
        "GOOGLE_ADS_CLIENT_ID",
        bundle
            .and_then(|bundle| bundle.google.as_ref())
            .and_then(|google| google.client_id.as_ref())
            .is_some(),
    );
    let client_secret = root_bundle_credential(
        "GOOGLE_ADS_CLIENT_SECRET",
        bundle
            .and_then(|bundle| bundle.google.as_ref())
            .and_then(|google| google.client_secret.as_ref())
            .is_some(),
    );
    let refresh_token = root_bundle_credential(
        "GOOGLE_ADS_REFRESH_TOKEN",
        bundle
            .and_then(|bundle| bundle.google.as_ref())
            .and_then(|google| google.refresh_token.as_ref())
            .is_some(),
    );
    let usable = developer_token.present
        && client_id.present
        && client_secret.present
        && refresh_token.present;

    provider_auth_summary(
        RootAuthProvider::Google,
        usable,
        credential_store_available(store_error),
        credential_store_error(store_error),
        [
            ("developer_token".to_string(), developer_token),
            ("client_id".to_string(), client_id),
            ("client_secret".to_string(), client_secret),
            ("refresh_token".to_string(), refresh_token),
        ],
    )
}

fn build_tiktok_root_auth_status(
    bundle: Option<&AuthBundle>,
    store_error: Option<&SecretStoreError>,
) -> RootProviderAuthStatus {
    let app_id = root_bundle_credential(
        agent_ads_core::TIKTOK_ADS_APP_ID_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.tiktok.as_ref())
            .and_then(|tiktok| tiktok.app_id.as_ref())
            .is_some(),
    );
    let app_secret = root_bundle_credential(
        agent_ads_core::TIKTOK_ADS_APP_SECRET_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.tiktok.as_ref())
            .and_then(|tiktok| tiktok.app_secret.as_ref())
            .is_some(),
    );
    let access_token = root_bundle_credential(
        agent_ads_core::TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.tiktok.as_ref())
            .and_then(|tiktok| tiktok.access_token.as_ref())
            .is_some(),
    );
    let refresh_token = root_bundle_credential(
        agent_ads_core::TIKTOK_ADS_REFRESH_TOKEN_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.tiktok.as_ref())
            .and_then(|tiktok| tiktok.refresh_token.as_ref())
            .is_some(),
    );

    provider_auth_summary(
        RootAuthProvider::Tiktok,
        access_token.present,
        credential_store_available(store_error),
        credential_store_error(store_error),
        [
            ("app_id".to_string(), app_id),
            ("app_secret".to_string(), app_secret),
            ("access_token".to_string(), access_token),
            ("refresh_token".to_string(), refresh_token),
        ],
    )
}

fn build_pinterest_root_auth_status(
    bundle: Option<&AuthBundle>,
    store_error: Option<&SecretStoreError>,
) -> RootProviderAuthStatus {
    let app_id = root_bundle_credential(
        "PINTEREST_ADS_APP_ID",
        bundle
            .and_then(|bundle| bundle.pinterest.as_ref())
            .and_then(|pinterest| pinterest.app_id.as_ref())
            .is_some(),
    );
    let app_secret = root_bundle_credential(
        "PINTEREST_ADS_APP_SECRET",
        bundle
            .and_then(|bundle| bundle.pinterest.as_ref())
            .and_then(|pinterest| pinterest.app_secret.as_ref())
            .is_some(),
    );
    let access_token = root_bundle_credential(
        "PINTEREST_ADS_ACCESS_TOKEN",
        bundle
            .and_then(|bundle| bundle.pinterest.as_ref())
            .and_then(|pinterest| pinterest.access_token.as_ref())
            .is_some(),
    );
    let refresh_token = root_bundle_credential(
        "PINTEREST_ADS_REFRESH_TOKEN",
        bundle
            .and_then(|bundle| bundle.pinterest.as_ref())
            .and_then(|pinterest| pinterest.refresh_token.as_ref())
            .is_some(),
    );

    provider_auth_summary(
        RootAuthProvider::Pinterest,
        access_token.present,
        credential_store_available(store_error),
        credential_store_error(store_error),
        [
            ("app_id".to_string(), app_id),
            ("app_secret".to_string(), app_secret),
            ("access_token".to_string(), access_token),
            ("refresh_token".to_string(), refresh_token),
        ],
    )
}

fn build_linkedin_root_auth_status(
    bundle: Option<&AuthBundle>,
    store_error: Option<&SecretStoreError>,
) -> RootProviderAuthStatus {
    let access_token = root_bundle_credential(
        agent_ads_core::LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.linkedin.as_ref())
            .and_then(|linkedin| linkedin.access_token.as_ref())
            .is_some(),
    );

    provider_auth_summary(
        RootAuthProvider::Linkedin,
        access_token.present,
        credential_store_available(store_error),
        credential_store_error(store_error),
        [("access_token".to_string(), access_token)],
    )
}

fn build_x_root_auth_status(
    bundle: Option<&AuthBundle>,
    store_error: Option<&SecretStoreError>,
) -> RootProviderAuthStatus {
    let consumer_key = root_bundle_credential(
        agent_ads_core::X_ADS_CONSUMER_KEY_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.x.as_ref())
            .and_then(|x| x.consumer_key.as_ref())
            .is_some(),
    );
    let consumer_secret = root_bundle_credential(
        agent_ads_core::X_ADS_CONSUMER_SECRET_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.x.as_ref())
            .and_then(|x| x.consumer_secret.as_ref())
            .is_some(),
    );
    let access_token = root_bundle_credential(
        agent_ads_core::X_ADS_ACCESS_TOKEN_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.x.as_ref())
            .and_then(|x| x.access_token.as_ref())
            .is_some(),
    );
    let access_token_secret = root_bundle_credential(
        agent_ads_core::X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR,
        bundle
            .and_then(|bundle| bundle.x.as_ref())
            .and_then(|x| x.access_token_secret.as_ref())
            .is_some(),
    );
    let usable = consumer_key.present
        && consumer_secret.present
        && access_token.present
        && access_token_secret.present;

    provider_auth_summary(
        RootAuthProvider::X,
        usable,
        credential_store_available(store_error),
        credential_store_error(store_error),
        [
            ("consumer_key".to_string(), consumer_key),
            ("consumer_secret".to_string(), consumer_secret),
            ("access_token".to_string(), access_token),
            ("access_token_secret".to_string(), access_token_secret),
        ],
    )
}

fn provider_auth_summary(
    provider: RootAuthProvider,
    usable: bool,
    credential_store_available: bool,
    credential_store_error: Option<String>,
    credentials: impl IntoIterator<Item = (String, RootCredentialStatus)>,
) -> RootProviderAuthStatus {
    let credentials: BTreeMap<String, RootCredentialStatus> = credentials.into_iter().collect();
    let configured_credentials = credentials.values().filter(|status| status.present).count();
    let total_credentials = credentials.len();
    let status = if configured_credentials == 0 {
        RootAuthState::Missing
    } else if configured_credentials == total_credentials {
        RootAuthState::Configured
    } else {
        RootAuthState::Partial
    };

    RootProviderAuthStatus {
        provider: provider.provider_name(),
        status,
        usable,
        configured_credentials,
        total_credentials,
        credential_store_available,
        credential_store_error,
        credentials,
    }
}

fn credential_status(
    env_var: &'static str,
    present: bool,
    source: &'static str,
    keychain_present: bool,
) -> RootCredentialStatus {
    RootCredentialStatus {
        env_var,
        credential_store_service: AUTH_BUNDLE_SERVICE,
        credential_store_account: AUTH_BUNDLE_ACCOUNT,
        present,
        source: source.to_string(),
        keychain_present,
    }
}

fn root_bundle_credential(env_var: &'static str, keychain_present: bool) -> RootCredentialStatus {
    let shell_env_present = env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .is_some();

    let source = if shell_env_present {
        "shell_env"
    } else if keychain_present {
        "keychain"
    } else {
        "missing"
    };

    credential_status(
        env_var,
        shell_env_present || keychain_present,
        source,
        keychain_present,
    )
}

fn credential_store_available(store_error: Option<&SecretStoreError>) -> bool {
    store_error
        .map(|error| error.kind() != SecretStoreErrorKind::Unavailable)
        .unwrap_or(true)
}

fn credential_store_error(store_error: Option<&SecretStoreError>) -> Option<String> {
    store_error.map(|error| error.to_string())
}

fn root_auth_state_label(state: RootAuthState) -> &'static str {
    match state {
        RootAuthState::Configured => "configured",
        RootAuthState::Partial => "partial",
        RootAuthState::Missing => "missing",
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
                },
                {
                    "provider": "pinterest",
                    "implemented": true,
                    "status": "available",
                    "summary": "Read-only Pinterest Ads API support."
                },
                {
                    "provider": "linkedin",
                    "implemented": true,
                    "status": "available",
                    "summary": "Read-only LinkedIn Marketing API support."
                },
                {
                    "provider": "x",
                    "implemented": true,
                    "status": "available",
                    "summary": "Read-only X Ads API support."
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
    render_error_payload(payload, options, error.exit_code() as u8)
}

fn exit_with_google_error(error: &GoogleError, options: &OutputOptions) -> ExitCode {
    render_error_payload(
        google_error_payload(error),
        options,
        error.exit_code() as u8,
    )
}

fn exit_with_tiktok_error(error: &TikTokError, options: &OutputOptions) -> ExitCode {
    render_error_payload(
        tiktok_error_payload(error),
        options,
        error.exit_code() as u8,
    )
}

fn exit_with_linkedin_error(error: &LinkedInError, options: &OutputOptions) -> ExitCode {
    render_error_payload(
        linkedin_error_payload(error),
        options,
        error.exit_code() as u8,
    )
}

fn exit_with_x_error(error: &XError, options: &OutputOptions) -> ExitCode {
    render_error_payload(x_error_payload(error), options, error.exit_code() as u8)
}

fn exit_with_pinterest_error(error: &PinterestError, options: &OutputOptions) -> ExitCode {
    render_error_payload(
        pinterest_error_payload(error),
        options,
        error.exit_code() as u8,
    )
}

fn render_error_payload(payload: Value, options: &OutputOptions, exit_code: u8) -> ExitCode {
    let rendered = if options.pretty {
        serde_json::to_string_pretty(&payload)
    } else {
        serde_json::to_string(&payload)
    }
    .unwrap_or_else(|_| "{\"error\":{\"message\":\"failed to serialize error\"}}".to_string());
    eprintln!("{rendered}");
    ExitCode::from(exit_code)
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
    use std::fs;
    use std::sync::{LazyLock, Mutex};

    use agent_ads_core::google_config::GoogleConfigOverrides;
    use agent_ads_core::linkedin_config::LinkedInConfigOverrides;
    use agent_ads_core::output::OutputFormat;
    use agent_ads_core::pinterest_config::PinterestConfigOverrides;
    use agent_ads_core::secret_store::{SecretStore, SecretStoreError};
    use agent_ads_core::{
        load_auth_bundle, store_auth_bundle, AuthBundle, GoogleAuthBundle, MetaAuthBundle,
    };
    use clap::{Command, CommandFactory, Parser};
    use std::path::Path;
    use tempfile::tempdir;

    use super::{
        build_root_auth_status, parse_root_auth_confirmation, parse_root_auth_selection,
        resolve_google_output_format, resolve_linkedin_output_format,
        resolve_pinterest_output_format, root_auth_clear_menu_items, root_auth_menu_items,
        run_root_auth_clear_with_bundle, run_root_auth_delete_provider,
        should_run_interactive_root_auth, validate_root_auth_clear_mode, Cli, OutputOptions,
        RootAuthProvider, RootAuthState,
    };

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
        get_error: Mutex<Option<SecretStoreError>>,
        get_calls: Mutex<usize>,
    }

    impl FakeSecretStore {
        fn get_call_count(&self) -> usize {
            *self.get_calls.lock().unwrap()
        }

        fn reset_get_call_count(&self) {
            *self.get_calls.lock().unwrap() = 0;
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

            *self.get_calls.lock().unwrap() += 1;
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
        assert!(help.contains("auth"));
        assert!(help.contains("providers"));
        assert!(help.contains("meta"));
        assert!(help.contains("google"));
        assert!(help.contains("tiktok"));
        assert!(help.contains("pinterest"));
        assert!(help.contains("linkedin"));
    }

    #[test]
    fn parses_root_auth_command() {
        let cli = Cli::try_parse_from(["agent-ads", "auth"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Auth"));
    }

    #[test]
    fn parses_root_auth_status_command() {
        let cli = Cli::try_parse_from(["agent-ads", "auth", "status"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Status"));
    }

    #[test]
    fn parses_root_auth_clear_command() {
        let cli = Cli::try_parse_from(["agent-ads", "auth", "clear"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Clear"));
    }

    #[test]
    fn root_auth_selection_parses_cancel_and_index() {
        assert_eq!(parse_root_auth_selection("q", 5).unwrap(), None);
        assert_eq!(parse_root_auth_selection("2", 5).unwrap(), Some(1));
        assert!(parse_root_auth_selection("9", 5).is_err());
    }

    #[test]
    fn root_auth_confirmation_parses_clear_and_cancel() {
        assert_eq!(parse_root_auth_confirmation("1").unwrap(), Some(true));
        assert_eq!(parse_root_auth_confirmation("y").unwrap(), Some(true));
        assert_eq!(parse_root_auth_confirmation("clear").unwrap(), Some(true));
        assert_eq!(parse_root_auth_confirmation("2").unwrap(), None);
        assert_eq!(parse_root_auth_confirmation("n").unwrap(), None);
        assert_eq!(parse_root_auth_confirmation("").unwrap(), None);
        assert!(parse_root_auth_confirmation("maybe").is_err());
    }

    #[test]
    fn root_auth_is_not_interactive_when_output_is_shaped() {
        assert!(!should_run_interactive_root_auth(true, None));
        assert!(!should_run_interactive_root_auth(
            false,
            Some(Path::new("out.json"))
        ));
    }

    #[test]
    fn root_auth_clear_requires_interactive_terminal() {
        let error = validate_root_auth_clear_mode(true, None).unwrap_err();
        assert!(error.to_string().contains("agent-ads auth clear"));
        assert!(error.to_string().contains("auth delete"));
    }

    #[test]
    fn root_auth_status_classifies_provider_states() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("META_ADS_ACCESS_TOKEN", "meta-token");
        env::set_var("GOOGLE_ADS_CLIENT_ID", "google-client-id");
        let store = FakeSecretStore::default();

        let summaries = build_root_auth_status(&store);

        assert_eq!(summaries[0].provider, "meta");
        assert_eq!(summaries[0].status, RootAuthState::Configured);
        assert!(summaries[0].usable);
        assert_eq!(summaries[1].provider, "google");
        assert_eq!(summaries[1].status, RootAuthState::Partial);
        assert!(!summaries[1].usable);

        env::remove_var("META_ADS_ACCESS_TOKEN");
        env::remove_var("GOOGLE_ADS_CLIENT_ID");
    }

    #[test]
    fn root_auth_menu_items_include_status_details() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("META_ADS_ACCESS_TOKEN");
        env::remove_var("GOOGLE_ADS_DEVELOPER_TOKEN");
        env::remove_var("GOOGLE_ADS_CLIENT_ID");
        env::remove_var("GOOGLE_ADS_CLIENT_SECRET");
        env::remove_var("GOOGLE_ADS_REFRESH_TOKEN");
        env::remove_var("TIKTOK_ADS_APP_ID");
        env::remove_var("TIKTOK_ADS_APP_SECRET");
        env::remove_var("TIKTOK_ADS_ACCESS_TOKEN");
        env::remove_var("TIKTOK_ADS_REFRESH_TOKEN");
        env::remove_var("PINTEREST_ADS_APP_ID");
        env::remove_var("PINTEREST_ADS_APP_SECRET");
        env::remove_var("PINTEREST_ADS_ACCESS_TOKEN");
        env::remove_var("PINTEREST_ADS_REFRESH_TOKEN");
        env::remove_var("LINKEDIN_ADS_ACCESS_TOKEN");
        env::remove_var("X_ADS_CONSUMER_KEY");
        env::remove_var("X_ADS_CONSUMER_SECRET");
        env::remove_var("X_ADS_ACCESS_TOKEN");
        env::remove_var("X_ADS_ACCESS_TOKEN_SECRET");

        let store = FakeSecretStore::default();
        let summaries = build_root_auth_status(&store);
        let items = root_auth_menu_items(&summaries);

        assert_eq!(items.len(), 6);
        assert!(items[0].contains("meta"));
        assert!(items[0].contains("0/1"));
        assert!(items[1].contains("google"));
        assert!(items[1].contains("0/4"));
        assert!(items[4].contains("linkedin"));
        assert!(items[4].contains("0/1"));
        assert!(items[5].contains("x"));
        assert!(items[5].contains("0/4"));
    }

    #[test]
    fn root_auth_clear_menu_items_offer_clear_and_cancel() {
        let items = root_auth_clear_menu_items(RootAuthProvider::Meta);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "Clear stored meta credentials");
        assert_eq!(items[1], "Cancel");
    }

    #[test]
    fn root_auth_status_reads_bundle_once() {
        let store = FakeSecretStore::default();
        store_auth_bundle(
            &store,
            &AuthBundle {
                meta: Some(MetaAuthBundle {
                    access_token: Some("meta-token".to_string()),
                }),
                ..AuthBundle::default()
            },
        )
        .unwrap();

        let summaries = build_root_auth_status(&store);

        assert_eq!(summaries.len(), 6);
        assert_eq!(store.get_call_count(), 1);
    }

    #[test]
    fn root_auth_clear_with_bundle_does_not_reread_store() {
        let store = FakeSecretStore::default();
        store_auth_bundle(
            &store,
            &AuthBundle {
                google: Some(GoogleAuthBundle {
                    developer_token: Some("developer-token".to_string()),
                    client_id: Some("client-id".to_string()),
                    client_secret: Some("client-secret".to_string()),
                    refresh_token: Some("refresh-token".to_string()),
                }),
                ..AuthBundle::default()
            },
        )
        .unwrap();
        let mut bundle = load_auth_bundle(&store).unwrap();
        store.reset_get_call_count();

        let exit_code = run_root_auth_clear_with_bundle(
            RootAuthProvider::Google,
            &store,
            &OutputOptions {
                format: OutputFormat::Json,
                pretty: false,
                envelope: false,
                include_meta: false,
                quiet: false,
            },
            &mut bundle,
            false,
        );

        assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
        assert_eq!(store.get_call_count(), 0);
    }

    #[test]
    fn root_auth_help_lists_clear_command() {
        let help = nested_help(&["auth"]);
        assert!(help.contains("status"));
        assert!(help.contains("clear"));
    }

    #[test]
    fn root_auth_clear_dispatches_to_selected_provider_delete() {
        let store = FakeSecretStore::default();
        store_auth_bundle(
            &store,
            &AuthBundle {
                meta: Some(MetaAuthBundle {
                    access_token: Some("meta-token".to_string()),
                }),
                google: Some(GoogleAuthBundle {
                    developer_token: Some("developer-token".to_string()),
                    client_id: Some("client-id".to_string()),
                    client_secret: Some("client-secret".to_string()),
                    refresh_token: Some("refresh-token".to_string()),
                }),
                ..AuthBundle::default()
            },
        )
        .unwrap();

        let exit_code = run_root_auth_delete_provider(
            RootAuthProvider::Google,
            &store,
            &OutputOptions {
                format: OutputFormat::Json,
                pretty: false,
                envelope: false,
                include_meta: false,
                quiet: false,
            },
        );

        assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
        let bundle = load_auth_bundle(&store).unwrap();
        assert!(bundle.meta.is_some());
        assert!(bundle.google.is_none());
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
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Google"));
        assert!(debug.contains("Gaql"));
    }

    #[test]
    fn google_page_size_flag_is_rejected() {
        let result = Cli::try_parse_from([
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
        ]);
        assert!(result.is_err());
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
    fn tiktok_auth_refresh_parses_without_inline_app_credentials() {
        let cli = Cli::try_parse_from(["agent-ads", "tiktok", "auth", "refresh"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Refresh"));
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
    fn google_output_format_uses_provider_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("GOOGLE_ADS_OUTPUT_FORMAT");

        let store = FakeSecretStore::default();
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(&path, r#"{"providers":{"google":{"output_format":"csv"}}}"#).unwrap();

        let format =
            resolve_google_output_format(Some(&path), &store, &GoogleConfigOverrides::default())
                .unwrap();

        assert_eq!(format, OutputFormat::Csv);
    }

    // -----------------------------------------------------------------------
    // Pinterest CLI tests
    // -----------------------------------------------------------------------

    #[test]
    fn pinterest_help_lists_command_topics() {
        let help = nested_help(&["pinterest"]);
        assert!(help.contains("ad-accounts"));
        assert!(help.contains("campaigns"));
        assert!(help.contains("adgroups"));
        assert!(help.contains("ads"));
        assert!(help.contains("analytics"));
        assert!(help.contains("report-runs"));
        assert!(help.contains("audiences"));
        assert!(help.contains("targeting-analytics"));
        assert!(help.contains("auth"));
        assert!(help.contains("doctor"));
        assert!(help.contains("config"));
    }

    #[test]
    fn pinterest_analytics_query_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "pinterest",
            "analytics",
            "query",
            "--ad-account-id",
            "123",
            "--level",
            "campaign",
            "--start-date",
            "2026-03-01",
            "--end-date",
            "2026-03-07",
            "--columns",
            "IMPRESSION_1,SPEND_IN_DOLLAR",
            "--granularity",
            "DAY",
            "--campaign-id",
            "456",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Pinterest"));
        assert!(debug.contains("Analytics"));
    }

    #[test]
    fn pinterest_auth_set_parses() {
        let cli =
            Cli::try_parse_from(["agent-ads", "pinterest", "auth", "set", "--stdin"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Auth"));
        assert!(debug.contains("Set"));
    }

    #[test]
    fn pinterest_report_runs_wait_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "pinterest",
            "report-runs",
            "wait",
            "--ad-account-id",
            "123",
            "--token",
            "report-token",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("ReportRuns"));
        assert!(debug.contains("Wait"));
    }

    #[test]
    fn pinterest_targeting_analytics_query_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "pinterest",
            "targeting-analytics",
            "query",
            "--ad-account-id",
            "123",
            "--level",
            "ad_group",
            "--start-date",
            "2026-03-01",
            "--end-date",
            "2026-03-07",
            "--targeting-type",
            "AGE_BUCKET",
            "--columns",
            "SPEND_IN_DOLLAR",
            "--granularity",
            "DAY",
            "--ad-group-id",
            "789",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("TargetingAnalytics"));
    }

    #[test]
    fn pinterest_output_format_uses_provider_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("PINTEREST_ADS_OUTPUT_FORMAT");

        let store = FakeSecretStore::default();
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"providers":{"pinterest":{"output_format":"jsonl"}}}"#,
        )
        .unwrap();

        let format = resolve_pinterest_output_format(
            Some(&path),
            &store,
            &PinterestConfigOverrides::default(),
        )
        .unwrap();

        assert_eq!(format, OutputFormat::Jsonl);
    }

    #[test]
    fn linkedin_help_lists_command_topics() {
        let help = nested_help(&["linkedin"]);
        assert!(help.contains("ad-accounts"));
        assert!(help.contains("campaign-groups"));
        assert!(help.contains("campaigns"));
        assert!(help.contains("creatives"));
        assert!(help.contains("analytics"));
        assert!(help.contains("auth"));
        assert!(help.contains("doctor"));
        assert!(help.contains("config"));
    }

    #[test]
    fn linkedin_analytics_query_parses() {
        let cli = Cli::try_parse_from([
            "agent-ads",
            "linkedin",
            "analytics",
            "query",
            "--finder",
            "statistics",
            "--account-id",
            "123",
            "--pivot",
            "CAMPAIGN",
            "--time-granularity",
            "DAILY",
            "--since",
            "2026-03-01",
            "--until",
            "2026-03-07",
            "--field",
            "impressions",
        ]);

        assert!(
            cli.is_err(),
            "unexpectedly accepted unsupported --field flag"
        );

        let cli = Cli::try_parse_from([
            "agent-ads",
            "linkedin",
            "analytics",
            "query",
            "--finder",
            "statistics",
            "--account-id",
            "123",
            "--pivot",
            "CAMPAIGN",
            "--time-granularity",
            "DAILY",
            "--since",
            "2026-03-01",
            "--until",
            "2026-03-07",
            "--fields",
            "impressions,clicks",
            "--start",
            "20",
            "--page-size",
            "50",
            "--all",
            "--max-items",
            "120",
        ])
        .unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Linkedin"));
        assert!(debug.contains("Analytics"));
    }

    #[test]
    fn linkedin_output_format_uses_provider_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("LINKEDIN_ADS_OUTPUT_FORMAT");

        let store = FakeSecretStore::default();
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"providers":{"linkedin":{"output_format":"csv"}}}"#,
        )
        .unwrap();

        let format = resolve_linkedin_output_format(
            Some(&path),
            &store,
            &LinkedInConfigOverrides::default(),
        )
        .unwrap();

        assert_eq!(format, OutputFormat::Csv);
    }

    #[test]
    fn linkedin_output_format_uses_root_config_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("LINKEDIN_ADS_OUTPUT_FORMAT");

        let store = FakeSecretStore::default();
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"output_format":"csv","providers":{"linkedin":{}}}"#,
        )
        .unwrap();

        let format = resolve_linkedin_output_format(
            Some(&path),
            &store,
            &LinkedInConfigOverrides::default(),
        )
        .unwrap();

        assert_eq!(format, OutputFormat::Csv);
    }
}
