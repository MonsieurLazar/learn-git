use std::io::Read;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use semver::Version;
use serde::Deserialize;

use crate::version::target;

const API: &str = "https://api.github.com/repos/MonsieurLazar/learn-git/releases/latest";

#[derive(Debug, Clone)]
pub struct Release {
    pub version: Version,
    pub asset_name: String,
    pub asset_url: String,
    pub checksum_url: String,
}

pub enum Latest {
    NotModified,
    Found {
        release: Release,
        etag: Option<String>,
    },
}

#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn latest(etag: Option<&str>) -> Result<Latest> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(3)))
        .timeout_global(Some(Duration::from_secs(6)))
        .build()
        .into();

    let mut request = agent.get(API).header("user-agent", "learn-git-updater");
    if let Some(etag) = etag {
        request = request.header("if-none-match", etag);
    }

    let response = request.call()?;
    if response.status() == 304 {
        return Ok(Latest::NotModified);
    }

    let etag = response
        .headers()
        .get("etag")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    let mut body = String::new();
    response
        .into_body()
        .into_reader()
        .read_to_string(&mut body)?;
    let release: ApiRelease = serde_json::from_str(&body)?;

    if release.draft || release.prerelease {
        return Err(anyhow!("latest release is not stable"));
    }

    let version = Version::parse(release.tag_name.trim_start_matches('v'))
        .with_context(|| format!("invalid release tag: {}", release.tag_name))?;

    let target = target();
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(".tar.gz") && asset.name.contains(target))
        .ok_or_else(|| anyhow!("no release asset for {target}"))?;

    let checksum = release
        .assets
        .iter()
        .find(|asset| asset.name == "SHA256SUMS")
        .ok_or_else(|| anyhow!("release is missing SHA256SUMS"))?;

    Ok(Latest::Found {
        release: Release {
            version,
            asset_name: asset.name.clone(),
            asset_url: asset.browser_download_url.clone(),
            checksum_url: checksum.browser_download_url.clone(),
        },
        etag,
    })
}

pub fn is_newer(release: &Release, current: &str) -> bool {
    Version::parse(current)
        .map(|current| release.version > current)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(version: &str) -> Release {
        Release {
            version: Version::parse(version).unwrap(),
            asset_name: String::new(),
            asset_url: String::new(),
            checksum_url: String::new(),
        }
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer(&release("0.2.0"), "0.1.0"));
        assert!(!is_newer(&release("0.2.0"), "0.2.0"));
        assert!(!is_newer(&release("0.1.0"), "0.2.0"));
    }
}
