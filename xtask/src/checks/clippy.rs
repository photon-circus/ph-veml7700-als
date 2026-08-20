use anyhow::Result;

use crate::cargo_cmd;

use super::GateCtx;

pub fn run(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "clippy",
            "-p",
            &ctx.packages.driver,
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "clippy",
            "-p",
            &ctx.packages.model,
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "clippy",
            "-p",
            &ctx.packages.conformance,
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    Ok(())
}
