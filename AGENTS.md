# Agent guidance

## Product boundary

This repository owns an async, allocation-free `no_std` VEML7700 driver plus
pure, scripted-I²C, and independent-model tests. The model's maintained claim is
[`crates/veml7700-model/README.md`](crates/veml7700-model/README.md).

Do not add MCU examples, board support, fixture definitions, physical-evidence
plans, hardware runners, or orchestration dependencies.

## Canonical documents

| Subject | Source |
| --- | --- |
| Device behavior and provenance | `docs/HARDWARE_CONTRACT.md`, `docs/vendor/README.md` |
| Public Rust surface | `docs/API_CONTRACT.md` |
| Ownership and dependencies | `docs/ARCHITECTURE.md` |
| Review-blocking truths | `docs/INVARIANTS.md` |
| Verification responsibilities | `docs/TEST_PLAN.md` |
| Durable rationale | `docs/DECISIONS.md` |
| Claim and terminology rules | `docs/DOCUMENTATION_STANDARDS.md` |
| Independent model claim | `crates/veml7700-model/README.md` |

Put bounded distributable work in GitHub issues. Do not retain bootstrap roles,
task packets, generated inventories, or tool-specific agent hierarchies.

## Change discipline

- Keep `new()` inert and return the exact bus from `release()`.
- Use low byte then high byte for every 16-bit register transaction.
- Never describe a snapshot as fresh.
- Bind explicit timing to the selected integration time.
- Preserve the complete threshold-monitor domain and reject silent retargeting.
- Keep white-channel counts distinct from ALS nominal scaling.
- Preserve concrete bus errors and multi-stage restoration context.
- Keep optical correction and calibration outside this driver.
- Keep the independent model derived from the hardware contract, not driver
  codecs, timing helpers, or private constants.

Public behavior changes require coupled tests, contracts, rationale, and
changelog updates.

## Operational traps

The compiler cannot catch any of these.

- **The candidate version is duplicated across tracked prose.** Both crates
  inherit `version` from `[workspace.package]`, so a bump edits one manifest
  line and drift between the crates is unrepresentable. The gate reads the
  resolved value back through `cargo pkgid` — not by parsing manifest text,
  which would now find nothing — and requires an `-incubating.N` prerelease
  without storing the literal itself. The root README, packaged crate README,
  AGENTS, API_CONTRACT, DECISIONS, CHANGELOG and RELEASING copies still rot
  silently. Grep for the literal before and after any bump.

  `lib.rs` is **not** on that list any more: it includes the packaged README
  with `#![doc = include_str!]`, so it holds no literal of its own. Do not add
  one back.
- **The status disclosure lives in two places** that must agree word for word:
  root `README.md` and the packaged `crates/veml7700/README.md`. The packaged one
  is what a consumer sees, and it is also the crate documentation, so docs.rs
  shows the same text rather than a third copy. The gate compares the two after
  normalizing `>` prefixes and line wrapping, so only the wording is free. Keep
  the disclosure to the four profile facts.

  The crate documentation is not a copy to maintain. `lib.rs` includes the
  packaged README verbatim; editing the README edits both.
- **Vendor provenance lives in two places:** `docs/vendor/README.md` holds the
  retrieval record and `crates/veml7700-model/README.md` repeats the digests as
  part of the model's source declaration. They are coupled but not gate-checked;
  the model README is canonical when they disagree.
- **Packaging is pinned to the repository `target/` directory** on purpose. See
  D-017: Cargo excludes only that path from workspace member discovery, so an
  extracted package placed anywhere else inside the repository becomes
  untestable.
- **A validating constructor needs private fields.** `Thresholds` enforces
  `low <= high` in `new()`; that is only meaningful because the fields are
  private. Adding `pub` to a field beside a validating constructor silently
  demotes the invariant to advice.

## Validation and release safety

Run `scripts/ci.sh`; `tools/check.ps1` is a thin PowerShell launcher for it.
`CI_PROFILE=bounded` selects the subset hosted CI runs and is never
authoritative. `CI_PROFILE=release` is `full` plus artifact identity: it refuses
a dirty worktree, packages without `--allow-dirty`, and writes
`target/release-evidence/evidence.md`. It performs no registry action. Add
checks to the script, not to the workflow: there must stay exactly one
implementation of the gate.
The gate is local and bounded. Version `0.1.0-incubating.1` is unpublished and
`publish = false` is intentional. Follow `RELEASING.md`; do not change
repository visibility, enable registry publication, add credentials, create
tags, or create releases without the corresponding recorded maintainer
decision.
