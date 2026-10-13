//! Download a model's files (feature `download`).

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::Model;

/// Progress callback: `(bytes done, bytes total)` over the whole model.
pub type Progress<'a> = &'a mut dyn FnMut(u64, u64);

/// Download every file of `model` into `dir`, skipping files already
/// there. Each file is written to a temporary name and renamed when
/// complete, so an interrupted download can simply be run again.
pub fn download(model: Model, dir: &Path, progress: Progress<'_>) -> Result<()> {
    fs::create_dir_all(dir)?;
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .user_agent(concat!("vieneu-tts-rs/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    let total = model.size();
    let mut before = 0u64;
    for (url, name) in model.files() {
        let dest = dir.join(&name);
        if dest.is_file() {
            before += fs::metadata(&dest)?.len();
            progress(before.min(total), total);
            continue;
        }
        let mut response = agent
            .get(&url)
            .call()
            .with_context(|| format!("GET {url}"))?;
        let expected: Option<u64> = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let mut reader = response.body_mut().with_config().limit(u64::MAX).reader();
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        let mut buf = vec![0u8; 64 * 1024];
        let mut done = 0u64;
        loop {
            let n = reader
                .read(&mut buf)
                .with_context(|| format!("reading {url}"))?;
            if n == 0 {
                break;
            }
            tmp.write_all(&buf[..n])?;
            done += n as u64;
            progress(before + done, total.max(before + done));
        }
        if let Some(expected) = expected {
            anyhow::ensure!(done == expected, "download of {url} was cut short");
        }
        tmp.persist(&dest)
            .with_context(|| format!("saving {}", dest.display()))?;
        before += done;
    }
    Ok(())
}
