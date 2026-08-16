# Releasing

Releases are deliberate maintainer actions. Passing CI or merging preparation
work does not authorize a repository visibility change, crates.io publication,
tagging, or a GitHub Release.

## Status dimensions

Track these independently in the packaged README and crate-level documentation:

- lifecycle;
- distribution state and exact version;
- operation-scoped behavioral-model conformance; and
- physical observation or qualification evidence.

Publication does not imply complete model conformance, physical observation,
hardware qualification, `ph-hil` adoption, or promotion from Incubating.

## 1. Prepare the candidate

Preparation may occur while the repository and crate remain private and
unpublished.

1. Use a lifecycle-matching prerelease version. The first candidate is
   `0.1.0-incubating.1`.
2. Retain `publish = false` until publication is separately approved.
3. Keep the README, crate documentation, changelog, security policy, and model
   declaration consistent with the actual distribution and evidence state.
4. Run `./scripts/ci.sh` (with Git Bash on Windows) and inspect the generated
   package contents.
5. Review the exact candidate through a pull request.

## 2. Decide repository visibility

After preparation is accepted, a maintainer records whether and when the
repository becomes public. Confirm that its contents are suitable for public
disclosure and that contribution, security, and source links match the chosen
visibility. Do not infer this decision from candidate readiness.

## 3. Decide registry publication

After the visibility decision is recorded and implemented as applicable, a
maintainer separately decides whether to publish the candidate to crates.io.
If publication is approved, open a focused release pull request that:

1. changes `publish = false` to `publish = ["crates-io"]`;
2. changes the distribution disclosure to the exact crates.io prerelease;
3. moves accumulated changes into a dated `0.1.0-incubating.1` changelog
   section while preserving an `Unreleased` section, and writes a value
   statement immediately below the release heading stating why the capability
   was added, which limitation it addresses, what value it provides, and which
   cost or constraint it introduces. A list of APIs is not a value statement;
   the organization changelog standard requires one for a release introducing
   substantial capability, which the first release does;
4. runs `CI_PROFILE=release scripts/ci.sh` from a clean tree, which refuses
   release-relevant dirtiness, packages without `--allow-dirty`, and writes
   `target/release-evidence/evidence.md`;
5. runs `cargo publish --dry-run` from that same unchanged tree; and
6. attaches the evidence record — commit, archive name and SHA-256, file
   inventory, normalized manifest, and VCS metadata — for maintainer review.

Registry credentials and `cargo publish` remain outside ordinary CI.

### What the prepublication archive does and does not establish

`cargo publish` **repackages from the source tree**. It does not upload a
previously built `.crate`, so the archive inspected during review is evidence
about a tree, not the exact bytes the registry will receive.

That has one practical consequence, and it is the whole reason the release
profile refuses a dirty worktree: publish from the *same unchanged clean pinned
tree* that produced the evidence. Any edit between review and publication —
including one that looks cosmetic — invalidates the inspected inventory and
checksum, because Cargo will build a different archive.

## 4. Publish the reviewed artifact

Only after the release pull request is approved and merged:

1. verify the release commit and rerun `CI_PROFILE=release scripts/ci.sh`,
   confirming the same commit and archive SHA-256 as the reviewed evidence;
2. tag that commit as `v0.1.0-incubating.1`;
3. publish from that same unchanged clean tree;
4. create a GitHub Release from the same tag using the matching changelog
   section and mark it as a prerelease;
5. download the published `.crate` from crates.io and verify its checksum, file
   inventory, normalized manifest, licence, and README against the recorded
   evidence — this is the only step that establishes what the registry actually
   holds; and
6. verify crates.io ownership, the repository and documentation links, and the
   docs.rs build before announcing availability.

Crates.io versions are permanent. If publication fails after any durable
release record is created, do not move or reuse that record; document the
failure and prepare a higher version when required.
