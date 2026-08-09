# `ph-veml7700-als` development pack manifest

Maintained as a contract-first development pack for a Photon Circus VEML7700
ambient-light-sensor driver. It contains the complete v0.1 host-side driver,
executable contract checks, and mock HIL integration; it is not a registry
release or a calibrated optical support claim.

## Pack boundary

- Repository and package: `ph-veml7700-als`.
- Packageable crate: `crates/veml7700`; Cargo publication is hard-disabled.
- Runtime: async-first, `#![no_std]`, allocation-free,
  `#![forbid(unsafe_code)]`, and `#![deny(missing_docs)]`.
- Primary HAL: `embedded_hal_async::i2c::I2c`.
- Facade: `Veml7700<I2C>` owning only the bus resource.
- Fixed 7-bit address: `0x10`; expected ID word: `0xC481`.

## Included

- normative hardware, architecture, API, invariant, decision, implementation,
  test, documentation, fixture, capability, and HIL evidence contracts;
- strict low-byte-first 16-bit register transfers;
- typed gain, integration-time, persistence, shutdown, power-saving, threshold,
  ID, raw-count, timing, and contextual-error models;
- integer nominal micro-lux scaling for all 24 documented gain/integration pairs,
  including explicit maximum-code saturation observation;
- explicit snapshot versus fresh-measurement results;
- complete fresh sequencing with PSM disable, requested-domain preparation in
  shutdown, an explicit wake edge, integration-bound conservative timing,
  shutdown freeze, ALS/white read, restoration, and staged state-uncertainty
  errors;
- threshold-monitor protection treating gain, integration, thresholds,
  persistence, power state, and PSM cadence as one monitored domain;
- no interrupt GPIO abstraction because the device has no dedicated pin;
- strict scripted-I²C tests and an autonomous fake sensor covering wake timing,
  retention, cadence, channel phasing, persistence, and MCU reset;
- external schema-1 `ph-hil` boundary with 12 capabilities, deterministic mock
  transcript, core and calibrated-optical plan separation, Lua modules, offline
  policy, build hooks, and evidence directories;
- CI, local gates, agent roles, contribution/security policy, release checklist,
  and deterministic SHA-256 inventory.
- manifest, validation, agent, and automation guardrails that prevent Cargo
  registry publication and registry-credential use.

## Deliberate non-claims

- Snapshot data is not fresh by definition.
- ALS and white registers are not claimed to be an atomic hardware pair.
- `MicroLux` is nominal table scaling, not calibrated system lux.
- No automatic range selection, empirical high-lux correction, cover/window
  compensation, spectral calibration, metrology certification, or physical
  interrupt output is claimed.
- Mock evidence is void for physical capabilities.

## Current software validation

- `python3 tools/validate-pack.py` and publication-policy regression tests passed.
- Every TOML and JSON file parsed; workflow YAML is parsed when PyYAML is present.
- Workspace, runtime dependency, no-std/unsafe, facade-state, fixed-address,
  byte-order, API-contract, monitor-domain, and rejected-feature guards passed.
- Capability inventory, contracts, plan, mock manifest, transcript commands, and
  case inventories aligned at 12 capabilities.
- Forty-six host tests cover codecs, exact transactions, staged failures, state
  restoration, and autonomous behavior.
- Formatting, no-default and all-feature compilation, Clippy with warnings
  denied, rustdoc/doctests, all five configured embedded targets, cargo-deny,
  and package listing passed with Rust 1.92.0.
- The deterministic mock harness build passed. Every retained core I²C capture
  has a policy decoder request.
- Core and calibrated-optical metadata gates were checked independently.
- Relative Markdown links and per-file SHA-256 inventory passed.

External `ph-hil` replay and policy analysis were not run because that CLI is
not installed in this environment. Mock firmware build success remains void for
physical capabilities.

## Owner verification still required

- pin and hash the official Vishay datasheet and application note;
- review every hardware-contract row and the vendor register-count discrepancy;
- implement and review the real managed ESP32 harness;
- characterize the optical fixture before enabling absolute optical claims;
- keep capability status Planned/Mock-integrated until sealed physical evidence
  and reviewed policy assessment justify promotion.
