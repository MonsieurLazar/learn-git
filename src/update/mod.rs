mod install;
mod release;

pub mod restart;

use std::sync::mpsc::{self, Receiver};
use std::thread;

use semver::Version;

pub use release::Release;

#[derive(Debug, Clone)]
pub enum UpdateState {
    Idle,
    Available {
        release: Release,
    },
    Downloading {
        latest: Version,
        downloaded: u64,
        total: Option<u64>,
    },
    Ready {
        latest: Version,
    },
    Failed {
        message: String,
    },
}

pub enum UpdateEvent {
    Checked {
        result: CheckResult,
        etag: Option<String>,
    },
    Progress {
        downloaded: u64,
        total: Option<u64>,
    },
    Installed,
    Failed(String),
}

pub enum CheckResult {
    Available(Release),
    UpToDate,
    Error,
}

pub fn check(etag: Option<String>) -> Receiver<UpdateEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let event = match release::latest(etag.as_deref()) {
            Ok(release::Latest::NotModified) => UpdateEvent::Checked {
                result: CheckResult::UpToDate,
                etag,
            },
            Ok(release::Latest::Found {
                release,
                etag: new_etag,
            }) => {
                let result = if release::is_newer(&release, crate::version::VERSION) {
                    CheckResult::Available(release)
                } else {
                    CheckResult::UpToDate
                };
                UpdateEvent::Checked {
                    result,
                    etag: new_etag,
                }
            }
            Err(_) => UpdateEvent::Checked {
                result: CheckResult::Error,
                etag,
            },
        };
        let _ = tx.send(event);
    });
    rx
}

pub fn install(release: Release) -> Receiver<UpdateEvent> {
    install::spawn(release)
}
