//! App updates use the shared fastframe installer and never Graph credentials.

use fastframe_update::{MacConfig, MacTarget, ReqwestTransport, UpdateConfig, Updater};

pub const CONFIG: UpdateConfig = UpdateConfig {
    macos: MacConfig {
        bundle_ids: &["app.wren.Wren"],
        executable_names: &[],
        legacy_bundle_names: &[],
    },
    mac_target: MacTarget::Arm64Only,
    prereleases: fastframe_update::Prereleases::WhenRunningPrerelease,
    publisher_key: Some(include_str!("../packaging/update-public-key.hex")),
    ..UpdateConfig::new(
        "MaxTheTurtle0/wren",
        "Wren",
        "wren",
        env!("CARGO_PKG_VERSION"),
    )
};

#[derive(Debug, Default)]
pub enum Status {
    #[default]
    Idle,
    Checking,
    Current,
    Available {
        release: fastframe_update::Release,
        reason: Option<String>,
    },
    Downloading {
        received: u64,
        total: u64,
    },
    Ready(Box<fastframe_update::Prepared>),
    Installing,
    Failed(&'static str),
}

impl Status {
    pub fn busy(&self) -> bool {
        matches!(
            self,
            Self::Checking | Self::Downloading { .. } | Self::Installing
        )
    }
}

#[derive(Debug)]
pub enum Event {
    Status(Status),
    Quit,
}

pub fn updater() -> anyhow::Result<Updater> {
    Ok(Updater::new(
        CONFIG,
        ReqwestTransport::new(reqwest::blocking::Client::builder())?,
    ))
}

pub fn due(last: i64, now: i64) -> bool {
    last == 0 || now.saturating_sub(last) >= fastframe_update::CHECK_INTERVAL.as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_and_daily_check_schedule() {
        CONFIG.validate().unwrap();
        assert!(CONFIG.publisher_key.is_some());
        assert!(due(0, 1));
        assert!(!due(1000, 1001));
        assert!(!due(1000, 500));
        assert!(due(1000, 1000 + 86_400_000));
    }

    #[tokio::test]
    async fn a_newer_release_is_discovered_without_credentials() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("GET")).and(path("/latest.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tag_name": "v0.1.1", "html_url": "https://github.com/MaxTheTurtle0/wren/releases/tag/v0.1.1"
            }))).expect(1).mount(&server).await;
        let uri = server.uri();
        let release = tokio::task::spawn_blocking(move || {
            Updater::new(
                UpdateConfig {
                    current_version: "0.1.0",
                    ..CONFIG
                },
                ReqwestTransport::new(reqwest::blocking::Client::builder()).unwrap(),
            )
            .with_source(fastframe_update::Source::local(&uri).unwrap())
            .check()
            .unwrap()
            .unwrap()
        })
        .await
        .unwrap();
        assert_eq!(release.version, "0.1.1");
        let requests = server.received_requests().await.unwrap();
        assert!(!requests[0].headers.contains_key("authorization"));
    }
}
