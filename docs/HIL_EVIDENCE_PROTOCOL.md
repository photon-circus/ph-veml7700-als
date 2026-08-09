# HIL evidence protocol

The VEML7700 repository is a downstream consumer of `ph-hil`. It has no runtime
or build dependency on `ph-hil` Rust crates or a source checkout. Integration
uses public schema-1 plans, capability contracts, firmware manifests, dutlink-v1
records, sealed run artifacts, and offline Lua assessment.

## Ownership

`ph-hil` owns build/flash orchestration, DUT transport, cancellation, reset,
instrument acquisition, prompts, event streams, fixture safing, panic-stop,
immutable sealing, and append-only analysis.

This project owns the case inventory, minimal harness command handlers, evidence
records, deterministic mock transcript, project modules, fixture contract, and
policy mapping evidence to confidence/capabilities.

Firmware and operators report observations; they do not assign confidence.

## Evidence classes

- device/runner evidence can directly support digital API behavior;
- logic evidence can directly support transfer order and timing only when
  instrument identity and acquisition metadata are retained;
- relative optical evidence is qualified unless source repeatability and geometry
  are witnessed;
- absolute optical evidence requires a calibrated reference and characterized
  fixture;
- mock transcript evidence is always void for physical claims;
- fail, skip, error, or void cannot support a capability.

## Project assets

- `hil/plan.toml` — non-calibrated core matrix;
- `hil/plan-optical.template.toml` — separately enabled calibrated plan;
- `hil/contracts/veml7700.toml` — exact harness capability contract;
- `hil/modules/` — project Lua validators/capture modules;
- `hil/bench.mock.toml` — deterministic mock bench;
- `hil/bench.real.template.toml` — explicit physical selectors/facts;
- `hil/mock/transcript.json` — retained deterministic replies;
- `hil/policy.lua` and `hil/policy-optical.lua` — offline assessment;
- `ph-hil-shim*.toml` — structured firmware build hooks.

## Confidence rules

The core policy downgrades optical cases without retained source/fixture facts,
logic cases without instrument identity, and any mock case. The optical policy
requires reference identity/calibration, geometry, source stability, dark
baseline, and paired reference readings before a capability can be Direct.

## Commands

```console
ph-hil shim-build ph-hil-shim.mock.toml
ph-hil run hil/plan.toml --bench hil/bench.mock.toml --headless --json
ph-hil analyze project <run> --policy hil/policy.lua
```

These validate the external integration boundary, not physical hardware.
