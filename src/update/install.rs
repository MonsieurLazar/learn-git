use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use anyhow::{Context, Result, anyhow};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};

use super::{Release, UpdateEvent};

pub fn spawn(release: Release) -> Receiver<UpdateEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Err(err) = run(&release, &tx) {
            let _ = tx.send(UpdateEvent::Failed(err.to_string()));
        }
    });
    rx
}

fn run(release: &Release, tx: &Sender<UpdateEvent>) -> Result<()> {
    let dir = std::env::temp_dir().join("learn-git-update");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;

    let archive = dir.join(&release.asset_name);
    download(&release.asset_url, &archive, tx)?;

    let checksums = dir.join("SHA256SUMS");
    download_checksums(&release.checksum_url, &checksums)?;
    verify(&archive, &release.asset_name, &checksums)?;

    let binary = extract(&archive, &dir)?;
    self_replace::self_replace(&binary).context("failed to replace the running binary")?;

    let _ = tx.send(UpdateEvent::Installed);
    Ok(())
}

fn download(url: &str, path: &Path, tx: &Sender<UpdateEvent>) -> Result<()> {
    let mut response = ureq::get(url)
        .header("user-agent", "learn-git-updater")
        .call()?;

    let total = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok());

    let mut file = File::create(path)?;
    let mut reader = response.body_mut().as_reader();
    let mut buffer = [0u8; 16 * 1024];
    let mut downloaded = 0u64;

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        downloaded += read as u64;
        let _ = tx.send(UpdateEvent::Progress { downloaded, total });
    }

    Ok(())
}

fn download_checksums(url: &str, path: &Path) -> Result<()> {
    let mut response = ureq::get(url)
        .header("user-agent", "learn-git-updater")
        .call()?;

    let mut body = String::new();
    response.body_mut().as_reader().read_to_string(&mut body)?;
    fs::write(path, body)?;
    Ok(())
}

fn verify(archive: &Path, name: &str, checksums: &Path) -> Result<()> {
    let text = fs::read_to_string(checksums)?;
    let expected = text
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let file = parts.next()?.trim_start_matches('*');
            (file == name).then(|| hash.to_ascii_lowercase())
        })
        .ok_or_else(|| anyhow!("no checksum for {name}"))?;

    let mut file = File::open(archive)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 16 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let actual = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    if actual != expected {
        return Err(anyhow!("checksum mismatch for {name}"));
    }

    Ok(())
}

fn extract(archive: &Path, dir: &Path) -> Result<PathBuf> {
    let file = File::open(archive)?;
    let mut archive = tar::Archive::new(GzDecoder::new(file));
    archive.unpack(dir)?;

    let name = if cfg!(windows) {
        "learn-git.exe"
    } else {
        "learn-git"
    };
    let binary = dir.join(name);

    if !binary.exists() {
        return Err(anyhow!("archive does not contain {name}"));
    }

    Ok(binary)
}
