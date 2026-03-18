pub mod auth;
pub mod client;
pub mod config;
pub mod endpoints;
pub mod error;
pub mod ids;
pub mod output;
pub mod secret_store;
pub mod tiktok_auth;
pub mod tiktok_client;
pub mod tiktok_config;
pub mod tiktok_endpoints;
pub mod tiktok_error;

pub use client::{GraphClient, GraphResponse, Paging, PagingCursors};
pub use config::{
    inspect, inspect_access_token, AccessTokenSource, AccessTokenStatus, ConfigOverrides,
    ConfigSnapshot, ResolvedConfig, DEFAULT_API_BASE_URL, DEFAULT_API_VERSION, DEFAULT_CONFIG_FILE,
};
pub use error::{GraphApiError, MetaAdsError, Result};
pub use output::{OutputEnvelope, OutputFormat};
pub use secret_store::{
    OsKeyringStore, SecretStore, SecretStoreError, SecretStoreErrorKind, META_ACCESS_TOKEN_ACCOUNT,
    META_ACCESS_TOKEN_SERVICE, TIKTOK_ACCESS_TOKEN_ACCOUNT, TIKTOK_ACCESS_TOKEN_SERVICE,
    TIKTOK_REFRESH_TOKEN_ACCOUNT, TIKTOK_REFRESH_TOKEN_SERVICE,
};

pub use tiktok_client::{TikTokClient, TikTokPageInfo, TikTokResponse};
pub use tiktok_config::{
    tiktok_inspect, tiktok_inspect_access_token, TikTokAccessTokenSource, TikTokAccessTokenStatus,
    TikTokConfigOverrides, TikTokConfigSnapshot, TikTokResolvedConfig, TIKTOK_DEFAULT_API_BASE_URL,
    TIKTOK_DEFAULT_API_VERSION,
};
pub use tiktok_error::{TikTokApiError, TikTokError, TikTokResult};
