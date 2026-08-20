use anyhow::{Result, bail};

use crate::cargo_cmd;
use crate::report::Reporter;

use super::GateCtx;

pub fn product(ctx: &GateCtx, reporter: &mut Reporter) -> Result<()> {
    let version = assert_pinned(ctx)?;
    reporter.note(&format!("cargo-deny {version}"));
    cargo_cmd::run(&ctx.repo_root, &["deny", "check", "-D", "warnings"])?;
    ctx.evidence.line(&format!(
        "- `cargo-deny` {version}: {}",
        ctx.evidence_copy.deny_pass_suffix
    ))?;
    Ok(())
}

pub fn assert_pinned(ctx: &GateCtx) -> Result<String> {
    let actual = cargo_cmd::deny_version()?;
    if actual != ctx.packages.cargo_deny_version {
        bail!(
            "cargo-deny {} is required, found {actual}\ninstall it with: cargo install cargo-deny --version {} --locked",
            ctx.packages.cargo_deny_version,
            ctx.packages.cargo_deny_version
        );
    }
    Ok(actual)
}
