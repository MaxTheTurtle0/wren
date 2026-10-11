//! Requests to the Entra ID token and device-code endpoints.
//!
//! Errors carry only fixed descriptions and the AADSTS code: the endpoint's
//! own text can echo request data, and nothing token-shaped may reach a log.

use std::time::Duration;

use serde::Deserialize;

use super::AuthConfig;

#[derive(Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    /// The scopes actually granted, space-separated.
    #[serde(default)]
    pub scope: Option<String>,
}

impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the tokens themselves.
        f.debug_struct("TokenResponse")
            .field("expires_in", &self.expires_in)
            .field("scope", &self.scope)
            .field("has_refresh_token", &self.refresh_token.is_some())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TokenError {
    /// The grant is gone (expired, revoked, password changed, policy). Only
    /// a new sign-in can mint one.
    #[error("{0}")]
    Rejected(String),
    /// Consent or admin approval is missing for a scope.
    #[error("{0}")]
    ConsentRequired(String),
    /// The endpoint could not answer; the grant may still be good.
    #[error("could not reach Microsoft's sign-in service: {0}")]
    Unreachable(String),
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    error: String,
    #[serde(default)]
    error_description: String,
    #[serde(default)]
    error_codes: Vec<u64>,
}

fn classify(status: reqwest::StatusCode, body: &str) -> TokenError {
    let parsed: Option<ErrorBody> = serde_json::from_str(body).ok();
    let Some(parsed) = parsed else {
        return TokenError::Unreachable(format!("unexpected answer ({status})"));
    };
    let code = parsed
        .error_codes
        .first()
        .map(|c| format!("AADSTS{c}"))
        .or_else(|| {
            parsed
                .error_description
                .split_whitespace()
                .find(|w| w.starts_with("AADSTS"))
                .map(|w| w.trim_end_matches(':').to_owned())
        });
    let suffix = code
        .as_deref()
        .map(|c| format!(" ({c})"))
        .unwrap_or_default();
    match (parsed.error.as_str(), code.as_deref()) {
        (_, Some("AADSTS65001" | "AADSTS90094" | "AADSTS90095")) | ("consent_required", _) => {
            TokenError::ConsentRequired(format!(
                "Consent is needed for Wren's permissions; your administrator may need to approve them{suffix}."
            ))
        }
        (_, Some("AADSTS7000218")) => TokenError::Rejected(format!(
            "The app registration does not allow public client flows. Enable 'Allow public client flows' in its Authentication settings{suffix}."
        )),
        (_, Some("AADSTS700016")) => TokenError::Rejected(format!(
            "The application (client) ID was not found. Check Wren's app registration settings{suffix}."
        )),
        ("invalid_grant", _) => TokenError::Rejected(format!(
            "Your Microsoft session expired. Sign in again{suffix}."
        )),
        ("invalid_client" | "unauthorized_client", _) => TokenError::Rejected(format!(
            "Microsoft did not accept Wren's app registration{suffix}."
        )),
        _ if status.is_server_error() => {
            TokenError::Unreachable(format!("server error ({status})"))
        }
        _ => TokenError::Rejected(format!("Microsoft refused the sign-in{suffix}.")),
    }
}

async fn post_form(
    http: &reqwest::Client,
    url: &str,
    form: &[(&str, &str)],
) -> Result<(reqwest::StatusCode, String), TokenError> {
    let response = http
        .post(url)
        .form(form)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|error| TokenError::Unreachable(transport_reason(&error)))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| TokenError::Unreachable(transport_reason(&error)))?;
    Ok((status, text))
}

/// A transport failure in words, without the URL (which may carry query
/// parameters).
pub fn transport_reason(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "the request timed out".into()
    } else if error.is_connect() {
        "no connection".into()
    } else if error.is_decode() {
        "an unreadable answer".into()
    } else {
        "a network error".into()
    }
}

async fn token_request(
    http: &reqwest::Client,
    config: &AuthConfig,
    form: &[(&str, &str)],
) -> Result<TokenResponse, TokenError> {
    let (status, text) = post_form(http, &config.token_url(), form).await?;
    if !status.is_success() {
        return Err(classify(status, &text));
    }
    serde_json::from_str(&text)
        .map_err(|_| TokenError::Unreachable("an unreadable token answer".into()))
}

pub async fn exchange_code(
    http: &reqwest::Client,
    config: &AuthConfig,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
    scopes: &str,
) -> Result<TokenResponse, TokenError> {
    token_request(
        http,
        config,
        &[
            ("client_id", config.client_id.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", verifier),
            ("scope", scopes),
        ],
    )
    .await
}

pub async fn refresh(
    http: &reqwest::Client,
    config: &AuthConfig,
    refresh_token: &str,
    scopes: &str,
) -> Result<TokenResponse, TokenError> {
    token_request(
        http,
        config,
        &[
            ("client_id", config.client_id.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", scopes),
        ],
    )
    .await
}

/// The device code flow's first answer: what to show the person.
#[derive(Clone, Debug, Deserialize)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    5
}

pub async fn start_device_code(
    http: &reqwest::Client,
    config: &AuthConfig,
    scopes: &str,
) -> Result<DeviceCode, TokenError> {
    let (status, text) = post_form(
        http,
        &config.device_code_url(),
        &[("client_id", config.client_id.as_str()), ("scope", scopes)],
    )
    .await?;
    if !status.is_success() {
        return Err(classify(status, &text));
    }
    serde_json::from_str(&text)
        .map_err(|_| TokenError::Unreachable("an unreadable device code answer".into()))
}

/// Polls until the person finishes on another device, the code expires, or
/// `cancel` turns true.
pub async fn poll_device_code(
    http: &reqwest::Client,
    config: &AuthConfig,
    code: &DeviceCode,
    mut cancel: tokio::sync::watch::Receiver<bool>,
) -> Result<TokenResponse, TokenError> {
    let mut interval = Duration::from_secs(code.interval.max(1));
    let deadline = tokio::time::Instant::now() + Duration::from_secs(code.expires_in.max(60));
    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            changed = cancel.changed() => {
                if changed.is_err() || *cancel.borrow() {
                    return Err(TokenError::Rejected("Sign-in cancelled.".into()));
                }
                continue;
            }
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(TokenError::Rejected(
                "The sign-in code expired. Try again.".into(),
            ));
        }
        let (status, text) = post_form(
            http,
            &config.token_url(),
            &[
                ("client_id", config.client_id.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", code.device_code.as_str()),
            ],
        )
        .await?;
        if status.is_success() {
            return serde_json::from_str(&text)
                .map_err(|_| TokenError::Unreachable("an unreadable token answer".into()));
        }
        let error = serde_json::from_str::<ErrorBody>(&text)
            .map(|b| b.error)
            .unwrap_or_default();
        match error.as_str() {
            "authorization_pending" => {}
            "slow_down" => interval += Duration::from_secs(5),
            "authorization_declined" => {
                return Err(TokenError::Rejected("Sign-in was declined.".into()));
            }
            "expired_token" => {
                return Err(TokenError::Rejected(
                    "The sign-in code expired. Try again.".into(),
                ));
            }
            _ => return Err(classify(status, &text)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn config(server: &MockServer) -> AuthConfig {
        AuthConfig {
            client_id: "client".into(),
            tenant: "organizations".into(),
            authority: server.uri(),
        }
    }

    #[tokio::test]
    async fn code_exchange_sends_the_verifier_and_reads_tokens() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/organizations/oauth2/v2.0/token"))
            .and(body_string_contains("grant_type=authorization_code"))
            .and(body_string_contains("code_verifier=verifier"))
            .and(body_string_contains(
                "redirect_uri=http%3A%2F%2Flocalhost%3A1234",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "at", "refresh_token": "rt", "expires_in": 3599,
                "scope": "Chat.ReadWrite User.Read"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let tokens = exchange_code(
            &http,
            &config(&server),
            "code",
            "verifier",
            "http://localhost:1234",
            "User.Read",
        )
        .await
        .unwrap();
        assert_eq!(tokens.access_token, "at");
        assert_eq!(tokens.refresh_token.as_deref(), Some("rt"));
        assert!(!format!("{tokens:?}").contains("at\""));
    }

    #[tokio::test]
    async fn rejected_refreshes_are_told_apart_from_outages() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_string_contains("refresh_token=expired"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "AADSTS70043: The refresh token has expired. Trace ID: secret-trace",
                "error_codes": [70043]
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(body_string_contains("refresh_token=flaky"))
            .respond_with(ResponseTemplate::new(503).set_body_string("<html>down</html>"))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let rejected = refresh(&http, &config(&server), "expired", "User.Read")
            .await
            .unwrap_err();
        assert!(
            matches!(&rejected, TokenError::Rejected(text) if text.contains("AADSTS70043") && !text.contains("secret-trace"))
        );
        let outage = refresh(&http, &config(&server), "flaky", "User.Read")
            .await
            .unwrap_err();
        assert!(matches!(outage, TokenError::Unreachable(_)));
    }

    #[tokio::test]
    async fn admin_consent_is_recognised() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant", "error_codes": [65001], "error_description": "AADSTS65001: no consent"
            })))
            .mount(&server)
            .await;
        let error = refresh(&reqwest::Client::new(), &config(&server), "x", "s")
            .await
            .unwrap_err();
        assert!(matches!(error, TokenError::ConsentRequired(_)));
    }

    #[tokio::test]
    async fn device_code_polls_until_granted() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/organizations/oauth2/v2.0/token"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(serde_json::json!({"error": "authorization_pending"})),
            )
            .up_to_n_times(2)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/organizations/oauth2/v2.0/token"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({"access_token": "at", "refresh_token": "rt"}),
                ),
            )
            .mount(&server)
            .await;
        let code = DeviceCode {
            device_code: "dc".into(),
            user_code: "ABCD".into(),
            verification_uri: "https://microsoft.com/devicelogin".into(),
            expires_in: 900,
            interval: 1,
        };
        let (_cancel, cancelled) = tokio::sync::watch::channel(false);
        let tokens = poll_device_code(&reqwest::Client::new(), &config(&server), &code, cancelled)
            .await
            .unwrap();
        assert_eq!(tokens.access_token, "at");
    }
}
