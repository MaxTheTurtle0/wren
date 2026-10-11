//! Microsoft Entra ID sign-in: the OAuth 2.0 authorization code flow with
//! PKCE for a public client, and the device code flow as a fallback.
//!
//! Wren never sees a password. The system browser signs in with Microsoft
//! and sends a one-time code back to a listener on the loopback interface
//! (`loopback`); the code and the PKCE verifier become tokens at the token
//! endpoint (`token`). `session::TokenSource` keeps the access token fresh
//! and stores each rotated refresh token in the OS credential store.
//!
//! Permissions are delegated and minimal; see `docs/permissions.md`.

pub mod loopback;
pub mod session;
pub mod token;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// The scopes asked for at sign-in. Each one is consentable by users in a
/// tenant with default settings; see docs/permissions.md for why each is
/// needed.
pub const SIGN_IN_SCOPES: &[&str] = &[
    "openid",
    "profile",
    "offline_access",
    "User.Read",
    "User.ReadBasic.All",
    "Chat.ReadWrite",
    "ChatMessage.Send",
    "Team.ReadBasic.All",
    "Channel.ReadBasic.All",
    "ChannelMessage.Send",
    "Presence.Read.All",
    "Calendars.Read",
];

/// Reading channel posts. Requires tenant admin consent, so Wren asks for it
/// separately (incremental consent) when someone opens a channel.
pub const CHANNEL_READ_SCOPE: &str = "ChannelMessage.Read.All";

/// Starting new chats. User-consentable; asked for on first use.
pub const CHAT_CREATE_SCOPE: &str = "Chat.Create";

/// Uploading files to the person's OneDrive and sharing them in messages.
/// User-consentable; asked for on first use.
pub const FILES_SCOPE: &str = "Files.ReadWrite";

/// Setting the person's own availability and status message.
/// User-consentable; asked for on first use.
pub const PRESENCE_WRITE_SCOPE: &str = "Presence.ReadWrite";

/// The scopes Wren may ask for beyond sign-in, each when a feature first
/// needs it (incremental consent).
pub const OPTIONAL_SCOPES: &[&str] = &[
    CHANNEL_READ_SCOPE,
    CHAT_CREATE_SCOPE,
    FILES_SCOPE,
    PRESENCE_WRITE_SCOPE,
];

/// Why a feature needs an optional scope, for the consent prompt.
pub fn scope_reason(scope: &str) -> &'static str {
    match scope {
        CHANNEL_READ_SCOPE => {
            "Reading channel posts. Your administrator must approve this permission."
        }
        CHAT_CREATE_SCOPE => "Starting new chats with people in your organisation.",
        FILES_SCOPE => {
            "Uploading files to your OneDrive ('Microsoft Teams Chat Files') and sharing them with the people in the conversation."
        }
        PRESENCE_WRITE_SCOPE => "Setting your availability and status message.",
        _ => "This feature.",
    }
}

pub const DEFAULT_AUTHORITY: &str = "https://login.microsoftonline.com";

/// Which application signs in, where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthConfig {
    /// The Entra application (client) ID.
    pub client_id: String,
    /// `organizations`, `common`, or a tenant ID or domain.
    pub tenant: String,
    /// `https://login.microsoftonline.com`, or a test server.
    pub authority: String,
}

impl AuthConfig {
    pub fn new(client_id: &str, tenant: &str) -> Self {
        Self {
            client_id: client_id.trim().to_owned(),
            tenant: match tenant.trim() {
                "" => "organizations".to_owned(),
                other => other.to_owned(),
            },
            authority: DEFAULT_AUTHORITY.to_owned(),
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}/oauth2/v2.0/{path}",
            self.authority.trim_end_matches('/'),
            urlencoding::encode(&self.tenant)
        )
    }

    pub fn authorize_url(&self) -> String {
        self.endpoint("authorize")
    }

    pub fn token_url(&self) -> String {
        self.endpoint("token")
    }

    pub fn device_code_url(&self) -> String {
        self.endpoint("devicecode")
    }
}

/// The scopes for a request: sign-in's plus any extra ones granted or asked
/// for, without duplicates, in a stable order.
pub fn scope_string(extra: &[String]) -> String {
    let mut scopes: Vec<String> = SIGN_IN_SCOPES.iter().map(|s| (*s).to_owned()).collect();
    for scope in extra {
        if !scopes.iter().any(|s| s.eq_ignore_ascii_case(scope)) {
            scopes.push(scope.clone());
        }
    }
    scopes.join(" ")
}

/// Whether the space-separated `granted` list holds `scope`. The token
/// endpoint answers with short names (`Chat.ReadWrite`) or full ones
/// (`https://graph.microsoft.com/Chat.ReadWrite`).
pub fn has_scope(granted: &str, scope: &str) -> bool {
    granted.split_whitespace().any(|item| {
        let short = item.rsplit('/').next().unwrap_or(item);
        short.eq_ignore_ascii_case(scope)
    })
}

/// A started browser sign-in: the URL to open and what finishing it needs.
#[derive(Clone, Debug)]
pub struct BrowserFlow {
    pub url: String,
    pub verifier: String,
    pub state: String,
    pub redirect_uri: String,
    pub scopes: String,
}

pub fn random_token(bytes: usize) -> String {
    let mut buffer = vec![0u8; bytes];
    // getrandom only fails when the OS has no entropy source at all.
    if getrandom::fill(&mut buffer).is_err() {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        buffer = Sha256::digest(seed.to_le_bytes()).to_vec();
        buffer.truncate(bytes);
    }
    URL_SAFE_NO_PAD.encode(buffer)
}

/// The S256 challenge for a PKCE verifier (RFC 7636).
pub fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Builds the authorize request for a loopback redirect on `port`.
///
/// `login_hint` pre-fills the account picker when re-authenticating an
/// account whose session expired.
pub fn begin_browser(
    config: &AuthConfig,
    port: u16,
    extra_scopes: &[String],
    login_hint: Option<&str>,
    prompt_consent: bool,
) -> BrowserFlow {
    // RFC 7636: 43 to 128 characters. 48 random bytes give 64.
    let verifier = random_token(48);
    let state = random_token(18);
    let redirect_uri = format!("http://localhost:{port}");
    let scopes = scope_string(extra_scopes);
    let mut url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        config.authorize_url(),
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&scopes),
        urlencoding::encode(&state),
        challenge(&verifier),
    );
    if let Some(hint) = login_hint {
        url.push_str("&login_hint=");
        url.push_str(&urlencoding::encode(hint));
    }
    if prompt_consent {
        url.push_str("&prompt=consent");
    } else {
        url.push_str("&prompt=select_account");
    }
    BrowserFlow {
        url,
        verifier,
        state,
        redirect_uri,
        scopes,
    }
}

/// The identity claims of an ID token. The token comes straight from the
/// token endpoint over TLS, so its signature is not checked (OpenID Connect
/// Core 3.1.3.7); the claims only name the account.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize)]
pub struct IdClaims {
    #[serde(default)]
    pub oid: Option<String>,
    #[serde(default)]
    pub tid: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
}

impl IdClaims {
    pub fn from_jwt(token: &str) -> Option<Self> {
        let payload = token.split('.').nth(1)?;
        let bytes = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// The account's id, as MSAL names a home account.
    pub fn account_id(&self) -> Option<String> {
        Some(format!("{}.{}", self.oid.as_deref()?, self.tid.as_deref()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_rfc_7636_example() {
        // Appendix B of RFC 7636.
        assert_eq!(
            challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn verifiers_are_long_enough_and_url_safe() {
        let flow = begin_browser(&AuthConfig::new("id", ""), 51234, &[], None, false);
        assert!((43..=128).contains(&flow.verifier.len()));
        assert!(
            flow.verifier
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert_ne!(
            flow.verifier,
            begin_browser(&AuthConfig::new("id", ""), 1, &[], None, false).verifier
        );
    }

    #[test]
    fn authorize_url_carries_pkce_and_loopback_redirect() {
        let config = AuthConfig::new("6731de76-14a6-49ae-97bc-6eba6914391e", "organizations");
        let flow = begin_browser(
            &config,
            51234,
            &["ChannelMessage.Read.All".into()],
            Some("ada@contoso.com"),
            false,
        );
        assert!(
            flow.url.starts_with(
                "https://login.microsoftonline.com/organizations/oauth2/v2.0/authorize?"
            )
        );
        assert!(flow.url.contains("code_challenge_method=S256"));
        assert!(
            flow.url
                .contains(&format!("code_challenge={}", challenge(&flow.verifier)))
        );
        assert!(
            flow.url
                .contains("redirect_uri=http%3A%2F%2Flocalhost%3A51234")
        );
        assert!(flow.url.contains("offline_access"));
        assert!(flow.url.contains("ChannelMessage.Read.All"));
        assert!(flow.url.contains("login_hint=ada%40contoso.com"));
        assert!(!flow.url.contains(&flow.verifier));
    }

    #[test]
    fn scopes_are_deduplicated_and_matched_in_either_form() {
        let scopes = scope_string(&["chat.readwrite".into(), "ChannelMessage.Read.All".into()]);
        assert_eq!(scopes.matches("Chat.ReadWrite").count(), 1);
        assert!(scopes.ends_with("ChannelMessage.Read.All"));
        assert!(has_scope(
            "openid https://graph.microsoft.com/Chat.ReadWrite",
            "Chat.ReadWrite"
        ));
        assert!(!has_scope("Chat.Read", "Chat.ReadWrite"));
    }

    #[test]
    fn id_claims_name_the_account() {
        let payload = URL_SAFE_NO_PAD.encode(
            br#"{"oid":"o","tid":"t","name":"Ada","preferred_username":"ada@contoso.com"}"#,
        );
        let claims = IdClaims::from_jwt(&format!("header.{payload}.signature")).unwrap();
        assert_eq!(claims.account_id().as_deref(), Some("o.t"));
        assert_eq!(IdClaims::from_jwt("garbage"), None);
    }
}
