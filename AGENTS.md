# Agent operating instructions

This repository is contract-first. Read, in order:

1. `docs/HARDWARE_CONTRACT.md`
2. `docs/INVARIANTS.md`
3. `docs/ARCHITECTURE.md`
4. `docs/API_CONTRACT.md`
5. `docs/IMPLEMENTATION_PLAN.md`
6. `docs/TEST_PLAN.md`

## Non-negotiable rules

- Preserve `#![no_std]`, allocation-free, async-first, and unsafe-free runtime
  policy.
- `Veml7700<I2C>` owns only the I²C resource. Do not cache configuration,
  shutdown, power-saving, threshold, status, or sample state.
- `new()` is inert and `release()` returns the exact resource.
- Every 16-bit register transaction uses low byte then high byte on the wire.
- The fixed address is `0x10`; do not expose an address constructor parameter.
- Snapshot reads never claim freshness. Fresh operations must include timing and
  configuration provenance.
- The VEML7700 has no dedicated interrupt pin. Use “threshold monitor” and
  “threshold status,” never an `InputPin` abstraction.
- An enabled threshold monitor owns gain, integration time, persistence,
  thresholds, power state, and power-saving cadence. Ordinary methods must not
  silently retarget it.
- Nominal lux scaling is integer and datasheet-table based. Empirical correction,
  cover-window compensation, source-spectrum correction, and auto-ranging remain
  application policy until separately contracted.
- Preserve the concrete I²C error and identify the semantic operation/stage.
- No public raw register access in v0.1.
- The publishable crate does not depend on `ph-hil`; HIL integration is through
  public schema-1 files and CLI contracts.
- Publication is hard-disabled. Agents and automation must not enable Cargo
  publication, invoke `cargo publish`, add registry credentials, or add a
  release workflow. Re-enabling publication requires a separate owner-reviewed
  contract change.
- Vendor PDFs are local review inputs, not repository content. Never stage or
  commit `docs/vendor/*.pdf` unless the owner has first documented permissive
  redistribution rights and deliberately changes the ignore and validation
  policy.
- GitHub Actions is disabled while this crate is under development. Do not add
  workflow YAML or depend on GitHub-hosted or GitHub-orchestrated runners. Run
  the complete CI gate locally with `tools/check.sh` or `tools/check.ps1`.

## Completion discipline

A task is not complete until documentation, implementation, tests, and pack
validation agree. Compiler or Clippy failures are bootstrap defects, not expected
follow-up work. Physical claims require sealed, reviewed evidence; source review,
mock replay, and compilation are not hardware validation.
