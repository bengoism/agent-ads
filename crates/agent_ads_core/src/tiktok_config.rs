use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{load_root_file_config, DEFAULT_CONFIG_FILE};
use crate::output::OutputFormat;
use crate::secret_store::{
    SecretStore, SecretStoreErrorKind, TIKTOK_ACCESS_TOKEN_ACCOUNT, TIKTOK_ACCESS_TOKEN_SERVICE,
};
use crate::tiktok_error::{TikTokError, TikTokResult};

pub const TIKTOK_DEFAULT_API_BASE_URL: &str = "https://business-api.tiktok.com";
pub const TIKTOK_DEFAULT_API_VERSION: &str = "v1.3";
const TIKTOK_DEFAULT_TIMEOUT_SECONDS: u64 = 60;
const TIKTOK_ACCESS_TOKEN_ENV_VAR: &str = "TIKTOK_ADS_ACCESS_TOKEN";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TikTokFileConfig {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_advertiser_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Default)]
pub struct TikTokConfigOverrides {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_advertiser_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone)]
pub struct TikTokResolvedConfig {
    pub access_token: String,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_advertiser_id: Option<String>,
    pub output_format: OutputFormat,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TikTokAccessTokenSource {
    ShellEnv,
    Keychain,
    Missing,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TikTokAccessTokenStatus {
    pub access_token_present: bool,
    pub access_token_source: TikTokAccessTokenSource,
    pub credential_store_available: bool,
    pub keychain_token_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_store_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TikTokConfigSnapshot {
    pub config_path: PathBuf,
    pub config_file_exists: bool,
    pub access_token_present: bool,
    pub access_token_source: TikTokAccessTokenSource,
    pub credential_store_available: bool,
    pub keychain_token_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_store_error: Option<String>,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_advertiser_id: Option<String>,
    pub output_format: OutputFormat,
}

struct TikTokAccessTokenResolution {
    token: Option<String>,
    status: TikTokAccessTokenStatus,
}

impl TikTokResolvedConfig {
    pub fn load(
        config_path: Option<&Path>,
        secret_store: &dyn SecretStore,
        overrides: &TikTokConfigOverrides,
    ) -> TikTokResult<Self> {
        let token_resolution = resolve_tiktok_access_token(secret_store);
        let snapshot =
            tiktok_inspect_with_status(config_path, &token_resolution.status, overrides)?;
        let access_token = token_resolution
            .token
            .ok_or_else(|| missing_tiktok_access_token_error(&token_resolution.status))?;

        Ok(Self {
            access_token,
            api_base_url: snapshot.api_base_url,
            api_version: snapshot.api_version,
            timeout_seconds: snapshot.timeout_seconds,
            default_advertiser_id: snapshot.default_advertiser_id,
            output_format: snapshot.output_format,
            config_path: snapshot.config_path,
        })
    }
}

pub fn tiktok_inspect_access_token(secret_store: &dyn SecretStore) -> TikTokAccessTokenStatus {
    resolve_tiktok_access_token(secret_store).status
}

pub fn tiktok_inspect(
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &TikTokConfigOverrides,
) -> TikTokResult<TikTokConfigSnapshot> {
    let token_status = tiktok_inspect_access_token(secret_store);
    tiktok_inspect_with_status(config_path, &token_status, overrides)
}

fn tiktok_inspect_with_status(
    config_path: Option<&Path>,
    token_status: &TikTokAccessTokenStatus,
    overrides: &TikTokConfigOverrides,
) -> TikTokResult<TikTokConfigSnapshot> {
    let config_path = config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));
    let file_config = load_tiktok_file_config(&config_path)?;

    let api_base_url = overrides
        .api_base_url
        .clone()
        .or_else(|| env::var("TIKTOK_ADS_API_BASE_URL").ok())
        .or(file_config.api_base_url)
        .unwrap_or_else(|| TIKTOK_DEFAULT_API_BASE_URL.to_string());
    let api_version = overrides
        .api_version
        .clone()
        .or_else(|| env::var("TIKTOK_ADS_API_VERSION").ok())
        .or(file_config.api_version)
        .unwrap_or_else(|| TIKTOK_DEFAULT_API_VERSION.to_string());
    let timeout_seconds = overrides
        .timeout_seconds
        .or_else(|| {
            env::var("TIKTOK_ADS_TIMEOUT_SECONDS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
        })
        .or(file_config.timeout_seconds)
        .unwrap_or(TIKTOK_DEFAULT_TIMEOUT_SECONDS);
    let default_advertiser_id = overrides
        .default_advertiser_id
        .clone()
        .or_else(|| env::var("TIKTOK_ADS_DEFAULT_ADVERTISER_ID").ok())
        .or(file_config.default_advertiser_id);
    let output_format = overrides
        .output_format
        .or_else(|| {
            env::var("TIKTOK_ADS_OUTPUT_FORMAT")
                .ok()
                .and_then(|value| value.parse::<OutputFormat>().ok())
        })
        .or(file_config.output_format)
        .unwrap_or(OutputFormat::Json);

    Ok(TikTokConfigSnapshot {
        config_file_exists: config_path.exists(),
        config_path,
        access_token_present: token_status.access_token_present,
        access_token_source: token_status.access_token_source,
        credential_store_available: token_status.credential_store_available,
        keychain_token_present: token_status.keychain_token_present,
        credential_store_error: token_status.credential_store_error.clone(),
        api_base_url,
        api_version,
        timeout_seconds,
        default_advertiser_id,
        output_format,
    })
}

fn resolve_tiktok_access_token(secret_store: &dyn SecretStore) -> TikTokAccessTokenResolution {
    let shell_token = env::var(TIKTOK_ACCESS_TOKEN_ENV_VAR).ok();
    let keychain_result =
        secret_store.get_secret(TIKTOK_ACCESS_TOKEN_SERVICE, TIKTOK_ACCESS_TOKEN_ACCOUNT);

    match (shell_token, keychain_result) {
        (Some(shell_token), Ok(keychain_token)) => TikTokAccessTokenResolution {
            token: Some(shell_token),
            status: TikTokAccessTokenStatus {
                access_token_present: true,
                access_token_source: TikTokAccessTokenSource::ShellEnv,
                credential_store_available: true,
                keychain_token_present: keychain_token.is_some(),
                credential_store_error: None,
            },
        },
        (Some(shell_token), Err(error)) => TikTokAccessTokenResolution {
            token: Some(shell_token),
            status: TikTokAccessTokenStatus {
                access_token_present: true,
                access_token_source: TikTokAccessTokenSource::ShellEnv,
                credential_store_available: error.kind() != SecretStoreErrorKind::Unavailable,
                keychain_token_present: false,
                credential_store_error: Some(error.to_string()),
            },
        },
        (None, Ok(Some(keychain_token))) => TikTokAccessTokenResolution {
            token: Some(keychain_token),
            status: TikTokAccessTokenStatus {
                access_token_present: true,
                access_token_source: TikTokAccessTokenSource::Keychain,
                credential_store_available: true,
                keychain_token_present: true,
                credential_store_error: None,
            },
        },
        (None, Ok(None)) => TikTokAccessTokenResolution {
            token: None,
            status: TikTokAccessTokenStatus {
                access_token_present: false,
                access_token_source: TikTokAccessTokenSource::Missing,
                credential_store_available: true,
                keychain_token_present: false,
                credential_store_error: None,
            },
        },
        (None, Err(error)) => TikTokAccessTokenResolution {
            token: None,
            status: TikTokAccessTokenStatus {
                access_token_present: false,
                access_token_source: TikTokAccessTokenSource::Missing,
                credential_store_available: error.kind() != SecretStoreErrorKind::Unavailable,
                keychain_token_present: false,
                credential_store_error: Some(error.to_string()),
            },
        },
    }
}

fn missing_tiktok_access_token_error(status: &TikTokAccessTokenStatus) -> TikTokError {
    let guidance = tiktok_access_token_guidance();
    match status.credential_store_error.as_deref() {
        Some(detail) => TikTokError::Config(format!(
            "{TIKTOK_ACCESS_TOKEN_ENV_VAR} is missing and the OS credential store could not be read: {detail}. {guidance}"
        )),
        None => TikTokError::Config(format!(
            "{TIKTOK_ACCESS_TOKEN_ENV_VAR} is missing. {guidance}"
        )),
    }
}

fn tiktok_access_token_guidance() -> String {
    let mut message = format!(
        "Set {TIKTOK_ACCESS_TOKEN_ENV_VAR} in the shell for this process or run `agent-ads tiktok auth set` to store it in your OS credential store."
    );
    if cfg!(target_os = "linux") {
        message.push_str(
            " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet.",
        );
    }
    message
}

fn load_tiktok_file_config(path: &Path) -> TikTokResult<TikTokFileConfig> {
    let root = load_root_file_config(path).map_err(|e| match e {
        crate::error::MetaAdsError::Io(io_err) => TikTokError::Io(io_err),
        crate::error::MetaAdsError::Json(json_err) => TikTokError::Json(json_err),
        other => TikTokError::Config(other.to_string()),
    })?;
    Ok(root.providers.tiktok.unwrap_or_default())
}
