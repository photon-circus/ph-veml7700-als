use std::fs;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::cargo_cmd;
use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &mut GateCtx, reporter: &mut Reporter) -> Result<()> {
    cargo_cmd::run_discard_stdout(&ctx.repo_root, &["metadata", "--format-version", "1"])?;

    let driver = cargo_cmd::package_version(&ctx.repo_root, &ctx.packages.driver)?;
    let model = cargo_cmd::package_version(&ctx.repo_root, &ctx.packages.model)?;
    if driver != model {
        bail!("workspace crates must share one version: driver {driver}, model {model}");
    }
    if !matches_lifecycle(&driver, &ctx.packages.lifecycle) {
        bail!(
            "version must be exactly X.Y.Z-{}.N: {driver}",
            ctx.packages.lifecycle
        );
    }

    check_conformance_manifest(ctx)?;
    let conformance = cargo_cmd::package_version(&ctx.repo_root, &ctx.packages.conformance)?;
    if conformance != ctx.packages.conformance_sentinel {
        bail!(
            "resolved conformance version {conformance} does not match the {} sentinel in its manifest",
            ctx.packages.conformance_sentinel
        );
    }

    reporter.note(&format!("candidate version {driver}"));
    ctx.evidence.line(&format!(
        "- Candidate version: `{driver}` {}",
        ctx.evidence_copy.candidate_version_suffix
    ))?;
    ctx.driver_version = Some(driver);
    Ok(())
}

fn check_conformance_manifest(ctx: &GateCtx) -> Result<()> {
    let path = ctx.repo_root.join(&ctx.paths.conformance_manifest);
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    if text
        .lines()
        .any(|line| line.trim_start().starts_with("version.workspace"))
    {
        bail!(
            "the conformance package must not inherit the product version: {} declares version.workspace",
            ctx.paths.conformance_manifest
        );
    }
    let sentinel =
        Regex::new(r#"^[[:space:]]*version[[:space:]]*=[[:space:]]*"0\.0\.0"[[:space:]]*$"#)
            .expect("sentinel regex");
    if !text.lines().any(|line| sentinel.is_match(line)) {
        bail!(
            "the conformance package must pin the literal {} sentinel in {}",
            ctx.packages.conformance_sentinel,
            ctx.paths.conformance_manifest
        );
    }
    if !text.lines().any(|line| line.trim() == "publish = false") {
        bail!("the conformance package must keep publish = false");
    }
    Ok(())
}

pub fn matches_lifecycle(version: &str, lifecycle: &str) -> bool {
    let pattern = format!(r"^[0-9]+\.[0-9]+\.[0-9]+-{lifecycle}\.[0-9]+$");
    Regex::new(&pattern)
        .expect("lifecycle regex")
        .is_match(version)
}

#[cfg(test)]
mod tests {
    use super::matches_lifecycle;

    #[test]
    fn accepts_the_incubating_form() {
        assert!(matches_lifecycle("0.1.0-incubating.1", "incubating"));
        assert!(matches_lifecycle("1.2.3-incubating.12", "incubating"));
    }

    #[test]
    fn rejects_other_cargo_versions() {
        assert!(!matches_lifecycle("0.1.0", "incubating"));
        assert!(!matches_lifecycle("0.1.0-incubating", "incubating"));
        assert!(!matches_lifecycle("0.1.0-alpha.1", "incubating"));
        assert!(!matches_lifecycle(
            "10.1.0-incubating.1-extra",
            "incubating"
        ));
    }
}
