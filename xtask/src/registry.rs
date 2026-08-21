//! Read-only crates.io access, used by `verify-package` and by nothing else.
//!
//! Two unauthenticated GETs: the sparse index entry for a version, which
//! carries the authoritative checksum of the uploaded artifact, and the
//! artifact itself. No credential is sent, nothing is mutated, and the
//! canonical gate never calls into this module.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::archive;

const INDEX_HOST: &str = "https://index.crates.io";
const STATIC_HOST: &str = "https://static.crates.io";
const USER_AGENT: &str =
    "ph-veml7700-als-xtask (+https://github.com/photon-circus/ph-veml7700-als)";
/// A published `.crate` is a few tens of kilobytes. This bounds a hostile or
/// misdirected response rather than approximating a real size.
const MAX_ARCHIVE_BYTES: u64 = 32 * 1024 * 1024;

pub struct Published {
    pub version: String,
    pub cksum: String,
    pub yanked: bool,
}

#[derive(Deserialize)]
struct IndexLine {
    vers: String,
    cksum: String,
    #[serde(default)]
    yanked: bool,
}

/// The sparse-index layout: one-character names live under `1/`, two under
/// `2/`, three under `3/<first>/`, and everything else under
/// `<first two>/<next two>/`.
pub fn index_path(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    match lower.len() {
        0 => String::new(),
        1 => format!("1/{lower}"),
        2 => format!("2/{lower}"),
        3 => format!("3/{}/{lower}", &lower[0..1]),
        _ => format!("{}/{}/{lower}", &lower[0..2], &lower[2..4]),
    }
}

/// Pick one version out of a newline-delimited sparse-index file.
pub fn parse_index(body: &str, version: &str) -> Result<Published> {
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parsed: IndexLine =
            serde_json::from_str(line).context("failed to parse a crates.io index line")?;
        if parsed.vers == version {
            return Ok(Published {
                version: parsed.vers,
                cksum: parsed.cksum,
                yanked: parsed.yanked,
            });
        }
    }
    bail!("the crates.io index lists no version {version}")
}

pub fn index_entry(package: &str, version: &str) -> Result<Published> {
    let url = format!("{INDEX_HOST}/{}", index_path(package));
    let body = get_text(&url)?;
    parse_index(&body, version)
}

/// Download a published archive and write it out only once it hashes to the
/// checksum the registry published for it. An artifact that fails that check
/// never reaches the disk.
pub fn download_crate(package: &str, published: &Published, dest_dir: &Path) -> Result<PathBuf> {
    let url = format!(
        "{STATIC_HOST}/crates/{package}/{package}-{}.crate",
        published.version
    );
    let bytes = get_bytes(&url)?;
    let actual = archive::sha256_hex(&bytes);
    if !actual.eq_ignore_ascii_case(&published.cksum) {
        bail!(
            "{url} hashes to {actual}, but the crates.io index publishes {} for this version",
            published.cksum
        );
    }
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("failed to create {}", dest_dir.display()))?;
    let dest = dest_dir.join(format!("{package}-{}.crate", published.version));
    fs::write(&dest, &bytes).with_context(|| format!("failed to write {}", dest.display()))?;
    Ok(dest)
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

fn get_text(url: &str) -> Result<String> {
    let bytes = get_bytes(url)?;
    String::from_utf8(bytes).with_context(|| format!("{url} did not return UTF-8"))
}

fn get_bytes(url: &str) -> Result<Vec<u8>> {
    let mut response = agent()
        .get(url)
        .call()
        .with_context(|| format!("failed to GET {url}"))?;
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(MAX_ARCHIVE_BYTES + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to read the response from {url}"))?;
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        bail!("{url} returned more than {MAX_ARCHIVE_BYTES} bytes");
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{index_path, parse_index};

    #[test]
    fn index_paths_follow_the_length_rules() {
        assert_eq!(index_path("a"), "1/a");
        assert_eq!(index_path("ab"), "2/ab");
        assert_eq!(index_path("abc"), "3/a/abc");
        assert_eq!(index_path("serde"), "se/rd/serde");
        assert_eq!(index_path("ph-veml7700-als"), "ph/-v/ph-veml7700-als");
    }

    #[test]
    fn index_paths_are_lowercased() {
        assert_eq!(index_path("Ph-Veml7700-Als"), "ph/-v/ph-veml7700-als");
    }

    const BODY: &str = concat!(
        r#"{"name":"demo","vers":"0.1.0","cksum":"aaaa","yanked":true}"#,
        "\n",
        r#"{"name":"demo","vers":"0.2.0","cksum":"bbbb","yanked":false}"#,
        "\n"
    );

    #[test]
    fn picks_the_requested_version_out_of_the_index() {
        let found = parse_index(BODY, "0.2.0").expect("version present");
        assert_eq!(found.cksum, "bbbb");
        assert!(!found.yanked);
    }

    #[test]
    fn reports_a_yanked_version_rather_than_hiding_it() {
        let found = parse_index(BODY, "0.1.0").expect("version present");
        assert!(found.yanked);
    }

    #[test]
    fn an_absent_version_is_an_error() {
        assert!(parse_index(BODY, "9.9.9").is_err());
    }
}
