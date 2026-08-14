# Decision log

## D-001 — Direct fixed-address I²C facade

`Veml7700<I2C>` owns the bus directly. The supported device has one fixed
address and no transport variation requiring an internal abstraction.

## D-002 — Preserve concrete bus errors

Errors retain the HAL error plus semantic operation/register/stage context.
Only address NACK is classified as absence during probe.

`probe` reports through `ProbeError` instead, because address NACK means absence
only there. It therefore carries no `Operation`, so `Operation` has no `Probe`
variant: a variant no driver path can construct is surface a caller could match
on and never reach.

## D-003 — Strict low-byte-first codec

All 16-bit register transfers use little-endian wire order, protected by exact
transaction tests.

## D-004 — Bright-start is explicit policy

`MeasurementConfig::safe_bright_start()` is a value constructor, not hidden
driver state or automatic ranging.

## D-005 — Snapshot and fresh results are distinct

A snapshot may be retained or straddle channel refresh. Fresh capture controls
the domain and timing, freezes results, records provenance, and restores state.

## D-006 — Shutdown freezes the result pair

After conservative waiting, complete capture enters shutdown before sequential
ALS/white reads. This is a driver coherence policy, not a vendor atomicity claim.

## D-007 — Nominal integer scaling only

The crate provides integer micro-lux using the vendor table. Empirical and
system calibration remain outside the driver.

## D-008 — Threshold state is polled

The device has no dedicated interrupt pin. APIs use monitor/status language and
never own GPIO.

## D-009 — Monitor owns its complete domain

Gain, integration, thresholds, persistence, cadence, and active state are one
semantic domain protected against silent retargeting.

## D-010 — No undocumented clearing promise

The official sources do not establish reliable threshold-flag clearing
semantics, so the API promises observation only.

## D-011 — No VEML6030 abstraction

Family extraction waits for independently reviewed contracts for more than one
device.

## D-012 — Speculative physical infrastructure removed

Hardware runners, fixtures, plans, policies, transcripts, evidence structures,
and orchestration shims are not part of this driver product. Future physical
qualification begins from accepted driver and independent-model contracts.

## D-013 — Timing is bound to integration selection

Explicit timing may extend but never shorten the conservative wait and is
rejected before I²C if derived for a different integration time.

## D-014 — Fresh capture creates a known wake edge

Complete measurement installs the selected domain in shutdown before changing
to active and starting its wait.

## D-015 — Model independence is required

The independent cross-validation model implements I²C from the hardware contract
and must not reuse driver codecs, timing helpers, semantic types, or state
machines as its oracle. Autonomous cadence and threshold behavior belong there,
not in a driver-coupled fake. Injected ALS/white phase skew is test scheduling
topology rather than a silicon timing claim. The bounded slice and its nonclaims
are declared in
[`crates/veml7700-model/README.md`](../crates/veml7700-model/README.md).

## D-016 — Vendor documents are not redistributed

Track official URLs, revisions, retrieval facts, and available hashes without
committing vendor PDFs. The untracked claim is enforced by the canonical gate
rather than by `.gitignore` alone.

## D-017 — Local bounded validation

There is one canonical gate, `scripts/ci.sh`, and it has three profiles. `full`
is authoritative for ordinary work and is what a maintainer runs. `bounded` is
the subset hosted CI runs; it drops the checks needing an extra binary or
substantial runner time and reports each as an explicit skip, so a green hosted
run can never be mistaken for a green release gate. `release` is `full` plus
artifact identity: it refuses a dirty worktree, packages without
`--allow-dirty`, and records the source-to-archive relationship as evidence.

`release` is a superset rather than a replacement. `full` deliberately keeps
packaging permissive so a work-in-progress tree stays checkable, which is the
common case; demanding a clean worktree for every routine run would make the
gate hostile to ordinary development. The two therefore differ only in what they
demand of the tree, not in what they verify about the code.

Only the PowerShell launcher is retained beside the script, because locating Git
Bash on Windows is real work and a POSIX passthrough wrapper is not.

The hosted workflow is prepared before the repository is public, but is
dispatch-only until then; its automatic triggers belong to the visibility
change. A dispatched failure is noted and investigated, but the full local run
is the only authority. Generated pack inventories remain rejected.

The gate also tests the unpacked
distributable package, so packaging-only failures such as a stripped path
dependency cannot pass verification. Packaging is pinned to the repository
target directory because Cargo excludes only that path from workspace member
discovery; a configured target directory elsewhere in the repository would make
the extracted package untestable.

## D-018 — Release decisions remain explicit and independent

The driver uses the lifecycle-matching candidate version
`0.1.0-incubating.1` while retaining `publish = false`. Repository preparation,
visibility, and crates.io publication are separate maintainer-controlled
decisions. Model completeness, physical evidence, hardware qualification, and
`ph-hil` adoption limit their corresponding claims but do not gate an honestly
disclosed Incubating publication.

## D-019 — Model input limits are not device behavior

The model accepts `u8` addresses but declares a 7-bit I²C boundary. Values above
`0x7F` cannot exist on that bus, so they are reported as
`Unsupported::AddressOutOfRange` model limitations rather than fabricated device
NACKs. Other valid 7-bit addresses remain source-backed address NACKs. The model
never invents device behavior for inputs outside its declared domain.

## D-020 — Validating constructors own their fields

A public type whose constructor enforces a rule keeps its fields private, so the
rule cannot be bypassed by a struct literal.

`Thresholds::new` rejects `low > high` and returns `Option`. While its fields
were public, downstream code could build a reversed pair directly and pass it to
`arm_threshold_monitor`, which would program it. That produced an asymmetry the
driver should never have: it would write device state that `read_thresholds`
rejects as `ConfigurationError::ReversedThresholds` when read back. The fields
are now private with `low()` and `high()` accessors.

`ThresholdMonitorConfig` keeps public fields because it enforces no rule of its
own; it is a bundle whose only invariant now lives inside `Thresholds`.

## D-021 — No documentation index

`docs/README.md` was deleted because it restated the AGENTS.md document table
and went stale; the root README links the contracts directly.

## D-022 — One version across the workspace

Both crates carry the same lifecycle-matching prerelease, currently
`0.1.0-incubating.1`, and the canonical gate fails if they diverge.

The Peripheral Driver Profile attaches its prerelease rule to the driver
package, so `ph-veml7700-als-model` could have kept an ordinary `0.1.0`. Aligning
was chosen instead because the two crates share one repository, one release
boundary, and one review cycle: a reader comparing them should not have to work
out which convention applies to which, and a second convention is a place for
drift to hide.

The gate no longer repeats the exact version. It reads the driver manifest,
requires the model manifest to match, and asserts only that the version carries
an `-incubating.N` prerelease identifier. A version bump therefore edits the
manifests, and the gate keeps verifying the declared distribution state without
holding another copy of the literal.

## D-023 — Undeclared reset values stay unavailable

The hardware-contract register map records which words have a source-declared
reset and which do not. Configuration (`0x00`) resets to `0x0001` and
power-saving (`0x03`) to `0x0000`. Thresholds, ALS, white, and threshold status
are **not declared by sources**. The ID word is a source-declared identity,
not a POR field.

Determinism does not authorize a convenient initial value for that undeclared
observable state. The independent model leaves those registers unavailable
until an explicit input establishes them: a threshold write, a completed
conversion, or a completed ALS refresh while the monitor is enabled. Inventing
`0x0000` for threshold status would make a fresh `read_threshold_status()`
decode to both flags clear with no source backing.

One residue is unavoidable and is therefore declared rather than hidden. Because
this model asserts threshold flags and never clears them, a refresh that
qualifies nothing cannot prove a flag is clear, so the establishing refresh
cannot derive the whole `0x06` word from transitions alone. The model declares
that construction represents a device with no prior qualification; the
observable consequence is that an unqualified flag reads clear after the first
monitored refresh. The alternative — tracking each flag's knowledge separately —
would leave `0x06` unavailable whenever a flag never qualifies, which is the
ordinary case, and would make the driver's polled-status path untestable for no
gain in evidence.

## D-024 — Error taxonomies are open, device domains are closed

Every public error type in both crates is `#[non_exhaustive]`. Every public
type that enumerates a device domain is deliberately not.

Error taxonomies grow as the driver and model grow. `Unsupported` alone gained
nine variants across five commits before the first release, and `Operation`,
`BusContext`, `MeasureStage` and `ThresholdMonitorStage` extend with each new
operation, register and stage. Without `#[non_exhaustive]` each of those
additions breaks a downstream exhaustive `match`, so ordinary additive work
would force a breaking release. The attribute is free before publication and
cannot be added afterwards without causing the break it prevents.

The device value types — `Gain`, `IntegrationTime`, `Persistence`,
`PowerState`, `ThresholdMonitorState`, `PowerSavingMode`, and
`MeasurementPairCoherence` — stay exhaustive. They enumerate a fixed domain
taken from the datasheet, not an open taxonomy. A caller that handles every
gain should keep getting a compile error when it misses one, and the device will
not grow a fifth gain. If a future part does, that is a new contract and a
deliberate breaking change rather than routine growth.

The three decode errors are open with the errors, not closed with the domains
they describe. `ConfigDecodeError`, `PowerSavingDecodeError` and
`ThresholdStatusDecodeError` currently track reserved-bit rules that are as
fixed as the register map, but they are error types: a caller matches them to
report, not to make a decision per variant, so exhaustiveness buys the caller
little and costs a break if the contract ever distinguishes a new decode
failure.

`ThresholdMonitorError` is a struct rather than an enum and carries the
attribute for the same reason: it is produced by the driver and read by the
caller, so preventing downstream literal construction costs nothing and lets it
gain a field later. The obligation it places on a caller is different from the
enums, though, and the API contract says so: a struct pattern must contain `..`,
and a wildcard match arm does not satisfy that. Ordinary field access is
unaffected, which is how callers actually read this type.

Result and snapshot types with public fields, such as `FreshMeasurement` and
`ConfigurationSnapshot`, are deliberately left alone here. Whether a caller
should be able to build one is a separate question from error growth, and
answering it under this decision would be scope creep.

## D-025 — No CODEOWNERS while there is one maintainer

**Date:** 2026-08-14 **Status:** Current

The organization standard requires `CODEOWNERS` only where there are multiple
maintainers or separated sensitive ownership, and asks a solo-maintained
repository to record whether it adds value. It does not, yet.

`CODEOWNERS` expresses that a particular person must review a particular path.
With one maintainer every path resolves to the same person, who is also the
author of nearly every change. The file would not add a second reviewer; it
would auto-request review from the author, and advertise a separation of
ownership that does not exist. Both are worse than saying plainly that this is a
single-maintainer project.

The boundaries a `CODEOWNERS` file would have protected — the source contracts,
the release manifests and workflow, and the security policy — are protected
instead by mechanisms that work with one maintainer: the canonical gate, the
required aggregate `ci` check once `main` is protected, and the rule that a
contract change is reviewed as a behavior change.

Add `CODEOWNERS` when a second maintainer joins, covering at minimum
`docs/HARDWARE_CONTRACT.md`, `docs/vendor/`, `scripts/ci.sh`,
`.github/workflows/`, the crate manifests, and `SECURITY.md`. That is the point
at which it starts routing review rather than describing a single person.

## D-026 — Reconfiguration is shutdown-first

**Date:** 2026-08-14 **Status:** Current

Every mutating operation writes the shutdown bit before changing any other
field, and returns the device to active last. `set_power_state` is not a
reconfiguration and is unaffected.

This is not a defensive choice. The vendor's own software flow sets `ALS_SD = 1`
before any reconfiguration, changes gain or integration time while shut down,
and clears `ALS_SD` afterwards. `docs/HARDWARE_CONTRACT.md` §5 records it as an
owner-verified row. It is a positive requirement, not an absence of permission
to write while active, and that distinction is what settled the question.

The driver and the independent model disagreed here: the driver accepted
monitor-disabled active starting states, and the model rejected changed or
repeated active configuration. **The disagreement was resolved against the
driver.** The model was correct and was not relaxed to admit the sequence it had
been rejecting — which is the outcome an independent oracle exists to produce.
Relaxing it would have destroyed the only evidence that the two derivations
disagree at all.

Three costs are accepted deliberately:

1. **More transactions.** `set_measurement_config` and `set_power_saving` take
   three writes instead of one when the device is active. A single write that
   moved both the shutdown bit and another field would be cheaper and is exactly
   what the sources forbid.
2. **A different failure state.** Because shutdown comes first, a failure part
   way through can leave an originally active device shut down. The contract
   states the read-back obligation rather than pretending the operation is
   atomic; the transport cannot make it so.
3. **A new public stage.** `MeasureStage::EnterShutdown` and
   `ThresholdMonitorStage::EnterShutdown` name the pre-reconfiguration write, so
   a failure there is attributable instead of folded into a later stage. Both
   enums are `#[non_exhaustive]`, so this is additive. See D-024.

Entering shutdown first also fixed recovery, which was not the goal but is the
stronger argument. Previously an active start failed *and* its restoration
failed, because restoration wrote the power-saving register to a still-active
device. Every stage after the first now runs on a shut-down device. The one
remaining case — a failure of the shutdown write itself — reports without
attempting restoration, because nothing has been mutated and the generic
sequence would commit the very active write this decision removes.

Two writes remain legal while active because both are transitions rather than
reconfigurations: setting the shutdown bit alone, and disabling an enabled
monitor alone. They cannot be combined. Re-arming an enabled monitor on an
active device therefore shuts down first with the monitored domain intact, then
disables the monitor while shut down.
