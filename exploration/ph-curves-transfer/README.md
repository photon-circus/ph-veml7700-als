# Exploration: VEML7700 transfer description as a `ph-curves` smoke test

> **Authority: non-normative exploration.** This tree is not a contract, not a
> shipped feature, and not listed in `AGENTS.md`. Do not cite it as evidence
> that the driver applies high-lux correction, that `ph-curves` 0.2.1 can
> represent this device, or that a compiling `ph-curves` Cargo feature exists.

Issue #46 asks this crate to export a source-backed ALS transfer description
for `ph-curves-gen`. [`ph-curves` 0.2.1](https://github.com/photon-circus/ph-curves)
cannot represent that description without dropping facts. This branch writes
the feature **against the API that would make the smoke test succeed**, and
records what 0.2.1 is missing.

## Status

| | |
| --- | --- |
| Target `ph-curves` | 0.2.1 (crates.io / `main` at the time of this probe) |
| This crate, default features | expected to build and test as before |
| This crate, `--features ph-curves` / `--all-features` | **expected not to compile** |
| `CI_PROFILE=full` | **not** the acceptance bar on this branch |
| Merge to `main` as a product feature | **no** — promote only after a published `ph-curves` accepts the schema in `REQUIREMENTS.md` |

`--all-features` going red is the smoke test firing: the export calls
`DefinitionsFile::transfer_families` and `DefinitionsFile::gaps`, which 0.2.1
does not provide. A 24-entry `formula` dump that 0.2.1 would accept is
rejected here because it would look complete and silently drop applicability,
gaps, encodings, and saturation.

## Contents

| File | What it is |
| --- | --- |
| [`REQUIREMENTS.md`](REQUIREMENTS.md) | Scoped feature proposal *for `ph-curves`*: each 0.2.1 failure, the generic capability that closes it, and the VEML7700 assertion |
| [`PATCH.md`](PATCH.md) | How to apply [`ph-curves-0.2.1.patch`](ph-curves-0.2.1.patch); what it changes and what it does not |
| [`CONTRACT_DRAFT.md`](CONTRACT_DRAFT.md) | Proposed D-007 / D-031 / §11 wording **if** the feature later lands. Normative files in this repository are unchanged |

The driver-side description and the family TOML live in
`crates/veml7700/src/transfer.rs` and `crates/veml7700/src/transfer.toml`.
They are the smoke-test payload, not a generated LUT.

## Hard bound: no dense LUT

`ph-curves` has two generators. This probe uses **only** sparse transfers.

- `[curves.*]` full-domain LUTs (`u8` × 256 or `u16` × 65 536) are forbidden.
- Family members use `PiecewiseLinearTransfer` knots: default `max_knots = 64`,
  hard cap 256. Exceeding the cap is an error, not a denser table.
- The polynomial is **not** evaluated over `0..=65535` counts at gain ×1/8 and
  25 ms. That uncorrected full scale is 140 926 lx; the quartic explodes and
  overflows `i32` milli-lux. Generation domain is the applicability window
  (uncorrected lux above ~100 lx, corrected results around 100 klx), not the
  ADC width.
