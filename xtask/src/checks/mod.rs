use anyhow::Result;

use crate::config::{EvidenceCopy, Packages, Paths, Profile};
use crate::evidence::EvidenceWriter;
use crate::report::Reporter;

mod claims;
mod clippy;
mod conformance_matrix;
mod deny;
mod docs;
mod error_variants;
mod feature_check;
mod fmt;
mod host_tests;
mod markdown_links;
mod package;
mod targets;
mod vendor;
mod verify_package;
mod version;
mod worktree;
mod xtask_self;

pub struct GateCtx {
    pub repo_root: std::path::PathBuf,
    pub host_triple: String,
    pub profile: String,
    pub packages: Packages,
    pub paths: Paths,
    pub evidence_copy: EvidenceCopy,
    pub profile_cfg: Profile,
    pub evidence: EvidenceWriter,
    pub driver_version: Option<String>,
    pub release_commit: Option<String>,
    pub package_sha: Option<String>,
}

pub fn run(id: &str, ctx: &mut GateCtx, reporter: &mut Reporter) -> Result<()> {
    match id {
        "worktree" => worktree::run(ctx, reporter),
        "version" => version::run(ctx, reporter),
        "vendor" => vendor::run(ctx, reporter),
        "conformance_matrix" => conformance_matrix::run(ctx, reporter),
        "claims" => claims::run(ctx, reporter),
        "markdown_links" => markdown_links::run(ctx, reporter),
        "error_variants" => error_variants::run(ctx, reporter),
        "xtask_fmt" => xtask_self::fmt(ctx),
        "xtask_tests" => xtask_self::tests(ctx),
        "xtask_clippy" => xtask_self::clippy(ctx),
        "fmt" => fmt::run(ctx),
        "driver_tests" => host_tests::driver(ctx),
        "model_tests" => host_tests::model(ctx),
        "conformance" => host_tests::conformance(ctx),
        "feature_check" => feature_check::run(ctx),
        "clippy" => clippy::run(ctx),
        "docs" => docs::build(ctx),
        "doctests" => docs::doctests(ctx),
        "targets" => targets::run(ctx, reporter),
        "deny" => deny::product(ctx, reporter),
        "xtask_deny" => xtask_self::deny(ctx, reporter),
        "package" => package::run(ctx, reporter),
        "verify_package" => verify_package::run(ctx, reporter),
        other => anyhow::bail!("unknown step id in gate.ron: {other}"),
    }
}
