use serde::{Deserialize, Serialize};

use crate::secret_store::{
    SecretStore, SecretStoreError, SecretStoreErrorKind, AUTH_BUNDLE_ACCOUNT, AUTH_BUNDLE_SERVICE,
};

pub const AUTH_BUNDLE_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthBundle {
    #[serde(default = "auth_bundle_version")]
    pub version: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaAuthBundle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google: Option<GoogleAuthBundle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiktok: Option<TikTokAuthBundle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinterest: Option<PinterestAuthBundle>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetaAuthBundle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoogleAuthBundle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TikTokAuthBundle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PinterestAuthBundle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

impl Default for AuthBundle {
    fn default() -> Self {
        Self {
            version: AUTH_BUNDLE_VERSION,
            meta: None,
            google: None,
            tiktok: None,
            pinterest: None,
        }
    }
}

impl AuthBundle {
    pub fn is_empty(&self) -> bool {
        self.meta.is_none()
            && self.google.is_none()
            && self.tiktok.is_none()
            && self.pinterest.is_none()
    }

    pub fn normalize(&mut self) {
        self.version = AUTH_BUNDLE_VERSION;

        normalize_section(&mut self.meta);
        normalize_section(&mut self.google);
        normalize_section(&mut self.tiktok);
        normalize_section(&mut self.pinterest);
    }
}

impl MetaAuthBundle {
    fn normalize(&mut self) {
        normalize_secret(&mut self.access_token);
    }

    fn is_empty(&self) -> bool {
        self.access_token.is_none()
    }
}

impl GoogleAuthBundle {
    fn normalize(&mut self) {
        normalize_secret(&mut self.developer_token);
        normalize_secret(&mut self.client_id);
        normalize_secret(&mut self.client_secret);
        normalize_secret(&mut self.refresh_token);
    }

    fn is_empty(&self) -> bool {
        self.developer_token.is_none()
            && self.client_id.is_none()
            && self.client_secret.is_none()
            && self.refresh_token.is_none()
    }
}

impl TikTokAuthBundle {
    fn normalize(&mut self) {
        normalize_secret(&mut self.app_id);
        normalize_secret(&mut self.app_secret);
        normalize_secret(&mut self.access_token);
        normalize_secret(&mut self.refresh_token);
    }

    fn is_empty(&self) -> bool {
        self.app_id.is_none()
            && self.app_secret.is_none()
            && self.access_token.is_none()
            && self.refresh_token.is_none()
    }
}

impl PinterestAuthBundle {
    fn normalize(&mut self) {
        normalize_secret(&mut self.app_id);
        normalize_secret(&mut self.app_secret);
        normalize_secret(&mut self.access_token);
        normalize_secret(&mut self.refresh_token);
    }

    fn is_empty(&self) -> bool {
        self.app_id.is_none()
            && self.app_secret.is_none()
            && self.access_token.is_none()
            && self.refresh_token.is_none()
    }
}

pub fn load_auth_bundle(secret_store: &dyn SecretStore) -> Result<AuthBundle, SecretStoreError> {
    match secret_store.get_secret(AUTH_BUNDLE_SERVICE, AUTH_BUNDLE_ACCOUNT) {
        Ok(Some(raw_bundle)) => deserialize_auth_bundle(&raw_bundle),
        Ok(None) => Ok(AuthBundle::default()),
        Err(error) => Err(error),
    }
}

pub fn store_auth_bundle(
    secret_store: &dyn SecretStore,
    bundle: &AuthBundle,
) -> Result<(), SecretStoreError> {
    let mut normalized = bundle.clone();
    normalized.normalize();

    if normalized.is_empty() {
        secret_store
            .delete_secret(AUTH_BUNDLE_SERVICE, AUTH_BUNDLE_ACCOUNT)
            .map(|_| ())
    } else {
        let serialized = serde_json::to_string(&normalized).map_err(|error| {
            SecretStoreError::new(
                SecretStoreErrorKind::Failure,
                format!("failed to serialize auth bundle: {error}"),
            )
        })?;

        secret_store.set_secret(AUTH_BUNDLE_SERVICE, AUTH_BUNDLE_ACCOUNT, &serialized)
    }
}

fn auth_bundle_version() -> u8 {
    AUTH_BUNDLE_VERSION
}

fn deserialize_auth_bundle(raw_bundle: &str) -> Result<AuthBundle, SecretStoreError> {
    let mut bundle: AuthBundle = serde_json::from_str(raw_bundle).map_err(|error| {
        SecretStoreError::new(
            SecretStoreErrorKind::Failure,
            format!("failed to deserialize stored auth bundle: {error}"),
        )
    })?;

    if bundle.version != AUTH_BUNDLE_VERSION {
        return Err(SecretStoreError::new(
            SecretStoreErrorKind::Failure,
            format!("unsupported stored auth bundle version {}", bundle.version),
        ));
    }

    bundle.normalize();
    Ok(bundle)
}

fn normalize_secret(value: &mut Option<String>) {
    if let Some(secret) = value.as_mut() {
        let trimmed = secret.trim().to_string();
        if trimmed.is_empty() {
            *value = None;
        } else {
            *secret = trimmed;
        }
    }
}

fn normalize_section<T>(section: &mut Option<T>)
where
    T: BundleSection,
{
    if let Some(section_value) = section.as_mut() {
        section_value.normalize();
        if section_value.is_empty() {
            *section = None;
        }
    }
}

trait BundleSection {
    fn normalize(&mut self);
    fn is_empty(&self) -> bool;
}

impl BundleSection for MetaAuthBundle {
    fn normalize(&mut self) {
        MetaAuthBundle::normalize(self);
    }

    fn is_empty(&self) -> bool {
        MetaAuthBundle::is_empty(self)
    }
}

impl BundleSection for GoogleAuthBundle {
    fn normalize(&mut self) {
        GoogleAuthBundle::normalize(self);
    }

    fn is_empty(&self) -> bool {
        GoogleAuthBundle::is_empty(self)
    }
}

impl BundleSection for TikTokAuthBundle {
    fn normalize(&mut self) {
        TikTokAuthBundle::normalize(self);
    }

    fn is_empty(&self) -> bool {
        TikTokAuthBundle::is_empty(self)
    }
}

impl BundleSection for PinterestAuthBundle {
    fn normalize(&mut self) {
        PinterestAuthBundle::normalize(self);
    }

    fn is_empty(&self) -> bool {
        PinterestAuthBundle::is_empty(self)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::{
        load_auth_bundle, store_auth_bundle, AuthBundle, GoogleAuthBundle, MetaAuthBundle,
        PinterestAuthBundle, TikTokAuthBundle, AUTH_BUNDLE_VERSION,
    };
    use crate::secret_store::{
        SecretStore, SecretStoreError, SecretStoreErrorKind, AUTH_BUNDLE_ACCOUNT,
        AUTH_BUNDLE_SERVICE, META_ACCESS_TOKEN_ACCOUNT, META_ACCESS_TOKEN_SERVICE,
    };

    #[derive(Default)]
    struct FakeSecretStore {
        secrets: Mutex<HashMap<(String, String), String>>,
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

    #[test]
    fn stores_and_loads_auth_bundle() {
        let store = FakeSecretStore::default();
        let bundle = AuthBundle {
            version: AUTH_BUNDLE_VERSION,
            meta: Some(MetaAuthBundle {
                access_token: Some(" meta-token ".to_string()),
            }),
            google: Some(GoogleAuthBundle {
                developer_token: Some("dev-token".to_string()),
                client_id: Some("client-id".to_string()),
                client_secret: Some("client-secret".to_string()),
                refresh_token: Some("refresh-token".to_string()),
            }),
            tiktok: Some(TikTokAuthBundle {
                app_id: Some("app-id".to_string()),
                app_secret: Some("app-secret".to_string()),
                access_token: Some("access-token".to_string()),
                refresh_token: Some("refresh-token".to_string()),
            }),
            pinterest: Some(PinterestAuthBundle {
                app_id: Some("pin-app-id".to_string()),
                app_secret: Some("pin-app-secret".to_string()),
                access_token: Some("pin-access-token".to_string()),
                refresh_token: Some("pin-refresh-token".to_string()),
            }),
        };

        store_auth_bundle(&store, &bundle).unwrap();
        let loaded = load_auth_bundle(&store).unwrap();

        assert_eq!(
            loaded.meta.unwrap().access_token.as_deref(),
            Some("meta-token")
        );
        assert_eq!(
            loaded.google.unwrap().developer_token.as_deref(),
            Some("dev-token")
        );
        assert_eq!(loaded.tiktok.unwrap().app_id.as_deref(), Some("app-id"));
        assert_eq!(
            loaded.pinterest.unwrap().access_token.as_deref(),
            Some("pin-access-token")
        );
    }

    #[test]
    fn storing_empty_bundle_deletes_keychain_entry() {
        let store = FakeSecretStore::default();
        store
            .set_secret(
                AUTH_BUNDLE_SERVICE,
                AUTH_BUNDLE_ACCOUNT,
                "{\"version\":1,\"meta\":{\"access_token\":\"token\"}}",
            )
            .unwrap();

        store_auth_bundle(&store, &AuthBundle::default()).unwrap();

        assert_eq!(
            store
                .get_secret(AUTH_BUNDLE_SERVICE, AUTH_BUNDLE_ACCOUNT)
                .unwrap(),
            None
        );
    }

    #[test]
    fn invalid_bundle_payload_reports_failure() {
        let store = FakeSecretStore::default();
        store
            .set_secret(AUTH_BUNDLE_SERVICE, AUTH_BUNDLE_ACCOUNT, "{not-json}")
            .unwrap();

        let error = load_auth_bundle(&store).unwrap_err();

        assert_eq!(error.kind(), SecretStoreErrorKind::Failure);
        assert!(error.to_string().contains("deserialize"));
    }

    #[test]
    fn legacy_per_secret_entries_are_ignored() {
        let store = FakeSecretStore::default();
        store
            .set_secret(
                META_ACCESS_TOKEN_SERVICE,
                META_ACCESS_TOKEN_ACCOUNT,
                "legacy-meta-token",
            )
            .unwrap();

        let bundle = load_auth_bundle(&store).unwrap();

        assert_eq!(bundle, AuthBundle::default());
    }
}
