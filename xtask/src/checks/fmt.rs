use anyhow::Result;

use crate::cargo_cmd;

use super::GateCtx;

pub fn run(ctx: &GateCtx) -> Result<()> {
    cargo_cmd::run(&ctx.repo_root, &["fmt", "--all", "--", "--check"])
}
