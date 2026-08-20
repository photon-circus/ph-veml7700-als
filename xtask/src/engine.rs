use anyhow::{Result, bail};

use crate::cargo_cmd;
use crate::checks::{self, GateCtx};
use crate::config::{self, Config};
use crate::evidence::EvidenceWriter;
use crate::report::{self, Reporter};

pub fn run_ci(profile: Option<&str>, only: Option<&str>) -> Result<()> {
    let repo_root = config::repo_root()?;
    std::env::set_current_dir(&repo_root)?;

    let config = Config::load(&repo_root)?;
    let selected = profile.unwrap_or(&config.gate.default_profile).to_string();
    let profile = selected.as_str();
    let Some(profile_cfg) = config.gate.profiles.get(profile).cloned() else {
        let names: Vec<_> = config.gate.profiles.keys().map(String::as_str).collect();
        bail!("profile must be one of {}: {profile}", names.join(", "));
    };

    // A release run resets the evidence directory, so a partial run would
    // replace a reviewed record with a stub that still reads as a complete
    // document. Release evidence is only meaningful for the whole step set.
    if only.is_some() && profile == "release" {
        bail!(
            "--only cannot be combined with the release profile: release evidence records the whole gate"
        );
    }

    let host_triple = cargo_cmd::host_triple()?;
    match only {
        Some(id) => println!("[ci] profile: {profile} (--only {id})"),
        None => println!("[ci] profile: {profile}"),
    }

    let evidence = EvidenceWriter::new(
        profile == "release",
        repo_root.join(&config.paths.evidence_file),
    )?;

    let mut ctx = GateCtx {
        repo_root,
        host_triple,
        profile: profile.to_string(),
        packages: config.packages,
        paths: config.paths,
        evidence_copy: config.evidence,
        profile_cfg,
        evidence,
        driver_version: None,
        release_commit: None,
        package_sha: None,
    };

    let mut reporter = Reporter::new();
    let steps: Vec<_> = config
        .gate
        .steps
        .iter()
        .filter(|step| step.profiles.iter().any(|name| name == profile))
        .filter(|step| only.is_none_or(|id| id == step.id))
        .collect();

    if let Some(id) = only
        && steps.is_empty()
    {
        bail!("unknown step id for this profile: {id}");
    }

    for step in &steps {
        reporter.begin(&step.title);
        if ctx.profile_cfg.skip_steps.iter().any(|id| id == &step.id) {
            let note = ctx
                .profile_cfg
                .skip_notes
                .get(&step.id)
                .map(String::as_str)
                .unwrap_or("skipped for this profile");
            reporter.skip(note);
            continue;
        }
        if let Err(err) = checks::run(&step.id, &mut ctx, &mut reporter) {
            reporter.fail();
            return Err(err);
        }
    }

    if profile == "release" {
        ctx.evidence.blank()?;
        ctx.evidence
            .line(&ctx.evidence_copy.not_established_heading)?;
        ctx.evidence.blank()?;
        ctx.evidence.lines(&ctx.evidence_copy.not_established)?;
    }

    let evidence_path = ctx.evidence.relative_to(&ctx.repo_root);
    report::print_footer(
        profile,
        only,
        reporter.step_number(),
        reporter.skipped(),
        (profile == "release").then_some(evidence_path.as_str()),
        ctx.release_commit.as_deref(),
        ctx.package_sha.as_deref(),
    );
    Ok(())
}
