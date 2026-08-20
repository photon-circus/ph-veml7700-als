use anyhow::Result;

use crate::cargo_cmd;
use crate::report::Reporter;

use super::GateCtx;
use super::deny;

const INCREMENTAL_OFF: &[(&str, &str)] = &[("CARGO_INCREMENTAL", "0")];

pub fn fmt(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "fmt",
            "--manifest-path",
            &ctx.paths.xtask_manifest,
            "--",
            "--check",
        ],
    )
}

pub fn tests(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run_with_env(
        &ctx.repo_root,
        &[
            "test",
            "--manifest-path",
            &ctx.paths.xtask_manifest,
            "--target-dir",
            &ctx.paths.xtask_check_target,
            "--lib",
        ],
        INCREMENTAL_OFF,
    )
}

pub fn clippy(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run_with_env(
        &ctx.repo_root,
        &[
            "clippy",
            "--manifest-path",
            &ctx.paths.xtask_manifest,
            "--target-dir",
            &ctx.paths.xtask_check_target,
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        INCREMENTAL_OFF,
    )
}

pub fn deny(ctx: &GateCtx, reporter: &mut Reporter) -> Result<()> {
    let version = deny::assert_pinned(ctx)?;
    reporter.note(&format!("cargo-deny {version}"));
    let config = ctx.repo_root.join(&ctx.paths.xtask_deny);
    if !config.is_file() {
        anyhow::bail!("missing {}", ctx.paths.xtask_deny);
    }
    cargo_cmd::run(
        &ctx.repo_root.join("xtask"),
        &["deny", "check", "-D", "warnings"],
    )
}
