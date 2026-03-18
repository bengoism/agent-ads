use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{MetaAdsError, Result};
use crate::output::OutputFormat;

pub const DEFAULT_CONFIG_FILE: &str = "agent-ads.config.json";
pub const DEFAULT_ENV_FILE: &str = ".env";
pub const DEFAULT_API_BASE_URL: &str = "https://graph.facebook.com";
pub const DEFAULT_API_VERSION: &str = "v25.0";
const DEFAULT_TIMEOUT_SECONDS: u64 = 60;

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
    pub app_secret: Option<String>,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_business_id: Option<String>,
    pub default_account_id: Option<String>,
    pub output_format: OutputFormat,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnvFileSource {
    Auto,
    Explicit,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct EnvFileState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file_path: Option<PathBuf>,
    pub env_file_exists: bool,
    pub env_file_loaded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file_source: Option<EnvFileSource>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigSnapshot {
    pub config_path: PathBuf,
    pub config_file_exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file_path: Option<PathBuf>,
    pub env_file_exists: bool,
    pub env_file_loaded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file_source: Option<EnvFileSource>,
    pub access_token_present: bool,
    pub app_secret_present: bool,
    pub api_base_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub default_business_id: Option<String>,
    pub default_account_id: Option<String>,
    pub output_format: OutputFormat,
}

impl ResolvedConfig {
    pub fn load(
        config_path: Option<&Path>,
        env_file_state: &EnvFileState,
        overrides: &ConfigOverrides,
    ) -> Result<Self> {
        let snapshot = inspect(config_path, env_file_state, overrides)?;
        let access_token = env::var("META_ADS_ACCESS_TOKEN").map_err(|_| {
            MetaAdsError::Config(
                "META_ADS_ACCESS_TOKEN must be set in the shell or a loaded .env file; secrets are not read from config files"
                    .to_string(),
            )
        })?;

        Ok(Self {
            access_token,
            app_secret: if snapshot.app_secret_present {
                env::var("META_ADS_APP_SECRET").ok()
            } else {
                None
            },
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

pub fn load_env(env_file: Option<&Path>) -> Result<EnvFileState> {
    let cwd = env::current_dir()?;
    load_env_with_cwd(env_file, &cwd)
}

pub fn inspect(
    config_path: Option<&Path>,
    env_file_state: &EnvFileState,
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
        env_file_path: env_file_state.env_file_path.clone(),
        env_file_exists: env_file_state.env_file_exists,
        env_file_loaded: env_file_state.env_file_loaded,
        env_file_source: env_file_state.env_file_source.clone(),
        access_token_present: env::var("META_ADS_ACCESS_TOKEN").is_ok(),
        app_secret_present: env::var("META_ADS_APP_SECRET").is_ok(),
        api_base_url,
        api_version,
        timeout_seconds,
        default_business_id,
        default_account_id,
        output_format,
    })
}

fn load_env_with_cwd(env_file: Option<&Path>, cwd: &Path) -> Result<EnvFileState> {
    let (path, source) = match env_file {
        Some(path) => (path.to_path_buf(), EnvFileSource::Explicit),
        None => (cwd.join(DEFAULT_ENV_FILE), EnvFileSource::Auto),
    };

    if !path.exists() {
        if source == EnvFileSource::Explicit {
            return Err(MetaAdsError::Config(format!(
                "env file not found at {}",
                path.display()
            )));
        }

        return Ok(EnvFileState {
            env_file_path: Some(path),
            env_file_exists: false,
            env_file_loaded: false,
            env_file_source: Some(source),
        });
    }

    let entries = dotenvy::from_path_iter(&path).map_err(|error| {
        MetaAdsError::Config(format!(
            "failed to read env file {}: {error}",
            path.display()
        ))
    })?;

    for entry in entries {
        let (key, value) = entry.map_err(|error| {
            MetaAdsError::Config(format!(
                "failed to parse env file {}: {error}",
                path.display()
            ))
        })?;
        if env::var_os(&key).is_none() {
            env::set_var(&key, &value);
        }
    }

    Ok(EnvFileState {
        env_file_path: Some(path),
        env_file_exists: true,
        env_file_loaded: true,
        env_file_source: Some(source),
    })
}

fn load_file_config(path: &Path) -> Result<FileConfig> {
    if !path.exists() {
        return Ok(FileConfig::default());
    }

    let contents = fs::read_to_string(path)?;
    let config = serde_json::from_str::<RootFileConfig>(&contents)?;
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
    use std::env;
    use std::fs;
    use std::sync::{LazyLock, Mutex};

    use tempfile::tempdir;

    use super::{
        load_env_with_cwd, ConfigOverrides, EnvFileSource, ResolvedConfig, DEFAULT_ENV_FILE,
    };
    use crate::output::OutputFormat;

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn clear_meta_env() {
        for key in [
            "META_ADS_ACCESS_TOKEN",
            "META_ADS_APP_SECRET",
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

        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-ads.config.json");
        fs::write(
            &path,
            r#"{"output_format":"csv","providers":{"meta":{"api_version":"v24.0","timeout_seconds":10}}}"#,
        )
        .unwrap();

        env::set_var("META_ADS_ACCESS_TOKEN", "token");
        env::set_var("META_ADS_API_VERSION", "v25.0");

        let env_state = load_env_with_cwd(None, dir.path()).unwrap();
        let config = ResolvedConfig::load(
            Some(&path),
            &env_state,
            &ConfigOverrides {
                timeout_seconds: Some(30),
                ..ConfigOverrides::default()
            },
        )
        .unwrap();

        assert_eq!(config.api_version, "v25.0");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.output_format, OutputFormat::Csv);

        clear_meta_env();
    }

    #[test]
    fn auto_env_file_is_optional() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_meta_env();

        let dir = tempdir().unwrap();
        let env_state = load_env_with_cwd(None, dir.path()).unwrap();

        assert_eq!(
            env_state.env_file_path,
            Some(dir.path().join(DEFAULT_ENV_FILE))
        );
        assert!(!env_state.env_file_exists);
        assert!(!env_state.env_file_loaded);
        assert_eq!(env_state.env_file_source, Some(EnvFileSource::Auto));
    }

    #[test]
    fn auto_env_file_loads_without_overriding_shell_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_meta_env();

        let dir = tempdir().unwrap();
        let config_path = dir.path().join("agent-ads.config.json");
        let env_path = dir.path().join(DEFAULT_ENV_FILE);

        fs::write(
            &config_path,
            r#"{"providers":{"meta":{"api_version":"v23.0"}}}"#,
        )
        .unwrap();
        fs::write(
            &env_path,
            "META_ADS_ACCESS_TOKEN=file_token\nMETA_ADS_API_VERSION=v24.0\n",
        )
        .unwrap();

        env::set_var("META_ADS_ACCESS_TOKEN", "shell_token");

        let env_state = load_env_with_cwd(None, dir.path()).unwrap();
        let config =
            ResolvedConfig::load(Some(&config_path), &env_state, &ConfigOverrides::default())
                .unwrap();

        assert_eq!(config.access_token, "shell_token");
        assert_eq!(config.api_version, "v24.0");
        assert!(env_state.env_file_loaded);

        clear_meta_env();
    }

    #[test]
    fn explicit_env_file_overrides_config_when_shell_env_is_absent() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_meta_env();

        let dir = tempdir().unwrap();
        let config_path = dir.path().join("agent-ads.config.json");
        let env_path = dir.path().join("custom.env");

        fs::write(
            &config_path,
            r#"{"providers":{"meta":{"api_version":"v23.0","timeout_seconds":10}}}"#,
        )
        .unwrap();
        fs::write(
            &env_path,
            "META_ADS_ACCESS_TOKEN=file_token\nMETA_ADS_API_VERSION=v24.0\nMETA_ADS_TIMEOUT_SECONDS=45\n",
        )
        .unwrap();

        let env_state = load_env_with_cwd(Some(&env_path), dir.path()).unwrap();
        let config =
            ResolvedConfig::load(Some(&config_path), &env_state, &ConfigOverrides::default())
                .unwrap();

        assert_eq!(config.access_token, "file_token");
        assert_eq!(config.api_version, "v24.0");
        assert_eq!(config.timeout_seconds, 45);
        assert_eq!(env_state.env_file_source, Some(EnvFileSource::Explicit));

        clear_meta_env();
    }

    #[test]
    fn explicit_missing_env_file_errors() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_meta_env();

        let dir = tempdir().unwrap();
        let error =
            load_env_with_cwd(Some(&dir.path().join("missing.env")), dir.path()).unwrap_err();

        assert!(error.to_string().contains("env file not found"));
    }
}
