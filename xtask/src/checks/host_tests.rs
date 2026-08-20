use anyhow::Result;

use crate::cargo_cmd;

use super::GateCtx;

pub fn driver(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "test",
            "-p",
            &ctx.packages.driver,
            "--no-default-features",
            "--target",
            &ctx.host_triple,
        ],
    )
}

pub fn model(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "test",
            "-p",
            &ctx.packages.model,
            "--no-default-features",
            "--target",
            &ctx.host_triple,
        ],
    )
}

pub fn conformance(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run(
        &ctx.repo_root,
        &[
            "test",
            "-p",
            &ctx.packages.conformance,
            "--target",
            &ctx.host_triple,
        ],
    )
}
