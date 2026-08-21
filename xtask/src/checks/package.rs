use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::archive::{self, sha256_file, sha256_hex};
use crate::cargo_cmd;
use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &mut GateCtx, reporter: &mut Reporter) -> Result<()> {
    let package_dir = ctx.repo_root.join("target").join("package");
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)
            .with_context(|| format!("failed to reset {}", package_dir.display()))?;
    }

    let mut args = vec![
        "package".to_string(),
        "-p".to_string(),
        ctx.packages.driver.clone(),
        "--locked".to_string(),
    ];
    if ctx.profile_cfg.package_allow_dirty {
        args.push("--allow-dirty".to_string());
    }
    args.extend([
        "--target-dir".to_string(),
        ctx.repo_root.join("target").to_string_lossy().into_owned(),
    ]);

    let list_args = with_flag(&args, "--list");
    cargo_cmd::run_os(&ctx.repo_root, &list_args)?;
    cargo_cmd::run_os(&ctx.repo_root, &args)?;

    if ctx.profile == "release" {
        record_archive(ctx, reporter, &package_dir)?;
    }

    let unpacked = find_unpacked(&package_dir, &ctx.packages.driver)?;
    let manifest = unpacked.join("Cargo.toml");
    let manifest = manifest.to_string_lossy();
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "test",
            "--manifest-path",
            manifest.as_ref(),
            "--target",
            &ctx.host_triple,
        ],
    )?;
    Ok(())
}

fn with_flag(args: &[String], flag: &str) -> Vec<String> {
    let mut out = args.to_vec();
    out.push(flag.to_string());
    out
}

fn record_archive(ctx: &mut GateCtx, reporter: &mut Reporter, package_dir: &Path) -> Result<()> {
    let version = ctx
        .driver_version
        .as_deref()
        .context("package evidence needs the candidate version")?;
    let archive_name = format!("{}-{version}.crate", ctx.packages.driver);
    let archive = package_dir.join(&archive_name);
    if !archive.is_file() {
        bail!(
            "expected prepublication archive not found: {}",
            archive.display()
        );
    }
    let prefix = format!("{}-{version}/", ctx.packages.driver);
    let entries = archive::entries(&archive, &prefix)?;
    let package_sha = sha256_file(&archive)?;
    let digests: Vec<String> = entries
        .iter()
        .map(|(name, data)| format!("{}  {name}", sha256_hex(data)))
        .collect();

    reporter.note(&format!("archive {archive_name}"));
    reporter.note(&format!("sha256  {package_sha}"));
    reporter.note(&format!("{} files in the archive", entries.len()));

    let copy = &ctx.evidence_copy;
    ctx.evidence.blank()?;
    ctx.evidence.line(&copy.archive_heading)?;
    ctx.evidence.blank()?;
    ctx.evidence.lines(&copy.archive_intro)?;
    ctx.evidence.blank()?;
    ctx.evidence.line(&format!("- File: `{archive_name}`"))?;
    ctx.evidence.line(&format!("- SHA-256: `{package_sha}`"))?;
    ctx.evidence.line(&format!("- Files: {}", entries.len()))?;
    ctx.evidence.blank()?;
    ctx.evidence.line(&copy.inventory_heading)?;
    ctx.evidence.blank()?;
    ctx.evidence.lines(&copy.inventory_intro)?;
    ctx.evidence.blank()?;
    ctx.evidence.line("```text")?;
    ctx.evidence.raw(&format!("{}\n", digests.join("\n")))?;
    ctx.evidence.line("```")?;
    ctx.evidence.blank()?;
    ctx.evidence.line(&copy.manifest_heading)?;
    ctx.evidence.blank()?;
    ctx.evidence.line("```toml")?;
    let unpacked = package_dir.join(format!("{}-{version}", ctx.packages.driver));
    ctx.evidence.raw(
        &fs::read_to_string(unpacked.join("Cargo.toml"))
            .with_context(|| format!("failed to read {}", unpacked.join("Cargo.toml").display()))?,
    )?;
    ctx.evidence.line("```")?;
    ctx.evidence.blank()?;
    ctx.evidence.line(&copy.vcs_heading)?;
    ctx.evidence.blank()?;
    ctx.evidence.line("```json")?;
    let vcs = unpacked.join(".cargo_vcs_info.json");
    if vcs.is_file() {
        ctx.evidence.raw(&fs::read_to_string(&vcs)?)?;
    } else {
        ctx.evidence.line(&copy.missing_vcs)?;
    }
    ctx.evidence.line("```")?;
    ctx.evidence.blank()?;
    ctx.evidence.line(&copy.test_boundary_heading)?;
    ctx.evidence.blank()?;
    ctx.evidence.lines(&copy.test_boundary)?;

    ctx.package_sha = Some(package_sha);
    Ok(())
}

fn find_unpacked(package_dir: &Path, driver: &str) -> Result<PathBuf> {
    let mut matches = Vec::new();
    for entry in fs::read_dir(package_dir)
        .with_context(|| format!("failed to read {}", package_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&format!("{driver}-")))
        {
            matches.push(path);
        }
    }
    matches.into_iter().next().with_context(|| {
        format!(
            "no unpacked {driver} package under {}",
            package_dir.display()
        )
    })
}
