use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::auth_bundle::load_auth_bundle;
use crate::config::{load_root_file_config, DEFAULT_CONFIG_FILE};
use crate::linkedin_error::{LinkedInError, LinkedInResult};
use crate::output::OutputFormat;
use crate::secret_store::{SecretStore, SecretStoreError, SecretStoreErrorKind};

pub const LINKEDIN_DEFAULT_API_BASE_URL: &str = "https://api.linkedin.com/rest";
pub const LINKEDIN_DEFAULT_API_VERSION: &str = "202603";
const LINKEDIN_DEFAULT_TIMEOUT_SECONDS: u64 = 60;

pub const LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR: &str = "LINKEDIN_ADS_ACCESS_TOKEN";
const LINKEDIN_ADS_API_BASE_URL_ENV_VAR: &str = "LINKEDIN_ADS_API_BASE_URL";
const LINKEDIN_ADS_API_VERSION_ENV_VAR: &str = "LINKEDIN_ADS_API_VERSION";
const LINKEDIN_ADS_TIMEOUT_SECONDS_ENV_VAR: &str = "LINKEDIN_ADS_TIMEOUT_SECONDS";
const LINKEDIN_ADS_DEFAULT_ACCOUNT_ID_ENV_VAR: &str = "LINKEDIN_ADS_DEFAULT_ACCOUNT_ID";
const LINKEDIN_ADS_OUTPUT_FORMAT_ENV_VAR: &str = "LINKEDIN_ADS_OUTPUT_FORMAT";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkedInFileConfig {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_account_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Default)]
pub struct LinkedInConfigOverrides {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_account_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone)]
pub struct LinkedInResolvedConfig {
    pub access_token: String,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_account_id: Option<String>,
    pub output_format: OutputFormat,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkedInAccessTokenSource {
    ShellEnv,
    Keychain,
    Missing,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LinkedInAccessTokenStatus {
    pub present: bool,
    pub source: LinkedInAccessTokenSource,
    pub keychain_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkedInAuthSnapshot {
    pub access_token: LinkedInAccessTokenStatus,
    pub credential_store_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_store_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkedInConfigSnapshot {
    pub config_path: PathBuf,
    pub config_file_exists: bool,
    pub auth: LinkedInAuthSnapshot,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_account_id: Option<String>,
    pub output_format: OutputFormat,
}

struct LinkedInAccessTokenResolution {
    value: Option<String>,
    status: LinkedInAccessTokenStatus,
    store_error: Option<SecretStoreError>,
}

impl LinkedInAccessTokenResolution {
    fn snapshot(&self) -> LinkedInAuthSnapshot {
        LinkedInAuthSnapshot {
            access_token: self.status.clone(),
            credential_store_available: self
                .store_error
                .as_ref()
                .map(|error| error.kind() != SecretStoreErrorKind::Unavailable)
                .unwrap_or(true),
            credential_store_error: self.store_error.as_ref().map(ToString::to_string),
        }
    }
}

impl LinkedInResolvedConfig {
    pub fn load(
        config_path: Option<&Path>,
        secret_store: &dyn SecretStore,
        overrides: &LinkedInConfigOverrides,
    ) -> LinkedInResult<Self> {
        let access_token_resolution = resolve_access_token(secret_store);
        let snapshot = linkedin_inspect_with_auth(
            config_path,
            &access_token_resolution.snapshot(),
            overrides,
        )?;
        let access_token = access_token_resolution
            .value
            .ok_or_else(|| missing_access_token_error(&snapshot.auth))?;

        Ok(Self {
            access_token,
            api_base_url: snapshot.api_base_url,
            api_version: snapshot.api_version,
            timeout_seconds: snapshot.timeout_seconds,
            default_account_id: snapshot.default_account_id,
            output_format: snapshot.output_format,
            config_path: snapshot.config_path,
        })
    }
}

pub fn linkedin_inspect_auth(secret_store: &dyn SecretStore) -> LinkedInAuthSnapshot {
    resolve_access_token(secret_store).snapshot()
}

pub fn linkedin_inspect(
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &LinkedInConfigOverrides,
) -> LinkedInResult<LinkedInConfigSnapshot> {
    let auth_snapshot = linkedin_inspect_auth(secret_store);
    linkedin_inspect_with_auth(config_path, &auth_snapshot, overrides)
}

fn linkedin_inspect_with_auth(
    config_path: Option<&Path>,
    auth_snapshot: &LinkedInAuthSnapshot,
    overrides: &LinkedInConfigOverrides,
) -> LinkedInResult<LinkedInConfigSnapshot> {
    let config_path = config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));
    let config_file_exists = config_path.exists();
    let file_config = load_linkedin_file_config(&config_path)?;

    let api_base_url = overrides
        .api_base_url
        .clone()
        .or_else(|| env::var(LINKEDIN_ADS_API_BASE_URL_ENV_VAR).ok())
        .or(file_config.api_base_url)
        .unwrap_or_else(|| LINKEDIN_DEFAULT_API_BASE_URL.to_string());
    let api_version = overrides
        .api_version
        .clone()
        .or_else(|| env::var(LINKEDIN_ADS_API_VERSION_ENV_VAR).ok())
        .or(file_config.api_version)
        .unwrap_or_else(|| LINKEDIN_DEFAULT_API_VERSION.to_string());
    let timeout_seconds = overrides
        .timeout_seconds
        .or_else(|| {
            env::var(LINKEDIN_ADS_TIMEOUT_SECONDS_ENV_VAR)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
        })
        .or(file_config.timeout_seconds)
        .unwrap_or(LINKEDIN_DEFAULT_TIMEOUT_SECONDS);
    let default_account_id = overrides
        .default_account_id
        .clone()
        .or_else(|| env::var(LINKEDIN_ADS_DEFAULT_ACCOUNT_ID_ENV_VAR).ok())
        .or(file_config.default_account_id);
    let output_format = overrides
        .output_format
        .or_else(|| {
            env::var(LINKEDIN_ADS_OUTPUT_FORMAT_ENV_VAR)
                .ok()
                .and_then(|value| value.parse::<OutputFormat>().ok())
        })
        .or(file_config.output_format)
        .unwrap_or(OutputFormat::Json);

    Ok(LinkedInConfigSnapshot {
        config_path,
        config_file_exists,
        auth: auth_snapshot.clone(),
        api_base_url,
        api_version,
        timeout_seconds,
        default_account_id,
        output_format,
    })
}

fn resolve_access_token(secret_store: &dyn SecretStore) -> LinkedInAccessTokenResolution {
    let shell_token = env::var(LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let bundle_result = load_auth_bundle(secret_store);
    let keychain_token = bundle_result
        .as_ref()
        .ok()
        .and_then(|bundle| bundle.linkedin.as_ref())
        .and_then(|linkedin| linkedin.access_token.clone());
    let store_error = bundle_result.as_ref().err().cloned();

    match (shell_token, keychain_token, store_error) {
        (Some(shell_token), keychain_token, None) => LinkedInAccessTokenResolution {
            value: Some(shell_token),
            status: LinkedInAccessTokenStatus {
                present: true,
                source: LinkedInAccessTokenSource::ShellEnv,
                keychain_present: keychain_token.is_some(),
            },
            store_error: None,
        },
        (Some(shell_token), _, Some(error)) => LinkedInAccessTokenResolution {
            value: Some(shell_token),
            status: LinkedInAccessTokenStatus {
                present: true,
                source: LinkedInAccessTokenSource::ShellEnv,
                keychain_present: false,
            },
            store_error: Some(error),
        },
        (None, Some(keychain_token), None) => LinkedInAccessTokenResolution {
            value: Some(keychain_token),
            status: LinkedInAccessTokenStatus {
                present: true,
                source: LinkedInAccessTokenSource::Keychain,
                keychain_present: true,
            },
            store_error: None,
        },
        (None, Some(keychain_token), Some(_)) => LinkedInAccessTokenResolution {
            value: Some(keychain_token),
            status: LinkedInAccessTokenStatus {
                present: true,
                source: LinkedInAccessTokenSource::Keychain,
                keychain_present: true,
            },
            store_error: None,
        },
        (None, None, None) => LinkedInAccessTokenResolution {
            value: None,
            status: LinkedInAccessTokenStatus {
                present: false,
                source: LinkedInAccessTokenSource::Missing,
                keychain_present: false,
            },
            store_error: None,
        },
        (None, None, Some(error)) => LinkedInAccessTokenResolution {
            value: None,
            status: LinkedInAccessTokenStatus {
                present: false,
                source: LinkedInAccessTokenSource::Missing,
                keychain_present: false,
            },
            store_error: Some(error),
        },
    }
}

fn load_linkedin_file_config(path: &Path) -> LinkedInResult<LinkedInFileConfig> {
    let config = load_root_file_config(path)?;
    Ok(config.providers.linkedin.unwrap_or_default())
}

fn missing_access_token_error(auth: &LinkedInAuthSnapshot) -> LinkedInError {
    let guidance = linkedin_access_token_guidance();
    match auth.credential_store_error.as_deref() {
        Some(detail) => LinkedInError::Config(format!(
            "{LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR} is missing and the OS credential store could not be read: {detail}. {guidance}"
        )),
        None => LinkedInError::Config(format!(
            "{LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR} is missing. {guidance}"
        )),
    }
}

fn linkedin_access_token_guidance() -> String {
    let mut message = format!(
        "Set {LINKEDIN_ADS_ACCESS_TOKEN_ENV_VAR} in the shell for this process or run `agent-ads linkedin auth set` to store it in your OS credential store."
    );
    if cfg!(target_os = "linux") {
        message.push_str(
            " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet.",
        );
    }
    message
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::sync::{LazyLock, Mutex};

    use tempfile::tempdir;

    use super::{
        linkedin_inspect, linkedin_inspect_auth, LinkedInAccessTokenSource,
        LinkedInConfigOverrides, LinkedInResolvedConfig,
    };
    use crate::output::OutputFormat;
    use crate::secret_store::{SecretStore, SecretStoreError};
    use crate::{store_auth_bundle, AuthBundle, LinkedInAuthBundle};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
    }

    impl FakeSecretStore {
        fn with_access_token(access_token: &str) -> Self {
            let store = Self::default();
            store_auth_bundle(
                &store,
                &AuthBundle {
                    linkedin: Some(LinkedInAuthBundle {
                        access_token: Some(access_token.to_string()),
                    }),
                    ..AuthBundle::default()
                },
            )
            .unwrap();
            store
        }
    }

    impl SecretStore for FakeSecretStore {
        fn get_secret(
            &self,
            service: &str,
            account: &str,
        ) -> Result<Option<String>, SecretStoreError> {
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
        ) -> Result<(), SecretStoreError> {
            self.secrets.lock().unwrap().insert(
                (service.to_string(), account.to_string()),
                secret.to_string(),
            );
            Ok(())
        }

        fn delete_secret(&self, service: &str, account: &str) -> Result<bool, SecretStoreError> {
            Ok(self
                .secrets
                .lock()
                .unwrap()
                .remove(&(service.to_string(), account.to_string()))
                .is_some())
        }
    }

    fn clear_linkedin_env() {
        for key in [
            "LINKEDIN_ADS_ACCESS_TOKEN",
            "LINKEDIN_ADS_API_BASE_URL",
            "LINKEDIN_ADS_API_VERSION",
            "LINKEDIN_ADS_TIMEOUT_SECONDS",
            "LINKEDIN_ADS_DEFAULT_ACCOUNT_ID",
            "LINKEDIN_ADS_OUTPUT_FORMAT",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn resolves_precedence() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_linkedin_env();

        let store = FakeSecretStore::with_access_token("keychain-token");
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"providers":{"linkedin":{"api_version":"202601","timeout_seconds":10,"output_format":"jsonl"}}}"#,
        )
        .unwrap();

        env::set_var("LINKEDIN_ADS_ACCESS_TOKEN", "shell-token");
        env::set_var("LINKEDIN_ADS_API_VERSION", "202602");

        let config = LinkedInResolvedConfig::load(
            Some(&path),
            &store,
            &LinkedInConfigOverrides {
                timeout_seconds: Some(30),
                ..LinkedInConfigOverrides::default()
            },
        )
        .unwrap();

        assert_eq!(config.access_token, "shell-token");
        assert_eq!(config.api_version, "202602");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.output_format, OutputFormat::Jsonl);

        clear_linkedin_env();
    }

    #[test]
    fn uses_keychain_token_when_shell_env_is_absent() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_linkedin_env();

        let store = FakeSecretStore::with_access_token("keychain-token");
        let snapshot = linkedin_inspect(None, &store, &LinkedInConfigOverrides::default()).unwrap();
        let config =
            LinkedInResolvedConfig::load(None, &store, &LinkedInConfigOverrides::default())
                .unwrap();

        assert_eq!(config.access_token, "keychain-token");
        assert_eq!(
            snapshot.auth.access_token.source,
            LinkedInAccessTokenSource::Keychain
        );
    }

    #[test]
    fn inspect_auth_reads_shell_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_linkedin_env();
        env::set_var("LINKEDIN_ADS_ACCESS_TOKEN", "shell-token");

        let snapshot = linkedin_inspect_auth(&FakeSecretStore::default());

        assert!(snapshot.access_token.present);
        assert_eq!(
            snapshot.access_token.source,
            LinkedInAccessTokenSource::ShellEnv
        );

        clear_linkedin_env();
    }
}
