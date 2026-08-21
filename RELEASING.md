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
3. Run `cargo xtask ci` and inspect the generated package contents.
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
4. runs `cargo xtask ci --profile release` from a clean tree, which refuses
   release-relevant dirtiness, packages without `--allow-dirty`, and writes
   `target/release-evidence/evidence.md`;
5. runs `cargo publish --dry-run` from that same unchanged tree; and
6. attaches the evidence record — commit, archive name and SHA-256, the
   per-entry content digests and blob object ids, the normalized manifest, and
   the VCS metadata — for maintainer review.

Registry credentials and `cargo publish` remain outside ordinary CI.

### What the prepublication archive does and does not establish

`cargo publish` **repackages from the source tree**. It does not upload a
previously built `.crate`, so the archive inspected during review is evidence
about a tree, not the exact bytes the registry will receive.

That has one practical consequence, and it is the whole reason the release
profile refuses a dirty worktree: publish from the *same unchanged clean pinned
tree* that produced the evidence. Any edit between review and publication —
including one that looks cosmetic — means Cargo will build a different archive.

The archive bytes are a property of the machine as well as of the commit: the
platform, the checkout's end-of-line configuration, and the Cargo version all
change them. `.gitattributes` pins `eol=lf`, so a fresh checkout now produces
the same working-tree bytes everywhere, but a checkout made before that change
still holds CRLF and `0.1.0-incubating.1` was published from one. Verification
is therefore anchored on the commit's Git blobs, which are normalized in the
object store, rather than on archive bytes. A difference confined to carriage
returns is reported as end-of-line-only and passes; anything that survives
normalization is edited content and fails. This is what makes an already
published archive verifiable at all, and it is why the inspected checksum is no
longer the thing a later step compares.

## 3. Publish the reviewed artifact

Only after the release pull request is approved and merged:

1. regenerate the evidence on the merged release commit by rerunning
   `cargo xtask ci --profile release`. Its final step re-derives every packaged
   source entry from that commit's Git blobs and reports each as
   content-identical, end-of-line-only, or mismatched, so the comparison no
   longer depends on which platform packaged the archive. Then compare the
   regenerated record's commit-anchored digest table against the reviewed one:
   the content-digest and blob-object-id columns are properties of the commit
   and must be identical apart from the three Cargo-generated entries. Do not
   expect the archive SHA-256, the packaged-bytes column, or the compressed
   size to match — the archive embeds the commit it was built from, so a squash
   merge changes all three with no content change, and DEFLATE output depends
   on byte values. Any content digest or blob object id that moves, or any entry
   that appears or disappears, is edited content rather than a restamp and stops
   the release. The regenerated record supersedes the reviewed one;
2. tag that commit with the matching `v`-prefixed version, as
   `v0.1.0-incubating.1` was tagged;
3. publish from that same unchanged clean tree;
4. create a GitHub Release from the same tag using the matching changelog
   section and mark it as a prerelease while the lifecycle is Incubating;
5. verify what the registry actually holds by running
   `cargo xtask verify-package --version <X.Y.Z-incubating.N>` from a clean
   checkout of the tagged commit. It reads that version's authoritative `cksum`
   from the crates.io sparse index, downloads the published `.crate`, refuses
   to keep a download that does not hash to that value, and then verifies every
   packaged source entry against the Git blobs of the tag it resolves —
   `v<version>` unless `--rev` says otherwise. It also checks that
   `.cargo_vcs_info.json` names that commit and was not packaged from a dirty
   tree, that the normalized manifest's dependency, feature, and target
   surface matches the commit's manifests, and that the packaged lock pins
   no version the commit's lock does not. Do not expect the published
   archive's SHA-256 to
   equal the one in the evidence record: `cargo publish` repackages from the
   working tree. End-of-line-only entries are a pass. A mismatched entry, an
   entry with no blob at the tag, a tracked file missing from the archive, or a
   `.cargo_vcs_info.json` naming a different commit is a failure and must be
   treated as a compromised or mis-published artifact. To verify an archive
   obtained another way, or without network access, pass
   `--archive <path> --rev v<version>` instead; and
6. verify crates.io ownership, the repository and documentation links, and the
   docs.rs build before announcing availability.

`cargo xtask verify-package --version` performs unauthenticated read-only GETs
of the sparse index and the published archive. It takes no registry action and
uses no credentials; the canonical gate itself never contacts the registry.

Crates.io versions are permanent. If publication fails after any durable
release record is created, do not move or reuse that record; document the
failure and prepare a higher version when required.
