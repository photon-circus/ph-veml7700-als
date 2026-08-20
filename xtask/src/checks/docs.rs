use anyhow::Result;

use crate::cargo_cmd;

use super::GateCtx;

pub fn build(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run_with_env(
        &ctx.repo_root,
        &[
            "doc",
            "-p",
            &ctx.packages.driver,
            "--all-features",
            "--no-deps",
        ],
        &[("RUSTDOCFLAGS", "-D warnings")],
    )?;
    cargo_cmd::run_with_env(
        &ctx.repo_root,
        &["doc", "-p", &ctx.packages.model, "--no-deps"],
        &[("RUSTDOCFLAGS", "-D warnings")],
    )?;
    Ok(())
}

pub fn doctests(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "test",
            "-p",
            &ctx.packages.driver,
            "--all-features",
            "--doc",
            "--target",
            &ctx.host_triple,
        ],
    )
}
