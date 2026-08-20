use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub fn available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn inside_work_tree(repo_root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn stdout(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .with_context(|| format!("failed to spawn git {}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {stderr}", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn ls_files(repo_root: &Path, globs: &[&str]) -> Result<Vec<String>> {
    let mut args = vec!["ls-files"];
    args.extend(globs.iter().copied());
    let text = stdout(repo_root, &args)?;
    Ok(text
        .lines()
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .collect())
}

pub fn head_commit(repo_root: &Path) -> Result<String> {
    Ok(stdout(repo_root, &["rev-parse", "HEAD"])?
        .trim()
        .to_string())
}

pub fn porcelain_status(repo_root: &Path) -> Result<String> {
    stdout(
        repo_root,
        &["status", "--porcelain", "--untracked-files=all"],
    )
}
