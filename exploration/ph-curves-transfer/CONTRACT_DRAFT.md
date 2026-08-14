# Contract drafts — not in force

> **Authority: non-normative exploration.** Proposed wording **if** issue #46
> later lands as a compiling feature. The files named below are **unchanged**
> on this branch. Applying these diffs before `ph-curves` can consume the
> description would claim a capability the crate does not have.

## D-007 — split sensor non-linearity from system calibration

Current:

> The crate provides integer micro-lux using the vendor table. Empirical and
> system calibration remain outside the driver.

Proposed:

> The crate provides integer micro-lux using the vendor resolution table.
> That value is *nominal*: it does not apply the vendor high-lux polynomial
> and is not calibrated at a product aperture.
>
> **Sensor non-linearity** (the vendor polynomial, parameterised by gain and
> integration time) is a device fact recorded in `HARDWARE_CONTRACT.md` §8.
> When a compiling `ph-curves` feature exists, the crate **exports** that
> fact as a transfer description for host-side generation. It does not
> evaluate the polynomial at runtime and gains no floating point.
>
> **System calibration** — cover glass, source spectrum, cosine response,
> geometry — remains outside the driver. A consumer who needs it applies
> `ph-curves` `AffineCalibration` (or their own layer) to generated tables.
> Exporting the vendor model is not performing or validating correction.

## D-031 — transfer description is a codegen contract (new)

Proposed:

> The optional `ph-curves` feature exports a description, not a lux
> pipeline. Enabling it does not add `corrected_illuminance()`. Knot density,
> generation range, and any later calibration are the consumer’s.
>
> Generation domain is the source-backed applicability window, not the ADC
> width. Undeclared coefficients, white-channel lux, and the application
> note’s IR “optimize conversion” step are **gaps**: they are declared
> undefined rather than filled. A description that no longer matches §8’s
> resolution and range tables is a defect.
>
> Dense `CurveLut` emission (`u8` × 256 or `u16` × 65 536) is forbidden for
> this description. Firmware tables are sparse `PiecewiseLinearTransfer`
> knots with a hard cap.

## Hardware contract §8 / §11

§8 currently: “This driver does not apply it” (the polynomial), and names
`ph-curves` as the intended home without committing this repository.

Leave the polynomial coefficients as a device fact. When the feature lands,
replace “does not apply it” with: the driver **exports** the polynomial as a
transfer description and **does not evaluate or validate** it. Low-gain
presets still sit where the source says uncorrected `nominal_illuminance` is
not a lux estimate.

§11 currently lists “empirical high-lux correction” as a non-claim.
Proposed split:

- still a non-claim: calibrated or empirically fitted system correction;
  silicon validation of the vendor polynomial;
- not a non-claim of the *export*: the crate may ship a description of the
  vendor model. “Corrected” then means that model applied by generated
  integer tables, never calibrated aperture lux.

Overclaiming calibration and disclaiming a shipped export are both
truthfulness failures — only after the export actually ships.

## Invariants and rejected shortcuts

I-11 stays: nominal lux never claims calibration; white counts are not
converted using ALS scaling.

Add, when landing: the exported description matches §8’s twenty-four
resolution/range pairs and contains no invented white lux scale.

Rejected shortcut “global high-lux polynomial | application-dependent
correction”: keep the **runtime** evaluation rejection. Exporting the vendor
polynomial for host generation is the allowed exception, not a runtime
quartic on `thumbv6m-none-eabi`.

## `CONTRIBUTING.md` / `AGENTS.md`

“Automatic optical correction” remains out of scope. Runtime correction and
system calibration stay out. Description export is the D-007 exception, and
only once it compiles against a published `ph-curves`.
