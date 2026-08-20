use anyhow::{Result, bail};

use crate::git;
use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &mut GateCtx, reporter: &mut Reporter) -> Result<()> {
    if !git::available() || !git::inside_work_tree(&ctx.repo_root) {
        bail!("release mode requires a Git work tree to establish artifact identity");
    }
    let dirt = git::porcelain_status(&ctx.repo_root)?;
    if !dirt.trim().is_empty() {
        bail!(
            "release mode requires a clean worktree; found:\n{dirt}\ncommit, stash, or remove these before producing release evidence."
        );
    }
    let commit = git::head_commit(&ctx.repo_root)?;
    reporter.note(&format!("clean at {commit}"));
    ctx.evidence.line(&ctx.evidence_copy.title)?;
    ctx.evidence.blank()?;
    ctx.evidence.lines(&ctx.evidence_copy.produced_by)?;
    ctx.evidence.blank()?;
    ctx.evidence.line(&ctx.evidence_copy.source_heading)?;
    ctx.evidence.blank()?;
    ctx.evidence.line(&format!("- Commit: `{commit}`"))?;
    ctx.evidence.line(&ctx.evidence_copy.worktree_clean)?;
    ctx.release_commit = Some(commit);
    let _ = ctx.profile_cfg.require_clean_worktree;
    Ok(())
}
