use anyhow::Result;

use crate::cargo_cmd;
use crate::config::{self, BareMetal};
use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &GateCtx, reporter: &mut Reporter) -> Result<()> {
    let targets = match &ctx.profile_cfg.bare_metal {
        BareMetal::AllToolchainTargets => {
            config::toolchain_targets(&ctx.repo_root, &ctx.paths.rust_toolchain)?
        }
        BareMetal::Selected(selected) => {
            if let Some(note) = ctx.profile_cfg.skip_notes.get("targets") {
                reporter.skip(note);
            }
            selected.clone()
        }
    };
    for target in targets {
        cargo_cmd::run(
            &ctx.repo_root,
            &[
                "check",
                "-p",
                &ctx.packages.driver,
                "--target",
                &target,
                "--no-default-features",
            ],
        )?;
        cargo_cmd::run(
            &ctx.repo_root,
            &["check", "-p", &ctx.packages.model, "--target", &target],
        )?;
    }
    Ok(())
}
