use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub fn host_triple() -> Result<String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .context("failed to spawn rustc -vV")?;
    if !output.status.success() {
        bail!("rustc -vV failed");
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(triple) = line.strip_prefix("host: ") {
            let triple = triple.trim();
            if !triple.is_empty() {
                return Ok(triple.to_string());
            }
        }
    }
    bail!("could not resolve the host triple from rustc -vV")
}

fn cargo_command(repo_root: &Path) -> Command {
    let mut command = Command::new("cargo");
    command.current_dir(repo_root);
    command.env_remove("CARGO_TARGET_DIR");
    command.env_remove("CARGO_MANIFEST_DIR");
    command
}

pub fn run(repo_root: &Path, args: &[&str]) -> Result<()> {
    run_with_env(repo_root, args, &[])
}

pub fn run_discard_stdout(repo_root: &Path, args: &[&str]) -> Result<()> {
    let status = cargo_command(repo_root)
        .args(args)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("failed to spawn cargo {}", args.join(" ")))?;
    if !status.success() {
        bail!("cargo {} failed", args.join(" "));
    }
    Ok(())
}

pub fn run_with_env(repo_root: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<()> {
    let mut command = cargo_command(repo_root);
    command.args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    let status = command
        .status()
        .with_context(|| format!("failed to spawn cargo {}", args.join(" ")))?;
    if !status.success() {
        bail!("cargo {} failed", args.join(" "));
    }
    Ok(())
}

pub fn stdout(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = cargo_command(repo_root)
        .args(args)
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| format!("failed to spawn cargo {}", args.join(" ")))?;
    if !output.status.success() {
        bail!("cargo {} failed", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn package_version(repo_root: &Path, package: &str) -> Result<String> {
    let pkgid = stdout(repo_root, &["pkgid", "-p", package])?;
    let version = pkgid
        .trim()
        .rsplit('@')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if version.is_empty() {
        bail!("could not resolve a package version through cargo pkgid");
    }
    Ok(version)
}

pub fn deny_version() -> Result<String> {
    let output = Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .context("failed to spawn cargo deny --version")?;
    if !output.status.success() {
        bail!("cargo deny --version failed");
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.split_whitespace()
        .nth(1)
        .map(str::to_string)
        .context("could not parse cargo deny --version")
}

pub fn run_os(repo_root: &Path, args: &[impl AsRef<OsStr>]) -> Result<()> {
    let pretty = args
        .iter()
        .map(|a| a.as_ref().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    let status = cargo_command(repo_root)
        .args(args)
        .status()
        .with_context(|| format!("failed to spawn cargo {pretty}"))?;
    if !status.success() {
        bail!("cargo {pretty} failed");
    }
    Ok(())
}
