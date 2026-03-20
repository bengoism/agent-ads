use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::auth_bundle::load_auth_bundle;
use crate::config::{load_root_file_config, DEFAULT_CONFIG_FILE};
use crate::output::OutputFormat;
use crate::secret_store::{SecretStore, SecretStoreError, SecretStoreErrorKind};
use crate::tiktok_error::{TikTokError, TikTokResult};

pub const TIKTOK_DEFAULT_API_BASE_URL: &str = "https://business-api.tiktok.com";
pub const TIKTOK_DEFAULT_API_VERSION: &str = "v1.3";
const TIKTOK_DEFAULT_TIMEOUT_SECONDS: u64 = 60;
pub const TIKTOK_ADS_APP_ID_ENV_VAR: &str = "TIKTOK_ADS_APP_ID";
pub const TIKTOK_ADS_APP_SECRET_ENV_VAR: &str = "TIKTOK_ADS_APP_SECRET";
pub const TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR: &str = "TIKTOK_ADS_ACCESS_TOKEN";
pub const TIKTOK_ADS_REFRESH_TOKEN_ENV_VAR: &str = "TIKTOK_ADS_REFRESH_TOKEN";

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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TikTokSecretSource {
    ShellEnv,
    Keychain,
    Missing,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TikTokSecretStatus {
    pub present: bool,
    pub source: TikTokSecretSource,
    pub keychain_present: bool,
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
pub struct TikTokAuthSnapshot {
    pub app_id: TikTokSecretStatus,
    pub app_secret: TikTokSecretStatus,
    pub access_token: TikTokSecretStatus,
    pub refresh_token: TikTokSecretStatus,
    pub credential_store_available: bool,
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

struct TikTokSecretResolution {
    value: Option<String>,
    status: TikTokSecretStatus,
    store_error: Option<SecretStoreError>,
}

struct TikTokAuthResolution {
    app_id: TikTokSecretResolution,
    app_secret: TikTokSecretResolution,
    access_token: TikTokSecretResolution,
    refresh_token: TikTokSecretResolution,
}

impl TikTokAuthResolution {
    fn snapshot(&self) -> TikTokAuthSnapshot {
        let store_error = [
            &self.app_id,
            &self.app_secret,
            &self.access_token,
            &self.refresh_token,
        ]
        .iter()
        .find_map(|resolution| resolution.store_error.as_ref().cloned());

        TikTokAuthSnapshot {
            app_id: self.app_id.status.clone(),
            app_secret: self.app_secret.status.clone(),
            access_token: self.access_token.status.clone(),
            refresh_token: self.refresh_token.status.clone(),
            credential_store_available: store_error
                .as_ref()
                .map(|error| error.kind() != SecretStoreErrorKind::Unavailable)
                .unwrap_or(true),
            credential_store_error: store_error.map(|error| error.to_string()),
        }
    }
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

pub fn tiktok_inspect_auth(secret_store: &dyn SecretStore) -> TikTokAuthSnapshot {
    resolve_tiktok_auth(secret_store).snapshot()
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
    let bundle_result = load_auth_bundle(secret_store);
    let keychain_value = bundle_result
        .as_ref()
        .ok()
        .and_then(|bundle| bundle.tiktok.as_ref())
        .and_then(|tiktok| tiktok.access_token.clone());
    let store_error = bundle_result.err();
    let resolution =
        resolve_tiktok_secret(TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR, keychain_value, store_error);

    TikTokAccessTokenResolution {
        token: resolution.value,
        status: TikTokAccessTokenStatus {
            access_token_present: resolution.status.present,
            access_token_source: match resolution.status.source {
                TikTokSecretSource::ShellEnv => TikTokAccessTokenSource::ShellEnv,
                TikTokSecretSource::Keychain => TikTokAccessTokenSource::Keychain,
                TikTokSecretSource::Missing => TikTokAccessTokenSource::Missing,
            },
            credential_store_available: resolution
                .store_error
                .as_ref()
                .map(|error| error.kind() != SecretStoreErrorKind::Unavailable)
                .unwrap_or(true),
            keychain_token_present: resolution.status.keychain_present,
            credential_store_error: resolution.store_error.map(|error| error.to_string()),
        },
    }
}

fn resolve_tiktok_auth(secret_store: &dyn SecretStore) -> TikTokAuthResolution {
    let bundle_result = load_auth_bundle(secret_store);
    let bundle = bundle_result.as_ref().ok();
    let store_error = bundle_result.as_ref().err().cloned();

    TikTokAuthResolution {
        app_id: resolve_tiktok_secret(
            TIKTOK_ADS_APP_ID_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.tiktok.as_ref())
                .and_then(|tiktok| tiktok.app_id.clone()),
            store_error.clone(),
        ),
        app_secret: resolve_tiktok_secret(
            TIKTOK_ADS_APP_SECRET_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.tiktok.as_ref())
                .and_then(|tiktok| tiktok.app_secret.clone()),
            store_error.clone(),
        ),
        access_token: resolve_tiktok_secret(
            TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.tiktok.as_ref())
                .and_then(|tiktok| tiktok.access_token.clone()),
            store_error.clone(),
        ),
        refresh_token: resolve_tiktok_secret(
            TIKTOK_ADS_REFRESH_TOKEN_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.tiktok.as_ref())
                .and_then(|tiktok| tiktok.refresh_token.clone()),
            store_error,
        ),
    }
}

fn resolve_tiktok_secret(
    env_var: &str,
    keychain_value: Option<String>,
    store_error: Option<SecretStoreError>,
) -> TikTokSecretResolution {
    let shell_value = env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    match (shell_value, keychain_value, store_error) {
        (Some(shell_value), keychain_value, None) => TikTokSecretResolution {
            value: Some(shell_value),
            status: TikTokSecretStatus {
                present: true,
                source: TikTokSecretSource::ShellEnv,
                keychain_present: keychain_value.is_some(),
            },
            store_error: None,
        },
        (Some(shell_value), _, Some(error)) => TikTokSecretResolution {
            value: Some(shell_value),
            status: TikTokSecretStatus {
                present: true,
                source: TikTokSecretSource::ShellEnv,
                keychain_present: false,
            },
            store_error: Some(error),
        },
        (None, Some(keychain_value), None) => TikTokSecretResolution {
            value: Some(keychain_value),
            status: TikTokSecretStatus {
                present: true,
                source: TikTokSecretSource::Keychain,
                keychain_present: true,
            },
            store_error: None,
        },
        (None, None, None) => TikTokSecretResolution {
            value: None,
            status: TikTokSecretStatus {
                present: false,
                source: TikTokSecretSource::Missing,
                keychain_present: false,
            },
            store_error: None,
        },
        (None, None, Some(error)) => TikTokSecretResolution {
            value: None,
            status: TikTokSecretStatus {
                present: false,
                source: TikTokSecretSource::Missing,
                keychain_present: false,
            },
            store_error: Some(error),
        },
        (None, Some(keychain_value), Some(_)) => TikTokSecretResolution {
            value: Some(keychain_value),
            status: TikTokSecretStatus {
                present: true,
                source: TikTokSecretSource::Keychain,
                keychain_present: true,
            },
            store_error: None,
        },
    }
}

fn missing_tiktok_access_token_error(status: &TikTokAccessTokenStatus) -> TikTokError {
    let guidance = tiktok_access_token_guidance();
    match status.credential_store_error.as_deref() {
        Some(detail) => TikTokError::Config(format!(
            "{TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR} is missing and the OS credential store could not be read: {detail}. {guidance}"
        )),
        None => TikTokError::Config(format!(
            "{TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR} is missing. {guidance}"
        )),
    }
}

fn tiktok_access_token_guidance() -> String {
    let mut message = format!(
        "Set {TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR} in the shell for this process or run `agent-ads tiktok auth set` to store it in your OS credential store."
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::sync::{LazyLock, Mutex};

    use super::{
        tiktok_inspect_access_token, tiktok_inspect_auth, TikTokConfigOverrides,
        TikTokResolvedConfig, TikTokSecretSource, TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR,
        TIKTOK_ADS_APP_ID_ENV_VAR,
    };
    use crate::secret_store::{SecretStore, SecretStoreError, SecretStoreErrorKind};
    use crate::{store_auth_bundle, AuthBundle, TikTokAuthBundle};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
        get_error: Mutex<Option<SecretStoreError>>,
    }

    impl FakeSecretStore {
        fn with_access_token(access_token: &str) -> Self {
            let store = Self::default();
            store_auth_bundle(
                &store,
                &AuthBundle {
                    tiktok: Some(TikTokAuthBundle {
                        access_token: Some(access_token.to_string()),
                        ..TikTokAuthBundle::default()
                    }),
                    ..AuthBundle::default()
                },
            )
            .unwrap();
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

    #[test]
    fn env_access_token_overrides_keychain() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var(TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR, "env-token");
        let store = FakeSecretStore::with_access_token("keychain-token");

        let auth =
            TikTokResolvedConfig::load(None, &store, &TikTokConfigOverrides::default()).unwrap();

        assert_eq!(auth.access_token, "env-token");
        env::remove_var(TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR);
    }

    #[test]
    fn missing_access_token_mentions_auth_set() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var(TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR);
        let store = FakeSecretStore::default();

        let error = TikTokResolvedConfig::load(None, &store, &TikTokConfigOverrides::default())
            .unwrap_err();

        assert!(error.to_string().contains("agent-ads tiktok auth set"));
    }

    #[test]
    fn unavailable_secret_store_reports_context() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var(TIKTOK_ADS_ACCESS_TOKEN_ENV_VAR);
        let store = FakeSecretStore::default();
        store.set_get_error(SecretStoreError::new(
            SecretStoreErrorKind::Unavailable,
            "secure storage backend is unavailable".to_string(),
        ));

        let status = tiktok_inspect_access_token(&store);

        assert!(!status.credential_store_available);
        assert!(status
            .credential_store_error
            .as_deref()
            .unwrap()
            .contains("secure storage backend is unavailable"));
    }

    #[test]
    fn auth_snapshot_includes_app_credentials() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var(TIKTOK_ADS_APP_ID_ENV_VAR, "env-app-id");
        let store = FakeSecretStore::default();
        store_auth_bundle(
            &store,
            &AuthBundle {
                tiktok: Some(TikTokAuthBundle {
                    app_id: Some("stored-app-id".to_string()),
                    ..TikTokAuthBundle::default()
                }),
                ..AuthBundle::default()
            },
        )
        .unwrap();

        let snapshot = tiktok_inspect_auth(&store);

        assert!(snapshot.app_id.present);
        assert_eq!(snapshot.app_id.source, TikTokSecretSource::ShellEnv);
        assert!(snapshot.app_id.keychain_present);

        env::remove_var(TIKTOK_ADS_APP_ID_ENV_VAR);
    }
}
