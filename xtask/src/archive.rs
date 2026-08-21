//! Reading a `.crate` archive, and the two digests taken of every entry.
//!
//! An entry has two identities. Its *archive* digest is the bytes Cargo
//! actually packaged, which depend on the machine that packaged them. Its
//! *content* digest is those bytes with carriage returns before newlines
//! removed, which is the form Git stores and is therefore a property of the
//! commit alone. Release verification compares the second.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;

/// Read every file entry out of a `.crate`, keyed by its path with the
/// `<name>-<version>/` prefix stripped, sorted by that path.
pub fn entries(archive_path: &Path, prefix: &str) -> Result<Vec<(String, Vec<u8>)>> {
    let file = File::open(archive_path)
        .with_context(|| format!("failed to open {}", archive_path.display()))?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let mut entries = Vec::new();
    let mut stripped = 0usize;
    for entry in archive.entries().context("failed to read crate archive")? {
        let mut entry = entry.context("failed to read crate entry")?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let path = entry
            .path()
            .context("crate entry path")?
            .to_string_lossy()
            .replace('\\', "/");
        let rel = match path.strip_prefix(prefix) {
            Some(rel) => {
                stripped += 1;
                rel
            }
            None => path.as_str(),
        };
        let rel = rel.trim_start_matches('/');
        // Every file entry must reach the inventory. Skipping one silently
        // would understate the archive the evidence record describes.
        if rel.is_empty() {
            bail!("crate archive holds a file entry with no path under {prefix}: {path}");
        }
        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .context("failed to read crate entry contents")?;
        entries.push((rel.to_string(), data));
    }
    // A prefix that never matched means the caller named the wrong package or
    // version. Every entry would then be reported as unknown to the commit,
    // which reads as a compromised archive rather than as a typo.
    if stripped == 0 && !entries.is_empty() {
        bail!(
            "no entry in {} lies under {prefix}; the package name or version does not match this archive",
            archive_path.display()
        );
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let data = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(sha256_hex(&data))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Drop every carriage return that immediately precedes a newline. Nothing
/// else changes: a lone `\r` is content and survives.
pub fn normalize_lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
            index += 1;
            continue;
        }
        out.push(bytes[index]);
        index += 1;
    }
    out
}

/// Digest of the end-of-line-normalized bytes: the same value on every
/// platform for the same content.
pub fn content_digest(bytes: &[u8]) -> String {
    sha256_hex(&normalize_lf(bytes))
}

/// Whether normalizing line endings is a sound thing to do to these bytes.
/// A NUL byte in the first 8 KiB means binary, where a `\r\n` pair is data
/// rather than a line ending.
pub fn looks_textual(bytes: &[u8]) -> bool {
    let window = bytes.len().min(8192);
    !bytes[..window].contains(&0)
}

#[cfg(test)]
mod tests {
    use super::{content_digest, looks_textual, normalize_lf};

    #[test]
    fn strips_carriage_returns_before_newlines() {
        assert_eq!(normalize_lf(b"a\r\nb\r\n"), b"a\nb\n");
    }

    #[test]
    fn keeps_a_carriage_return_that_is_not_a_line_ending() {
        assert_eq!(normalize_lf(b"a\rb"), b"a\rb");
        assert_eq!(normalize_lf(b"trailing\r"), b"trailing\r");
    }

    #[test]
    fn normalizing_is_idempotent_on_lf_input() {
        let once = normalize_lf(b"a\nb\n");
        assert_eq!(normalize_lf(&once), once);
    }

    #[test]
    fn the_two_line_ending_forms_share_a_content_digest() {
        assert_eq!(content_digest(b"a\r\nb\r\n"), content_digest(b"a\nb\n"));
    }

    #[test]
    fn a_content_change_moves_the_content_digest() {
        assert_ne!(content_digest(b"a\r\nb\r\n"), content_digest(b"a\nc\n"));
    }

    #[test]
    fn a_nul_byte_marks_content_as_binary() {
        assert!(looks_textual(b"plain text\r\n"));
        assert!(!looks_textual(b"pre\0post\r\n"));
    }
}
