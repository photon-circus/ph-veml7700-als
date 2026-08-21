use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Packages {
    pub driver: String,
    pub model: String,
    pub conformance: String,
    pub lifecycle: String,
    pub conformance_sentinel: String,
    pub cargo_deny_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paths {
    pub conformance_manifest: String,
    pub conformance_tests: String,
    pub verification: String,
    pub claim_registry: String,
    pub error_source: String,
    pub vendor_dir: String,
    pub vendor_readme: String,
    pub evidence_file: String,
    pub rust_toolchain: String,
    pub xtask_manifest: String,
    pub xtask_deny: String,
    pub xtask_check_target: String,
    pub crates_dir: String,
    pub driver_package_dir: String,
    pub coverage_start: String,
    pub coverage_end: String,
    pub covered_start: String,
    pub covered_end: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub package_allow_dirty: bool,
    pub skip_steps: Vec<String>,
    pub skip_notes: BTreeMap<String, String>,
    pub bare_metal: BareMetal,
}

#[derive(Debug, Clone, Deserialize)]
pub enum BareMetal {
    AllToolchainTargets,
    Selected(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub title: String,
    pub profiles: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gate {
    pub default_profile: String,
    pub profiles: BTreeMap<String, Profile>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceCopy {
    pub title: String,
    pub produced_by: Vec<String>,
    pub source_heading: String,
    pub worktree_clean: String,
    pub candidate_version_suffix: String,
    pub deny_pass_suffix: String,
    pub archive_heading: String,
    pub archive_intro: Vec<String>,
    pub inventory_heading: String,
    pub inventory_intro: Vec<String>,
    pub verify_heading: String,
    pub verify_intro: Vec<String>,
    pub commit_inventory_heading: String,
    pub commit_inventory_intro: Vec<String>,
    pub manifest_heading: String,
    pub vcs_heading: String,
    pub missing_vcs: String,
    pub test_boundary_heading: String,
    pub test_boundary: Vec<String>,
    pub not_established_heading: String,
    pub not_established: Vec<String>,
}

pub struct Config {
    pub packages: Packages,
    pub paths: Paths,
    pub gate: Gate,
    pub evidence: EvidenceCopy,
}

impl Config {
    pub fn load(repo_root: &Path) -> Result<Self> {
        let dir = data_dir(repo_root);
        Ok(Self {
            packages: load_ron(&dir, "packages.ron")?,
            paths: load_ron(&dir, "paths.ron")?,
            gate: load_ron(&dir, "gate.ron")?,
            evidence: load_ron(&dir, "evidence.ron")?,
        })
    }
}

pub fn repo_root() -> Result<PathBuf> {
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        let xtask_dir = PathBuf::from(manifest);
        if xtask_dir.join("data").is_dir()
            && let Some(parent) = xtask_dir.parent()
            && parent.join("Cargo.toml").is_file()
        {
            return Ok(parent.to_path_buf());
        }
    }

    let mut dir = env::current_dir().context("could not read the current directory")?;
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("xtask").join("data").is_dir() {
            return Ok(dir);
        }
        dir = dir
            .parent()
            .map(Path::to_path_buf)
            .context("not inside the repository")?;
    }
}

fn data_dir(repo_root: &Path) -> PathBuf {
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        let dir = PathBuf::from(manifest).join("data");
        if dir.is_dir() {
            return dir;
        }
    }
    repo_root.join("xtask").join("data")
}

fn load_ron<T: for<'de> Deserialize<'de>>(dir: &Path, name: &str) -> Result<T> {
    let path = dir.join(name);
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    ron::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn toolchain_targets(repo_root: &Path, toolchain_path: &str) -> Result<Vec<String>> {
    let path = repo_root.join(toolchain_path);
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let parsed: ToolchainFile =
        toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;
    if parsed.toolchain.targets.is_empty() {
        bail!("{} lists no targets", path.display());
    }
    Ok(parsed.toolchain.targets)
}

#[derive(Deserialize)]
struct ToolchainFile {
    toolchain: ToolchainSection,
}

#[derive(Deserialize)]
struct ToolchainSection {
    targets: Vec<String>,
}
