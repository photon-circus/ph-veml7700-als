# Agent guidance

## Product boundary

This repository owns an async, allocation-free `no_std` VEML7700 driver plus
pure, scripted-I²C, and autonomous-state tests. The coupled fake is test-only,
shares driver semantic types, does not implement an I²C device endpoint, and is
never driven through `Veml7700`; do not present it as driver evidence or as
independent cross-validation. The independent device
behavioral model lives in `crates/veml7700-model` and currently covers only
`probe` and one successful `measure_once` slice.

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

- **The candidate version is duplicated in 11 tracked files.** Only the pair
  `crates/veml7700/Cargo.toml` and the exact-match check in `scripts/ci.sh` is
  machine-verified; the README, packaged crate README, `lib.rs`, AGENTS,
  SECURITY, API_CONTRACT, DECISIONS, CHANGELOG and RELEASING copies rot
  silently. Grep for the literal before and after any bump.
- **The status disclosure lives in three places** that must agree word for word:
  root `README.md`, the packaged `crates/veml7700/README.md`, and the `lib.rs`
  crate documentation. The packaged two are the ones a consumer sees.
- **Vendor provenance lives in two places:** `docs/vendor/README.md` holds the
  retrieval record and `crates/veml7700-model/README.md` repeats the digests as
  part of the model's source declaration. They are coupled; the model README is
  canonical when they disagree.
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
The gate is local and bounded. Version `0.1.0-incubating.1` is unpublished and
`publish = false` is intentional. Follow `RELEASING.md`; do not change
repository visibility, enable registry publication, add credentials, create
tags, or create releases without the corresponding recorded maintainer
decision.
