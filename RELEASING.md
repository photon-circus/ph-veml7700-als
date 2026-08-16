# Releasing

Releases are deliberate maintainer actions. Passing CI or merging preparation
work does not authorize crates.io publication, tagging, or a GitHub Release.
The repository is public and the driver is published, so each new version is
permanent and immediately consumable the moment it lands. Neither fact makes
the next release automatic: publication is decided again every time.

## Status dimensions

Track these independently in the packaged README and crate-level documentation:

- lifecycle;
- distribution state and exact version;
- operation-scoped behavioral-model conformance; and
- physical observation or qualification evidence.

Publication does not imply complete model conformance, physical observation,
hardware qualification, `ph-hil` adoption, or promotion from Incubating. A
version on crates.io is a distribution fact and nothing more.

## 1. Prepare the candidate

1. Use a lifecycle-matching prerelease version, incremented from the last
   published one. `0.1.0-incubating.1` was the first.
2. Keep the README, crate documentation, changelog, security policy, and model
   declaration consistent with the actual distribution and evidence state.
3. Run `./scripts/ci.sh` (with Git Bash on Windows) and inspect the generated
   package contents.
4. Review the exact candidate through a pull request.

`crates/veml7700-model` and the conformance package keep `publish = false`
permanently. They are test oracles, and publishing either would invite a
consumer to depend on a derivation that exists only to challenge the driver.

## 2. Decide registry publication

A maintainer decides separately, for every candidate, whether to publish it.
That decision is never inferred from candidate readiness, a green gate, issue
closure, or pull-request approval. If publication is approved, open a focused
release pull request that:

1. confirms the driver manifest permits publication as `publish =
   ["crates-io"]`. `cargo publish` refuses a `publish = false` package outright,
   so a dry run cannot even report on the candidate until this holds; the first
   release is what changed it;
2. changes the distribution disclosure to the exact crates.io prerelease;
3. moves accumulated changes into a dated changelog section for that version
   while preserving an `Unreleased` section, and writes a value statement
   immediately below the release heading stating why the capability was added,
   which limitation it addresses, what value it provides, and which cost or
   constraint it introduces. A list of APIs is not a value statement; the
   organization changelog standard requires one for any release introducing
   substantial capability;
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

## 3. Publish the reviewed artifact

Only after the release pull request is approved and merged:

1. regenerate the evidence on the merged release commit by rerunning
   `CI_PROFILE=release scripts/ci.sh`, then confirm the new archive differs
   from the reviewed record in nothing but `.cargo_vcs_info.json`. Do not
   expect the SHA-256 to match: the archive embeds the commit it was built
   from, so a squash merge necessarily changes it. The file inventory,
   normalized manifest, and compressed size must be unchanged, and a
   difference in any of those is a real discrepancy rather than a restamp.
   The regenerated record supersedes the reviewed one;
2. tag that commit with the matching `v`-prefixed version, as
   `v0.1.0-incubating.1` was tagged;
3. publish from that same unchanged clean tree;
4. create a GitHub Release from the same tag using the matching changelog
   section and mark it as a prerelease while the lifecycle is Incubating;
5. download the published `.crate` from crates.io and verify its checksum, file
   inventory, normalized manifest, licence, and README against the regenerated
   record from step 1 — this is the only step that establishes what the registry
   actually holds, and here the checksum must match exactly, because the
   registry receives the archive built from the tagged commit; and
6. verify crates.io ownership, the repository and documentation links, and the
   docs.rs build before announcing availability.

Crates.io versions are permanent. If publication fails after any durable
release record is created, do not move or reuse that record; document the
failure and prepare a higher version when required.
