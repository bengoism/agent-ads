use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::auth_bundle::load_auth_bundle;
use crate::error::{MetaAdsError, Result};
use crate::google_config::GoogleFileConfig;
use crate::output::OutputFormat;
use crate::pinterest_config::PinterestFileConfig;
use crate::secret_store::{SecretStore, SecretStoreErrorKind};
use crate::tiktok_config::TikTokFileConfig;

pub const DEFAULT_CONFIG_FILE: &str = "agent-ads.config.json";
pub const DEFAULT_API_BASE_URL: &str = "https://graph.facebook.com";
pub const DEFAULT_API_VERSION: &str = "v25.0";
const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
const ACCESS_TOKEN_ENV_VAR: &str = "META_ADS_ACCESS_TOKEN";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileConfig {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_business_id: Option<String>,
    pub default_account_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderFileConfigs {
    pub meta: Option<FileConfig>,
    #[serde(default)]
    pub google: Option<GoogleFileConfig>,
    #[serde(default)]
    pub pinterest: Option<PinterestFileConfig>,
    #[serde(default)]
    pub tiktok: Option<TikTokFileConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootFileConfig {
    #[serde(flatten)]
    pub legacy_meta: FileConfig,
    #[serde(default)]
    pub providers: ProviderFileConfigs,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigOverrides {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_business_id: Option<String>,
    pub default_account_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub access_token: String,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_business_id: Option<String>,
    pub default_account_id: Option<String>,
    pub output_format: OutputFormat,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessTokenSource {
    ShellEnv,
    Keychain,
    Missing,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AccessTokenStatus {
    pub access_token_present: bool,
    pub access_token_source: AccessTokenSource,
    pub credential_store_available: bool,
    pub keychain_token_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_store_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigSnapshot {
    pub config_path: PathBuf,
    pub config_file_exists: bool,
    pub access_token_present: bool,
    pub access_token_source: AccessTokenSource,
    pub credential_store_available: bool,
    pub keychain_token_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_store_error: Option<String>,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_business_id: Option<String>,
    pub default_account_id: Option<String>,
    pub output_format: OutputFormat,
}

struct AccessTokenResolution {
    token: Option<String>,
    status: AccessTokenStatus,
}

impl ResolvedConfig {
    pub fn load(
        config_path: Option<&Path>,
        secret_store: &dyn SecretStore,
        overrides: &ConfigOverrides,
    ) -> Result<Self> {
        let token_resolution = resolve_access_token(secret_store);
        let snapshot = inspect_with_status(config_path, &token_resolution.status, overrides)?;
        let access_token = token_resolution
            .token
            .ok_or_else(|| missing_access_token_error(&token_resolution.status))?;

        Ok(Self {
            access_token,
            api_base_url: snapshot.api_base_url,
            api_version: snapshot.api_version,
            timeout_seconds: snapshot.timeout_seconds,
            default_business_id: snapshot.default_business_id,
            default_account_id: snapshot.default_account_id,
            output_format: snapshot.output_format,
            config_path: snapshot.config_path,
        })
    }
}

pub fn inspect_access_token(secret_store: &dyn SecretStore) -> AccessTokenStatus {
    resolve_access_token(secret_store).status
}

pub fn inspect(
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &ConfigOverrides,
) -> Result<ConfigSnapshot> {
    let token_status = inspect_access_token(secret_store);
    inspect_with_status(config_path, &token_status, overrides)
}

fn inspect_with_status(
    config_path: Option<&Path>,
    token_status: &AccessTokenStatus,
    overrides: &ConfigOverrides,
) -> Result<ConfigSnapshot> {
    let config_path = config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));
    let file_config = load_file_config(&config_path)?;

    let api_base_url = overrides
        .api_base_url
        .clone()
        .or_else(|| env::var("META_ADS_API_BASE_URL").ok())
        .or(file_config.api_base_url)
        .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string());
    let api_version = overrides
        .api_version
        .clone()
        .or_else(|| env::var("META_ADS_API_VERSION").ok())
        .or(file_config.api_version)
        .unwrap_or_else(|| DEFAULT_API_VERSION.to_string());
    let timeout_seconds = overrides
        .timeout_seconds
        .or_else(|| {
            env::var("META_ADS_TIMEOUT_SECONDS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
        })
        .or(file_config.timeout_seconds)
        .unwrap_or(DEFAULT_TIMEOUT_SECONDS);
    let default_business_id = overrides
        .default_business_id
        .clone()
        .or_else(|| env::var("META_ADS_DEFAULT_BUSINESS_ID").ok())
        .or(file_config.default_business_id);
    let default_account_id = overrides
        .default_account_id
        .clone()
        .or_else(|| env::var("META_ADS_DEFAULT_ACCOUNT_ID").ok())
        .or(file_config.default_account_id);
    let output_format = overrides
        .output_format
        .or_else(|| {
            env::var("META_ADS_OUTPUT_FORMAT")
                .ok()
                .and_then(|value| value.parse::<OutputFormat>().ok())
        })
        .or(file_config.output_format)
        .unwrap_or(OutputFormat::Json);

    Ok(ConfigSnapshot {
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
        default_business_id,
        default_account_id,
        output_format,
    })
}

fn resolve_access_token(secret_store: &dyn SecretStore) -> AccessTokenResolution {
    let shell_token = env::var(ACCESS_TOKEN_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let bundle_result = load_auth_bundle(secret_store);
    let keychain_token = bundle_result
        .as_ref()
        .ok()
        .and_then(|bundle| bundle.meta.as_ref())
        .and_then(|meta| meta.access_token.clone());

    match (shell_token, keychain_token, bundle_result.err()) {
        (Some(shell_token), keychain_token, None) => AccessTokenResolution {
            token: Some(shell_token),
            status: AccessTokenStatus {
                access_token_present: true,
                access_token_source: AccessTokenSource::ShellEnv,
                credential_store_available: true,
                keychain_token_present: keychain_token.is_some(),
                credential_store_error: None,
            },
        },
        (Some(shell_token), _, Some(error)) => AccessTokenResolution {
            token: Some(shell_token),
            status: AccessTokenStatus {
                access_token_present: true,
                access_token_source: AccessTokenSource::ShellEnv,
                credential_store_available: error.kind() != SecretStoreErrorKind::Unavailable,
                keychain_token_present: false,
                credential_store_error: Some(error.to_string()),
            },
        },
        (None, Some(keychain_token), None) => AccessTokenResolution {
            token: Some(keychain_token),
            status: AccessTokenStatus {
                access_token_present: true,
                access_token_source: AccessTokenSource::Keychain,
                credential_store_available: true,
                keychain_token_present: true,
                credential_store_error: None,
            },
        },
        (None, None, None) => AccessTokenResolution {
            token: None,
            status: AccessTokenStatus {
                access_token_present: false,
                access_token_source: AccessTokenSource::Missing,
                credential_store_available: true,
                keychain_token_present: false,
                credential_store_error: None,
            },
        },
        (None, None, Some(error)) => AccessTokenResolution {
            token: None,
            status: AccessTokenStatus {
                access_token_present: false,
                access_token_source: AccessTokenSource::Missing,
                credential_store_available: error.kind() != SecretStoreErrorKind::Unavailable,
                keychain_token_present: false,
                credential_store_error: Some(error.to_string()),
            },
        },
        (None, Some(keychain_token), Some(_)) => AccessTokenResolution {
            token: Some(keychain_token),
            status: AccessTokenStatus {
                access_token_present: true,
                access_token_source: AccessTokenSource::Keychain,
                credential_store_available: true,
                keychain_token_present: true,
                credential_store_error: None,
            },
        },
    }
}

fn missing_access_token_error(status: &AccessTokenStatus) -> MetaAdsError {
    let guidance = access_token_guidance();
    match status.credential_store_error.as_deref() {
        Some(detail) => MetaAdsError::Config(format!(
            "{ACCESS_TOKEN_ENV_VAR} is missing and the OS credential store could not be read: {detail}. {guidance}"
        )),
        None => MetaAdsError::Config(format!(
            "{ACCESS_TOKEN_ENV_VAR} is missing. {guidance}"
        )),
    }
}

fn access_token_guidance() -> String {
    let mut message = format!(
        "Set {ACCESS_TOKEN_ENV_VAR} in the shell for this process or run `agent-ads meta auth set` to store it in your OS credential store."
    );
    if cfg!(target_os = "linux") {
        message.push_str(
            " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet.",
        );
    }
    message
}

/// Load and return the full root file config. Used by provider-specific config
/// modules (e.g. tiktok_config) to read their section from the shared config file.
pub fn load_root_file_config(path: &Path) -> Result<RootFileConfig> {
    if !path.exists() {
        return Ok(RootFileConfig::default());
    }

    let contents = fs::read_to_string(path)?;
    Ok(serde_json::from_str::<RootFileConfig>(&contents)?)
}

fn load_file_config(path: &Path) -> Result<FileConfig> {
    let config = load_root_file_config(path)?;
    Ok(merge_file_config(
        config.legacy_meta,
        config.providers.meta.unwrap_or_default(),
    ))
}

fn merge_file_config(base: FileConfig, overlay: FileConfig) -> FileConfig {
    FileConfig {
        api_base_url: overlay.api_base_url.or(base.api_base_url),
        api_version: overlay.api_version.or(base.api_version),
        timeout_seconds: overlay.timeout_seconds.or(base.timeout_seconds),
        default_business_id: overlay.default_business_id.or(base.default_business_id),
        default_account_id: overlay.default_account_id.or(base.default_account_id),
        output_format: overlay.output_format.or(base.output_format),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::sync::{LazyLock, Mutex};

    use tempfile::tempdir;

    use super::{inspect_access_token, AccessTokenSource, ConfigOverrides, ResolvedConfig};
    use crate::output::OutputFormat;
    use crate::secret_store::{SecretStore, SecretStoreError, SecretStoreErrorKind};
    use crate::{inspect, store_auth_bundle, AuthBundle, MetaAuthBundle};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
        get_error: Mutex<Option<SecretStoreError>>,
        set_error: Mutex<Option<SecretStoreError>>,
        delete_error: Mutex<Option<SecretStoreError>>,
    }

    impl FakeSecretStore {
        fn with_secret(secret: &str) -> Self {
            let store = Self::default();
            store.put_secret(secret);
            store
        }

        fn put_secret(&self, secret: &str) {
            store_auth_bundle(
                self,
                &AuthBundle {
                    meta: Some(MetaAuthBundle {
                        access_token: Some(secret.to_string()),
                    }),
                    ..AuthBundle::default()
                },
            )
            .unwrap();
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
            if let Some(error) = self.set_error.lock().unwrap().clone() {
                return Err(error);
            }

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
            if let Some(error) = self.delete_error.lock().unwrap().clone() {
                return Err(error);
            }

            Ok(self
                .secrets
                .lock()
                .unwrap()
                .remove(&(service.to_string(), account.to_string()))
                .is_some())
        }
    }

    fn clear_meta_env() {
        for key in [
            "META_ADS_ACCESS_TOKEN",
            "META_ADS_API_BASE_URL",
            "META_ADS_API_VERSION",
            "META_ADS_TIMEOUT_SECONDS",
            "META_ADS_DEFAULT_BUSINESS_ID",
            "META_ADS_DEFAULT_ACCOUNT_ID",
            "META_ADS_OUTPUT_FORMAT",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn resolves_precedence() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_meta_env();

        let store = FakeSecretStore::with_secret("keychain-token");
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"output_format":"csv","providers":{"meta":{"api_version":"v24.0","timeout_seconds":10}}}"#,
        )
        .unwrap();

        env::set_var("META_ADS_ACCESS_TOKEN", "shell-token");
        env::set_var("META_ADS_API_VERSION", "v25.0");

        let config = ResolvedConfig::load(
            Some(&path),
            &store,
            &ConfigOverrides {
                timeout_seconds: Some(30),
                ..ConfigOverrides::default()
            },
        )
        .unwrap();

        assert_eq!(config.access_token, "shell-token");
        assert_eq!(config.api_version, "v25.0");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.output_format, OutputFormat::Csv);

        clear_meta_env();
    }

    #[test]
    fn uses_keychain_token_when_shell_env_is_absent() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_meta_env();

        let store = FakeSecretStore::with_secret("keychain-token");
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("agent-ads.config.json");

        fs::write(
            &config_path,
            r#"{"providers":{"meta":{"api_version":"v23.0"}}}"#,
        )
        .unwrap();

        let config =
            ResolvedConfig::load(Some(&config_path), &store, &ConfigOverrides::default()).unwrap();
        let snapshot = inspect(Some(&config_path), &store, &ConfigOverrides::default()).unwrap();

        assert_eq!(config.access_token, "keychain-token");
        assert_eq!(snapshot.access_token_source, AccessTokenSource::Keychain);
        assert!(snapshot.keychain_token_present);

        clear_meta_env();
    }

    #[test]
    fn missing_token_errors_with_setup_guidance() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_meta_env();

        let store = FakeSecretStore::default();
        let error = ResolvedConfig::load(None, &store, &ConfigOverrides::default()).unwrap_err();

        assert!(error.to_string().contains("agent-ads meta auth set"));
    }

    #[test]
    fn inspect_reports_unavailable_store_without_breaking_shell_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_meta_env();

        let store = FakeSecretStore::default();
        store.set_get_error(SecretStoreError::new(
            SecretStoreErrorKind::Unavailable,
            "secure storage backend is unavailable".to_string(),
        ));
        env::set_var("META_ADS_ACCESS_TOKEN", "shell-token");

        let status = inspect_access_token(&store);

        assert!(status.access_token_present);
        assert_eq!(status.access_token_source, AccessTokenSource::ShellEnv);
        assert!(!status.credential_store_available);
        assert_eq!(
            status.credential_store_error.as_deref(),
            Some("secure storage backend is unavailable")
        );

        clear_meta_env();
    }
}
