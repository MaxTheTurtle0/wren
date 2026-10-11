//! The Microsoft Graph client.
//!
//! Every request carries the session's access token and nothing else of the
//! person's. The client:
//!
//! - sends the token only to the Graph origin it was built with, so a
//!   `nextLink` or an image URL from message content can never leak it;
//! - refreshes the token once and retries after a 401;
//! - honours `Retry-After` on 429, 503 and 504, pausing every request until
//!   then, and retries up to three times with backoff;
//! - reports errors as [`GraphError`], with Graph's error code but never
//!   its message text or URLs, which can quote request data.
//!
//! Endpoints live in `api.rs`; JSON shapes in `types.rs`; translation into
//! the model in `convert.rs`.

pub mod api;
pub mod convert;
pub mod files;
pub mod people;
pub mod types;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::auth::session::Session;
use crate::auth::token::{TokenError, transport_reason};

pub const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

const MAX_ATTEMPTS: u32 = 4;
const MAX_RETRY_AFTER: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    #[error("{0}")]
    Auth(#[from] TokenError),
    /// 403: the scope is missing or a policy refuses.
    #[error("Microsoft Graph refused access ({code})")]
    Forbidden { code: String },
    #[error("not found")]
    NotFound,
    #[error("Microsoft Graph is throttling requests; retry in {retry_after:?}")]
    Throttled { retry_after: Duration },
    #[error("Microsoft Graph answered {status} ({code})")]
    Http { status: u16, code: String },
    #[error("could not reach Microsoft Graph: {0}")]
    Network(String),
    #[error("Microsoft Graph sent an unreadable answer")]
    Decode,
    #[error("refused to send credentials to another host")]
    ForeignUrl,
}

impl GraphError {
    pub fn is_forbidden(&self) -> bool {
        matches!(self, Self::Forbidden { .. })
    }
}

/// What a request carries.
#[derive(Clone, Copy)]
enum Body<'a> {
    Json(&'a serde_json::Value),
    Bytes(&'a [u8]),
}

pub struct Graph {
    http: reqwest::Client,
    base: String,
    origin: String,
    session: Arc<Session>,
    /// Requests wait until this instant after a throttling answer.
    paused_until: Mutex<Option<tokio::time::Instant>>,
}

impl Graph {
    pub fn new(http: reqwest::Client, session: Arc<Session>) -> Self {
        Self::with_base(http, session, GRAPH_BASE)
    }

    /// A client for another base URL: national clouds, or a test server.
    pub fn with_base(http: reqwest::Client, session: Arc<Session>, base: &str) -> Self {
        let base = base.trim_end_matches('/').to_owned();
        let origin = origin_of(&base).unwrap_or_default();
        Self {
            http,
            base,
            origin,
            session,
            paused_until: Mutex::new(None),
        }
    }

    pub fn session(&self) -> &Arc<Session> {
        &self.session
    }

    /// A path below the base, or a full URL on the Graph origin.
    fn url(&self, path_or_url: &str) -> Result<String, GraphError> {
        if path_or_url.starts_with("https://") || path_or_url.starts_with("http://") {
            if origin_of(path_or_url).as_deref() == Some(self.origin.as_str()) {
                return Ok(path_or_url.to_owned());
            }
            return Err(GraphError::ForeignUrl);
        }
        Ok(format!(
            "{}/{}",
            self.base,
            path_or_url.trim_start_matches('/')
        ))
    }

    pub fn is_graph_url(&self, url: &str) -> bool {
        origin_of(url).as_deref() == Some(self.origin.as_str())
    }

    async fn wait_if_paused(&self) {
        let until = *self.paused_until.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(until) = until
            && until > tokio::time::Instant::now()
        {
            tokio::time::sleep_until(until).await;
        }
    }

    fn pause(&self, delay: Duration) {
        let until = tokio::time::Instant::now() + delay;
        let mut paused = self.paused_until.lock().unwrap_or_else(|p| p.into_inner());
        if paused.is_none_or(|current| current < until) {
            *paused = Some(until);
        }
    }

    async fn send(
        &self,
        method: Method,
        path_or_url: &str,
        body: Option<Body<'_>>,
        headers: &[(&str, &str)],
    ) -> Result<reqwest::Response, GraphError> {
        let url = self.url(path_or_url)?;
        let mut token = self.session.access_token().await?;
        let mut refreshed = false;
        let mut attempt = 0;
        loop {
            attempt += 1;
            self.wait_if_paused().await;
            let mut request = self
                .http
                .request(method.clone(), &url)
                .bearer_auth(&token)
                .timeout(Duration::from_secs(45));
            for (name, value) in headers {
                request = request.header(*name, *value);
            }
            match body {
                Some(Body::Json(json)) => request = request.json(json),
                Some(Body::Bytes(bytes)) => {
                    request = request
                        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
                        .body(bytes.to_vec())
                }
                None => {}
            }
            let response = match request.send().await {
                Ok(response) => response,
                Err(error)
                    if attempt < MAX_ATTEMPTS && (error.is_timeout() || error.is_connect()) =>
                {
                    tokio::time::sleep(backoff(attempt)).await;
                    continue;
                }
                Err(error) => return Err(GraphError::Network(transport_reason(&error))),
            };
            let status = response.status();
            if status.is_success() {
                return Ok(response);
            }
            match status {
                StatusCode::UNAUTHORIZED if !refreshed => {
                    refreshed = true;
                    token = self.session.force_refresh(&token).await?;
                    continue;
                }
                StatusCode::TOO_MANY_REQUESTS
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT => {
                    let delay = retry_after(&response).unwrap_or_else(|| backoff(attempt));
                    let delay = delay.min(MAX_RETRY_AFTER);
                    log::info!(
                        "Graph answered {} for {}; waiting {:?}",
                        status.as_u16(),
                        method,
                        delay
                    );
                    self.pause(delay);
                    if attempt >= MAX_ATTEMPTS {
                        return Err(GraphError::Throttled { retry_after: delay });
                    }
                    continue;
                }
                _ => {}
            }
            let code = error_code(response).await;
            return Err(match status {
                StatusCode::UNAUTHORIZED => GraphError::Auth(TokenError::Rejected(
                    "Microsoft Graph did not accept the session. Sign in again.".into(),
                )),
                StatusCode::FORBIDDEN => GraphError::Forbidden { code },
                StatusCode::NOT_FOUND => GraphError::NotFound,
                _ => GraphError::Http {
                    status: status.as_u16(),
                    code,
                },
            });
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, GraphError> {
        self.get_with(path, &[]).await
    }

    pub async fn get_with<T: DeserializeOwned>(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<T, GraphError> {
        let response = self.send(Method::GET, path, None, headers).await?;
        decode(response).await
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, GraphError> {
        let body = serde_json::to_value(body).map_err(|_| GraphError::Decode)?;
        let response = self
            .send(Method::POST, path, Some(Body::Json(&body)), &[])
            .await?;
        decode(response).await
    }

    /// A POST whose answer has no body Wren needs (204, or ignored).
    pub async fn post_no_content<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), GraphError> {
        let body = serde_json::to_value(body).map_err(|_| GraphError::Decode)?;
        self.send(Method::POST, path, Some(Body::Json(&body)), &[])
            .await
            .map(drop)
    }

    pub async fn patch<B: Serialize>(&self, path: &str, body: &B) -> Result<(), GraphError> {
        let body = serde_json::to_value(body).map_err(|_| GraphError::Decode)?;
        self.send(Method::PATCH, path, Some(Body::Json(&body)), &[])
            .await
            .map(drop)
    }

    /// `PUT` raw bytes (small file uploads) and decode the answer.
    pub async fn put_bytes<T: DeserializeOwned>(
        &self,
        path: &str,
        bytes: &[u8],
    ) -> Result<T, GraphError> {
        let response = self
            .send(Method::PUT, path, Some(Body::Bytes(bytes)), &[])
            .await?;
        decode(response).await
    }

    /// The HTTP client without credentials, for pre-authenticated upload
    /// URLs (which must not receive the access token).
    pub fn plain_http(&self) -> &reqwest::Client {
        &self.http
    }

    pub async fn delete(&self, path: &str) -> Result<(), GraphError> {
        self.send(Method::DELETE, path, None, &[]).await.map(drop)
    }

    /// Raw bytes (photos, inline images), at most `limit` bytes.
    pub async fn get_bytes(&self, path_or_url: &str, limit: usize) -> Result<Vec<u8>, GraphError> {
        let response = self.send(Method::GET, path_or_url, None, &[]).await?;
        if response
            .content_length()
            .is_some_and(|len| len as usize > limit)
        {
            return Err(GraphError::Decode);
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|error| GraphError::Network(transport_reason(&error)))?;
        if bytes.len() > limit {
            return Err(GraphError::Decode);
        }
        Ok(bytes.to_vec())
    }
}

async fn decode<T: DeserializeOwned>(response: reqwest::Response) -> Result<T, GraphError> {
    let bytes = response
        .bytes()
        .await
        .map_err(|error| GraphError::Network(transport_reason(&error)))?;
    // A 204 or empty body decodes as JSON null, for callers expecting ().
    let bytes: &[u8] = if bytes.is_empty() { b"null" } else { &bytes };
    serde_json::from_slice(bytes).map_err(|error| {
        log::debug!("undecodable Graph answer: {error}");
        GraphError::Decode
    })
}

async fn error_code(response: reqwest::Response) -> String {
    let body = response.text().await.unwrap_or_default();
    serde_json::from_str::<types::ErrorBody>(&body)
        .ok()
        .and_then(|b| b.error)
        .and_then(|e| e.code)
        .filter(|code| {
            code.len() < 80
                && code
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        })
        .unwrap_or_else(|| "unknown".into())
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    let value = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?;
    value.trim().parse::<u64>().ok().map(Duration::from_secs)
}

fn backoff(attempt: u32) -> Duration {
    Duration::from_secs(1u64 << attempt.min(6))
}

/// `scheme://host[:port]` of a URL.
fn origin_of(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    Some(format!(
        "{}://{}",
        scheme.to_ascii_lowercase(),
        authority.to_ascii_lowercase()
    ))
}

/// Percent-encodes one path segment (chat and message ids contain `:` and
/// `@`, which Graph accepts as they are; anything else unusual is encoded).
pub fn segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b':' | b'@' => {
                out.push(byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod testing {
    use std::sync::Arc;

    use base64::Engine;

    use crate::auth::AuthConfig;
    use crate::auth::session::Session;
    use crate::auth::token::TokenResponse;
    use crate::secrets::MemoryStore;

    /// A session holding `access`, refreshing against `authority`.
    pub async fn session(access: &str, authority: &str) -> Arc<Session> {
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(br#"{"oid":"u-me","tid":"t1","name":"Me"}"#);
        let config = AuthConfig {
            authority: authority.to_owned(),
            ..AuthConfig::new("client", "organizations")
        };
        let (session, _) = Session::from_sign_in(
            config,
            reqwest::Client::new(),
            Arc::new(MemoryStore::default()),
            TokenResponse {
                access_token: access.into(),
                refresh_token: Some("rt".into()),
                id_token: Some(format!("h.{payload}.s")),
                expires_in: Some(3600),
                scope: Some("User.Read Chat.ReadWrite ChatMessage.Send".into()),
            },
        )
        .await
        .unwrap();
        Arc::new(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn origins_and_segments() {
        assert_eq!(
            origin_of("https://graph.microsoft.com/v1.0/me").as_deref(),
            Some("https://graph.microsoft.com")
        );
        assert_eq!(origin_of("https://evil@graph.microsoft.com/x"), None);
        assert_eq!(segment("19:abc@thread.v2"), "19:abc@thread.v2");
        assert_eq!(segment("a/b?c#d"), "a%2Fb%3Fc%23d");
    }

    #[tokio::test]
    async fn tokens_never_go_to_other_hosts() {
        let server = MockServer::start().await;
        let graph = Graph::with_base(
            reqwest::Client::new(),
            testing::session("at", &server.uri()).await,
            &server.uri(),
        );
        let result = graph
            .get_bytes("https://attacker.example/steal", 1024)
            .await;
        assert_eq!(result, Err(GraphError::ForeignUrl));
        assert!(graph.is_graph_url(&format!("{}/chats", server.uri())));
    }

    #[tokio::test]
    async fn a_401_refreshes_once_and_retries() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/me"))
            .and(header("authorization", "Bearer stale"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/organizations/oauth2/v2.0/token"))
            .and(body_string_contains("grant_type=refresh_token"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({"access_token": "fresh", "expires_in": 3600}),
                ),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/me"))
            .and(header("authorization", "Bearer fresh"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": "u-me", "displayName": "Me"})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let graph = Graph::with_base(
            reqwest::Client::new(),
            testing::session("stale", &server.uri()).await,
            &server.uri(),
        );
        let me: types::User = graph.get("me").await.unwrap();
        assert_eq!(me.display_name.as_deref(), Some("Me"));
    }

    #[tokio::test]
    async fn throttling_waits_for_retry_after() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/me/joinedTeams"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "1"))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/me/joinedTeams"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": []})),
            )
            .mount(&server)
            .await;
        let graph = Graph::with_base(
            reqwest::Client::new(),
            testing::session("at", &server.uri()).await,
            &server.uri(),
        );
        let started = std::time::Instant::now();
        let page: types::Page<types::Team> = graph.get("me/joinedTeams").await.unwrap();
        assert!(page.value.is_empty());
        assert!(started.elapsed() >= Duration::from_millis(900));
    }

    #[tokio::test]
    async fn forbidden_carries_the_code_only() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "error": {"code": "Forbidden", "message": "Missing scope ChannelMessage.Read.All for user ada@contoso.com"}
            })))
            .mount(&server)
            .await;
        let graph = Graph::with_base(
            reqwest::Client::new(),
            testing::session("at", &server.uri()).await,
            &server.uri(),
        );
        let error = graph
            .get::<serde_json::Value>("teams/t/channels/c/messages")
            .await
            .unwrap_err();
        assert_eq!(
            error,
            GraphError::Forbidden {
                code: "Forbidden".into()
            }
        );
        assert!(!error.to_string().contains("ada@"));
    }
}
