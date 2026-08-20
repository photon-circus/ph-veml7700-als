use anyhow::Result;

use crate::git;
use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &GateCtx, reporter: &mut Reporter) -> Result<()> {
    if !git::available() || !git::inside_work_tree(&ctx.repo_root) {
        reporter.skip("no Git work tree, so tracked vendor documents cannot be checked");
        return Ok(());
    }

    let tracked = git::ls_files(&ctx.repo_root, &[&ctx.paths.vendor_dir])?;
    let unexpected: Vec<_> = tracked
        .into_iter()
        .filter(|path| path != &ctx.paths.vendor_readme)
        .collect();
    if !unexpected.is_empty() {
        anyhow::bail!(
            "vendor documents must not be tracked:\n{}",
            unexpected.join("\n")
        );
    }
    Ok(())
}
