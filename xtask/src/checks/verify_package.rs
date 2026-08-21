use anyhow::{Context, Result, bail};

use crate::report::Reporter;
use crate::verify::{self, Verdict, VerifyRequest};

use super::GateCtx;

/// Release-profile only, and deliberately so: `full` and `bounded` package
/// with `--allow-dirty`, where `.cargo_vcs_info.json` is absent or marked
/// dirty and this check would have nothing sound to say. A step that means
/// something in one profile and nothing in the others is worse than one that
/// exists only where it means something.
///
/// This never contacts the registry. It verifies the archive the `package`
/// step just built, against the commit the release is being cut from.
pub fn run(ctx: &mut GateCtx, reporter: &mut Reporter) -> Result<()> {
    let version = ctx
        .driver_version
        .clone()
        .context("archive verification needs the candidate version")?;
    let commit = ctx
        .release_commit
        .clone()
        .context("archive verification needs the release commit")?;
    let archive = ctx
        .repo_root
        .join("target")
        .join("package")
        .join(format!("{}-{version}.crate", ctx.packages.driver));

    let report = verify::verify(&VerifyRequest {
        repo_root: &ctx.repo_root,
        archive: &archive,
        package: &ctx.packages.driver,
        version: &version,
        expect_path_in_vcs: &ctx.paths.driver_package_dir,
        rev: Some(&commit),
        expect_sha256: ctx.package_sha.as_deref(),
    })?;

    reporter.note(&format!(
        "{} source entries against {commit}",
        report.source_entries()
    ));
    reporter.note(&format!(
        "{} content-identical, {} end-of-line-only, {} mismatched",
        report.count(Verdict::Identical),
        report.count(Verdict::EolOnly),
        report.count(Verdict::Mismatch),
    ));

    if !report.is_ok() {
        bail!(
            "the packaged archive does not match {commit}:\n{}",
            report.failures.join("\n")
        );
    }

    let copy = &ctx.evidence_copy;
    ctx.evidence.blank()?;
    ctx.evidence.line(&copy.verify_heading)?;
    ctx.evidence.blank()?;
    ctx.evidence.lines(&copy.verify_intro)?;
    ctx.evidence.blank()?;
    ctx.evidence.lines(&report.summary_lines())?;
    ctx.evidence.blank()?;
    ctx.evidence.line(&copy.commit_inventory_heading)?;
    ctx.evidence.blank()?;
    ctx.evidence.lines(&copy.commit_inventory_intro)?;
    ctx.evidence.blank()?;
    ctx.evidence.line("```text")?;
    ctx.evidence
        .raw(&format!("{}\n", report.digest_table().join("\n")))?;
    ctx.evidence.line("```")?;
    Ok(())
}
