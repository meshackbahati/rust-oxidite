/// OAuth2 client for making requests to authorization servers.
pub mod client;
/// Server-side OAuth2 provider implementation.
pub mod provider;
/// Preconfigured OAuth2 provider definitions (Google, GitHub, Microsoft).
pub mod providers;
/// OAuth2 grant types and data structures.
pub mod grants;

/// Re-exports from the `client` module.
pub use client::{OAuth2Client, OAuth2Config, generate_pkce};
/// Re-exports from the `provider` module.
pub use provider::{OAuth2Provider, AuthorizationRequest, TokenRequest, TokenResponse};
/// Re-export of `ProviderConfig`.
pub use providers::ProviderConfig;
/// Re-exports from the `grants` module.
pub use grants::{GrantType, AuthorizationCodeGrant, ClientCredentialsGrant};
