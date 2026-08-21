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
    let bytes = stdout_bytes(repo_root, args)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Raw stdout. Blob contents must come through here rather than [`stdout`],
/// which is lossy and would silently rewrite any byte it could not decode.
pub fn stdout_bytes(repo_root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .with_context(|| format!("failed to spawn git {}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {stderr}", args.join(" "));
    }
    Ok(output.stdout)
}

/// Resolve a revision -- tag, branch, or commit -- to a full commit id.
pub fn rev_parse(repo_root: &Path, rev: &str) -> Result<String> {
    let spec = format!("{rev}^{{commit}}");
    Ok(stdout(repo_root, &["rev-parse", &spec])?.trim().to_string())
}

/// The bytes Git stores for one path at one commit. Git normalizes line
/// endings on the way in, so this is the same on every platform.
pub fn blob(repo_root: &Path, commit: &str, path: &str) -> Result<Vec<u8>> {
    let spec = format!("{commit}:{path}");
    stdout_bytes(repo_root, &["cat-file", "blob", &spec])
}

/// Every blob under `prefix` at `commit`, as `(object id, path)`. One process
/// yields both, so the object ids cost nothing beyond the listing.
pub fn ls_tree(repo_root: &Path, commit: &str, prefix: &str) -> Result<Vec<(String, String)>> {
    let bytes = stdout_bytes(repo_root, &["ls-tree", "-r", "-z", commit, "--", prefix])?;
    let text = String::from_utf8_lossy(&bytes);
    let mut out = Vec::new();
    for record in text.split('\0').filter(|record| !record.is_empty()) {
        // "<mode> <type> <oid>\t<path>"
        let Some((meta, path)) = record.split_once('\t') else {
            bail!("unparsable git ls-tree record: {record}");
        };
        let fields: Vec<_> = meta.split_whitespace().collect();
        let [_mode, kind, oid] = fields.as_slice() else {
            bail!("unparsable git ls-tree record: {record}");
        };
        if *kind != "blob" {
            continue;
        }
        out.push(((*oid).to_string(), path.to_string()));
    }
    Ok(out)
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
