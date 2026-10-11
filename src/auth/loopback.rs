//! The loopback listener that receives the authorization code.
//!
//! Entra ID lets a desktop app register `http://localhost` and use any port
//! at run time. Wren binds a random free port on 127.0.0.1 (and the same
//! port on ::1 when it can, since browsers may resolve `localhost` to
//! either), opens the browser, and waits for one request carrying the code
//! and the state it sent. Anything else (a favicon, a stale tab) gets an
//! error page and the wait goes on.

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

const LOGIN_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Listeners bound to one port, ready before the browser opens.
pub struct Loopback {
    v4: TcpListener,
    v6: Option<TcpListener>,
    port: u16,
}

impl Loopback {
    pub async fn bind() -> Result<Self> {
        let v4 = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .context("could not listen on the loopback interface for the sign-in redirect")?;
        let port = v4.local_addr()?.port();
        let v6 = TcpListener::bind(SocketAddr::from((std::net::Ipv6Addr::LOCALHOST, port)))
            .await
            .ok();
        Ok(Self { v4, v6, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Waits for the redirect and returns the authorization code. Ends early
    /// when `cancel` turns true, or after ten minutes.
    pub async fn wait_for_code(
        &self,
        expected_state: &str,
        mut cancel: watch::Receiver<bool>,
    ) -> Result<String> {
        let deadline = tokio::time::sleep(LOGIN_TIMEOUT);
        tokio::pin!(deadline);
        loop {
            let stream = tokio::select! {
                accepted = self.v4.accept() => accepted.map(|(s, _)| s),
                accepted = accept_optional(self.v6.as_ref()) => accepted.map(|(s, _)| s),
                changed = cancel.changed() => {
                    if changed.is_err() || *cancel.borrow() {
                        bail!("sign-in cancelled");
                    }
                    continue;
                }
                _ = &mut deadline => bail!("sign-in timed out; try again"),
            };
            let Ok(stream) = stream else { continue };
            match answer(stream, expected_state).await {
                Ok(Some(code)) => return Ok(code),
                Ok(None) => continue,
                Err(error) => return Err(error),
            }
        }
    }
}

async fn accept_optional(
    listener: Option<&TcpListener>,
) -> std::io::Result<(TcpStream, SocketAddr)> {
    match listener {
        Some(listener) => listener.accept().await,
        None => std::future::pending().await,
    }
}

/// Reads one request, answers it, and returns the code if it carried one.
/// A refusal from Entra (`error=access_denied`, say) ends the sign-in.
async fn answer(mut stream: TcpStream, expected_state: &str) -> Result<Option<String>> {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    let read =
        tokio::time::timeout(Duration::from_secs(10), reader.read_line(&mut request_line)).await;
    if !matches!(read, Ok(Ok(n)) if n > 0) {
        return Ok(None);
    }
    let outcome = parse_request_line(&request_line, expected_state);
    let (status, body) = match &outcome {
        Redirect::Code(_) => (
            "200 OK",
            page(
                "You're signed in",
                "You can close this tab and return to Wren.",
            ),
        ),
        Redirect::Refused(reason) => ("400 Bad Request", page("Sign-in did not finish", reason)),
        Redirect::Ignored => (
            "404 Not Found",
            page("Nothing here", "This tab is not part of a Wren sign-in."),
        ),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
    match outcome {
        Redirect::Code(code) => Ok(Some(code)),
        Redirect::Refused(reason) => Err(anyhow!(reason)),
        Redirect::Ignored => Ok(None),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Redirect {
    Code(String),
    Refused(String),
    Ignored,
}

fn parse_request_line(line: &str, expected_state: &str) -> Redirect {
    let Some(target) = line.split_whitespace().nth(1) else {
        return Redirect::Ignored;
    };
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if path != "/" {
        return Redirect::Ignored;
    }
    let mut code = None;
    let mut state = None;
    let mut error = None;
    let mut description = None;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let value = urlencoding::decode(&value.replace('+', " "))
            .map(|v| v.into_owned())
            .unwrap_or_default();
        match key {
            "code" => code = Some(value),
            "state" => state = Some(value),
            "error" => error = Some(value),
            "error_description" => description = Some(value),
            _ => {}
        }
    }
    if state.as_deref() != Some(expected_state) {
        // Not our sign-in (or a forged request): never act on it.
        return Redirect::Ignored;
    }
    if let Some(error) = error {
        return Redirect::Refused(refusal_text(&error, description.as_deref()));
    }
    match code {
        Some(code) if !code.is_empty() => Redirect::Code(code),
        _ => Redirect::Ignored,
    }
}

/// Words for Entra's refusals. The AADSTS code is kept (it is what admins
/// search for); the rest of the description is not shown, since it can be
/// long and quote request data.
fn refusal_text(error: &str, description: Option<&str>) -> String {
    let code = description
        .and_then(|d| d.split_whitespace().find(|w| w.starts_with("AADSTS")))
        .map(|c| c.trim_end_matches(':').to_owned());
    let base = match (error, code.as_deref()) {
        (_, Some("AADSTS65001")) => "Wren has not been granted the permissions it asked for.".to_owned(),
        (_, Some("AADSTS90094" | "AADSTS90095")) => {
            "An administrator must approve Wren's permissions for your organisation.".to_owned()
        }
        (_, Some("AADSTS700016")) => {
            "The application (client) ID was not found in this tenant. Check Wren's app registration settings.".to_owned()
        }
        (_, Some("AADSTS50011")) => {
            "The app registration has no matching redirect URI. Add http://localhost under 'Mobile and desktop applications'.".to_owned()
        }
        ("access_denied", _) => "Sign-in was cancelled or declined.".to_owned(),
        ("consent_required", _) => "Consent is required for Wren's permissions.".to_owned(),
        _ => "Microsoft refused the sign-in.".to_owned(),
    };
    match code {
        Some(code) => format!("{base} ({code})"),
        None => base,
    }
}

fn page(title: &str, text: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Wren</title><style>body{{font:16px system-ui,sans-serif;display:grid;place-items:center;height:100vh;margin:0;background:#17181b;color:#e8e8ea}}main{{text-align:center}}h1{{font-size:20px;font-weight:600}}p{{color:#a4a6ad}}</style></head><body><main><h1>{}</h1><p>{}</p></main></body></html>",
        crate::html::escape(title),
        crate::html::escape(text)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_needs_the_right_state() {
        assert_eq!(
            parse_request_line("GET /?code=abc&state=s1 HTTP/1.1", "s1"),
            Redirect::Code("abc".into())
        );
        assert_eq!(
            parse_request_line("GET /?code=abc&state=other HTTP/1.1", "s1"),
            Redirect::Ignored
        );
        assert_eq!(
            parse_request_line("GET /favicon.ico HTTP/1.1", "s1"),
            Redirect::Ignored
        );
        assert_eq!(parse_request_line("garbage", "s1"), Redirect::Ignored);
    }

    #[test]
    fn refusals_keep_the_aadsts_code_only() {
        let line = "GET /?error=invalid_request&error_description=AADSTS50011%3A+The+redirect+URI+secret-stuff&state=s HTTP/1.1";
        let Redirect::Refused(text) = parse_request_line(line, "s") else {
            panic!("refusal expected");
        };
        assert!(text.contains("AADSTS50011"));
        assert!(!text.contains("secret-stuff"));
    }

    #[tokio::test]
    async fn a_browser_redirect_delivers_the_code() {
        let loopback = Loopback::bind().await.unwrap();
        let port = loopback.port();
        let (_cancel, cancelled) = watch::channel(false);
        let client = tokio::spawn(async move {
            // A stray request first, then the real redirect.
            for path in ["/favicon.ico", "/?code=the-code&state=st"] {
                let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
                stream
                    .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
                    .await
                    .unwrap();
                let mut response = String::new();
                tokio::io::AsyncReadExt::read_to_string(&mut stream, &mut response)
                    .await
                    .unwrap();
                assert!(response.starts_with("HTTP/1.1"));
            }
        });
        let code = loopback.wait_for_code("st", cancelled).await.unwrap();
        assert_eq!(code, "the-code");
        client.await.unwrap();
    }

    #[tokio::test]
    async fn cancelling_ends_the_wait() {
        let loopback = Loopback::bind().await.unwrap();
        let (cancel, cancelled) = watch::channel(false);
        cancel.send(true).unwrap();
        assert!(loopback.wait_for_code("st", cancelled).await.is_err());
    }
}
