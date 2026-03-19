use keyring::{Entry, Error as KeyringError};

pub const META_ACCESS_TOKEN_SERVICE: &str = "agent-ads";
pub const META_ACCESS_TOKEN_ACCOUNT: &str = "meta-access-token";

pub const GOOGLE_ADS_DEVELOPER_TOKEN_SERVICE: &str = "agent-ads";
pub const GOOGLE_ADS_DEVELOPER_TOKEN_ACCOUNT: &str = "google-ads-developer-token";
pub const GOOGLE_ADS_CLIENT_ID_SERVICE: &str = "agent-ads";
pub const GOOGLE_ADS_CLIENT_ID_ACCOUNT: &str = "google-ads-client-id";
pub const GOOGLE_ADS_CLIENT_SECRET_SERVICE: &str = "agent-ads";
pub const GOOGLE_ADS_CLIENT_SECRET_ACCOUNT: &str = "google-ads-client-secret";
pub const GOOGLE_ADS_REFRESH_TOKEN_SERVICE: &str = "agent-ads";
pub const GOOGLE_ADS_REFRESH_TOKEN_ACCOUNT: &str = "google-ads-refresh-token";

pub const TIKTOK_ACCESS_TOKEN_SERVICE: &str = "agent-ads";
pub const TIKTOK_ACCESS_TOKEN_ACCOUNT: &str = "tiktok-access-token";
pub const TIKTOK_REFRESH_TOKEN_SERVICE: &str = "agent-ads";
pub const TIKTOK_REFRESH_TOKEN_ACCOUNT: &str = "tiktok-refresh-token";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretStoreErrorKind {
    Unavailable,
    Failure,
}

#[derive(Debug, Clone)]
pub struct SecretStoreError {
    kind: SecretStoreErrorKind,
    message: String,
}

impl SecretStoreError {
    pub fn new(kind: SecretStoreErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    pub fn kind(&self) -> SecretStoreErrorKind {
        self.kind
    }
}

impl std::fmt::Display for SecretStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SecretStoreError {}

pub trait SecretStore {
    fn get_secret(
        &self,
        service: &str,
        account: &str,
    ) -> std::result::Result<Option<String>, SecretStoreError>;

    fn set_secret(
        &self,
        service: &str,
        account: &str,
        secret: &str,
    ) -> std::result::Result<(), SecretStoreError>;

    fn delete_secret(
        &self,
        service: &str,
        account: &str,
    ) -> std::result::Result<bool, SecretStoreError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OsKeyringStore;

impl SecretStore for OsKeyringStore {
    fn get_secret(
        &self,
        service: &str,
        account: &str,
    ) -> std::result::Result<Option<String>, SecretStoreError> {
        let entry = Entry::new(service, account).map_err(map_keyring_error)?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(map_keyring_error(error)),
        }
    }

    fn set_secret(
        &self,
        service: &str,
        account: &str,
        secret: &str,
    ) -> std::result::Result<(), SecretStoreError> {
        let entry = Entry::new(service, account).map_err(map_keyring_error)?;
        entry.set_password(secret).map_err(map_keyring_error)
    }

    fn delete_secret(
        &self,
        service: &str,
        account: &str,
    ) -> std::result::Result<bool, SecretStoreError> {
        let entry = Entry::new(service, account).map_err(map_keyring_error)?;
        match entry.delete_credential() {
            Ok(()) => Ok(true),
            Err(KeyringError::NoEntry) => Ok(false),
            Err(error) => Err(map_keyring_error(error)),
        }
    }
}

fn map_keyring_error(error: KeyringError) -> SecretStoreError {
    let kind = match error {
        KeyringError::NoStorageAccess(_) | KeyringError::PlatformFailure(_) => {
            SecretStoreErrorKind::Unavailable
        }
        _ => SecretStoreErrorKind::Failure,
    };

    SecretStoreError::new(kind, error.to_string())
}
