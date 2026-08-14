# `ph-curves` 0.2.1 gaps — VEML7700 transfer smoke test

> **Authority: non-normative exploration.** Feature proposal *for
> `ph-curves`*, hosted here because the smoke-test payload is this device.
> Generic names only: `ph-curves` must not grow a `kind = "veml7700"` model.
> See [`README.md`](README.md).

The smoke test is: parse
[`crates/veml7700/src/transfer.toml`](../../crates/veml7700/src/transfer.toml)
and emit sparse integer `PiecewiseLinearTransfer` tables that preserve every
row below. 0.2.1 cannot do that without dropping facts. A `formula` dump of
the quartic would compile, look complete, and silently omit half the payload
— the same invented-agreement failure this repository closed in #56 / #28.

---

## 1. Discrete transfer family

**Why the smoke test needs it.** The part has twenty-four gain × integration
resolution pairs ([`NominalScale`](../../crates/veml7700/src/illuminance.rs);
hardware contract §8). Default emission is the twelve **correction-required**
members (gain ×1/4 and ×1/8). Scale is per member; coefficients are not.

**Why 0.2.1 fails.** [`TransferDef`](https://github.com/photon-circus/ph-curves/blob/main/src/gen/transfer/mod.rs)
is one TOML table → one transfer. There is no family, no selector, and no
rule against interpolating between members.

**Generic feature.** `[transfer_families.<name>]` with an explicit `members`
array. `interpolate_selectors = false` is required; `true` is an error.
Selectors are opaque strings/integers, never interpolated, never ordered by
a bitfield.

**Bound.** Default emit only members the description marks for generation
(here: `correction = "required"`). Do not expand 24 full-domain tables.

**Acceptance.** The VEML7700 fixture expands to 12 named transfers, one per
×1/4 and ×1/8 × {25,50,100,200,400,800} ms, each carrying that pair’s
`scale_micro_lux_per_count`. ×1 and ×2 are in the driver description as
do-not-use-above-100 lx and are **not** generated.

---

## 2. Scaled polynomial model

**Why the smoke test needs it.**
`lux_uncorr = counts × resolution`;
`lux_corr = a·u⁴ + b·u³ + c·u² + d·u` with the vendor coefficients, `u` in
uncorrected lux. One coefficient vector; twenty-four scales.

**Why 0.2.1 fails.** [`ModelDef`](https://github.com/photon-circus/ph-curves/blob/main/src/gen/transfer/model.rs)
is `NtcBetaDivider` only. `formula` can encode one member by inlining
`scale` and the quartic as a string, which duplicates coefficients 12–24
times and has no identity as “the vendor polynomial”.

**Generic feature.** `kind = "scaled_polynomial"`:
`y = c0 + c1·u + c2·u² + …` with `u = (scale_micro_lux_per_count / 1e6) · x`.
Coefficients live on the family model; `scale_micro_lux_per_count` lives on
the member.

**Bound.** Host evaluation only. Generated source remains integer knots.
Empty or non-finite coefficients are errors.

**Acceptance.** 5581 counts at ×1/4 and 100 ms (`scale_micro_lux_per_count =
268800`) produces 1500 lx uncorrected and 1658 lx corrected within a stated
tolerance (hardware contract §8 worked example).

---

## 3. Semantic selectors, not bit-magnitude order

**Why the smoke test needs it.** Gain encoding `10` is ×1/8 and `11` is ×1/4;
integration `1100` is the *shortest* time. A table sorted by bit value is
silently transposed.

**Why 0.2.1 fails.** Generated const names are sorted by TOML key. There are
no selectors, so nothing prevents a caller from naming members in encoding
order.

**Generic feature.** Members are keyed by declared selector names
(`gain = "div8"`, `integration_time_ms = 25`), not by a numeric encoding.
The generator must not sort members by an integer encoding it does not own.

**Bound.** None beyond “do not invent an encoding order”.

**Acceptance.** The ×1/8 25 ms member’s scale is 2 150 400 µlx/count, twice
×1/4 25 ms (1 075 200), matching §8 rather than swapped gain bits.

---

## 4. Applicability distinct from domain clamp

**Why the smoke test needs it.** Sources constrain *where the mapping means
anything*: ×1/×2 only below 100 lx; ×1/4 and ×1/8 should use the formula;
above 1000 lx the formula is required; linear behaviour ~0.0042–1 klx;
corrected results “around 100 klux”. `below` / `above` are observation-domain
clamp/error, not those statements.

**Why 0.2.1 fails.** [`BoundaryDef`](https://github.com/photon-circus/ph-curves/blob/main/src/gen/transfer/mod.rs)
is `Error | Clamp` on the fitted domain only.

**Generic feature.** Per-member `applicability` (correction required / none /
do-not-use, uncorrected-lux window). Generation domain is the intersection of
that window with representable counts. Firmware `below`/`above` remain
endpoint behaviour of the *fitted* domain.

**Bound.** Uncorrected full scale at ×1/8 25 ms (140 926 lx) is outside the
applicability window used for generation.

**Acceptance.** A member with `correction = "required"` and
`uncorrected_lux = [100, 22000]` (≈100 klx corrected) generates. The same
polynomial asked to cover 0..=65535 counts at ×1/8 25 ms is rejected.

---

## 5. Extrapolation is an error

**Why the smoke test needs it.** The quartic is claimed “around 100 klux” on
the corrected axis. At 140 926 lx uncorrected it evaluates to ~2×10⁸ lx and
overflows `i32` milli-lux. Silently fitting that range produces agreement
with a nonsense curve.

**Why 0.2.1 fails.** Transfers never extrapolate *past the fitted knots*, but
the fitter will happily consume whatever domain it is given, up to 4096 knots
over tens of thousands of codes, until `scale_truth` overflows `i32`.

**Generic feature.** Family `extrapolation = "error"` (default): refuse a
domain outside applicability. An explicit opt-in is required to widen it.
`scale_truth` overflow remains an error (keep that).

**Bound.** Corrected output must fit `i32` quanta at `output_scale`. Milli-lux
(`output_scale = 1000`) fits ~100 klx; micro-lux does not.

**Acceptance.** Fixture uses milli-lux. Full-scale ×1/8 25 ms generation
fails closed, not with a 64 Ki-entry table.

---

## 6. Saturation ≠ domain-end clamp

**Why the smoke test needs it.** `u16::MAX` ALS counts is a clipped
conversion, not a lux observation. Clamping the last fitted knot would report
a number for a non-measurement.

**Why 0.2.1 fails.** `above = "error"` and `above = "clamp"` both describe
“past `domain_max`”. They cannot say “`u16::MAX` is saturation even if it
lies inside a naive 0..=65535 domain”.

**Generic feature.** Family `saturation = "error"`: never include `u16::MAX`
in a generated domain. Distinct from `above` on the fitted (applicability)
endpoint.

**Bound.** `domain_max ≤ 65534` for 16-bit observations.

**Acceptance.** No generated member’s `domain_max` is 65535.

---

## 7. Declared gaps

**Why the smoke test needs it.** The white channel is 16-bit counts with no
lux scale. The application note’s white/ALS ≳ 2 IR heuristic has **no**
conversion formula. Omitting those keys looks like a forgotten field;
inventing a white lux scale or an IR correction would be a D-030 violation.

**Why 0.2.1 fails.** No gap object. Absence is indistinguishable from
forgetfulness.

**Generic feature.** `[gaps.<name>]` with `status = "undefined"` and a
`reason`. Gaps are not generated as transfers. Unknown `status` is an error.

**Acceptance.** The fixture declares `white_channel` and
`ir_lux_optimization`. Generator emits no `WHITE` transfer and no IR
compensation table.

---

## 8. Unknown fields are errors

**Why the smoke test needs it.** Applicability, saturation, and gaps are
load-bearing. If the generator ignores them, 0.2.1 will parse the family TOML
as an empty document and succeed.

**Why 0.2.1 fails.** [`DefinitionsFile`](https://github.com/photon-circus/ph-curves/blob/main/src/gen/curve.rs)
and `TransferDef` do not use `deny_unknown_fields`. Extra keys are dropped.

**Generic feature.** `#[serde(deny_unknown_fields)]` on the definitions file,
family, member, gap, and model. A document that only 0.2.1 would accept by
ignoring `[transfer_families]` / `[gaps]` must fail.

**Bound.** None.

**Acceptance.** Parsing the VEML7700 fixture on **unpatched** 0.2.1 via
`generate_from_str` must not be treated as success in this crate. The probe
calls `transfer_families()` / `gaps()`, which do not exist, so the feature
does not compile. After the patch, parse fails unless those tables are
honoured.

---

## 9. Bounded sparse knots — never `CurveLut`

**Why the smoke test needs it.** ALS is a 16-bit observation. A curve-style
`u16` LUT is 65 536 entries per member. Twelve members would be 768 Ki
forward entries before inverses. That is the failure mode this probe exists
to catch.

**Why 0.2.1 fails.** [`ValueType::U16`](https://github.com/photon-circus/ph-curves/blob/main/src/gen/api.rs)
**requires** `lut_size = 65536`. Transfer knot cap is 4096. Nothing stops a
caller from putting this device on the curve path, or from setting
`max_knots = 4096` over the full ADC domain.

**Generic feature.** A non-empty `transfer_families` table **forbids**
`[curves.*]` in the same document (`forbid_dense_lut`). Family
`max_knots` default 64, hard cap 256. Exceeding the cap is an error (keep
today’s no-full-domain-fallback behaviour). Host may sample the truth on the
bounded count window; firmware sees only the knots.

**Bound.** 12 members × 64 knots × 6 bytes ≈ 4.5 KiB array payload upper
bound for the default VEML7700 emission, not 12 × 64 Ki.

**Acceptance.** Generated source contains `PiecewiseLinearTransfer`, no
`CurveLut`, no `f32`/`f64`, and each member `knot_count ≤ 64`. NTC
precedent: 61 knots / 366 bytes.

---

## 10. Worked example as an oracle (crate or generator)

**Why the smoke test needs it.** 5581 counts, ×1/4, 100 ms → 1500 lx
uncorrected → 1658 lx corrected is the only source-worked point for the
polynomial.

**Why 0.2.1 fails.** No verification-example field. Nothing requires a
generated table to reproduce that point.

**Generic feature.** Optional; this crate’s unit tests own the integer
uncorrected half and a host check of the corrected half. A future generator
may accept `[[example]]` rows. Not required to land the patch if the driver
crate asserts it.

**Bound.** Tolerance stated next to the test, not “looks close”.

**Acceptance.** Driver tests: `5581 * 268800` µlx floors to 1500 lx;
corrected value rounds to 1658 lx.

---

## Out of scope for `ph-curves` (remain this device’s §11)

Cover-glass transmission, source spectrum, cosine response, auto-ranging,
calibrated system lux, and any white-channel lux scale. `AffineCalibration`
is how a consumer adapts a generated table; it is not this description.
