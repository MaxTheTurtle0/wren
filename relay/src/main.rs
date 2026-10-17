//! `wren-relay [--listen 127.0.0.1:8787]`
//!
//! Serve it over HTTPS on a public hostname (Caddy, nginx, a Cloudflare
//! tunnel, a cloud load balancer); Graph only posts to `https://` URLs.
//! See docs/realtime.md.

use std::net::SocketAddr;

#[path = "../../src/notices.rs"]
mod notices;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut args = std::env::args().skip(1);
    let mut listen: SocketAddr = std::env::var("WREN_RELAY_LISTEN")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 8787)));
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--licenses" => {
                print!("{}", notices::TEXT);
                return;
            }
            "--listen" => match args.next().and_then(|value| value.parse().ok()) {
                Some(address) => listen = address,
                None => {
                    eprintln!("--listen needs an address such as 0.0.0.0:8787");
                    std::process::exit(2);
                }
            },
            "--help" | "-h" => {
                println!(
                    "wren-relay [--listen 127.0.0.1:8787] [--licenses]\nForwards Microsoft Graph change notifications to Wren. Put it behind HTTPS."
                );
                return;
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }
    let listener = match tokio::net::TcpListener::bind(listen).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("could not listen on {listen}: {error}");
            std::process::exit(1);
        }
    };
    log::info!("wren-relay listening on {listen}");
    let app = wren_relay::router(wren_relay::Relay::default());
    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    if let Err(error) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
    {
        eprintln!("wren-relay stopped: {error}");
        std::process::exit(1);
    }
}
