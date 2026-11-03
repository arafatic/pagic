//! In-app update checker for Pagic.
//!
//! Queries the GitHub Release API asynchronously in the background and notifies
//! the user when a newer version is available.

use std::sync::mpsc::{Receiver, channel};

/// GitHub release information returned by the releases API.
#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
pub struct ReleaseResponse {
    pub tag_name: String,
    pub html_url: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

/// Structured details for an available update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateDetails {
    pub version: String,
    pub url: String,
    pub notes: String,
}

/// The outcome of an update check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateOutcome {
    Available(UpdateDetails),
    UpToDate,
    Error(String),
}

/// Checks whether `remote_tag` is newer than `current_version`.
pub fn is_newer_version(remote_tag: &str, current_version: &str) -> bool {
    let clean_remote = remote_tag.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let clean_current = current_version.trim().trim_start_matches(|c| c == 'v' || c == 'V');

    if let (Ok(remote_sem), Ok(cur_sem)) = (semver::Version::parse(clean_remote), semver::Version::parse(clean_current)) {
        return remote_sem > cur_sem;
    }

    // Fallback comparison for non-strict semver strings
    let parse_parts = |s: &str| -> Vec<u64> {
        s.split(|c: char| !c.is_ascii_digit()).filter(|part| !part.is_empty()).filter_map(|part| part.parse::<u64>().ok()).collect()
    };

    let rem_parts = parse_parts(clean_remote);
    let cur_parts = parse_parts(clean_current);

    rem_parts > cur_parts
}

/// Fetches the latest release info from the GitHub API synchronously.
#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_latest_release() -> Result<ReleaseResponse, String> {
    let url = pagic_engine::identity::UPDATE_API.ok_or("No application update source is configured for this build")?;
    let tls = ureq::tls::TlsConfig::builder().root_certs(ureq::tls::RootCerts::PlatformVerifier).build();

    let mut builder = ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .user_agent(concat!("Pagic/", env!("CARGO_PKG_VERSION")));

    if let Some(proxy) = ureq::Proxy::try_from_env() {
        builder = builder.proxy(Some(proxy));
    }

    let agent = builder.build().new_agent();

    let mut resp = agent.get(url).header("Accept", "application/vnd.github+json").call().map_err(|e| format!("{e}"))?;

    let status = resp.status().as_u16();
    if status != 200 {
        return Err(format!("GitHub API returned HTTP status {status}"));
    }

    let body = resp.body_mut().read_to_string().map_err(|e| format!("{e}"))?;
    serde_json::from_str::<ReleaseResponse>(&body).map_err(|e| format!("invalid release payload: {e}"))
}

/// An in-flight background update check job.
pub struct UpdateJob {
    pub manual: bool,
    pub receiver: Receiver<UpdateOutcome>,
}

impl UpdateJob {
    /// Launches an asynchronous background update check.
    pub fn start(manual: bool) -> Self {
        let (sender, receiver) = channel();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = std::thread::Builder::new().name("pagic-updater".into()).spawn(move || {
                let outcome = match fetch_latest_release() {
                    Ok(release) => {
                        let current = env!("CARGO_PKG_VERSION");
                        if is_newer_version(&release.tag_name, current) {
                            let version = release.tag_name.trim_start_matches(|c| c == 'v' || c == 'V').to_string();
                            let notes = release.body.unwrap_or_default().trim().to_string();
                            let url = release.html_url;
                            UpdateOutcome::Available(UpdateDetails { version, url, notes })
                        } else {
                            UpdateOutcome::UpToDate
                        }
                    }
                    Err(e) => UpdateOutcome::Error(e),
                };
                let _ = sender.send(outcome);
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = sender.send(UpdateOutcome::UpToDate);
        }

        Self { manual, receiver }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconfigured_source_returns_before_any_network_request() {
        assert_eq!(fetch_latest_release().unwrap_err(), "No application update source is configured for this build");
    }
}
