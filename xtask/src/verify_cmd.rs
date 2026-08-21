//! The `verify-package` command: what an operator runs after publication.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::config::{self, Config};
use crate::registry;
use crate::verify::{self, Verdict, VerifyRequest};

pub struct Args {
    pub archive: Option<PathBuf>,
    pub version: Option<String>,
    pub rev: Option<String>,
    pub sha256: Option<String>,
    pub download_dir: PathBuf,
}

pub fn run(args: Args) -> Result<()> {
    let repo_root = config::repo_root()?;
    std::env::set_current_dir(&repo_root)?;
    let config = Config::load(&repo_root)?;
    let package = config.packages.driver.as_str();

    let (archive, version, rev, expect_sha256, contacted_registry) = match &args.version {
        Some(version) => {
            let published = registry::index_entry(package, version)?;
            if published.yanked {
                println!("[verify] note: crates.io reports {package} {version} as yanked");
            }
            if let Some(pinned) = &args.sha256
                && !pinned.eq_ignore_ascii_case(&published.cksum)
            {
                bail!(
                    "the crates.io index publishes {} for {package} {version}, not the pinned {pinned}",
                    published.cksum
                );
            }
            println!("[verify] index checksum {}", published.cksum);
            let dest = repo_root.join(&args.download_dir);
            let path = registry::download_crate(package, &published, &dest)?;
            println!("[verify] downloaded {}", path.display());
            let rev = args.rev.clone().unwrap_or_else(|| format!("v{version}"));
            (path, version.clone(), rev, Some(published.cksum), true)
        }
        None => {
            let archive = args
                .archive
                .clone()
                .expect("clap requires --archive or --version");
            let archive = if archive.is_absolute() {
                archive
            } else {
                repo_root.join(archive)
            };
            let rev = args
                .rev
                .clone()
                .expect("clap requires --rev alongside --archive");
            let version = version_from_archive(&archive, package)?;
            (archive, version, rev, args.sha256.clone(), false)
        }
    };

    let report = verify::verify(&VerifyRequest {
        repo_root: &repo_root,
        archive: &archive,
        package,
        version: &version,
        expect_path_in_vcs: &config.paths.driver_package_dir,
        rev: Some(&rev),
        expect_sha256: expect_sha256.as_deref(),
    })?;

    println!(
        "[verify] {} against {rev} ({})",
        report.archive_name, report.resolved_commit
    );
    println!("[verify] archive sha256 {}", report.archive_sha256);
    for entry in &report.entries {
        println!("  {:<9} {}", entry.verdict.label(), entry.archive_path);
    }
    println!(
        "[verify] {} source entries: {} identical, {} end-of-line-only, {} mismatched; {} extra, {} missing",
        report.source_entries(),
        report.count(Verdict::Identical),
        report.count(Verdict::EolOnly),
        report.count(Verdict::Mismatch),
        report.count(Verdict::ExtraInArchive),
        report.count(Verdict::MissingFromArchive),
    );

    if contacted_registry {
        println!(
            "[verify] Registry contact: unauthenticated read-only GET of the sparse index and"
        );
        println!("[verify] the published archive. No credential use, no publish, tag, or release.");
    } else {
        println!("[verify] Offline: no registry contact.");
    }

    if !report.is_ok() {
        bail!(
            "the archive does not match {rev}:\n{}",
            report.failures.join("\n")
        );
    }
    println!(
        "[verify] PASS: every packaged source entry matches {}",
        report.resolved_commit
    );
    Ok(())
}

/// `<package>-<version>.crate` is the only name Cargo gives an archive, so the
/// version can be read back off it rather than asked for twice.
fn version_from_archive(archive: &Path, package: &str) -> Result<String> {
    let name = archive
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stem = name.strip_suffix(".crate").unwrap_or(name.as_str());
    match stem.strip_prefix(&format!("{package}-")) {
        Some(version) if !version.is_empty() => Ok(version.to_string()),
        _ => bail!("cannot read a version out of {name}; expected {package}-<version>.crate"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::version_from_archive;

    #[test]
    fn reads_the_version_off_the_archive_name() {
        let path = Path::new("target/package/demo-0.1.0-incubating.1.crate");
        assert_eq!(
            version_from_archive(path, "demo").expect("parsed"),
            "0.1.0-incubating.1"
        );
    }

    #[test]
    fn rejects_an_archive_named_for_another_package() {
        let path = Path::new("other-0.1.0.crate");
        assert!(version_from_archive(path, "demo").is_err());
    }
}
