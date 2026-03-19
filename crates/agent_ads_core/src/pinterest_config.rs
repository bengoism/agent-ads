use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{load_root_file_config, DEFAULT_CONFIG_FILE};
use crate::output::OutputFormat;
use crate::pinterest_error::{PinterestError, PinterestResult};
use crate::secret_store::{
    SecretStore, SecretStoreError, SecretStoreErrorKind, PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT,
    PINTEREST_ADS_ACCESS_TOKEN_SERVICE, PINTEREST_ADS_APP_ID_ACCOUNT, PINTEREST_ADS_APP_ID_SERVICE,
    PINTEREST_ADS_APP_SECRET_ACCOUNT, PINTEREST_ADS_APP_SECRET_SERVICE,
    PINTEREST_ADS_REFRESH_TOKEN_ACCOUNT, PINTEREST_ADS_REFRESH_TOKEN_SERVICE,
};

pub const PINTEREST_DEFAULT_API_BASE_URL: &str = "https://api.pinterest.com";
pub const PINTEREST_DEFAULT_API_VERSION: &str = "v5";
const PINTEREST_DEFAULT_TIMEOUT_SECONDS: u64 = 60;

const PINTEREST_ADS_APP_ID_ENV_VAR: &str = "PINTEREST_ADS_APP_ID";
const PINTEREST_ADS_APP_SECRET_ENV_VAR: &str = "PINTEREST_ADS_APP_SECRET";
const PINTEREST_ADS_ACCESS_TOKEN_ENV_VAR: &str = "PINTEREST_ADS_ACCESS_TOKEN";
const PINTEREST_ADS_REFRESH_TOKEN_ENV_VAR: &str = "PINTEREST_ADS_REFRESH_TOKEN";
const PINTEREST_ADS_API_BASE_URL_ENV_VAR: &str = "PINTEREST_ADS_API_BASE_URL";
const PINTEREST_ADS_API_VERSION_ENV_VAR: &str = "PINTEREST_ADS_API_VERSION";
const PINTEREST_ADS_TIMEOUT_SECONDS_ENV_VAR: &str = "PINTEREST_ADS_TIMEOUT_SECONDS";
const PINTEREST_ADS_DEFAULT_AD_ACCOUNT_ID_ENV_VAR: &str = "PINTEREST_ADS_DEFAULT_AD_ACCOUNT_ID";
const PINTEREST_ADS_OUTPUT_FORMAT_ENV_VAR: &str = "PINTEREST_ADS_OUTPUT_FORMAT";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PinterestFileConfig {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_ad_account_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Default)]
pub struct PinterestConfigOverrides {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_ad_account_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone)]
pub struct PinterestResolvedConfig {
    pub access_token: String,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_ad_account_id: Option<String>,
    pub output_format: OutputFormat,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PinterestSecretSource {
    ShellEnv,
    Keychain,
    Missing,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PinterestSecretStatus {
    pub present: bool,
    pub source: PinterestSecretSource,
    pub keychain_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PinterestAuthSnapshot {
    pub app_id: PinterestSecretStatus,
    pub app_secret: PinterestSecretStatus,
    pub access_token: PinterestSecretStatus,
    pub refresh_token: PinterestSecretStatus,
    pub credential_store_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_store_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PinterestConfigSnapshot {
    pub config_path: PathBuf,
    pub config_file_exists: bool,
    pub auth: PinterestAuthSnapshot,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_ad_account_id: Option<String>,
    pub output_format: OutputFormat,
}

struct PinterestSecretResolution {
    value: Option<String>,
    status: PinterestSecretStatus,
    store_error: Option<SecretStoreError>,
}

struct PinterestAuthResolution {
    app_id: PinterestSecretResolution,
    app_secret: PinterestSecretResolution,
    access_token: PinterestSecretResolution,
    refresh_token: PinterestSecretResolution,
}

impl PinterestAuthResolution {
    fn snapshot(&self) -> PinterestAuthSnapshot {
        let store_error = [
            &self.app_id,
            &self.app_secret,
            &self.access_token,
            &self.refresh_token,
        ]
        .iter()
        .find_map(|resolution| resolution.store_error.as_ref().cloned());

        PinterestAuthSnapshot {
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

impl PinterestResolvedConfig {
    pub fn load(
        config_path: Option<&Path>,
        secret_store: &dyn SecretStore,
        overrides: &PinterestConfigOverrides,
    ) -> PinterestResult<Self> {
        let auth_resolution = resolve_pinterest_auth(secret_store);
        let snapshot =
            pinterest_inspect_with_auth(config_path, &auth_resolution.snapshot(), overrides)?;

        let access_token = auth_resolution
            .access_token
            .value
            .ok_or_else(|| missing_pinterest_access_token_error(&snapshot.auth))?;

        Ok(Self {
            access_token,
            api_base_url: snapshot.api_base_url,
            api_version: snapshot.api_version,
            timeout_seconds: snapshot.timeout_seconds,
            default_ad_account_id: snapshot.default_ad_account_id,
            output_format: snapshot.output_format,
            config_path: snapshot.config_path,
        })
    }
}

pub fn pinterest_inspect_auth(secret_store: &dyn SecretStore) -> PinterestAuthSnapshot {
    resolve_pinterest_auth(secret_store).snapshot()
}

pub fn pinterest_inspect(
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &PinterestConfigOverrides,
) -> PinterestResult<PinterestConfigSnapshot> {
    let auth_snapshot = pinterest_inspect_auth(secret_store);
    pinterest_inspect_with_auth(config_path, &auth_snapshot, overrides)
}

fn pinterest_inspect_with_auth(
    config_path: Option<&Path>,
    auth_snapshot: &PinterestAuthSnapshot,
    overrides: &PinterestConfigOverrides,
) -> PinterestResult<PinterestConfigSnapshot> {
    let config_path = config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));
    let config_file_exists = config_path.exists();
    let file_config = load_pinterest_file_config(&config_path)?;

    let api_base_url = overrides
        .api_base_url
        .clone()
        .or_else(|| env::var(PINTEREST_ADS_API_BASE_URL_ENV_VAR).ok())
        .or(file_config.api_base_url)
        .unwrap_or_else(|| PINTEREST_DEFAULT_API_BASE_URL.to_string());
    let api_version = overrides
        .api_version
        .clone()
        .or_else(|| env::var(PINTEREST_ADS_API_VERSION_ENV_VAR).ok())
        .or(file_config.api_version)
        .unwrap_or_else(|| PINTEREST_DEFAULT_API_VERSION.to_string());
    let timeout_seconds = overrides
        .timeout_seconds
        .or_else(|| {
            env::var(PINTEREST_ADS_TIMEOUT_SECONDS_ENV_VAR)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
        })
        .or(file_config.timeout_seconds)
        .unwrap_or(PINTEREST_DEFAULT_TIMEOUT_SECONDS);
    let default_ad_account_id = overrides
        .default_ad_account_id
        .clone()
        .or_else(|| env::var(PINTEREST_ADS_DEFAULT_AD_ACCOUNT_ID_ENV_VAR).ok())
        .or(file_config.default_ad_account_id);
    let output_format = overrides
        .output_format
        .or_else(|| {
            env::var(PINTEREST_ADS_OUTPUT_FORMAT_ENV_VAR)
                .ok()
                .and_then(|value| value.parse::<OutputFormat>().ok())
        })
        .or(file_config.output_format)
        .unwrap_or(OutputFormat::Json);

    Ok(PinterestConfigSnapshot {
        config_path,
        config_file_exists,
        auth: auth_snapshot.clone(),
        api_base_url,
        api_version,
        timeout_seconds,
        default_ad_account_id,
        output_format,
    })
}

fn resolve_pinterest_auth(secret_store: &dyn SecretStore) -> PinterestAuthResolution {
    PinterestAuthResolution {
        app_id: resolve_secret(
            secret_store,
            PINTEREST_ADS_APP_ID_ENV_VAR,
            PINTEREST_ADS_APP_ID_SERVICE,
            PINTEREST_ADS_APP_ID_ACCOUNT,
        ),
        app_secret: resolve_secret(
            secret_store,
            PINTEREST_ADS_APP_SECRET_ENV_VAR,
            PINTEREST_ADS_APP_SECRET_SERVICE,
            PINTEREST_ADS_APP_SECRET_ACCOUNT,
        ),
        access_token: resolve_secret(
            secret_store,
            PINTEREST_ADS_ACCESS_TOKEN_ENV_VAR,
            PINTEREST_ADS_ACCESS_TOKEN_SERVICE,
            PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT,
        ),
        refresh_token: resolve_secret(
            secret_store,
            PINTEREST_ADS_REFRESH_TOKEN_ENV_VAR,
            PINTEREST_ADS_REFRESH_TOKEN_SERVICE,
            PINTEREST_ADS_REFRESH_TOKEN_ACCOUNT,
        ),
    }
}

fn resolve_secret(
    secret_store: &dyn SecretStore,
    env_var: &str,
    service: &str,
    account: &str,
) -> PinterestSecretResolution {
    let shell_value = env::var(env_var).ok();
    let keychain_result = secret_store.get_secret(service, account);

    match (shell_value, keychain_result) {
        (Some(shell_value), Ok(keychain_value)) => PinterestSecretResolution {
            value: Some(shell_value),
            status: PinterestSecretStatus {
                present: true,
                source: PinterestSecretSource::ShellEnv,
                keychain_present: keychain_value.is_some(),
            },
            store_error: None,
        },
        (Some(shell_value), Err(error)) => PinterestSecretResolution {
            value: Some(shell_value),
            status: PinterestSecretStatus {
                present: true,
                source: PinterestSecretSource::ShellEnv,
                keychain_present: false,
            },
            store_error: Some(error),
        },
        (None, Ok(Some(keychain_value))) => PinterestSecretResolution {
            value: Some(keychain_value),
            status: PinterestSecretStatus {
                present: true,
                source: PinterestSecretSource::Keychain,
                keychain_present: true,
            },
            store_error: None,
        },
        (None, Ok(None)) => PinterestSecretResolution {
            value: None,
            status: PinterestSecretStatus {
                present: false,
                source: PinterestSecretSource::Missing,
                keychain_present: false,
            },
            store_error: None,
        },
        (None, Err(error)) => PinterestSecretResolution {
            value: None,
            status: PinterestSecretStatus {
                present: false,
                source: PinterestSecretSource::Missing,
                keychain_present: false,
            },
            store_error: Some(error),
        },
    }
}

fn missing_pinterest_access_token_error(auth: &PinterestAuthSnapshot) -> PinterestError {
    let guidance = pinterest_access_token_guidance();
    match auth.credential_store_error.as_deref() {
        Some(detail) => PinterestError::Config(format!(
            "Pinterest access token is missing and the OS credential store could not be read: {detail}. {guidance}"
        )),
        None => PinterestError::Config(format!(
            "Pinterest access token is missing. {guidance}"
        )),
    }
}

fn pinterest_access_token_guidance() -> String {
    let mut message = format!(
        "Set {PINTEREST_ADS_ACCESS_TOKEN_ENV_VAR} in the shell or run `agent-ads pinterest auth set` to store it in your OS credential store."
    );
    if cfg!(target_os = "linux") {
        message.push_str(
            " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet.",
        );
    }
    message
}

fn load_pinterest_file_config(path: &Path) -> PinterestResult<PinterestFileConfig> {
    let config = load_root_file_config(path).map_err(PinterestError::from)?;
    Ok(config.providers.pinterest.unwrap_or_default())
}

impl From<crate::error::MetaAdsError> for PinterestError {
    fn from(value: crate::error::MetaAdsError) -> Self {
        match value {
            crate::error::MetaAdsError::Config(message) => Self::Config(message),
            crate::error::MetaAdsError::InvalidArgument(message) => Self::InvalidArgument(message),
            crate::error::MetaAdsError::Http(error) => Self::Http(error),
            crate::error::MetaAdsError::Io(error) => Self::Io(error),
            crate::error::MetaAdsError::Json(error) => Self::Json(error),
            crate::error::MetaAdsError::Csv(error) => Self::Csv(error),
            crate::error::MetaAdsError::Api(error) => Self::Config(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::sync::{LazyLock, Mutex};

    use tempfile::tempdir;

    use super::{pinterest_inspect, PinterestConfigOverrides, PinterestResolvedConfig};
    use crate::output::OutputFormat;
    use crate::secret_store::{
        SecretStore, SecretStoreError, SecretStoreErrorKind, PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT,
        PINTEREST_ADS_ACCESS_TOKEN_SERVICE,
    };

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
        get_error: Mutex<Option<SecretStoreError>>,
    }

    impl FakeSecretStore {
        fn with_access_token(access_token: &str) -> Self {
            let store = Self::default();
            store.secrets.lock().unwrap().insert(
                (
                    PINTEREST_ADS_ACCESS_TOKEN_SERVICE.to_string(),
                    PINTEREST_ADS_ACCESS_TOKEN_ACCOUNT.to_string(),
                ),
                access_token.to_string(),
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

    fn clear_pinterest_env() {
        for key in [
            "PINTEREST_ADS_APP_ID",
            "PINTEREST_ADS_APP_SECRET",
            "PINTEREST_ADS_ACCESS_TOKEN",
            "PINTEREST_ADS_REFRESH_TOKEN",
            "PINTEREST_ADS_API_BASE_URL",
            "PINTEREST_ADS_API_VERSION",
            "PINTEREST_ADS_TIMEOUT_SECONDS",
            "PINTEREST_ADS_DEFAULT_AD_ACCOUNT_ID",
            "PINTEREST_ADS_OUTPUT_FORMAT",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn load_uses_access_token_without_requiring_refresh_credentials() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_pinterest_env();

        let store = FakeSecretStore::with_access_token("keychain-token");
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        std::fs::write(
            &path,
            r#"{"providers":{"pinterest":{"api_version":"v6","output_format":"jsonl"}}}"#,
        )
        .unwrap();

        let config = PinterestResolvedConfig::load(
            Some(&path),
            &store,
            &PinterestConfigOverrides::default(),
        )
        .unwrap();

        assert_eq!(config.access_token, "keychain-token");
        assert_eq!(config.api_version, "v6");
        assert_eq!(config.output_format, OutputFormat::Jsonl);
    }

    #[test]
    fn missing_access_token_errors_with_setup_guidance() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_pinterest_env();

        let store = FakeSecretStore::default();
        let error =
            PinterestResolvedConfig::load(None, &store, &PinterestConfigOverrides::default())
                .unwrap_err();

        assert!(error.to_string().contains("PINTEREST_ADS_ACCESS_TOKEN"));
    }

    #[test]
    fn inspect_reports_unavailable_store_without_breaking_shell_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_pinterest_env();

        env::set_var("PINTEREST_ADS_ACCESS_TOKEN", "shell-token");

        let store = FakeSecretStore::default();
        store.set_get_error(SecretStoreError::new(
            SecretStoreErrorKind::Unavailable,
            "keychain unavailable".to_string(),
        ));

        let snapshot =
            pinterest_inspect(None, &store, &PinterestConfigOverrides::default()).unwrap();

        assert!(snapshot.auth.access_token.present);
        assert_eq!(snapshot.output_format, OutputFormat::Json);

        clear_pinterest_env();
    }
}
