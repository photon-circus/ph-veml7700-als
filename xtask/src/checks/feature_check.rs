use anyhow::Result;

use crate::cargo_cmd;

use super::GateCtx;

pub fn run(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run(
        &ctx.repo_root,
        &["check", "-p", &ctx.packages.driver, "--all-features"],
    )?;
    cargo_cmd::run(&ctx.repo_root, &["check", "-p", &ctx.packages.model])?;
    cargo_cmd::run(&ctx.repo_root, &["check", "-p", &ctx.packages.conformance])?;
    Ok(())
}
