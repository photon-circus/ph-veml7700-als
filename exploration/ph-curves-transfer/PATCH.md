# Suggested patch against `ph-curves` 0.2.1

> **Not applied in this repository.** Apply it on
> [`photon-circus/ph-curves`](https://github.com/photon-circus/ph-curves) at the
> 0.2.1 tree (`main` when this probe was written). Firmware APIs are unchanged.

## Apply

```sh
git clone https://github.com/photon-circus/ph-curves.git
cd ph-curves
git checkout 0.2.1   # or the 0.2.1 commit on main
git apply /path/to/ph-veml7700-als/exploration/ph-curves-transfer/ph-curves-0.2.1.patch
cargo test --features gen-lib --lib
```

Verified in this probe: `342 passed` on `cargo test --features gen-lib --lib`.

## What it adds

| Piece | Where |
| --- | --- |
| `ModelDef::ScaledPolynomial` | `src/gen/transfer/model.rs` |
| `[transfer_families]` expansion, `interpolate_selectors = false`, knot cap 256 | `src/gen/transfer/family.rs` |
| `[gaps]` with `status = "undefined"` | same |
| `DefinitionsFile::transfer_families` / `gaps` / `resolved_transfers` | `src/gen/curve.rs` |
| Families forbid `[curves]` in the same document | `src/gen/codegen.rs` |
| `deny_unknown_fields` on the definitions file | `src/gen/curve.rs` |
| Saturation code `u16::MAX` rejected for scaled polynomials | `evaluate_scaled_polynomial` |
| Worked-example + full-scale overflow tests | `src/gen/transfer/mod.rs` |

Generated firmware remains `PiecewiseLinearTransfer` with a small knot count. No `CurveLut`, no `f32`/`f64` in emitted source.

## What it does not do

- No `kind = "veml7700"` (device-specific models stay out of `ph-curves`)
- No runtime firmware API growth
- No dense `u16` LUT path for transfers
- Does not publish a new `ph-curves` version — that is a `ph-curves` maintainer decision
