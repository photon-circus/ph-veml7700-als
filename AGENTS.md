# Agent guidance

## Product boundary

This repository owns an async, allocation-free `no_std` VEML7700 driver plus
pure, scripted-I²C, and autonomous-state tests. The coupled fake is test-only,
shares driver semantic types, and does not implement an I²C device endpoint; do
not present it as independent cross-validation. The independent device
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

## Validation and release safety

Run `scripts/ci.sh`; `tools/check.sh` and `tools/check.ps1` are thin launchers.
The gate is local and bounded. Version 0.1.0 is unpublished and `publish = false`
is intentional. Do not add registry credentials, publication automation, tags,
releases, or an inferred release procedure.
