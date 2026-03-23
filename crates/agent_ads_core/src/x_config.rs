use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::auth_bundle::load_auth_bundle;
use crate::config::{load_root_file_config, DEFAULT_CONFIG_FILE};
use crate::output::OutputFormat;
use crate::secret_store::{SecretStore, SecretStoreError, SecretStoreErrorKind};
use crate::x_error::{XError, XResult};

pub const X_DEFAULT_API_BASE_URL: &str = "https://ads-api.x.com";
pub const X_DEFAULT_API_VERSION: &str = "12";
const X_DEFAULT_TIMEOUT_SECONDS: u64 = 60;

const X_ADS_API_BASE_URL_ENV_VAR: &str = "X_ADS_API_BASE_URL";
const X_ADS_API_VERSION_ENV_VAR: &str = "X_ADS_API_VERSION";
const X_ADS_TIMEOUT_SECONDS_ENV_VAR: &str = "X_ADS_TIMEOUT_SECONDS";
const X_ADS_DEFAULT_ACCOUNT_ID_ENV_VAR: &str = "X_ADS_DEFAULT_ACCOUNT_ID";
const X_ADS_OUTPUT_FORMAT_ENV_VAR: &str = "X_ADS_OUTPUT_FORMAT";
pub const X_ADS_CONSUMER_KEY_ENV_VAR: &str = "X_ADS_CONSUMER_KEY";
pub const X_ADS_CONSUMER_SECRET_ENV_VAR: &str = "X_ADS_CONSUMER_SECRET";
pub const X_ADS_ACCESS_TOKEN_ENV_VAR: &str = "X_ADS_ACCESS_TOKEN";
pub const X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR: &str = "X_ADS_ACCESS_TOKEN_SECRET";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XFileConfig {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_account_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Default)]
pub struct XConfigOverrides {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_account_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone)]
pub struct XResolvedConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_account_id: Option<String>,
    pub output_format: OutputFormat,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum XSecretSource {
    ShellEnv,
    Keychain,
    Missing,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct XSecretStatus {
    pub present: bool,
    pub source: XSecretSource,
    pub keychain_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct XAuthSnapshot {
    pub consumer_key: XSecretStatus,
    pub consumer_secret: XSecretStatus,
    pub access_token: XSecretStatus,
    pub access_token_secret: XSecretStatus,
    pub credential_store_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_store_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct XConfigSnapshot {
    pub config_path: PathBuf,
    pub config_file_exists: bool,
    pub auth: XAuthSnapshot,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_account_id: Option<String>,
    pub output_format: OutputFormat,
}

struct XSecretResolution {
    value: Option<String>,
    status: XSecretStatus,
    store_error: Option<SecretStoreError>,
}

struct XAuthResolution {
    consumer_key: XSecretResolution,
    consumer_secret: XSecretResolution,
    access_token: XSecretResolution,
    access_token_secret: XSecretResolution,
}

impl XAuthResolution {
    fn snapshot(&self) -> XAuthSnapshot {
        let store_error = [
            &self.consumer_key,
            &self.consumer_secret,
            &self.access_token,
            &self.access_token_secret,
        ]
        .iter()
        .find_map(|resolution| resolution.store_error.as_ref().cloned());

        XAuthSnapshot {
            consumer_key: self.consumer_key.status.clone(),
            consumer_secret: self.consumer_secret.status.clone(),
            access_token: self.access_token.status.clone(),
            access_token_secret: self.access_token_secret.status.clone(),
            credential_store_available: store_error
                .as_ref()
                .map(|error| error.kind() != SecretStoreErrorKind::Unavailable)
                .unwrap_or(true),
            credential_store_error: store_error.map(|error| error.to_string()),
        }
    }
}

impl XResolvedConfig {
    pub fn load(
        config_path: Option<&Path>,
        secret_store: &dyn SecretStore,
        overrides: &XConfigOverrides,
    ) -> XResult<Self> {
        let auth_resolution = resolve_x_auth(secret_store);
        let snapshot = x_inspect_with_auth(config_path, &auth_resolution.snapshot(), overrides)?;

        Ok(Self {
            consumer_key: auth_resolution
                .consumer_key
                .value
                .ok_or_else(|| missing_x_credentials_error(&snapshot.auth))?,
            consumer_secret: auth_resolution
                .consumer_secret
                .value
                .ok_or_else(|| missing_x_credentials_error(&snapshot.auth))?,
            access_token: auth_resolution
                .access_token
                .value
                .ok_or_else(|| missing_x_credentials_error(&snapshot.auth))?,
            access_token_secret: auth_resolution
                .access_token_secret
                .value
                .ok_or_else(|| missing_x_credentials_error(&snapshot.auth))?,
            api_base_url: snapshot.api_base_url,
            api_version: snapshot.api_version,
            timeout_seconds: snapshot.timeout_seconds,
            default_account_id: snapshot.default_account_id,
            output_format: snapshot.output_format,
            config_path: snapshot.config_path,
        })
    }
}

pub fn x_inspect_auth(secret_store: &dyn SecretStore) -> XAuthSnapshot {
    resolve_x_auth(secret_store).snapshot()
}

pub fn x_inspect(
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &XConfigOverrides,
) -> XResult<XConfigSnapshot> {
    let auth_snapshot = x_inspect_auth(secret_store);
    x_inspect_with_auth(config_path, &auth_snapshot, overrides)
}

fn x_inspect_with_auth(
    config_path: Option<&Path>,
    auth_snapshot: &XAuthSnapshot,
    overrides: &XConfigOverrides,
) -> XResult<XConfigSnapshot> {
    let config_path = config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));
    let file_config = load_x_file_config(&config_path)?;

    let api_base_url = overrides
        .api_base_url
        .clone()
        .or_else(|| env::var(X_ADS_API_BASE_URL_ENV_VAR).ok())
        .or(file_config.api_base_url)
        .unwrap_or_else(|| X_DEFAULT_API_BASE_URL.to_string());
    let api_version = overrides
        .api_version
        .clone()
        .or_else(|| env::var(X_ADS_API_VERSION_ENV_VAR).ok())
        .or(file_config.api_version)
        .unwrap_or_else(|| X_DEFAULT_API_VERSION.to_string());
    let timeout_seconds = overrides
        .timeout_seconds
        .or_else(|| {
            env::var(X_ADS_TIMEOUT_SECONDS_ENV_VAR)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
        })
        .or(file_config.timeout_seconds)
        .unwrap_or(X_DEFAULT_TIMEOUT_SECONDS);
    let default_account_id = overrides
        .default_account_id
        .clone()
        .or_else(|| env::var(X_ADS_DEFAULT_ACCOUNT_ID_ENV_VAR).ok())
        .or(file_config.default_account_id);
    let output_format = overrides
        .output_format
        .or_else(|| {
            env::var(X_ADS_OUTPUT_FORMAT_ENV_VAR)
                .ok()
                .and_then(|value| value.parse::<OutputFormat>().ok())
        })
        .or(file_config.output_format)
        .unwrap_or(OutputFormat::Json);

    Ok(XConfigSnapshot {
        config_path: config_path.clone(),
        config_file_exists: config_path.exists(),
        auth: auth_snapshot.clone(),
        api_base_url,
        api_version,
        timeout_seconds,
        default_account_id,
        output_format,
    })
}

fn resolve_x_auth(secret_store: &dyn SecretStore) -> XAuthResolution {
    let bundle_result = load_auth_bundle(secret_store);
    let bundle = bundle_result.as_ref().ok();
    let store_error = bundle_result.as_ref().err().cloned();

    XAuthResolution {
        consumer_key: resolve_x_secret(
            X_ADS_CONSUMER_KEY_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.x.as_ref())
                .and_then(|x| x.consumer_key.clone()),
            store_error.clone(),
        ),
        consumer_secret: resolve_x_secret(
            X_ADS_CONSUMER_SECRET_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.x.as_ref())
                .and_then(|x| x.consumer_secret.clone()),
            store_error.clone(),
        ),
        access_token: resolve_x_secret(
            X_ADS_ACCESS_TOKEN_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.x.as_ref())
                .and_then(|x| x.access_token.clone()),
            store_error.clone(),
        ),
        access_token_secret: resolve_x_secret(
            X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR,
            bundle
                .and_then(|bundle| bundle.x.as_ref())
                .and_then(|x| x.access_token_secret.clone()),
            store_error,
        ),
    }
}

fn resolve_x_secret(
    env_var: &str,
    keychain_value: Option<String>,
    store_error: Option<SecretStoreError>,
) -> XSecretResolution {
    let shell_value = env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    match (shell_value, keychain_value, store_error) {
        (Some(shell_value), keychain_value, None) => XSecretResolution {
            value: Some(shell_value),
            status: XSecretStatus {
                present: true,
                source: XSecretSource::ShellEnv,
                keychain_present: keychain_value.is_some(),
            },
            store_error: None,
        },
        (Some(shell_value), _, Some(error)) => XSecretResolution {
            value: Some(shell_value),
            status: XSecretStatus {
                present: true,
                source: XSecretSource::ShellEnv,
                keychain_present: false,
            },
            store_error: Some(error),
        },
        (None, Some(keychain_value), None) => XSecretResolution {
            value: Some(keychain_value),
            status: XSecretStatus {
                present: true,
                source: XSecretSource::Keychain,
                keychain_present: true,
            },
            store_error: None,
        },
        (None, None, None) => XSecretResolution {
            value: None,
            status: XSecretStatus {
                present: false,
                source: XSecretSource::Missing,
                keychain_present: false,
            },
            store_error: None,
        },
        (None, None, Some(error)) => XSecretResolution {
            value: None,
            status: XSecretStatus {
                present: false,
                source: XSecretSource::Missing,
                keychain_present: false,
            },
            store_error: Some(error),
        },
        (None, Some(keychain_value), Some(_)) => XSecretResolution {
            value: Some(keychain_value),
            status: XSecretStatus {
                present: true,
                source: XSecretSource::Keychain,
                keychain_present: true,
            },
            store_error: None,
        },
    }
}

fn missing_x_credentials_error(auth: &XAuthSnapshot) -> XError {
    let missing = [
        (X_ADS_CONSUMER_KEY_ENV_VAR, auth.consumer_key.present),
        (X_ADS_CONSUMER_SECRET_ENV_VAR, auth.consumer_secret.present),
        (X_ADS_ACCESS_TOKEN_ENV_VAR, auth.access_token.present),
        (
            X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR,
            auth.access_token_secret.present,
        ),
    ]
    .iter()
    .filter_map(|(env_var, present)| (!present).then_some(*env_var))
    .collect::<Vec<_>>();

    let mut message = format!(
        "missing X Ads credentials: {}. Set them in the shell for this process or run `agent-ads x auth set` to store them in your OS credential store.",
        missing.join(", ")
    );
    if let Some(error) = auth.credential_store_error.as_deref() {
        message.push_str(&format!(" OS credential store detail: {error}."));
    } else if cfg!(target_os = "linux") {
        message.push_str(
            " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet.",
        );
    }

    XError::Config(message)
}

fn load_x_file_config(path: &Path) -> XResult<XFileConfig> {
    let root = load_root_file_config(path).map_err(|error| match error {
        crate::error::MetaAdsError::Io(io_error) => XError::Io(io_error),
        crate::error::MetaAdsError::Json(json_error) => XError::Json(json_error),
        other => XError::Config(other.to_string()),
    })?;
    let mut config = root.providers.x.unwrap_or_default();
    if config.output_format.is_none() {
        config.output_format = root.legacy_meta.output_format;
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::sync::{LazyLock, Mutex};

    use tempfile::tempdir;

    use super::{
        x_inspect, x_inspect_auth, XConfigOverrides, XResolvedConfig, XSecretSource,
        X_ADS_ACCESS_TOKEN_ENV_VAR, X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR, X_ADS_CONSUMER_KEY_ENV_VAR,
        X_ADS_CONSUMER_SECRET_ENV_VAR,
    };
    use crate::output::OutputFormat;
    use crate::secret_store::{SecretStore, SecretStoreError};
    use crate::{store_auth_bundle, AuthBundle, XAuthBundle};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
        get_error: Mutex<Option<SecretStoreError>>,
    }

    impl FakeSecretStore {
        fn with_x_secrets() -> Self {
            let store = Self::default();
            store.put_x_secrets();
            store
        }

        fn put_x_secrets(&self) {
            store_auth_bundle(
                self,
                &AuthBundle {
                    x: Some(XAuthBundle {
                        consumer_key: Some("consumer-key".to_string()),
                        consumer_secret: Some("consumer-secret".to_string()),
                        access_token: Some("access-token".to_string()),
                        access_token_secret: Some("access-token-secret".to_string()),
                    }),
                    ..AuthBundle::default()
                },
            )
            .unwrap();
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

    fn clear_x_env() {
        for key in [
            X_ADS_CONSUMER_KEY_ENV_VAR,
            X_ADS_CONSUMER_SECRET_ENV_VAR,
            X_ADS_ACCESS_TOKEN_ENV_VAR,
            X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR,
            "X_ADS_API_BASE_URL",
            "X_ADS_API_VERSION",
            "X_ADS_TIMEOUT_SECONDS",
            "X_ADS_DEFAULT_ACCOUNT_ID",
            "X_ADS_OUTPUT_FORMAT",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn resolves_precedence() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_x_env();

        let store = FakeSecretStore::with_x_secrets();
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"output_format":"csv","providers":{"x":{"api_version":"11","timeout_seconds":10}}}"#,
        )
        .unwrap();

        env::set_var(X_ADS_CONSUMER_KEY_ENV_VAR, "shell-consumer-key");
        env::set_var(X_ADS_CONSUMER_SECRET_ENV_VAR, "shell-consumer-secret");
        env::set_var(X_ADS_ACCESS_TOKEN_ENV_VAR, "shell-access-token");
        env::set_var(
            X_ADS_ACCESS_TOKEN_SECRET_ENV_VAR,
            "shell-access-token-secret",
        );
        env::set_var("X_ADS_API_VERSION", "12");

        let config = XResolvedConfig::load(
            Some(&path),
            &store,
            &XConfigOverrides {
                timeout_seconds: Some(30),
                ..XConfigOverrides::default()
            },
        )
        .unwrap();

        assert_eq!(config.consumer_key, "shell-consumer-key");
        assert_eq!(config.api_version, "12");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.output_format, OutputFormat::Csv);

        clear_x_env();
    }

    #[test]
    fn inspect_uses_keychain_secrets() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_x_env();

        let store = FakeSecretStore::with_x_secrets();
        let snapshot = x_inspect_auth(&store);

        assert_eq!(snapshot.consumer_key.source, XSecretSource::Keychain);
        assert!(snapshot.access_token.present);
        assert!(snapshot.access_token_secret.present);
    }

    #[test]
    fn resolves_output_format_from_provider_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_x_env();

        let store = FakeSecretStore::with_x_secrets();
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"providers":{"x":{"output_format":"jsonl","default_account_id":"18ce54d4x5t"}}}"#,
        )
        .unwrap();

        let snapshot = x_inspect(Some(&path), &store, &XConfigOverrides::default()).unwrap();
        assert_eq!(snapshot.output_format, OutputFormat::Jsonl);
        assert_eq!(snapshot.default_account_id.as_deref(), Some("18ce54d4x5t"));
    }
}
