use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{load_root_file_config, DEFAULT_CONFIG_FILE};
use crate::google_error::{GoogleError, GoogleResult};
use crate::output::OutputFormat;
use crate::secret_store::{
    SecretStore, SecretStoreError, SecretStoreErrorKind, GOOGLE_ADS_CLIENT_ID_ACCOUNT,
    GOOGLE_ADS_CLIENT_ID_SERVICE, GOOGLE_ADS_CLIENT_SECRET_ACCOUNT,
    GOOGLE_ADS_CLIENT_SECRET_SERVICE, GOOGLE_ADS_DEVELOPER_TOKEN_ACCOUNT,
    GOOGLE_ADS_DEVELOPER_TOKEN_SERVICE, GOOGLE_ADS_REFRESH_TOKEN_ACCOUNT,
    GOOGLE_ADS_REFRESH_TOKEN_SERVICE,
};

pub const GOOGLE_DEFAULT_API_BASE_URL: &str = "https://googleads.googleapis.com";
pub const GOOGLE_DEFAULT_API_VERSION: &str = "v23";
const GOOGLE_DEFAULT_TIMEOUT_SECONDS: u64 = 60;

const GOOGLE_ADS_API_BASE_URL_ENV_VAR: &str = "GOOGLE_ADS_API_BASE_URL";
const GOOGLE_ADS_API_VERSION_ENV_VAR: &str = "GOOGLE_ADS_API_VERSION";
const GOOGLE_ADS_TIMEOUT_SECONDS_ENV_VAR: &str = "GOOGLE_ADS_TIMEOUT_SECONDS";
const GOOGLE_ADS_DEFAULT_CUSTOMER_ID_ENV_VAR: &str = "GOOGLE_ADS_DEFAULT_CUSTOMER_ID";
const GOOGLE_ADS_LOGIN_CUSTOMER_ID_ENV_VAR: &str = "GOOGLE_ADS_LOGIN_CUSTOMER_ID";
const GOOGLE_ADS_OUTPUT_FORMAT_ENV_VAR: &str = "GOOGLE_ADS_OUTPUT_FORMAT";
const GOOGLE_ADS_DEVELOPER_TOKEN_ENV_VAR: &str = "GOOGLE_ADS_DEVELOPER_TOKEN";
const GOOGLE_ADS_CLIENT_ID_ENV_VAR: &str = "GOOGLE_ADS_CLIENT_ID";
const GOOGLE_ADS_CLIENT_SECRET_ENV_VAR: &str = "GOOGLE_ADS_CLIENT_SECRET";
const GOOGLE_ADS_REFRESH_TOKEN_ENV_VAR: &str = "GOOGLE_ADS_REFRESH_TOKEN";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleFileConfig {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_customer_id: Option<String>,
    pub login_customer_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Default)]
pub struct GoogleConfigOverrides {
    pub api_base_url: Option<String>,
    pub api_version: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub default_customer_id: Option<String>,
    pub login_customer_id: Option<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone)]
pub struct GoogleResolvedConfig {
    pub developer_token: String,
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_customer_id: Option<String>,
    pub login_customer_id: Option<String>,
    pub output_format: OutputFormat,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoogleSecretSource {
    ShellEnv,
    Keychain,
    Missing,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GoogleSecretStatus {
    pub present: bool,
    pub source: GoogleSecretSource,
    pub keychain_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoogleAuthSnapshot {
    pub developer_token: GoogleSecretStatus,
    pub client_id: GoogleSecretStatus,
    pub client_secret: GoogleSecretStatus,
    pub refresh_token: GoogleSecretStatus,
    pub credential_store_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_store_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoogleConfigSnapshot {
    pub config_path: PathBuf,
    pub config_file_exists: bool,
    pub auth: GoogleAuthSnapshot,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_customer_id: Option<String>,
    pub login_customer_id: Option<String>,
    pub output_format: OutputFormat,
}

struct GoogleSecretResolution {
    value: Option<String>,
    status: GoogleSecretStatus,
    store_error: Option<SecretStoreError>,
}

struct GoogleAuthResolution {
    developer_token: GoogleSecretResolution,
    client_id: GoogleSecretResolution,
    client_secret: GoogleSecretResolution,
    refresh_token: GoogleSecretResolution,
}

impl GoogleAuthResolution {
    fn snapshot(&self) -> GoogleAuthSnapshot {
        let store_error = [
            &self.developer_token,
            &self.client_id,
            &self.client_secret,
            &self.refresh_token,
        ]
        .iter()
        .find_map(|resolution| resolution.store_error.as_ref().cloned());

        GoogleAuthSnapshot {
            developer_token: self.developer_token.status.clone(),
            client_id: self.client_id.status.clone(),
            client_secret: self.client_secret.status.clone(),
            refresh_token: self.refresh_token.status.clone(),
            credential_store_available: store_error
                .as_ref()
                .map(|error| error.kind() != SecretStoreErrorKind::Unavailable)
                .unwrap_or(true),
            credential_store_error: store_error.map(|error| error.to_string()),
        }
    }
}

impl GoogleResolvedConfig {
    pub fn load(
        config_path: Option<&Path>,
        secret_store: &dyn SecretStore,
        overrides: &GoogleConfigOverrides,
    ) -> GoogleResult<Self> {
        let auth_resolution = resolve_google_auth(secret_store);
        let snapshot =
            google_inspect_with_auth(config_path, &auth_resolution.snapshot(), overrides)?;

        let developer_token = auth_resolution
            .developer_token
            .value
            .ok_or_else(|| missing_google_credentials_error(&snapshot.auth))?;
        let client_id = auth_resolution
            .client_id
            .value
            .ok_or_else(|| missing_google_credentials_error(&snapshot.auth))?;
        let client_secret = auth_resolution
            .client_secret
            .value
            .ok_or_else(|| missing_google_credentials_error(&snapshot.auth))?;
        let refresh_token = auth_resolution
            .refresh_token
            .value
            .ok_or_else(|| missing_google_credentials_error(&snapshot.auth))?;

        Ok(Self {
            developer_token,
            client_id,
            client_secret,
            refresh_token,
            api_base_url: snapshot.api_base_url,
            api_version: snapshot.api_version,
            timeout_seconds: snapshot.timeout_seconds,
            default_customer_id: snapshot.default_customer_id,
            login_customer_id: snapshot.login_customer_id,
            output_format: snapshot.output_format,
            config_path: snapshot.config_path,
        })
    }
}

pub fn google_inspect_auth(secret_store: &dyn SecretStore) -> GoogleAuthSnapshot {
    resolve_google_auth(secret_store).snapshot()
}

pub fn google_inspect(
    config_path: Option<&Path>,
    secret_store: &dyn SecretStore,
    overrides: &GoogleConfigOverrides,
) -> GoogleResult<GoogleConfigSnapshot> {
    let auth_snapshot = google_inspect_auth(secret_store);
    google_inspect_with_auth(config_path, &auth_snapshot, overrides)
}

pub fn normalize_google_customer_id(value: &str) -> GoogleResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(GoogleError::InvalidArgument(
            "customer ID input was empty".to_string(),
        ));
    }

    let normalized = trimmed.replace('-', "");
    if normalized.is_empty()
        || !normalized
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(GoogleError::InvalidArgument(format!(
            "invalid Google customer ID `{value}`"
        )));
    }

    Ok(normalized)
}

fn google_inspect_with_auth(
    config_path: Option<&Path>,
    auth_snapshot: &GoogleAuthSnapshot,
    overrides: &GoogleConfigOverrides,
) -> GoogleResult<GoogleConfigSnapshot> {
    let config_path = config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));
    let file_config = load_google_file_config(&config_path)?;

    let api_base_url = overrides
        .api_base_url
        .clone()
        .or_else(|| env::var(GOOGLE_ADS_API_BASE_URL_ENV_VAR).ok())
        .or(file_config.api_base_url)
        .unwrap_or_else(|| GOOGLE_DEFAULT_API_BASE_URL.to_string());
    let api_version = overrides
        .api_version
        .clone()
        .or_else(|| env::var(GOOGLE_ADS_API_VERSION_ENV_VAR).ok())
        .or(file_config.api_version)
        .unwrap_or_else(|| GOOGLE_DEFAULT_API_VERSION.to_string());
    let timeout_seconds = overrides
        .timeout_seconds
        .or_else(|| {
            env::var(GOOGLE_ADS_TIMEOUT_SECONDS_ENV_VAR)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
        })
        .or(file_config.timeout_seconds)
        .unwrap_or(GOOGLE_DEFAULT_TIMEOUT_SECONDS);
    let default_customer_id = overrides
        .default_customer_id
        .clone()
        .or_else(|| env::var(GOOGLE_ADS_DEFAULT_CUSTOMER_ID_ENV_VAR).ok())
        .or(file_config.default_customer_id)
        .map(|value| normalize_google_customer_id(&value))
        .transpose()?;
    let login_customer_id = overrides
        .login_customer_id
        .clone()
        .or_else(|| env::var(GOOGLE_ADS_LOGIN_CUSTOMER_ID_ENV_VAR).ok())
        .or(file_config.login_customer_id)
        .map(|value| normalize_google_customer_id(&value))
        .transpose()?;
    let output_format = overrides
        .output_format
        .or_else(|| {
            env::var(GOOGLE_ADS_OUTPUT_FORMAT_ENV_VAR)
                .ok()
                .and_then(|value| value.parse::<OutputFormat>().ok())
        })
        .or(file_config.output_format)
        .unwrap_or(OutputFormat::Json);

    Ok(GoogleConfigSnapshot {
        config_path: config_path.clone(),
        config_file_exists: config_path.exists(),
        auth: auth_snapshot.clone(),
        api_base_url,
        api_version,
        timeout_seconds,
        default_customer_id,
        login_customer_id,
        output_format,
    })
}

fn resolve_google_auth(secret_store: &dyn SecretStore) -> GoogleAuthResolution {
    GoogleAuthResolution {
        developer_token: resolve_google_secret(
            GOOGLE_ADS_DEVELOPER_TOKEN_ENV_VAR,
            GOOGLE_ADS_DEVELOPER_TOKEN_SERVICE,
            GOOGLE_ADS_DEVELOPER_TOKEN_ACCOUNT,
            secret_store,
        ),
        client_id: resolve_google_secret(
            GOOGLE_ADS_CLIENT_ID_ENV_VAR,
            GOOGLE_ADS_CLIENT_ID_SERVICE,
            GOOGLE_ADS_CLIENT_ID_ACCOUNT,
            secret_store,
        ),
        client_secret: resolve_google_secret(
            GOOGLE_ADS_CLIENT_SECRET_ENV_VAR,
            GOOGLE_ADS_CLIENT_SECRET_SERVICE,
            GOOGLE_ADS_CLIENT_SECRET_ACCOUNT,
            secret_store,
        ),
        refresh_token: resolve_google_secret(
            GOOGLE_ADS_REFRESH_TOKEN_ENV_VAR,
            GOOGLE_ADS_REFRESH_TOKEN_SERVICE,
            GOOGLE_ADS_REFRESH_TOKEN_ACCOUNT,
            secret_store,
        ),
    }
}

fn resolve_google_secret(
    env_var: &str,
    service: &str,
    account: &str,
    secret_store: &dyn SecretStore,
) -> GoogleSecretResolution {
    let shell_value = env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let keychain_result = secret_store.get_secret(service, account);

    match (shell_value, keychain_result) {
        (Some(shell_value), Ok(keychain_value)) => GoogleSecretResolution {
            value: Some(shell_value),
            status: GoogleSecretStatus {
                present: true,
                source: GoogleSecretSource::ShellEnv,
                keychain_present: keychain_value.is_some(),
            },
            store_error: None,
        },
        (Some(shell_value), Err(error)) => GoogleSecretResolution {
            value: Some(shell_value),
            status: GoogleSecretStatus {
                present: true,
                source: GoogleSecretSource::ShellEnv,
                keychain_present: false,
            },
            store_error: Some(error),
        },
        (None, Ok(Some(keychain_value))) => GoogleSecretResolution {
            value: Some(keychain_value),
            status: GoogleSecretStatus {
                present: true,
                source: GoogleSecretSource::Keychain,
                keychain_present: true,
            },
            store_error: None,
        },
        (None, Ok(None)) => GoogleSecretResolution {
            value: None,
            status: GoogleSecretStatus {
                present: false,
                source: GoogleSecretSource::Missing,
                keychain_present: false,
            },
            store_error: None,
        },
        (None, Err(error)) => GoogleSecretResolution {
            value: None,
            status: GoogleSecretStatus {
                present: false,
                source: GoogleSecretSource::Missing,
                keychain_present: false,
            },
            store_error: Some(error),
        },
    }
}

fn missing_google_credentials_error(auth: &GoogleAuthSnapshot) -> GoogleError {
    let missing = [
        (
            GOOGLE_ADS_DEVELOPER_TOKEN_ENV_VAR,
            auth.developer_token.present,
        ),
        (GOOGLE_ADS_CLIENT_ID_ENV_VAR, auth.client_id.present),
        (GOOGLE_ADS_CLIENT_SECRET_ENV_VAR, auth.client_secret.present),
        (GOOGLE_ADS_REFRESH_TOKEN_ENV_VAR, auth.refresh_token.present),
    ]
    .iter()
    .filter_map(|(env_var, present)| (!present).then_some(*env_var))
    .collect::<Vec<_>>();

    let mut message = format!(
        "missing Google Ads credentials: {}. Set them in the shell for this process or run `agent-ads google auth set` to store them in your OS credential store.",
        missing.join(", ")
    );
    if let Some(error) = auth.credential_store_error.as_deref() {
        message.push_str(&format!(" OS credential store detail: {error}."));
    } else if cfg!(target_os = "linux") {
        message.push_str(
            " On Linux, secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet.",
        );
    }

    GoogleError::Config(message)
}

fn load_google_file_config(path: &Path) -> GoogleResult<GoogleFileConfig> {
    let root = load_root_file_config(path).map_err(|error| match error {
        crate::error::MetaAdsError::Io(io_error) => GoogleError::Io(io_error),
        crate::error::MetaAdsError::Json(json_error) => GoogleError::Json(json_error),
        other => GoogleError::Config(other.to_string()),
    })?;
    let mut config = root.providers.google.unwrap_or_default();
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
        google_inspect, google_inspect_auth, normalize_google_customer_id, GoogleConfigOverrides,
        GoogleResolvedConfig, GoogleSecretSource,
    };
    use crate::output::OutputFormat;
    use crate::secret_store::{
        SecretStore, SecretStoreError, SecretStoreErrorKind, GOOGLE_ADS_CLIENT_ID_ACCOUNT,
        GOOGLE_ADS_CLIENT_ID_SERVICE, GOOGLE_ADS_CLIENT_SECRET_ACCOUNT,
        GOOGLE_ADS_CLIENT_SECRET_SERVICE, GOOGLE_ADS_DEVELOPER_TOKEN_ACCOUNT,
        GOOGLE_ADS_DEVELOPER_TOKEN_SERVICE, GOOGLE_ADS_REFRESH_TOKEN_ACCOUNT,
        GOOGLE_ADS_REFRESH_TOKEN_SERVICE,
    };

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
        get_error: Mutex<Option<SecretStoreError>>,
    }

    impl FakeSecretStore {
        fn with_google_secrets() -> Self {
            let store = Self::default();
            store.put_google_secrets();
            store
        }

        fn put_google_secrets(&self) {
            let mut secrets = self.secrets.lock().unwrap();
            secrets.insert(
                (
                    GOOGLE_ADS_DEVELOPER_TOKEN_SERVICE.to_string(),
                    GOOGLE_ADS_DEVELOPER_TOKEN_ACCOUNT.to_string(),
                ),
                "dev-token".to_string(),
            );
            secrets.insert(
                (
                    GOOGLE_ADS_CLIENT_ID_SERVICE.to_string(),
                    GOOGLE_ADS_CLIENT_ID_ACCOUNT.to_string(),
                ),
                "client-id".to_string(),
            );
            secrets.insert(
                (
                    GOOGLE_ADS_CLIENT_SECRET_SERVICE.to_string(),
                    GOOGLE_ADS_CLIENT_SECRET_ACCOUNT.to_string(),
                ),
                "client-secret".to_string(),
            );
            secrets.insert(
                (
                    GOOGLE_ADS_REFRESH_TOKEN_SERVICE.to_string(),
                    GOOGLE_ADS_REFRESH_TOKEN_ACCOUNT.to_string(),
                ),
                "refresh-token".to_string(),
            );
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

    fn clear_google_env() {
        for key in [
            "GOOGLE_ADS_API_BASE_URL",
            "GOOGLE_ADS_API_VERSION",
            "GOOGLE_ADS_TIMEOUT_SECONDS",
            "GOOGLE_ADS_DEFAULT_CUSTOMER_ID",
            "GOOGLE_ADS_LOGIN_CUSTOMER_ID",
            "GOOGLE_ADS_OUTPUT_FORMAT",
            "GOOGLE_ADS_DEVELOPER_TOKEN",
            "GOOGLE_ADS_CLIENT_ID",
            "GOOGLE_ADS_CLIENT_SECRET",
            "GOOGLE_ADS_REFRESH_TOKEN",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn resolves_precedence_and_normalizes_customer_ids() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_google_env();

        let store = FakeSecretStore::with_google_secrets();
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"output_format":"csv","providers":{"google":{"api_version":"v22","timeout_seconds":10,"default_customer_id":"123-456-7890","login_customer_id":"111-222-3333"}}}"#,
        )
        .unwrap();

        env::set_var("GOOGLE_ADS_API_VERSION", "v23");
        env::set_var("GOOGLE_ADS_CLIENT_ID", "env-client-id");

        let config = GoogleResolvedConfig::load(
            Some(&path),
            &store,
            &GoogleConfigOverrides {
                timeout_seconds: Some(30),
                ..GoogleConfigOverrides::default()
            },
        )
        .unwrap();

        assert_eq!(config.client_id, "env-client-id");
        assert_eq!(config.api_version, "v23");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.output_format, OutputFormat::Csv);
        assert_eq!(config.default_customer_id.as_deref(), Some("1234567890"));
        assert_eq!(config.login_customer_id.as_deref(), Some("1112223333"));

        clear_google_env();
    }

    #[test]
    fn inspect_uses_keychain_when_shell_env_is_absent() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_google_env();

        let store = FakeSecretStore::with_google_secrets();
        let auth = google_inspect_auth(&store);
        let snapshot = google_inspect(None, &store, &GoogleConfigOverrides::default()).unwrap();

        assert_eq!(auth.developer_token.source, GoogleSecretSource::Keychain);
        assert!(snapshot.auth.refresh_token.present);

        clear_google_env();
    }

    #[test]
    fn missing_google_credentials_include_setup_guidance() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_google_env();

        let store = FakeSecretStore::default();
        let error = GoogleResolvedConfig::load(None, &store, &GoogleConfigOverrides::default())
            .unwrap_err();

        assert!(error.to_string().contains("agent-ads google auth set"));
    }

    #[test]
    fn inspect_reports_unavailable_store_without_breaking_shell_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_google_env();

        let store = FakeSecretStore::default();
        store.set_get_error(SecretStoreError::new(
            SecretStoreErrorKind::Unavailable,
            "secure storage backend is unavailable".to_string(),
        ));
        env::set_var("GOOGLE_ADS_DEVELOPER_TOKEN", "env-dev-token");
        env::set_var("GOOGLE_ADS_CLIENT_ID", "env-client-id");
        env::set_var("GOOGLE_ADS_CLIENT_SECRET", "env-client-secret");
        env::set_var("GOOGLE_ADS_REFRESH_TOKEN", "env-refresh-token");

        let auth = google_inspect_auth(&store);

        assert!(!auth.credential_store_available);
        assert_eq!(auth.developer_token.source, GoogleSecretSource::ShellEnv);
        assert_eq!(
            auth.credential_store_error.as_deref(),
            Some("secure storage backend is unavailable")
        );

        clear_google_env();
    }

    #[test]
    fn normalizes_customer_id_variants() {
        assert_eq!(
            normalize_google_customer_id("123-456-7890").unwrap(),
            "1234567890"
        );
        assert!(normalize_google_customer_id("abc").is_err());
    }
}
