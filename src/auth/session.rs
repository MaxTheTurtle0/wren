//! A signed-in account's tokens.
//!
//! The access token stays in memory and is refreshed five minutes before it
//! expires, or at once after Graph answers 401. Entra rotates refresh
//! tokens: each new one replaces the stored one in the OS credential store.
//! One refresh runs at a time; callers waiting on it share its result.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::token::{self, TokenError, TokenResponse};
use super::{AuthConfig, IdClaims};
use crate::secrets::{SecretStore, refresh_key};

const REFRESH_MARGIN: Duration = Duration::from_secs(5 * 60);

/// What Wren remembers about an account outside the credential store. No
/// secrets: the refresh token lives in the OS store under the account's id.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: String,
    pub display_name: String,
    pub username: String,
    pub tenant_id: String,
    /// The scopes granted at the last token answer, space-separated.
    pub granted: String,
    pub client_id: String,
    pub tenant: String,
}

impl AccountRecord {
    pub fn load(path: &std::path::Path) -> Option<Self> {
        serde_json::from_slice(&std::fs::read(path).ok()?).ok()
    }

    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        crate::paths::write_atomically(path, &json)
    }
}

struct Tokens {
    access: Option<(String, Instant)>,
    refresh: Option<String>,
    granted: String,
}

pub struct Session {
    pub account: String,
    pub config: AuthConfig,
    http: reqwest::Client,
    secrets: Arc<dyn SecretStore>,
    tokens: tokio::sync::Mutex<Tokens>,
    /// The granted scopes, readable without awaiting the token lock.
    granted: Mutex<String>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("account", &self.account)
            .finish_non_exhaustive()
    }
}

impl Session {
    /// A session from a fresh token answer (sign-in). Stores the refresh
    /// token.
    pub async fn from_sign_in(
        config: AuthConfig,
        http: reqwest::Client,
        secrets: Arc<dyn SecretStore>,
        response: TokenResponse,
    ) -> Result<(Self, IdClaims), TokenError> {
        let claims = response
            .id_token
            .as_deref()
            .and_then(IdClaims::from_jwt)
            .ok_or_else(|| {
                TokenError::Rejected("Microsoft did not say which account signed in.".into())
            })?;
        let account = claims.account_id().ok_or_else(|| {
            TokenError::Rejected("Microsoft did not say which account signed in.".into())
        })?;
        let granted = response.scope.clone().unwrap_or_default();
        let session = Self {
            account,
            config,
            http,
            secrets,
            tokens: tokio::sync::Mutex::new(Tokens {
                access: None,
                refresh: None,
                granted: granted.clone(),
            }),
            granted: Mutex::new(granted),
        };
        session
            .accept(&mut *session.tokens.lock().await, response)
            .await?;
        Ok((session, claims))
    }

    /// A session restored from the credential store at start. `None` when
    /// no refresh token is stored (signed out, or the store was cleared).
    pub async fn restore(
        record: &AccountRecord,
        config: AuthConfig,
        http: reqwest::Client,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<Option<Self>, crate::secrets::SecretError> {
        let key = refresh_key(&record.id);
        let store = Arc::clone(&secrets);
        let refresh = tokio::task::spawn_blocking(move || store.read(&key))
            .await
            .map_err(|_| crate::secrets::SecretError::Unavailable)??;
        let Some(refresh) = refresh else {
            return Ok(None);
        };
        Ok(Some(Self {
            account: record.id.clone(),
            config,
            http,
            secrets,
            tokens: tokio::sync::Mutex::new(Tokens {
                access: None,
                refresh: Some(refresh),
                granted: record.granted.clone(),
            }),
            granted: Mutex::new(record.granted.clone()),
        }))
    }

    pub fn granted(&self) -> String {
        self.granted
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        super::has_scope(&self.granted(), scope)
    }

    /// A valid access token, refreshing first when it is missing or about
    /// to expire.
    pub async fn access_token(&self) -> Result<String, TokenError> {
        let mut tokens = self.tokens.lock().await;
        if let Some((token, expires)) = &tokens.access
            && Instant::now() + REFRESH_MARGIN < *expires
        {
            return Ok(token.clone());
        }
        self.refresh_locked(&mut tokens).await
    }

    /// Refreshes now, after Graph refused the current token. A token that
    /// changed while this caller waited is used as it is.
    pub async fn force_refresh(&self, rejected: &str) -> Result<String, TokenError> {
        let mut tokens = self.tokens.lock().await;
        if let Some((token, expires)) = &tokens.access
            && token != rejected
            && Instant::now() < *expires
        {
            return Ok(token.clone());
        }
        self.refresh_locked(&mut tokens).await
    }

    async fn refresh_locked(&self, tokens: &mut Tokens) -> Result<String, TokenError> {
        let Some(refresh) = tokens.refresh.clone() else {
            return Err(TokenError::Rejected("Sign in again.".into()));
        };
        let scopes = refresh_scopes(&tokens.granted);
        let response = token::refresh(&self.http, &self.config, &refresh, &scopes).await?;
        self.accept(tokens, response).await
    }

    async fn accept(
        &self,
        tokens: &mut Tokens,
        response: TokenResponse,
    ) -> Result<String, TokenError> {
        let lifetime =
            Duration::from_secs(response.expires_in.unwrap_or(3600).clamp(60, 24 * 3600));
        tokens.access = Some((response.access_token.clone(), Instant::now() + lifetime));
        if let Some(scope) = response.scope.as_ref().filter(|s| !s.trim().is_empty()) {
            tokens.granted = scope.clone();
            *self.granted.lock().unwrap_or_else(|p| p.into_inner()) = scope.clone();
        }
        if let Some(refresh) = response.refresh_token
            && tokens.refresh.as_deref() != Some(refresh.as_str())
        {
            tokens.refresh = Some(refresh.clone());
            let key = refresh_key(&self.account);
            let store = Arc::clone(&self.secrets);
            match tokio::task::spawn_blocking(move || store.write(&key, &refresh)).await {
                Ok(Ok(())) => {}
                // The session still works for this run; the next start asks
                // to sign in again.
                Ok(Err(error)) => log::warn!("could not store the refresh token: {error}"),
                Err(_) => log::warn!("could not store the refresh token"),
            }
        }
        Ok(response.access_token)
    }

    /// Forgets the tokens here and in the credential store.
    pub async fn sign_out(&self) {
        let mut tokens = self.tokens.lock().await;
        tokens.access = None;
        tokens.refresh = None;
        let key = refresh_key(&self.account);
        let store = Arc::clone(&self.secrets);
        if let Ok(Err(error)) = tokio::task::spawn_blocking(move || store.delete(&key)).await {
            log::warn!("could not remove the stored refresh token: {error}");
        }
    }
}

/// The scopes a refresh asks for: what was granted, plus the OpenID ones the
/// token endpoint leaves out of its answer.
fn refresh_scopes(granted: &str) -> String {
    let mut scopes: Vec<String> = granted
        .split_whitespace()
        .filter(|s| !matches!(*s, "openid" | "profile" | "offline_access" | "email"))
        .map(str::to_owned)
        .collect();
    if scopes.is_empty() {
        return super::scope_string(&[]);
    }
    scopes.extend(["openid", "profile", "offline_access"].map(String::from));
    scopes.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::MemoryStore;
    use base64::Engine;
    use wiremock::matchers::{body_string_contains, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn id_token() -> String {
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(br#"{"oid":"oid1","tid":"tid1","name":"Ada"}"#);
        format!("h.{payload}.s")
    }

    #[tokio::test]
    async fn sign_in_stores_and_rotates_the_refresh_token() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_string_contains("refresh_token=rt1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "at2", "refresh_token": "rt2", "expires_in": 3600,
                "scope": "User.Read Chat.ReadWrite"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let secrets: Arc<dyn SecretStore> = Arc::new(MemoryStore::default());
        let config = AuthConfig {
            authority: server.uri(),
            ..AuthConfig::new("client", "organizations")
        };
        let (session, claims) = Session::from_sign_in(
            config,
            reqwest::Client::new(),
            Arc::clone(&secrets),
            TokenResponse {
                access_token: "at1".into(),
                refresh_token: Some("rt1".into()),
                id_token: Some(id_token()),
                // Short-lived, so the next call refreshes.
                expires_in: Some(60),
                scope: Some("User.Read".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(claims.name.as_deref(), Some("Ada"));
        assert_eq!(session.account, "oid1.tid1");
        assert_eq!(
            secrets.read(&refresh_key("oid1.tid1")).unwrap().as_deref(),
            Some("rt1")
        );
        assert_eq!(session.access_token().await.unwrap(), "at2");
        assert_eq!(
            secrets.read(&refresh_key("oid1.tid1")).unwrap().as_deref(),
            Some("rt2")
        );
        assert!(session.has_scope("Chat.ReadWrite"));
        session.sign_out().await;
        assert_eq!(secrets.read(&refresh_key("oid1.tid1")).unwrap(), None);
        assert!(session.access_token().await.is_err());
    }

    #[test]
    fn refresh_asks_for_what_was_granted() {
        let scopes = refresh_scopes("User.Read Chat.ReadWrite openid profile email");
        assert_eq!(
            scopes,
            "User.Read Chat.ReadWrite openid profile offline_access"
        );
        assert!(refresh_scopes("").contains("Chat.ReadWrite"));
    }
}
